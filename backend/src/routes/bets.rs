use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::AppError;
use crate::models::{Bet, BetStatus, BetWithUser, CreateBetRequest, GroupMember};

// ── Bets ──────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ListBetsQuery {
    group_id: Option<Uuid>,
}

/// GET /api/bets — list bets from the user's groups (authenticated)
pub async fn list_bets(
    AuthUser { id: user_id, .. }: AuthUser,
    State(pool): State<PgPool>,
    Query(query): Query<ListBetsQuery>,
) -> Result<Json<Value>, AppError> {
    let bets: Vec<BetWithUser> = if let Some(group_id) = query.group_id {
        let is_member: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM group_members WHERE group_id = $1 AND user_id = $2)",
        )
        .bind(group_id)
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        if !is_member {
            return Err(AppError::Forbidden(
                "You're not a member of this group".into(),
            ));
        }

        sqlx::query_as(
            r#"SELECT b.*, COALESCE(u.username, u.email) AS user_name, u.email AS user_email,
                      u.avatar_url AS user_avatar_url,
                      e.home_team, e.away_team
               FROM bets b
               JOIN users u ON u.id = b.user_id
               LEFT JOIN events e ON e.id = b.event_id
               WHERE b.group_id = $1
               ORDER BY b.created_at DESC"#,
        )
        .bind(group_id)
        .fetch_all(&pool)
        .await
    } else {
        sqlx::query_as(
            r#"SELECT b.*, COALESCE(u.username, u.email) AS user_name, u.email AS user_email,
                      u.avatar_url AS user_avatar_url,
                      e.home_team, e.away_team
               FROM bets b
               JOIN group_members gm ON gm.group_id = b.group_id
               JOIN users u ON u.id = b.user_id
               LEFT JOIN events e ON e.id = b.event_id
               WHERE gm.user_id = $1
               ORDER BY b.created_at DESC"#,
        )
        .bind(user_id)
        .fetch_all(&pool)
        .await
    }
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
            .ok_or_else(|| AppError::BadRequest("You're not a member of this group".into()))?;

    if member.balance < payload.amount {
        return Err(AppError::BadRequest(format!(
            "Insufficient balance. You have {:.0} points, bet is {:.0}.",
            member.balance, payload.amount
        )));
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
        return Err(AppError::BadRequest(
            "You already have a pending bet on this event".into(),
        ));
    }

    // Check event starts at least 1 hour from now
    let event_start: chrono::DateTime<Utc> =
        sqlx::query_scalar("SELECT start_time FROM events WHERE id = $1")
            .bind(payload.event_id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or_else(|| AppError::BadRequest("Event not found".into()))?;

    let cutoff = Utc::now() + chrono::Duration::hours(1);
    if event_start < cutoff {
        return Err(AppError::BadRequest(
            "Bets close 1 hour before kickoff".into(),
        ));
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

/// PATCH /api/bets/:id/resolve — resolve a bet and update group balance
pub async fn resolve_bet(
    _auth: AuthUser,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
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

    let bet: Bet = sqlx::query_as("SELECT * FROM bets WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Bet {id} not found")))?;

    if bet.status != BetStatus::Pending {
        return Err(AppError::BadRequest("Bet is already resolved".into()));
    }

    // Update bet status
    let bet: Bet = sqlx::query_as("UPDATE bets SET status = $1 WHERE id = $2 RETURNING *")
        .bind(&new_status)
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // Update group balance (payout = amount * odds if won)
    if new_status == BetStatus::Won {
        let payout = bet.amount * bet.odds;
        if let Some(group_id) = bet.group_id {
            sqlx::query(
                "UPDATE group_members SET balance = balance + $1 WHERE group_id = $2 AND user_id = $3",
            )
            .bind(payout)
            .bind(group_id)
            .bind(bet.user_id)
            .execute(&pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        }
    }

    Ok(Json(json!(bet)))
}
