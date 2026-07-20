use crate::config::AppConfig;
use crate::db::DbPool;
use crate::error::{AppError, Result};
use crate::models::user::{LoginRequest, User, UserCreate, UserUpdate};
use crate::models::ApiResponse;
use crate::repo::user_repo;
use crate::service::auth_service;
use crate::service::authz_service::{
    self, AuthContext, ROLE_ANALYSIS_LEADER, ROLE_ANALYST, ROLE_RD_LEADER, ROLE_RD_SENDER,
    ROLE_SYSTEM_ADMIN,
};
use axum::{
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::HeaderMap,
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

pub fn router(pool: DbPool, config: Arc<AppConfig>) -> Router {
    Router::new()
        .route("/api/users/login", axum::routing::post(login))
        .route("/api/users/me", axum::routing::get(me))
        .route("/api/users/logout", axum::routing::post(logout))
        .route(
            "/api/users/change-password",
            axum::routing::put(change_password),
        )
        .route(
            "/api/users",
            axum::routing::get(list_users).post(create_user),
        )
        .route(
            "/api/users/:id",
            axum::routing::put(update_user).delete(delete_user),
        )
        .route(
            "/api/users/:id/permanent",
            axum::routing::delete(permanent_delete_user),
        )
        .route(
            "/api/users/import",
            axum::routing::post(import_users).layer(DefaultBodyLimit::max(50 * 1024 * 1024)),
        )
        .with_state((pool, config))
}

/// 从 HeaderMap 中提取 JWT claims
fn extract_claims_from_headers(pool: &DbPool, headers: &HeaderMap) -> Result<auth_service::Claims> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::Validation("未提供登录凭证".into()))?;
    auth_service::verify_active_token(pool, token)
}

/// 校验管理员权限
fn require_admin(claims: &auth_service::Claims) -> Result<()> {
    if !claims.is_admin {
        return Err(AppError::Forbidden("需要管理员权限".into()));
    }
    Ok(())
}

/// POST /api/users/register — 注册用户
async fn register(
    State((pool, _config)): State<(DbPool, Arc<AppConfig>)>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<User>>> {
    let username = body
        .get("username")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("用户名不能为空".into()))?
        .trim()
        .to_string();
    let password = body
        .get("password")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("密码不能为空".into()))?
        .to_string();
    let division_id = body.get("division_id").and_then(|v| v.as_i64());
    let group_id = body.get("group_id").and_then(|v| v.as_i64());

    if username.is_empty() || password.is_empty() {
        return Err(AppError::Validation("用户名和密码不能为空".into()));
    }

    let user = auth_service::register(&pool, &username, &password, division_id, group_id)?;
    Ok(Json(ApiResponse::ok(user)))
}

/// POST /api/users/login — 登录
async fn login(
    State((pool, _config)): State<(DbPool, Arc<AppConfig>)>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<ApiResponse<crate::models::user::LoginResponse>>> {
    let resp = auth_service::login(&pool, &body)?;
    Ok(Json(ApiResponse::ok(resp)))
}

/// GET /api/users/me — 获取当前用户信息
async fn me(
    State((pool, _config)): State<(DbPool, Arc<AppConfig>)>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<User>>> {
    let ctx = authz_service::authenticate(&pool, &headers)?;
    Ok(Json(ApiResponse::ok(ctx.user)))
}

/// GET /api/users — 用户列表（管理员）
async fn list_users(
    State((pool, _config)): State<(DbPool, Arc<AppConfig>)>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<Vec<User>>>> {
    let ctx = authz_service::authenticate(&pool, &headers)?;
    require_user_manager(&ctx)?;
    let users = user_repo::list_all(&pool)?;
    Ok(Json(ApiResponse::ok(users)))
}

/// PUT /api/users/:id — 更新用户
/// POST /api/users - admin-only user creation with role_ids support.
async fn create_user(
    State((pool, _config)): State<(DbPool, Arc<AppConfig>)>,
    headers: HeaderMap,
    Json(body): Json<UserCreate>,
) -> Result<Json<ApiResponse<User>>> {
    let ctx = authz_service::authenticate(&pool, &headers)?;
    require_user_manager(&ctx)?;

    if body.username.trim().is_empty() || body.password.is_empty() {
        return Err(AppError::Validation("用户名和密码不能为空".into()));
    }

    let mut body = body;
    let mut assigned = body.role_ids.clone();
    if let Some(role_id) = body.role_id {
        assigned.push(role_id);
    }
    assigned.sort_unstable();
    assigned.dedup();
    let names = role_names(&pool, &assigned)?;
    validate_leader_roles(&ctx, &names)?;
    let has_rd_role = names
        .iter()
        .any(|name| matches!(name.as_str(), ROLE_RD_SENDER | ROLE_RD_LEADER));
    if has_rd_role && body.group_id.is_none() {
        return Err(AppError::Validation(
            "包含研发送样角色时必须设置所属实验室".into(),
        ));
    }
    if !has_rd_role {
        body.division_id = None;
        body.group_id = None;
    }
    body.role_id = assigned.first().copied();
    body.role_ids = assigned;
    let password_hash = auth_service::hash_password(&body.password)?;
    let user = user_repo::create(
        &pool,
        &body,
        &password_hash,
        Some((ctx.user.id, &ctx.user.username)),
    )?;
    Ok(Json(ApiResponse::ok(user)))
}

fn role_names(pool: &DbPool, role_ids: &[i64]) -> Result<Vec<String>> {
    let conn = pool.get()?;
    let mut names = Vec::with_capacity(role_ids.len());
    for role_id in role_ids {
        let name = conn
            .query_row("SELECT name FROM roles WHERE id=?1", [role_id], |row| {
                row.get(0)
            })
            .map_err(|_| AppError::Validation("请选择有效角色".into()))?;
        names.push(name);
    }
    Ok(names)
}

fn leader_assignable_role(name: &str) -> bool {
    matches!(name, ROLE_ANALYST | ROLE_RD_SENDER | ROLE_RD_LEADER)
}

fn require_user_manager(ctx: &AuthContext) -> Result<()> {
    if ctx.is_system_admin() || ctx.is_analysis_leader() {
        Ok(())
    } else {
        Err(AppError::Forbidden("无用户管理权限".into()))
    }
}

fn validate_leader_roles(ctx: &AuthContext, names: &[String]) -> Result<()> {
    if ctx.is_system_admin() {
        return Ok(());
    }
    if names.is_empty() || names.iter().any(|name| !leader_assignable_role(name)) {
        return Err(AppError::Forbidden(
            "分析检测组长只能分配分析检测员、研发送样员和研发送样组长角色".into(),
        ));
    }
    Ok(())
}

fn target_is_protected(target: &User) -> bool {
    target.is_admin
        || target
            .role_names
            .iter()
            .any(|name| matches!(name.as_str(), ROLE_SYSTEM_ADMIN | ROLE_ANALYSIS_LEADER))
}

async fn update_user(
    State((pool, _config)): State<(DbPool, Arc<AppConfig>)>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Json(mut body): Json<UserUpdate>,
) -> Result<Json<ApiResponse<User>>> {
    let ctx = authz_service::authenticate(&pool, &headers)?;
    require_user_manager(&ctx)?;
    let target =
        user_repo::find_by_id(&pool, id)?.ok_or_else(|| AppError::NotFound("用户不存在".into()))?;
    if body.is_admin.is_some() {
        return Err(AppError::Validation(
            "管理员权限请通过“系统管理员”角色配置".into(),
        ));
    }
    if id == ctx.user.id && !ctx.is_system_admin() {
        if body.role_id.is_some()
            || body.role_ids.is_some()
            || body.is_admin.is_some()
            || body.is_active.is_some()
            || body.division_id.is_some()
            || body.group_id.is_some()
        {
            return Err(AppError::Forbidden("只能修改自己的用户名和密码".into()));
        }
    } else if !ctx.is_system_admin() {
        if target_is_protected(&target) {
            return Err(AppError::Forbidden(
                "不能编辑系统管理员或其他分析检测组长".into(),
            ));
        }
        if body.is_admin == Some(true) {
            return Err(AppError::Forbidden("不能授予系统管理员权限".into()));
        }
    }
    if let Some(ref mut assigned) = body.role_ids {
        assigned.sort_unstable();
        assigned.dedup();
        let names = role_names(&pool, assigned)?;
        validate_leader_roles(&ctx, &names)?;
        let has_rd_role = names
            .iter()
            .any(|name| matches!(name.as_str(), ROLE_RD_SENDER | ROLE_RD_LEADER));
        if has_rd_role && body.group_id.flatten().or(target.group_id).is_none() {
            return Err(AppError::Validation(
                "包含研发送样角色时必须设置所属实验室".into(),
            ));
        }
        if !has_rd_role {
            body.division_id = Some(None);
            body.group_id = Some(None);
        }
        body.role_id = Some(assigned.first().copied());
    }
    let user = user_repo::update(&pool, id, &body, ctx.user.id, &ctx.user.username)?;
    Ok(Json(ApiResponse::ok(user)))
}

/// DELETE /api/users/:id — 删除用户（软删除）
async fn delete_user(
    State((pool, _config)): State<(DbPool, Arc<AppConfig>)>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<()>>> {
    let ctx = authz_service::authenticate(&pool, &headers)?;
    require_user_manager(&ctx)?;
    if id == ctx.user.id {
        return Err(AppError::Forbidden("不能停用当前登录账号".into()));
    }
    let target =
        user_repo::find_by_id(&pool, id)?.ok_or_else(|| AppError::NotFound("用户不存在".into()))?;
    if !ctx.is_system_admin() && target_is_protected(&target) {
        return Err(AppError::Forbidden(
            "不能停用系统管理员或分析检测组长".into(),
        ));
    }
    user_repo::soft_delete(&pool, id, &ctx.user.username)?;
    Ok(Json(ApiResponse::ok_msg("删除成功")))
}

/// DELETE /api/users/:id/permanent — 彻底删除用户（v0.4.65）
async fn permanent_delete_user(
    State((pool, _config)): State<(DbPool, Arc<AppConfig>)>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<()>>> {
    let ctx = authz_service::authenticate(&pool, &headers)?;
    if !ctx.is_system_admin() {
        return Err(AppError::Forbidden("只有系统管理员可以彻底删除用户".into()));
    }
    if id == ctx.user.id {
        return Err(AppError::Forbidden("不能永久停用当前登录账号".into()));
    }
    user_repo::permanent_delete(&pool, id, ctx.user.id, &ctx.user.username)?;
    Ok(Json(ApiResponse::ok_msg("用户已匿名化并永久停用，历史记录与审计仍保留")))
}

/// POST /api/users/logout — 登出
async fn logout(
    State((pool, _config)): State<(DbPool, Arc<AppConfig>)>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<()>>> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("");

    if !token.is_empty() {
        auth_service::logout(&pool, token)?;
    }
    Ok(Json(ApiResponse::ok_msg("已登出")))
}

/// PUT /api/users/change-password — 修改密码
#[derive(Deserialize)]
struct ChangePasswordRequest {
    old_password: String,
    new_password: String,
}

async fn change_password(
    State((pool, _config)): State<(DbPool, Arc<AppConfig>)>,
    headers: HeaderMap,
    Json(body): Json<ChangePasswordRequest>,
) -> Result<Json<ApiResponse<()>>> {
    let claims = extract_claims_from_headers(&pool, &headers)?;

    if body.old_password.is_empty() || body.new_password.is_empty() {
        return Err(AppError::Validation("密码不能为空".into()));
    }
    if body.new_password.len() < 4 {
        return Err(AppError::Validation("新密码至少4位".into()));
    }

    let user = user_repo::find_by_id(&pool, claims.sub)?
        .ok_or_else(|| AppError::NotFound("用户不存在".into()))?;

    // 验证旧密码
    if !auth_service::verify_password(&body.old_password, &user.password) {
        return Err(AppError::Validation("旧密码错误".into()));
    }

    // 哈希新密码
    let new_hash = auth_service::hash_password(&body.new_password)?;

    let conn = pool.get()?;
    conn.execute(
        "UPDATE users SET password = ?1, updated_at = datetime('now', 'localtime') WHERE id = ?2",
        rusqlite::params![new_hash, claims.sub],
    )?;

    Ok(Json(ApiResponse::ok_msg("密码修改成功")))
}

async fn import_users(
    State((pool, _config)): State<(DbPool, Arc<AppConfig>)>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let ctx = authz_service::authenticate(&pool, &headers)?;
    require_user_manager(&ctx)?;
    let mut file_data = Vec::new();
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Internal(format!("读取上传文件失败: {}", e)))?
    {
        if field.name().unwrap_or("") == "file" {
            file_data = field
                .bytes()
                .await
                .map_err(|e| AppError::Internal(format!("读取文件内容失败: {}", e)))?
                .to_vec();
        }
    }
    if file_data.is_empty() {
        return Err(AppError::Validation("未选择文件".into()));
    }
    let content = String::from_utf8_lossy(&file_data);
    let mut lines: Vec<&str> = content.lines().collect();
    if lines.len() < 2 {
        return Err(AppError::Validation(
            "文件至少需要 2 行（表头 + 数据）".into(),
        ));
    }
    let _header = lines.remove(0);
    let mut created = 0i64;
    let mut skipped = 0i64;
    let mut errors: Vec<String> = vec![];
    let conn = pool.get()?;
    for (i, line) in lines.iter().enumerate() {
        let vals: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if vals.is_empty() || vals[0].is_empty() {
            skipped += 1;
            continue;
        }
        let username = vals[0].trim_start_matches('\u{feff}');
        let password = vals.get(1).copied().filter(|value| !value.is_empty()).unwrap_or("123456");
        if password.len() < 6 {
            errors.push(format!("行{}: 密码长度不足", i + 2));
            skipped += 1;
            continue;
        }
        let existing: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM users WHERE username=?1",
                [username],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if existing > 0 {
            errors.push(format!("行{}: 用户名「{}」已存在", i + 2, username));
            skipped += 1;
            continue;
        }
        let mut row_errors = Vec::new();
        let division_id: Option<i64> = if vals.len() > 3 && !vals[3].is_empty() {
            let dname = vals[3];
            match conn.query_row("SELECT id FROM divisions WHERE name=?1", [dname], |r| {
                r.get(0)
            }) {
                Ok(id) => Some(id),
                Err(_) => {
                    row_errors.push(format!("行{}: 部门「{}」不存在", i + 2, dname));
                    None
                }
            }
        } else {
            None
        };
        let group_id: Option<i64> = if vals.len() > 4 && !vals[4].is_empty() {
            let gname = vals[4];
            match conn.query_row(
                "SELECT id FROM project_groups WHERE name=?1",
                [gname],
                |r| r.get(0),
            ) {
                Ok(id) => Some(id),
                Err(_) => {
                    row_errors.push(format!("行{}: 实验室「{}」不存在", i + 2, gname));
                    None
                }
            }
        } else {
            None
        };
        let mut role_ids = Vec::new();
        if vals.len() > 5 && !vals[5].is_empty() {
            for role_name in vals[5].split(|ch| matches!(ch, '|' | ';' | '；')).map(str::trim).filter(|value| !value.is_empty()) {
                match conn.query_row("SELECT id FROM roles WHERE name=?1", [role_name], |r| r.get(0)) {
                    Ok(id) => role_ids.push(id),
                    Err(_) => row_errors.push(format!("行{}: 角色「{}」不存在", i + 2, role_name)),
                }
            }
        }
        role_ids.sort_unstable();
        role_ids.dedup();
        if !row_errors.is_empty() {
            errors.extend(row_errors);
            skipped += 1;
            continue;
        }
        let names = role_names(&pool, &role_ids)?;
        if let Err(error) = validate_leader_roles(&ctx, &names) {
            errors.push(format!("行{}: {}", i + 2, error));
            skipped += 1;
            continue;
        }
        let has_rd_role = names.iter().any(|name| matches!(name.as_str(), ROLE_RD_SENDER | ROLE_RD_LEADER));
        if has_rd_role && group_id.is_none() {
            errors.push(format!("行{}: 研发送样角色必须设置所属实验室", i + 2));
            skipped += 1;
            continue;
        }
        let data = UserCreate {
            username: username.to_string(),
            password: password.to_string(),
            division_id: if has_rd_role { division_id } else { None },
            group_id: if has_rd_role { group_id } else { None },
            role_id: role_ids.first().copied(),
            role_ids,
        };
        let hashed = auth_service::hash_password(password)?;
        match user_repo::create(&pool, &data, &hashed, Some((ctx.user.id, &ctx.user.username))) {
            Ok(_) => created += 1,
            Err(error) => {
                errors.push(format!("行{}: {}", i + 2, error));
                skipped += 1;
            }
        }
    }
    let detail = format!("批量导入用户: 成功 {} 条, 跳过 {} 条", created, skipped);
    crate::repo::audit_repo::log_actor(&pool, "import", "users", None, ctx.user.id, &ctx.user.username, &detail, "shared")?;
    let result = serde_json::json!({"created": created, "skipped": skipped, "errors": errors});
    Ok(Json(ApiResponse::ok(result)))
}
