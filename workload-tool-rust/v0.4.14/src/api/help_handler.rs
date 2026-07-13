use axum::{
    body::Body,
    extract::{DefaultBodyLimit, Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json, Router,
};
use serde::Deserialize;
use crate::db::DbPool;
use crate::error::{AppError, Result};
use crate::models::ApiResponse;
use crate::models::help::{HelpDocUpdateRequest, HelpDocument};
use crate::repo::help_repo;

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/help-documents", axum::routing::get(list).post(upload))
        .route(
            "/api/help-documents/:id",
            axum::routing::put(update).delete(delete),
        )
        .route("/api/help-documents/:id/file", axum::routing::get(get_file))
        .with_state(pool)
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024))
}

#[derive(Deserialize)]
struct HelpDocQuery {
    visible_only: Option<bool>,
}

/// GET /api/help-documents?visible_only=true
async fn list(
    State(pool): State<DbPool>,
    Query(q): Query<HelpDocQuery>,
) -> Result<Json<ApiResponse<Vec<HelpDocument>>>> {
    let visible_only = q.visible_only.unwrap_or(false);
    let items = help_repo::list(&pool, visible_only)?;
    Ok(Json(ApiResponse::ok(items)))
}

/// POST /api/help-documents — multipart upload (file + title)
async fn upload(
    State(pool): State<DbPool>,
    mut mp: Multipart,
) -> Result<Json<ApiResponse<HelpDocument>>> {
    let mut title = String::new();
    let mut file_data: Vec<u8> = Vec::new();
    let mut original_filename = String::new();

    while let Ok(Some(field)) = mp.next_field().await
    {
        match field.name() {
            Some("title") => {
                title = field
                    .text()
                    .await
                    .map_err(|e| AppError::Validation(format!("读取 title 失败: {}", e)))?;
            }
            Some("file") => {
                original_filename = field
                    .file_name()
                    .unwrap_or("unknown")
                    .to_string();
                file_data = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::Validation(format!("读取文件失败: {}", e)))?
                    .to_vec();
            }
            _ => {}
        }
    }

    if file_data.is_empty() {
        return Err(AppError::Validation("未收到文件".into()));
    }
    if title.trim().is_empty() {
        title = original_filename.clone();
    }

    // 确定文件扩展名
    let ext = std::path::Path::new(&original_filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin")
        .to_string();

    // 生成唯一存储名
    let stored_name = format!("{}_{}", uuid::Uuid::new_v4(), original_filename);

    // 确定存储目录（相对于 exe 所在目录）
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let help_dir = exe_dir.join("data").join("help_docs");
    std::fs::create_dir_all(&help_dir)
        .map_err(|e| AppError::Internal(format!("创建目录失败: {}", e)))?;

    let file_path = help_dir.join(&stored_name);
    std::fs::write(&file_path, &file_data)
        .map_err(|e| AppError::Internal(format!("写入文件失败: {}", e)))?;

    let file_size = file_data.len() as i64;

    // 存储相对路径
    let relative_path = format!("data/help_docs/{}", stored_name);

    let doc = help_repo::create(&pool, &title, &original_filename, &relative_path, &ext, file_size)?;
    Ok(Json(ApiResponse::ok(doc)))
}

/// PUT /api/help-documents/:id — 编辑标题/显隐/排序
async fn update(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
    Json(body): Json<HelpDocUpdateRequest>,
) -> Result<Json<ApiResponse<HelpDocument>>> {
    let item = help_repo::update(&pool, id, &body)?;
    Ok(Json(ApiResponse::ok(item)))
}

/// DELETE /api/help-documents/:id — 删除 DB 记录 + 磁盘文件
async fn delete(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<()>>> {
    let doc = help_repo::delete(&pool, id)?;

    // 删除磁盘文件
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let file_abs = exe_dir.join(&doc.file_path);
    std::fs::remove_file(&file_abs).ok();

    Ok(Json(ApiResponse::ok_msg("删除成功")))
}

/// GET /api/help-documents/:id/file — 下载/查看文件
async fn get_file(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse> {
    let doc = help_repo::get_by_id(&pool, id)?;

    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let file_abs = exe_dir.join(&doc.file_path);

    let data = tokio::fs::read(&file_abs)
        .await
        .map_err(|_| AppError::NotFound("文件不存在或已被删除".into()))?;

    let content_type = mime_type(&doc.file_type);

    let mut response = Response::new(Body::from(data));
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        content_type.parse().unwrap(),
    );
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        format!("inline; filename=\"{}\"", doc.filename)
            .parse()
            .unwrap(),
    );

    Ok(response)
}

/// 简单扩展名 → MIME 映射
fn mime_type(ext: &str) -> &'static str {
    match ext.to_lowercase().as_str() {
        "pdf" => "application/pdf",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "ppt" => "application/vnd.ms-powerpoint",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "txt" => "text/plain; charset=utf-8",
        "html" | "htm" => "text/html; charset=utf-8",
        "csv" => "text/csv; charset=utf-8",
        "zip" => "application/zip",
        "rar" => "application/vnd.rar",
        "7z" => "application/x-7z-compressed",
        _ => "application/octet-stream",
    }
}
