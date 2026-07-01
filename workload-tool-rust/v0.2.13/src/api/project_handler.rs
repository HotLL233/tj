use axum::{extract::{Multipart, Path, Query, State}, Json, Router, routing::get};
use serde::Deserialize;
use std::collections::BTreeMap;
use crate::db::DbPool;
use crate::error::{Result, AppError};
use crate::models::ApiResponse;
use crate::models::project::*;
use crate::repo::project_repo;

#[derive(Deserialize)] pub struct ProjectQuery { pub group_id: Option<i64>, pub active_only: Option<bool>, pub method_type: Option<String> }
#[derive(Deserialize)] pub struct BatchCoefficientPayload { pub group_id: i64, pub coefficient: f64 }

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/projects", get(list).post(create))
        .route("/api/projects/:id", axum::routing::put(update).delete(delete))
        .route("/api/projects/batch-coefficient", axum::routing::put(batch_coefficient))
        .route("/api/projects/method-import", axum::routing::post(method_import))
        .route("/api/method-types", get(list_method_types).post(create_method_type))
        .route("/api/method-types/:id", axum::routing::put(update_method_type).delete(delete_method_type))
        .with_state(pool)
}

async fn list(State(pool): State<DbPool>, Query(q): Query<ProjectQuery>) -> Result<Json<ApiResponse<Vec<ProjectResponse>>>> {
    let items = project_repo::list(&pool, q.group_id, q.active_only.unwrap_or(false), q.method_type.as_deref())?;
    Ok(Json(ApiResponse::ok(items)))
}
async fn create(State(pool): State<DbPool>, Json(b): Json<ProjectCreate>) -> Result<Json<ApiResponse<ProjectResponse>>> { Ok(Json(ApiResponse::ok(project_repo::create(&pool, &b)?))) }
async fn update(State(pool): State<DbPool>, Path(id): Path<i64>, Json(b): Json<ProjectUpdate>) -> Result<Json<ApiResponse<ProjectResponse>>> { Ok(Json(ApiResponse::ok(project_repo::update(&pool, id, &b)?))) }
async fn delete(State(pool): State<DbPool>, Path(id): Path<i64>) -> Result<Json<ApiResponse<()>>> { project_repo::delete(&pool, id)?; Ok(Json(ApiResponse::ok_msg("删除成功"))) }
async fn batch_coefficient(State(pool): State<DbPool>, Json(b): Json<BatchCoefficientPayload>) -> Result<Json<ApiResponse<i64>>> { let c = project_repo::batch_coefficient(&pool, b.group_id, b.coefficient)?; Ok(Json(ApiResponse::ok_msg(format!("已更新{}个项目系数", c)))) }

/// v0.2.13: 按列名精准分类导入
/// Excel 结构: 列0=实验室, 列1=研发项目, 列2+=检测方法类型(液相/气相/理化/ICP/热分析/其他)
/// 实验室列(410/412/413)→ 实验室卡片
/// 研发项目列(A001/A002)→ 研发项目管理卡片
/// 方法类型列(液相/气相/...)→ 方法管理卡片
/// 非空单元格=1条独立方法, 跨行同方法名+同组去重
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
    if rows.len() < 2 { return Err(AppError::Validation("至少2行(表头+数据)".into())); }
    let hdrs: Vec<String> = rows[0].iter().map(|v| v.as_string().unwrap_or_default().to_string()).collect();
    if hdrs.len() < 3 { return Err(AppError::Validation("至少3列(实验室, 研发项目, 检测方法)".into())); }

    // 解析表头→类型映射
    let mut col_types: Vec<String> = Vec::new();
    for h in hdrs.iter().skip(2) {
        let h = h.trim();
        let t = if h.contains("液相") { "液相" }
        else if h.contains("气相") { "气相" }
        else if h.contains("理化") { "理化" }
        else if h.to_uppercase().contains("ICP") { "ICP" }
        else if h.contains("热分析") { "热分析" }
        else { "其他" };
        col_types.push(t.to_string());
    }

    // 按 (实验室, 研发项目) 分组, 每组下 (类型, 方法名) 去重
    let mut grouped: BTreeMap<(String, String), Vec<(String, String)>> = BTreeMap::new();
    for r in rows.iter().skip(1) {
        let lab = r.get(0).and_then(|v| v.as_string()).unwrap_or_default().trim().to_string();
        let proj = r.get(1).and_then(|v| v.as_string()).unwrap_or_default().trim().to_string();
        if lab.is_empty() || proj.is_empty() { continue; }
        for (i, mt) in col_types.iter().enumerate() {
            let mname = r.get(i + 2).and_then(|v| v.as_string()).unwrap_or_default().trim().to_string();
            if mname.is_empty() { continue; }
            let entry = grouped.entry((lab.clone(), proj.clone())).or_default();
            // 同组内去重
            if !entry.iter().any(|(t, n): &(String, String)| t == mt && n == &mname) {
                entry.push((mt.clone(), mname));
            }
        }
    }
    if grouped.is_empty() { return Err(AppError::Validation("无有效数据".into())); }

    let conn = pool.get()?;
    let summary = project_repo::batch_import_card_classified(&conn, &grouped)?;
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
