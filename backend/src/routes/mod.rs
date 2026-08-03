use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::{json, Value};
use sqlx::PgPool;

use crate::auth::AuthUser;
use crate::error::AppError;
use crate::models::{
    AuthResponse, Bet, BetStatus, CreateBetRequest, GoogleAuthRequest, GoogleTokenClaims,
    JwtClaims, PublicUser, User,
};

/// GET /health
pub async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

// ── Auth ──────────────────────────────────────────────

/// POST /api/auth/google
///
/// Receives a Google ID token, verifies it, upserts the user,
/// and returns a JWT session token.
#[tracing::instrument(skip(pool, body))]
pub async fn google_login(
    State(pool): State<PgPool>,
    Json(body): Json<GoogleAuthRequest>,
) -> Result<Json<Value>, AppError> {
    // 1. Verify the Google ID token
    let client_id = std::env::var("GOOGLE_CLIENT_ID").map_err(|e| {
        tracing::error!("GOOGLE_CLIENT_ID not set: {e}");
        AppError::Internal("GOOGLE_CLIENT_ID not set".into())
    })?;

    tracing::debug!("Calling Google tokeninfo…");

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

    // Verify audience matches our client ID
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

    // 2. Upsert user in the database
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

    // 3. Generate JWT
    let jwt_secret = std::env::var("JWT_SECRET").map_err(|e| {
        tracing::error!("JWT_SECRET not set: {e}");
        AppError::Internal("JWT_SECRET not set".into())
    })?;

    let now = Utc::now().timestamp() as usize;
    let claims = JwtClaims {
        sub: user.id.to_string(),
        email: email.clone(),
        exp: now + 86400 * 7, // 7 days
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

/// GET /api/auth/me — return the currently authenticated user
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

    Ok(Json(json!(PublicUser::from(user))))
}

// ── Bets ──────────────────────────────────────────────

/// GET /api/bets — list all bets ordered by most recent first
pub async fn list_bets(State(pool): State<PgPool>) -> Result<Json<Value>, AppError> {
    let bets: Vec<Bet> = sqlx::query_as("SELECT * FROM bets ORDER BY created_at DESC")
        .fetch_all(&pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(json!(bets)))
}

/// POST /api/bets — create a new bet (authenticated)
pub async fn create_bet(
    AuthUser { id: user_id, .. }: AuthUser,
    State(pool): State<PgPool>,
    Json(payload): Json<CreateBetRequest>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let bet: Bet = sqlx::query_as(
        "INSERT INTO bets (id, user_id, amount, odds)
         VALUES (gen_random_uuid(), $1, $2, $3)
         RETURNING *",
    )
    .bind(user_id)
    .bind(payload.amount)
    .bind(payload.odds)
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(json!(bet))))
}

/// PATCH /api/bets/:id/resolve — resolve a bet as Won or Lost (authenticated)
pub async fn resolve_bet(
    _auth: AuthUser,
    State(pool): State<PgPool>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<Value>, AppError> {
    let status_str = body
        .get("status")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Missing 'status' field".into()))?;

    let new_status = match status_str {
        "won" => BetStatus::Won,
        "lost" => BetStatus::Lost,
        _ => {
            return Err(AppError::BadRequest(
                "Status must be 'won' or 'lost'".into(),
            ))
        }
    };

    let bet: Option<Bet> = sqlx::query_as("UPDATE bets SET status = $1 WHERE id = $2 RETURNING *")
        .bind(&new_status)
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    match bet {
        Some(b) => Ok(Json(json!(b))),
        None => Err(AppError::NotFound(format!("Bet {id} not found"))),
    }
}
