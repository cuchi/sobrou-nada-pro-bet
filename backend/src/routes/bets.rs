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
use crate::email::client::EmailClient;
use crate::error::{AppError, ErrorCode};
use crate::models::{Bet, BetWithUser, CreateBetRequest, GroupMember};
use crate::routes::admin::resolve_event;

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

// ── Dev only ───────────────────────────────────────────

/// Body for `POST /api/dev/resolve-bet`. The caller picks an outcome;
/// the server synthesizes a score and runs the same resolve + email
/// pipeline that `/admin/bets/resolve` would for the parent event.
#[derive(Debug, Deserialize)]
pub struct DevResolveBetRequest {
    pub bet_id: Uuid,
    pub outcome: String,
}

/// POST /api/dev/resolve-bet
///
/// Dev-only: returns 404 when `ENVIRONMENT == "production"` so the
/// endpoint is invisible to prod traffic (matches the `dev_login`
/// pattern). Synthesizes a final score for the outcome:
/// - `home_win` → home 1, away 0
/// - `draw`     → home 1, away 1
/// - `away_win` → home 0, away 1
///
/// Errors:
/// - 404 if the bet doesn't exist
/// - 409 if the bet is already resolved/cancelled
/// - 400 if the outcome is invalid
pub async fn dev_resolve_bet(
    AuthUser { .. }: AuthUser,
    State(pool): State<PgPool>,
    Json(body): Json<DevResolveBetRequest>,
) -> Result<Json<Value>, AppError> {
    if crate::env::ENV.is_prod() {
        return Err(AppError::legacy_not_found("Not found"));
    }

    let (home_score, away_score) = match body.outcome.as_str() {
        "home_win" => (1, 0),
        "draw" => (1, 1),
        "away_win" => (0, 1),
        other => {
            return Err(AppError::legacy_bad_request(format!(
                "Invalid outcome: {other}. Expected home_win | draw | away_win."
            )));
        }
    };

    // Load the bet and its event in one query.
    let row: Option<(String, String, String, Option<Uuid>, String)> = sqlx::query_as(
        r#"SELECT b.status::text, e.external_id, e.home_team, b.event_id, e.away_team
           FROM bets b
           JOIN events e ON e.id = b.event_id
           WHERE b.id = $1"#,
    )
    .bind(body.bet_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let (status, external_id, home_team, _event_id, away_team) =
        row.ok_or_else(|| AppError::legacy_not_found("Bet not found"))?;

    if status != "pending" {
        return Err(AppError::legacy_bad_request(format!(
            "Bet is already {status}"
        )));
    }

    // Run the shared resolve pipeline. resolve_event updates the event
    // row's scores and flips every pending bet on it — same as the prod
    // path, just driven by synthetic scores.
    let client = EmailClient::from_env();
    let synthetic = crate::routes::admin::FinishedMatch {
        external_id: external_id.clone(),
        home_team,
        away_team,
        home_score,
        away_score,
    };
    let (resolved, _updated_scores) = resolve_event(&pool, &client, &synthetic).await;

    tracing::info!(
        bet_id = %body.bet_id,
        outcome = %body.outcome,
        resolved,
        "Dev-resolved bet"
    );

    Ok(Json(json!({
        "bet_id": body.bet_id,
        "outcome": body.outcome,
        "score": format!("{home_score}–{away_score}"),
        "resolved": resolved,
    })))
}
