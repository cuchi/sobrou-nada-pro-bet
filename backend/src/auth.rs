use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::error::AppError;
use crate::models::JwtClaims;

/// Extracted from a valid Bearer token — user is authenticated.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: uuid::Uuid,
    #[allow(dead_code)]
    pub email: String,
}

/// Axum `FromRequestParts` impl so route handlers can use `AuthUser` directly.
impl<S> axum::extract::FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Read JWT secret from env (lazy — cached in practice, but fine for dev)
        let secret = std::env::var("JWT_SECRET")
            .map_err(|_| AppError::Internal("JWT_SECRET not set".into()))?;

        let header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".into()))?;

        let token = header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Unauthorized("Expected Bearer token".into()))?;

        let token_data = decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| AppError::Unauthorized(format!("Invalid token: {e}")))?;

        let claims = token_data.claims;
        let id = claims
            .sub
            .parse::<uuid::Uuid>()
            .map_err(|_| AppError::Unauthorized("Invalid user id in token".into()))?;

        Ok(AuthUser {
            id,
            email: claims.email,
        })
    }
}
