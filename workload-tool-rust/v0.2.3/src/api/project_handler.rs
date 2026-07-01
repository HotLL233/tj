use axum::{extract::{Path, Query, State}, Json, Router, routing::get};
use serde::Deserialize;
use crate::db::DbPool;
use crate::error::Result;
use crate::models::ApiResponse;
use crate::models::project::{ProjectCreate, ProjectResponse, ProjectUpdate};
use crate::repo::project_repo;

#[derive(Deserialize)]
pub struct ProjectQuery {
    pub group_id: Option<i64>,
    pub active_only: Option<bool>,
}

#[derive(Deserialize)]
pub struct BatchCoefficientPayload {
    pub group_id: i64,
    pub coefficient: f64,
}

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/projects", get(list).post(create))
        .route("/api/projects/:id", axum::routing::put(update).delete(delete))
        .route("/api/projects/batch-coefficient", axum::routing::put(batch_coefficient))
        .with_state(pool)
}

async fn list(State(pool): State<DbPool>, Query(q): Query<ProjectQuery>) -> Result<Json<ApiResponse<Vec<ProjectResponse>>>> {
    let items = project_repo::list(&pool, q.group_id, q.active_only.unwrap_or(false))?;
    Ok(Json(ApiResponse::ok(items)))
}

async fn create(State(pool): State<DbPool>, Json(body): Json<ProjectCreate>) -> Result<Json<ApiResponse<ProjectResponse>>> {
    Ok(Json(ApiResponse::ok(project_repo::create(&pool, &body)?)))
}

async fn update(State(pool): State<DbPool>, Path(id): Path<i64>, Json(body): Json<ProjectUpdate>) -> Result<Json<ApiResponse<ProjectResponse>>> {
    Ok(Json(ApiResponse::ok(project_repo::update(&pool, id, &body)?)))
}

async fn delete(State(pool): State<DbPool>, Path(id): Path<i64>) -> Result<Json<ApiResponse<()>>> {
    project_repo::delete(&pool, id)?;
    Ok(Json(ApiResponse::ok_msg("删除成功")))
}

async fn batch_coefficient(State(pool): State<DbPool>, Json(body): Json<BatchCoefficientPayload>) -> Result<Json<ApiResponse<i64>>> {
    let count = project_repo::batch_coefficient(&pool, body.group_id, body.coefficient)?;
    Ok(Json(ApiResponse::ok_msg(format!("已批量更新 {} 个项目的系数为 {}", count, body.coefficient))))
}
