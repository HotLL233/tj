use crate::db::DbPool;
use crate::error::{AppError, Result};
use crate::models::user::{LoginRequest, LoginResponse, User};
use crate::repo::user_repo;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JWT secret key — 生产环境应从环境变量读取
const JWT_SECRET: &str = "workload-tool-jwt-secret-v0.4.29";

/// JWT Claims
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i64,       // user id
    pub username: String,
    pub is_admin: bool,
    pub exp: usize,     // expiry
    pub iat: usize,     // issued at
}

/// 生成 JWT token
pub fn generate_token(user: &User) -> Result<String> {
    let now = Utc::now();
    let claims = Claims {
        sub: user.id,
        username: user.username.clone(),
        is_admin: user.is_admin,
        exp: (now + Duration::hours(24)).timestamp() as usize,
        iat: now.timestamp() as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Token 生成失败: {}", e)))
}

/// 验证 JWT token 并返回 Claims
pub fn verify_token(token: &str) -> Result<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::Validation("登录已过期，请重新登录".into()),
        _ => AppError::Validation("无效的登录凭证".into()),
    })
}

/// 验证密码
pub fn verify_password(password: &str, hash: &str) -> bool {
    bcrypt::verify(password, hash).unwrap_or(false)
}

/// 哈希密码
pub fn hash_password(password: &str) -> Result<String> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(format!("密码哈希失败: {}", e)))
}

/// 登录：验证用户名密码，返回 token + user 信息
pub fn login(pool: &DbPool, req: &LoginRequest) -> Result<LoginResponse> {
    let user = user_repo::find_by_username(pool, &req.username)?
        .ok_or_else(|| AppError::Validation("用户名或密码错误".into()))?;

    if !user.is_active {
        return Err(AppError::Validation("该账号已被停用".into()));
    }

    if !verify_password(&req.password, &user.password) {
        return Err(AppError::Validation("用户名或密码错误".into()));
    }

    let token = generate_token(&user)?;

    // 创建会话记录
    let conn = pool.get()?;
    let expires_at = (Utc::now() + Duration::hours(24)).format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO user_sessions (user_id, token, expires_at) VALUES (?1, ?2, ?3)",
        rusqlite::params![user.id, token, expires_at],
    )?;

    Ok(LoginResponse { token, user })
}

/// 登出：删除会话
pub fn logout(pool: &DbPool, token: &str) -> Result<()> {
    let conn = pool.get()?;
    conn.execute("DELETE FROM user_sessions WHERE token = ?1", [token])?;
    Ok(())
}

/// 注册用户
pub fn register(pool: &DbPool, username: &str, password: &str, division_id: Option<i64>, group_id: Option<i64>) -> Result<User> {
    let hash = hash_password(password)?;
    let data = crate::models::user::UserCreate {
        username: username.to_string(),
        password: password.to_string(),
        division_id,
        group_id,
    };
    user_repo::create(pool, &data, &hash)
}
