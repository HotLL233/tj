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
        .merge(group_handler::router(pool.clone()))
        .merge(project_handler::router(pool.clone()))
        .merge(record_handler::router(pool.clone()))
        .merge(stats_handler::router(pool.clone()))
        .merge(export_handler::router(pool.clone()))
        .merge(audit_handler::router(pool))
}
