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

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/projects", get(list).post(create))
        .route("/api/projects/:id", axum::routing::put(update).delete(delete))
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
