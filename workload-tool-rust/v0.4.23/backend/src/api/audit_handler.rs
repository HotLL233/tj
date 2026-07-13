use axum::{extract::{Query, State}, Json, Router, routing::get};
use serde::Deserialize;

use crate::db::DbPool;
use crate::error::Result;
use crate::models::ApiResponse;
use crate::models::audit::AuditLogResponse;
use crate::models::PaginatedResponse;
use crate::repo::audit_repo;
use crate::middleware::auth::AuthedUser;

#[derive(Deserialize)]
pub struct AuditQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    /// 模块过滤：work=分析检测, rd=研发送样, shared=仅主数据, 具体模块名(如 instrument)=该模块；不传=全部
    pub module: Option<String>,
}

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/audit-logs", get(list))
        .with_state(pool)
}

async fn list(
    State(pool): State<DbPool>,
    user: AuthedUser,
    Query(q): Query<AuditQuery>,
) -> Result<Json<ApiResponse<PaginatedResponse<AuditLogResponse>>>> {
    // 审计日志需 audit:read 权限
    user.require("audit:read")?;
    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(50).min(500);
    let (items, total) = audit_repo::list(&pool, page, page_size, q.module.as_deref())?;
    Ok(Json(ApiResponse::ok(PaginatedResponse { items, total, page, page_size })))
}
