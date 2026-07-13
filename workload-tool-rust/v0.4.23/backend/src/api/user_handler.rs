use axum::{extract::{Path, State, Json}, Router, routing::{get, post, put, delete}};

use crate::db::DbPool;
use crate::error::Result;
use crate::models::ApiResponse;
use crate::models::user::{UserCreate, UserPublic, UserUpdate, ResetPasswordRequest};
use crate::middleware::auth::AuthedUser;
use crate::repo;
use crate::service::auth_service;

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/users", get(list).post(create))
        .route("/api/users/:id", get(get_one).put(update).delete(soft_delete))
        .route("/api/users/:id/reset-password", post(reset_password))
        .with_state(pool)
}

async fn list(
    State(pool): State<DbPool>,
    user: AuthedUser,
) -> Result<Json<ApiResponse<Vec<UserPublic>>>> {
    user.require("user:manage")?;
    let items = repo::user_repo::list(&pool)?;
    Ok(Json(ApiResponse::ok(items)))
}

async fn get_one(
    State(pool): State<DbPool>,
    user: AuthedUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<UserPublic>>> {
    user.require("user:manage")?;
    let item = repo::user_repo::get_public(&pool, id)?;
    Ok(Json(ApiResponse::ok(item)))
}

async fn create(
    State(pool): State<DbPool>,
    user: AuthedUser,
    Json(body): Json<UserCreate>,
) -> Result<Json<ApiResponse<UserPublic>>> {
    user.require("user:manage")?;
    let hash = auth_service::hash_password(&body.password)?;
    let item = repo::user_repo::create(&pool, &body, &hash)?;
    Ok(Json(ApiResponse::ok(item)))
}

async fn update(
    State(pool): State<DbPool>,
    user: AuthedUser,
    Path(id): Path<i64>,
    Json(body): Json<UserUpdate>,
) -> Result<Json<ApiResponse<UserPublic>>> {
    user.require("user:manage")?;
    let item = repo::user_repo::update(&pool, id, &body)?;
    Ok(Json(ApiResponse::ok(item)))
}

async fn soft_delete(
    State(pool): State<DbPool>,
    user: AuthedUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<()>>> {
    user.require("user:manage")?;
    // 禁止删除自己
    if user.uid == id {
        return Err(crate::error::AppError::Validation("不能删除当前登录账号".into()));
    }
    repo::user_repo::soft_delete(&pool, id)?;
    Ok(Json(ApiResponse::ok_msg("用户已停用")))
}

async fn reset_password(
    State(pool): State<DbPool>,
    user: AuthedUser,
    Path(id): Path<i64>,
    Json(body): Json<ResetPasswordRequest>,
) -> Result<Json<ApiResponse<()>>> {
    user.require("user:manage")?;
    let hash = auth_service::hash_password(&body.new_password)?;
    // 管理员重置后清除强制改密标记，用户可用新密码直接登录
    repo::user_repo::set_password(&pool, id, &hash, true)?;
    Ok(Json(ApiResponse::ok_msg("密码已重置")))
}
