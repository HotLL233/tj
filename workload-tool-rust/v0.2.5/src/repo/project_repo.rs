use crate::db::DbPool;
use crate::error::Result;
use crate::models::project::{ProjectCreate, ProjectResponse, ProjectUpdate};

pub fn list(pool: &DbPool, group_id: Option<i64>, active_only: bool) -> Result<Vec<ProjectResponse>> {
    let conn = pool.get()?;
    let mut sql = String::from(
        "SELECT p.id, p.group_id, pg.name, p.name, p.full_name, p.notes,
                p.sort_order, p.is_active, p.coefficient, p.created_at
         FROM projects p JOIN project_groups pg ON p.group_id = pg.id WHERE 1=1"
    );
    if let Some(gid) = group_id { sql.push_str(&format!(" AND p.group_id={}", gid)); }
    if active_only { sql.push_str(" AND p.is_active=1"); }
    sql.push_str(" ORDER BY pg.sort_order, p.sort_order");

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| {
        Ok(ProjectResponse {
            id: row.get(0)?, group_id: row.get(1)?, group_name: row.get(2)?,
            name: row.get(3)?, full_name: row.get::<_, String>(4).unwrap_or_default(),
            notes: row.get::<_, String>(5).unwrap_or_default(), sort_order: row.get(6)?,
            is_active: row.get::<_, i32>(7).unwrap_or(1) != 0,
            coefficient: row.get::<_, f64>(8).unwrap_or(1.0),
            created_at: row.get(9)?,
        })
    })?;
    rows.collect::<std::result::Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn get_by_id(pool: &DbPool, id: i64) -> Result<ProjectResponse> {
    let conn = pool.get()?;
    conn.query_row(
        "SELECT p.id, p.group_id, pg.name, p.name, p.full_name, p.notes,
                p.sort_order, p.is_active, p.coefficient, p.created_at
         FROM projects p JOIN project_groups pg ON p.group_id = pg.id WHERE p.id=?1",
        [id],
        |row| Ok(ProjectResponse {
            id: row.get(0)?, group_id: row.get(1)?, group_name: row.get(2)?,
            name: row.get(3)?, full_name: row.get::<_, String>(4).unwrap_or_default(),
            notes: row.get::<_, String>(5).unwrap_or_default(), sort_order: row.get(6)?,
            is_active: row.get::<_, i32>(7).unwrap_or(1) != 0,
            coefficient: row.get::<_, f64>(8).unwrap_or(1.0),
            created_at: row.get(9)?,
        }),
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => crate::error::AppError::NotFound("项目不存在".into()),
        _ => e.into(),
    })
}

pub fn create(pool: &DbPool, body: &ProjectCreate) -> Result<ProjectResponse> {
    let conn = pool.get()?;
    let coeff = body.coefficient.unwrap_or(1.0);
    conn.execute(
        "INSERT INTO projects (group_id, name, sort_order, coefficient) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params!(body.group_id, &body.name, body.sort_order.unwrap_or(0), coeff),
    )?;
    get_by_id(pool, conn.last_insert_rowid())
}

pub fn update(pool: &DbPool, id: i64, body: &ProjectUpdate) -> Result<ProjectResponse> {
    let conn = pool.get()?;
    if let Some(ref name) = body.name {
        conn.execute("UPDATE projects SET name=?1 WHERE id=?2", (name, id))?;
    }
    if let Some(ref full) = body.full_name {
        conn.execute("UPDATE projects SET full_name=?1 WHERE id=?2", (full, id))?;
    }
    if let Some(ref notes) = body.notes {
        conn.execute("UPDATE projects SET notes=?1 WHERE id=?2", (notes, id))?;
    }
    if let Some(so) = body.sort_order {
        conn.execute("UPDATE projects SET sort_order=?1 WHERE id=?2", (so, id))?;
    }
    if let Some(active) = body.is_active {
        conn.execute("UPDATE projects SET is_active=?1 WHERE id=?2", (active as i32, id))?;
    }
    if let Some(coeff) = body.coefficient {
        conn.execute("UPDATE projects SET coefficient=?1 WHERE id=?2", (coeff, id))?;
    }
    get_by_id(pool, id)
}

/// 批量更新指定分组下所有项目的管理系数
pub fn batch_coefficient(pool: &DbPool, group_id: i64, coefficient: f64) -> Result<i64> {
    let conn = pool.get()?;
    let count = conn.execute(
        "UPDATE projects SET coefficient=?1 WHERE group_id=?2",
        rusqlite::params![coefficient, group_id],
    )?;
    Ok(count as i64)
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
