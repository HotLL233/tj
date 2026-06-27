use crate::db::DbPool;
use crate::error::Result;
use crate::models::record::{RecordCreate, RecordResponse, RecordUpdate};
use crate::repo::audit_repo;

pub fn list(
    pool: &DbPool, project_id: Option<i64>, user_name: Option<&str>,
    start: Option<&str>, end: Option<&str>, page: i64, page_size: i64,
    include_deleted: bool,
) -> Result<(Vec<RecordResponse>, i64)> {
    let conn = pool.get()?;
    let mut where_clauses = vec![];
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];

    if !include_deleted { where_clauses.push("wr.deleted_at IS NULL".to_string()); }
    if let Some(pid) = project_id { let idx = params.len() + 1; where_clauses.push(format!("wr.project_id=?{}", idx)); params.push(Box::new(pid)); }
    if let Some(un) = user_name { let idx = params.len() + 1; where_clauses.push(format!("wr.user_name=?{}", idx)); params.push(Box::new(un.to_string())); }
    if let Some(s) = start { let idx = params.len() + 1; where_clauses.push(format!("wr.recorded_at>=?{}", idx)); params.push(Box::new(s.to_string())); }
    if let Some(e) = end { let idx = params.len() + 1; where_clauses.push(format!("wr.recorded_at<=?{}", idx)); params.push(Box::new(format!("{}T23:59:59", e))); }

    let where_sql = if where_clauses.is_empty() { String::new() } else { format!("WHERE {}", where_clauses.join(" AND ")) };

    let count: i64 = conn.query_row(
        &format!("SELECT COUNT(*) FROM work_records wr {}", where_sql),
        rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
        |r| r.get(0),
    )?;

    let offset = (page - 1) * page_size;
    let sql = format!(
        "SELECT wr.id, wr.project_id, p.name, pg.name, wr.user_name, wr.quantity,
                wr.recorded_at, wr.created_at, wr.deleted_at
         FROM work_records wr
         JOIN projects p ON wr.project_id = p.id
         JOIN project_groups pg ON p.group_id = pg.id
         {} ORDER BY wr.recorded_at DESC, pg.sort_order, p.sort_order
         LIMIT {} OFFSET {}",
        where_sql, page_size, offset
    );

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(
        rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
        |row| Ok(RecordResponse {
            id: row.get(0)?, project_id: row.get(1)?, project_name: row.get(2)?,
            group_name: row.get(3)?, user_name: row.get(4)?, quantity: row.get(5)?,
            recorded_at: row.get(6)?, created_at: row.get(7)?, deleted_at: row.get(8)?,
        }),
    )?;
    let items: Vec<RecordResponse> = rows.collect::<std::result::Result<Vec<_>, _>>()?;
    Ok((items, count))
}

/// Internal helper: query a single record on an existing connection.
fn get_by_id_on_conn(conn: &rusqlite::Connection, id: i64) -> Result<RecordResponse> {
    conn.query_row(
        "SELECT wr.id, wr.project_id, p.name, pg.name, wr.user_name, wr.quantity,
                wr.recorded_at, wr.created_at, wr.deleted_at
         FROM work_records wr
         JOIN projects p ON wr.project_id = p.id
         JOIN project_groups pg ON p.group_id = pg.id
         WHERE wr.id=?1", [id],
        |row| Ok(RecordResponse {
            id: row.get(0)?, project_id: row.get(1)?, project_name: row.get(2)?,
            group_name: row.get(3)?, user_name: row.get(4)?, quantity: row.get(5)?,
            recorded_at: row.get(6)?, created_at: row.get(7)?, deleted_at: row.get(8)?,
        }),
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => crate::error::AppError::NotFound("记录不存在".into()),
        _ => e.into(),
    })
}

pub fn get_by_id(pool: &DbPool, id: i64) -> Result<RecordResponse> {
    let conn = pool.get()?;
    get_by_id_on_conn(&conn, id)
}

pub fn create(pool: &DbPool, body: &RecordCreate) -> Result<RecordResponse> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO work_records (project_id, user_name, quantity, recorded_at)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params!(body.project_id, &body.user_name, body.quantity, &body.recorded_at),
    )?;
    let id = tx.last_insert_rowid();
    audit_repo::log_on_conn(&tx, "create", "work_records", Some(id), &body.user_name, "创建记录")?;
    tx.commit()?;
    get_by_id_on_conn(&conn, id)
}

pub fn update(pool: &DbPool, id: i64, body: &RecordUpdate, user_name: &str) -> Result<RecordResponse> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;

    // Existence + soft-delete check on same connection
    let existing = get_by_id_on_conn(&tx, id)?;
    if existing.deleted_at.is_some() {
        return Err(crate::error::AppError::Validation("记录已被删除，无法编辑".into()));
    }

    // Build dynamic UPDATE with only changed fields in a single statement
    let mut sets: Vec<&str> = vec![];
    let mut has_changes = false;
    if body.user_name.is_some() { sets.push("user_name=?1"); has_changes = true; }
    if body.quantity.is_some() { sets.push("quantity=?2"); has_changes = true; }
    if body.recorded_at.is_some() { sets.push("recorded_at=?3"); has_changes = true; }
    if !has_changes {
        return Err(crate::error::AppError::Validation("没有需要更新的字段".into()));
    }
    let set_clause = sets.join(", ");
    let rows = tx.execute(&format!("UPDATE work_records SET {} WHERE id=?4", set_clause), rusqlite::params![
        body.user_name,
        body.quantity,
        body.recorded_at,
        id,
    ])?;
    if rows == 0 {
        return Err(crate::error::AppError::NotFound("记录不存在".into()));
    }
    audit_repo::log_on_conn(&tx, "update", "work_records", Some(id), user_name, "编辑记录")?;
    tx.commit()?;
    get_by_id_on_conn(&conn, id)
}

pub fn soft_delete(pool: &DbPool, id: i64, user_name: &str) -> Result<()> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    let deleted: Option<String> = tx.query_row(
        "SELECT deleted_at FROM work_records WHERE id=?1", [id], |r| r.get(0)
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => crate::error::AppError::NotFound("记录不存在".into()),
        _ => e.into(),
    })?;
    if deleted.is_some() { return Err(crate::error::AppError::Validation("记录已被删除".into())); }
    let rows = tx.execute("UPDATE work_records SET deleted_at=datetime('now') WHERE id=?1", [id])?;
    if rows == 0 {
        return Err(crate::error::AppError::NotFound("记录不存在".into()));
    }
    audit_repo::log_on_conn(&tx, "delete", "work_records", Some(id), user_name, "软删除记录")?;
    tx.commit()?;
    Ok(())
}

pub fn restore(pool: &DbPool, id: i64, user_name: &str) -> Result<RecordResponse> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;

    // Verify record exists and is indeed in a deleted state
    let deleted: Option<String> = tx.query_row(
        "SELECT deleted_at FROM work_records WHERE id=?1", [id], |r| r.get(0)
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => crate::error::AppError::NotFound("记录不存在".into()),
        _ => e.into(),
    })?;
    if deleted.is_none() {
        return Err(crate::error::AppError::Validation("记录未被删除，无需恢复".into()));
    }

    let rows = tx.execute("UPDATE work_records SET deleted_at=NULL WHERE id=?1", [id])?;
    if rows == 0 {
        return Err(crate::error::AppError::NotFound("记录不存在".into()));
    }
    audit_repo::log_on_conn(&tx, "restore", "work_records", Some(id), user_name, "恢复记录")?;
    tx.commit()?;
    get_by_id_on_conn(&conn, id)
}

pub fn delete_by_user(pool: &DbPool, user_name: &str, start: Option<&str>, end: Option<&str>) -> Result<i64> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    let mut sql = "UPDATE work_records SET deleted_at=datetime('now') WHERE user_name=?1 AND deleted_at IS NULL".to_string();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(user_name.to_string())];
    if let Some(s) = start { let i = params.len()+1; sql.push_str(&format!(" AND recorded_at>=?{}",i)); params.push(Box::new(s.to_string())); }
    if let Some(e) = end { let i = params.len()+1; sql.push_str(&format!(" AND recorded_at<=?{}",i)); params.push(Box::new(format!("{}T23:59:59",e))); }
    let deleted_count = tx.execute(&sql, rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())))?;
    audit_repo::log_on_conn(&tx, "batch_delete", "work_records", None, user_name, &format!("批量删除 {} 条", deleted_count))?;
    tx.commit()?;
    Ok(deleted_count as i64)
}

