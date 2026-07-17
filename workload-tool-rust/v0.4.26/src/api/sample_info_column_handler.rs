use axum::{
    extract::{Path, State},
    Json, Router,
};
use crate::db::DbPool;
use crate::error::Result;
use crate::models::sample_info_column::*;
use crate::models::ApiResponse;
use crate::repo::sample_info_column_repo;

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/sample-info/columns", axum::routing::get(list).post(create))
        .route("/api/sample-info/columns/active", axum::routing::get(list_active))
        .route(
            "/api/sample-info/columns/sort",
            axum::routing::put(reorder),
        )
        .route(
            "/api/sample-info/columns/:id",
            axum::routing::put(update).delete(delete),
        )
        .with_state(pool)
}

async fn list(
    State(pool): State<DbPool>,
) -> Result<Json<ApiResponse<Vec<SampleInfoColumn>>>> {
    let items = sample_info_column_repo::list_all(&pool)?;
    Ok(Json(ApiResponse::ok(items)))
}

async fn list_active(
    State(pool): State<DbPool>,
) -> Result<Json<ApiResponse<Vec<SampleInfoColumn>>>> {
    let items = sample_info_column_repo::list_active(&pool)?;
    Ok(Json(ApiResponse::ok(items)))
}

async fn create(
    State(pool): State<DbPool>,
    Json(body): Json<ColumnCreate>,
) -> Result<Json<ApiResponse<SampleInfoColumn>>> {
    let item = sample_info_column_repo::create(&pool, &body)?;
    Ok(Json(ApiResponse::ok(item)))
}

async fn update(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
    Json(body): Json<ColumnUpdate>,
) -> Result<Json<ApiResponse<SampleInfoColumn>>> {
    let item = sample_info_column_repo::update(&pool, id, &body)?;
    Ok(Json(ApiResponse::ok(item)))
}

async fn delete(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<()>>> {
    sample_info_column_repo::soft_delete(&pool, id)?;
    Ok(Json(ApiResponse::ok_msg("删除成功")))
}

async fn reorder(
    State(pool): State<DbPool>,
    Json(body): Json<ColumnReorder>,
) -> Result<Json<ApiResponse<Vec<SampleInfoColumn>>>> {
    let items = sample_info_column_repo::reorder(&pool, &body)?;
    Ok(Json(ApiResponse::ok(items)))
}
