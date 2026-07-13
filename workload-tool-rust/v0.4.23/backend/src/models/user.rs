use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub display_name: String,
    pub password_hash: String,
    pub role_id: i64,
    pub lab_id: Option<i64>,
    pub is_active: i32,
    pub must_change_password: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UserCreate {
    pub username: String,
    pub display_name: Option<String>,
    pub password: String,
    pub role_id: i64,
    pub lab_id: Option<i64>,
    #[serde(default = "default_active")] pub is_active: i32,
}

fn default_active() -> i32 { 1 }

#[derive(Debug, Deserialize)]
pub struct UserUpdate {
    pub display_name: Option<String>,
    pub role_id: Option<i64>,
    pub lab_id: Option<i64>,
    pub is_active: Option<i32>,
}

/// 对外暴露的用户信息（不含密码哈希）
#[derive(Debug, Serialize)]
pub struct UserPublic {
    pub id: i64,
    pub username: String,
    pub display_name: String,
    pub role_id: i64,
    pub role_name: String,
    pub lab_id: Option<i64>,
    pub is_active: i32,
    pub must_change_password: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub new_password: String,
}
