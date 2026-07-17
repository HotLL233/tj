use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json, Router,
};
use crate::db::DbPool;
use crate::error::Result;
use crate::models::ApiResponse;
use crate::models::role::{PermissionDef, PERMISSIONS, RoleCreate, RolePermissionSet, RoleUpdate};
use crate::repo::role_repo;
use crate::repo::audit_repo;
use crate::service::auth_service;

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/roles", axum::routing::get(list).post(create))
        .route(
            "/api/roles/:id",
            axum::routing::put(update).delete(delete_one),
        )
        .route("/api/roles/:id/permissions", axum::routing::put(set_perms))
        .route("/api/roles/permissions", axum::routing::get(permission_whitelist))
        .with_state(pool)
}

/// 从 HeaderMap 中提取 JWT claims
fn extract_claims_from_headers(headers: &HeaderMap) -> Result<auth_service::Claims> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| crate::error::AppError::Validation("未提供登录凭证".into()))?;
    auth_service::verify_token(token)
}

/// 校验管理员权限
fn require_admin(claims: &auth_service::Claims) -> Result<()> {
    if !claims.is_admin {
        return Err(crate::error::AppError::Forbidden("需要管理员权限".into()));
    }
    Ok(())
}

/// GET /api/roles — 列出全部角色（含权限点）
async fn list(
    State(pool): State<DbPool>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<Vec<crate::models::role::RoleWithPermissions>>>> {
    let claims = extract_claims_from_headers(&headers)?;
    require_admin(&claims)?;
    let roles = role_repo::list_with_permissions(&pool)?;
    Ok(Json(ApiResponse::ok(roles)))
}

/// POST /api/roles — 新建角色
async fn create(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Json(body): Json<RoleCreate>,
) -> Result<Json<ApiResponse<crate::models::role::RoleWithPermissions>>> {
    let claims = extract_claims_from_headers(&headers)?;
    require_admin(&claims)?;
    let role = role_repo::create(&pool, &body)?;
    let conn = pool.get()?;
    audit_repo::log_on_conn_with_module(
        &conn, "create_role", "roles", Some(role.id),
        &claims.username, &format!("创建角色「{}」", role.name), "shared",
    )?;
    Ok(Json(ApiResponse::ok(role)))
}

/// PUT /api/roles/:id — 更新角色基础信息
async fn update(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Json(body): Json<RoleUpdate>,
) -> Result<Json<ApiResponse<crate::models::role::RoleWithPermissions>>> {
    let claims = extract_claims_from_headers(&headers)?;
    require_admin(&claims)?;
    let role = role_repo::update(&pool, id, &body)?;
    if let Some(ref name) = body.name {
        let conn = pool.get()?;
        audit_repo::log_on_conn_with_module(
            &conn, "update_role", "roles", Some(id),
            &claims.username, &format!("更新角色「{}」", name), "shared",
        )?;
    }
    Ok(Json(ApiResponse::ok(role)))
}

/// DELETE /api/roles/:id — 删除角色（系统角色拒绝）
async fn delete_one(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<()>>> {
    let claims = extract_claims_from_headers(&headers)?;
    require_admin(&claims)?;
    let role_name = {
        // 先获取角色名用于审计
        let roles = role_repo::list_all(&pool)?;
        roles.iter().find(|r| r.id == id).map(|r| r.name.clone()).unwrap_or_default()
    };
    role_repo::delete(&pool, id)?;
    let conn = pool.get()?;
    audit_repo::log_on_conn_with_module(
        &conn, "delete_role", "roles", Some(id),
        &claims.username, &format!("删除角色「{}」", role_name), "shared",
    )?;
    Ok(Json(ApiResponse::ok_msg("角色已删除")))
}

/// PUT /api/roles/:id/permissions — 设置角色权限点
async fn set_perms(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Json(body): Json<RolePermissionSet>,
) -> Result<Json<ApiResponse<crate::models::role::RoleWithPermissions>>> {
    let claims = extract_claims_from_headers(&headers)?;
    require_admin(&claims)?;
    let role = role_repo::set_permissions(&pool, id, &body)?;
    let conn = pool.get()?;
    audit_repo::log_on_conn_with_module(
        &conn, "set_permissions", "roles", Some(id),
        &claims.username, &format!("设置角色「{}」权限: {:?}", role.name, role.permissions), "shared",
    )?;
    Ok(Json(ApiResponse::ok(role)))
}

/// GET /api/roles/permissions — 返回权限点白名单（登录即可，无需 admin）
async fn permission_whitelist(
    _headers: HeaderMap,
) -> Result<Json<ApiResponse<Vec<PermissionDef>>>> {
    Ok(Json(ApiResponse::ok(PERMISSIONS.to_vec())))
}
