pub mod group_handler;
pub mod project_handler;
pub mod record_handler;
pub mod stats_handler;
pub mod export_handler;
pub mod audit_handler;

use axum::Router;
use crate::db::DbPool;

pub fn api_router(pool: DbPool) -> Router {
    Router::new()
        .nest("/api/groups", group_handler::router(pool.clone()))
        .nest("/api/projects", project_handler::router(pool.clone()))
        .nest("/api/records", record_handler::router(pool.clone()))
        .nest("/api/stats", stats_handler::router(pool.clone()))
        .nest("/api/export", export_handler::router(pool.clone()))
        .nest("/api/audit-logs", audit_handler::router(pool))
}
