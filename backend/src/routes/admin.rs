use axum::{
    extract::{FromRequestParts, State},
    http::request::Parts,
    Json,
};
use serde_json::{json, Value};
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
        let expected = std::env::var("ADMIN_TOKEN")
            .map_err(|_| AppError::Internal("ADMIN_TOKEN not set".into()))?;

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

// ── Handlers ───────────────────────────────────────────

pub async fn sync_events(
    _auth: AdminAuth,
    State(pool): State<PgPool>,
) -> Result<Json<Value>, AppError> {
    let api_key = std::env::var("ODDS_API_KEY")
        .map_err(|_| AppError::Internal("ODDS_API_KEY not set".into()))?;

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

    let empty = vec![];
    let matches = raw.as_array().unwrap_or(&empty);
    let mut inserted = 0;

    for m in matches {
        let external_id = m["id"].as_str().unwrap_or("").to_string();
        if external_id.is_empty() {
            continue;
        }

        let home_team = m["home_team"].as_str().unwrap_or("TBD");
        let away_team = m["away_team"].as_str().unwrap_or("TBD");
        let championship = m["sport_title"].as_str().unwrap_or("Brazil Série A");

        let start_time_str = m["commence_time"].as_str().unwrap_or("");
        let start_time = chrono::DateTime::parse_from_rfc3339(start_time_str)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now());

        let status = if start_time > chrono::Utc::now() {
            "scheduled"
        } else {
            "live"
        };

        let mut home_odds: Option<f64> = None;
        let mut draw_odds: Option<f64> = None;
        let mut away_odds: Option<f64> = None;

        if let Some(bookmakers) = m["bookmakers"].as_array() {
            for bm in bookmakers {
                if let Some(markets) = bm["markets"].as_array() {
                    for market in markets {
                        if market["key"].as_str() == Some("h2h") {
                            if let Some(outcomes) = market["outcomes"].as_array() {
                                for o in outcomes {
                                    let name = o["name"].as_str().unwrap_or("");
                                    let price = o["price"].as_f64();
                                    if name == home_team {
                                        home_odds = price;
                                    } else if name == away_team {
                                        away_odds = price;
                                    } else if name.to_lowercase() == "draw" {
                                        draw_odds = price;
                                    }
                                }
                            }
                            break;
                        }
                    }
                }
                if home_odds.is_some() {
                    break;
                }
            }
        }

        tracing::info!(%home_team, %away_team, ?home_odds, ?draw_odds, ?away_odds, "Match with odds");

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
        .bind(&external_id)
        .bind(home_team)
        .bind(away_team)
        .bind(championship)
        .bind(start_time)
        .bind(status)
        .bind::<Option<i32>>(None)
        .bind::<Option<i32>>(None)
        .bind(home_odds)
        .bind(draw_odds)
        .bind(away_odds)
        .bind(Some(m.clone()))
        .execute(&pool)
        .await;

        if let Ok(r) = result {
            inserted += r.rows_affected() as usize;
        }
    }

    tracing::info!(inserted, total = matches.len(), "Admin sync complete");

    Ok(Json(json!({
        "inserted": inserted,
        "total_fetched": matches.len()
    })))
}
