use std::sync::Arc;

use axum::{extract::{State, Json}, Router, routing::{post, get}};
use serde::Serialize;

use crate::db::DbPool;
use crate::error::Result;
use crate::models::ApiResponse;
use crate::models::auth::{ChangePasswordRequest, LoginRequest, LoginResponse, MeResponse};
use crate::middleware::auth::AuthedUser;
use crate::service::auth_service;

pub fn router(pool: DbPool, jwt_secret: String) -> Router {
    let state = Arc::new(AuthHandlerState { pool, jwt_secret });
    Router::new()
        .route("/api/auth/login", post(login))
        .route("/api/auth/change-password", post(change_password))
        .route("/api/auth/me", get(me))
        .with_state(state)
}

#[derive(Clone)]
struct AuthHandlerState {
    pool: DbPool,
    jwt_secret: String,
}

#[derive(Serialize)]
struct LoginData {
    token: String,
    must_change_password: bool,
    username: String,
    role: String,
    permissions: Vec<String>,
}

async fn login(
    State(state): State<Arc<AuthHandlerState>>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginData>>> {
    let resp: LoginResponse = auth_service::login(&state.pool, &body, &state.jwt_secret)?;
    Ok(Json(ApiResponse::ok(LoginData {
        token: resp.token,
        must_change_password: resp.must_change_password,
        username: resp.username,
        role: resp.role,
        permissions: resp.permissions,
    })))
}

async fn change_password(
    State(state): State<Arc<AuthHandlerState>>,
    user: AuthedUser,
    Json(body): Json<ChangePasswordRequest>,
) -> Result<Json<ApiResponse<()>>> {
    // 仅允许本人改密（首登强制改密期间也必须允许本人）；他人需 user:manage
    user.require_self(user.uid)?;
    auth_service::change_password(&state.pool, user.uid, &body)?;
    Ok(Json(ApiResponse::ok_msg("密码修改成功")))
}

async fn me(
    State(state): State<Arc<AuthHandlerState>>,
    user: AuthedUser,
) -> Result<Json<ApiResponse<MeData>>> {
    let resp: MeResponse = auth_service::me(&state.pool, user.uid)?;
    Ok(Json(ApiResponse::ok(MeData {
        id: resp.id,
        username: resp.username,
        display_name: resp.display_name,
        role: resp.role,
        permissions: resp.permissions,
        must_change_password: resp.must_change_password,
    })))
}

#[derive(Serialize)]
struct MeData {
    id: i64,
    username: String,
    display_name: String,
    role: String,
    permissions: Vec<String>,
    must_change_password: bool,
}
