//! 采购管理 API：采购申请 / 采购单 / 到货登记。

use axum::{extract::{Path, Query, State, Json}, Router, routing::{get, post}};

use crate::db::DbPool;
use crate::error::Result;
use crate::models::ApiResponse;
use crate::models::purchase::{OrderCreate, RequisitionCreate};
use crate::middleware::auth::AuthedUser;
use crate::repo;
use crate::service::purchase_service;

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/purchase/requisitions", get(list_requisitions).post(submit_requisition))
        .route("/api/purchase/orders", get(list_orders).post(create_order))
        .route("/api/purchase/orders/:id/receive", post(receive_order))
        .with_state(pool)
}

#[derive(serde::Deserialize)]
struct ReqQuery { status: Option<String>, applicant: Option<String> }
#[derive(serde::Deserialize)]
struct OrderQuery { status: Option<String> }

async fn list_requisitions(State(pool): State<DbPool>, user: AuthedUser, Query(q): Query<ReqQuery>) -> Result<Json<ApiResponse<Vec<crate::models::purchase::PurchaseRequisition>>>> {
    user.require("purchase:read")?;
    Ok(Json(ApiResponse::ok(repo::purchase_repo::list_requisitions(&pool, q.status.as_deref(), q.applicant.as_deref())?)))
}

async fn submit_requisition(State(pool): State<DbPool>, user: AuthedUser, Json(body): Json<RequisitionCreate>) -> Result<Json<ApiResponse<()>>> {
    user.require("purchase:request")?;
    purchase_service::submit_requisition(&pool, &body, &user.username, &user.role, &user.username, &user.username)?;
    Ok(Json(ApiResponse::ok_msg("采购申请已提交，等待审批")))
}

async fn list_orders(State(pool): State<DbPool>, user: AuthedUser, Query(q): Query<OrderQuery>) -> Result<Json<ApiResponse<Vec<crate::models::purchase::OrderResponse>>>> {
    user.require("purchase:read")?;
    Ok(Json(ApiResponse::ok(repo::purchase_repo::list_orders(&pool, q.status.as_deref())?)))
}

async fn create_order(State(pool): State<DbPool>, user: AuthedUser, Json(body): Json<OrderCreate>) -> Result<Json<ApiResponse<()>>> {
    user.require("purchase:write")?;
    purchase_service::create_order(&pool, &body, &user.username, &user.role, &user.username)?;
    Ok(Json(ApiResponse::ok_msg("采购单已创建，等待审批")))
}

async fn receive_order(State(pool): State<DbPool>, user: AuthedUser, Path(id): Path<i64>) -> Result<Json<ApiResponse<()>>> {
    user.require("purchase:approve")?;
    purchase_service::receive_order(&pool, id, &user.username)?;
    Ok(Json(ApiResponse::ok_msg("已登记到货")))
}
