use crate::db::DbPool;
use crate::error::Result;
use crate::models::sample::{SampleRecordResponse, SampleRecordCreate, SampleRecordUpdate};

pub fn create(pool: &DbPool, body: &SampleRecordCreate) -> Result<SampleRecordResponse> {
    let conn = pool.get()?;
    let sql = "INSERT INTO sample_records (project_id, group_id, user_name, sample_name, sample_count, unit, batch_no, notes, submitted_at) VALUES (?1, (SELECT group_id FROM projects WHERE id=?1), ?2, ?3, ?4, ?5, ?6, ?7, ?8) RETURNING id";
    let id: i64 = conn.query_row(sql,
        rusqlite::params![body.project_id, body.user_name, body.sample_name, body.sample_count,
            body.unit.as_deref().unwrap_or("个"), body.batch_no.as_deref().unwrap_or(""), body.notes.as_deref().unwrap_or(""), body.submitted_at],
        |row| row.get(0),
    )?;
    get_by_id(pool, id)
}

pub fn get_by_id(pool: &DbPool, id: i64) -> Result<SampleRecordResponse> {
    let conn = pool.get()?;
    conn.query_row(
        "SELECT sr.id, sr.project_id, p.name, sr.group_id, pg.name, sr.user_name, sr.sample_name, sr.sample_count, sr.unit, sr.batch_no, sr.notes, sr.submitted_at, sr.created_at, sr.deleted_at FROM sample_records sr JOIN projects p ON sr.project_id=p.id JOIN project_groups pg ON sr.group_id=pg.id WHERE sr.id=?1",
        [id],
        |row| Ok(SampleRecordResponse {
            id: row.get(0)?, project_id: row.get(1)?, project_name: row.get(2)?,
            group_id: row.get(3)?, group_name: row.get(4)?, user_name: row.get(5)?,
            sample_name: row.get(6)?, sample_count: row.get(7)?, unit: row.get(8)?,
            batch_no: row.get(9)?, notes: row.get(10)?, submitted_at: row.get(11)?,
            created_at: row.get(12)?, deleted_at: row.get(13)?,
        }),
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => crate::error::AppError::NotFound("送样记录不存在".into()),
        _ => e.into(),
    })
}

pub fn list(
    pool: &DbPool, group_id: Option<i64>, user_name: Option<&str>,
    start: Option<&str>, end: Option<&str>, page: i64, page_size: i64,
) -> Result<(Vec<SampleRecordResponse>, i64)> {
    let conn = pool.get()?;
    let mut where_clauses = vec!["sr.deleted_at IS NULL".to_string()];
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];

    if let Some(gid) = group_id {
        where_clauses.push(format!("sr.group_id=?{}", params.len() + 1));
        params.push(Box::new(gid));
    }
    if let Some(un) = user_name {
        where_clauses.push(format!("sr.user_name=?{}", params.len() + 1));
        params.push(Box::new(un.to_string()));
    }
    if let Some(s) = start {
        where_clauses.push(format!("sr.submitted_at>=?{}", params.len() + 1));
        params.push(Box::new(s.to_string()));
    }
    if let Some(e) = end {
        where_clauses.push(format!("sr.submitted_at<=?{}", params.len() + 1));
        params.push(Box::new(format!("{} 23:59:59", e)));
    }

    let where_clause = where_clauses.join(" AND ");
    let count_sql = format!("SELECT COUNT(*) FROM sample_records sr JOIN projects p ON sr.project_id=p.id JOIN project_groups pg ON sr.group_id=pg.id WHERE {}", where_clause);
    let total: i64 = conn.query_row(&count_sql, rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())), |row| row.get(0))?;

    let offset = (page - 1) * page_size;
    let data_sql = format!(
        "SELECT sr.id, sr.project_id, p.name, sr.group_id, pg.name, sr.user_name, sr.sample_name, sr.sample_count, sr.unit, sr.batch_no, sr.notes, sr.submitted_at, sr.created_at, sr.deleted_at FROM sample_records sr JOIN projects p ON sr.project_id=p.id JOIN project_groups pg ON sr.group_id=pg.id WHERE {} ORDER BY sr.submitted_at DESC LIMIT {} OFFSET {}",
        where_clause, page_size, offset
    );

    let mut stmt = conn.prepare(&data_sql)?;
    let items = stmt.query_map(
        rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
        |row| Ok(SampleRecordResponse {
            id: row.get(0)?, project_id: row.get(1)?, project_name: row.get(2)?,
            group_id: row.get(3)?, group_name: row.get(4)?, user_name: row.get(5)?,
            sample_name: row.get(6)?, sample_count: row.get(7)?, unit: row.get(8)?,
            batch_no: row.get(9)?, notes: row.get(10)?, submitted_at: row.get(11)?,
            created_at: row.get(12)?, deleted_at: row.get(13)?,
        }),
    )?.collect::<std::result::Result<Vec<_>, _>>()?;

    Ok((items, total))
}

pub fn update(pool: &DbPool, id: i64, body: &SampleRecordUpdate) -> Result<SampleRecordResponse> {
    let conn = pool.get()?;
    let existing = get_by_id(pool, id)?;
    if existing.deleted_at.is_some() {
        return Err(crate::error::AppError::Validation("记录已被删除，无法编辑".into()));
    }
    let mut sets = vec![];
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
    if let Some(ref v) = body.sample_name { sets.push(format!("sample_name=?{}", params.len() + 1)); params.push(Box::new(v.clone())); }
    if let Some(v) = body.sample_count { sets.push(format!("sample_count=?{}", params.len() + 1)); params.push(Box::new(v)); }
    if let Some(ref v) = body.unit { sets.push(format!("unit=?{}", params.len() + 1)); params.push(Box::new(v.clone())); }
    if let Some(ref v) = body.batch_no { sets.push(format!("batch_no=?{}", params.len() + 1)); params.push(Box::new(v.clone())); }
    if let Some(ref v) = body.notes { sets.push(format!("notes=?{}", params.len() + 1)); params.push(Box::new(v.clone())); }
    if let Some(ref v) = body.submitted_at { sets.push(format!("submitted_at=?{}", params.len() + 1)); params.push(Box::new(v.clone())); }
    if sets.is_empty() { return Err(crate::error::AppError::Validation("没有需要更新的字段".into())); }
    let sql = format!("UPDATE sample_records SET {} WHERE id={}", sets.join(", "), id);
    conn.execute(&sql, rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())))?;
    get_by_id(pool, id)
}

pub fn soft_delete(pool: &DbPool, id: i64) -> Result<()> {
    let conn = pool.get()?;
    let deleted: Option<String> = conn.query_row("SELECT deleted_at FROM sample_records WHERE id=?1", [id], |r| r.get(0))
        .map_err(|e| match e { rusqlite::Error::QueryReturnedNoRows => crate::error::AppError::NotFound("记录不存在".into()), _ => e.into() })?;
    if deleted.is_some() { return Err(crate::error::AppError::Validation("记录已被删除".into())); }
    conn.execute("UPDATE sample_records SET deleted_at=datetime('now','localtime') WHERE id=?1", [id])?;
    Ok(())
}

pub fn restore(pool: &DbPool, id: i64) -> Result<SampleRecordResponse> {
    let conn = pool.get()?;
    conn.execute("UPDATE sample_records SET deleted_at=NULL WHERE id=?1", [id])?;
    get_by_id(pool, id)
}
