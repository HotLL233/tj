use crate::db::DbPool;
use crate::error::{AppError, Result};
use crate::models::article::{HelpArticle, HelpArticleUpdate};

const COLS: &str = "id, title, content_html, toc_json, source_file, is_visible, sort_order, created_at, updated_at";

fn row_to_article(row: &rusqlite::Row) -> std::result::Result<HelpArticle, rusqlite::Error> {
    Ok(HelpArticle {
        id: row.get(0)?,
        title: row.get(1)?,
        content_html: row.get(2)?,
        toc_json: row.get(3)?,
        source_file: row.get(4)?,
        is_visible: row.get::<_, bool>(5).unwrap_or(true),
        sort_order: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

pub fn list(pool: &DbPool, visible_only: bool) -> Result<Vec<HelpArticle>> {
    let conn = pool.get()?;
    let sql = if visible_only {
        &format!("SELECT {COLS} FROM help_articles WHERE is_visible = 1 ORDER BY sort_order, id")
    } else {
        &format!("SELECT {COLS} FROM help_articles ORDER BY sort_order, id")
    };
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], |row| row_to_article(row))?;
    rows.collect::<std::result::Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn get_by_id(pool: &DbPool, id: i64) -> Result<HelpArticle> {
    let conn = pool.get()?;
    conn.query_row(
        &format!("SELECT {COLS} FROM help_articles WHERE id = ?1"),
        [id],
        |row| row_to_article(row),
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound("文章不存在".into()),
        _ => e.into(),
    })
}

pub fn create(pool: &DbPool, title: &str, content_html: &str, toc_json: Option<&str>, source_file: Option<&str>) -> Result<HelpArticle> {
    let conn = pool.get()?;
    conn.execute(
        "INSERT INTO help_articles (title, content_html, toc_json, source_file) VALUES (?1, ?2, ?3, ?4)",
        (title, content_html, toc_json, source_file),
    )?;
    let id = conn.last_insert_rowid();
    get_by_id(pool, id)
}

pub fn update(pool: &DbPool, id: i64, body: &HelpArticleUpdate) -> Result<HelpArticle> {
    let conn = pool.get()?;
    if let Some(ref title) = body.title {
        conn.execute("UPDATE help_articles SET title=?1, updated_at=datetime('now','localtime') WHERE id=?2", (title, id))?;
    }
    if let Some(ref html) = body.content_html {
        conn.execute("UPDATE help_articles SET content_html=?1, updated_at=datetime('now','localtime') WHERE id=?2", (html, id))?;
    }
    if let Some(ref toc) = body.toc_json {
        conn.execute("UPDATE help_articles SET toc_json=?1, updated_at=datetime('now','localtime') WHERE id=?2", (toc, id))?;
    }
    if let Some(v) = body.is_visible {
        conn.execute("UPDATE help_articles SET is_visible=?1, updated_at=datetime('now','localtime') WHERE id=?2", (v, id))?;
    }
    if let Some(so) = body.sort_order {
        conn.execute("UPDATE help_articles SET sort_order=?1, updated_at=datetime('now','localtime') WHERE id=?2", (so, id))?;
    }
    get_by_id(pool, id)
}

pub fn delete(pool: &DbPool, id: i64) -> Result<HelpArticle> {
    let doc = get_by_id(pool, id)?;
    let conn = pool.get()?;
    conn.execute("DELETE FROM help_articles WHERE id=?1", [id])?;
    Ok(doc)
}

pub fn reorder(pool: &DbPool, ids: &[(i64, i64)]) -> Result<()> {
    let conn = pool.get()?;
    for (id, sort_order) in ids {
        conn.execute(
            "UPDATE help_articles SET sort_order=?1, updated_at=datetime('now','localtime') WHERE id=?2",
            (sort_order, id),
        ).ok();
    }
    Ok(())
}
