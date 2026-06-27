use crate::db::DbPool;
use crate::error::Result;
use crate::models::project::{ProjectCreate, ProjectResponse, ProjectUpdate};

pub fn list(pool: &DbPool, group_id: Option<i64>, active_only: bool) -> Result<Vec<ProjectResponse>> {
    let conn = pool.get()?;
    let mut sql = String::from(
        "SELECT p.id, p.group_id, pg.name, p.name, p.full_name, p.notes,
                p.sort_order, p.is_active, p.created_at
         FROM projects p JOIN project_groups pg ON p.group_id = pg.id WHERE 1=1"
    );
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
    if let Some(gid) = group_id { let idx = params.len() + 1; sql.push_str(&format!(" AND p.group_id=?{}", idx)); params.push(Box::new(gid)); }
    if active_only { sql.push_str(" AND p.is_active=1"); }
    sql.push_str(" ORDER BY pg.sort_order, p.sort_order");

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(
        rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
        |row| Ok(ProjectResponse {
            id: row.get(0)?, group_id: row.get(1)?, group_name: row.get(2)?,
            name: row.get(3)?, full_name: row.get::<_, String>(4).unwrap_or_default(),
            notes: row.get::<_, String>(5).unwrap_or_default(), sort_order: row.get(6)?,
            is_active: row.get::<_, i32>(7).unwrap_or(1) != 0, created_at: row.get(8)?,
        })
    )?;
    rows.collect::<std::result::Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn get_by_id(pool: &DbPool, id: i64) -> Result<ProjectResponse> {
    let conn = pool.get()?;
    conn.query_row(
        "SELECT p.id, p.group_id, pg.name, p.name, p.full_name, p.notes,
                p.sort_order, p.is_active, p.created_at
         FROM projects p JOIN project_groups pg ON p.group_id = pg.id WHERE p.id=?1",
        [id],
        |row| Ok(ProjectResponse {
            id: row.get(0)?, group_id: row.get(1)?, group_name: row.get(2)?,
            name: row.get(3)?, full_name: row.get::<_, String>(4).unwrap_or_default(),
            notes: row.get::<_, String>(5).unwrap_or_default(), sort_order: row.get(6)?,
            is_active: row.get::<_, i32>(7).unwrap_or(1) != 0, created_at: row.get(8)?,
        }),
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => crate::error::AppError::NotFound("项目不存在".into()),
        _ => e.into(),
    })
}

pub fn create(pool: &DbPool, body: &ProjectCreate) -> Result<ProjectResponse> {
    let conn = pool.get()?;
    conn.execute(
        "INSERT INTO projects (group_id, name, sort_order) VALUES (?1, ?2, ?3)",
        rusqlite::params!(body.group_id, &body.name, body.sort_order.unwrap_or(0)),
    )?;
    get_by_id(pool, conn.last_insert_rowid())
}

pub fn update(pool: &DbPool, id: i64, body: &ProjectUpdate) -> Result<ProjectResponse> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;

    // Build dynamic UPDATE with only changed fields in a single statement
    let mut sets: Vec<String> = vec![];
    let mut has_changes = false;
    let mut col_idx = 1;
    if body.name.is_some() { sets.push(format!("name=?{}", col_idx)); col_idx += 1; has_changes = true; }
    if body.full_name.is_some() { sets.push(format!("full_name=?{}", col_idx)); col_idx += 1; has_changes = true; }
    if body.notes.is_some() { sets.push(format!("notes=?{}", col_idx)); col_idx += 1; has_changes = true; }
    if body.sort_order.is_some() { sets.push(format!("sort_order=?{}", col_idx)); col_idx += 1; has_changes = true; }
    if body.is_active.is_some() { sets.push(format!("is_active=?{}", col_idx)); col_idx += 1; has_changes = true; }

    if !has_changes {
        return Err(crate::error::AppError::Validation("没有需要更新的字段".into()));
    }

    let set_clause = sets.join(", ");
    let params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![
        body.name.as_ref().map(|v| Box::new(v.clone()) as Box<dyn rusqlite::types::ToSql>),
        body.full_name.as_ref().map(|v| Box::new(v.clone()) as Box<dyn rusqlite::types::ToSql>),
        body.notes.as_ref().map(|v| Box::new(v.clone()) as Box<dyn rusqlite::types::ToSql>),
        body.sort_order.as_ref().map(|v| Box::new(*v) as Box<dyn rusqlite::types::ToSql>),
        body.is_active.as_ref().map(|v| Box::new(*v as i32) as Box<dyn rusqlite::types::ToSql>),
    ].into_iter().flatten().collect();

    tx.execute(&format!("UPDATE projects SET {} WHERE id=?{}", set_clause, col_idx),
        rusqlite::params_from_iter(params.iter().map(|p| p.as_ref()))
    )?;

    tx.commit()?;
    get_by_id(pool, id)
}

pub fn delete(pool: &DbPool, id: i64) -> Result<()> {
    let count: i64 = pool.get()?.query_row(
        "SELECT COUNT(*) FROM work_records WHERE project_id=?1 AND deleted_at IS NULL", [id], |r| r.get(0)
    )?;
    if count > 0 {
        return Err(crate::error::AppError::Conflict("该项目下还有工作记录，无法删除".into()));
    }
    pool.get()?.execute("DELETE FROM projects WHERE id=?1", [id])?;
    Ok(())
}
