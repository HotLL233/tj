pub mod group_handler;
pub mod project_handler;
pub mod method_handler;
pub mod record_handler;
pub mod stats_handler;
pub mod export_handler;
pub mod export_data;
pub mod export_write;
pub mod audit_handler;
pub mod rd_record_handler;
pub mod rd_stats_handler;
pub mod rd_export_handler;
pub mod rd_export_data;
pub mod rd_export_preview_handler;
pub mod import_handler;
pub mod help_handler;
pub mod auth_handler;
pub mod backup_handler;
pub mod export_preview_handler;

use axum::{Router, Json, routing::get};
use serde::Serialize;
use crate::config::AppConfig;
use crate::db::DbPool;
use std::sync::Arc;

#[derive(Serialize)]
struct VersionInfo { version: &'static str }

#[utoipa::path(get, path = "/api/version", responses((status = 200, body = VersionInfo)))]
async fn version() -> Json<VersionInfo> {
    Json(VersionInfo { version: env!("CARGO_PKG_VERSION") })
}

pub fn api_router(pool: DbPool, config: Arc<AppConfig>) -> Router {
    Router::new()
        .route("/api/version", get(version))
        .merge(group_handler::router(pool.clone()))
        .merge(project_handler::router(pool.clone()))
        .merge(method_handler::router(pool.clone()))
        .merge(record_handler::router(pool.clone()))
        .merge(rd_record_handler::router(pool.clone()))
        .merge(rd_stats_handler::router(pool.clone()))
        .merge(rd_export_handler::router(pool.clone()))
        .merge(rd_export_preview_handler::router(pool.clone()))
        .merge(stats_handler::router(pool.clone()))
        .merge(export_handler::router(pool.clone()))
        .merge(import_handler::router(pool.clone()))
        .merge(audit_handler::router(pool.clone()))
        .merge(auth_handler::router(config.clone()))
        .merge(backup_handler::router(config))
        .merge(export_preview_handler::router(pool.clone()))
        .merge(help_handler::router(pool.clone()))
}
