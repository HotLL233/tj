use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_port")]
    pub server_port: u16,

    #[serde(default = "default_db_dir")]
    pub db_dir: String,

    #[serde(default = "default_log_level")]
    pub log_level: String,

    #[serde(default)]
    pub log_file: Option<String>,

    /// 管理员账号，默认 admin
    #[serde(default = "default_admin_user")]
    pub admin_user: String,

    /// 管理员密码，默认 admin123 (config.toml 中可修改)
    #[serde(default = "default_admin_pass")]
    pub admin_pass: String,
}

fn default_port() -> u16 { 8000 }
fn default_db_dir() -> String { "data".to_string() }
fn default_log_level() -> String { "info".to_string() }
fn default_admin_user() -> String { "admin".to_string() }
fn default_admin_pass() -> String { "admin123".to_string() }

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server_port: default_port(),
            db_dir: default_db_dir(),
            log_level: default_log_level(),
            log_file: None,
            admin_user: default_admin_user(),
            admin_pass: default_admin_pass(),
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_default();

        let config_path = exe_dir.join("config.toml");
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path).unwrap_or_default();
            toml::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn db_path(&self) -> PathBuf {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_default();
        exe_dir.join(&self.db_dir).join("workload.db")
    }
}
