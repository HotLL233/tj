use axum::{
    extract::{Multipart, Path, State},
    http::HeaderMap,
    routing::{delete, get, post},
    Json, Router,
};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, sync::Arc};

use crate::config::AppConfig;
use crate::error::{AppError, Result};
use crate::models::ApiResponse;
use crate::repo::audit_repo;
use crate::service::{auth_service, backup_service};

#[derive(Serialize)]
pub struct BkStatus {
    pub auto_enabled: bool,
    pub auto_interval_hours: u64,
    pub max_backup_count: u64,
    pub backup_mode: String,
    pub backup_sync_dir: Option<String>,
    pub last_backup: Option<String>,
    pub backup_count: usize,
    pub backup_files: Vec<BkFile>,
    pub db_size: u64,
    pub tables: Vec<TableCount>,
    pub backups_dir: String,
    pub pending_restore: bool,
}

#[derive(Serialize)]
pub struct BkFile {
    pub name: String,
    pub size: u64,
    pub time: String,
    pub kind: String,
}

#[derive(Serialize)]
pub struct TableCount {
    pub table: String,
    pub rows: i64,
    pub label: String,
}

#[derive(Serialize)]
pub struct BkConfig {
    pub enabled: bool,
    pub interval_hours: u64,
    pub max_backup_count: u64,
    pub mode: String,
    pub sync_dir: Option<String>,
}

#[derive(Deserialize)]
pub struct BkUpdate {
    pub enabled: bool,
    pub interval_hours: u64,
    pub max_backup_count: Option<u64>,
    pub mode: Option<String>,
    pub sync_dir: Option<String>,
}

#[derive(Deserialize)]
struct SyncTestRequest {
    sync_dir: String,
}

pub fn router(config: Arc<AppConfig>) -> Router {
    Router::new()
        .route("/api/backup/status", get(status))
        .route("/api/backup/now", post(backup_now))
        .route("/api/backup/restore", post(restore))
        .route("/api/backup/restore/:fname", post(restore_file))
        .route("/api/backup/config", get(get_config).put(update_config))
        .route("/api/backup/test-sync", post(test_sync))
        .route("/api/backup/file/:fname", delete(delete_backup))
        .with_state(config)
}

fn require_admin(headers: &HeaderMap) -> Result<auth_service::Claims> {
    let token = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::Forbidden("未登录".into()))?;
    let claims = auth_service::verify_token(token)?;
    if !claims.is_admin { return Err(AppError::Forbidden("仅管理员可执行备份操作".into())); }
    Ok(claims)
}

fn current_config(base: &AppConfig) -> AppConfig {
    if AppConfig::config_path().exists() { AppConfig::load() } else { base.clone() }
}

fn table_counts(db_path: &std::path::Path) -> std::result::Result<Vec<TableCount>, String> {
    let conn = rusqlite::Connection::open(db_path).map_err(|e| e.to_string())?;
    let specs = [
        ("work_records", "分析检测记录", "SELECT COUNT(*) FROM work_records WHERE deleted_at IS NULL"),
        ("rd_work_records", "研发送样记录", "SELECT COUNT(*) FROM rd_work_records WHERE deleted_at IS NULL"),
        ("sample_info_records", "样品信息登记", "SELECT COUNT(*) FROM sample_info_records WHERE deleted_at IS NULL"),
        ("projects", "研发项目", "SELECT COUNT(*) FROM projects"),
        ("project_groups", "实验室", "SELECT COUNT(*) FROM project_groups"),
        ("methods", "检测方法", "SELECT COUNT(*) FROM methods"),
        ("divisions", "部门", "SELECT COUNT(*) FROM divisions"),
        ("users", "用户", "SELECT COUNT(*) FROM users"),
        ("audit_log", "审计日志", "SELECT COUNT(*) FROM audit_log"),
    ];
    let mut counts = Vec::new();
    for (table, label, sql) in specs {
        if let Ok(rows) = conn.query_row(sql, [], |row| row.get::<_, i64>(0)) {
            counts.push(TableCount { table: table.into(), rows, label: label.into() });
        }
    }
    Ok(counts)
}

async fn status(State(base): State<Arc<AppConfig>>) -> Result<Json<ApiResponse<BkStatus>>> {
    let cfg = current_config(&base);
    let dir = cfg.backup_dir();
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let Some(name) = entry.file_name().into_string().ok() else { continue; };
            if !name.ends_with(".db") && !name.ends_with(".zip") { continue; }
            let metadata = entry.metadata().ok();
            let size = metadata.as_ref().map(|item| item.len()).unwrap_or(0);
            let time = metadata.and_then(|item| item.modified().ok()).map(|value| {
                chrono::DateTime::<Local>::from(value).format("%Y-%m-%d %H:%M:%S").to_string()
            }).unwrap_or_default();
            let kind = if name.ends_with(".zip") { "full" } else { "database" };
            files.push(BkFile { name, size, time, kind: kind.into() });
        }
    }
    files.sort_by(|a, b| b.time.cmp(&a.time));
    let response = BkStatus {
        auto_enabled: cfg.backup_enabled,
        auto_interval_hours: cfg.backup_interval_hours,
        max_backup_count: cfg.max_backup_count,
        backup_mode: cfg.backup_mode.clone(),
        backup_sync_dir: cfg.backup_sync_dir.clone(),
        last_backup: files.first().map(|file| file.name.clone()),
        backup_count: files.len(),
        backup_files: files,
        db_size: fs::metadata(cfg.db_path()).map(|item| item.len()).unwrap_or(0),
        tables: table_counts(&cfg.db_path()).unwrap_or_default(),
        backups_dir: dir.to_string_lossy().to_string(),
        pending_restore: cfg.data_dir().join("restore_pending.json").exists(),
    };
    Ok(Json(ApiResponse::ok(response)))
}

async fn backup_now(State(base): State<Arc<AppConfig>>, headers: HeaderMap) -> Result<Json<ApiResponse<String>>> {
    let claims = require_admin(&headers)?;
    let cfg = current_config(&base);
    let result = backup_service::create_backup(&cfg, false).map_err(AppError::Internal)?;
    let detail = format!("手动备份: {} ({} KB)", result.name, result.size / 1024);
    audit_repo::log_for_backup("backup", &format!("{}，操作人: {}", detail, claims.username))?;
    let message = match result.sync_warning {
        Some(warning) => format!("本地备份成功: {}；同步目录失败: {}", result.name, warning),
        None => format!("备份成功: {} ({} KB)", result.name, result.size / 1024),
    };
    Ok(Json(ApiResponse::ok_msg(message)))
}

async fn restore(State(base): State<Arc<AppConfig>>, headers: HeaderMap, mut multipart: Multipart) -> Result<Json<ApiResponse<String>>> {
    let claims = require_admin(&headers)?;
    let cfg = current_config(&base);
    let mut upload: Option<PathBuf> = None;
    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::Validation(format!("读取上传失败: {e}")))? {
        if field.name() != Some("file") { continue; }
        let file_name = field.file_name().unwrap_or("backup.db").to_string();
        let extension = if file_name.to_ascii_lowercase().ends_with(".zip") { "zip" } else { "db" };
        let path = std::env::temp_dir().join(format!("restore_upload_{}.{}", uuid::Uuid::new_v4(), extension));
        let bytes = field.bytes().await.map_err(|e| AppError::Validation(format!("读取上传失败: {e}")))?;
        fs::write(&path, bytes).map_err(|e| AppError::Internal(e.to_string()))?;
        upload = Some(path);
    }
    let path = upload.ok_or_else(|| AppError::Validation("未收到备份文件".into()))?;
    let result = backup_service::stage_restore(&cfg, &path);
    let _ = fs::remove_file(&path);
    let safety = result.map_err(AppError::Validation)?;
    audit_repo::log_for_backup("restore", &format!("已暂存上传恢复，操作人: {}，恢复前备份: {}", claims.username, safety))?;
    Ok(Json(ApiResponse::ok_msg(format!("备份校验通过并已暂存。恢复前备份: {}。请重启程序完成恢复。", safety))))
}

async fn restore_file(State(base): State<Arc<AppConfig>>, headers: HeaderMap, Path(file_name): Path<String>) -> Result<Json<ApiResponse<String>>> {
    let claims = require_admin(&headers)?;
    if file_name.contains("..") || file_name.contains('/') || file_name.contains('\\') { return Err(AppError::Validation("非法文件名".into())); }
    let cfg = current_config(&base);
    let source = cfg.backup_dir().join(&file_name);
    let safety = backup_service::stage_restore(&cfg, &source).map_err(AppError::Validation)?;
    audit_repo::log_for_backup("restore", &format!("已暂存文件恢复: {}，操作人: {}，恢复前备份: {}", file_name, claims.username, safety))?;
    Ok(Json(ApiResponse::ok_msg(format!("备份校验通过并已暂存。恢复前备份: {}。请重启程序完成恢复。", safety))))
}

async fn get_config(State(base): State<Arc<AppConfig>>) -> Json<ApiResponse<BkConfig>> {
    let cfg = current_config(&base);
    Json(ApiResponse::ok(BkConfig {
        enabled: cfg.backup_enabled,
        interval_hours: cfg.backup_interval_hours,
        max_backup_count: cfg.max_backup_count,
        mode: cfg.backup_mode,
        sync_dir: cfg.backup_sync_dir,
    }))
}

async fn update_config(State(base): State<Arc<AppConfig>>, headers: HeaderMap, Json(body): Json<BkUpdate>) -> Result<Json<ApiResponse<String>>> {
    let claims = require_admin(&headers)?;
    if body.interval_hours == 0 { return Err(AppError::Validation("备份间隔至少为1小时".into())); }
    let mut cfg = current_config(&base);
    cfg.backup_enabled = body.enabled;
    cfg.backup_interval_hours = body.interval_hours;
    if let Some(count) = body.max_backup_count { cfg.max_backup_count = count.clamp(1, 200); }
    if let Some(mode) = body.mode {
        cfg.backup_mode = match mode.as_str() {
            "database" | "full" => mode,
            _ => return Err(AppError::Validation("备份模式仅支持数据库备份或全量备份".into())),
        };
    }
    if let Some(sync_dir) = body.sync_dir {
        cfg.backup_sync_dir = if sync_dir.trim().is_empty() { None } else { Some(sync_dir.trim().to_string()) };
    }
    cfg.save();
    audit_repo::log_for_backup("config", &format!("备份设置: 自动={} 间隔={}h 最大={} 模式={} 操作人={}", cfg.backup_enabled, cfg.backup_interval_hours, cfg.max_backup_count, cfg.backup_mode, claims.username))?;
    Ok(Json(ApiResponse::ok_msg("备份设置已保存并立即生效")))
}

async fn test_sync(headers: HeaderMap, Json(body): Json<SyncTestRequest>) -> Result<Json<ApiResponse<String>>> {
    require_admin(&headers)?;
    backup_service::test_sync_directory(&body.sync_dir).map_err(AppError::Validation)?;
    Ok(Json(ApiResponse::ok_msg("同步目录可正常写入")))
}

async fn delete_backup(State(base): State<Arc<AppConfig>>, headers: HeaderMap, Path(file_name): Path<String>) -> Result<Json<ApiResponse<String>>> {
    let claims = require_admin(&headers)?;
    if file_name.contains("..") || file_name.contains('/') || file_name.contains('\\') { return Err(AppError::Validation("非法文件名".into())); }
    let cfg = current_config(&base);
    fs::remove_file(cfg.backup_dir().join(&file_name)).map_err(|e| AppError::Validation(format!("删除失败: {e}")))?;
    audit_repo::log_for_backup("delete_backup", &format!("删除备份: {}，操作人: {}", file_name, claims.username))?;
    Ok(Json(ApiResponse::ok_msg(format!("已删除: {file_name}"))))
}
