use axum::{extract::{Path, Query, State}, http::HeaderMap, Json, Router};
use serde::Deserialize;
use crate::db::DbPool;
use crate::error::Result;
use crate::models::ApiResponse;
use crate::models::project::*;
use crate::repo::project_repo;

fn current_username(headers: &HeaderMap) -> String {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .and_then(|token| crate::service::auth_service::verify_token(token).ok())
        .map(|claims| claims.username)
        .unwrap_or_else(|| "system".to_string())
}

#[derive(Deserialize)]
pub struct ProjectQuery {
    pub group_id: Option<i64>,
    pub active_only: Option<bool>,
    pub method_type: Option<String>,
    pub status: Option<String>,
}

#[derive(Deserialize)]
pub struct BatchCoefficientPayload {
    pub group_id: i64,
    pub coefficient: f64,
}

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/projects", axum::routing::get(list).post(create))
        .route("/api/projects/:id", axum::routing::put(update).delete(delete))
        .route("/api/projects/batch-coefficient", axum::routing::put(batch_coefficient))
        .with_state(pool)
}

async fn list(State(pool): State<DbPool>, Query(q): Query<ProjectQuery>) -> Result<Json<ApiResponse<Vec<ProjectResponse>>>> {
    let items = project_repo::list(&pool, q.group_id, q.active_only.unwrap_or(false), q.method_type.as_deref(), q.status.as_deref())?;
    Ok(Json(ApiResponse::ok(items)))
}

async fn create(State(pool): State<DbPool>, headers: HeaderMap, Json(b): Json<ProjectCreate>) -> Result<Json<ApiResponse<ProjectResponse>>> {
    Ok(Json(ApiResponse::ok(project_repo::create(&pool, &b, &current_username(&headers))?)))
}

async fn update(State(pool): State<DbPool>, headers: HeaderMap, Path(id): Path<i64>, Json(b): Json<ProjectUpdate>) -> Result<Json<ApiResponse<ProjectResponse>>> {
    Ok(Json(ApiResponse::ok(project_repo::update(&pool, id, &b, &current_username(&headers))?)))
}

async fn delete(State(pool): State<DbPool>, Path(id): Path<i64>) -> Result<Json<ApiResponse<()>>> {
    project_repo::delete(&pool, id)?;
    Ok(Json(ApiResponse::ok_msg("删除成功")))
}

async fn batch_coefficient(State(pool): State<DbPool>, Json(b): Json<BatchCoefficientPayload>) -> Result<Json<ApiResponse<i64>>> {
    let c = project_repo::batch_coefficient(&pool, b.group_id, b.coefficient)?;
    Ok(Json(ApiResponse::ok_msg(format!("已更新{}个项目系数", c))))
}
