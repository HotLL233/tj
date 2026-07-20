use crate::db::DbPool;
use crate::error::{AppError, Result};
use crate::models::sample_info::{
    SampleInfoCreate, SampleInfoQuery, SampleInfoResponse, SampleInfoStatusUpdate, SampleInfoUpdate,
};
use crate::models::{ApiResponse, PaginatedResponse};
use crate::repo::sample_info_repo;
use crate::service::authz_service::{self, AuthContext};
use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::get,
    Json, Router,
};
use serde::Serialize;

fn ensure_status_permission(
    pool: &DbPool,
    headers: &HeaderMap,
    target_status: &str,
) -> Result<String> {
    let ctx = authz_service::authenticate(pool, headers)?;
    let required = match target_status {
        "待检测" => "sample-info:collect",
        "已检测" => "sample-info:complete",
        _ => return Err(AppError::Validation("无效的样品状态操作".into())),
    };
    if !ctx.has_permission(required) {
        return Err(AppError::Forbidden("无权执行该样品状态操作".into()));
    }
    Ok(ctx.user.username)
}

fn apply_record_scope(ctx: &AuthContext, query: &mut SampleInfoQuery) -> Result<()> {
    if ctx.is_system_admin() || (ctx.is_analysis_member() && (!query.include_deleted.unwrap_or(false) || ctx.is_analysis_leader())) {
        return Ok(());
    }
    if ctx.is_rd_leader() {
        query.group_id = Some(
            ctx.user.group_id
                .ok_or_else(|| AppError::Validation("研发送样组长尚未设置所属实验室".into()))?,
        );
    } else {
        query.created_by_user_id = Some(ctx.user.id);
    }
    Ok(())
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
        .route("/api/sample-info/:id/sample", axum::routing::put(sample))
        .route("/api/sample-info/:id/restore", axum::routing::post(restore))
        .route(
            "/api/sample-info/:id/complete",
            axum::routing::put(complete),
        )
        .with_state(pool)
}

async fn list(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Query(mut q): Query<SampleInfoQuery>,
) -> Result<Json<ApiResponse<PaginatedResponse<SampleInfoResponse>>>> {
    let ctx = authz_service::authenticate(&pool, &headers)?;
    authz_service::require_permission(&ctx, "entry:sample-info")?;
    apply_record_scope(&ctx, &mut q)?;
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
    Json(mut body): Json<SampleInfoCreate>,
) -> Result<Json<ApiResponse<SampleInfoResponse>>> {
    let ctx = authz_service::authenticate(&pool, &headers)?;
    authz_service::require_permission(&ctx, "entry:sample-info")?;
    body.user_name = ctx.user.username.clone();
    let record = sample_info_repo::create(&pool, &body, &ctx.user.username)?;
    Ok(Json(ApiResponse::ok(record)))
}

async fn update(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(body): Json<SampleInfoUpdate>,
) -> Result<Json<ApiResponse<SampleInfoResponse>>> {
    let ctx = authz_service::authenticate(&pool, &headers)?;
    let existing = sample_info_repo::get_by_id(&pool, id)?;
    if !ctx.is_system_admin() && existing.created_by_user_id != Some(ctx.user.id) {
        return Err(AppError::Forbidden("只能修改本人提交的样品信息".into()));
    }
    if existing.sampled_at.is_some() {
        return Err(AppError::Forbidden("该记录已取样，不可修改".into()));
    }
    let record = sample_info_repo::update(&pool, id, &body, &ctx.user.username)?;
    Ok(Json(ApiResponse::ok(record)))
}

async fn soft_delete(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<()>>> {
    let ctx = authz_service::authenticate(&pool, &headers)?;
    let existing = sample_info_repo::get_by_id(&pool, id)?;
    if !ctx.is_system_admin() && existing.created_by_user_id != Some(ctx.user.id) {
        return Err(AppError::Forbidden("只能删除本人提交的样品信息".into()));
    }
    sample_info_repo::soft_delete(&pool, id, &ctx.user.username)?;
    Ok(Json(ApiResponse::ok_msg("删除成功")))
}

async fn restore(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<SampleInfoResponse>>> {
    let ctx = authz_service::authenticate(&pool, &headers)?;
    let existing = sample_info_repo::get_by_id(&pool, id)?;
    let can_restore = ctx.is_system_admin()
        || ctx.is_analysis_leader()
        || existing.created_by_user_id == Some(ctx.user.id)
        || (ctx.is_rd_leader() && existing.group_id.is_some() && existing.group_id == ctx.user.group_id);
    if !can_restore {
        return Err(AppError::Forbidden("只能恢复本人提交的样品信息".into()));
    }
    Ok(Json(ApiResponse::ok(sample_info_repo::restore(
        &pool,
        id,
        &ctx.user.username,
    )?)))
}

async fn update_status(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(body): Json<SampleInfoStatusUpdate>,
) -> Result<Json<ApiResponse<SampleInfoResponse>>> {
    let user_name = ensure_status_permission(&pool, &headers, &body.status)?;
    let record = sample_info_repo::update_status(&pool, id, &body.status, &user_name)?;
    Ok(Json(ApiResponse::ok(record)))
}

async fn sample(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<SampleInfoResponse>>> {
    let user_name = ensure_status_permission(&pool, &headers, "待检测")?;
    Ok(Json(ApiResponse::ok(sample_info_repo::update_status(
        &pool,
        id,
        "待检测",
        &user_name,
    )?)))
}

async fn complete(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<SampleInfoResponse>>> {
    let user_name = ensure_status_permission(&pool, &headers, "已检测")?;
    Ok(Json(ApiResponse::ok(sample_info_repo::update_status(
        &pool,
        id,
        "已检测",
        &user_name,
    )?)))
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
    if let Some(user_name) = &q.user_name {
        let i = params.len() + 1;
        clauses.push(format!("user_name=?{}", i));
        params.push(user_name.clone());
    }
    if let Some(user_id) = q.created_by_user_id {
        let i = params.len() + 1;
        clauses.push(format!("created_by_user_id=?{}", i));
        params.push(user_id.to_string());
    }
    if let Some(lab_name) = &q.lab_name {
        let i = params.len() + 1;
        clauses.push(format!("lab_name=?{}", i));
        params.push(lab_name.clone());
    }
    if let Some(group_id) = q.group_id {
        let i = params.len() + 1;
        clauses.push(format!("group_id=?{}", i));
        params.push(group_id.to_string());
    }
    (clauses.join(" AND "), params)
}

async fn stats(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Query(mut q): Query<SampleInfoQuery>,
) -> Result<Json<ApiResponse<SampleInfoStats>>> {
    let ctx = authz_service::authenticate(&pool, &headers)?;
    apply_record_scope(&ctx, &mut q)?;
    let conn = pool.get()?;
    let (wc, params) = stats_where(&q);
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params
        .iter()
        .map(|s| s as &dyn rusqlite::types::ToSql)
        .collect();

    let total: i64 = conn.query_row(
        &format!("SELECT COUNT(*) FROM sample_info_records WHERE {}", wc),
        rusqlite::params_from_iter(param_refs.iter()),
        |r| r.get(0),
    )?;

    let mut q_status = conn.prepare(
        &format!("SELECT status, COUNT(*) FROM sample_info_records WHERE {} GROUP BY status ORDER BY COUNT(*) DESC", wc),
    )?;
    let by_status: Vec<NameCount> = q_status
        .query_map(rusqlite::params_from_iter(param_refs.iter()), |row| {
            Ok(NameCount {
                name: row.get(0)?,
                count: row.get(1)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let mut q_type = conn.prepare(
        &format!("SELECT COALESCE(sit.label, sir.detection_type) AS t, sir.type_key, COUNT(*) \
                  FROM sample_info_records sir LEFT JOIN sample_info_types sit ON sit.type_key = sir.type_key \
                  WHERE {} GROUP BY t, sir.type_key ORDER BY COUNT(*) DESC", wc),
    )?;
    let by_type: Vec<TypeCount> = q_type
        .query_map(rusqlite::params_from_iter(param_refs.iter()), |row| {
            Ok(TypeCount {
                label: row.get(0)?,
                type_key: row.get::<_, String>(1).unwrap_or_default(),
                count: row.get(2)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let mut q_lab = conn.prepare(
        &format!("SELECT lab_name, COUNT(*) FROM sample_info_records WHERE {} GROUP BY lab_name ORDER BY COUNT(*) DESC", wc),
    )?;
    let by_lab: Vec<NameCount> = q_lab
        .query_map(rusqlite::params_from_iter(param_refs.iter()), |row| {
            Ok(NameCount {
                name: row.get(0)?,
                count: row.get(1)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let mut q_proj = conn.prepare(
        &format!("SELECT project_name, COUNT(*) FROM sample_info_records WHERE {} GROUP BY project_name ORDER BY COUNT(*) DESC", wc),
    )?;
    let by_project: Vec<NameCount> = q_proj
        .query_map(rusqlite::params_from_iter(param_refs.iter()), |row| {
            Ok(NameCount {
                name: row.get(0)?,
                count: row.get(1)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let mut q_user = conn.prepare(
        &format!("SELECT COALESCE(u.username,MAX(sir.user_name)), COUNT(*)
                  FROM sample_info_records sir LEFT JOIN users u ON u.id=sir.created_by_user_id
                  WHERE {} GROUP BY COALESCE(CAST(sir.created_by_user_id AS TEXT),'legacy:' || sir.user_name)
                  ORDER BY COUNT(*) DESC", wc),
    )?;
    let by_user: Vec<NameCount> = q_user
        .query_map(rusqlite::params_from_iter(param_refs.iter()), |row| {
            Ok(NameCount {
                name: row.get(0)?,
                count: row.get(1)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let mut q_month = conn.prepare(
        &format!("SELECT strftime('%Y-%m', submitted_at) AS m, COUNT(*) FROM sample_info_records WHERE {} GROUP BY m ORDER BY m ASC", wc),
    )?;
    let by_month: Vec<MonthCount> = q_month
        .query_map(rusqlite::params_from_iter(param_refs.iter()), |row| {
            Ok(MonthCount {
                month: row.get::<_, String>(0).unwrap_or_default(),
                count: row.get(1)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(Json(ApiResponse::ok(SampleInfoStats {
        total,
        by_status,
        by_type,
        by_lab,
        by_project,
        by_user,
        by_month,
    })))
}
