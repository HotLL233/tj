pub mod group_handler;
pub mod project_handler;
pub mod record_handler;
pub mod stats_handler;
pub mod export_handler;
pub mod audit_handler;

use axum::{Router, Json, routing::get};
use serde::Serialize;
use crate::db::DbPool;

#[derive(Serialize)]
struct VersionInfo { version: &'static str }

async fn version() -> Json<VersionInfo> {
    Json(VersionInfo { version: env!("CARGO_PKG_VERSION") })
}

pub fn api_router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/version", get(version))
        .nest("/api/groups", group_handler::router(pool.clone()))
        .nest("/api/projects", project_handler::router(pool.clone()))
        .nest("/api/records", record_handler::router(pool.clone()))
        .nest("/api/stats", stats_handler::router(pool.clone()))
        .nest("/api/export", export_handler::router(pool.clone()))
        .nest("/api/audit-logs", audit_handler::router(pool))
}
