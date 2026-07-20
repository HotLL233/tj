use crate::db::DbPool;
use crate::error::Result;
use crate::models::role::{PermissionDef, RoleCreate, RolePermissionSet, RoleUpdate, PERMISSIONS};
use crate::models::ApiResponse;
use crate::repo::audit_repo;
use crate::repo::role_repo;
use crate::service::authz_service;
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json, Router,
};

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/roles", axum::routing::get(list).post(create))
        .route(
            "/api/roles/:id",
            axum::routing::put(update).delete(delete_one),
        )
        .route("/api/roles/:id/permissions", axum::routing::put(set_perms))
        .route(
            "/api/roles/permissions",
            axum::routing::get(permission_whitelist),
        )
        .with_state(pool)
}

/// GET /api/roles — 列出全部角色（含权限点）
async fn list(
    State(pool): State<DbPool>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<Vec<crate::models::role::RoleWithPermissions>>>> {
    let ctx = authz_service::authenticate(&pool, &headers)?;
    if !ctx.is_system_admin() && !ctx.is_analysis_leader() {
        return Err(crate::error::AppError::Forbidden("无角色查看权限".into()));
    }
    let roles = role_repo::list_with_permissions(&pool)?;
    Ok(Json(ApiResponse::ok(roles)))
}

/// POST /api/roles — 新建角色
async fn create(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Json(body): Json<RoleCreate>,
) -> Result<Json<ApiResponse<crate::models::role::RoleWithPermissions>>> {
    let ctx = authz_service::authenticate(&pool, &headers)?;
    if !ctx.is_system_admin() { return Err(crate::error::AppError::Forbidden("仅系统管理员可新增自定义角色".into())); }
    let role = role_repo::create(&pool, &body)?;
    let conn = pool.get()?;
    audit_repo::log_on_conn_with_module(&conn, "create_role", "roles", Some(role.id), &ctx.user.username, &format!("创建角色「{}」", role.name), "shared")?;
    Ok(Json(ApiResponse::ok(role)))
}

/// PUT /api/roles/:id — 更新角色基础信息
async fn update(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Json(body): Json<RoleUpdate>,
) -> Result<Json<ApiResponse<crate::models::role::RoleWithPermissions>>> {
    let ctx = authz_service::authenticate(&pool, &headers)?;
    if !ctx.is_system_admin() { return Err(crate::error::AppError::Forbidden("仅系统管理员可编辑自定义角色".into())); }
    let role = role_repo::update(&pool, id, &body)?;
    if let Some(ref name) = body.name {
        let conn = pool.get()?;
        audit_repo::log_on_conn_with_module(&conn, "update_role", "roles", Some(id), &ctx.user.username, &format!("更新角色「{}」", name), "shared")?;
    }
    Ok(Json(ApiResponse::ok(role)))
}

/// DELETE /api/roles/:id — 删除角色（系统角色拒绝）
async fn delete_one(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<()>>> {
    let ctx = authz_service::authenticate(&pool, &headers)?;
    if !ctx.is_system_admin() { return Err(crate::error::AppError::Forbidden("仅系统管理员可删除自定义角色".into())); }
    let role_name = role_repo::list_all(&pool)?.iter().find(|r| r.id == id).map(|r| r.name.clone()).unwrap_or_default();
    role_repo::delete(&pool, id)?;
    let conn = pool.get()?;
    audit_repo::log_on_conn_with_module(&conn, "delete_role", "roles", Some(id), &ctx.user.username, &format!("删除角色「{}」", role_name), "shared")?;
    Ok(Json(ApiResponse::ok_msg("角色已删除")))
}

/// PUT /api/roles/:id/permissions — 设置角色权限点
async fn set_perms(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Json(body): Json<RolePermissionSet>,
) -> Result<Json<ApiResponse<crate::models::role::RoleWithPermissions>>> {
    let ctx = authz_service::authenticate(&pool, &headers)?;
    if !ctx.is_system_admin() { return Err(crate::error::AppError::Forbidden("仅系统管理员可设置自定义角色权限".into())); }
    let target = role_repo::list_all(&pool)?.into_iter().find(|role| role.id == id)
        .ok_or_else(|| crate::error::AppError::NotFound("角色不存在".into()))?;
    if target.is_system == 1 { return Err(crate::error::AppError::Forbidden("五个固定角色的权限矩阵不能修改".into())); }
    let role = role_repo::set_permissions(&pool, id, &body)?;
    let conn = pool.get()?;
    audit_repo::log_on_conn_with_module(&conn, "set_permissions", "roles", Some(id), &ctx.user.username, &format!("设置角色「{}」权限: {:?}", role.name, role.permissions), "shared")?;
    Ok(Json(ApiResponse::ok(role)))
}

/// GET /api/roles/permissions — 返回权限点白名单（登录即可，无需 admin）
async fn permission_whitelist(
    State(pool): State<DbPool>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<Vec<PermissionDef>>>> {
    authz_service::authenticate(&pool, &headers)?;
    Ok(Json(ApiResponse::ok(PERMISSIONS.to_vec())))
}
