use axum::{extract::{Multipart, State}, Router, routing::post};
use axum::response::IntoResponse;
use crate::db::DbPool;
use crate::error::{Result, AppError};
use crate::models::{ApiResponse, import_record::{ImportRecord, ImportResult}};
use crate::db::import;
use serde::Deserialize;
use std::process::Command;

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/import/excel", post(import_excel))
        .with_state(pool)
}

// ===================== CLI JSON 协议（与 excel-parser.exe 一致） =====================

#[derive(Deserialize)]
struct CliOutput {
    success: bool,
    sheets: Vec<CliSheet>,
    error: String,
}

#[derive(Deserialize)]
struct CliSheet {
    name: String,
    rows_read: usize,
    rows_parsed: usize,
    rows_skipped: usize,
    records: Vec<CliRecord>,
}

#[derive(Deserialize)]
struct CliRecord {
    date: String,
    batch_no: String,
}

// ===================== 定位 excel-parser.exe =====================

fn find_parser() -> String {
    // 1. 与主程序同目录 (安装部署路径)
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_default();
    let bundled = exe_dir.join("excel-parser.exe");
    if bundled.exists() {
        return bundled.to_string_lossy().to_string();
    }
    // 2. 开发环境：相对于项目根目录
    let dev_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()
        .join("excel-parser").join("target").join("release").join("excel-parser.exe");
    if dev_path.exists() {
        return dev_path.to_string_lossy().to_string();
    }
    // 3. 回退到 PATH
    "excel-parser.exe".to_string()
}

// ===================== 主处理函数 =====================

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

    // ===== 调用 excel-parser CLI =====
    let parser_exe = find_parser();
    tracing::info!("Calling excel-parser: {} {}", parser_exe, &path);

    let output = Command::new(&parser_exe)
        .arg(&path)
        .output()
        .map_err(|e| AppError::Internal(format!("无法启动 Excel 解析器 ({}): {}", parser_exe, e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        // 尝试从 stdout 解析错误信息
        if let Ok(cli) = serde_json::from_str::<CliOutput>(&stdout) {
            std::fs::remove_file(&path).ok();
            return Err(AppError::Validation(format!("Excel 解析失败: {}", cli.error)));
        }
        std::fs::remove_file(&path).ok();
        return Err(AppError::Internal(format!("Excel 解析器异常退出: {}", stderr)));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let cli: CliOutput = serde_json::from_str(&stdout)
        .map_err(|e| AppError::Internal(format!("解析器返回格式异常: {}. 原始输出: {}", e, &stdout[..stdout.len().min(200)])))?;

    // Clean up temp file ASAP
    std::fs::remove_file(&path).ok();

    if !cli.success {
        return Err(AppError::Validation(format!("Excel 解析失败: {}", cli.error)));
    }

    // ===== 映射 CLI 结果 → ImportRecord =====
    // rust-tool 的数据模型: 每个 Sheet → 实验室, 每条 (date, batch_no) → 1条记录
    let mut records: Vec<ImportRecord> = vec![];
    let mut sheets_info: Vec<String> = vec![];
    let mut total_read = 0usize;
    let mut total_skipped = 0usize;

    for sheet in &cli.sheets {
        sheets_info.push(format!("{}:解析{}行/跳过{}行", sheet.name, sheet.rows_parsed, sheet.rows_skipped));
        total_read += sheet.rows_read;
        total_skipped += sheet.rows_skipped;

        for rec in &sheet.records {
            records.push(ImportRecord {
                project_name: "通用项目".to_string(),
                group_name: sheet.name.clone(),
                recorded_at: rec.date.clone(),
                batch_no: rec.batch_no.clone(),
                quantity: 1,
                user_name: None,
                extra_info: None,
            });
        }
    }

    if records.is_empty() {
        return Err(AppError::Validation("Excel 中无有效数据".into()));
    }

    // ===== 写入数据库 =====
    let conn = pool.get()?;
    let (inserted, updated, db_errors) = import::batch_import(&conn, &records)?;

    // ===== 构建结果 =====
    let mut warnings: Vec<String> = vec![];
    if total_skipped > 0 {
        warnings.push(format!("共读取{}行，跳过{}行无效数据（日期/批号为空或示例行）", total_read, total_skipped));
    }

    let result = ImportResult {
        success: true,
        total_rows_read: total_read,
        inserted,
        updated,
        skipped: total_skipped,
        sheet_name: cli.sheets.iter().map(|s| s.name.clone()).collect::<Vec<_>>().join(", "),
        columns_found: sheets_info,
        errors: db_errors,
        warnings,
    };

    tracing::info!("Import completed: {} sheets, {} inserted, {} updated, {} skipped",
        cli.sheets.len(), result.inserted, result.updated, result.skipped);

    Ok(axum::Json(ApiResponse {
        code: 0,
        message: result.summary(),
        data: Some(result),
    }))
}
