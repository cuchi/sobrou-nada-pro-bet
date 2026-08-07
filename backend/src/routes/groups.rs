use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use rand::RngExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::{AppError, ErrorCode};
use crate::models::{CreateGroupRequest, Group, GroupMember, GroupWithBalance, LeaderboardEntry};

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
            .ok_or_else(|| AppError::legacy_not_found("Group not found or you're not a member"))?;

    let group: Group = sqlx::query_as("SELECT * FROM groups WHERE id = $1")
        .bind(group_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::legacy_not_found("Group not found"))?;

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
        .ok_or_else(|| AppError::legacy_not_found("Group not found"))?;

    if group.owner_id != user_id {
        return Err(AppError::legacy_forbidden(
            "Only the group owner can view the invite code",
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
        .ok_or_else(|| AppError::legacy_not_found("Group not found"))?;

    if group.owner_id != user_id {
        return Err(AppError::legacy_forbidden(
            "Only the group owner can regenerate the invite code",
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
        .ok_or_else(|| AppError::NotFound(ErrorCode::InvalidInviteCode, None))?;

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
        return Err(AppError::BadRequest(ErrorCode::AlreadyInGroup, None));
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
            .ok_or_else(|| AppError::legacy_not_found("Group not found or you're not a member"))?;

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
           ORDER BY balance DESC, betted DESC"#,
    )
    .bind(group_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(json!(entries)))
}

// ── Helpers ───────────────────────────────────────────

fn generate_invite_code() -> String {
    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyz0123456789".chars().collect();
    let mut rng = rand::rng();
    (0..32)
        .map(|_| chars[rng.random_range(0..chars.len())])
        .collect()
}
