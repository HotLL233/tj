use serde::{Deserialize, Serialize};

/// 用户 Model
#[derive(Debug, Serialize, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub division_id: Option<i64>,
    pub division_name: Option<String>,
    pub group_id: Option<i64>,
    pub group_name: Option<String>,
    pub is_admin: bool,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: Option<String>,
}

/// 登录请求
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// 登录响应
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: User,
}

/// 注册请求
#[derive(Debug, Deserialize)]
pub struct UserCreate {
    pub username: String,
    pub password: String,
    pub division_id: Option<i64>,
    pub group_id: Option<i64>,
}

/// 用户更新请求
#[derive(Debug, Deserialize)]
pub struct UserUpdate {
    pub username: Option<String>,
    pub password: Option<String>,
    pub division_id: Option<Option<i64>>,
    pub group_id: Option<Option<i64>>,
    pub is_admin: Option<bool>,
    pub is_active: Option<bool>,
}
