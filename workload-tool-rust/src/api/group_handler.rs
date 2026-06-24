use axum::{extract::{Path, State}, Json, Router, routing::get};
use crate::db::DbPool;
use crate::error::Result;
use crate::models::ApiResponse;
use crate::models::group::{GroupCreate, GroupResponse, GroupUpdate};
use crate::repo::group_repo;

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", axum::routing::put(update).delete(delete))
        .with_state(pool)
}

async fn list(State(pool): State<DbPool>) -> Result<Json<ApiResponse<Vec<GroupResponse>>>> {
    let items = group_repo::list(&pool)?;
    Ok(Json(ApiResponse::ok(items)))
}

async fn create(State(pool): State<DbPool>, Json(body): Json<GroupCreate>) -> Result<Json<ApiResponse<GroupResponse>>> {
    let item = group_repo::create(&pool, &body)?;
    Ok(Json(ApiResponse::ok(item)))
}

async fn update(State(pool): State<DbPool>, Path(id): Path<i64>, Json(body): Json<GroupUpdate>) -> Result<Json<ApiResponse<GroupResponse>>> {
    let item = group_repo::update(&pool, id, &body)?;
    Ok(Json(ApiResponse::ok(item)))
}

async fn delete(State(pool): State<DbPool>, Path(id): Path<i64>) -> Result<Json<ApiResponse<()>>> {
    group_repo::delete(&pool, id)?;
    Ok(Json(ApiResponse::ok_msg("删除成功")))
}
