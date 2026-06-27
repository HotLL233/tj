use axum::{extract::{Multipart, State}, Router, routing::post};
use axum::response::IntoResponse;
use crate::db::DbPool;
use crate::error::{Result, AppError};
use crate::models::ApiResponse;
use crate::db::import;
use crate::utils::excel_reader;

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/import/excel", post(import_excel))
        .with_state(pool)
}

async fn import_excel(State(pool): State<DbPool>, mut multipart: Multipart) -> Result<impl IntoResponse> {
    let mut file_path: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::Validation(format!("上传错误: {}", e)))? {
        if field.name() == Some("file") {
            let data = field.bytes().await.map_err(|e| AppError::Validation(format!("读取文件失败: {}", e)))?;
            if data.is_empty() {
                return Err(AppError::Validation("上传文件为空".into()));
            }
            // Write to temp file
            let tmp = std::env::temp_dir().join(format!("import_{}.xlsx", uuid::Uuid::new_v4()));
            std::fs::write(&tmp, &data).map_err(|e| AppError::Internal(format!("临时文件写入失败: {}", e)))?;
            file_path = Some(tmp.to_str().unwrap().to_string());
        }
    }

    let path = file_path.ok_or(AppError::Validation("未收到文件，请选择 .xlsx 文件上传".into()))?;

    // Parse Excel
    let records = excel_reader::read_excel(&path)?;

    // Import to DB
    let conn = pool.get()?;
    let count = import::batch_import(&conn, &records)?;

    // Clean up temp file
    std::fs::remove_file(&path).ok();

    Ok(axum::Json(ApiResponse::<serde_json::Value>::ok_msg(format!("成功导入 {} 条记录", count))))
}
