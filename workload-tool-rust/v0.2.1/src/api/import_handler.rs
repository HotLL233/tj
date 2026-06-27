use axum::{extract::{Multipart, State}, Router, routing::post};
use axum::response::IntoResponse;
use crate::db::DbPool;
use crate::error::{Result, AppError};
use crate::models::{ApiResponse, import_record::ImportResult};
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
            let tmp = std::env::temp_dir().join(format!("import_{}.xlsx", uuid::Uuid::new_v4()));
            std::fs::write(&tmp, &data).map_err(|e| AppError::Internal(format!("临时文件写入失败: {}", e)))?;
            file_path = Some(tmp.to_str().unwrap().to_string());
        }
    }

    let path = file_path.ok_or(AppError::Validation("未收到文件，请选择 .xlsx 文件上传".into()))?;

    // Parse Excel
    let parse_result = excel_reader::read_excel(&path)?;

    // Clean up temp file
    std::fs::remove_file(&path).ok();

    // Import to DB
    let conn = pool.get()?;
    let (inserted, updated, db_errors) = import::batch_import(&conn, &parse_result.records)?;

    // Build structured result
    let mut warnings: Vec<String> = vec![];
    if parse_result.skipped_rows > 0 {
        warnings.push(format!("Excel中共读取{}行，跳过{}行无效数据（日期/批号为空或数量≤0）", 
            parse_result.total_rows_read, parse_result.skipped_rows));
    }

    let result = ImportResult {
        success: true,
        total_rows_read: parse_result.total_rows_read,
        inserted,
        updated,
        skipped: parse_result.skipped_rows,
        sheet_name: parse_result.sheet_name,
        columns_found: parse_result.columns_found,
        errors: db_errors,
        warnings,
    };

    tracing::info!("Import completed: {} inserted, {} updated, {} skipped", 
        result.inserted, result.updated, result.skipped);

    Ok(axum::Json(ApiResponse {
        code: 0,
        message: result.summary(),
        data: Some(result),
    }))
}
