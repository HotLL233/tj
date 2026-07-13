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
pub mod pdf_render;
pub mod docx_parser;
pub mod pdf_parser;
pub mod article_handler;
pub mod auth_handler;
pub mod backup_handler;
pub mod export_preview_handler;

// v0.4.21: 鉴权 / RBAC / 运营模块
pub mod user_handler;
pub mod role_handler;
pub mod instrument_handler;
pub mod inventory_handler;
pub mod purchase_handler;
pub mod approval_handler;
pub mod notification_handler;
pub mod supplier_handler;

use axum::{Router, Json, routing::get};
use axum::middleware::from_fn_with_state;
use serde::Serialize;
use crate::config::AppConfig;
use crate::db::DbPool;
use crate::middleware::auth::{auth_middleware, AuthState};
use crate::models::ApiResponse;
use std::sync::Arc;

#[derive(Serialize)]
struct VersionInfo { version: &'static str }

#[utoipa::path(get, path = "/api/version", responses((status = 200, body = VersionInfo)))]
async fn version() -> Json<VersionInfo> {
    Json(VersionInfo { version: env!("CARGO_PKG_VERSION") })
}

/// Health check 端点（Docker HEALTHCHECK / 负载均衡探活）
async fn health_check() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse::ok("ok"))
}

/// 聚合全部 API 路由，并套用 JWT 鉴权中间件（白名单：/api/version、/api/health、/api/auth/login）。
pub fn api_router(pool: DbPool, config: Arc<AppConfig>) -> Router {
    // 解析 JWT 签名密钥（config > .jwt_secret 文件 > 随机生成）
    let jwt_secret = config.resolve_jwt_secret();
    let auth_state = Arc::new(AuthState { jwt_secret: jwt_secret.clone() });

    let inner = Router::new()
        .route("/api/version", get(version))
        .route("/api/health", get(health_check))
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
        .merge(auth_handler::router(pool.clone(), jwt_secret))
        .merge(user_handler::router(pool.clone()))
        .merge(role_handler::router(pool.clone()))
        .merge(instrument_handler::router(pool.clone()))
        .merge(inventory_handler::router(pool.clone()))
        .merge(supplier_handler::router(pool.clone()))
        .merge(purchase_handler::router(pool.clone()))
        .merge(approval_handler::router(pool.clone()))
        .merge(notification_handler::router(pool.clone()))
        .merge(backup_handler::router(config))
        .merge(export_preview_handler::router(pool.clone()))
        .merge(help_handler::router(pool.clone()))
        .merge(article_handler::router(pool.clone()));

    // 全局 JWT 鉴权层：所有 /api/* 经此后被校验，非 /api 路径放行
    inner.layer(from_fn_with_state(auth_state, auth_middleware))
}
