use axum::{extract::{Path, Query, State}, Json, Router, routing::get};
use serde::Deserialize;
use crate::db::DbPool;
use crate::error::Result;
use crate::models::ApiResponse;
use crate::models::PaginatedResponse;
use crate::models::record::{RecordCreate, RecordResponse, RecordUpdate};
use crate::repo::record_repo;
use crate::service::record_service;

#[derive(Deserialize)]
pub struct RecordQuery {
    pub project_id: Option<i64>,
    pub group_id: Option<i64>,
    pub user_name: Option<String>,
    pub start: Option<String>,
    pub end: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub include_deleted: Option<bool>,
}

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/records", get(list).post(create))
        .route("/api/records/:id", axum::routing::put(update).delete(soft_delete))
        .route("/api/records/restore/:id", axum::routing::post(restore))
        .route("/api/records/by-user/:user_name", axum::routing::delete(delete_by_user))
        .with_state(pool)
}

async fn list(State(pool): State<DbPool>, Query(q): Query<RecordQuery>) -> Result<Json<ApiResponse<PaginatedResponse<RecordResponse>>>> {
    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(50).min(500);
    let (items, total) = record_repo::list(
        &pool, q.project_id, q.group_id, q.user_name.as_deref(), q.start.as_deref(), q.end.as_deref(),
        page, page_size, q.include_deleted.unwrap_or(false),
    )?;
    Ok(Json(ApiResponse::ok(PaginatedResponse { items, total, page, page_size })))
}

async fn create(State(pool): State<DbPool>, Json(body): Json<RecordCreate>) -> Result<Json<ApiResponse<RecordResponse>>> {
    // Service layer: validates quantity > 0 and project existence
    Ok(Json(ApiResponse::ok(record_service::create_record(&pool, &body)?)))
}

async fn update(State(pool): State<DbPool>, Path(id): Path<i64>, Json(body): Json<RecordUpdate>) -> Result<Json<ApiResponse<RecordResponse>>> {
    // Service layer: validates not-deleted, change detection
    Ok(Json(ApiResponse::ok(record_service::update_record(&pool, id, &body, "system")?)))
}

async fn soft_delete(State(pool): State<DbPool>, Path(id): Path<i64>) -> Result<Json<ApiResponse<()>>> {
    // Service layer: validates record exists and not already deleted
    record_service::delete_record(&pool, id, "system")?;
    Ok(Json(ApiResponse::ok_msg("删除成功")))
}

async fn restore(State(pool): State<DbPool>, Path(id): Path<i64>) -> Result<Json<ApiResponse<RecordResponse>>> {
    Ok(Json(ApiResponse::ok(record_repo::restore(&pool, id, "system")?)))
}

#[derive(Deserialize)]
pub struct DeleteByUserQuery {
    pub start: Option<String>,
    pub end: Option<String>,
}

async fn delete_by_user(
    State(pool): State<DbPool>,
    Path(user_name): Path<String>,
    Query(q): Query<DeleteByUserQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let count = record_repo::delete_by_user(&pool, &user_name, q.start.as_deref(), q.end.as_deref())?;
    Ok(Json(ApiResponse::ok(serde_json::json!({"deleted_count": count}))))
}
