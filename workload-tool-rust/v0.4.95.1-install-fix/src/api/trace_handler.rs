use axum::{extract::{Path, State}, Json, Router};
use crate::db::DbPool;
use crate::error::{AppError, Result};
use crate::models::{ApiResponse, trace::RecordEvent};
use crate::repo::trace_repo;

fn table_for_module(module: &str) -> Result<&'static str> {
    match module {
        "work" => Ok("work_records"),
        "rd" => Ok("rd_work_records"),
        "sample-info" => Ok("sample_info_records"),
        _ => Err(AppError::Validation("无效的溯源模块".into())),
    }
}

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/trace/:module/:id", axum::routing::get(list))
        .with_state(pool)
}

async fn list(
    State(pool): State<DbPool>,
    Path((module, id)): Path<(String, i64)>,
) -> Result<Json<ApiResponse<Vec<RecordEvent>>>> {
    Ok(Json(ApiResponse::ok(trace_repo::list(&pool, table_for_module(&module)?, id)?)))
}
