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
    if let Some(pid) = project_id { where_clauses.push(format!("wr.project_id={}", pid)); }
    if let Some(un) = user_name { where_clauses.push("wr.user_name=?1".to_string()); params.push(Box::new(un.to_string())); }
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

pub fn get_by_id(pool: &DbPool, id: i64) -> Result<RecordResponse> {
    let conn = pool.get()?;
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

pub fn create(pool: &DbPool, body: &RecordCreate) -> Result<RecordResponse> {
    let conn = pool.get()?;
    conn.execute(
        "INSERT INTO work_records (project_id, user_name, quantity, recorded_at)
         VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params!(body.project_id, &body.user_name, body.quantity, &body.recorded_at),
    )?;
    let id = conn.last_insert_rowid();
    audit_repo::log(pool, "create", "work_records", Some(id), &body.user_name, "创建记录")?;
    get_by_id(pool, id)
}

pub fn update(pool: &DbPool, id: i64, body: &RecordUpdate, user_name: &str) -> Result<RecordResponse> {
    let conn = pool.get()?;
    let mut updated = false;
    if let Some(ref un) = body.user_name {
        conn.execute("UPDATE work_records SET user_name=?1 WHERE id=?2 AND deleted_at IS NULL", (un, id))?;
        updated = true;
    }
    if let Some(q) = body.quantity {
        conn.execute("UPDATE work_records SET quantity=?1 WHERE id=?2 AND deleted_at IS NULL", (q, id))?;
        updated = true;
    }
    if let Some(ref dt) = body.recorded_at {
        conn.execute("UPDATE work_records SET recorded_at=?1 WHERE id=?2 AND deleted_at IS NULL", (dt, id))?;
        updated = true;
    }
    if updated {
        audit_repo::log(pool, "update", "work_records", Some(id), user_name, "编辑记录")?;
    }
    get_by_id(pool, id)
}

pub fn soft_delete(pool: &DbPool, id: i64, user_name: &str) -> Result<()> {
    let conn = pool.get()?;
    let r = get_by_id(pool, id)?;
    if r.deleted_at.is_some() { return Err(crate::error::AppError::Validation("记录已被删除".into())); }
    conn.execute(
        "UPDATE work_records SET deleted_at=datetime('now') WHERE id=?1", [id],
    )?;
    audit_repo::log(pool, "delete", "work_records", Some(id), user_name, "软删除记录")?;
    Ok(())
}

pub fn restore(pool: &DbPool, id: i64, user_name: &str) -> Result<RecordResponse> {
    let conn = pool.get()?;
    conn.execute("UPDATE work_records SET deleted_at=NULL WHERE id=?1", [id])?;
    audit_repo::log(pool, "restore", "work_records", Some(id), user_name, "恢复记录")?;
    get_by_id(pool, id)
}

pub fn delete_by_user(pool: &DbPool, user_name: &str, start: Option<&str>, end: Option<&str>) -> Result<i64> {
    let conn = pool.get()?;
    let mut sql = "UPDATE work_records SET deleted_at=datetime('now') WHERE user_name=?1 AND deleted_at IS NULL".to_string();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(user_name.to_string())];
    if let Some(s) = start { let i = params.len()+1; sql.push_str(&format!(" AND recorded_at>=?{}",i)); params.push(Box::new(s.to_string())); }
    if let Some(e) = end { let i = params.len()+1; sql.push_str(&format!(" AND recorded_at<=?{}",i)); params.push(Box::new(format!("{}T23:59:59",e))); }
    let count = conn.execute(&sql, rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())))?;
    audit_repo::log(pool, "batch_delete", "work_records", None, user_name, &format!("批量删除 {} 条", count))?;
    Ok(count as i64)
}
