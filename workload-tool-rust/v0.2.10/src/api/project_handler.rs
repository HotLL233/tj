use axum::{extract::{Multipart, Path, Query, State}, Json, Router, routing::get};
use serde::Deserialize;
use crate::db::DbPool;
use crate::error::{Result, AppError};
use crate::models::ApiResponse;
use crate::models::project::*;
use crate::repo::project_repo;

#[derive(Deserialize)] pub struct ProjectQuery { pub group_id: Option<i64>, pub active_only: Option<bool>, pub method_type: Option<String>, pub include_methods: Option<bool> }
#[derive(Deserialize)] pub struct MethodsQuery { pub method_type: Option<String> }
#[derive(Deserialize)] pub struct BatchCoefficientPayload { pub group_id: i64, pub coefficient: f64 }

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/projects", get(list).post(create))
        .route("/api/projects/:id", axum::routing::put(update).delete(delete))
        .route("/api/projects/batch-coefficient", axum::routing::put(batch_coefficient))
        .route("/api/projects/method-import", axum::routing::post(method_import))
        .route("/api/projects/:id/methods", get(child_methods))
        .route("/api/methods", get(list_methods))
        .route("/api/method-types", get(list_method_types).post(create_method_type))
        .route("/api/method-types/:id", axum::routing::put(update_method_type).delete(delete_method_type))
        .with_state(pool)
}

// ── 项目管理：仅顶级项目(include_methods=true 时含子方法) ──

async fn list(State(pool): State<DbPool>, Query(q): Query<ProjectQuery>) -> Result<Json<ApiResponse<Vec<ProjectResponse>>>> {
    let items = project_repo::list(&pool, q.group_id, q.active_only.unwrap_or(false), q.method_type.as_deref(), q.include_methods.unwrap_or(false))?;
    Ok(Json(ApiResponse::ok(items)))
}

/// 查某项目子方法
async fn child_methods(State(pool): State<DbPool>, Path(id): Path<i64>) -> Result<Json<ApiResponse<Vec<ProjectResponse>>>> {
    let conn = pool.get()?; // 复用内部查询
    let items: Vec<ProjectResponse> = conn.prepare(
        "SELECT p.id, p.group_id, pg.name, p.name, COALESCE(p.full_name,''), COALESCE(p.notes,''), p.sort_order, p.is_active, COALESCE(p.coefficient,1.0), p.method_type, COALESCE(p.parent_id,0), COALESCE(p.created_at,'') FROM projects p JOIN project_groups pg ON p.group_id=pg.id WHERE p.parent_id=? ORDER BY p.sort_order"
    )?.query_map([id], |row| project_repo::row_to_project(row)).map_err(|e| AppError::Internal(format!("{}",e)))?.collect::<std::result::Result<Vec<_>,_>>().map_err(|e| AppError::Internal(format!("{}",e)))?;
    Ok(Json(ApiResponse::ok(items)))
}

async fn create(State(pool): State<DbPool>, Json(b): Json<ProjectCreate>) -> Result<Json<ApiResponse<ProjectResponse>>> { Ok(Json(ApiResponse::ok(project_repo::create(&pool, &b)?))) }
async fn update(State(pool): State<DbPool>, Path(id): Path<i64>, Json(b): Json<ProjectUpdate>) -> Result<Json<ApiResponse<ProjectResponse>>> { Ok(Json(ApiResponse::ok(project_repo::update(&pool, id, &b)?))) }
async fn delete(State(pool): State<DbPool>, Path(id): Path<i64>) -> Result<Json<ApiResponse<()>>> { project_repo::delete(&pool, id)?; Ok(Json(ApiResponse::ok_msg("删除成功"))) }
async fn batch_coefficient(State(pool): State<DbPool>, Json(b): Json<BatchCoefficientPayload>) -> Result<Json<ApiResponse<i64>>> { let c = project_repo::batch_coefficient(&pool, b.group_id, b.coefficient)?; Ok(Json(ApiResponse::ok_msg(format!("已更新{}个项目系数", c)))) }

// ── 方法管理：仅子级检测方法 ──

async fn list_methods(State(pool): State<DbPool>, Query(q): Query<MethodsQuery>) -> Result<Json<ApiResponse<Vec<ProjectResponse>>>> {
    Ok(Json(ApiResponse::ok(project_repo::list_methods(&pool, q.method_type.as_deref())?)))
}

// ── 三级导入 ──

async fn method_import(State(pool): State<DbPool>, mut mp: Multipart) -> Result<Json<ApiResponse<ImportSummary>>> {
    let mut tp = String::new();
    while let Ok(Some(f)) = mp.next_field().await {
        if f.name()==Some("file") {
            if let Ok(d) = f.bytes().await { if !d.is_empty() {
                let p = std::env::temp_dir().join(format!("md_{}.xlsx", uuid::Uuid::new_v4()));
                if std::fs::write(&p, &d).is_ok() { tp = p.to_string_lossy().to_string(); }
            }}
        }
    }
    if tp.is_empty() { return Err(AppError::Validation("未收到文件".into())); }
    use calamine::{open_workbook, Reader, Xlsx};
    let mut wb: Xlsx<_> = open_workbook(&tp).map_err(|e| AppError::Validation(format!("打开失败:{}", e)))?;
    let sh = wb.sheet_names().to_vec(); if sh.is_empty() { return Err(AppError::Validation("无工作表".into())); }
    let rng = wb.worksheet_range(&sh[0]).map_err(|e| AppError::Validation(format!("读取失败:{}", e)))?;
    let rows: Vec<Vec<calamine::DataType>> = rng.rows().map(|r| r.to_vec()).collect();
    if rows.len() < 2 { return Err(AppError::Validation("至少2行".into())); }
    let hdrs: Vec<String> = rows[0].iter().map(|v| v.as_string().unwrap_or_default().to_string()).collect();
    if hdrs.len() < 3 { return Err(AppError::Validation("至少3列(实验室, 研发项目, 方法类型)".into())); }
    let mut type_columns: Vec<String> = Vec::new();
    for h in hdrs.iter().skip(2) {
        let h = h.trim();
        if h.contains("液相") { type_columns.push("液相".into()); }
        else if h.contains("气相") { type_columns.push("气相".into()); }
        else if h.contains("理化") { type_columns.push("理化".into()); }
        else if h.contains("ICP") || h.contains("icp") { type_columns.push("ICP".into()); }
        else if h.contains("热分析") { type_columns.push("热分析".into()); }
        else { type_columns.push("其他".into()); }
    }
    let mut triplets: Vec<(String, String, String, String)> = Vec::new();
    for r in rows.iter().skip(1) {
        let lab = r.get(0).and_then(|v| v.as_string()).unwrap_or_default().trim().to_string();
        let proj = r.get(1).and_then(|v| v.as_string()).unwrap_or_default().trim().to_string();
        if lab.is_empty() || proj.is_empty() { continue; }
        for (i, mt) in type_columns.iter().enumerate() {
            if let Some(v) = r.get(i + 2) {
                let mname = v.as_string().unwrap_or_default().trim().to_string();
                if mname.is_empty() { continue; }
                triplets.push((lab.clone(), proj.clone(), mt.clone(), mname));
            }
        }
    }
    if triplets.is_empty() { return Err(AppError::Validation("无有效数据".into())); }
    let mut items: Vec<MethodImportItem> = Vec::new();
    for (lab, proj, mt, mname) in triplets {
        let upper = mname.to_uppercase();
        let final_type = if upper.starts_with("LC-") || upper.starts_with("LC") { "液相".to_string() }
        else if upper.starts_with("GC-") || upper.starts_with("GC") { "气相".to_string() }
        else { mt };
        items.push(MethodImportItem { group_name: lab, project_name: proj, method_name: mname, method_type: final_type, coefficient: 1.0 });
    }
    let conn = pool.get()?;
    let summary = project_repo::batch_import_tree(&conn, &items)?;
    std::fs::remove_file(&tp).ok();
    Ok(Json(ApiResponse::ok(summary)))
}

// ── 方法类型 ──

async fn list_method_types(State(pool): State<DbPool>) -> Result<Json<ApiResponse<Vec<MethodType>>>> {
    Ok(Json(ApiResponse::ok(project_repo::list_method_types(&pool)?)))
}
async fn create_method_type(State(pool): State<DbPool>, Json(b): Json<MethodTypeCreate>) -> Result<Json<ApiResponse<MethodType>>> {
    Ok(Json(ApiResponse::ok(project_repo::create_method_type(&pool, &b)?)))
}
async fn update_method_type(State(pool): State<DbPool>, Path(id): Path<i64>, Json(b): Json<MethodTypeUpdate>) -> Result<Json<ApiResponse<MethodType>>> {
    Ok(Json(ApiResponse::ok(project_repo::update_method_type(&pool, id, &b)?)))
}
async fn delete_method_type(State(pool): State<DbPool>, Path(id): Path<i64>) -> Result<Json<ApiResponse<()>>> {
    project_repo::delete_method_type(&pool, id)?; Ok(Json(ApiResponse::ok_msg("删除成功")))
}
