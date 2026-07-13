use serde::{Deserialize, Serialize};

/// JWT 载荷（Claims）。权限点以逗号分隔写入 `perms`，避免每请求查库。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// 用户名（sub）
    pub sub: String,
    /// 用户主键
    pub uid: i64,
    /// 角色名
    pub role: String,
    /// 权限点（逗号分隔，`*` 表示全部）
    pub perms: String,
    /// 是否强制修改初始密码
    pub must_change: bool,
    /// 过期时间（Unix 秒）
    pub exp: usize,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub must_change_password: bool,
    pub username: String,
    pub role: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    /// 旧密码（首登强制改密时可不传）
    pub old_password: Option<String>,
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub id: i64,
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub permissions: Vec<String>,
    pub must_change_password: bool,
}
