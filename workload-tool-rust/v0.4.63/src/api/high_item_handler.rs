use axum::{extract::{Path, State}, http::HeaderMap, Json, Router};
use crate::db::DbPool;
use crate::error::Result;
use crate::models::high_item::{HighItem, HighItemCreate, HighItemUpdate};
use crate::models::ApiResponse;
use crate::repo::high_item_repo;
use crate::service::auth_service;

fn require_admin(headers: &HeaderMap) -> Result<auth_service::Claims> {
    let token = headers.get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| crate::error::AppError::Validation("未提供登录凭证".into()))?;
    let claims = auth_service::verify_token(token)?;
    if !claims.is_admin { return Err(crate::error::AppError::Forbidden("需要管理员权限".into())); }
    Ok(claims)
}

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/high-items", axum::routing::get(list).post(create))
        .route("/api/high-items/all", axum::routing::get(list_all))
        .route("/api/high-items/:id", axum::routing::put(update).delete(soft_delete))
        .route("/api/high-items/:id/permanent", axum::routing::delete(permanent_delete))
        .with_state(pool)
}

async fn list(State(pool): State<DbPool>) -> Result<Json<ApiResponse<Vec<HighItem>>>> {
    Ok(Json(ApiResponse::ok(high_item_repo::list(&pool)?)))
}

async fn list_all(State(pool): State<DbPool>) -> Result<Json<ApiResponse<Vec<HighItem>>>> {
    Ok(Json(ApiResponse::ok(high_item_repo::list_all(&pool)?)))
}

async fn create(State(pool): State<DbPool>, headers: HeaderMap, Json(body): Json<HighItemCreate>) -> Result<Json<ApiResponse<HighItem>>> {
    require_admin(&headers)?;
    let item = high_item_repo::create(&pool, &body)?;
    Ok(Json(ApiResponse::ok(item)))
}

async fn update(State(pool): State<DbPool>, headers: HeaderMap, Path(id): Path<i64>, Json(body): Json<HighItemUpdate>) -> Result<Json<ApiResponse<HighItem>>> {
    require_admin(&headers)?;
    let item = high_item_repo::update(&pool, id, &body)?;
    Ok(Json(ApiResponse::ok(item)))
}

async fn soft_delete(State(pool): State<DbPool>, headers: HeaderMap, Path(id): Path<i64>) -> Result<Json<ApiResponse<()>>> {
    require_admin(&headers)?;
    high_item_repo::soft_delete(&pool, id)?;
    Ok(Json(ApiResponse::ok_msg("已移入回收站")))
}

async fn permanent_delete(State(pool): State<DbPool>, headers: HeaderMap, Path(id): Path<i64>) -> Result<Json<ApiResponse<()>>> {
    require_admin(&headers)?;
    high_item_repo::permanent_delete(&pool, id)?;
    Ok(Json(ApiResponse::ok_msg("彻底删除成功")))
}
