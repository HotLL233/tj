use crate::db::DbPool;
use crate::error::Result;
use serde::Serialize;

/// Daily summary used by statistics
#[derive(Debug, Serialize)]
pub struct DailySummary {
    pub date: String,
    pub count: i64,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct UserSummary {
    pub user_name: String,
    pub count: i64,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct ProjectSummary {
    pub project_name: String,
    pub group_name: String,
    pub count: i64,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct TypeSummary {
    pub instrument_type: String,
    pub count: i64,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct InstrumentSummary {
    pub project_id: i64,
    pub project_name: String,
    pub group_name: String,
    pub count: i64,
    pub total: i64,
}

/// Aggregate statistics by day within a date range
pub fn daily_summary(pool: &DbPool, start: &str, end: &str) -> Result<Vec<DailySummary>> {
    let conn = pool.get()?;
    let end_closed = format!("{}T23:59:59", end);
    let mut stmt = conn.prepare(
        "SELECT date(wr.recorded_at) AS d, COUNT(*), SUM(wr.quantity)
         FROM work_records wr WHERE wr.deleted_at IS NULL AND wr.recorded_at>=?1 AND wr.recorded_at<=?2
         GROUP BY date(wr.recorded_at) ORDER BY d"
    )?;
    let rows = stmt.query_map(rusqlite::params![start, end_closed], |row| {
        Ok(DailySummary {
            date: row.get(0)?, count: row.get(1)?, total: row.get(2)?,
        })
    })?.collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Aggregate statistics by user within a date range
pub fn by_user(pool: &DbPool, start: &str, end: &str) -> Result<Vec<UserSummary>> {
    let conn = pool.get()?;
    let end_closed = format!("{}T23:59:59", end);
    let mut stmt = conn.prepare(
        "SELECT wr.user_name, COUNT(*), SUM(wr.quantity)
         FROM work_records wr WHERE wr.deleted_at IS NULL AND wr.recorded_at>=?1 AND wr.recorded_at<=?2
         GROUP BY wr.user_name ORDER BY wr.user_name"
    )?;
    let rows = stmt.query_map(rusqlite::params![start, end_closed], |row| {
        Ok(UserSummary { user_name: row.get(0)?, count: row.get(1)?, total: row.get(2)? })
    })?.collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Aggregate statistics by project within a date range
pub fn by_project(pool: &DbPool, start: &str, end: &str) -> Result<Vec<ProjectSummary>> {
    let conn = pool.get()?;
    let end_closed = format!("{}T23:59:59", end);
    let mut stmt = conn.prepare(
        "SELECT p.name, COALESCE(pg.name, '未分组'), COUNT(*), SUM(wr.quantity)
         FROM work_records wr JOIN projects p ON wr.project_id=p.id LEFT JOIN project_groups pg ON pg.id=wr.group_id
         WHERE wr.deleted_at IS NULL AND wr.recorded_at>=?1 AND wr.recorded_at<=?2
         GROUP BY p.id ORDER BY pg.sort_order, p.sort_order"
    )?;
    let rows = stmt.query_map(rusqlite::params![start, end_closed], |row| {
        Ok(ProjectSummary { project_name: row.get(0)?, group_name: row.get(1)?, count: row.get(2)?, total: row.get(3)? })
    })?.collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Aggregate by instrument type (液相/气相)
pub fn by_type(pool: &DbPool, start: &str, end: &str) -> Result<Vec<TypeSummary>> {
    let conn = pool.get()?;
    let end_closed = format!("{}T23:59:59", end);
    // Parse instrument type from project name: "LC" → "液相", "GC" → "气相"
    let mut stmt = conn.prepare(
        "SELECT CASE WHEN p.name LIKE '%-LC-%' THEN '液相' WHEN p.name LIKE '%-GC-%' THEN '气相' ELSE '其他' END AS itype,
         COUNT(*), SUM(wr.quantity)
         FROM work_records wr JOIN projects p ON wr.project_id=p.id
         WHERE wr.deleted_at IS NULL AND wr.recorded_at>=?1 AND wr.recorded_at<=?2
         GROUP BY itype ORDER BY itype"
    )?;
    let rows = stmt.query_map(rusqlite::params![start, end_closed], |row| {
        Ok(TypeSummary { instrument_type: row.get(0)?, count: row.get(1)?, total: row.get(2)? })
    })?.collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Aggregate by individual instrument (project-level)
pub fn by_instrument(pool: &DbPool, start: &str, end: &str) -> Result<Vec<InstrumentSummary>> {
    let conn = pool.get()?;
    let end_closed = format!("{}T23:59:59", end);
    let mut stmt = conn.prepare(
        "SELECT p.id, p.name, COALESCE(pg.name, '未分组'), COUNT(*), SUM(wr.quantity)
         FROM work_records wr JOIN projects p ON wr.project_id=p.id LEFT JOIN project_groups pg ON pg.id=wr.group_id
         WHERE wr.deleted_at IS NULL AND wr.recorded_at>=?1 AND wr.recorded_at<=?2
         GROUP BY p.id ORDER BY pg.sort_order, p.sort_order"
    )?;
    let rows = stmt.query_map(rusqlite::params![start, end_closed], |row| {
        Ok(InstrumentSummary { project_id: row.get(0)?, project_name: row.get(1)?, group_name: row.get(2)?, count: row.get(3)?, total: row.get(4)? })
    })?.collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}
