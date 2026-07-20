use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json, Router, routing::{get, put, delete as axum_delete},
};
use crate::db::DbPool;
use crate::error::Result;
use crate::models::sample_info_type::{
    SampleInfoType, SampleInfoTypeCreate, SampleInfoTypeUpdate,
};
use crate::models::ApiResponse;
use crate::repo::{sample_info_type_repo, sample_info_column_visibility_repo};
use crate::service::authz_service;

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/sample-info-types", get(list).post(create))
        .route("/api/sample-info-types/all", get(list_all))
        .route("/api/sample-info-types/:id", put(update).delete(soft_delete))
        .route("/api/sample-info-types/:id/permanent", axum_delete(permanent_delete))
        .with_state(pool)
}

/// 从 HeaderMap 中提取 JWT claims 并校验管理员权限
fn require_admin(pool: &DbPool, headers: &HeaderMap) -> Result<authz_service::AuthContext> {
    let ctx = authz_service::authenticate(pool, headers)?;
    authz_service::require_permission(&ctx, "manage:sampleinfo")?;
    Ok(ctx)
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
    require_admin(&pool, &headers)?;
    let item = sample_info_type_repo::create(&pool, &body)?;
    // v0.4.60: 初始化预置列可见性失败不应让整个 create 失败（类型已成功创建）
    // 改为可恢复的警告日志 + 后台诊断
    let conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(
                "新建检测类型「{}」后初始化可见性失败: 连接池获取失败: {} — 类型已创建但预置列可见性未初始化",
                &item.type_key, e
            );
            return Ok(Json(ApiResponse::ok(item)));
        }
    };
    if let Err(e) = sample_info_column_visibility_repo::init_for_type(&conn, &item.type_key) {
        tracing::error!(
            "新建检测类型「{}」后初始化可见性失败: {} — 类型已创建但预置列可见性未初始化",
            &item.type_key, e
        );
        // 不返回错误：list_active_by_type 已有 LEFT JOIN + IS NULL 兜底，新类型仍可正常显示
    }
    Ok(Json(ApiResponse::ok(item)))
}

async fn update(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(body): Json<SampleInfoTypeUpdate>,
) -> Result<Json<ApiResponse<SampleInfoType>>> {
    require_admin(&pool, &headers)?;
    let item = sample_info_type_repo::update(&pool, id, &body)?;
    Ok(Json(ApiResponse::ok(item)))
}

async fn soft_delete(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<()>>> {
    require_admin(&pool, &headers)?;
    sample_info_type_repo::soft_delete(&pool, id)?;
    Ok(Json(ApiResponse::ok_msg("已移入回收站")))
}

async fn permanent_delete(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<()>>> {
    require_admin(&pool, &headers)?;
    sample_info_type_repo::permanent_delete(&pool, id)?;
    Ok(Json(ApiResponse::ok_msg("彻底删除成功")))
}
