use axum::{
    extract::{Path, State},
    Json, Router, routing::{get, put},
};
use crate::db::DbPool;
use crate::error::Result;
use crate::models::sample_info_type::{
    SampleInfoType, SampleInfoTypeCreate, SampleInfoTypeUpdate,
};
use crate::models::{ApiResponse};
use crate::repo::sample_info_type_repo;

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/sample-info-types", get(list).post(create))
        .route("/api/sample-info-types/all", get(list_all))
        .route("/api/sample-info-types/:id", put(update).delete(soft_delete))
        .with_state(pool)
}

async fn list(State(pool): State<DbPool>) -> Result<Json<ApiResponse<Vec<SampleInfoType>>>> {
    let items = sample_info_type_repo::list(&pool)?;
    Ok(Json(ApiResponse::ok(items)))
}

async fn list_all(State(pool): State<DbPool>) -> Result<Json<ApiResponse<Vec<SampleInfoType>>>> {
    let items = sample_info_type_repo::list_all(&pool)?;
    Ok(Json(ApiResponse::ok(items)))
}

async fn create(
    State(pool): State<DbPool>,
    Json(body): Json<SampleInfoTypeCreate>,
) -> Result<Json<ApiResponse<SampleInfoType>>> {
    let item = sample_info_type_repo::create(&pool, &body)?;
    Ok(Json(ApiResponse::ok(item)))
}

async fn update(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
    Json(body): Json<SampleInfoTypeUpdate>,
) -> Result<Json<ApiResponse<SampleInfoType>>> {
    let item = sample_info_type_repo::update(&pool, id, &body)?;
    Ok(Json(ApiResponse::ok(item)))
}

async fn soft_delete(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<()>>> {
    sample_info_type_repo::soft_delete(&pool, id)?;
    Ok(Json(ApiResponse::ok_msg("删除成功")))
}
