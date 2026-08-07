//! Seed the local dev database with fake events covering every status branch
//! the new EventPicker renders:
//!   - scheduled (various kickoff offsets → exercises the kickoffLabel ladder)
//!   - live (within the 2h match window)
//!   - finished (home win, draw, away win — within last 7 days)
//!   - finished (> 7 days ago, must be excluded from the recent-results UI)
//!   - cancelled
//!   - scheduled but window elapsed (exercises the derived
//!     `awaiting_result: true` path in the frontend)
//!
//! Usage:  cargo run --bin seed
//!
//! Runs migrations first, so it's safe to run against a fresh database.
//! Idempotent — re-running refreshes the rows in place by `external_id`.

use chrono::{Duration, Timelike, Utc};
use serde_json::json;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Honour the same env loading as the real app so DATABASE_URL resolves.
    let env = sobrou_nada_pro_bet::env::Env::load();
    // db::init runs pending migrations, so a fresh database is fine.
    let pool = sobrou_nada_pro_bet::db::init(&env.database_url).await;

    let now = Utc::now();

    let rows: Vec<SeedEvent> = vec![
        // ── scheduled (exercises kickoffLabel ladder) ──────────────────
        SeedEvent {
            external_id: "seed-sched-5m",
            home: "Flamengo",
            away: "Palmeiras",
            championship: "Brasileirão",
            offset: Duration::minutes(5),
            status: "scheduled",
            home_score: None,
            away_score: None,
            home_odds: Some(2.10),
            draw_odds: Some(3.20),
            away_odds: Some(3.50),
        },
        SeedEvent {
            external_id: "seed-sched-47m",
            home: "Corinthians",
            away: "São Paulo",
            championship: "Brasileirão",
            offset: Duration::minutes(47),
            status: "scheduled",
            home_score: None,
            away_score: None,
            home_odds: Some(2.40),
            draw_odds: Some(3.10),
            away_odds: Some(2.90),
        },
        SeedEvent {
            external_id: "seed-sched-3h14m",
            home: "Atlético Mineiro",
            away: "Cruzeiro",
            championship: "Brasileirão",
            offset: Duration::hours(3) + Duration::minutes(14),
            status: "scheduled",
            home_score: None,
            away_score: None,
            home_odds: Some(1.85),
            draw_odds: Some(3.40),
            away_odds: Some(4.20),
        },
        SeedEvent {
            external_id: "seed-sched-tomorrow",
            home: "Fluminense",
            away: "Botafogo",
            championship: "Brasileirão",
            offset: Duration::days(1),
            status: "scheduled",
            home_score: None,
            away_score: None,
            home_odds: Some(2.60),
            draw_odds: Some(3.00),
            away_odds: Some(2.70),
        },
        SeedEvent {
            external_id: "seed-sched-weekday",
            home: "Vasco",
            away: "Santos",
            championship: "Brasileirão",
            offset: Duration::days(5),
            status: "scheduled",
            home_score: None,
            away_score: None,
            home_odds: Some(2.30),
            draw_odds: Some(3.10),
            away_odds: Some(3.10),
        },
        SeedEvent {
            external_id: "seed-sched-far-future",
            home: "Grêmio",
            away: "Internacional",
            championship: "Brasileirão",
            offset: Duration::days(14),
            status: "scheduled",
            home_score: None,
            away_score: None,
            home_odds: Some(2.05),
            draw_odds: Some(3.25),
            away_odds: Some(3.55),
        },
        // ── live (within the 2h match window) ─────────────────────────
        SeedEvent {
            external_id: "seed-live-30m",
            home: "Bahia",
            away: "Vasco",
            championship: "Brasileirão",
            offset: -Duration::minutes(30),
            status: "live",
            home_score: None,
            away_score: None,
            home_odds: Some(2.50),
            draw_odds: Some(2.90),
            away_odds: Some(3.00),
        },
        SeedEvent {
            external_id: "seed-live-edge",
            home: "Santos",
            away: "Fluminense",
            championship: "Brasileirão",
            // 1h 50m ago — still within the 2h MATCH_DURATION window
            offset: -Duration::minutes(110),
            status: "live",
            home_score: None,
            away_score: None,
            home_odds: Some(2.20),
            draw_odds: Some(3.20),
            away_odds: Some(3.20),
        },
        // ── finished (within 7 days, exercises Recent Results) ─────────
        SeedEvent {
            external_id: "seed-finished-home-win",
            home: "Flamengo",
            away: "Corinthians",
            championship: "Brasileirão",
            offset: -Duration::days(2),
            status: "finished",
            home_score: Some(2),
            away_score: Some(0),
            home_odds: Some(1.80),
            draw_odds: Some(3.40),
            away_odds: Some(4.50),
        },
        SeedEvent {
            external_id: "seed-finished-draw",
            home: "Palmeiras",
            away: "São Paulo",
            championship: "Brasileirão",
            offset: -Duration::days(1),
            status: "finished",
            home_score: Some(1),
            away_score: Some(1),
            home_odds: Some(2.10),
            draw_odds: Some(3.00),
            away_odds: Some(3.60),
        },
        SeedEvent {
            external_id: "seed-finished-away-win",
            home: "Botafogo",
            away: "Atlético Mineiro",
            championship: "Brasileirão",
            offset: -Duration::days(3),
            status: "finished",
            home_score: Some(0),
            away_score: Some(1),
            home_odds: Some(2.25),
            draw_odds: Some(3.20),
            away_odds: Some(3.10),
        },
        SeedEvent {
            external_id: "seed-finished-stale",
            // 10 days ago — proves the UI's 7-day recency filter excludes it
            home: "Cruzeiro",
            away: "Internacional",
            championship: "Brasileirão",
            offset: -Duration::days(10),
            status: "finished",
            home_score: Some(3),
            away_score: Some(0),
            home_odds: Some(1.95),
            draw_odds: Some(3.30),
            away_odds: Some(4.00),
        },
        // ── scheduled but window elapsed (exercises "awaiting result") ─
        SeedEvent {
            external_id: "seed-sched-unresolved",
            // 3h ago — past the 2h match window but no resolve call yet, so
            // backend derives `status: "finished"` + `awaiting_result: true`.
            home: "Athletico Paranaense",
            away: "Cuiabá",
            championship: "Brasileirão",
            offset: -Duration::hours(3),
            status: "scheduled",
            home_score: None,
            away_score: None,
            home_odds: Some(2.15),
            draw_odds: Some(3.10),
            away_odds: Some(3.40),
        },
        // ── cancelled ─────────────────────────────────────────────
        SeedEvent {
            external_id: "seed-cancelled",
            home: "Grêmio",
            away: "Bahia",
            championship: "Brasileirão",
            offset: -Duration::days(2),
            status: "cancelled",
            home_score: None,
            away_score: None,
            home_odds: Some(2.10),
            draw_odds: Some(3.20),
            away_odds: Some(3.40),
        },
    ];

    for r in &rows {
        let start = snap_to_half_hour(now + r.offset);
        let id = Uuid::new_v4();

        sqlx::query(
            r#"INSERT INTO events
                       (id, external_id, home_team, away_team, championship,
                        start_time, status, home_score, away_score,
                        home_odds, draw_odds, away_odds, raw_data)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                   ON CONFLICT (external_id) DO UPDATE SET
                       home_team     = EXCLUDED.home_team,
                       away_team     = EXCLUDED.away_team,
                       championship  = EXCLUDED.championship,
                       start_time    = EXCLUDED.start_time,
                       status        = EXCLUDED.status,
                       home_score    = EXCLUDED.home_score,
                       away_score    = EXCLUDED.away_score,
                       home_odds     = EXCLUDED.home_odds,
                       draw_odds     = EXCLUDED.draw_odds,
                       away_odds     = EXCLUDED.away_odds,
                       raw_data      = EXCLUDED.raw_data"#,
        )
        .bind(id)
        .bind(r.external_id)
        .bind(r.home)
        .bind(r.away)
        .bind(r.championship)
        .bind(start)
        .bind(r.status)
        .bind(r.home_score)
        .bind(r.away_score)
        .bind(r.home_odds)
        .bind(r.draw_odds)
        .bind(r.away_odds)
        .bind(json!({
            "id": r.external_id,
            "home_team": r.home,
            "away_team": r.away,
            "sport_title": r.championship,
            "commence_time": start.to_rfc3339(),
            "seed": true,
        }))
        .execute(&pool)
        .await?;
    }

    let total: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE external_id LIKE 'seed-%'")
            .fetch_one(&pool)
            .await?;

    // Per-status counts are the ground truth for what's in the DB. The
    // picker's `live` count will be a derived superset of these (any
    // scheduled row whose start_time is now inside the 2h match window also
    // renders as live) — but that's a frontend concern, not a seed one.
    let counts: Vec<(String, i64)> = sqlx::query_as(
        "SELECT status, COUNT(*)::BIGINT
           FROM events
          WHERE external_id LIKE 'seed-%'
          GROUP BY status
          ORDER BY status",
    )
    .fetch_all(&pool)
    .await?;

    println!("✅ Seeded {} events.", total);
    for (status, n) in counts {
        println!("   {status}: {n}");
    }

    Ok(())
}

struct SeedEvent {
    external_id: &'static str,
    home: &'static str,
    away: &'static str,
    championship: &'static str,
    offset: Duration,
    status: &'static str,
    home_score: Option<i32>,
    away_score: Option<i32>,
    home_odds: Option<f64>,
    draw_odds: Option<f64>,
    away_odds: Option<f64>,
}

/// Snap a UTC datetime to the nearest :00 or :30 mark so kickoff times
/// in dev data look realistic. Below :30 → :00, otherwise → :30.
fn snap_to_half_hour(dt: chrono::DateTime<Utc>) -> chrono::DateTime<Utc> {
    let target_minute = if dt.minute() < 30 { 0 } else { 30 };
    dt.with_minute(target_minute)
        .and_then(|d| d.with_second(0))
        .and_then(|d| d.with_nanosecond(0))
        .unwrap_or(dt)
}
