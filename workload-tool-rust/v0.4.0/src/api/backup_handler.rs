use axum::{extract::{Multipart, State, Path}, Router, Json, routing::{get, post, delete}, response::IntoResponse};
use serde::{Deserialize, Serialize};
use crate::config::AppConfig;
use crate::models::ApiResponse;
use crate::repo::audit_repo;
use std::sync::Arc; use std::fs; use chrono::Local;
use rusqlite::Connection;

#[derive(Serialize)] pub struct BkStatus { pub auto_enabled: bool, pub auto_interval_hours: u64, pub max_backup_count: u64, pub last_backup: Option<String>, pub backup_count: usize, pub backup_files: Vec<BkFile>, pub db_size: u64, pub tables: Vec<TableCount>, pub backups_dir: String }
#[derive(Serialize)] pub struct BkFile { pub name: String, pub size: u64, pub time: String }
#[derive(Serialize)] pub struct TableCount { pub table: String, pub rows: i64 }
#[derive(Serialize)] pub struct BkConfig { pub enabled: bool, pub interval_hours: u64, pub max_backup_count: u64 }
#[derive(Deserialize)] pub struct BkUpdate { pub enabled: bool, pub interval_hours: u64, pub max_backup_count: Option<u64> }

pub fn router(config: Arc<AppConfig>) -> Router {
    Router::new().route("/api/backup/status", get(status)).route("/api/backup/now", post(backup_now)).route("/api/backup/restore", post(restore)).route("/api/backup/restore/{fname}", post(restore_file)).route("/api/backup/config", get(get_config).put(update_config)).route("/api/backup/file/{fname}", delete(delete_backup)).with_state(config)
}

/// 使用 VACUUM INTO 进行原子一致性备份
fn do_backup(db_path: &str, backup_dir: &std::path::Path) -> Result<(String, u64), String> {
    fs::create_dir_all(backup_dir).map_err(|e| e.to_string())?;
    let name = format!("workload_{}.db", Local::now().format("%Y%m%d_%H%M%S"));
    let dst = backup_dir.join(&name);
    let conn = Connection::open(db_path).map_err(|e| format!("无法打开数据库: {}", e))?;
    conn.execute_batch(&format!("VACUUM INTO '{}'", dst.to_string_lossy().replace('\'', "''"))).map_err(|e| format!("备份失败: {}", e))?;
    let size = fs::metadata(&dst).map(|m| m.len()).unwrap_or(0);
    Ok((name, size))
}

fn table_counts(db_path: &str) -> Result<Vec<TableCount>, String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    let tables = ["work_records", "sample_records", "projects", "project_groups", "methods", "audit_log"];
    let mut counts = vec![];
    for t in &tables {
        if let Ok(n) = conn.query_row(&format!("SELECT COUNT(*) FROM {} WHERE deleted_at IS NULL", t), [], |r| r.get::<_, i64>(0)) {
            counts.push(TableCount { table: t.to_string(), rows: n });
        }
    }
    if let Ok(n) = conn.query_row("SELECT COUNT(*) FROM audit_log", [], |r| r.get::<_, i64>(0)) {
        counts.push(TableCount { table: "audit_log".to_string(), rows: n });
    }
    Ok(counts)
}

fn verify_backup(path: &std::path::Path) -> Result<String, String> {
    let conn = Connection::open(path).map_err(|e| format!("验证失败: {}", e))?;
    let ok: String = conn.query_row("PRAGMA integrity_check", [], |r| r.get(0)).map_err(|e| e.to_string())?;
    if ok == "ok" { Ok(ok) } else { Err(format!("数据库损坏: {}", ok)) }
}

async fn status(State(cfg): State<Arc<AppConfig>>) -> impl IntoResponse {
    let dir = cfg.backup_dir(); let mut files: Vec<BkFile> = vec![];
    if let Ok(es) = fs::read_dir(&dir) {
        for e in es.flatten() {
            if let Ok(n) = e.file_name().into_string() {
                if n.ends_with(".db") {
                    let size = e.metadata().map(|m| m.len()).unwrap_or(0);
                    let time = e.metadata().and_then(|m| m.modified()).map(|t| {
                        chrono::DateTime::<Local>::from(t).format("%Y-%m-%d %H:%M:%S").to_string()
                    }).unwrap_or_default();
                    files.push(BkFile { name: n, size, time });
                }
            }
        }
    }
    files.sort_by(|a,b| b.name.cmp(&a.name));
    let db = cfg.db_path().to_string_lossy().to_string();
    let tables = table_counts(&db).unwrap_or_default();
    let db_size = fs::metadata(&db).map(|m| m.len()).unwrap_or(0);
    Json(ApiResponse::ok(BkStatus { auto_enabled: cfg.backup_enabled, auto_interval_hours: cfg.backup_interval_hours, max_backup_count: cfg.max_backup_count, last_backup: files.first().map(|f| f.name.clone()), backup_count: files.len(), backup_files: files, db_size, tables, backups_dir: dir.to_string_lossy().to_string() }))
}

async fn backup_now(State(cfg): State<Arc<AppConfig>>) -> impl IntoResponse {
    let db = cfg.db_path().to_string_lossy().to_string();
    let backup_dir = cfg.backup_dir();
    match do_backup(&db, &backup_dir) {
        Ok((name, size)) => {
            cleanup_old_backups(&backup_dir, cfg.max_backup_count);
            let tables = table_counts(&db).unwrap_or_default();
            let total: i64 = tables.iter().map(|t| t.rows).sum();
            let _ = audit_repo::log_for_backup("backup", &format!("手动备份: {} ({}KB, {}条)", name, size / 1024, total));
            Json(ApiResponse::<String>::ok_msg(format!("备份成功: {} ({}KB, {}条记录)", name, size / 1024, total)))
        }
        Err(e) => Json(ApiResponse::<String>::ok_msg(format!("备份失败: {}", e)))
    }
}

fn cleanup_old_backups(dir: &std::path::Path, max_count: u64) {
    if max_count == 0 { return; }
    if let Ok(es) = fs::read_dir(dir) {
        let mut files: Vec<_> = es.flatten().filter_map(|e| {
            let name = e.file_name().into_string().ok()?;
            if !name.ends_with(".db") { return None; }
            let time = e.metadata().ok()?.modified().ok()?;
            Some((name, time))
        }).collect();
        files.sort_by(|a,b| b.1.cmp(&a.1));
        for (name, _) in files.iter().skip(max_count as usize) {
            let _ = fs::remove_file(dir.join(name));
        }
    }
}

/// 上传文件恢复
async fn restore(State(cfg): State<Arc<AppConfig>>, mut mp: Multipart) -> impl IntoResponse {
    let mut tmp = String::new();
    while let Ok(Some(f)) = mp.next_field().await { if f.name() == Some("file") { if let Ok(d) = f.bytes().await { if !d.is_empty() { let p = std::env::temp_dir().join("restore_tmp.db"); if fs::write(&p, &d).is_ok() { tmp = p.to_string_lossy().to_string(); } } } } }
    if tmp.is_empty() { return Json(ApiResponse::<String>::ok_msg(String::from("未收到文件"))); }
    if let Err(e) = verify_backup(std::path::Path::new(&tmp)) {
        let _ = fs::remove_file(&tmp);
        return Json(ApiResponse::<String>::ok_msg(format!("备份文件无效: {}", e)));
    }
    let db = cfg.db_path().to_string_lossy().to_string();
    let bk = do_backup(&db, &cfg.backup_dir()).map(|(n, _)| n).unwrap_or_else(|_| "unknown".into());
    if let Err(e) = fs::copy(&tmp, &db) { let _ = fs::remove_file(&tmp); return Json(ApiResponse::<String>::ok_msg(format!("恢复失败: {}", e))); }
    let _ = fs::remove_file(&tmp);
    let check = verify_backup(std::path::Path::new(&db)).unwrap_or_else(|e| e);
    let _ = audit_repo::log_for_backup("restore", &format!("上传恢复: {} | 旧库: {}", check, bk));
    Json(ApiResponse::<String>::ok_msg(format!("恢复完成。旧库备份: {} | 完整性: {}", bk, check)))
}

/// 从已有备份文件恢复
async fn restore_file(State(cfg): State<Arc<AppConfig>>, Path(fname): Path<String>) -> impl IntoResponse {
    if fname.contains("..") || fname.contains("/") || fname.contains("\\") { return Json(ApiResponse::<String>::ok_msg(String::from("非法文件名"))); }
    let src = cfg.backup_dir().join(&fname);
    if !src.exists() { return Json(ApiResponse::<String>::ok_msg(format!("备份文件不存在: {}", fname))); }
    if let Err(e) = verify_backup(&src) {
        return Json(ApiResponse::<String>::ok_msg(format!("备份文件无效: {}", e)));
    }
    let db = cfg.db_path().to_string_lossy().to_string();
    // 恢复前备份当前库
    let bk = do_backup(&db, &cfg.backup_dir()).map(|(n, _)| n).unwrap_or_else(|_| "unknown".into());
    match fs::copy(&src, &db) {
        Ok(_) => {
            let _ = audit_repo::log_for_backup("restore", &format!("文件恢复: {} | 旧库: {}", fname, bk));
            Json(ApiResponse::<String>::ok_msg(format!("恢复成功: {}。请重启程序以使新数据库生效。旧库已备份为 {}", fname, bk)))
        }
        Err(e) => Json(ApiResponse::<String>::ok_msg(format!("恢复失败（数据库可能被占用，请关闭程序后重试）: {}", e)))
    }
}

async fn get_config(State(cfg): State<Arc<AppConfig>>) -> impl IntoResponse { Json(ApiResponse::ok(BkConfig { enabled: cfg.backup_enabled, interval_hours: cfg.backup_interval_hours, max_backup_count: cfg.max_backup_count })) }

async fn update_config(State(cfg): State<Arc<AppConfig>>, Json(b): Json<BkUpdate>) -> impl IntoResponse {
    // 更新内存中的配置
    let mut new_cfg = (*cfg).clone();
    new_cfg.backup_enabled = b.enabled;
    new_cfg.backup_interval_hours = b.interval_hours;
    if let Some(mc) = b.max_backup_count { new_cfg.max_backup_count = mc; }
    // 持久化到 config.toml
    new_cfg.save();
    let _ = audit_repo::log_for_backup("config", &format!("备份设置: 自动={} 间隔={}h 最大={}", b.enabled, b.interval_hours, b.max_backup_count.unwrap_or(10)));
    Json(ApiResponse::<String>::ok_msg(format!("自动备份已{}，重启后生效", if b.enabled { "启用" } else { "禁用" })))
}

async fn delete_backup(State(cfg): State<Arc<AppConfig>>, Path(fname): Path<String>) -> impl IntoResponse {
    if fname.contains("..") || fname.contains("/") || fname.contains("\\") { return Json(ApiResponse::<String>::ok_msg(String::from("非法文件名"))); }
    let path = cfg.backup_dir().join(&fname);
    match fs::remove_file(&path) { 
        Ok(_) => { let _ = audit_repo::log_for_backup("delete_backup", &format!("删除备份: {}", fname)); Json(ApiResponse::<String>::ok_msg(format!("已删除: {}", fname))) }
        Err(e) => Json(ApiResponse::<String>::ok_msg(format!("删除失败: {}", e))) 
    }
}
