mod common;

use serde_json::json;
use uuid::Uuid;

/// `parse_finished_match` accepts the v4 the-odds-api `/scores` payload
/// shape: `scores` is an array of `{name, score}` where `score` is a
/// string. The parser matches names against `home_team` / `away_team`
/// (verbatim) so a bet on Flamengo wins against a Vasco prediction when
/// the API echoes the same strings. Test exercises the full pipeline by
/// seeding the event so the UPDATE matches a row.
#[tokio::test]
async fn parse_finished_match_handles_v4_scores_shape() {
    let (_router, pool) = common::app().await;

    // Seed the event so the UPDATE in `process_scores` matches a row.
    let external_id = "v4-shape-match";
    sqlx::query(
        "INSERT INTO events (id, external_id, home_team, away_team, championship, start_time, status)
         VALUES (gen_random_uuid(), $1, 'Grêmio', 'Sao Paulo', 'Brazil Série A', NOW() - INTERVAL '3 hours', 'scheduled')",
    )
    .bind(external_id)
    .execute(&pool)
    .await
    .unwrap();

    let mock = json!([{
        "id": external_id,
        "sport_key": "soccer_brazil_campeonato",
        "sport_title": "Brazil Série A",
        "commence_time": "2026-08-08T19:00:00Z",
        "completed": true,
        "home_team": "Grêmio",
        "away_team": "Sao Paulo",
        "scores": [
            {"name": "Grêmio", "score": "2"},
            {"name": "Sao Paulo", "score": "1"}
        ],
        "last_update": "2026-08-08T21:20:53Z"
    }]);

    let result = sobrou_nada_pro_bet::routes::admin::process_scores(&pool, &mock)
        .await
        .unwrap();
    let body = result.0;
    assert_eq!(body["updated_scores"], 1);
    assert_eq!(body["resolved"], 0);

    // Confirm the row was updated.
    let scores: (Option<i32>, Option<i32>) =
        sqlx::query_as("SELECT home_score, away_score FROM events WHERE external_id = $1")
            .bind(external_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(scores, (Some(2), Some(1)));
}

/// A completed match with the wrong shapes (object-style scores, missing
/// score field, non-numeric score) must be skipped via `process_scores`,
/// not panic.
#[tokio::test]
async fn parse_finished_match_skips_malformed_shapes() {
    let (_router, pool) = common::app().await;

    // Object-style scores — the v3 shape. Parser skips it.
    let mock = json!([{
        "id": "v3-shape",
        "home_team": "A",
        "away_team": "B",
        "completed": true,
        "scores": {"home_score": 1, "away_score": 0}
    }]);
    let result = sobrou_nada_pro_bet::routes::admin::process_scores(&pool, &mock)
        .await
        .unwrap();
    assert_eq!(result.0["updated_scores"], 0);

    // Completed:false — skipped.
    let mock = json!([{
        "id": "not-done",
        "home_team": "A",
        "away_team": "B",
        "completed": false,
        "scores": [
            {"name": "A", "score": "0"},
            {"name": "B", "score": "0"}
        ]
    }]);
    let result = sobrou_nada_pro_bet::routes::admin::process_scores(&pool, &mock)
        .await
        .unwrap();
    assert_eq!(result.0["updated_scores"], 0);

    // Score missing the expected team name (diacritic drift) — skipped.
    let mock = json!([{
        "id": "wrong-name",
        "home_team": "Grêmio",
        "away_team": "Sao Paulo",
        "completed": true,
        "scores": [
            {"name": "Gremio", "score": "2"},
            {"name": "Sao Paulo", "score": "1"}
        ]
    }]);
    let result = sobrou_nada_pro_bet::routes::admin::process_scores(&pool, &mock)
        .await
        .unwrap();
    assert_eq!(result.0["updated_scores"], 0);
}

#[tokio::test]
async fn sync_events_from_mock() {
    let (_, pool) = common::app().await;

    let mock = json!([{
        "id": "mock-match-1",
        "home_team": "Flamengo",
        "away_team": "Vasco",
        "sport_title": "Brazil Série A",
        "commence_time": "2026-08-10T19:00:00Z",
        "bookmakers": [{
            "markets": [{
                "key": "h2h",
                "outcomes": [
                    {"name": "Flamengo", "price": 1.5},
                    {"name": "Vasco", "price": 4.0},
                    {"name": "Draw", "price": 3.0}
                ]
            }]
        }]
    }]);

    let result = sobrou_nada_pro_bet::routes::admin::process_odds(&pool, &mock)
        .await
        .unwrap();
    let body = result.0;
    assert_eq!(body["inserted"], 1);
    assert_eq!(body["total_fetched"], 1);

    // Verify it's in the DB
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM events WHERE external_id = 'mock-match-1')",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(exists);
}

#[tokio::test]
async fn resolve_bets_from_mock() {
    let (_, pool) = common::app().await;

    // Seed: create user + group + bet
    let user_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let event_id = Uuid::new_v4();
    let bet_id = Uuid::new_v4();

    sqlx::query("INSERT INTO users (id, username, email, google_id) VALUES ($1, 'Tester', 't@t.com', 'g-1')")
        .bind(user_id)
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO beta_allowlist (email) VALUES ('t@t.com')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO groups (id, name, invite_code, owner_id) VALUES ($1, 'G', 'abc12345', $2)",
    )
    .bind(group_id)
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO group_members (group_id, user_id, balance) VALUES ($1, $2, 900)")
        .bind(group_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO events (id, external_id, home_team, away_team, championship, start_time, status, home_odds, draw_odds, away_odds)
         VALUES ($1, 'mock-match-2', 'Flamengo', 'Vasco', 'Brasileirão', NOW() - INTERVAL '3 hours', 'scheduled', 1.5, 3.0, 4.0)",
    )
    .bind(event_id).execute(&pool).await.unwrap();
    sqlx::query(
        "INSERT INTO bets (id, user_id, group_id, event_id, prediction, amount, odds, status)
         VALUES ($1, $2, $3, $4, 'home_win', 100, 1.5, 'pending')",
    )
    .bind(bet_id)
    .bind(user_id)
    .bind(group_id)
    .bind(event_id)
    .execute(&pool)
    .await
    .unwrap();

    // Resolve: Flamengo won 2-0
    let mock = json!([{
        "id": "mock-match-2",
        "home_team": "Flamengo",
        "away_team": "Vasco",
        "completed": true,
        "scores": [
            {"name": "Flamengo", "score": "2"},
            {"name": "Vasco", "score": "0"}
        ]
    }]);

    let result = sobrou_nada_pro_bet::routes::admin::process_scores(&pool, &mock)
        .await
        .unwrap();
    let body = result.0;
    assert_eq!(body["resolved"], 1);
    assert_eq!(body["updated_scores"], 1);

    // Verify bet was resolved as won
    let status: String = sqlx::query_scalar("SELECT status::TEXT FROM bets WHERE id = $1")
        .bind(bet_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "won");

    // Verify balance was updated (900 + 150 payout = 1050)
    let balance: f64 = sqlx::query_scalar(
        "SELECT balance FROM group_members WHERE group_id = $1 AND user_id = $2",
    )
    .bind(group_id)
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(balance, 1050.0);

    // Verify the bet-resolved email was attempted (MAILGUN_API_KEY
    // is unset in tests, so the no-op path stamps notified_at).
    let notified_at: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT notified_at FROM bets WHERE id = $1")
            .bind(bet_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(notified_at.is_some(), "notified_at should be set");
}

/// A user with `email_notifications = false` is filtered out at the
/// SQL layer, so their bet still gets resolved (balance updated) but
/// `notified_at` stays NULL — no send attempt was made.
#[tokio::test]
async fn resolve_bets_skips_opted_out_users() {
    let (_, pool) = common::app().await;

    let user_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let event_id = Uuid::new_v4();
    let bet_id = Uuid::new_v4();

    sqlx::query("INSERT INTO users (id, username, email, google_id, email_notifications) VALUES ($1, 'Tester', 't@t.com', 'g-opted-out', FALSE)")
        .bind(user_id)
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO beta_allowlist (email) VALUES ('t@t.com')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO groups (id, name, invite_code, owner_id) VALUES ($1, 'G', 'opt12345', $2)",
    )
    .bind(group_id)
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO group_members (group_id, user_id, balance) VALUES ($1, $2, 1000)")
        .bind(group_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO events (id, external_id, home_team, away_team, championship, start_time, status, home_odds, draw_odds, away_odds)
         VALUES ($1, 'opt-match', 'Flamengo', 'Vasco', 'Brasileirão', NOW() - INTERVAL '3 hours', 'scheduled', 1.5, 3.0, 4.0)",
    )
    .bind(event_id).execute(&pool).await.unwrap();
    sqlx::query(
        "INSERT INTO bets (id, user_id, group_id, event_id, prediction, amount, odds, status)
         VALUES ($1, $2, $3, $4, 'home_win', 100, 1.5, 'pending')",
    )
    .bind(bet_id)
    .bind(user_id)
    .bind(group_id)
    .bind(event_id)
    .execute(&pool)
    .await
    .unwrap();

    let mock = json!([{
        "id": "opt-match",
        "home_team": "Flamengo",
        "away_team": "Vasco",
        "completed": true,
        "scores": [
            {"name": "Flamengo", "score": "2"},
            {"name": "Vasco", "score": "0"}
        ]
    }]);

    let result = sobrou_nada_pro_bet::routes::admin::process_scores(&pool, &mock)
        .await
        .unwrap();
    let body = result.0;
    assert_eq!(body["resolved"], 1);

    // The bet still resolves (status flips to 'won'), but notified_at
    // stays NULL because the SQL filter excludes opted-out users.
    let status: String = sqlx::query_scalar("SELECT status::TEXT FROM bets WHERE id = $1")
        .bind(bet_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "won");

    let notified_at: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT notified_at FROM bets WHERE id = $1")
            .bind(bet_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(
        notified_at.is_none(),
        "opted-out users must not get notified_at"
    );
}

/// The retry pass only stamps `notified_at` on bets where it's NULL.
/// If we manually clear it (simulating a failed send), the next
/// `process_scores` run (even with empty input) retries the email.
#[tokio::test]
async fn resolve_bets_retries_unnotified_on_subsequent_run() {
    let (_, pool) = common::app().await;

    let user_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let event_id = Uuid::new_v4();
    let bet_id = Uuid::new_v4();

    sqlx::query("INSERT INTO users (id, username, email, google_id) VALUES ($1, 'Tester', 'retry@t.com', 'g-retry')")
        .bind(user_id).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO beta_allowlist (email) VALUES ('retry@t.com')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO groups (id, name, invite_code, owner_id) VALUES ($1, 'G', 'rty12345', $2)",
    )
    .bind(group_id)
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO group_members (group_id, user_id, balance) VALUES ($1, $2, 1000)")
        .bind(group_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO events (id, external_id, home_team, away_team, championship, start_time, status, home_odds, draw_odds, away_odds, home_score, away_score)
         VALUES ($1, 'retry-match', 'Flamengo', 'Vasco', 'Brasileirão', NOW() - INTERVAL '3 hours', 'finished', 1.5, 3.0, 4.0, 2, 0)",
    )
    .bind(event_id).execute(&pool).await.unwrap();
    sqlx::query(
        "INSERT INTO bets (id, user_id, group_id, event_id, prediction, amount, odds, status, notified_at)
         VALUES ($1, $2, $3, $4, 'home_win', 100, 1.5, 'won', NULL)",
    )
    .bind(bet_id).bind(user_id).bind(group_id).bind(event_id)
    .execute(&pool).await.unwrap();

    // Empty scores payload — only the retry pass will fire.
    let result = sobrou_nada_pro_bet::routes::admin::process_scores(&pool, &json!([]))
        .await
        .unwrap();
    let body = result.0;
    assert_eq!(body["resolved"], 0);

    // The retry pass should have stamped notified_at.
    let notified_at: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT notified_at FROM bets WHERE id = $1")
            .bind(bet_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(notified_at.is_some(), "retry pass should stamp notified_at");

    // Running again does NOT change notified_at (idempotent).
    let first_ts = notified_at.unwrap();
    let result = sobrou_nada_pro_bet::routes::admin::process_scores(&pool, &json!([]))
        .await
        .unwrap();
    let _ = result;
    let notified_at: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT notified_at FROM bets WHERE id = $1")
            .bind(bet_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(notified_at, Some(first_ts), "second run is a no-op");
}

/// A user with no email on file is filtered out by the SQL — same
/// outcome as opt-out (no `notified_at`), but the balance still updates.
#[tokio::test]
async fn resolve_bets_skips_users_with_no_email() {
    let (_, pool) = common::app().await;

    let user_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let event_id = Uuid::new_v4();
    let bet_id = Uuid::new_v4();

    sqlx::query("INSERT INTO users (id, username, google_id) VALUES ($1, 'NoMail', 'g-nomail')")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO groups (id, name, invite_code, owner_id) VALUES ($1, 'G', 'nom12345', $2)",
    )
    .bind(group_id)
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO group_members (group_id, user_id, balance) VALUES ($1, $2, 500)")
        .bind(group_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO events (id, external_id, home_team, away_team, championship, start_time, status, home_odds, draw_odds, away_odds)
         VALUES ($1, 'nomail-match', 'Flamengo', 'Vasco', 'Brasileirão', NOW() - INTERVAL '3 hours', 'scheduled', 1.5, 3.0, 4.0)",
    )
    .bind(event_id).execute(&pool).await.unwrap();
    sqlx::query(
        "INSERT INTO bets (id, user_id, group_id, event_id, prediction, amount, odds, status)
         VALUES ($1, $2, $3, $4, 'home_win', 100, 1.5, 'pending')",
    )
    .bind(bet_id)
    .bind(user_id)
    .bind(group_id)
    .bind(event_id)
    .execute(&pool)
    .await
    .unwrap();

    let mock = json!([{
        "id": "nomail-match",
        "home_team": "Flamengo",
        "away_team": "Vasco",
        "completed": true,
        "scores": [
            {"name": "Flamengo", "score": "2"},
            {"name": "Vasco", "score": "0"}
        ]
    }]);

    let result = sobrou_nada_pro_bet::routes::admin::process_scores(&pool, &mock)
        .await
        .unwrap();
    assert_eq!(result.0["resolved"], 1);

    let status: String = sqlx::query_scalar("SELECT status::TEXT FROM bets WHERE id = $1")
        .bind(bet_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "won");

    let notified_at: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT notified_at FROM bets WHERE id = $1")
            .bind(bet_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(
        notified_at.is_none(),
        "users with no email must not get notified_at"
    );
}

/// When `process_odds` inserts new events, opted-in users with a NULL
/// `new_events_notified_at` get a digest and the timestamp gets stamped.
#[tokio::test]
async fn sync_events_sends_digest_to_opted_in_users() {
    let (_, pool) = common::app().await;

    let user_id = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, username, email, google_id) VALUES ($1, 'Tester', 'digest@t.com', 'g-digest')")
        .bind(user_id).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO beta_allowlist (email) VALUES ('digest@t.com')")
        .execute(&pool)
        .await
        .unwrap();

    let mock = json!([{
        "id": "fresh-match",
        "home_team": "Palmeiras",
        "away_team": "Corinthians",
        "sport_title": "Brasileirão",
        "commence_time": "2026-08-15T19:00:00Z",
        "bookmakers": [{
            "markets": [{
                "key": "h2h",
                "outcomes": [
                    {"name": "Palmeiras", "price": 1.5},
                    {"name": "Corinthians", "price": 4.0},
                    {"name": "Draw", "price": 3.0}
                ]
            }]
        }]
    }]);

    let result = sobrou_nada_pro_bet::routes::admin::process_odds(&pool, &mock)
        .await
        .unwrap();
    let body = result.0;
    assert_eq!(body["inserted"], 1);

    // Opted-in user with NULL digest stamp should now have it stamped.
    let stamped: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT new_events_notified_at FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(
        stamped.is_some(),
        "digest should be stamped for opted-in user"
    );
}
