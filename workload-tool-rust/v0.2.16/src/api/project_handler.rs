use axum::{extract::{Multipart, Path, Query, State}, Json, Router, routing::get};
use serde::Deserialize;
// BTreeMap removed — no longer needed in handler (moved to repo layer)
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

/// v0.2.14: 三层级映射导入
/// 第一层级: 实验室(410/412/413) → 实验室管理
///           研发项目(A001/A002/A003) → 研发项目管理
///           检测方法 = 液相方法/气相方法 → 检测方法管理
/// 第二层级: 液相方法/气相方法 下的方法名(11, 21) → 检测方法管理
/// 第三层级: 理化方法/ICP/热分析/其他 下的方法名(氢离子, 33, 41) → 各自管理模块
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
    if hdrs.len() < 3 { return Err(AppError::Validation("至少3列".into())); }

    // 3. 新逻辑：按列解析 — 每列独立，列头=分组名
    let headers: Vec<String> = rows[0].iter()
        .map(|v| v.as_string().unwrap_or_default().trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if headers.is_empty() { return Err(AppError::Validation("表头为空".into())); }

    let ncols = headers.len().min(rows[0].len());
    let mut items: Vec<(String, String, String)> = Vec::new();

    for col_idx in 0..ncols {
        let header = &headers[col_idx];
        let method_type = classify_method_type(header);

        // "实验室管理" 列只创建分组，不创建项目 (item_name="")
        if header == "实验室管理" {
            for r in rows.iter().skip(1) {
                if let Some(v) = r.get(col_idx) {
                    let val = v.as_string().unwrap_or_default().trim().to_string();
                    if !val.is_empty() {
                        items.push((val.clone(), String::new(), String::new()));
                    }
                }
            }
            continue;
        }

        for r in rows.iter().skip(1) {
            if let Some(v) = r.get(col_idx) {
                let val = v.as_string().unwrap_or_default().trim().to_string();
                if !val.is_empty() {
                    if !items.iter().any(|(g, n, _)| g == header && n == &val) {
                        items.push((header.clone(), val, method_type.clone()));
                    }
                }
            }
        }
    }
    if items.is_empty() { return Err(AppError::Validation("无有效数据".into())); }

    let conn = pool.get()?;
    let summary = project_repo::batch_import_by_column(&conn, &items)?;
    std::fs::remove_file(&tp).ok();
    Ok(Json(ApiResponse::ok(summary)))
}

/// 根据列头分类 method_type：含"方法"→"检测方法"，含"研发"→"研发项目"
fn classify_method_type(header: &str) -> String {
    if header.contains("方法") { return "检测方法".into(); }
    if header.contains("研发") { return "研发项目".into(); }
    header.to_string()
}

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
