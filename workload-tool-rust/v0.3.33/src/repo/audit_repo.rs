use crate::db::DbPool;
use crate::error::Result;
use crate::models::audit::AuditLogResponse;

/// 独立写入审计日志（不依赖连接池，由调用方提供数据库路径）
pub fn log_for_backup(action: &str, detail: &str) -> Result<()> {
    let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf())).unwrap_or_default();
    let db_path = exe_dir.join("data").join("workload.db");
    let conn = rusqlite::Connection::open(db_path)?;
    conn.execute(
        "INSERT INTO audit_log (action, table_name, record_id, user_name, detail, created_at) VALUES (?1,'backups',NULL,'system',?2, datetime('now','localtime'))",
        rusqlite::params!(action, detail),
    )?;
    Ok(())
}

pub fn log(pool: &DbPool, action: &str, table: &str, record_id: Option<i64>, user_name: &str, detail: &str) -> Result<()> {
    pool.get()?.execute(
        "INSERT INTO audit_log (action, table_name, record_id, user_name, detail, created_at) VALUES (?1,?2,?3,?4,?5, datetime('now','localtime'))",
        rusqlite::params!(action, table, record_id, user_name, detail),
    )?;
    Ok(())
}

/// Write audit log on an existing connection (same-connection, avoids pool contention)
pub fn log_on_conn(conn: &rusqlite::Connection, action: &str, table: &str, record_id: Option<i64>, user_name: &str, detail: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO audit_log (action, table_name, record_id, user_name, detail, created_at) VALUES (?1,?2,?3,?4,?5, datetime('now','localtime'))",
        rusqlite::params!(action, table, record_id, user_name, detail),
    )?;
    Ok(())
}

pub fn list(pool: &DbPool, page: i64, page_size: i64) -> Result<(Vec<AuditLogResponse>, i64)> {
    let conn = pool.get()?;
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM audit_log", [], |r| r.get(0))?;
    let offset = (page - 1) * page_size;
    let mut stmt = conn.prepare(
        "SELECT id, action, table_name, record_id, user_name, detail, created_at
         FROM audit_log ORDER BY created_at DESC LIMIT ?1 OFFSET ?2"
    )?;
    let rows = stmt.query_map([page_size, offset], |row| {
        Ok(AuditLogResponse {
            id: row.get(0)?, action: row.get(1)?, table_name: row.get(2)?,
            record_id: row.get(3)?, user_name: row.get::<_, String>(4).unwrap_or_default(),
            detail: row.get::<_, String>(5).unwrap_or_default(), created_at: row.get(6)?,
        })
    })?;
    let items: Vec<AuditLogResponse> = rows.collect::<std::result::Result<Vec<_>, _>>()?;
    Ok((items, count))
}
