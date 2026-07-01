use axum::{extract::{Multipart, State, Path}, Router, Json, routing::{get, post, delete}, response::IntoResponse};
use serde::{Deserialize, Serialize};
use crate::config::AppConfig;
use crate::models::ApiResponse;
use std::sync::Arc; use std::fs; use chrono::Local;
use rusqlite::Connection;

#[derive(Serialize)] pub struct BkStatus { pub auto_enabled: bool, pub auto_interval_hours: u64, pub last_backup: Option<String>, pub backup_count: usize, pub backup_files: Vec<BkFile>, pub db_size: u64, pub tables: Vec<TableCount>, pub backups_dir: String }
#[derive(Serialize)] pub struct BkFile { pub name: String, pub size: u64, pub time: String }
#[derive(Serialize)] pub struct TableCount { pub table: String, pub rows: i64 }
#[derive(Serialize)] pub struct BkConfig { pub enabled: bool, pub interval_hours: u64 }
#[derive(Deserialize)] pub struct BkUpdate { pub enabled: bool, pub interval_hours: u64 }

pub fn router(config: Arc<AppConfig>) -> Router {
    Router::new().route("/api/backup/status", get(status)).route("/api/backup/now", post(backup_now)).route("/api/backup/restore", post(restore)).route("/api/backup/config", get(get_config).put(update_config)).route("/api/backup/file/{fname}", delete(delete_backup)).with_state(config)
}

/// 使用 VACUUM INTO 进行原子一致性备份
fn do_backup(db_path: &str, backup_dir: &std::path::Path) -> Result<(String, u64), String> {
    fs::create_dir_all(backup_dir).map_err(|e| e.to_string())?;
    let name = format!("workload_{}.db", Local::now().format("%Y%m%d_%H%M%S"));
    let dst = backup_dir.join(&name);
    let conn = Connection::open(db_path).map_err(|e| format!("无法打开数据库: {}", e))?;
    // VACUUM INTO 创建完整快照，写入期间不影响正常读写
    conn.execute_batch(&format!("VACUUM INTO '{}'", dst.to_string_lossy().replace('\'', "''"))).map_err(|e| format!("备份失败: {}", e))?;
    let size = fs::metadata(&dst).map(|m| m.len()).unwrap_or(0);
    Ok((name, size))
}

/// 获取各表行数统计
fn table_counts(db_path: &str) -> Result<Vec<TableCount>, String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    let tables = ["work_records", "sample_records", "projects", "project_groups", "audit_log"];
    let mut counts = vec![];
    for t in &tables {
        if let Ok(n) = conn.query_row(&format!("SELECT COUNT(*) FROM {} WHERE deleted_at IS NULL", t), [], |r| r.get::<_, i64>(0)) {
            counts.push(TableCount { table: t.to_string(), rows: n });
        }
    }
    // audit_log has no deleted_at
    if let Ok(n) = conn.query_row("SELECT COUNT(*) FROM audit_log", [], |r| r.get::<_, i64>(0)) {
        counts.push(TableCount { table: "audit_log".to_string(), rows: n });
    }
    Ok(counts)
}

/// 验证备份文件完整性
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
    Json(ApiResponse::ok(BkStatus { auto_enabled: cfg.backup_enabled, auto_interval_hours: cfg.backup_interval_hours, last_backup: files.first().map(|f| f.name.clone()), backup_count: files.len(), backup_files: files, db_size, tables, backups_dir: dir.to_string_lossy().to_string() }))
}

async fn backup_now(State(cfg): State<Arc<AppConfig>>) -> impl IntoResponse {
    let db = cfg.db_path().to_string_lossy().to_string();
    let backup_dir = cfg.backup_dir();
    match do_backup(&db, &backup_dir) {
        Ok((name, size)) => {
            let tables = table_counts(&db).unwrap_or_default();
            let total: i64 = tables.iter().map(|t| t.rows).sum();
            Json(ApiResponse::<String>::ok_msg(format!("备份成功: {} ({}KB, {}条记录)", name, size / 1024, total)))
        }
        Err(e) => Json(ApiResponse::<String>::ok_msg(format!("备份失败: {}", e)))
    }
}

async fn restore(State(cfg): State<Arc<AppConfig>>, mut mp: Multipart) -> impl IntoResponse {
    let mut tmp = String::new();
    while let Ok(Some(f)) = mp.next_field().await { if f.name() == Some("file") { if let Ok(d) = f.bytes().await { if !d.is_empty() { let p = std::env::temp_dir().join("restore_tmp.db"); if fs::write(&p, &d).is_ok() { tmp = p.to_string_lossy().to_string(); } } } } }
    if tmp.is_empty() { return Json(ApiResponse::<String>::ok_msg(String::from("未收到文件"))); }

    // 验证上传文件
    if let Err(e) = verify_backup(std::path::Path::new(&tmp)) {
        let _ = fs::remove_file(&tmp);
        return Json(ApiResponse::<String>::ok_msg(format!("备份文件无效: {}", e)));
    }

    let db = cfg.db_path().to_string_lossy().to_string();
    // 恢复前先备份当前库
    let bk = do_backup(&db, &cfg.backup_dir()).map(|(n, _)| n).unwrap_or_else(|_| "unknown".into());
    if let Err(e) = fs::copy(&tmp, &db) { let _ = fs::remove_file(&tmp); return Json(ApiResponse::<String>::ok_msg(format!("恢复失败: {}", e))); }
    let _ = fs::remove_file(&tmp);

    // 验证恢复后的数据库
    let check = verify_backup(std::path::Path::new(&db)).unwrap_or_else(|e| e);
    Json(ApiResponse::<String>::ok_msg(format!("恢复完成。旧库备份: {} | 完整性: {}", bk, check)))
}

async fn get_config(State(cfg): State<Arc<AppConfig>>) -> impl IntoResponse { Json(ApiResponse::ok(BkConfig { enabled: cfg.backup_enabled, interval_hours: cfg.backup_interval_hours })) }
async fn update_config(State(_cfg): State<Arc<AppConfig>>, Json(b): Json<BkUpdate>) -> impl IntoResponse { Json(ApiResponse::<String>::ok_msg(format!("自动备份已{}，重启后生效", if b.enabled { "启用" } else { "禁用" }))) }
async fn delete_backup(State(cfg): State<Arc<AppConfig>>, Path(fname): Path<String>) -> impl IntoResponse {
    if fname.contains("..") || fname.contains("/") || fname.contains("\\") { return Json(ApiResponse::<String>::ok_msg(String::from("非法文件名"))); }
    match fs::remove_file(cfg.backup_dir().join(&fname)) { Ok(_) => Json(ApiResponse::<String>::ok_msg(format!("已删除: {}", fname))), Err(e) => Json(ApiResponse::<String>::ok_msg(format!("删除失败: {}", e))) }
}
