//! Outbound email via Mailgun.
//!
//! All sends go through [`client::EmailClient`]. When `MAILGUN_API_KEY`
//! or `MAILGUN_DOMAIN` is unset (dev / tests), the client short-circuits
//! to a logged no-op so callers don't have to branch on environment.
//!
//! ## Triggers
//! - `send_bet_resolved` — fires from `admin::resolve_bets` once a bet's
//!   status flips to won/lost and `notified_at` is NULL. Sets
//!   `notified_at` only on successful send so re-runs retry.
//! - `send_new_events_digest` — fires from `admin::sync_events` when
//!   fresh events land. Per-user digest timestamp on `users.new_events_notified_at`
//!   keeps it idempotent.
//!
//! Both functions are no-ops when:
//! - the user has `email_notifications = false`, or
//! - the user has no email on file.

pub mod client;
pub mod templates;

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::email::client::EmailClient;

/// Locales we currently template. Anything else falls back to English.
pub type Locale = String;

/// Outcome of a resolved bet, mirrored from `routes/admin::outcome`.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BetOutcome {
    Won,
    Lost,
}

/// Payload for `send_bet_resolved`. All fields are required for the
/// template to render — the caller is responsible for gathering them.
#[derive(Debug, Clone, Serialize)]
pub struct ResolvedBetPayload {
    pub user_email: String,
    pub user_locale: Locale,
    pub user_name: String,
    pub home_team: String,
    pub away_team: String,
    pub prediction: String,
    pub amount: f64,
    pub odds: f64,
    pub outcome: BetOutcome,
    pub final_score: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NewEventsPayload {
    pub user_email: String,
    pub user_locale: Locale,
    pub user_name: String,
    pub events: Vec<NewEvent>,
    pub since: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NewEvent {
    pub home_team: String,
    pub away_team: String,
    pub championship: String,
    pub start_time: DateTime<Utc>,
}

/// Send the "your bet was resolved" email. Returns `Ok(())` either way
/// (send succeeded, or it was a no-op because of an unset key /
/// opt-out / missing email). Errors are logged and swallowed — the
/// caller already updated `bets.notified_at` based on the Ok/Err
/// return.
pub async fn send_bet_resolved(
    pool: &PgPool,
    client: &EmailClient,
    bet_id: Uuid,
    payload: ResolvedBetPayload,
) -> Result<(), String> {
    // Idempotency check — re-read notified_at in case another caller beat
    // us to it.
    let notified: Option<(Option<DateTime<Utc>>,)> =
        sqlx::query_as("SELECT notified_at FROM bets WHERE id = $1")
            .bind(bet_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;
    if let Some((Some(_),)) = notified {
        return Ok(());
    }
    if payload.user_email.is_empty() {
        return Ok(());
    }

    let (subject, body_text, body_html) = templates::render_bet_resolved(&payload);
    client
        .send(
            &payload.user_email,
            &payload.user_name,
            &subject,
            &body_text,
            &body_html,
        )
        .await?;

    sqlx::query("UPDATE bets SET notified_at = NOW() WHERE id = $1 AND notified_at IS NULL")
        .bind(bet_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Send the "new events are up" digest. The caller has already filtered
/// for opted-in users; this function renders and sends.
pub async fn send_new_events_digest(
    pool: &PgPool,
    client: &EmailClient,
    user_id: Uuid,
    payload: NewEventsPayload,
) -> Result<(), String> {
    if payload.user_email.is_empty() {
        return Ok(());
    }

    let (subject, body_text, body_html) = templates::render_new_events(&payload);
    client
        .send(
            &payload.user_email,
            &payload.user_name,
            &subject,
            &body_text,
            &body_html,
        )
        .await?;

    sqlx::query("UPDATE users SET new_events_notified_at = NOW() WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
