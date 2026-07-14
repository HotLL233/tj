use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json, Router, routing::{get, put, delete as axum_delete},
};
use crate::db::DbPool;
use crate::error::{AppError, Result};
use crate::models::sample_info_type::{
    SampleInfoType, SampleInfoTypeCreate, SampleInfoTypeUpdate,
};
use crate::models::ApiResponse;
use crate::repo::{sample_info_type_repo, sample_info_column_visibility_repo};
use crate::service::auth_service;

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/sample-info-types", get(list).post(create))
        .route("/api/sample-info-types/all", get(list_all))
        .route("/api/sample-info-types/:id", put(update).delete(soft_delete))
        .route("/api/sample-info-types/:id/permanent", axum_delete(permanent_delete))
        .with_state(pool)
}

/// 从 HeaderMap 中提取 JWT claims 并校验管理员权限
fn require_admin(headers: &HeaderMap) -> Result<auth_service::Claims> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::Validation("未提供登录凭证".into()))?;
    let claims = auth_service::verify_token(token)?;
    if !claims.is_admin {
        return Err(AppError::Forbidden("需要管理员权限".into()));
    }
    Ok(claims)
}

async fn list(State(pool): State<DbPool>) -> Result<Json<ApiResponse<Vec<SampleInfoType>>>> {
    let items = sample_info_type_repo::list(&pool)?;
    Ok(Json(ApiResponse::ok(items)))
}

async fn list_all(State(pool): State<DbPool>) -> Result<Json<ApiResponse<Vec<SampleInfoType>>>> {
    let items = sample_info_type_repo::list_all(&pool)?;
    Ok(Json(ApiResponse::ok(items)))
}

async fn create(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Json(body): Json<SampleInfoTypeCreate>,
) -> Result<Json<ApiResponse<SampleInfoType>>> {
    require_admin(&headers)?;
    let item = sample_info_type_repo::create(&pool, &body)?;
    // 自动为新类型初始化所有预置列的可见性
    let conn = pool.get()?;
    sample_info_column_visibility_repo::init_for_type(&conn, &item.type_key)?;
    Ok(Json(ApiResponse::ok(item)))
}

async fn update(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(body): Json<SampleInfoTypeUpdate>,
) -> Result<Json<ApiResponse<SampleInfoType>>> {
    require_admin(&headers)?;
    let item = sample_info_type_repo::update(&pool, id, &body)?;
    Ok(Json(ApiResponse::ok(item)))
}

async fn soft_delete(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<()>>> {
    require_admin(&headers)?;
    sample_info_type_repo::soft_delete(&pool, id)?;
    Ok(Json(ApiResponse::ok_msg("已移入回收站")))
}

async fn permanent_delete(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<()>>> {
    require_admin(&headers)?;
    sample_info_type_repo::permanent_delete(&pool, id)?;
    Ok(Json(ApiResponse::ok_msg("彻底删除成功")))
}
