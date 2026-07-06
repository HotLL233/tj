use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_port")] pub server_port: u16,
    #[serde(default = "default_db_dir")] pub db_dir: String,
    #[serde(default = "default_log_level")] pub log_level: String,
    #[serde(default)] pub log_file: Option<String>,
    #[serde(default = "default_admin_user")] pub admin_user: String,
    #[serde(default = "default_admin_pass")] pub admin_pass: String,
    #[serde(default = "default_backup_enabled")] pub backup_enabled: bool,
    #[serde(default = "default_backup_interval")] pub backup_interval_hours: u64,
}
fn default_port() -> u16 { 8000 } fn default_db_dir() -> String { "data".to_string() }
fn default_log_level() -> String { "info".to_string() } fn default_admin_user() -> String { "admin".to_string() }
fn default_admin_pass() -> String { "admin123".to_string() } fn default_backup_enabled() -> bool { false }
fn default_backup_interval() -> u64 { 24 }

impl Default for AppConfig { fn default() -> Self { Self { server_port: default_port(), db_dir: default_db_dir(), log_level: default_log_level(), log_file: None, admin_user: default_admin_user(), admin_pass: default_admin_pass(), backup_enabled: default_backup_enabled(), backup_interval_hours: default_backup_interval() } } }

impl AppConfig {
    pub fn load() -> Self { let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf())).unwrap_or_default(); let cp = exe_dir.join("config.toml"); if cp.exists() { let c = std::fs::read_to_string(&cp).unwrap_or_default(); toml::from_str(&c).unwrap_or_default() } else { Self::default() } }
    pub fn db_path(&self) -> PathBuf { let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf())).unwrap_or_default(); exe_dir.join(&self.db_dir).join("workload.db") }
    pub fn backup_dir(&self) -> PathBuf { let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf())).unwrap_or_default(); exe_dir.join(&self.db_dir).join("backups") }

}
