use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    #[serde(default = "default_port")] pub server_port: u16,
    #[serde(default = "default_db_dir")] pub db_dir: String,
    #[serde(default = "default_log_level")] pub log_level: String,
    #[serde(default)] pub log_file: Option<String>,
    /// 已废弃：不再作为鉴权依据（首启强制改密迁移）。保留字段仅为兼容旧 config.toml。
    #[serde(default = "default_admin_user")] pub admin_user: String,
    /// 已废弃：明文密码，不再读取。
    #[serde(default = "default_admin_pass")] pub admin_pass: String,
    /// JWT 签名密钥（可选）。若为空，则使用数据目录下生成的 .jwt_secret 文件。
    #[serde(default)] pub jwt_secret: Option<String>,
    #[serde(default = "default_backup_enabled")] pub backup_enabled: bool,
    #[serde(default = "default_backup_interval")] pub backup_interval_hours: u64,
    #[serde(default = "default_max_backup_count")] pub max_backup_count: u64,
}
fn default_port() -> u16 { 8000 } fn default_db_dir() -> String { "data".to_string() }
fn default_log_level() -> String { "info".to_string() } fn default_admin_user() -> String { "admin".to_string() }
fn default_admin_pass() -> String { "admin123".to_string() } fn default_backup_enabled() -> bool { false }
fn default_backup_interval() -> u64 { 24 }
fn default_max_backup_count() -> u64 { 10 }

impl Default for AppConfig { fn default() -> Self { Self { server_port: default_port(), db_dir: default_db_dir(), log_level: default_log_level(), log_file: None, admin_user: default_admin_user(), admin_pass: default_admin_pass(), jwt_secret: None, backup_enabled: default_backup_enabled(), backup_interval_hours: default_backup_interval(), max_backup_count: default_max_backup_count() } } }

impl AppConfig {
    pub fn load() -> Self { let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf())).unwrap_or_default(); let cp = exe_dir.join("config.toml"); if cp.exists() { let c = std::fs::read_to_string(&cp).unwrap_or_default(); toml::from_str(&c).unwrap_or_default() } else { Self::default() } }
    pub fn save(&self) { let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf())).unwrap_or_default(); let cp = exe_dir.join("config.toml"); if let Ok(s) = toml::to_string_pretty(self) { let _ = std::fs::write(&cp, s); } }

    /// 数据目录：支持 WORKLOAD_DATA_DIR 环境变量（Docker/Linux），fallback 到 exe 同级目录下的 db_dir
    pub fn data_dir(&self) -> PathBuf {
        if let Ok(d) = std::env::var("WORKLOAD_DATA_DIR") {
            PathBuf::from(d)
        } else {
            let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf())).unwrap_or_default();
            exe_dir.join(&self.db_dir)
        }
    }
    pub fn db_path(&self) -> PathBuf { self.data_dir().join("workload.db") }
    pub fn backup_dir(&self) -> PathBuf { self.data_dir().join("backups") }

    /// 解析并持久化 JWT 签名密钥：
    /// 1) config.toml `[auth] jwt_secret` 显式覆盖优先；
    /// 2) 否则读取数据目录下 `.jwt_secret` 文件；
    /// 3) 首次启动：随机生成 UUID 写入 `.jwt_secret`（Unix 下 chmod 600）。
    pub fn resolve_jwt_secret(&self) -> String {
        if let Some(ref s) = self.jwt_secret {
            if !s.trim().is_empty() { return s.trim().to_string(); }
        }
        let path = self.data_dir().join(".jwt_secret");
        if let Ok(existing) = std::fs::read_to_string(&path) {
            let t = existing.trim().to_string();
            if !t.is_empty() { return t; }
        }
        // 首次启动：生成随机密钥并写入文件
        let secret = uuid::Uuid::new_v4().to_string();
        if let Some(parent) = path.parent() { let _ = std::fs::create_dir_all(parent); }
        if std::fs::write(&path, &secret).is_ok() {
            #[cfg(unix)]
            { let _ = std::fs::set_permissions(&path, std::os::unix::fs::Permissions::from_mode(0o600)); }
        }
        secret
    }
}
