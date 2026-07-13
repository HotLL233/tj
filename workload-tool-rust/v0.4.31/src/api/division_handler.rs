use axum::{extract::{Path, State}, Json, Router, routing::get};
use serde::Deserialize;
use crate::db::DbPool;
use crate::error::Result;
use crate::models::ApiResponse;
use crate::models::division::{Division, DivisionCreate, DivisionResponse, DivisionUpdate};
use crate::repo::division_repo;
use crate::repo::audit_repo;

#[derive(Deserialize)]
pub struct SetLabsRequest {
    pub group_ids: Vec<i64>,
}

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/divisions", get(list).post(create))
        .route("/api/divisions/:id", axum::routing::put(update).delete(delete))
        .route("/api/divisions/:id/labs", axum::routing::put(set_labs))
        .with_state(pool)
}

async fn list(State(pool): State<DbPool>) -> Result<Json<ApiResponse<Vec<DivisionResponse>>>> {
    let items = division_repo::list(&pool)?;
    Ok(Json(ApiResponse::ok(items)))
}

async fn create(State(pool): State<DbPool>, Json(body): Json<DivisionCreate>) -> Result<Json<ApiResponse<Division>>> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    let item = division_repo::create_on_conn(&tx, &body)?;
    audit_repo::log_on_conn_with_module(&tx, "create", "divisions", Some(item.id), "system", &format!("创建事业部「{}」", item.name), "division")?;
    tx.commit()?;
    Ok(Json(ApiResponse::ok(item)))
}

async fn update(State(pool): State<DbPool>, Path(id): Path<i64>, Json(body): Json<DivisionUpdate>) -> Result<Json<ApiResponse<Division>>> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    let item = division_repo::update_on_conn(&tx, id, &body)?;
    let detail = body.name.clone().map_or_else(|| "更新事业部".to_string(), |n| format!("更新事业部「{}」", n));
    audit_repo::log_on_conn_with_module(&tx, "update", "divisions", Some(id), "system", &detail, "division")?;
    tx.commit()?;
    Ok(Json(ApiResponse::ok(item)))
}

async fn delete(State(pool): State<DbPool>, Path(id): Path<i64>) -> Result<Json<ApiResponse<()>>> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    // 级联置空：先解除下属实验室关联（保留实验室），再删除事业部
    let affected = tx.execute("UPDATE project_groups SET division_id = NULL WHERE division_id = ?1", [id])?;
    tx.execute("DELETE FROM divisions WHERE id = ?1", [id])?;
    audit_repo::log_on_conn_with_module(&tx, "delete", "divisions", Some(id), "system", &format!("删除事业部（解除 {} 个实验室关联）", affected), "division")?;
    tx.commit()?;
    Ok(Json(ApiResponse::ok_msg("删除成功")))
}

/// PUT /api/divisions/:id/labs — 批量设置事业部关联的实验室
async fn set_labs(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
    Json(body): Json<SetLabsRequest>,
) -> Result<Json<ApiResponse<()>>> {
    division_repo::set_division_labs(&pool, id, &body.group_ids, "system")?;
    Ok(Json(ApiResponse::ok_msg("实验室关联更新成功")))
}
