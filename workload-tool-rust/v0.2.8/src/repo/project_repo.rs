use rusqlite::Connection;
use crate::db::DbPool;
use crate::error::{Result, AppError};
use crate::models::project::*;

// ── 项目 CRUD ──

pub fn list(pool: &DbPool, group_id: Option<i64>, active_only: bool, method_type: Option<&str>) -> Result<Vec<ProjectResponse>> {
    let conn = pool.get()?;
    let mut sql = String::from("SELECT p.id, p.group_id, pg.name, p.name, COALESCE(p.full_name,''), COALESCE(p.notes,''), p.sort_order, p.is_active, COALESCE(p.coefficient,1.0), p.method_type, COALESCE(p.created_at,'') FROM projects p JOIN project_groups pg ON p.group_id=pg.id WHERE 1=1");
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
    if let Some(gid) = group_id { if gid > 0 { sql.push_str(" AND p.group_id=?"); params.push(Box::new(gid)); } }
    if active_only { sql.push_str(" AND p.is_active=1"); }
    if let Some(mt) = method_type { if mt != "全部" && !mt.is_empty() { sql.push_str(" AND p.method_type=?"); params.push(Box::new(mt.to_string())); } }
    sql.push_str(" ORDER BY CASE p.method_type WHEN '液相' THEN 0 WHEN '气相' THEN 1 WHEN '理化' THEN 2 WHEN '检测类型' THEN 3 ELSE 4 END, pg.sort_order, p.sort_order");
    let mut stmt = conn.prepare(&sql)?;
    let refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let rows = stmt.query_map(rusqlite::params_from_iter(refs.iter()), |row| Ok(ProjectResponse {
        id: row.get(0)?, group_id: row.get(1)?, group_name: row.get(2)?, name: row.get(3)?, full_name: row.get(4)?,
        notes: row.get(5)?, sort_order: row.get(6)?, is_active: row.get::<_, i64>(7).unwrap_or(1) != 0,
        coefficient: row.get::<_, f64>(8).unwrap_or(1.0), method_type: row.get(9)?, created_at: row.get(10)?,
    }))?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

pub fn get_by_id(pool: &DbPool, id: i64) -> Result<ProjectResponse> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare("SELECT p.id, p.group_id, pg.name, p.name, COALESCE(p.full_name,''), COALESCE(p.notes,''), p.sort_order, p.is_active, COALESCE(p.coefficient,1.0), p.method_type, COALESCE(p.created_at,'') FROM projects p JOIN project_groups pg ON p.group_id=pg.id WHERE p.id=?")?;
    stmt.query_row([id], |row| Ok(ProjectResponse {
        id: row.get(0)?, group_id: row.get(1)?, group_name: row.get(2)?, name: row.get(3)?, full_name: row.get(4)?,
        notes: row.get(5)?, sort_order: row.get(6)?, is_active: row.get::<_, i64>(7).unwrap_or(1) != 0,
        coefficient: row.get::<_, f64>(8).unwrap_or(1.0), method_type: row.get(9)?, created_at: row.get(10)?,
    })).map_err(|e| AppError::NotFound(format!("项目不存在: {}", e)))
}

pub fn create(pool: &DbPool, body: &ProjectCreate) -> Result<ProjectResponse> {
    let conn = pool.get()?;
    let so = body.sort_order.unwrap_or(0);
    let cf = body.coefficient.unwrap_or(1.0);
    let mt = body.method_type.as_deref().unwrap_or("其他");
    conn.execute("INSERT INTO projects (group_id, name, sort_order, coefficient, method_type) VALUES (?1,?2,?3,?4,?5)",
        rusqlite::params![body.group_id, body.name, so, cf, mt])?;
    get_by_id(pool, conn.last_insert_rowid())
}

pub fn batch_import(_pool: &DbPool, conn: &Connection, items: &[MethodImportItem]) -> Result<usize> {
    let mut count = 0usize;
    for item in items {
        // upsert group
        conn.execute("INSERT OR IGNORE INTO project_groups (name) VALUES (?1)", rusqlite::params![item.group_name]).ok();
        let gid: i64 = conn.query_row("SELECT id FROM project_groups WHERE name=?1", rusqlite::params![item.group_name], |r| r.get(0))?;
        // upsert project
        let existing: Option<i64> = conn.query_row("SELECT id FROM projects WHERE name=?1 AND group_id=?2", rusqlite::params![item.name, gid], |r| r.get(0)).ok();
        if let Some(pid) = existing {
            conn.execute("UPDATE projects SET coefficient=?1, method_type=?2 WHERE id=?3", rusqlite::params![item.coefficient, item.method_type, pid])?;
        } else {
            conn.execute("INSERT INTO projects (group_id, name, coefficient, method_type) VALUES (?1,?2,?3,?4)", rusqlite::params![gid, item.name, item.coefficient, item.method_type])?;
        }
        count += 1;
    }
    Ok(count)
}

pub fn update(pool: &DbPool, id: i64, body: &ProjectUpdate) -> Result<ProjectResponse> {
    let conn = pool.get()?;
    if let Some(ref n) = body.name { conn.execute("UPDATE projects SET name=?1 WHERE id=?2", (n, id))?; }
    if let Some(ref f) = body.full_name { conn.execute("UPDATE projects SET full_name=?1 WHERE id=?2", (f, id))?; }
    if let Some(ref n) = body.notes { conn.execute("UPDATE projects SET notes=?1 WHERE id=?2", (n, id))?; }
    if let Some(s) = body.sort_order { conn.execute("UPDATE projects SET sort_order=?1 WHERE id=?2", (s, id))?; }
    if let Some(a) = body.is_active { conn.execute("UPDATE projects SET is_active=?1 WHERE id=?2", (a as i64, id))?; }
    if let Some(c) = body.coefficient { conn.execute("UPDATE projects SET coefficient=?1 WHERE id=?2", (c, id))?; }
    if let Some(ref mt) = body.method_type { conn.execute("UPDATE projects SET method_type=?1 WHERE id=?2", (mt, id))?; }
    get_by_id(pool, id)
}

pub fn batch_coefficient(pool: &DbPool, group_id: i64, coefficient: f64) -> Result<i64> {
    let conn = pool.get()?;
    let count = conn.execute("UPDATE projects SET coefficient=?1 WHERE group_id=?2", rusqlite::params![coefficient, group_id])?;
    Ok(count as i64)
}

pub fn delete(pool: &DbPool, id: i64) -> Result<()> {
    let conn = pool.get()?;
    let wr_count: i64 = conn.query_row("SELECT COUNT(*) FROM work_records WHERE project_id=?1 AND deleted_at IS NULL", [id], |r| r.get(0))?;
    if wr_count > 0 { return Err(AppError::Validation(format!("该项目有{}条工作记录，无法删除", wr_count))); }
    conn.execute("DELETE FROM projects WHERE id=?1", [id])?;
    Ok(())
}

// ── 方法类型 CRUD ──

pub fn list_method_types(pool: &DbPool) -> Result<Vec<MethodType>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare("SELECT id, name, sort_order FROM method_types ORDER BY sort_order")?;
    let rows = stmt.query_map([], |row| Ok(MethodType { id: row.get(0)?, name: row.get(1)?, sort_order: row.get(2)? }))?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

pub fn create_method_type(pool: &DbPool, body: &MethodTypeCreate) -> Result<MethodType> {
    let conn = pool.get()?;
    let so = body.sort_order.unwrap_or(10);
    conn.execute("INSERT INTO method_types (name, sort_order) VALUES (?1,?2)", rusqlite::params![body.name, so])?;
    let id = conn.last_insert_rowid();
    Ok(MethodType { id, name: body.name.clone(), sort_order: so })
}

pub fn update_method_type(pool: &DbPool, id: i64, body: &MethodTypeUpdate) -> Result<MethodType> {
    let conn = pool.get()?;
    if let Some(ref n) = body.name { conn.execute("UPDATE method_types SET name=?1 WHERE id=?2", (n, id))?; }
    if let Some(s) = body.sort_order { conn.execute("UPDATE method_types SET sort_order=?1 WHERE id=?2", (s, id))?; }
    let mt = conn.query_row("SELECT id, name, sort_order FROM method_types WHERE id=?", [id], |r| Ok(MethodType { id: r.get(0)?, name: r.get(1)?, sort_order: r.get(2)? }))?;
    Ok(mt)
}

pub fn delete_method_type(pool: &DbPool, id: i64) -> Result<()> {
    let conn = pool.get()?;
    let mt_name: String = conn.query_row("SELECT name FROM method_types WHERE id=?", [id], |r| r.get(0))?;
    let used: i64 = conn.query_row("SELECT COUNT(*) FROM projects WHERE method_type=?", [&mt_name], |r| r.get(0))?;
    if used > 0 { return Err(AppError::Validation(format!("类型'{}'被{}个项目使用，无法删除", mt_name, used))); }
    conn.execute("DELETE FROM method_types WHERE id=?", [id])?;
    Ok(())
}
