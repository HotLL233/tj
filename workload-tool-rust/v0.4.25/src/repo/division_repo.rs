use crate::db::DbPool;
use crate::error::Result;
use crate::models::division::{Division, DivisionCreate, DivisionResponse, DivisionUpdate};

/// 列出全部事业部，并聚合每个事业部下属实验室数量（lab_count）。
/// `project_groups.division_id` 为 NULL 的实验室不计入任何事业部。
pub fn list(pool: &DbPool) -> Result<Vec<DivisionResponse>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT d.id, d.name, d.sort_order, d.color, d.is_active,
                (SELECT COUNT(*) FROM project_groups g WHERE g.division_id = d.id) AS lab_count
         FROM divisions d
         ORDER BY d.sort_order"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(DivisionResponse {
            id: row.get(0)?,
            name: row.get(1)?,
            sort_order: row.get(2)?,
            color: row.get(3)?,
            is_active: row.get::<_, bool>(4).unwrap_or(true),
            lab_count: row.get(5)?,
        })
    })?;
    rows.collect::<std::result::Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn get_by_id(pool: &DbPool, id: i64) -> Result<Division> {
    let conn = pool.get()?;
    conn.query_row(
        "SELECT id, name, sort_order, color, is_active, created_at FROM divisions WHERE id = ?1",
        [id],
        |row| Ok(Division {
            id: row.get(0)?,
            name: row.get(1)?,
            sort_order: row.get(2)?,
            color: row.get(3)?,
            is_active: row.get::<_, bool>(4).unwrap_or(true),
            created_at: row.get(5)?,
        }),
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => crate::error::AppError::NotFound("事业部不存在".into()),
        _ => e.into(),
    })
}

/// 创建事业部（在已有连接上执行，便于上层包裹事务 + 审计）
pub fn create_on_conn(conn: &rusqlite::Connection, body: &DivisionCreate) -> Result<Division> {
    conn.execute(
        "INSERT INTO divisions (name, sort_order, color) VALUES (?1, ?2, ?3)",
        (&body.name, body.sort_order.unwrap_or(0), body.color.clone().unwrap_or_else(|| "#1976d2".to_string())),
    )?;
    let id = conn.last_insert_rowid();
    get_by_id_on_conn(conn, id)
}

pub fn create(pool: &DbPool, body: &DivisionCreate) -> Result<Division> {
    let conn = pool.get()?;
    create_on_conn(&conn, body)
}

pub fn update(pool: &DbPool, id: i64, body: &DivisionUpdate) -> Result<Division> {
    let conn = pool.get()?;
    update_on_conn(&conn, id, body)
}

/// 在已有连接上更新事业部（供事务内使用）
pub fn update_on_conn(conn: &rusqlite::Connection, id: i64, body: &DivisionUpdate) -> Result<Division> {
    if let Some(ref name) = body.name {
        conn.execute("UPDATE divisions SET name=?1 WHERE id=?2", (name, id))?;
    }
    if let Some(so) = body.sort_order {
        conn.execute("UPDATE divisions SET sort_order=?1 WHERE id=?2", (so, id))?;
    }
    if let Some(ref color) = body.color {
        conn.execute("UPDATE divisions SET color=?1 WHERE id=?2", (color, id))?;
    }
    get_by_id_on_conn(conn, id)
}

/// 删除事业部：先解除下属实验室的关联（置空 division_id），再删除事业部本身。
/// 下属实验室保留，仅解除与事业部的归属关系（级联置空，不阻止删除）。
pub fn delete(pool: &DbPool, id: i64) -> Result<()> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    // 1) 级联置空：解除该事业部下所有实验室的关联
    tx.execute("UPDATE project_groups SET division_id = NULL WHERE division_id = ?1", [id])?;
    // 2) 删除事业部
    tx.execute("DELETE FROM divisions WHERE id = ?1", [id])?;
    tx.commit()?;
    Ok(())
}

/// 在已有连接上查询事业部（供事务内使用）
fn get_by_id_on_conn(conn: &rusqlite::Connection, id: i64) -> Result<Division> {
    conn.query_row(
        "SELECT id, name, sort_order, color, is_active, created_at FROM divisions WHERE id = ?1",
        [id],
        |row| Ok(Division {
            id: row.get(0)?,
            name: row.get(1)?,
            sort_order: row.get(2)?,
            color: row.get(3)?,
            is_active: row.get::<_, bool>(4).unwrap_or(true),
            created_at: row.get(5)?,
        }),
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => crate::error::AppError::NotFound("事业部不存在".into()),
        _ => e.into(),
    })
}
