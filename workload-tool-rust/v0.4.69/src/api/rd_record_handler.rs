use axum::{extract::{Path, Query, State}, http::HeaderMap, Json, Router, routing::get};
use serde::Deserialize;
use crate::db::DbPool;
use crate::error::Result;
use crate::models::ApiResponse;
use crate::models::PaginatedResponse;
use crate::models::rd_record::{RdRecordCreate, RdRecordResponse};
use crate::models::record::{RecordCreate, RecordUpdate};
use crate::repo::rd_record_repo;
use crate::repo::audit_repo;
use crate::service::rd_record_service;
use crate::service::auth_service;

fn extract_claims_username(headers: &HeaderMap) -> String {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .and_then(|token| crate::service::auth_service::verify_token(token).ok())
        .map(|c| c.username)
        .unwrap_or_else(|| "system".to_string())
}

fn extract_claims(headers: &HeaderMap) -> Option<auth_service::Claims> {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .and_then(|token| auth_service::verify_token(token).ok())
}

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

async fn create(State(pool): State<DbPool>, headers: HeaderMap, Json(body): Json<RdRecordCreate>) -> Result<Json<ApiResponse<RdRecordResponse>>> {
    let record = RecordCreate {
        project_id: body.project_id,
        method_id: body.method_id,
        user_name: body.user_name,
        quantity: body.quantity,
        recorded_at: body.recorded_at,
        group_id: body.group_id,
        multiplier: None,
        division_id: body.division_id,
        high_item: None,
    };
    let user_name = extract_claims_username(&headers);
    // Service layer: validates quantity > 0 and project existence
    let result = rd_record_service::create_record(&pool, &record, body.batch_no, body.notes)?;
    let conn = pool.get()?;
    audit_repo::log_on_conn_with_module(
        &conn, "create", "rd_work_records", Some(result.id),
        &user_name, &format!("创建研发送样记录#{}", result.id), "rd",
    )?;
    Ok(Json(ApiResponse::ok(result)))
}

async fn update(State(pool): State<DbPool>, headers: HeaderMap, Path(id): Path<i64>, Json(body): Json<RecordUpdate>) -> Result<Json<ApiResponse<RdRecordResponse>>> {
    // v0.4.34: 已取样记录不可修改
    if rd_record_repo::is_sampled(&pool, id)? {
        return Err(crate::error::AppError::Forbidden("该记录已取样，不可修改".to_string()));
    }
    let user_name = extract_claims_username(&headers);
    let result = rd_record_service::update_record(&pool, id, &body, &user_name)?;
    let conn = pool.get()?;
    audit_repo::log_on_conn_with_module(
        &conn, "update", "rd_work_records", Some(id),
        &user_name, &format!("更新研发送样记录#{}", id), "rd",
    )?;
    Ok(Json(ApiResponse::ok(result)))
}

async fn soft_delete(State(pool): State<DbPool>, headers: HeaderMap, Path(id): Path<i64>) -> Result<Json<ApiResponse<()>>> {
    let user_name = extract_claims_username(&headers);
    rd_record_service::delete_record(&pool, id, &user_name)?;
    let conn = pool.get()?;
    audit_repo::log_on_conn_with_module(
        &conn, "delete", "rd_work_records", Some(id),
        &user_name, &format!("删除研发送样记录#{}", id), "rd",
    )?;
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

async fn sample(State(pool): State<DbPool>, headers: HeaderMap, Path(id): Path<i64>) -> Result<Json<ApiResponse<RdRecordResponse>>> {
    // 权限检查
    let claims = extract_claims(&headers);
    if let Some(ref c) = claims {
        if !c.permissions.contains(&"sample:collect".to_string()) && !c.is_admin {
            return Err(crate::error::AppError::Forbidden("无取样权限".to_string()));
        }
    }
    let user_name = extract_claims_username(&headers);
    let result = rd_record_service::sample(&pool, id, &user_name)?;
    let conn = pool.get()?;
    audit_repo::log_on_conn_with_module(
        &conn, "sample", "rd_work_records", Some(id),
        &user_name, &format!("研发送样记录#{} 取样人「{}」", id, user_name), "rd",
    )?;
    Ok(Json(ApiResponse::ok(result)))
}
