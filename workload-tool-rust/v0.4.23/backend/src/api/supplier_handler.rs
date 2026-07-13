//! 供应商管理 API。

use axum::{extract::{Path, State, Json}, Router, routing::{get, post, put, delete as delete_route}};

use crate::db::DbPool;
use crate::error::Result;
use crate::models::ApiResponse;
use crate::models::supplier::{SupplierCreate, SupplierUpdate};
use crate::middleware::auth::AuthedUser;
use crate::repo;

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/suppliers", get(list).post(create))
        .route("/api/suppliers/:id", get(get_one).put(update).delete(delete))
        .with_state(pool)
}

async fn list(State(pool): State<DbPool>, user: AuthedUser) -> Result<Json<ApiResponse<Vec<crate::models::supplier::Supplier>>>> {
    user.require("supplier:read")?;
    Ok(Json(ApiResponse::ok(repo::purchase_repo::list_suppliers(&pool)?)))
}

async fn get_one(State(pool): State<DbPool>, user: AuthedUser, Path(id): Path<i64>) -> Result<Json<ApiResponse<crate::models::supplier::Supplier>>> {
    user.require("supplier:read")?;
    Ok(Json(ApiResponse::ok(repo::purchase_repo::get_supplier(&pool, id)?)))
}

async fn create(State(pool): State<DbPool>, user: AuthedUser, Json(body): Json<SupplierCreate>) -> Result<Json<ApiResponse<crate::models::supplier::Supplier>>> {
    user.require("supplier:write")?;
    Ok(Json(ApiResponse::ok(repo::purchase_repo::create_supplier(&pool, &body, &user.username)?)))
}

async fn update(State(pool): State<DbPool>, user: AuthedUser, Path(id): Path<i64>, Json(body): Json<SupplierUpdate>) -> Result<Json<ApiResponse<crate::models::supplier::Supplier>>> {
    user.require("supplier:write")?;
    Ok(Json(ApiResponse::ok(repo::purchase_repo::update_supplier(&pool, id, &body, &user.username)?)))
}

async fn delete(State(pool): State<DbPool>, user: AuthedUser, Path(id): Path<i64>) -> Result<Json<ApiResponse<()>>> {
    user.require("supplier:write")?;
    repo::purchase_repo::soft_delete_supplier(&pool, id, &user.username)?;
    Ok(Json(ApiResponse::ok_msg("供应商已删除")))
}
