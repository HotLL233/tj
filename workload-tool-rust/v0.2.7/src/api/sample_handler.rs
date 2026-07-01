use axum::{extract::{Path, Query, State}, Json, Router, routing::get};
use serde::{Deserialize, Serialize};
use crate::db::DbPool;
use crate::error::Result;
use crate::models::ApiResponse;
use crate::models::PaginatedResponse;
use crate::models::sample::{SampleRecordResponse, SampleRecordCreate, SampleRecordUpdate};
use crate::repo::sample_repo;
use crate::service::sample_service;

#[derive(Deserialize)]
pub struct SampleQuery {
    pub group_id: Option<i64>,
    pub user_name: Option<String>,
    pub start: Option<String>,
    pub end: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Deserialize)]
pub struct StatsQuery {
    pub start: Option<String>,
    pub end: Option<String>,
}

#[derive(Serialize)]
pub struct SampleStats {
    pub total_count: i64,
    pub total_samples: i64,
    pub by_group: Vec<GroupStat>,
    pub by_project: Vec<ProjectStat>,
    pub by_user: Vec<UserStat>,
}

#[derive(Serialize)]
pub struct GroupStat { pub group_name: String, pub count: i64, pub total_samples: i64 }

#[derive(Serialize)]
pub struct ProjectStat { pub project_name: String, pub group_name: String, pub count: i64, pub total_samples: i64 }

#[derive(Serialize)]
pub struct UserStat { pub user_name: String, pub count: i64, pub total_samples: i64 }

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/samples", get(list).post(create))
        .route("/api/samples/stats", get(stats))
        .route("/api/samples/:id", get(get_one).put(update).delete(soft_delete))
        .route("/api/samples/restore/:id", axum::routing::post(restore))
        .with_state(pool)
}

async fn stats(State(pool): State<DbPool>, Query(q): Query<StatsQuery>) -> Result<Json<ApiResponse<SampleStats>>> {
    let conn = pool.get()?;
    let s = q.start.as_deref().unwrap_or("2026-01-01");
    let e = q.end.as_deref().unwrap_or("2099-12-31");
    let e_closed = format!("{} 23:59:59", e);

    // Total
    let (tc, ts): (i64, i64) = conn.query_row(
        "SELECT COUNT(*), COALESCE(SUM(sample_count),0) FROM sample_records WHERE deleted_at IS NULL AND submitted_at>=?1 AND submitted_at<=?2",
        rusqlite::params![s, e_closed], |r| Ok((r.get(0)?, r.get(1)?)))?;

    // By group
    let mut stmt = conn.prepare(
        "SELECT pg.name, COUNT(*), COALESCE(SUM(sr.sample_count),0) FROM sample_records sr JOIN project_groups pg ON sr.group_id=pg.id WHERE sr.deleted_at IS NULL AND sr.submitted_at>=?1 AND sr.submitted_at<=?2 GROUP BY sr.group_id ORDER BY pg.sort_order"
    )?;
    let by_group: Vec<GroupStat> = stmt.query_map(rusqlite::params![s, e_closed], |r| Ok(GroupStat{group_name: r.get(0)?, count: r.get(1)?, total_samples: r.get(2)?}))?.collect::<std::result::Result<Vec<_>,_>>()?;

    // By project
    let mut stmt = conn.prepare(
        "SELECT p.name, pg.name, COUNT(*), COALESCE(SUM(sr.sample_count),0) FROM sample_records sr JOIN projects p ON sr.project_id=p.id JOIN project_groups pg ON sr.group_id=pg.id WHERE sr.deleted_at IS NULL AND sr.submitted_at>=?1 AND sr.submitted_at<=?2 GROUP BY sr.project_id ORDER BY COUNT(*) DESC LIMIT 20"
    )?;
    let by_project: Vec<ProjectStat> = stmt.query_map(rusqlite::params![s, e_closed], |r| Ok(ProjectStat{project_name: r.get(0)?, group_name: r.get(1)?, count: r.get(2)?, total_samples: r.get(3)?}))?.collect::<std::result::Result<Vec<_>,_>>()?;

    // By user
    let mut stmt = conn.prepare(
        "SELECT sr.user_name, COUNT(*), COALESCE(SUM(sr.sample_count),0) FROM sample_records sr WHERE sr.deleted_at IS NULL AND sr.submitted_at>=?1 AND sr.submitted_at<=?2 GROUP BY sr.user_name ORDER BY COUNT(*) DESC LIMIT 20"
    )?;
    let by_user: Vec<UserStat> = stmt.query_map(rusqlite::params![s, e_closed], |r| Ok(UserStat{user_name: r.get(0)?, count: r.get(1)?, total_samples: r.get(2)?}))?.collect::<std::result::Result<Vec<_>,_>>()?;

    Ok(Json(ApiResponse::ok(SampleStats { total_count: tc, total_samples: ts, by_group, by_project, by_user })))
}

async fn list(State(pool): State<DbPool>, Query(q): Query<SampleQuery>) -> Result<Json<ApiResponse<PaginatedResponse<SampleRecordResponse>>>> {
    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(50).min(500);
    let (items, total) = sample_repo::list(&pool, q.group_id, q.user_name.as_deref(), q.start.as_deref(), q.end.as_deref(), page, page_size)?;
    Ok(Json(ApiResponse::ok(PaginatedResponse { items, total, page, page_size })))
}

async fn get_one(State(pool): State<DbPool>, Path(id): Path<i64>) -> Result<Json<ApiResponse<SampleRecordResponse>>> {
    Ok(Json(ApiResponse::ok(sample_repo::get_by_id(&pool, id)?)))
}

async fn create(State(pool): State<DbPool>, Json(body): Json<SampleRecordCreate>) -> Result<Json<ApiResponse<SampleRecordResponse>>> {
    Ok(Json(ApiResponse::ok(sample_service::create(&pool, &body)?)))
}

async fn update(State(pool): State<DbPool>, Path(id): Path<i64>, Json(body): Json<SampleRecordUpdate>) -> Result<Json<ApiResponse<SampleRecordResponse>>> {
    Ok(Json(ApiResponse::ok(sample_service::update(&pool, id, &body)?)))
}

async fn soft_delete(State(pool): State<DbPool>, Path(id): Path<i64>) -> Result<Json<ApiResponse<()>>> {
    sample_service::delete(&pool, id)?;
    Ok(Json(ApiResponse::ok_msg("删除成功")))
}

async fn restore(State(pool): State<DbPool>, Path(id): Path<i64>) -> Result<Json<ApiResponse<SampleRecordResponse>>> {
    Ok(Json(ApiResponse::ok(sample_repo::restore(&pool, id)?)))
}
