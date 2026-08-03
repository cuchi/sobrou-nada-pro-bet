use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use rand::Rng;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

pub mod admin;

use crate::auth::AuthUser;
use crate::error::AppError;
use crate::models::{
    AuthResponse, Bet, BetStatus, BetWithUser, CreateBetRequest, CreateGroupRequest, Event,
    GoogleAuthRequest, GoogleTokenClaims, Group, GroupMember, GroupWithBalance, JwtClaims,
    LeaderboardEntry, PublicUser, User,
};

/// GET /health
pub async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

// ── Auth ──────────────────────────────────────────────

#[tracing::instrument(skip(pool, body))]
pub async fn google_login(
    State(pool): State<PgPool>,
    Json(body): Json<GoogleAuthRequest>,
) -> Result<Json<Value>, AppError> {
    let client_id = std::env::var("GOOGLE_CLIENT_ID").map_err(|e| {
        tracing::error!("GOOGLE_CLIENT_ID not set: {e}");
        AppError::Internal("GOOGLE_CLIENT_ID not set".into())
    })?;

    tracing::debug!("Calling Google tokeninfo...");

    let token = body.credential;
    let url = format!("https://oauth2.googleapis.com/tokeninfo?id_token={token}");
    let resp = reqwest::get(&url).await.map_err(|e| {
        tracing::error!("Failed to reach Google tokeninfo: {e}");
        AppError::Unauthorized(format!("Failed to verify token: {e}"))
    })?;

    let status = resp.status();
    let raw_body = resp.text().await.map_err(|e| {
        tracing::error!("Failed to read Google response body: {e}");
        AppError::Unauthorized(format!("Failed to read token response: {e}"))
    })?;

    tracing::debug!(%status, %raw_body, "Google tokeninfo response");

    let google_claims: GoogleTokenClaims = serde_json::from_str(&raw_body).map_err(|e| {
        tracing::error!(%raw_body, "Failed to parse Google tokeninfo: {e}");
        AppError::Unauthorized(format!("Invalid token response: {e}"))
    })?;

    if let Some(aud) = &google_claims.aud {
        if aud != &client_id {
            tracing::error!(expected=%client_id, got=%aud, "Token audience mismatch");
            return Err(AppError::Unauthorized("Token audience mismatch".into()));
        }
    }
    if google_claims.sub.is_empty() {
        tracing::error!("Google token has empty sub claim");
        return Err(AppError::Unauthorized("Invalid Google token".into()));
    }

    let email = google_claims
        .email
        .filter(|_| google_claims.email_verified.as_deref() == Some("true"))
        .ok_or_else(|| {
            tracing::error!("Google email not verified or missing");
            AppError::Unauthorized("Email not verified by Google".into())
        })?;

    tracing::info!(%email, google_sub=%google_claims.sub, "Google user verified");

    // Beta allowlist check
    let is_allowed: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM beta_allowlist WHERE email = $1)")
            .bind(&email)
            .fetch_one(&pool)
            .await
            .map_err(|e| {
                tracing::error!("Failed to check beta allowlist: {e}");
                AppError::Internal(format!("Database error: {e}"))
            })?;

    if !is_allowed {
        tracing::warn!(%email, "User not on beta allowlist");
        return Err(AppError::Forbidden(
            "You're not on the beta list yet. This app is currently invite-only.".into(),
        ));
    }

    // Upsert user
    let user: User = sqlx::query_as(
        r#"INSERT INTO users (id, username, email, google_id, avatar_url)
           VALUES (gen_random_uuid(), $1, $2, $3, $4)
           ON CONFLICT (google_id) DO UPDATE SET
               username   = COALESCE(EXCLUDED.username, users.username),
               email      = EXCLUDED.email,
               avatar_url = COALESCE(EXCLUDED.avatar_url, users.avatar_url)
           RETURNING *"#,
    )
    .bind(&google_claims.name)
    .bind(&email)
    .bind(&google_claims.sub)
    .bind(&google_claims.picture)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error upserting user: {e}");
        AppError::Internal(format!("Database error: {e}"))
    })?;

    tracing::info!(user_id=%user.id, "User upserted");

    let jwt_secret = std::env::var("JWT_SECRET").map_err(|e| {
        tracing::error!("JWT_SECRET not set: {e}");
        AppError::Internal("JWT_SECRET not set".into())
    })?;

    let now = Utc::now().timestamp() as usize;
    let claims = JwtClaims {
        sub: user.id.to_string(),
        email: email.clone(),
        exp: now + 86400 * 7,
        iat: now,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("JWT encoding error: {e}")))?;

    Ok(Json(json!(AuthResponse {
        token,
        user: PublicUser::from(user),
    })))
}

/// GET /api/auth/me
pub async fn me(
    AuthUser { id, .. }: AuthUser,
    State(pool): State<PgPool>,
) -> Result<Json<Value>, AppError> {
    let user: User = sqlx::query_as("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    let groups: Vec<GroupWithBalance> = sqlx::query_as(
        r#"SELECT g.*, gm.balance
           FROM groups g
           JOIN group_members gm ON gm.group_id = g.id
           WHERE gm.user_id = $1
           ORDER BY g.name"#,
    )
    .bind(id)
    .fetch_all(&pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(json!({
        "user": PublicUser::from(user),
        "groups": groups,
    })))
}

// ── Groups ────────────────────────────────────────────

/// POST /api/groups — create a group (caller becomes owner + member)
pub async fn create_group(
    AuthUser { id: user_id, .. }: AuthUser,
    State(pool): State<PgPool>,
    Json(body): Json<CreateGroupRequest>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let invite_code = generate_invite_code();

    let group: Group = sqlx::query_as(
        r#"INSERT INTO groups (id, name, invite_code, owner_id)
           VALUES (gen_random_uuid(), $1, $2, $3)
           RETURNING *"#,
    )
    .bind(&body.name)
    .bind(&invite_code)
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::Internal(format!("Failed to create group: {e}")))?;

    // Auto-join as member with default balance
    sqlx::query("INSERT INTO group_members (group_id, user_id) VALUES ($1, $2)")
        .bind(group.id)
        .bind(user_id)
        .execute(&pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to join group: {e}")))?;

    tracing::info!(%group.id, %group.name, "Group created");

    Ok((StatusCode::CREATED, Json(json!(group))))
}

/// GET /api/groups — list groups the user belongs to
pub async fn list_my_groups(
    AuthUser { id: user_id, .. }: AuthUser,
    State(pool): State<PgPool>,
) -> Result<Json<Value>, AppError> {
    let groups: Vec<GroupWithBalance> = sqlx::query_as(
        r#"SELECT g.*, gm.balance
           FROM groups g
           JOIN group_members gm ON gm.group_id = g.id
           WHERE gm.user_id = $1
           ORDER BY g.name"#,
    )
    .bind(user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(json!(groups)))
}

/// GET /api/groups/:id — group details with members
pub async fn get_group(
    AuthUser { id: user_id, .. }: AuthUser,
    State(pool): State<PgPool>,
    Path(group_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    // Ensure caller is a member
    let _: GroupMember =
        sqlx::query_as("SELECT * FROM group_members WHERE group_id = $1 AND user_id = $2")
            .bind(group_id)
            .bind(user_id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Group not found or you're not a member".into()))?;

    let group: Group = sqlx::query_as("SELECT * FROM groups WHERE id = $1")
        .bind(group_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Group not found".into()))?;

    let members: Vec<GroupMember> =
        sqlx::query_as("SELECT * FROM group_members WHERE group_id = $1 ORDER BY joined_at")
            .bind(group_id)
            .fetch_all(&pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(json!({ "group": group, "members": members })))
}

/// GET /api/groups/:id/invite — get invite code (owner only)
pub async fn get_invite(
    AuthUser { id: user_id, .. }: AuthUser,
    State(pool): State<PgPool>,
    Path(group_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let group: Group = sqlx::query_as("SELECT * FROM groups WHERE id = $1")
        .bind(group_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Group not found".into()))?;

    if group.owner_id != user_id {
        return Err(AppError::Forbidden(
            "Only the group owner can view the invite code".into(),
        ));
    }

    Ok(Json(json!({ "invite_code": group.invite_code })))
}

/// POST /api/groups/:id/regenerate-invite — rotate invite code (owner only)
pub async fn regenerate_invite(
    AuthUser { id: user_id, .. }: AuthUser,
    State(pool): State<PgPool>,
    Path(group_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let group: Group = sqlx::query_as("SELECT * FROM groups WHERE id = $1")
        .bind(group_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Group not found".into()))?;

    if group.owner_id != user_id {
        return Err(AppError::Forbidden(
            "Only the group owner can regenerate the invite code".into(),
        ));
    }

    let new_code = generate_invite_code();
    sqlx::query("UPDATE groups SET invite_code = $1 WHERE id = $2")
        .bind(&new_code)
        .bind(group_id)
        .execute(&pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(json!({ "invite_code": new_code })))
}

/// POST /api/groups/join/:invite_code — join a group by invite code
pub async fn join_group(
    AuthUser { id: user_id, .. }: AuthUser,
    State(pool): State<PgPool>,
    Path(invite_code): Path<String>,
) -> Result<Json<Value>, AppError> {
    let group: Group = sqlx::query_as("SELECT * FROM groups WHERE invite_code = $1")
        .bind(&invite_code)
        .fetch_optional(&pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Invalid invite code".into()))?;

    // Check not already a member
    let already: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM group_members WHERE group_id = $1 AND user_id = $2)",
    )
    .bind(group.id)
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    if already {
        return Err(AppError::BadRequest("You're already in this group".into()));
    }

    sqlx::query("INSERT INTO group_members (group_id, user_id) VALUES ($1, $2)")
        .bind(group.id)
        .bind(user_id)
        .execute(&pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    tracing::info!(%group.id, %user_id, "User joined group");

    Ok(Json(json!({ "group": group })))
}

/// GET /api/groups/:id/leaderboard
pub async fn leaderboard(
    AuthUser { id: user_id, .. }: AuthUser,
    State(pool): State<PgPool>,
    Path(group_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    // Ensure caller is a member
    let _: GroupMember =
        sqlx::query_as("SELECT * FROM group_members WHERE group_id = $1 AND user_id = $2")
            .bind(group_id)
            .bind(user_id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("Group not found or you're not a member".into()))?;

    let entries: Vec<LeaderboardEntry> = sqlx::query_as(
        r#"SELECT u.id AS user_id, COALESCE(u.username, u.email) AS name,
                  u.email, u.avatar_url,
                  gm.balance + COALESCE(SUM(b.amount) FILTER (WHERE b.status = 'pending'), 0) AS balance,
                  COALESCE(SUM(b.amount) FILTER (WHERE b.status = 'pending'), 0) AS betted
           FROM group_members gm
           JOIN users u ON u.id = gm.user_id
           LEFT JOIN bets b ON b.user_id = gm.user_id AND b.group_id = gm.group_id AND b.status = 'pending'
           WHERE gm.group_id = $1
           GROUP BY u.id, u.username, u.email, u.avatar_url, gm.balance
           ORDER BY balance DESC"#,
    )
    .bind(group_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(json!(entries)))
}

// ── Events ────────────────────────────────────────────

#[derive(Deserialize)]
pub struct EventsQuery {
    status: Option<String>,
}

/// GET /api/events — list events, optionally filtered by status (comma-separated)
pub async fn list_events(
    _auth: AuthUser,
    State(pool): State<PgPool>,
    Query(query): Query<EventsQuery>,
) -> Result<Json<Value>, AppError> {
    let events: Vec<Event> = if let Some(status) = &query.status {
        // Split "scheduled,live" into ["scheduled", "live"]
        let statuses: Vec<&str> = status.split(',').map(|s| s.trim()).collect();
        // Use ANY() to match any of the given statuses
        sqlx::query_as("SELECT * FROM events WHERE status = ANY($1) ORDER BY start_time")
            .bind(&statuses)
            .fetch_all(&pool)
            .await
    } else {
        sqlx::query_as("SELECT * FROM events ORDER BY start_time")
            .fetch_all(&pool)
            .await
    }
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(json!(events)))
}

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

// ── Dev only ──────────────────────────────────────────

/// POST /api/dev/login — creates a random test user and returns a JWT.
/// Only works when ENVIRONMENT is not "production".
pub async fn dev_login(State(pool): State<PgPool>) -> Result<Json<Value>, AppError> {
    let is_prod = std::env::var("ENVIRONMENT")
        .map(|v| v == "production")
        .unwrap_or(false);

    if is_prod {
        return Err(AppError::NotFound("Not found".into()));
    }

    let random_id = Uuid::new_v4();
    let email = format!("test-{random_id}@dev.local");

    // Ensure the email is in the beta allowlist
    sqlx::query("INSERT INTO beta_allowlist (email) VALUES ($1) ON CONFLICT DO NOTHING")
        .bind(&email)
        .execute(&pool)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {e}")))?;

    // Upsert the test user
    let user: User = sqlx::query_as(
        r#"INSERT INTO users (id, username, email, google_id, avatar_url)
               VALUES (gen_random_uuid(), $1, $2, $3, $4)
               ON CONFLICT (google_id) DO NOTHING
               RETURNING *"#,
    )
    .bind(&format!("Tester {random_id}"))
    .bind(&email)
    .bind(&random_id.to_string())
    .bind::<Option<String>>(None)
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {e}")))?;

    // Generate JWT
    let jwt_secret = std::env::var("JWT_SECRET").map_err(|e| {
        tracing::error!("JWT_SECRET not set: {e}");
        AppError::Internal("JWT_SECRET not set".into())
    })?;

    let now = Utc::now().timestamp() as usize;
    let claims = JwtClaims {
        sub: user.id.to_string(),
        email: email.clone(),
        exp: now + 86400 * 7,
        iat: now,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("JWT encoding error: {e}")))?;

    tracing::info!(%email, user_id=%user.id, "Dev user logged in");

    Ok(Json(json!(AuthResponse {
        token,
        user: PublicUser::from(user),
    })))
}

// ── Helpers ───────────────────────────────────────────

fn generate_invite_code() -> String {
    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyz0123456789".chars().collect();
    let mut rng = rand::thread_rng();
    (0..8)
        .map(|_| chars[rng.gen_range(0..chars.len())])
        .collect()
}
