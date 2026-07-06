use axum::{extract::{Query, State}, Json, Router, routing::get};
use serde::{Deserialize, Serialize};
use crate::db::DbPool;
use crate::error::Result;
use crate::models::ApiResponse;

#[derive(Deserialize)]
pub struct StatsQuery {
    pub start: Option<String>,
    pub end: Option<String>,
    pub group_by: Option<String>,  // day | week | month
    pub group_id: Option<i64>,
}

#[derive(Serialize)]
pub struct StatsSummary {
    pub total_quantity: i64,
    pub total_records: i64,
    pub user_count: i64,
    pub project_count: i64,
    pub coefficient_score: f64,
    #[serde(rename = "details")]
    pub breakdown: Vec<PeriodBreakdown>,
}

#[derive(Serialize)]
pub struct PeriodBreakdown {
    pub period: String,
    pub total_quantity: i64,
    pub record_count: i64,
    pub coefficient_score: f64,
}

#[derive(Serialize)]
pub struct UserStats {
    pub user_name: String,
    pub total_quantity: i64,
    pub record_count: i64,
    pub coefficient_score: f64,
}

#[derive(Serialize)]
pub struct ProjectStats {
    pub project_id: i64,
    pub project_name: String,
    pub group_name: String,
    pub total_quantity: i64,
    pub record_count: i64,
    pub coefficient_score: f64,
}

#[derive(Serialize)]
pub struct TypeStats {
    pub instrument_type: String,
    pub total_quantity: i64,
    pub record_count: i64,
    pub coefficient_score: f64,
}

#[derive(Serialize)]
pub struct InstrumentStats {
    pub instrument: String,
    pub instrument_type: String,
    pub total_quantity: i64,
    pub record_count: i64,
    pub user_count: i64,
    pub coefficient_score: f64,
}

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/stats/summary", get(summary))
        .route("/api/stats/by-user", get(by_user))
        .route("/api/stats/by-project", get(by_project))
        .route("/api/stats/by-type", get(by_type))
        .route("/api/stats/by-instrument", get(by_instrument))
        .with_state(pool)
}

fn build_where(start: Option<&str>, end: Option<&str>, group_id: Option<i64>) -> (String, Vec<String>) {
    let mut clauses = vec!["wr.deleted_at IS NULL".to_string()];
    let mut params = vec![];
    if let Some(s) = start { let i = params.len()+1; clauses.push(format!("wr.recorded_at>=?{}", i)); params.push(s.to_string()); }
    if let Some(e) = end { let i = params.len()+1; clauses.push(format!("wr.recorded_at<=?{}", i)); params.push(format!("{}T23:59:59", e)); }
    if let Some(gid) = group_id { let i = params.len()+1; clauses.push(format!("pg.id=?{}", i)); params.push(gid.to_string()); }
    (clauses.join(" AND "), params)
}

fn coeff_sql() -> &'static str {
    "COALESCE(SUM(wr.quantity * p.coefficient), 0.0)"
}

async fn summary(State(pool): State<DbPool>, Query(q): Query<StatsQuery>) -> Result<Json<ApiResponse<StatsSummary>>> {
    let (wc, params) = build_where(q.start.as_deref(), q.end.as_deref(), q.group_id);
    let conn = pool.get()?;
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect();

    let (tq, tr, uc, pc, cs): (i64, i64, i64, i64, f64) = conn.query_row(
        &format!("SELECT COALESCE(SUM(wr.quantity),0), COUNT(*), COUNT(DISTINCT wr.user_name), COUNT(DISTINCT wr.project_id), {} FROM work_records wr JOIN projects p ON wr.project_id=p.id JOIN project_groups pg ON p.group_id=pg.id WHERE {}", coeff_sql(), wc),
        rusqlite::params_from_iter(param_refs.iter()), |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
    )?;

    let gb = q.group_by.as_deref().unwrap_or("day");
    let period_expr = match gb {
        "week" => "strftime('%Y-W%W', wr.recorded_at)",
        "month" => "strftime('%Y-%m', wr.recorded_at)",
        _ => "date(wr.recorded_at)",
    };
    let breakdown_sql = format!(
        "SELECT {} AS period, SUM(wr.quantity), COUNT(*), {} FROM work_records wr JOIN projects p ON wr.project_id=p.id JOIN project_groups pg ON p.group_id=pg.id WHERE {} GROUP BY period ORDER BY period",
        period_expr, coeff_sql(), wc
    );
    let mut stmt = conn.prepare(&breakdown_sql)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(param_refs.iter()), |row| {
        Ok(PeriodBreakdown {
            period: row.get(0)?,
            total_quantity: row.get(1)?,
            record_count: row.get(2)?,
            coefficient_score: row.get::<_, f64>(3).unwrap_or(0.0),
        })
    })?;
    let breakdown: Vec<PeriodBreakdown> = rows.collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(Json(ApiResponse::ok(StatsSummary { total_quantity: tq, total_records: tr, user_count: uc, project_count: pc, coefficient_score: cs, breakdown })))
}

async fn by_user(State(pool): State<DbPool>, Query(q): Query<StatsQuery>) -> Result<Json<ApiResponse<Vec<UserStats>>>> {
    let (wc, params) = build_where(q.start.as_deref(), q.end.as_deref(), q.group_id);
    let conn = pool.get()?;
    let refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect();
    let mut stmt = conn.prepare(&format!(
        "SELECT wr.user_name, SUM(wr.quantity), COUNT(*), {}
         FROM work_records wr JOIN projects p ON wr.project_id=p.id JOIN project_groups pg ON p.group_id=pg.id
         WHERE {} GROUP BY wr.user_name ORDER BY SUM(wr.quantity) DESC", coeff_sql(), wc
    ))?;
    let rows = stmt.query_map(rusqlite::params_from_iter(refs.iter()), |row| Ok(UserStats {
        user_name: row.get(0)?, total_quantity: row.get(1)?, record_count: row.get(2)?,
        coefficient_score: row.get::<_, f64>(3).unwrap_or(0.0),
    }))?;
    Ok(Json(ApiResponse::ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)))
}

async fn by_project(State(pool): State<DbPool>, Query(q): Query<StatsQuery>) -> Result<Json<ApiResponse<Vec<ProjectStats>>>> {
    let (wc, params) = build_where(q.start.as_deref(), q.end.as_deref(), q.group_id);
    let conn = pool.get()?;
    let refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect();
    let mut stmt = conn.prepare(&format!(
        "SELECT p.id, p.name, pg.name, SUM(wr.quantity), COUNT(*), {}
         FROM work_records wr JOIN projects p ON wr.project_id=p.id JOIN project_groups pg ON p.group_id=pg.id
         WHERE {} GROUP BY p.id ORDER BY pg.sort_order, p.sort_order", coeff_sql(), wc
    ))?;
    let rows = stmt.query_map(rusqlite::params_from_iter(refs.iter()), |row| Ok(ProjectStats {
        project_id: row.get(0)?, project_name: row.get(1)?, group_name: row.get(2)?,
        total_quantity: row.get(3)?, record_count: row.get(4)?,
        coefficient_score: row.get::<_, f64>(5).unwrap_or(0.0),
    }))?;
    Ok(Json(ApiResponse::ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)))
}

async fn by_type(State(pool): State<DbPool>, Query(q): Query<StatsQuery>) -> Result<Json<ApiResponse<Vec<TypeStats>>>> {
    let (wc, params) = build_where(q.start.as_deref(), q.end.as_deref(), None);
    let conn = pool.get()?;
    let refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect();
    let sql = format!(
        "SELECT CASE WHEN p.name LIKE '%GC-%' THEN '气相' WHEN p.name LIKE '%LC-%' THEN '液相' ELSE '其他' END AS itype,
                SUM(wr.quantity), COUNT(*), {}
         FROM work_records wr JOIN projects p ON wr.project_id=p.id JOIN project_groups pg ON p.group_id=pg.id
         WHERE {} GROUP BY itype", coeff_sql(), wc
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(refs.iter()), |row| Ok(TypeStats {
        instrument_type: row.get(0)?, total_quantity: row.get(1)?, record_count: row.get(2)?,
        coefficient_score: row.get::<_, f64>(3).unwrap_or(0.0),
    }))?;
    Ok(Json(ApiResponse::ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)))
}

async fn by_instrument(State(pool): State<DbPool>, Query(q): Query<StatsQuery>) -> Result<Json<ApiResponse<Vec<InstrumentStats>>>> {
    let (wc, params) = build_where(q.start.as_deref(), q.end.as_deref(), None);
    let conn = pool.get()?;
    let refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect();
    let mut stmt = conn.prepare(&format!(
        "SELECT SUBSTR(p.name, INSTR(p.name,'-')+1) AS instrument,
                CASE WHEN p.name LIKE '%GC-%' THEN '气相' WHEN p.name LIKE '%LC-%' THEN '液相' ELSE '其他' END AS instrument_type,
                SUM(wr.quantity), COUNT(*), COUNT(DISTINCT wr.user_name), {}
         FROM work_records wr JOIN projects p ON wr.project_id=p.id JOIN project_groups pg ON p.group_id=pg.id
         WHERE (p.name LIKE '%LC-%' OR p.name LIKE '%GC-%') AND {}
         GROUP BY instrument ORDER BY instrument", coeff_sql(), wc
    ))?;
    let rows = stmt.query_map(rusqlite::params_from_iter(refs.iter()), |row| Ok(InstrumentStats {
        instrument: row.get(0)?, instrument_type: row.get(1)?, total_quantity: row.get(2)?,
        record_count: row.get(3)?, user_count: row.get(4)?,
        coefficient_score: row.get::<_, f64>(5).unwrap_or(0.0),
    }))?;
    Ok(Json(ApiResponse::ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)))
}
