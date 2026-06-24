use crate::db::DbPool;
use crate::error::Result;
use crate::models::audit::AuditLogResponse;

pub fn log(pool: &DbPool, action: &str, table: &str, record_id: Option<i64>, user_name: &str, detail: &str) -> Result<()> {
    pool.get()?.execute(
        "INSERT INTO audit_log (action, table_name, record_id, user_name, detail) VALUES (?1,?2,?3,?4,?5)",
        rusqlite::params!(action, table, record_id, user_name, detail),
    )?;
    Ok(())
}

/// Write audit log on an existing connection (same-connection, avoids pool contention)
pub fn log_on_conn(conn: &rusqlite::Connection, action: &str, table: &str, record_id: Option<i64>, user_name: &str, detail: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO audit_log (action, table_name, record_id, user_name, detail) VALUES (?1,?2,?3,?4,?5)",
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
