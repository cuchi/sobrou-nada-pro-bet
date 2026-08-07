use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};

/// Stable, machine-readable error codes for the Phase D i18n contract.
///
/// Each variant carries an English fallback message via `#[error("...")]`
/// that the API emits verbatim in the `message` field of the response body.
/// The frontend uses `as_code()` to look up a translated template by locale
/// and `as_params()` to interpolate them.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ErrorCode {
    #[error("Failed to verify Google token")]
    AuthGoogleFailed,
    #[error("Invalid Google token")]
    AuthGoogleInvalid,
    #[error("This app is currently in closed beta. Contact paulo@cuchi.me to request access.")]
    AuthNotOnAllowlist,
    #[error("You're not a member of this group")]
    NotGroupMember,
    #[error("Insufficient balance. You have {have:.0} points, bet is {bet:.0}.")]
    InsufficientBalance { have: f64, bet: f64 },
    #[error("You already have a pending bet on this event")]
    AlreadyBetOnEvent,
    #[error("Event not found")]
    EventNotFound,
    #[error("Bets close 1 hour before kickoff")]
    BettingClosed,
    #[error("Invalid invite code")]
    InvalidInviteCode,
    #[error("You're already in this group")]
    AlreadyInGroup,
    #[error("Internal server error")]
    Internal,
}

impl ErrorCode {
    /// Stable snake_case wire identifier.
    pub fn as_code(&self) -> &'static str {
        match self {
            ErrorCode::AuthGoogleFailed => "auth_google_failed",
            ErrorCode::AuthGoogleInvalid => "auth_google_invalid",
            ErrorCode::AuthNotOnAllowlist => "auth_not_on_allowlist",
            ErrorCode::NotGroupMember => "not_group_member",
            ErrorCode::InsufficientBalance { .. } => "insufficient_balance",
            ErrorCode::AlreadyBetOnEvent => "already_bet_on_event",
            ErrorCode::EventNotFound => "event_not_found",
            ErrorCode::BettingClosed => "betting_closed",
            ErrorCode::InvalidInviteCode => "invalid_invite_code",
            ErrorCode::AlreadyInGroup => "already_in_group",
            ErrorCode::Internal => "internal",
        }
    }

    /// JSON params object, or `null` if this code takes no parameters.
    pub fn as_params(&self) -> Value {
        match self {
            ErrorCode::InsufficientBalance { have, bet } => json!({ "have": have, "bet": bet }),
            _ => Value::Null,
        }
    }
}

/// API error type. Two parallel paths:
///
/// - **Structured variants** (`Unauthorized`, `Forbidden`, `NotFound`,
///   `BadRequest`) carry an [`ErrorCode`] plus an optional override message.
///   `IntoResponse` emits the new `{ code, params, message }` shape.
///
/// - **Legacy variants** (`LegacyUnauthorized`, `LegacyForbidden`,
///   `LegacyNotFound`, `LegacyBadRequest`) carry a free-text `String` and
///   emit the old `{ "error": "<string>" }` shape. They exist so out-of-scope
///   routes (per the Phase D contract) can keep their current behavior
///   without churn. New code should prefer the structured variants.
///
/// - `Internal` always emits the structured shape with `code: "internal"`,
///   `params: null`, `message: "Internal server error"`. The internal detail
///   is logged but never sent to the client.
#[derive(Debug, Clone, thiserror::Error)]
pub enum AppError {
    // ── Structured variants (in-scope routes) ────────────
    #[error("{0}")]
    Unauthorized(ErrorCode, Option<String>),
    #[error("{0}")]
    Forbidden(ErrorCode, Option<String>),
    #[error("{0}")]
    NotFound(ErrorCode, Option<String>),
    #[error("{0}")]
    BadRequest(ErrorCode, Option<String>),

    // ── Legacy variants (out-of-scope routes) ────────────
    // Preserve the old `{ "error": "<string>" }` wire shape exactly.
    #[error("{0}")]
    LegacyUnauthorized(String),
    #[error("{0}")]
    LegacyForbidden(String),
    #[error("{0}")]
    LegacyNotFound(String),
    #[error("{0}")]
    LegacyBadRequest(String),

    // ── Internal (unchanged) ─────────────────────────────
    /// The inner message is **never** sent to the client — it's logged only.
    #[error("{0}")]
    Internal(String),
}

impl AppError {
    /// Convenience constructors for the legacy path. Call sites that don't
    /// participate in the Phase D contract should use these instead of
    /// touching the structured variants.
    pub fn legacy_unauthorized(msg: impl Into<String>) -> Self {
        AppError::LegacyUnauthorized(msg.into())
    }
    pub fn legacy_forbidden(msg: impl Into<String>) -> Self {
        AppError::LegacyForbidden(msg.into())
    }
    pub fn legacy_not_found(msg: impl Into<String>) -> Self {
        AppError::LegacyNotFound(msg.into())
    }
    pub fn legacy_bad_request(msg: impl Into<String>) -> Self {
        AppError::LegacyBadRequest(msg.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Legacy variants — preserve the original `{ "error": "<string>" }` shape.
        if let AppError::LegacyUnauthorized(msg) = &self {
            return (StatusCode::UNAUTHORIZED, Json(json!({ "error": msg }))).into_response();
        }
        if let AppError::LegacyForbidden(msg) = &self {
            return (StatusCode::FORBIDDEN, Json(json!({ "error": msg }))).into_response();
        }
        if let AppError::LegacyNotFound(msg) = &self {
            return (StatusCode::NOT_FOUND, Json(json!({ "error": msg }))).into_response();
        }
        if let AppError::LegacyBadRequest(msg) = &self {
            return (StatusCode::BAD_REQUEST, Json(json!({ "error": msg }))).into_response();
        }

        // Internal — never leak the detail to the client.
        if let AppError::Internal(detail) = &self {
            tracing::error!(%detail, "Internal server error");
            let body = json!({
                "code": "internal",
                "params": Value::Null,
                "message": "Internal server error",
            });
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response();
        }

        // Structured variants — emit the new contract shape.
        let (status, code, message_override) = match &self {
            AppError::Unauthorized(code, msg) => (StatusCode::UNAUTHORIZED, code, msg),
            AppError::Forbidden(code, msg) => (StatusCode::FORBIDDEN, code, msg),
            AppError::NotFound(code, msg) => (StatusCode::NOT_FOUND, code, msg),
            AppError::BadRequest(code, msg) => (StatusCode::BAD_REQUEST, code, msg),
            // Handled above; this arm is unreachable but keeps the match exhaustive.
            AppError::Internal(_)
            | AppError::LegacyUnauthorized(_)
            | AppError::LegacyForbidden(_)
            | AppError::LegacyNotFound(_)
            | AppError::LegacyBadRequest(_) => unreachable!(),
        };

        let message = message_override.clone().unwrap_or_else(|| code.to_string());

        let body = json!({
            "code": code.as_code(),
            "params": code.as_params(),
            "message": message,
        });

        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use serde_json::Value;

    async fn body_of(resp: Response) -> (StatusCode, Value) {
        let status = resp.status();
        let bytes = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
        let v: Value = serde_json::from_slice(&bytes).unwrap();
        (status, v)
    }

    #[tokio::test]
    async fn internal_error_uses_canonical_shape() {
        let resp = AppError::Internal("db blew up".into()).into_response();
        let (status, body) = body_of(resp).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body["code"], "internal");
        assert_eq!(body["params"], Value::Null);
        assert_eq!(body["message"], "Internal server error");
        // The internal detail must NOT leak.
        assert!(body.get("error").is_none());
        assert!(!body.to_string().contains("db blew up"));
    }

    #[tokio::test]
    async fn insufficient_balance_carries_params() {
        let err = AppError::BadRequest(
            ErrorCode::InsufficientBalance {
                have: 50.0,
                bet: 100.0,
            },
            None,
        );
        let (status, body) = body_of(err.into_response()).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["code"], "insufficient_balance");
        assert_eq!(body["params"]["have"], 50.0);
        assert_eq!(body["params"]["bet"], 100.0);
        assert_eq!(
            body["message"],
            "Insufficient balance. You have 50 points, bet is 100."
        );
    }

    #[tokio::test]
    async fn not_on_allowlist_is_403_with_null_params() {
        let err = AppError::Forbidden(ErrorCode::AuthNotOnAllowlist, None);
        let (status, body) = body_of(err.into_response()).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(body["code"], "auth_not_on_allowlist");
        assert_eq!(body["params"], Value::Null);
    }

    #[tokio::test]
    async fn legacy_unauthorized_keeps_old_shape() {
        let err = AppError::LegacyUnauthorized("Missing Authorization header".into());
        let (status, body) = body_of(err.into_response()).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body["error"], "Missing Authorization header");
        // No structured fields.
        assert!(body.get("code").is_none());
        assert!(body.get("params").is_none());
        assert!(body.get("message").is_none());
    }
}
