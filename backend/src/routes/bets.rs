use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::{AppError, ErrorCode};
use crate::models::{Bet, BetWithUser, CreateBetRequest, GroupMember};

// ── Bets ──────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ListBetsQuery {
    group_id: Uuid,
}

/// GET /api/bets?group_id=... — list bets for a group (authenticated)
pub async fn list_bets(
    AuthUser { id: user_id, .. }: AuthUser,
    State(pool): State<PgPool>,
    Query(query): Query<ListBetsQuery>,
) -> Result<Json<Value>, AppError> {
    let is_member: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM group_members WHERE group_id = $1 AND user_id = $2)",
    )
    .bind(query.group_id)
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    if !is_member {
        return Err(AppError::legacy_forbidden(
            "You're not a member of this group",
        ));
    }

    let bets: Vec<BetWithUser> = sqlx::query_as(
        r#"SELECT b.*, COALESCE(u.username, u.email) AS user_name, u.email AS user_email,
                  u.avatar_url AS user_avatar_url,
                  e.home_team, e.away_team
           FROM bets b
           JOIN users u ON u.id = b.user_id
           LEFT JOIN events e ON e.id = b.event_id
           WHERE b.group_id = $1
           ORDER BY b.created_at DESC"#,
    )
    .bind(query.group_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(json!(bets)))
}

/// POST /api/bets — create a bet (authenticated, group-scoped, deducts balance)
pub async fn create_bet(
    AuthUser { id: user_id, .. }: AuthUser,
    State(pool): State<PgPool>,
    Json(payload): Json<CreateBetRequest>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    // Validate membership and check balance
    let member: GroupMember =
        sqlx::query_as("SELECT * FROM group_members WHERE group_id = $1 AND user_id = $2")
            .bind(payload.group_id)
            .bind(user_id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or_else(|| AppError::Forbidden(ErrorCode::NotGroupMember, None))?;

    if member.balance < payload.amount {
        return Err(AppError::BadRequest(
            ErrorCode::InsufficientBalance {
                have: member.balance,
                bet: payload.amount,
            },
            None,
        ));
    }

    // Check for duplicate bet on same event
    let already_bet: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM bets WHERE user_id = $1 AND group_id = $2 AND event_id = $3 AND status = 'pending')",
    )
    .bind(user_id)
    .bind(payload.group_id)
    .bind(payload.event_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    if already_bet {
        return Err(AppError::BadRequest(ErrorCode::AlreadyBetOnEvent, None));
    }

    // Check event starts at least 1 hour from now
    let event_start: chrono::DateTime<Utc> =
        sqlx::query_scalar("SELECT start_time FROM events WHERE id = $1")
            .bind(payload.event_id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or_else(|| AppError::BadRequest(ErrorCode::EventNotFound, None))?;

    let cutoff = Utc::now() + chrono::Duration::hours(1);
    if event_start < cutoff {
        return Err(AppError::BadRequest(ErrorCode::BettingClosed, None));
    }

    // Deduct points
    sqlx::query(
        "UPDATE group_members SET balance = balance - $1 WHERE group_id = $2 AND user_id = $3",
    )
    .bind(payload.amount)
    .bind(payload.group_id)
    .bind(user_id)
    .execute(&pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    // Create bet
    let bet: Bet = sqlx::query_as(
        "INSERT INTO bets (id, user_id, group_id, event_id, prediction, amount, odds)
         VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, $6)
         RETURNING *",
    )
    .bind(user_id)
    .bind(payload.group_id)
    .bind(payload.event_id)
    .bind(&payload.prediction)
    .bind(payload.amount)
    .bind(payload.odds)
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(json!(bet))))
}
