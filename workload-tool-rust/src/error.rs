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
        let (status, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };
        let body = json!({ "code": status.as_u16(), "message": message, "data": null });
        (status, Json(body)).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
