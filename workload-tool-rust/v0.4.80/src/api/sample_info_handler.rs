use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json, Router, routing::get,
};
use serde::Serialize;
use crate::db::DbPool;
use crate::error::Result;
use crate::models::sample_info::{
    SampleInfoCreate, SampleInfoQuery, SampleInfoStatusUpdate, SampleInfoResponse,
    SampleInfoUpdate,
};
use crate::models::{ApiResponse, PaginatedResponse};
use crate::repo::sample_info_repo;
use crate::repo::audit_repo;

fn extract_claims_username(headers: &HeaderMap) -> String {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .and_then(|token| crate::service::auth_service::verify_token(token).ok())
        .map(|c| c.username)
        .unwrap_or_else(|| "system".to_string())
}

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/sample-info", get(list).post(create))
        .route("/api/sample-info/stats", get(stats))
        .route(
            "/api/sample-info/:id",
            axum::routing::put(update).delete(soft_delete),
        )
        .route(
            "/api/sample-info/:id/status",
            axum::routing::put(update_status),
        )
        .with_state(pool)
}

async fn list(
    State(pool): State<DbPool>,
    Query(q): Query<SampleInfoQuery>,
) -> Result<Json<ApiResponse<PaginatedResponse<SampleInfoResponse>>>> {
    let (items, total) = sample_info_repo::list(&pool, &q)?;
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).min(500);
    Ok(Json(ApiResponse::ok(PaginatedResponse {
        items,
        total,
        page,
        page_size,
    })))
}

async fn create(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Json(body): Json<SampleInfoCreate>,
) -> Result<Json<ApiResponse<SampleInfoResponse>>> {
    let record = sample_info_repo::create(&pool, &body)?;
    let user_name = extract_claims_username(&headers);
    let conn = pool.get()?;
    audit_repo::log_on_conn_with_module(
        &conn, "create", "sample_info_records", Some(record.id),
        &user_name, &format!("创建样品信息登记#{} 批号「{}」", record.seq_no, record.batch_no), "shared",
    )?;
    Ok(Json(ApiResponse::ok(record)))
}

async fn update(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(body): Json<SampleInfoUpdate>,
) -> Result<Json<ApiResponse<SampleInfoResponse>>> {
    let user_name = extract_claims_username(&headers);
    let record = sample_info_repo::update(&pool, id, &body, &user_name)?;
    let conn = pool.get()?;
    audit_repo::log_on_conn_with_module(
        &conn, "update", "sample_info_records", Some(id),
        &user_name, &format!("更新样品信息记录#{}", id), "shared",
    )?;
    Ok(Json(ApiResponse::ok(record)))
}

async fn soft_delete(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<()>>> {
    let user_name = extract_claims_username(&headers);
    sample_info_repo::soft_delete(&pool, id, &user_name)?;
    let conn = pool.get()?;
    audit_repo::log_on_conn_with_module(
        &conn, "delete", "sample_info_records", Some(id),
        &user_name, &format!("删除样品信息记录#{}", id), "shared",
    )?;
    Ok(Json(ApiResponse::ok_msg("删除成功")))
}

async fn update_status(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(body): Json<SampleInfoStatusUpdate>,
) -> Result<Json<ApiResponse<SampleInfoResponse>>> {
    let user_name = extract_claims_username(&headers);
    let record = sample_info_repo::update_status(&pool, id, &body.status, &user_name)?;
    let conn = pool.get()?;
    audit_repo::log_on_conn_with_module(
        &conn, "change_status", "sample_info_records", Some(id),
        &user_name, &format!("样品信息记录#{} 状态变更为「{}」", id, body.status), "shared",
    )?;
    Ok(Json(ApiResponse::ok(record)))
}

// ========== 独立统计（不接分析检测 /stats） ==========

#[derive(Serialize)]
pub struct NameCount {
    pub name: String,
    pub count: i64,
}

#[derive(Serialize)]
pub struct TypeCount {
    pub type_key: String,
    pub label: String,
    pub count: i64,
}

#[derive(Serialize)]
pub struct MonthCount {
    pub month: String,
    pub count: i64,
}

#[derive(Serialize)]
pub struct SampleInfoStats {
    pub total: i64,
    pub by_status: Vec<NameCount>,
    pub by_type: Vec<TypeCount>,
    pub by_lab: Vec<NameCount>,
    pub by_project: Vec<NameCount>,
    pub by_user: Vec<NameCount>,
    pub by_month: Vec<MonthCount>,
}

fn stats_where(q: &SampleInfoQuery) -> (String, Vec<String>) {
    let mut clauses: Vec<String> = vec!["deleted_at IS NULL".to_string()];
    let mut params: Vec<String> = vec![];
    if let Some(tk) = &q.type_key {
        if !tk.is_empty() {
            let i = params.len() + 1;
            clauses.push(format!("type_key=?{}", i));
            params.push(tk.clone());
        }
    }
    if let Some(s) = &q.status {
        if !s.is_empty() && s != "全部" {
            let i = params.len() + 1;
            clauses.push(format!("status=?{}", i));
            params.push(s.clone());
        }
    }
    if let Some(st) = &q.start {
        let i = params.len() + 1;
        clauses.push(format!("submitted_at>=?{}", i));
        params.push(st.clone());
    }
    if let Some(e) = &q.end {
        let i = params.len() + 1;
        clauses.push(format!("submitted_at<=?{}", i));
        params.push(format!("{}T23:59:59", e));
    }
    (clauses.join(" AND "), params)
}

async fn stats(
    State(pool): State<DbPool>,
    Query(q): Query<SampleInfoQuery>,
) -> Result<Json<ApiResponse<SampleInfoStats>>> {
    let conn = pool.get()?;
    let (wc, params) = stats_where(&q);
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect();

    let total: i64 = conn.query_row(
        &format!("SELECT COUNT(*) FROM sample_info_records WHERE {}", wc),
        rusqlite::params_from_iter(param_refs.iter()),
        |r| r.get(0),
    )?;

    let mut q_status = conn.prepare(
        &format!("SELECT status, COUNT(*) FROM sample_info_records WHERE {} GROUP BY status ORDER BY COUNT(*) DESC", wc),
    )?;
    let by_status: Vec<NameCount> = q_status.query_map(rusqlite::params_from_iter(param_refs.iter()), |row| {
        Ok(NameCount { name: row.get(0)?, count: row.get(1)? })
    })?.collect::<std::result::Result<Vec<_>, _>>()?;

    let mut q_type = conn.prepare(
        &format!("SELECT COALESCE(sit.label, sir.detection_type) AS t, sir.type_key, COUNT(*) \
                  FROM sample_info_records sir LEFT JOIN sample_info_types sit ON sit.type_key = sir.type_key \
                  WHERE {} GROUP BY t, sir.type_key ORDER BY COUNT(*) DESC", wc),
    )?;
    let by_type: Vec<TypeCount> = q_type.query_map(rusqlite::params_from_iter(param_refs.iter()), |row| {
        Ok(TypeCount { label: row.get(0)?, type_key: row.get::<_, String>(1).unwrap_or_default(), count: row.get(2)? })
    })?.collect::<std::result::Result<Vec<_>, _>>()?;

    let mut q_lab = conn.prepare(
        &format!("SELECT lab_name, COUNT(*) FROM sample_info_records WHERE {} GROUP BY lab_name ORDER BY COUNT(*) DESC", wc),
    )?;
    let by_lab: Vec<NameCount> = q_lab.query_map(rusqlite::params_from_iter(param_refs.iter()), |row| {
        Ok(NameCount { name: row.get(0)?, count: row.get(1)? })
    })?.collect::<std::result::Result<Vec<_>, _>>()?;

    let mut q_proj = conn.prepare(
        &format!("SELECT project_name, COUNT(*) FROM sample_info_records WHERE {} GROUP BY project_name ORDER BY COUNT(*) DESC", wc),
    )?;
    let by_project: Vec<NameCount> = q_proj.query_map(rusqlite::params_from_iter(param_refs.iter()), |row| {
        Ok(NameCount { name: row.get(0)?, count: row.get(1)? })
    })?.collect::<std::result::Result<Vec<_>, _>>()?;

    let mut q_user = conn.prepare(
        &format!("SELECT user_name, COUNT(*) FROM sample_info_records WHERE {} GROUP BY user_name ORDER BY COUNT(*) DESC", wc),
    )?;
    let by_user: Vec<NameCount> = q_user.query_map(rusqlite::params_from_iter(param_refs.iter()), |row| {
        Ok(NameCount { name: row.get(0)?, count: row.get(1)? })
    })?.collect::<std::result::Result<Vec<_>, _>>()?;

    let mut q_month = conn.prepare(
        &format!("SELECT strftime('%Y-%m', submitted_at) AS m, COUNT(*) FROM sample_info_records WHERE {} GROUP BY m ORDER BY m ASC", wc),
    )?;
    let by_month: Vec<MonthCount> = q_month.query_map(rusqlite::params_from_iter(param_refs.iter()), |row| {
        Ok(MonthCount { month: row.get::<_, String>(0).unwrap_or_default(), count: row.get(1)? })
    })?.collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(Json(ApiResponse::ok(SampleInfoStats {
        total, by_status, by_type, by_lab, by_project, by_user, by_month,
    })))
}
