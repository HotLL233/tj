use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json, Router,
};
use crate::db::DbPool;
use crate::error::{AppError, Result};
use crate::models::rd_record_column::{RdRecordColumn, RdRecordColumnUpdate};
use crate::models::ApiResponse;
use crate::repo::rd_record_column_repo;
use crate::service::auth_service;
use crate::repo::audit_repo;

fn require_admin(headers: &HeaderMap) -> Result<auth_service::Claims> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::Validation("未提供登录凭证".into()))?;
    let claims = auth_service::verify_token(token)?;
    if !claims.is_admin {
        return Err(AppError::Forbidden("需要管理员权限".into()));
    }
    Ok(claims)
}

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/rd-record-columns", axum::routing::get(list))
        .route("/api/rd-record-columns/:id", axum::routing::put(update))
        .with_state(pool)
}

/// GET /api/rd-record-columns — 列出全部研发送样列配置
async fn list(
    State(pool): State<DbPool>,
) -> Result<Json<ApiResponse<Vec<RdRecordColumn>>>> {
    let items = rd_record_column_repo::list_all(&pool)?;
    Ok(Json(ApiResponse::ok(items)))
}

/// PUT /api/rd-record-columns/:id — 更新列宽/显隐
async fn update(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(body): Json<RdRecordColumnUpdate>,
) -> Result<Json<ApiResponse<RdRecordColumn>>> {
    let claims = require_admin(&headers)?;
    let item = rd_record_column_repo::update(&pool, id, &body)?;
    // 审计
    let conn = pool.get()?;
    let detail = format!("更新研发送样列配置「{}」: width={}, show_in_list={}, show_in_form={}",
        item.label, item.width, item.show_in_list, item.show_in_form);
    audit_repo::log_on_conn_with_module(
        &conn, "update", "rd_record_columns", Some(id), &claims.username, &detail, "rd",
    )?;
    Ok(Json(ApiResponse::ok(item)))
}
