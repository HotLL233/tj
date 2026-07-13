//! 鉴权业务服务：JWT 签发/校验、argon2 密码哈希/校验、登录、改密、权限聚合。

use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use rand_core::OsRng;
use argon2::Argon2;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};

use crate::db::DbPool;
use crate::error::{AppError, Result};
use crate::models::auth::{ChangePasswordRequest, Claims, LoginRequest, LoginResponse, MeResponse};
use crate::repo;

/// argon2id 哈希密码，返回编码后的哈希串（含 salt，可直接入库）。
pub fn hash_password(pwd: &str) -> Result<String> {
    if pwd.len() < 6 {
        return Err(AppError::Validation("密码长度至少 6 位".into()));
    }
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(pwd.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("密码哈希失败: {}", e)))?;
    Ok(hash.to_string())
}

/// 校验明文密码与哈希是否匹配。失败返回 Unauthorized（账号或密码错误）。
pub fn verify_password(pwd: &str, hash: &str) -> Result<()> {
    let parsed = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(format!("密码哈希解析失败: {}", e)))?;
    Argon2::default()
        .verify_password(pwd.as_bytes(), &parsed)
        .map_err(|_| AppError::Unauthorized("账号或密码错误".into()))?;
    Ok(())
}

/// 签发 JWT（HS256，过期 12 小时）。权限点以逗号串写入 claims。
pub fn issue_token(uid: i64, username: &str, role: &str, perms: &[String], must_change: bool, secret: &str) -> Result<String> {
    let exp = (chrono::Utc::now() + chrono::Duration::hours(12)).timestamp() as usize;
    let claims = Claims {
        sub: username.to_string(),
        uid,
        role: role.to_string(),
        perms: perms.join(","),
        must_change,
        exp,
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .map_err(|e| AppError::Internal(format!("令牌签发失败: {}", e)))
}

/// 校验 JWT 并返回 Claims（供中间件使用）。
pub fn verify_token(token: &str, secret: &str) -> Result<Claims> {
    let data = decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &Validation::new(Algorithm::HS256))
        .map_err(|_| AppError::Unauthorized("登录已过期或无效，请重新登录".into()))?;
    Ok(data.claims)
}

/// 登录：校验 argon2 密码 → 聚合权限 → 签发 JWT。
pub fn login(pool: &DbPool, req: &LoginRequest, secret: &str) -> Result<LoginResponse> {
    let user = repo::user_repo::get_by_username(pool, &req.username)?;
    if user.is_active != 1 {
        return Err(AppError::Forbidden("该账号已被禁用".into()));
    }
    verify_password(&req.password, &user.password_hash)?;
    let role_name = repo::role_repo::get_name(pool, user.role_id)?;
    let perms = repo::role_repo::get_permissions(pool, user.role_id)?;
    let must_change = user.must_change_password != 0;
    let token = issue_token(user.id, &user.username, &role_name, &perms, must_change, secret)?;
    Ok(LoginResponse {
        token,
        must_change_password: must_change,
        username: user.username,
        role: role_name,
        permissions: perms,
    })
}

/// 修改密码：非首登必须校验旧密码；成功后清除「强制改密」标记。
pub fn change_password(pool: &DbPool, uid: i64, req: &ChangePasswordRequest) -> Result<()> {
    let user = repo::user_repo::get_by_id(pool, uid)?;
    // 已改过密码的账号必须提供旧密码
    if user.must_change_password == 0 {
        let old = req.old_password.as_ref().ok_or_else(|| AppError::Validation("请提供旧密码".into()))?;
        verify_password(old, &user.password_hash)?;
    }
    let hash = hash_password(&req.new_password)?;
    repo::user_repo::set_password(pool, uid, &hash, true)?;
    Ok(())
}

/// 当前用户档案（供 /auth/me）。
pub fn me(pool: &DbPool, uid: i64) -> Result<MeResponse> {
    let user = repo::user_repo::get_by_id(pool, uid)?;
    let role_name = repo::role_repo::get_name(pool, user.role_id)?;
    let perms = repo::role_repo::get_permissions(pool, user.role_id)?;
    Ok(MeResponse {
        id: user.id,
        username: user.username,
        display_name: user.display_name,
        role: role_name,
        permissions: perms,
        must_change_password: user.must_change_password != 0,
    })
}
