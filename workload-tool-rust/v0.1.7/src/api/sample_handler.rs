use axum::{extract::{Path, Query, State}, Json, Router, routing::get};
use serde::Deserialize;
use crate::db::DbPool;
use crate::error::Result;
use crate::models::ApiResponse;
use crate::models::PaginatedResponse;
use crate::models::sample::{SampleRecordResponse, SampleRecordCreate, SampleRecordUpdate};
use crate::repo::sample_repo;
use crate::service::sample_service;

#[derive(Deserialize)]
pub struct SampleQuery {
    pub group_id: Option<i64>,
    pub user_name: Option<String>,
    pub start: Option<String>,
    pub end: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/samples", get(list).post(create))
        .route("/api/samples/:id", get(get_one).put(update).delete(soft_delete))
        .route("/api/samples/restore/:id", axum::routing::post(restore))
        .with_state(pool)
}

async fn list(State(pool): State<DbPool>, Query(q): Query<SampleQuery>) -> Result<Json<ApiResponse<PaginatedResponse<SampleRecordResponse>>>> {
    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(50).min(500);
    let (items, total) = sample_repo::list(&pool, q.group_id, q.user_name.as_deref(), q.start.as_deref(), q.end.as_deref(), page, page_size)?;
    Ok(Json(ApiResponse::ok(PaginatedResponse { items, total, page, page_size })))
}

async fn get_one(State(pool): State<DbPool>, Path(id): Path<i64>) -> Result<Json<ApiResponse<SampleRecordResponse>>> {
    Ok(Json(ApiResponse::ok(sample_repo::get_by_id(&pool, id)?)))
}

async fn create(State(pool): State<DbPool>, Json(body): Json<SampleRecordCreate>) -> Result<Json<ApiResponse<SampleRecordResponse>>> {
    Ok(Json(ApiResponse::ok(sample_service::create(&pool, &body)?)))
}

async fn update(State(pool): State<DbPool>, Path(id): Path<i64>, Json(body): Json<SampleRecordUpdate>) -> Result<Json<ApiResponse<SampleRecordResponse>>> {
    Ok(Json(ApiResponse::ok(sample_service::update(&pool, id, &body)?)))
}

async fn soft_delete(State(pool): State<DbPool>, Path(id): Path<i64>) -> Result<Json<ApiResponse<()>>> {
    sample_service::delete(&pool, id)?;
    Ok(Json(ApiResponse::ok_msg("删除成功")))
}

async fn restore(State(pool): State<DbPool>, Path(id): Path<i64>) -> Result<Json<ApiResponse<SampleRecordResponse>>> {
    Ok(Json(ApiResponse::ok(sample_repo::restore(&pool, id)?)))
}
