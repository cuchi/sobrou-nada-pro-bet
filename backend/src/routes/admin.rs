use axum::{
    Json,
    extract::{FromRequestParts, State},
    http::request::Parts,
};
use serde_json::{Value, json};
use sqlx::PgPool;

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
            .ok_or_else(|| AppError::Unauthorized("Missing X-Admin-Token header".into()))?;

        if provided != expected {
            return Err(AppError::Forbidden("Invalid admin token".into()));
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

struct FinishedMatch {
    external_id: String,
    home_score: i32,
    away_score: i32,
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
    let completed = m["completed"].as_bool().unwrap_or(false);
    if !completed {
        return None;
    }
    let scores = &m["scores"];
    let home = scores["home_score"].as_i64().map(|s| s as i32)?;
    let away = scores["away_score"].as_i64().map(|s| s as i32)?;
    Some(FinishedMatch {
        external_id,
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

    for m in &matches {
        let result = sqlx::query(
            r#"INSERT INTO events (external_id, home_team, away_team, championship, start_time, status, home_score, away_score, home_odds, draw_odds, away_odds, raw_data)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
               ON CONFLICT (external_id) DO UPDATE SET
                   status = EXCLUDED.status,
                   home_odds = EXCLUDED.home_odds,
                   draw_odds = EXCLUDED.draw_odds,
                   away_odds = EXCLUDED.away_odds,
                   start_time = EXCLUDED.start_time,
                   raw_data = EXCLUDED.raw_data"#,
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
        .execute(pool)
        .await;

        if let Ok(r) = result {
            inserted += r.rows_affected() as usize;
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

    for m in &finished {
        // Update event scores
        let result = sqlx::query(
            "UPDATE events SET status = 'finished', home_score = $1, away_score = $2 WHERE external_id = $3",
        )
        .bind(m.home_score)
        .bind(m.away_score)
        .bind(&m.external_id)
        .execute(pool)
        .await;

        if let Ok(ref r) = result {
            updated_scores += r.rows_affected() as usize;
        }

        // Fetch pending bets for this event
        let bet_outcome = outcome(m.home_score, m.away_score);

        let bets: Vec<(uuid::Uuid, uuid::Uuid, uuid::Uuid, String, f64, f64)> = sqlx::query_as(
            r#"SELECT b.id, b.user_id, b.group_id, b.prediction, b.amount, b.odds
               FROM bets b
               JOIN events e ON e.id = b.event_id
               WHERE e.external_id = $1 AND b.status = 'pending'"#,
        )
        .bind(&m.external_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        if bets.is_empty() {
            continue;
        }

        // Separate winners from losers
        let (winners, losers): (Vec<_>, Vec<_>) = bets
            .iter()
            .partition(|(_, _, _, prediction, _, _)| prediction.as_str() == bet_outcome);

        // Batch-update bet statuses inside a transaction
        let mut tx = pool
            .begin()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        for (bet_id, _, _, _, _, _) in &losers {
            sqlx::query("UPDATE bets SET status = 'lost' WHERE id = $1")
                .bind(bet_id)
                .execute(&mut *tx)
                .await
                .ok();
        }

        for (bet_id, user_id, group_id, _, amount, odds) in &winners {
            let payout = amount * odds;

            sqlx::query("UPDATE bets SET status = 'won' WHERE id = $1")
                .bind(bet_id)
                .execute(&mut *tx)
                .await
                .ok();

            sqlx::query(
                "UPDATE group_members SET balance = balance + $1 WHERE user_id = $2 AND group_id = $3",
            )
            .bind(payout)
            .bind(user_id)
            .bind(group_id)
            .execute(&mut *tx)
            .await
            .ok();
        }

        tx.commit()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        for (bet_id, _, _, prediction, _, _) in &bets {
            let won = prediction.as_str() == bet_outcome;
            tracing::info!(
                bet_id = %bet_id,
                prediction = %prediction,
                outcome = bet_outcome,
                won,
                "Bet resolved"
            );
        }

        resolved += bets.len();
    }

    tracing::info!(resolved, updated_scores, "Resolve complete");

    Ok(Json(json!({
        "resolved": resolved,
        "updated_scores": updated_scores,
    })))
}
