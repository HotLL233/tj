use axum::{extract::{Query, State}, Json, Router, routing::get};
use serde::Deserialize;
use crate::db::DbPool;
use crate::error::Result;
use crate::models::ApiResponse;
use crate::models::audit::AuditLogResponse;
use crate::models::PaginatedResponse;
use crate::repo::audit_repo;

#[derive(Deserialize)]
pub struct AuditQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/", get(list))
        .with_state(pool)
}

async fn list(
    State(pool): State<DbPool>,
    Query(q): Query<AuditQuery>,
) -> Result<Json<ApiResponse<PaginatedResponse<AuditLogResponse>>>> {
    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(50).min(500);
    let (items, total) = audit_repo::list(&pool, page, page_size)?;
    Ok(Json(ApiResponse::ok(PaginatedResponse { items, total, page, page_size })))
}
