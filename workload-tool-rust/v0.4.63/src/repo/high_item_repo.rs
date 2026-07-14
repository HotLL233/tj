use crate::db::DbPool;
use crate::error::{AppError, Result};
use crate::models::high_item::{HighItem, HighItemCreate, HighItemUpdate};
use crate::repo::audit_repo;

pub fn list(pool: &DbPool) -> Result<Vec<HighItem>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT id, name, sort_order, is_active, created_at FROM high_items WHERE is_active=1 ORDER BY sort_order, id"
    )?;
    let rows = stmt.query_map([], |row| Ok(HighItem {
        id: row.get(0)?, name: row.get(1)?, sort_order: row.get(2)?,
        is_active: row.get(3)?, created_at: row.get(4)?,
    }))?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

pub fn list_all(pool: &DbPool) -> Result<Vec<HighItem>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT id, name, sort_order, is_active, created_at FROM high_items ORDER BY sort_order, id"
    )?;
    let rows = stmt.query_map([], |row| Ok(HighItem {
        id: row.get(0)?, name: row.get(1)?, sort_order: row.get(2)?,
        is_active: row.get(3)?, created_at: row.get(4)?,
    }))?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

pub fn create(pool: &DbPool, data: &HighItemCreate) -> Result<HighItem> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    let name = data.name.trim();
    if name.is_empty() { return Err(AppError::Validation("高项名称不能为空".into())); }
    tx.execute("INSERT INTO high_items (name, sort_order) VALUES (?1, ?2)",
        rusqlite::params![name, data.sort_order.unwrap_or(0)])?;
    let id = tx.last_insert_rowid();
    audit_repo::log_on_conn_with_module(&tx, "create", "high_items", Some(id), "system",
        &format!("创建高项#{}：{}", id, name), "shared")?;
    tx.commit()?;
    get_by_id(&pool, id)
}

pub fn update(pool: &DbPool, id: i64, data: &HighItemUpdate) -> Result<HighItem> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    let old = get_by_id_on_conn(&tx, id)?;
    if let Some(ref name) = data.name {
        let n = name.trim();
        if !n.is_empty() && n != old.name {
            tx.execute("UPDATE high_items SET name=?1 WHERE id=?2", rusqlite::params![n, id])?;
        }
    }
    if let Some(so) = data.sort_order {
        tx.execute("UPDATE high_items SET sort_order=?1 WHERE id=?2", rusqlite::params![so, id])?;
    }
    if let Some(ia) = data.is_active {
        tx.execute("UPDATE high_items SET is_active=?1 WHERE id=?2", rusqlite::params![ia, id])?;
    }
    audit_repo::log_on_conn_with_module(&tx, "update", "high_items", Some(id), "system",
        &format!("更新高项#{}", id), "shared")?;
    tx.commit()?;
    get_by_id(&pool, id)
}

pub fn soft_delete(pool: &DbPool, id: i64) -> Result<()> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    let old = get_by_id_on_conn(&tx, id)?;
    tx.execute("UPDATE high_items SET is_active=0 WHERE id=?1", [id])?;
    audit_repo::log_on_conn_with_module(&tx, "delete", "high_items", Some(id), "system",
        &format!("删除高项#{}：{}", id, old.name), "shared")?;
    tx.commit()?;
    Ok(())
}

pub fn permanent_delete(pool: &DbPool, id: i64) -> Result<()> {
    let conn = pool.get()?;
    conn.execute("DELETE FROM high_items WHERE id=?1", [id])?;
    Ok(())
}

fn get_by_id(pool: &DbPool, id: i64) -> Result<HighItem> {
    let conn = pool.get()?;
    get_by_id_on_conn(&conn, id)
}

fn get_by_id_on_conn(conn: &rusqlite::Connection, id: i64) -> Result<HighItem> {
    conn.query_row(
        "SELECT id, name, sort_order, is_active, created_at FROM high_items WHERE id=?1", [id],
        |row| Ok(HighItem {
            id: row.get(0)?, name: row.get(1)?, sort_order: row.get(2)?,
            is_active: row.get(3)?, created_at: row.get(4)?,
        })
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound("高项不存在".into()),
        _ => e.into(),
    })
}
