use axum::{
    Json,
    extract::{FromRequestParts, State},
    http::request::Parts,
};
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::email::client::EmailClient;
use crate::email::{self, BetOutcome, NewEvent, NewEventsPayload, ResolvedBetPayload};
use crate::error::AppError;

// ── Admin token extractor ──────────────────────────────

/// Validated via `X-Admin-Token` header matching `ADMIN_TOKEN` env var.
#[derive(Debug, Clone)]
pub struct AdminAuth;

impl<S> FromRequestParts<S> for AdminAuth
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let expected = &crate::env::ENV.admin_token;

        let provided = parts
            .headers
            .get("x-admin-token")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AppError::legacy_unauthorized("Missing X-Admin-Token header"))?;

        if provided != expected {
            return Err(AppError::legacy_forbidden("Invalid admin token"));
        }

        Ok(AdminAuth)
    }
}

// ── Data types ─────────────────────────────────────────

struct ParsedMatch {
    external_id: String,
    home_team: String,
    away_team: String,
    championship: String,
    start_time: chrono::DateTime<chrono::Utc>,
    status: &'static str,
    home_odds: Option<f64>,
    draw_odds: Option<f64>,
    away_odds: Option<f64>,
}

pub struct FinishedMatch {
    pub external_id: String,
    pub home_team: String,
    pub away_team: String,
    pub home_score: i32,
    pub away_score: i32,
}

/// Per-bet info gathered by `process_scores` for the email fan-out.
/// Includes user preferences so the SQL can filter opted-out users
/// in one pass.
#[derive(FromRow)]
struct PendingBet {
    bet_id: Uuid,
    user_id: Uuid,
    group_id: Option<Uuid>,
    user_email: Option<String>,
    user_locale: String,
    user_name: String,
    email_notifications: bool,
    prediction: String,
    amount: f64,
    odds: f64,
}

/// A bet that was resolved in a previous run but the email send
/// failed. Re-run picks these up via the retry pass.
#[derive(FromRow)]
struct UnnotifiedBet {
    bet_id: Uuid,
    user_email: String,
    user_locale: String,
    user_name: String,
    home_team: String,
    away_team: String,
    prediction: String,
    amount: f64,
    odds: f64,
    final_score: String,
    won: bool,
}

// ── Pure parsing helpers ───────────────────────────────

/// Extract odds from the first h2h market of the first bookmaker.
fn extract_odds(
    m: &serde_json::Value,
    home: &str,
    away: &str,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let fold_outcome = |(h, d, a): (Option<f64>, Option<f64>, Option<f64>),
                        o: &serde_json::Value| {
        let name = o["name"].as_str().unwrap_or("");
        let price = o["price"].as_f64();
        match name {
            n if n == home => (price, d, a),
            n if n == away => (h, d, price),
            n if n.eq_ignore_ascii_case("draw") => (h, price, a),
            _ => (h, d, a),
        }
    };

    m["bookmakers"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|bm| bm["markets"].as_array())
        .flatten()
        .find(|market| market["key"].as_str() == Some("h2h"))
        .and_then(|market| market["outcomes"].as_array())
        .map(|outcomes| outcomes.iter().fold((None, None, None), fold_outcome))
        .unwrap_or((None, None, None))
}

fn parse_match_odds(m: &serde_json::Value) -> Option<ParsedMatch> {
    let external_id = m["id"].as_str().filter(|s| !s.is_empty())?.to_string();
    let home_team = m["home_team"].as_str().unwrap_or("TBD").to_string();
    let away_team = m["away_team"].as_str().unwrap_or("TBD").to_string();
    let championship = m["sport_title"]
        .as_str()
        .unwrap_or("Brazil Série A")
        .to_string();

    let start_time = m["commence_time"]
        .as_str()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|| chrono::Utc::now());

    // Sync only stores scheduled events. 'live' / 'finished' / 'cancelled'
    // states are derived dynamically in the events listing.
    let status = "scheduled";

    let (home_odds, draw_odds, away_odds) = extract_odds(m, &home_team, &away_team);

    tracing::info!(%home_team, %away_team, ?home_odds, ?draw_odds, ?away_odds, "Match with odds");

    Some(ParsedMatch {
        external_id,
        home_team,
        away_team,
        championship,
        start_time,
        status,
        home_odds,
        draw_odds,
        away_odds,
    })
}

fn outcome(home: i32, away: i32) -> &'static str {
    match home.cmp(&away) {
        std::cmp::Ordering::Greater => "home_win",
        std::cmp::Ordering::Less => "away_win",
        std::cmp::Ordering::Equal => "draw",
    }
}

fn parse_finished_match(m: &serde_json::Value) -> Option<FinishedMatch> {
    let external_id = m["id"].as_str().filter(|s| !s.is_empty())?.to_string();
    let home_team = m["home_team"].as_str().unwrap_or("TBD").to_string();
    let away_team = m["away_team"].as_str().unwrap_or("TBD").to_string();
    let completed = m["completed"].as_bool().unwrap_or(false);
    if !completed {
        return None;
    }
    // the-odds-api v4 scores shape:
    //   "scores": [{"name": "<home_team>", "score": "<int_as_string>"}, ...]
    // Names are matched against `home_team` / `away_team` (case-sensitive
    // since the API echoes them verbatim). `score` is a string in the
    // documented payload — parse it as i64 rather than reading as a number.
    let scores = m["scores"].as_array()?;
    let home = scores
        .iter()
        .find(|s| s["name"].as_str() == Some(home_team.as_str()))
        .and_then(|s| s["score"].as_str())
        .and_then(|s| s.parse::<i32>().ok())?;
    let away = scores
        .iter()
        .find(|s| s["name"].as_str() == Some(away_team.as_str()))
        .and_then(|s| s["score"].as_str())
        .and_then(|s| s.parse::<i32>().ok())?;
    Some(FinishedMatch {
        external_id,
        home_team,
        away_team,
        home_score: home,
        away_score: away,
    })
}

// ── Handlers ───────────────────────────────────────────

pub async fn sync_events(
    _auth: AdminAuth,
    State(pool): State<PgPool>,
) -> Result<Json<Value>, AppError> {
    let api_key = crate::env::ENV.odds_api_key.clone()?;

    let client = reqwest::Client::new();
    let sport = "soccer_brazil_campeonato";
    let base = "https://api.the-odds-api.com/v4";

    let url = format!(
        "{base}/sports/{sport}/odds/?apiKey={api_key}&regions=us&markets=h2h&oddsFormat=decimal"
    );

    let resp = client.get(&url).send().await.map_err(|e| {
        tracing::error!("Failed to fetch odds: {e}");
        AppError::Internal(format!("Failed to fetch odds: {e}"))
    })?;

    let raw: serde_json::Value = resp.json().await.map_err(|e| {
        tracing::error!("Failed to parse response: {e}");
        AppError::Internal(format!("Failed to parse response: {e}"))
    })?;

    process_odds(&pool, &raw).await
}

/// Process odds JSON (callable from tests without API key)
pub async fn process_odds(pool: &PgPool, raw: &serde_json::Value) -> Result<Json<Value>, AppError> {
    let empty = vec![];
    let matches: Vec<_> = raw
        .as_array()
        .unwrap_or(&empty)
        .iter()
        .filter_map(parse_match_odds)
        .collect();

    let total_fetched = matches.len();
    let mut inserted = 0;
    let mut new_events: Vec<NewEvent> = Vec::new();

    for m in &matches {
        // `(xmax = 0)` is set when the row was inserted (not updated),
        // so we only collect events that didn't already exist.
        let row: Option<(Uuid, DateTime<Utc>, bool)> = sqlx::query_as(
            r#"INSERT INTO events (external_id, home_team, away_team, championship, start_time, status, home_score, away_score, home_odds, draw_odds, away_odds, raw_data)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
               ON CONFLICT (external_id) DO UPDATE SET
                   status = EXCLUDED.status,
                   home_odds = EXCLUDED.home_odds,
                   draw_odds = EXCLUDED.draw_odds,
                   away_odds = EXCLUDED.away_odds,
                   start_time = EXCLUDED.start_time,
                   raw_data = EXCLUDED.raw_data
               RETURNING id, start_time, (xmax = 0) AS was_inserted"#,
        )
        .bind(&m.external_id)
        .bind(&m.home_team)
        .bind(&m.away_team)
        .bind(&m.championship)
        .bind(m.start_time)
        .bind(m.status)
        .bind::<Option<i32>>(None)
        .bind::<Option<i32>>(None)
        .bind(m.home_odds)
        .bind(m.draw_odds)
        .bind(m.away_odds)
        .bind(json!({
            "id": m.external_id,
            "home_team": m.home_team,
            "away_team": m.away_team,
            "sport_title": m.championship,
            "commence_time": m.start_time.to_rfc3339(),
        }))
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();

        if let Some((_, _, true)) = &row {
            inserted += 1;
            new_events.push(NewEvent {
                home_team: m.home_team.clone(),
                away_team: m.away_team.clone(),
                championship: m.championship.clone(),
                start_time: m.start_time,
            });
        }
    }

    // Fan out the new-events digest to opted-in users with no recent
    // digest stamp. Skipped when nothing was actually inserted.
    if !new_events.is_empty() {
        let since = Utc::now();
        let client = EmailClient::from_env();
        let users: Vec<(Uuid, String, String, String)> = sqlx::query_as(
            r#"SELECT id, email, locale, COALESCE(username, email) AS user_name
               FROM users
               WHERE email_notifications = TRUE
                 AND email IS NOT NULL
                 AND new_events_notified_at IS NULL"#,
        )
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        for (user_id, email_addr, locale, user_name) in users {
            let payload = NewEventsPayload {
                user_email: email_addr,
                user_locale: locale,
                user_name,
                events: new_events.clone(),
                since,
            };
            if let Err(e) = email::send_new_events_digest(pool, &client, user_id, payload).await {
                tracing::error!(user_id = %user_id, "send_new_events_digest failed: {e}");
            }
        }
    }

    tracing::info!(inserted, total_fetched, "Admin sync complete");

    Ok(Json(json!({
        "inserted": inserted,
        "total_fetched": total_fetched,
    })))
}

/// POST /admin/bets/resolve — fetch scores and resolve pending bets
pub async fn resolve_bets(
    _auth: AdminAuth,
    State(pool): State<PgPool>,
) -> Result<Json<Value>, AppError> {
    let api_key = crate::env::ENV.odds_api_key.clone()?;

    let client = reqwest::Client::new();
    let sport = "soccer_brazil_campeonato";
    let base = "https://api.the-odds-api.com/v4";

    let url = format!("{base}/sports/{sport}/scores/?apiKey={api_key}&daysFrom=3");

    let resp = client.get(&url).send().await.map_err(|e| {
        tracing::error!("Failed to fetch scores: {e}");
        AppError::Internal(format!("Failed to fetch scores: {e}"))
    })?;

    let raw: serde_json::Value = resp.json().await.map_err(|e| {
        tracing::error!("Failed to parse scores response: {e}");
        AppError::Internal(format!("Failed to parse scores response: {e}"))
    })?;

    process_scores(&pool, &raw).await
}

/// Process scores JSON (callable from tests without API key)
pub async fn process_scores(
    pool: &PgPool,
    raw: &serde_json::Value,
) -> Result<Json<Value>, AppError> {
    let empty = vec![];
    let finished: Vec<_> = raw
        .as_array()
        .unwrap_or(&empty)
        .iter()
        .filter_map(parse_finished_match)
        .collect();

    let mut resolved = 0;
    let mut updated_scores = 0;
    let client = EmailClient::from_env();

    for m in &finished {
        let (r, u) = resolve_event(pool, &client, m).await;
        resolved += r;
        updated_scores += u;
    }

    // Retry pass: any bet that's already won/lost from a previous run
    // but never had a successful email gets another chance. Same email
    // fan-out, but no status / balance updates.
    let pending_emails: Vec<UnnotifiedBet> = sqlx::query_as(
        r#"SELECT b.id AS bet_id,
                  u.email AS user_email, u.locale AS user_locale,
                  COALESCE(u.username, u.email) AS user_name,
                  e.home_team, e.away_team,
                  b.prediction, b.amount, b.odds,
                  CONCAT(e.home_score, ' – ', e.away_score) AS final_score,
                  (b.status = 'won') AS won
           FROM bets b
           JOIN events e ON e.id = b.event_id
           JOIN users u ON u.id = b.user_id
           WHERE b.status IN ('won', 'lost') AND b.notified_at IS NULL
             AND u.email IS NOT NULL AND u.email_notifications = TRUE
             AND e.home_score IS NOT NULL AND e.away_score IS NOT NULL"#,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    for b in pending_emails {
        let payload = ResolvedBetPayload {
            user_email: b.user_email.clone(),
            user_locale: b.user_locale.clone(),
            user_name: b.user_name.clone(),
            home_team: b.home_team.clone(),
            away_team: b.away_team.clone(),
            prediction: b.prediction.clone(),
            amount: b.amount,
            odds: b.odds,
            outcome: if b.won {
                BetOutcome::Won
            } else {
                BetOutcome::Lost
            },
            final_score: b.final_score.clone(),
        };
        if let Err(e) = email::send_bet_resolved(pool, &client, b.bet_id, payload).await {
            tracing::error!(bet_id = %b.bet_id, "send_bet_resolved (retry) failed: {e}");
        }
    }

    tracing::info!(resolved, updated_scores, "Resolve complete");

    Ok(Json(json!({
        "resolved": resolved,
        "updated_scores": updated_scores,
    })))
}

/// Resolve a single event by `external_id`: update its scores, flip
/// every pending bet on it, adjust balances, and fan out the
/// bet-resolved email. Returns `(bets_resolved, event_rows_updated)`.
///
/// Shared by `process_scores` (prod path) and the dev-only single-bet
/// resolver. SQL-filters only on `status = 'pending'`; opted-out /
/// no-email users still get status flips and balance updates — the
/// email fan-out at the end is where the preference filter applies.
pub(crate) async fn resolve_event(
    pool: &PgPool,
    client: &EmailClient,
    m: &FinishedMatch,
) -> (usize, usize) {
    // Update event scores
    let result = sqlx::query(
        "UPDATE events SET status = 'finished', home_score = $1, away_score = $2 WHERE external_id = $3",
    )
    .bind(m.home_score)
    .bind(m.away_score)
    .bind(&m.external_id)
    .execute(pool)
    .await;

    let updated_scores = if let Ok(ref r) = result {
        r.rows_affected() as usize
    } else {
        0
    };

    // Fetch pending bets for this event. Joined to users so we have
    // email + locale + name for the fan-out, but we don't filter
    // on opt-out / no-email at the SQL layer — every pending bet
    // still gets resolved (status flips, balance updates), the
    // user-preference filter is applied later when fanning out
    // emails.
    let bet_outcome = outcome(m.home_score, m.away_score);
    let final_score = format!("{} – {}", m.home_score, m.away_score);

    let bets: Vec<PendingBet> = sqlx::query_as(
        r#"SELECT b.id AS bet_id, b.user_id, b.group_id,
                  u.email AS user_email,
                  u.locale AS user_locale,
                  COALESCE(u.username, u.email, '') AS user_name,
                  u.email_notifications,
                  b.prediction, b.amount, b.odds
           FROM bets b
           JOIN events e ON e.id = b.event_id
           JOIN users u ON u.id = b.user_id
           WHERE e.external_id = $1 AND b.status = 'pending'"#,
    )
    .bind(&m.external_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    if bets.is_empty() {
        return (0, updated_scores);
    }

    // Separate winners from losers
    let (winners, losers): (Vec<_>, Vec<_>) = bets
        .iter()
        .partition(|b| b.prediction.as_str() == bet_outcome);

    // Batch-update bet statuses inside a transaction
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return (0, updated_scores),
    };

    for bet in &losers {
        sqlx::query("UPDATE bets SET status = 'lost' WHERE id = $1")
            .bind(bet.bet_id)
            .execute(&mut *tx)
            .await
            .ok();
    }

    for bet in &winners {
        let payout = bet.amount * bet.odds;

        sqlx::query("UPDATE bets SET status = 'won' WHERE id = $1")
            .bind(bet.bet_id)
            .execute(&mut *tx)
            .await
            .ok();

        if let Some(gid) = bet.group_id {
            sqlx::query(
                "UPDATE group_members SET balance = balance + $1 WHERE user_id = $2 AND group_id = $3",
            )
            .bind(payout)
            .bind(bet.user_id)
            .bind(gid)
            .execute(&mut *tx)
            .await
            .ok();
        }
    }

    if tx.commit().await.is_err() {
        return (0, updated_scores);
    }

    // Fan out bet-resolved emails after the tx commits. Only
    // opted-in users with an email on file get an attempt.
    for bet in &bets {
        let won = bet.prediction.as_str() == bet_outcome;
        tracing::info!(
            bet_id = %bet.bet_id,
            prediction = %bet.prediction,
            outcome = bet_outcome,
            won,
            "Bet resolved"
        );

        if bet.user_email.as_deref().is_none_or(str::is_empty) {
            tracing::debug!(bet_id = %bet.bet_id, "Skipping email (no email on file)");
            continue;
        }
        if !bet.email_notifications {
            tracing::debug!(bet_id = %bet.bet_id, "Skipping email (user opted out)");
            continue;
        }

        let payload = ResolvedBetPayload {
            user_email: bet.user_email.clone().unwrap_or_default(),
            user_locale: bet.user_locale.clone(),
            user_name: bet.user_name.clone(),
            home_team: m.home_team.clone(),
            away_team: m.away_team.clone(),
            prediction: bet.prediction.clone(),
            amount: bet.amount,
            odds: bet.odds,
            outcome: if won {
                BetOutcome::Won
            } else {
                BetOutcome::Lost
            },
            final_score: final_score.clone(),
        };
        if let Err(e) = email::send_bet_resolved(pool, client, bet.bet_id, payload).await {
            tracing::error!(bet_id = %bet.bet_id, "send_bet_resolved failed: {e}");
        }
    }

    (bets.len(), updated_scores)
}
