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
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicUser {
    pub id: Uuid,
    pub name: String,
    pub email: String,
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
            avatar_url: u.avatar_url,
        }
    }
}

// ── Group ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub invite_code: String,
    pub owner_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupWithBalance {
    #[serde(flatten)]
    pub group: Group,
    pub balance: f64,
}

impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for GroupWithBalance {
    fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;
        let group = Group {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            invite_code: row.try_get("invite_code")?,
            owner_id: row.try_get("owner_id")?,
            created_at: row.try_get("created_at")?,
        };
        Ok(GroupWithBalance {
            group,
            balance: row.try_get("balance")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GroupMember {
    pub group_id: Uuid,
    pub user_id: Uuid,
    pub balance: f64,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LeaderboardEntry {
    pub user_id: Uuid,
    pub name: String,
    pub email: String,
    pub avatar_url: Option<String>,
    pub balance: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
}

// ── Event ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Event {
    pub id: Uuid,
    pub external_id: String,
    pub home_team: String,
    pub away_team: String,
    pub championship: String,
    pub start_time: DateTime<Utc>,
    pub status: String,
    pub home_score: Option<i32>,
    pub away_score: Option<i32>,
    pub home_odds: Option<f64>,
    pub draw_odds: Option<f64>,
    pub away_odds: Option<f64>,
    pub raw_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Prediction {
    HomeWin,
    AwayWin,
    Draw,
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
    pub group_id: Option<Uuid>,
    pub event_id: Option<Uuid>,
    pub prediction: Option<Prediction>,
    pub amount: f64,
    pub odds: f64,
    pub status: BetStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateBetRequest {
    pub group_id: Uuid,
    pub event_id: Uuid,
    pub prediction: Prediction,
    pub amount: f64,
    pub odds: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BetWithUser {
    pub id: Uuid,
    pub user_id: Uuid,
    pub group_id: Option<Uuid>,
    pub event_id: Option<Uuid>,
    pub prediction: Option<Prediction>,
    pub amount: f64,
    pub odds: f64,
    pub status: BetStatus,
    pub created_at: DateTime<Utc>,
    pub user_name: String,
    pub user_email: String,
    pub home_team: Option<String>,
    pub away_team: Option<String>,
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
    pub sub: String,
    pub email: String,
    pub exp: usize,
    pub iat: usize,
}
