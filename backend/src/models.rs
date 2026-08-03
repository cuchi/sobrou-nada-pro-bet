use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

// ── User ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: Option<String>,
    pub email: Option<String>,
    pub google_id: Option<String>,
    pub balance: f64,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicUser {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub balance: f64,
    pub avatar_url: Option<String>,
}

impl From<User> for PublicUser {
    fn from(u: User) -> Self {
        PublicUser {
            id: u.id,
            name: u
                .username
                .unwrap_or_else(|| u.email.clone().unwrap_or_default()),
            email: u.email.unwrap_or_default(),
            balance: u.balance,
            avatar_url: u.avatar_url,
        }
    }
}

// ── Bet ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum BetStatus {
    Pending,
    Won,
    Lost,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Bet {
    pub id: Uuid,
    pub user_id: Uuid,
    pub amount: f64,
    pub odds: f64,
    pub status: BetStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateBetRequest {
    pub amount: f64,
    pub odds: f64,
}

// ── Auth ──────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct GoogleAuthRequest {
    pub credential: String,
}

#[derive(Debug, Deserialize)]
pub struct GoogleTokenClaims {
    pub aud: Option<String>,
    pub sub: String,
    pub email: Option<String>,
    pub email_verified: Option<String>,
    pub name: Option<String>,
    pub picture: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: PublicUser,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String, // user id (UUID)
    pub email: String,
    pub exp: usize, // expiry
    pub iat: usize, // issued at
}
