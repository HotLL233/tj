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

/// 模块感知审计 — 追加 module 列（work=分析检测, rd=研发送样, shared=主数据）
pub fn log_with_module(pool: &DbPool, action: &str, table: &str, record_id: Option<i64>, user_name: &str, detail: &str, module: &str) -> Result<()> {
    pool.get()?.execute(
        "INSERT INTO audit_log (action, table_name, record_id, user_name, detail, module, created_at) VALUES (?1,?2,?3,?4,?5,?6, datetime('now','localtime'))",
        rusqlite::params!(action, table, record_id, user_name, detail, module),
    )?;
    Ok(())
}

/// 模块感知审计 — 在已有连接上写入（同连接，避免连接池争用）
pub fn log_on_conn_with_module(conn: &rusqlite::Connection, action: &str, table: &str, record_id: Option<i64>, user_name: &str, detail: &str, module: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO audit_log (action, table_name, record_id, user_name, detail, module, created_at) VALUES (?1,?2,?3,?4,?5,?6, datetime('now','localtime'))",
        rusqlite::params!(action, table, record_id, user_name, detail, module),
    )?;
    Ok(())
}

/// 审计日志列表，支持按 module 过滤：
/// - work → 分析检测录入 + 共享主数据（module IN ('work','shared')）
/// - rd  → 研发送样录入 + 共享主数据（module IN ('rd','shared')）
/// - shared → 仅主数据
/// - None → 全部
pub fn list(pool: &DbPool, page: i64, page_size: i64, module: Option<&str>) -> Result<(Vec<AuditLogResponse>, i64)> {
    let conn = pool.get()?;
    let (count_sql, where_clause): (&str, String) = match module {
        Some("work") => ("SELECT COUNT(*) FROM audit_log WHERE module IN ('work','shared')", "WHERE module IN ('work','shared')".to_string()),
        Some("rd") => ("SELECT COUNT(*) FROM audit_log WHERE module IN ('rd','shared')", "WHERE module IN ('rd','shared')".to_string()),
        Some(m) => ("SELECT COUNT(*) FROM audit_log WHERE module=?1", format!("WHERE module='{}'", m.replace('\'', "''"))),
        None => ("SELECT COUNT(*) FROM audit_log", String::new()),
    };
    let count: i64 = if let Some(m) = module { if m == "work" || m == "rd" { conn.query_row(count_sql, [], |r| r.get(0))? } else { conn.query_row(count_sql, [m], |r| r.get(0))? } } else { conn.query_row(count_sql, [], |r| r.get(0))? };
    let offset = (page - 1) * page_size;
    let base = "SELECT id, action, table_name, record_id, user_name, detail, module, created_at FROM audit_log";
    let sql = if where_clause.is_empty() {
        format!("{} ORDER BY created_at DESC LIMIT ?1 OFFSET ?2", base)
    } else {
        format!("{} {} ORDER BY created_at DESC LIMIT ?1 OFFSET ?2", base, where_clause)
    };
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([page_size, offset], |row| {
        Ok(AuditLogResponse {
            id: row.get(0)?, action: row.get(1)?, table_name: row.get(2)?,
            record_id: row.get(3)?, user_name: row.get::<_, String>(4).unwrap_or_default(),
            detail: row.get::<_, String>(5).unwrap_or_default(),
            module: row.get::<_, String>(6).unwrap_or_else(|_| "shared".to_string()),
            created_at: row.get(7)?,
        })
    })?;
    let items: Vec<AuditLogResponse> = rows.collect::<std::result::Result<Vec<_>, _>>()?;
    Ok((items, count))
}
