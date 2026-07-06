use axum::{extract::{State, Json}, Router, routing::post};
use serde::{Deserialize, Serialize};
use crate::config::AppConfig;
use crate::models::ApiResponse;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}

pub fn router(config: Arc<AppConfig>) -> Router {
    let state = Arc::new(AuthState { config });
    Router::new()
        .route("/api/auth/login", post(login))
        .with_state(state)
}

#[derive(Clone)]
struct AuthState {
    config: Arc<AppConfig>,
}

async fn login(
    State(state): State<Arc<AuthState>>,
    Json(body): Json<LoginRequest>,
) -> Json<ApiResponse<LoginResponse>> {
    if body.username == state.config.admin_user && body.password == state.config.admin_pass {
        let ts = chrono::Utc::now().timestamp();
        let token = format!("tok_{}_{:x}", &body.username, ts);
        Json(ApiResponse::ok(LoginResponse { token }))
    } else {
        Json(ApiResponse {
            code: 403,
            message: "账号或密码错误".into(),
            data: None,
        })
    }
}
