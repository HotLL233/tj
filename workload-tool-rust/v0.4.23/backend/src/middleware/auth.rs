//! JWT 鉴权中间件 + 当前用户上下文（AuthedUser）+ 细粒度权限守卫。
//!
//! 设计：所有 `/api/*` 请求经此后被校验；非 `/api` 路径（前端 SPA 路由）直接放行；
//! 白名单 `/api/version`、`/api/health`、`/api/auth/login` 公开。
//! 校验通过后将 [`AuthedUser`] 注入请求扩展，业务 Handler 直接提取使用。
//! 全部响应 HTTP 200（沿用 `ApiResponse{code,...}` 约定），401/403 仅体现在业务 code。

use std::sync::Arc;

use axum::{
    extract::{FromRequestParts, Request, State},
    http::header,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum::http::request::Parts;
use axum::async_trait;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};

use crate::error::AppError;
use crate::models::auth::Claims;

/// 鉴权中间件共享状态：仅持有 JWT 密钥（无状态校验，无需连接池）。
#[derive(Clone)]
pub struct AuthState {
    pub jwt_secret: String,
}

/// 无需 JWT 即可访问的 API 路径（精确匹配）。
const API_WHITELIST: &[&str] = &["/api/version", "/api/health", "/api/auth/login"];

/// 鉴权中间件：拦截所有 `/api/*` 请求完成 JWT 校验与用户注入。
pub async fn auth_middleware(
    State(state): State<Arc<AuthState>>,
    mut req: Request,
    next: Next,
) -> Response {
    let path = req.uri().path().to_string();

    // 1) 非 API 路径（前端 SPA 路由、静态资源）直接放行，保证登录页等可用
    if !path.starts_with("/api/") {
        return next.run(req).await;
    }

    // 2) API 白名单放行
    if API_WHITELIST.contains(&path.as_str()) {
        return next.run(req).await;
    }

    // 3) 解析 Bearer Token
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer ").or_else(|| v.strip_prefix("Bearer:")))
        .map(|s| s.trim().to_string());

    let token = match token {
        Some(t) if !t.is_empty() => t,
        _ => return AppError::Unauthorized("未登录或缺少凭证".into()).into_response(),
    };

    let claims = match decode::<Claims>(
        &token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(data) => data.claims,
        Err(_) => return AppError::Unauthorized("登录已过期或无效，请重新登录".into()).into_response(),
    };

    // 4) 注入当前用户，供后续 Handler 提取
    let user = AuthedUser::from_claims(&claims);
    req.extensions_mut().insert(user);

    next.run(req).await
}

/// 已认证用户上下文（从 JWT Claims 解析，注入到请求扩展中）。
#[derive(Clone, Debug)]
pub struct AuthedUser {
    pub uid: i64,
    pub username: String,
    pub role: String,
    pub perms: Vec<String>,
    pub must_change: bool,
}

impl AuthedUser {
    pub fn from_claims(c: &Claims) -> Self {
        let perms = if c.perms.is_empty() {
            Vec::new()
        } else {
            c.perms
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        };
        Self {
            uid: c.uid,
            username: c.sub.clone(),
            role: c.role.clone(),
            perms,
            must_change: c.must_change,
        }
    }

    /// 细粒度权限校验：无权限返回 `Forbidden(403)`；`*` 通配符放行所有。
    /// 首登强制改密期间，除自身改密外一律禁止。
    pub fn require(&self, perm: &str) -> Result<(), AppError> {
        if self.must_change {
            return Err(AppError::Forbidden("请先修改初始密码后再操作".into()));
        }
        if self.perms.iter().any(|p| p == "*") || self.perms.iter().any(|p| p == perm) {
            Ok(())
        } else {
            Err(AppError::Forbidden(format!("无权限执行此操作（需要权限：{}）", perm)))
        }
    }

    /// 自身资源访问：允许本人；他人则需 `user:manage` 权限。
    /// 即使处于强制改密状态也允许本人改密。
    pub fn require_self(&self, uid: i64) -> Result<(), AppError> {
        if self.uid == uid {
            Ok(())
        } else {
            self.require("user:manage")
        }
    }
}

/// 从请求扩展中提取 [`AuthedUser`]。对该类型不依赖具体路由状态，故对任意状态 S 均可用。
#[async_trait]
impl<S> FromRequestParts<S> for AuthedUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthedUser>()
            .cloned()
            .ok_or_else(|| AppError::Unauthorized("未登录".into()))
    }
}
