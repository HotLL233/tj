use crate::db::DbPool;
use crate::error::{AppError, Result};
use crate::models::help::{HelpDocUpdateRequest, HelpDocument};

const COLS: &str = "id, title, filename, file_path, file_type, file_size, is_visible, sort_order, page_count, created_at, updated_at";

fn row_to_doc(row: &rusqlite::Row) -> std::result::Result<HelpDocument, rusqlite::Error> {
    Ok(HelpDocument {
        id: row.get(0)?,
        title: row.get(1)?,
        filename: row.get(2)?,
        file_path: row.get(3)?,
        file_type: row.get(4)?,
        file_size: row.get(5)?,
        is_visible: row.get::<_, bool>(6).unwrap_or(true),
        sort_order: row.get(7)?,
        page_count: row.get::<_, i64>(8).unwrap_or(0),
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

pub fn list(pool: &DbPool, visible_only: bool) -> Result<Vec<HelpDocument>> {
    let conn = pool.get()?;
    let sql = if visible_only {
        &format!("SELECT {COLS} FROM help_documents WHERE is_visible = 1 ORDER BY sort_order, id")
    } else {
        &format!("SELECT {COLS} FROM help_documents ORDER BY sort_order, id")
    };
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], |row| row_to_doc(row))?;
    rows.collect::<std::result::Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn get_by_id(pool: &DbPool, id: i64) -> Result<HelpDocument> {
    let conn = pool.get()?;
    conn.query_row(
        &format!("SELECT {COLS} FROM help_documents WHERE id = ?1"),
        [id],
        |row| row_to_doc(row),
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound("帮助文档不存在".into()),
        _ => e.into(),
    })
}

pub fn create(
    pool: &DbPool,
    title: &str,
    filename: &str,
    file_path: &str,
    file_type: &str,
    file_size: i64,
) -> Result<HelpDocument> {
    let conn = pool.get()?;
    conn.execute(
        "INSERT INTO help_documents (title, filename, file_path, file_type, file_size)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        (title, filename, file_path, file_type, file_size),
    )?;
    let id = conn.last_insert_rowid();
    get_by_id(pool, id)
}

pub fn set_page_count(pool: &DbPool, id: i64, page_count: i64) -> Result<()> {
    let conn = pool.get()?;
    conn.execute(
        "UPDATE help_documents SET page_count=?1, updated_at=datetime('now','localtime') WHERE id=?2",
        (page_count, id),
    )?;
    Ok(())
}

pub fn update(pool: &DbPool, id: i64, body: &HelpDocUpdateRequest) -> Result<HelpDocument> {
    let conn = pool.get()?;
    if let Some(ref title) = body.title {
        conn.execute(
            "UPDATE help_documents SET title=?1, updated_at=datetime('now','localtime') WHERE id=?2",
            (title, id),
        )?;
    }
    if let Some(v) = body.is_visible {
        conn.execute(
            "UPDATE help_documents SET is_visible=?1, updated_at=datetime('now','localtime') WHERE id=?2",
            (v, id),
        )?;
    }
    if let Some(so) = body.sort_order {
        conn.execute(
            "UPDATE help_documents SET sort_order=?1, updated_at=datetime('now','localtime') WHERE id=?2",
            (so, id),
        )?;
    }
    get_by_id(pool, id)
}

/// v0.4.74: 批量更新排序
pub fn reorder_documents(pool: &DbPool, ids: &[(i64, i64)]) -> Result<()> {
    let conn = pool.get()?;
    for (id, sort_order) in ids {
        conn.execute(
            "UPDATE help_documents SET sort_order=?1, updated_at=datetime('now','localtime') WHERE id=?2",
            (sort_order, id),
        ).ok();
    }
    Ok(())
}

pub fn delete(pool: &DbPool, id: i64) -> Result<HelpDocument> {
    let doc = get_by_id(pool, id)?;
    let conn = pool.get()?;
    conn.execute("DELETE FROM help_documents WHERE id=?1", [id])?;
    Ok(doc)
}
