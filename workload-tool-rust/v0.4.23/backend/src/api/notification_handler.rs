//! 通知 / 站内信 API：我的通知、未读计数、标记已读、管理员发送。

use axum::{extract::{Path, Query, State, Json}, Router, routing::{get, post}};

use crate::db::DbPool;
use crate::error::Result;
use crate::models::ApiResponse;
use crate::models::notification::NotificationCreate;
use crate::middleware::auth::AuthedUser;
use crate::repo;

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/notifications", get(list_mine).post(send))
        .route("/api/notifications/unread-count", get(unread_count))
        .route("/api/notifications/:id/read", post(mark_read))
        .route("/api/notifications/read-all", post(mark_all))
        .with_state(pool)
}

#[derive(serde::Deserialize)]
struct NoteQuery { unread_only: Option<bool> }

async fn list_mine(State(pool): State<DbPool>, user: AuthedUser, Query(q): Query<NoteQuery>) -> Result<Json<ApiResponse<Vec<crate::models::notification::NotificationResponse>>>> {
    user.require("notification:read")?;
    Ok(Json(ApiResponse::ok(repo::notification_repo::list_for(&pool, &user.username, q.unread_only.unwrap_or(false))?)))
}

async fn unread_count(State(pool): State<DbPool>, user: AuthedUser) -> Result<Json<ApiResponse<i64>>> {
    user.require("notification:read")?;
    Ok(Json(ApiResponse::ok(repo::notification_repo::unread_count(&pool, &user.username)?)))
}

async fn mark_read(State(pool): State<DbPool>, user: AuthedUser, Path(id): Path<i64>) -> Result<Json<ApiResponse<()>>> {
    user.require("notification:read")?;
    repo::notification_repo::mark_read(&pool, id, &user.username)?;
    Ok(Json(ApiResponse::ok_msg("已标记已读")))
}

async fn mark_all(State(pool): State<DbPool>, user: AuthedUser) -> Result<Json<ApiResponse<i64>>> {
    user.require("notification:read")?;
    let n = repo::notification_repo::mark_all_read(&pool, &user.username)?;
    Ok(Json(ApiResponse::ok(n)))
}

async fn send(State(pool): State<DbPool>, user: AuthedUser, Json(body): Json<NotificationCreate>) -> Result<Json<ApiResponse<()>>> {
    user.require("notification:manage")?;
    repo::notification_repo::create(&pool, &body, &user.username)?;
    Ok(Json(ApiResponse::ok_msg("通知已发送")))
}
