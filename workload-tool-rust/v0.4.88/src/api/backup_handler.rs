use axum::{extract::{Multipart, State, Path}, Router, Json, routing::{get, post, delete}, response::IntoResponse, http::HeaderMap};
use serde::{Deserialize, Serialize};
use crate::config::AppConfig;
use crate::error::{AppError, Result};
use crate::models::ApiResponse;
use crate::repo::audit_repo;
use crate::service::auth_service;
use std::sync::Arc; use std::{fs, io::{Read, Write}}; use chrono::Local;
use rusqlite::Connection;
use zip::{write::SimpleFileOptions, ZipArchive, ZipWriter};

#[derive(Serialize)] pub struct BkStatus { pub auto_enabled: bool, pub auto_interval_hours: u64, pub max_backup_count: u64, pub backup_mode: String, pub backup_sync_dir: Option<String>, pub last_backup: Option<String>, pub backup_count: usize, pub backup_files: Vec<BkFile>, pub db_size: u64, pub tables: Vec<TableCount>, pub backups_dir: String }
#[derive(Serialize)] pub struct BkFile { pub name: String, pub size: u64, pub time: String }
#[derive(Serialize)] pub struct TableCount { pub table: String, pub rows: i64, pub label: String }
#[derive(Serialize)] pub struct BkConfig { pub enabled: bool, pub interval_hours: u64, pub max_backup_count: u64, pub mode: String, pub sync_dir: Option<String> }
#[derive(Deserialize)] pub struct BkUpdate { pub enabled: bool, pub interval_hours: u64, pub max_backup_count: Option<u64>, pub mode: Option<String>, pub sync_dir: Option<String> }

pub fn router(config: Arc<AppConfig>) -> Router {
    Router::new().route("/api/backup/status", get(status)).route("/api/backup/now", post(backup_now)).route("/api/backup/restore", post(restore)).route("/api/backup/restore/{fname}", post(restore_file)).route("/api/backup/config", get(get_config).put(update_config)).route("/api/backup/file/{fname}", delete(delete_backup)).with_state(config)
}

/// 从 HeaderMap 中提取 JWT claims 并校验管理员权限
fn extract_admin_from_headers(headers: &HeaderMap) -> Result<auth_service::Claims> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::Validation("未提供登录凭证".into()))?;
    let claims = auth_service::verify_token(token)?;
    if !claims.is_admin {
        return Err(AppError::Forbidden("需要管理员权限".into()));
    }
    Ok(claims)
}

/// 使用 VACUUM INTO 进行原子一致性备份
fn do_backup(db_path: &str, backup_dir: &std::path::Path) -> std::result::Result<(String, u64), String> {
    fs::create_dir_all(backup_dir).map_err(|e| e.to_string())?;
    let name = format!("workload_{}.db", Local::now().format("%Y%m%d_%H%M%S"));
    let dst = backup_dir.join(&name);
    let conn = Connection::open(db_path).map_err(|e| format!("无法打开数据库: {}", e))?;
    conn.execute_batch(&format!("VACUUM INTO '{}'", dst.to_string_lossy().replace('\'', "''"))).map_err(|e| format!("备份失败: {}", e))?;
    let size = fs::metadata(&dst).map(|m| m.len()).unwrap_or(0);
    Ok((name, size))
}

fn create_configured_backup(cfg: &AppConfig) -> std::result::Result<(String, u64), String> {
    let backup_dir = cfg.backup_dir();
    let result = if cfg.backup_mode.trim().eq_ignore_ascii_case("full") {
        do_full_backup(cfg, &backup_dir)
    } else {
        do_backup(&cfg.db_path().to_string_lossy(), &backup_dir)
    }?;
    sync_backup_file(cfg, &backup_dir.join(&result.0)).ok();
    Ok(result)
}

fn do_full_backup(cfg: &AppConfig, backup_dir: &std::path::Path) -> std::result::Result<(String, u64), String> {
    fs::create_dir_all(backup_dir).map_err(|e| e.to_string())?;
    let name = format!("workload_full_{}.zip", Local::now().format("%Y%m%d_%H%M%S"));
    let dst = backup_dir.join(&name);
    let snapshot = backup_dir.join(format!("workload_snapshot_{}.db", uuid::Uuid::new_v4()));
    let conn = Connection::open(cfg.db_path()).map_err(|e| e.to_string())?;
    conn.execute_batch(&format!("VACUUM INTO '{}'", snapshot.to_string_lossy().replace('\'', "''"))).map_err(|e| e.to_string())?;
    let file = fs::File::create(&dst).map_err(|e| e.to_string())?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    zip.start_file("workload.db", options).map_err(|e| e.to_string())?;
    let mut bytes = Vec::new();
    fs::File::open(&snapshot).map_err(|e| e.to_string())?.read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    zip.write_all(&bytes).map_err(|e| e.to_string())?;
    add_dir_to_zip(&mut zip, &cfg.attachments_dir(), "attachments", options)?;
    zip.finish().map_err(|e| e.to_string())?;
    let _ = fs::remove_file(snapshot);
    let size = fs::metadata(&dst).map(|m| m.len()).unwrap_or(0);
    Ok((name, size))
}

fn add_dir_to_zip(zip: &mut ZipWriter<fs::File>, dir: &std::path::Path, zip_root: &str, options: SimpleFileOptions) -> std::result::Result<(), String> {
    if !dir.exists() { return Ok(()); }
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let rel = path.strip_prefix(dir).map_err(|e| e.to_string())?;
        let zip_name = format!("{}/{}", zip_root, rel.to_string_lossy().replace('\\', "/"));
        if path.is_dir() {
            add_dir_to_zip(zip, &path, &zip_name, options)?;
        } else {
            zip.start_file(zip_name, options).map_err(|e| e.to_string())?;
            let mut bytes = Vec::new();
            fs::File::open(&path).map_err(|e| e.to_string())?.read_to_end(&mut bytes).map_err(|e| e.to_string())?;
            zip.write_all(&bytes).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn sync_backup_file(cfg: &AppConfig, path: &std::path::Path) -> std::result::Result<(), String> {
    let Some(sync_dir) = cfg.backup_sync_dir.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) else { return Ok(()); };
    let dst_dir = std::path::PathBuf::from(sync_dir);
    fs::create_dir_all(&dst_dir).map_err(|e| e.to_string())?;
    let file_name = path.file_name().ok_or_else(|| "invalid backup file".to_string())?;
    fs::copy(path, dst_dir.join(file_name)).map_err(|e| e.to_string())?;
    Ok(())
}

fn table_counts(db_path: &str) -> std::result::Result<Vec<TableCount>, String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    let specs: &[(&str, &str)] = &[
        ("work_records", "分析检测记录"),
        ("sample_records", "送样记录(已退役)"),
        ("rd_work_records", "研发送样记录"),
    ];
    let mut counts = vec![];
    for (t, label) in specs {
        if let Ok(n) = conn.query_row(&format!("SELECT COUNT(*) FROM {} WHERE deleted_at IS NULL", t), [], |r| r.get::<_, i64>(0)) {
            counts.push(TableCount { table: t.to_string(), rows: n, label: label.to_string() });
        }
    }
    Ok(counts)
}

fn verify_backup(path: &std::path::Path) -> std::result::Result<String, String> {
    let conn = Connection::open(path).map_err(|e| format!("验证失败: {}", e))?;
    let ok: String = conn.query_row("PRAGMA integrity_check", [], |r| r.get(0)).map_err(|e| e.to_string())?;
    if ok == "ok" { Ok(ok) } else { Err(format!("数据库损坏: {}", ok)) }
}

fn restore_full_backup(cfg: &AppConfig, zip_path: &std::path::Path) -> std::result::Result<String, String> {
    let file = fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;
    let tmp_db = std::env::temp_dir().join(format!("restore_full_{}.db", uuid::Uuid::new_v4()));
    {
        let mut db_entry = archive.by_name("workload.db").map_err(|_| "missing workload.db".to_string())?;
        let mut out = fs::File::create(&tmp_db).map_err(|e| e.to_string())?;
        std::io::copy(&mut db_entry, &mut out).map_err(|e| e.to_string())?;
    }
    verify_backup(&tmp_db)?;
    let old = do_backup(&cfg.db_path().to_string_lossy(), &cfg.backup_dir()).map(|(n, _)| n).unwrap_or_else(|_| "unknown".into());
    fs::copy(&tmp_db, cfg.db_path()).map_err(|e| e.to_string())?;
    let _ = fs::remove_file(&tmp_db);
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().replace('\\', "/");
        if name.starts_with("attachments/") && !entry.is_dir() {
            let rel = name.trim_start_matches("attachments/");
            if rel.contains("..") { continue; }
            let dst = cfg.attachments_dir().join(rel);
            if let Some(parent) = dst.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
            let mut out = fs::File::create(dst).map_err(|e| e.to_string())?;
            std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
        }
    }
    Ok(old)
}

async fn status(State(cfg): State<Arc<AppConfig>>) -> impl IntoResponse {
    let dir = cfg.backup_dir(); let mut files: Vec<BkFile> = vec![];
    if let Ok(es) = fs::read_dir(&dir) {
        for e in es.flatten() {
            if let Ok(n) = e.file_name().into_string() {
                if n.ends_with(".db") || n.ends_with(".zip") {
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
    Json(ApiResponse::ok(BkStatus { auto_enabled: cfg.backup_enabled, auto_interval_hours: cfg.backup_interval_hours, max_backup_count: cfg.max_backup_count, backup_mode: cfg.backup_mode.clone(), backup_sync_dir: cfg.backup_sync_dir.clone(), last_backup: files.first().map(|f| f.name.clone()), backup_count: files.len(), backup_files: files, db_size, tables, backups_dir: dir.to_string_lossy().to_string() }))
}

async fn backup_now(State(cfg): State<Arc<AppConfig>>, headers: HeaderMap) -> impl IntoResponse {
    if let Err(e) = extract_admin_from_headers(&headers) {
        return Json(ApiResponse::<String>::ok_msg(format!("{}", e)));
    }
    let db = cfg.db_path().to_string_lossy().to_string();
    let backup_dir = cfg.backup_dir();
    match create_configured_backup(&cfg) {
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
            if !name.ends_with(".db") && !name.ends_with(".zip") { return None; }
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
async fn restore(State(cfg): State<Arc<AppConfig>>, headers: HeaderMap, mut mp: Multipart) -> impl IntoResponse {
    if let Err(e) = extract_admin_from_headers(&headers) {
        return Json(ApiResponse::<String>::ok_msg(format!("{}", e)));
    }
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
async fn restore_file(State(cfg): State<Arc<AppConfig>>, headers: HeaderMap, Path(fname): Path<String>) -> impl IntoResponse {
    if let Err(e) = extract_admin_from_headers(&headers) {
        return Json(ApiResponse::<String>::ok_msg(format!("{}", e)));
    }
    if fname.contains("..") || fname.contains("/") || fname.contains("\\") { return Json(ApiResponse::<String>::ok_msg(String::from("非法文件名"))); }
    let src = cfg.backup_dir().join(&fname);
    if !src.exists() { return Json(ApiResponse::<String>::ok_msg(format!("备份文件不存在: {}", fname))); }
    if let Err(e) = verify_backup(&src) {
        return Json(ApiResponse::<String>::ok_msg(format!("备份文件无效: {}", e)));
    }
    let db = cfg.db_path().to_string_lossy().to_string();
    let bk = do_backup(&db, &cfg.backup_dir()).map(|(n, _)| n).unwrap_or_else(|_| "unknown".into());
    match fs::copy(&src, &db) {
        Ok(_) => {
            let _ = audit_repo::log_for_backup("restore", &format!("文件恢复: {} | 旧库: {}", fname, bk));
            Json(ApiResponse::<String>::ok_msg(format!("恢复成功: {}。请重启程序以使新数据库生效。旧库已备份为 {}", fname, bk)))
        }
        Err(e) => Json(ApiResponse::<String>::ok_msg(format!("恢复失败（数据库可能被占用，请关闭程序后重试）: {}", e)))
    }
}

async fn get_config(State(cfg): State<Arc<AppConfig>>) -> impl IntoResponse { Json(ApiResponse::ok(BkConfig { enabled: cfg.backup_enabled, interval_hours: cfg.backup_interval_hours, max_backup_count: cfg.max_backup_count, mode: cfg.backup_mode.clone(), sync_dir: cfg.backup_sync_dir.clone() })) }

async fn update_config(State(cfg): State<Arc<AppConfig>>, headers: HeaderMap, Json(b): Json<BkUpdate>) -> impl IntoResponse {
    if let Err(e) = extract_admin_from_headers(&headers) {
        return Json(ApiResponse::<String>::ok_msg(format!("{}", e)));
    }
    let mut new_cfg = (*cfg).clone();
    new_cfg.backup_enabled = b.enabled;
    new_cfg.backup_interval_hours = b.interval_hours;
    if let Some(mc) = b.max_backup_count { new_cfg.max_backup_count = mc; }
    if let Some(mode) = b.mode { new_cfg.backup_mode = if mode == "full" { "full".into() } else { "database".into() }; }
    if let Some(sync_dir) = b.sync_dir {
        let trimmed = sync_dir.trim().to_string();
        new_cfg.backup_sync_dir = if trimmed.is_empty() { None } else { Some(trimmed) };
    }
    new_cfg.save();
    let _ = audit_repo::log_for_backup("config", &format!("备份设置: 自动={} 间隔={}h 最大={}", b.enabled, b.interval_hours, b.max_backup_count.unwrap_or(10)));
    Json(ApiResponse::<String>::ok_msg(format!("自动备份已{}，重启后生效", if b.enabled { "启用" } else { "禁用" })))
}

async fn delete_backup(State(cfg): State<Arc<AppConfig>>, headers: HeaderMap, Path(fname): Path<String>) -> impl IntoResponse {
    if let Err(e) = extract_admin_from_headers(&headers) {
        return Json(ApiResponse::<String>::ok_msg(format!("{}", e)));
    }
    if fname.contains("..") || fname.contains("/") || fname.contains("\\") { return Json(ApiResponse::<String>::ok_msg(String::from("非法文件名"))); }
    let path = cfg.backup_dir().join(&fname);
    match fs::remove_file(&path) { 
        Ok(_) => { let _ = audit_repo::log_for_backup("delete_backup", &format!("删除备份: {}", fname)); Json(ApiResponse::<String>::ok_msg(format!("已删除: {}", fname))) }
        Err(e) => Json(ApiResponse::<String>::ok_msg(format!("删除失败: {}", e))) 
    }
}
