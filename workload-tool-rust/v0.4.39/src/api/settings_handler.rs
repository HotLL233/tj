use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json, Router,
};
use crate::db::DbPool;
use crate::error::{AppError, Result};
use crate::models::settings::{SettingUpdate, SystemSetting};
use crate::models::ApiResponse;
use crate::repo::settings_repo;
use crate::service::auth_service;
use std::sync::Arc;

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/settings", axum::routing::get(list))
        .route(
            "/api/settings/:key",
            axum::routing::get(get_by_key).put(upsert),
        )
        .with_state(pool)
}

/// 从 HeaderMap 中提取 JWT claims
fn extract_claims_from_headers(headers: &HeaderMap) -> Result<auth_service::Claims> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::Validation("未提供登录凭证".into()))?;
    auth_service::verify_token(token)
}

/// GET /api/settings — 获取所有系统设置
async fn list(State(pool): State<DbPool>) -> Result<Json<ApiResponse<Vec<SystemSetting>>>> {
    let settings = settings_repo::get_all(&pool)?;
    Ok(Json(ApiResponse::ok(settings)))
}

/// GET /api/settings/:key — 获取单个系统设置
async fn get_by_key(
    State(pool): State<DbPool>,
    Path(key): Path<String>,
) -> Result<Json<ApiResponse<SystemSetting>>> {
    let setting = settings_repo::get(&pool, &key)?
        .ok_or_else(|| AppError::NotFound(format!("设置 '{}' 不存在", key)))?;
    Ok(Json(ApiResponse::ok(setting)))
}

/// PUT /api/settings/:key — 更新系统设置（需管理员权限）
async fn upsert(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Path(key): Path<String>,
    Json(body): Json<SettingUpdate>,
) -> Result<Json<ApiResponse<SystemSetting>>> {
    // 鉴权：检查管理员权限
    let claims = extract_claims_from_headers(&headers)?;
    if !claims.is_admin && !claims.permissions.iter().any(|p| p == "*" || p == "manage:settings") {
        return Err(AppError::Forbidden("需要管理员权限才能修改系统设置".into()));
    }

    let value_str = serde_json::to_string(&body.value)
        .map_err(|e| AppError::Internal(format!("JSON 序列化失败: {}", e)))?;

    settings_repo::upsert(&pool, &key, &value_str, None)?;

    // 审计日志
    let conn = pool.get().map_err(AppError::Pool)?;
    conn.execute(
        "INSERT INTO audit_log (action, table_name, record_id, user_name, detail, module)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            "update",
            "system_settings",
            0,
            claims.username,
            format!("更新系统设置: {} = {}", key, &value_str[..value_str.len().min(200)]),
            "shared",
        ],
    )?;

    let setting = settings_repo::get(&pool, &key)?
        .ok_or_else(|| AppError::Internal("保存后读取失败".into()))?;

    Ok(Json(ApiResponse::ok(setting)))
}
