use axum::{Json, extract::State};
use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::{AppError, ErrorCode};
use crate::models::{
    AuthResponse, GoogleAuthRequest, GoogleTokenClaims, GroupWithBalance, JwtClaims, PublicUser,
    User,
};

// ── Auth ──────────────────────────────────────────────

#[tracing::instrument(skip(pool, body))]
pub async fn google_login(
    State(pool): State<PgPool>,
    Json(body): Json<GoogleAuthRequest>,
) -> Result<Json<Value>, AppError> {
    let client_id = crate::env::ENV.google_client_id.clone()?;

    tracing::debug!("Calling Google tokeninfo...");

    let token = body.credential;
    let url = format!("https://oauth2.googleapis.com/tokeninfo?id_token={token}");
    let resp = reqwest::get(&url).await.map_err(|e| {
        tracing::error!("Failed to reach Google tokeninfo: {e}");
        AppError::Unauthorized(ErrorCode::AuthGoogleFailed, None)
    })?;

    let status = resp.status();
    let raw_body = resp.text().await.map_err(|e| {
        tracing::error!("Failed to read Google response body: {e}");
        AppError::Unauthorized(ErrorCode::AuthGoogleFailed, None)
    })?;

    tracing::debug!(%status, %raw_body, "Google tokeninfo response");

    let google_claims: GoogleTokenClaims = serde_json::from_str(&raw_body).map_err(|e| {
        tracing::error!(%raw_body, "Failed to parse Google tokeninfo: {e}");
        AppError::Unauthorized(ErrorCode::AuthGoogleFailed, None)
    })?;

    if let Some(aud) = &google_claims.aud {
        if aud != &client_id {
            tracing::error!(expected=%client_id, got=%aud, "Token audience mismatch");
            return Err(AppError::Unauthorized(ErrorCode::AuthGoogleInvalid, None));
        }
    }
    if google_claims.sub.is_empty() {
        tracing::error!("Google token has empty sub claim");
        return Err(AppError::Unauthorized(ErrorCode::AuthGoogleInvalid, None));
    }

    let email = google_claims
        .email
        .filter(|_| google_claims.email_verified.as_deref() == Some("true"))
        .ok_or_else(|| {
            tracing::error!("Google email not verified or missing");
            AppError::Unauthorized(ErrorCode::AuthGoogleInvalid, None)
        })?;

    tracing::info!(%email, google_sub=%google_claims.sub, "Google user verified");

    // Beta allowlist check
    let is_allowed: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM beta_allowlist WHERE email = $1)")
            .bind(&email)
            .fetch_one(&pool)
            .await
            .map_err(|e| {
                tracing::error!("Failed to check beta allowlist: {e}");
                AppError::Internal(format!("Database error: {e}"))
            })?;

    if !is_allowed {
        tracing::warn!(%email, "User not on beta allowlist");
        return Err(AppError::Forbidden(ErrorCode::AuthNotOnAllowlist, None));
    }

    // Upsert user
    let user: User = sqlx::query_as(
        r#"INSERT INTO users (id, username, email, google_id, avatar_url)
           VALUES (gen_random_uuid(), $1, $2, $3, $4)
           ON CONFLICT (google_id) DO UPDATE SET
               username   = COALESCE(EXCLUDED.username, users.username),
               email      = EXCLUDED.email,
               avatar_url = COALESCE(EXCLUDED.avatar_url, users.avatar_url)
           RETURNING *"#,
    )
    .bind(&google_claims.name)
    .bind(&email)
    .bind(&google_claims.sub)
    .bind(&google_claims.picture)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error upserting user: {e}");
        AppError::Internal(format!("Database error: {e}"))
    })?;

    tracing::info!(user_id=%user.id, "User upserted");

    let jwt_secret = &crate::env::ENV.jwt_secret;

    let now = Utc::now().timestamp() as usize;
    let claims = JwtClaims {
        sub: user.id.to_string(),
        email: email.clone(),
        exp: now + 86400 * 7,
        iat: now,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("JWT encoding error: {e}")))?;

    Ok(Json(json!(AuthResponse {
        token,
        user: PublicUser::from(user),
    })))
}

/// GET /api/auth/me
pub async fn me(
    AuthUser { id, .. }: AuthUser,
    State(pool): State<PgPool>,
) -> Result<Json<Value>, AppError> {
    let user: User = sqlx::query_as("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::legacy_not_found("User not found"))?;

    let groups: Vec<GroupWithBalance> = sqlx::query_as(
        r#"SELECT g.*, gm.balance
           FROM groups g
           JOIN group_members gm ON gm.group_id = g.id
           WHERE gm.user_id = $1
           ORDER BY g.name"#,
    )
    .bind(id)
    .fetch_all(&pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(json!({
        "user": PublicUser::from(user),
        "groups": groups,
    })))
}

/// PATCH /api/me — update mutable per-user fields.
///
/// Body: `{ "email_notifications"?: bool, "locale"?: string }`. At
/// least one field must be present. Returns the updated `PublicUser`
/// so the frontend can keep its local state in sync without a second
/// `GET /api/auth/me`.
#[derive(Debug, Deserialize)]
pub struct PatchMeRequest {
    #[serde(default)]
    pub email_notifications: Option<bool>,
    #[serde(default)]
    pub locale: Option<String>,
}

/// PATCH /api/me
pub async fn patch_me(
    AuthUser { id, .. }: AuthUser,
    State(pool): State<PgPool>,
    Json(body): Json<PatchMeRequest>,
) -> Result<Json<Value>, AppError> {
    if body.email_notifications.is_none() && body.locale.is_none() {
        return Err(AppError::BadRequest(
            ErrorCode::Internal,
            Some("At least one of email_notifications or locale must be set".into()),
        ));
    }

    if let Some(locale) = &body.locale {
        if locale.is_empty() || locale.len() > 10 {
            return Err(AppError::BadRequest(
                ErrorCode::Internal,
                Some("locale must be 1..=10 chars".into()),
            ));
        }
    }

    // Coalesce the two optional fields into one UPDATE so we only hit
    // the DB once. SQLx doesn't have a native COALESCE-set pattern,
    // so we hand-roll it with a CASE expression.
    let result = sqlx::query(
        r#"UPDATE users SET
               email_notifications = COALESCE($2, email_notifications),
               locale              = COALESCE($3, locale)
           WHERE id = $1"#,
    )
    .bind(id)
    .bind(body.email_notifications)
    .bind(body.locale.as_deref())
    .execute(&pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {e}")))?;

    if result.rows_affected() == 0 {
        return Err(AppError::legacy_not_found("User not found"));
    }

    let user: User = sqlx::query_as("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {e}")))?;

    Ok(Json(json!({ "user": PublicUser::from(user) })))
}

// ── Dev only ──────────────────────────────────────────
pub async fn dev_login(State(pool): State<PgPool>) -> Result<Json<Value>, AppError> {
    if crate::env::ENV.is_prod() {
        return Err(AppError::legacy_not_found("Not found"));
    }

    let names = ["Pelé", "Zico", "Romário", "Ronaldo", "Ronaldinho", "Kaká"];
    let random_id = Uuid::new_v4();
    let name = names[random_id.as_bytes()[0] as usize % names.len()];
    let email = format!("test-{random_id}@dev.local");

    // Ensure the email is in the beta allowlist
    sqlx::query("INSERT INTO beta_allowlist (email) VALUES ($1) ON CONFLICT DO NOTHING")
        .bind(&email)
        .execute(&pool)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {e}")))?;

    // Upsert the test user
    let user: User = sqlx::query_as(
        r#"INSERT INTO users (id, username, email, google_id, avatar_url)
               VALUES (gen_random_uuid(), $1, $2, $3, $4)
               ON CONFLICT (google_id) DO NOTHING
               RETURNING *"#,
    )
    .bind(&format!("{name} {}", &random_id.to_string()[..4]))
    .bind(&email)
    .bind(&random_id.to_string())
    .bind::<Option<String>>(None)
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {e}")))?;

    // Generate JWT
    let jwt_secret = &crate::env::ENV.jwt_secret;

    let now = Utc::now().timestamp() as usize;
    let claims = JwtClaims {
        sub: user.id.to_string(),
        email: email.clone(),
        exp: now + 86400 * 7,
        iat: now,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("JWT encoding error: {e}")))?;

    tracing::info!(%email, user_id=%user.id, "Dev user logged in");

    Ok(Json(json!(AuthResponse {
        token,
        user: PublicUser::from(user),
    })))
}
