use axum::{extract::{Path, Query, State}, Json, Router, routing::get};
use serde::Deserialize;
use crate::db::DbPool;
use crate::error::Result;
use crate::models::ApiResponse;
use crate::models::PaginatedResponse;
use crate::models::rd_record::{RdRecordResponse, RdSampleUpdate};
use crate::models::record::{RecordCreate, RecordUpdate};
use crate::repo::rd_record_repo;
use crate::service::rd_record_service;

#[derive(Deserialize)]
pub struct RecordQuery {
    pub project_id: Option<i64>,
    pub group_id: Option<i64>,
    pub user_name: Option<String>,
    pub division_id: Option<i64>,
    pub start: Option<String>,
    pub end: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub include_deleted: Option<bool>,
}

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/rd-records", get(list).post(create))
        .route("/api/rd-records/:id", axum::routing::put(update).delete(soft_delete))
        .route("/api/rd-records/:id/sample", axum::routing::put(sample))
        .route("/api/rd-records/restore/:id", axum::routing::post(restore))
        .route("/api/rd-records/by-user/:user_name", axum::routing::delete(delete_by_user))
        .with_state(pool)
}

async fn list(State(pool): State<DbPool>, Query(q): Query<RecordQuery>) -> Result<Json<ApiResponse<PaginatedResponse<RdRecordResponse>>>> {
    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(50).min(500);
    let (items, total) = rd_record_repo::list(
        &pool, q.project_id, q.group_id, q.user_name.as_deref(), q.division_id,
        q.start.as_deref(), q.end.as_deref(),
        page, page_size, q.include_deleted.unwrap_or(false),
    )?;
    Ok(Json(ApiResponse::ok(PaginatedResponse { items, total, page, page_size })))
}

async fn create(State(pool): State<DbPool>, Json(body): Json<RecordCreate>) -> Result<Json<ApiResponse<RdRecordResponse>>> {
    let record = RecordCreate {
        project_id: body.project_id,
        method_id: body.method_id,
        user_name: body.user_name,
        quantity: body.quantity,
        recorded_at: body.recorded_at,
        group_id: body.group_id,
        multiplier: body.multiplier,
        division_id: body.division_id,
    };
    // Service layer: validates quantity > 0 and project existence
    Ok(Json(ApiResponse::ok(rd_record_service::create_record(&pool, &record)?)))
}

async fn update(State(pool): State<DbPool>, Path(id): Path<i64>, Json(body): Json<RecordUpdate>) -> Result<Json<ApiResponse<RdRecordResponse>>> {
    // Service layer: validates not-deleted, change detection
    Ok(Json(ApiResponse::ok(rd_record_service::update_record(&pool, id, &body, "system")?)))
}

async fn soft_delete(State(pool): State<DbPool>, Path(id): Path<i64>) -> Result<Json<ApiResponse<()>>> {
    // Service layer: validates record exists and not already deleted
    rd_record_service::delete_record(&pool, id, "system")?;
    Ok(Json(ApiResponse::ok_msg("删除成功")))
}

async fn restore(State(pool): State<DbPool>, Path(id): Path<i64>) -> Result<Json<ApiResponse<RdRecordResponse>>> {
    Ok(Json(ApiResponse::ok(rd_record_repo::restore(&pool, id, "system")?)))
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
    let count = rd_record_repo::delete_by_user(&pool, &user_name, q.start.as_deref(), q.end.as_deref())?;
    Ok(Json(ApiResponse::ok(serde_json::json!({"deleted_count": count}))))
}

async fn sample(State(pool): State<DbPool>, Path(id): Path<i64>, Json(body): Json<RdSampleUpdate>) -> Result<Json<ApiResponse<RdRecordResponse>>> {
    Ok(Json(ApiResponse::ok(rd_record_service::sample(&pool, id, &body)?)))
}
