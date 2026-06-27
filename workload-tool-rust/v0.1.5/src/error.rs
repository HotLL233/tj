use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("连接池错误: {0}")]
    Pool(#[from] r2d2::Error),
    #[error("Excel错误: {0}")]
    Xlsx(#[from] rust_xlsxwriter::XlsxError),
    #[error("未找到: {0}")]
    NotFound(String),
    #[error("验证失败: {0}")]
    Validation(String),
    #[error("冲突: {0}")]
    Conflict(String),
    #[error("内部错误: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // 统一返回 HTTP 200 + 业务错误码，与 Python 版一致，避免 axios 将 4xx/5xx 当作网络异常
        let code = match &self {
            AppError::NotFound(_) => 1,
            AppError::Validation(_) | AppError::Conflict(_) => 1,
            _ => 500,
        };
        let message = match &self {
            AppError::NotFound(msg) => msg.clone(),
            AppError::Validation(msg) => msg.clone(),
            AppError::Conflict(msg) => msg.clone(),
            AppError::Database(e) => format!("数据库错误: {}", e),
            AppError::Pool(e) => format!("连接池错误: {}", e),
            AppError::Xlsx(e) => format!("Excel错误: {}", e),
            AppError::Internal(msg) => msg.clone(),
        };
        let body = json!({ "code": code, "message": message, "data": null });
        (StatusCode::OK, Json(body)).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
