use axum::{extract::{Path, State, Json}, Router, routing::{get, post, put, delete}};

use crate::db::DbPool;
use crate::error::Result;
use crate::models::ApiResponse;
use crate::models::role::{RoleCreate, RoleUpdate, RoleWithPermissions};
use crate::middleware::auth::AuthedUser;
use crate::repo;

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/roles", get(list).post(create))
        .route("/api/roles/:id", get(get_one).put(update).delete(delete_one))
        .route("/api/roles/:id/permissions", get(get_perms).put(set_perms))
        .with_state(pool)
}

async fn list(
    State(pool): State<DbPool>,
    user: AuthedUser,
) -> Result<Json<ApiResponse<Vec<RoleWithPermissions>>>> {
    user.require("role:manage")?;
    Ok(Json(ApiResponse::ok(repo::role_repo::list(&pool)?)))
}

async fn get_one(
    State(pool): State<DbPool>,
    user: AuthedUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<RoleWithPermissions>>> {
    user.require("role:manage")?;
    let role = repo::role_repo::get_by_id(&pool, id)?;
    let perms = repo::role_repo::get_permissions(&pool, id)?;
    Ok(Json(ApiResponse::ok(RoleWithPermissions {
        id: role.id, name: role.name, description: role.description, is_system: role.is_system, sort_order: role.sort_order, permissions: perms,
    })))
}

async fn create(
    State(pool): State<DbPool>,
    user: AuthedUser,
    Json(body): Json<RoleCreate>,
) -> Result<Json<ApiResponse<RoleWithPermissions>>> {
    user.require("role:manage")?;
    Ok(Json(ApiResponse::ok(repo::role_repo::create(&pool, &body)?)))
}

async fn update(
    State(pool): State<DbPool>,
    user: AuthedUser,
    Path(id): Path<i64>,
    Json(body): Json<RoleUpdate>,
) -> Result<Json<ApiResponse<RoleWithPermissions>>> {
    user.require("role:manage")?;
    Ok(Json(ApiResponse::ok(repo::role_repo::update(&pool, id, &body)?)))
}

async fn delete_one(
    State(pool): State<DbPool>,
    user: AuthedUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<()>>> {
    user.require("role:manage")?;
    repo::role_repo::delete(&pool, id)?;
    Ok(Json(ApiResponse::ok_msg("角色已删除")))
}

async fn get_perms(
    State(pool): State<DbPool>,
    user: AuthedUser,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<Vec<String>>>> {
    user.require("role:manage")?;
    Ok(Json(ApiResponse::ok(repo::role_repo::get_permissions(&pool, id)?)))
}

async fn set_perms(
    State(pool): State<DbPool>,
    user: AuthedUser,
    Path(id): Path<i64>,
    Json(body): Json<Vec<String>>,
) -> Result<Json<ApiResponse<RoleWithPermissions>>> {
    user.require("role:manage")?;
    Ok(Json(ApiResponse::ok(repo::role_repo::set_permissions(&pool, id, &body)?)))
}
