use axum::{extract::{Multipart, Path, State}, Json, Router};
use serde::Deserialize;
use crate::db::DbPool;
use crate::error::{Result, AppError};
use crate::models::ApiResponse;
use crate::models::method::*;
use crate::models::project::*;
use crate::repo::method_repo;

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/methods", axum::routing::get(list).post(create))
        .route("/api/methods/:id", axum::routing::put(update).delete(delete))
        .route("/api/methods/import", axum::routing::post(method_import))
        .route("/api/method-types", axum::routing::get(list_method_types).post(create_method_type))
        .route("/api/method-types/:id", axum::routing::put(update_method_type).delete(delete_method_type))
        .with_state(pool)
}

#[derive(Deserialize)]
pub struct MethodQuery {
    pub type_id: Option<i64>,
}

async fn list(State(pool): State<DbPool>, axum::extract::Query(q): axum::extract::Query<MethodQuery>) -> Result<Json<ApiResponse<Vec<MethodResponse>>>> {
    let items = method_repo::list(&pool, q.type_id)?;
    Ok(Json(ApiResponse::ok(items)))
}

async fn create(State(pool): State<DbPool>, Json(b): Json<MethodCreate>) -> Result<Json<ApiResponse<MethodResponse>>> {
    Ok(Json(ApiResponse::ok(method_repo::create(&pool, &b)?)))
}

async fn update(State(pool): State<DbPool>, Path(id): Path<i64>, Json(b): Json<MethodUpdate>) -> Result<Json<ApiResponse<MethodResponse>>> {
    Ok(Json(ApiResponse::ok(method_repo::update(&pool, id, &b)?)))
}

async fn delete(State(pool): State<DbPool>, Path(id): Path<i64>) -> Result<Json<ApiResponse<()>>> {
    method_repo::delete(&pool, id)?;
    Ok(Json(ApiResponse::ok_msg("删除成功")))
}

// ── 方法类型 ──

async fn list_method_types(State(pool): State<DbPool>) -> Result<Json<ApiResponse<Vec<MethodType>>>> {
    Ok(Json(ApiResponse::ok(method_repo::list_method_types(&pool)?)))
}

async fn create_method_type(State(pool): State<DbPool>, Json(b): Json<MethodTypeCreate>) -> Result<Json<ApiResponse<MethodType>>> {
    Ok(Json(ApiResponse::ok(method_repo::create_method_type(&pool, &b)?)))
}

async fn update_method_type(State(pool): State<DbPool>, Path(id): Path<i64>, Json(b): Json<MethodTypeUpdate>) -> Result<Json<ApiResponse<MethodType>>> {
    Ok(Json(ApiResponse::ok(method_repo::update_method_type(&pool, id, &b)?)))
}

async fn delete_method_type(State(pool): State<DbPool>, Path(id): Path<i64>) -> Result<Json<ApiResponse<()>>> {
    method_repo::delete_method_type(&pool, id)?;
    Ok(Json(ApiResponse::ok_msg("删除成功")))
}

// ── 导入 ──

/// v0.2.17: 方法导入 — 从 project_handler 移入，路由 POST /api/methods/import
async fn method_import(State(pool): State<DbPool>, mut mp: Multipart) -> Result<Json<ApiResponse<ImportSummary>>> {
    let mut tp = String::new();
    while let Ok(Some(f)) = mp.next_field().await {
        if f.name() == Some("file") {
            if let Ok(d) = f.bytes().await {
                if !d.is_empty() {
                    let p = std::env::temp_dir().join(format!("md_{}.xlsx", uuid::Uuid::new_v4()));
                    if std::fs::write(&p, &d).is_ok() { tp = p.to_string_lossy().to_string(); }
                }
            }
        }
    }
    if tp.is_empty() { return Err(AppError::Validation("未收到文件".into())); }

    use calamine::{open_workbook, Reader, Xlsx};
    let mut wb: Xlsx<_> = open_workbook(&tp).map_err(|e| AppError::Validation(format!("打开失败:{}", e)))?;
    let sh = wb.sheet_names().to_vec();
    if sh.is_empty() { return Err(AppError::Validation("无工作表".into())); }
    let rng = wb.worksheet_range(&sh[0]).map_err(|e| AppError::Validation(format!("读取失败:{}", e)))?;
    let rows: Vec<Vec<calamine::DataType>> = rng.rows().map(|r| r.to_vec()).collect();
    if rows.len() < 2 { return Err(AppError::Validation("至少2行(表头+数据)".into())); }

    let headers: Vec<String> = rows[0].iter()
        .map(|v| v.as_string().unwrap_or_default().trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if headers.is_empty() { return Err(AppError::Validation("表头为空".into())); }

    // v0.2.17: 按列类型分三路 — 实验室管理→project_groups, 研发项目→projects, 方法→methods
    let ncols = headers.len().min(rows[0].len());
    let mut method_items: Vec<(String, String, String)> = Vec::new(); // (header, name, method_type)
    let mut project_items: Vec<String> = Vec::new();  // 研发项目名列表
    let mut group_items: Vec<String> = Vec::new();    // 实验室名列表

    for col_idx in 0..ncols {
        let header = &headers[col_idx];
        let h_clean = header.trim();

        // 实验室管理 → 只建分组
        if h_clean.contains("实验室") && !h_clean.contains("方法") {
            for r in rows.iter().skip(1) {
                if let Some(v) = r.get(col_idx) {
                    let val = v.as_string().unwrap_or_default().trim().to_string();
                    if !val.is_empty() && !group_items.contains(&val) {
                        group_items.push(val);
                    }
                }
            }
            continue;
        }

        // 研发项目 → projects 表
        if h_clean.contains("研发") {
            for r in rows.iter().skip(1) {
                if let Some(v) = r.get(col_idx) {
                    let val = v.as_string().unwrap_or_default().trim().to_string();
                    if !val.is_empty() && !project_items.contains(&val) {
                        project_items.push(val);
                    }
                }
            }
            continue;
        }

        // 方法列（含"方法"关键词）
        let method_type = classify_method_type(h_clean);
        for r in rows.iter().skip(1) {
            if let Some(v) = r.get(col_idx) {
                let val = v.as_string().unwrap_or_default().trim().to_string();
                if !val.is_empty() {
                    if !method_items.iter().any(|(g, n, _)| g == header && n == &val) {
                        method_items.push((header.clone(), val, method_type.clone()));
                    }
                }
            }
        }
    }

    if method_items.is_empty() && project_items.is_empty() && group_items.is_empty() {
        return Err(AppError::Validation("无有效数据".into()));
    }

    let conn = pool.get()?;
    let summary = method_repo::batch_import_column_split(
        &conn, &group_items, &project_items, &method_items
    )?;
    std::fs::remove_file(&tp).ok();
    Ok(Json(ApiResponse::ok(summary)))
}

/// 根据列头分类 method_type
fn classify_method_type(header: &str) -> String {
    let h = header.trim();
    if h.contains("液相") { "液相".into() }
    else if h.contains("气相") { "气相".into() }
    else if h.contains("理化") { "理化".into() }
    else if h.to_uppercase().contains("ICP") { "ICP".into() }
    else if h.contains("热分析") { "热分析".into() }
    else if h.contains("质谱") { "质谱".into() }
    else if h.contains("研发") { "研发项目".into() }
    else { "其他".into() }
}
