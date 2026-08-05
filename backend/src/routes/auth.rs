use axum::{Json, extract::State};
use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header, encode};
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::AppError;
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
        AppError::Unauthorized(format!("Failed to verify token: {e}"))
    })?;

    let status = resp.status();
    let raw_body = resp.text().await.map_err(|e| {
        tracing::error!("Failed to read Google response body: {e}");
        AppError::Unauthorized(format!("Failed to read token response: {e}"))
    })?;

    tracing::debug!(%status, %raw_body, "Google tokeninfo response");

    let google_claims: GoogleTokenClaims = serde_json::from_str(&raw_body).map_err(|e| {
        tracing::error!(%raw_body, "Failed to parse Google tokeninfo: {e}");
        AppError::Unauthorized(format!("Invalid token response: {e}"))
    })?;

    if let Some(aud) = &google_claims.aud {
        if aud != &client_id {
            tracing::error!(expected=%client_id, got=%aud, "Token audience mismatch");
            return Err(AppError::Unauthorized("Token audience mismatch".into()));
        }
    }
    if google_claims.sub.is_empty() {
        tracing::error!("Google token has empty sub claim");
        return Err(AppError::Unauthorized("Invalid Google token".into()));
    }

    let email = google_claims
        .email
        .filter(|_| google_claims.email_verified.as_deref() == Some("true"))
        .ok_or_else(|| {
            tracing::error!("Google email not verified or missing");
            AppError::Unauthorized("Email not verified by Google".into())
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
        return Err(AppError::Forbidden(
            "This app is currently in closed beta. Contact paulo@cuchi.me to request access."
                .into(),
        ));
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
        .ok_or_else(|| AppError::NotFound("User not found".into()))?;

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

// ── Dev only ──────────────────────────────────────────

/// POST /api/dev/login — creates a random test user and returns a JWT.
/// Only works when ENVIRONMENT is not "production".
pub async fn dev_login(State(pool): State<PgPool>) -> Result<Json<Value>, AppError> {
    if crate::env::ENV.is_prod() {
        return Err(AppError::NotFound("Not found".into()));
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
