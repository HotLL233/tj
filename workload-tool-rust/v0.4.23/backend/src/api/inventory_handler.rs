//! 库存管理 API：分类 / 物料 / 批次 / 流水（出库走审批）。

use axum::{extract::{Path, Query, State, Json}, Router, routing::{get, post, put, delete}};
use serde::Deserialize;

use crate::db::DbPool;
use crate::error::Result;
use crate::models::ApiResponse;
use crate::models::inventory::{BatchCreate, CategoryCreate, CategoryUpdate, ItemCreate, ItemUpdate, TransactionCreate};
use crate::middleware::auth::AuthedUser;
use crate::repo;
use crate::service::inventory_service;

pub fn router(pool: DbPool) -> Router {
    Router::new()
        .route("/api/inventory/categories", get(list_categories).post(create_category))
        .route("/api/inventory/categories/:id", put(update_category).delete(delete_category))
        .route("/api/inventory/items", get(list_items).post(create_item))
        .route("/api/inventory/items/:id", get(get_item).put(update_item).delete(delete_item))
        .route("/api/inventory/items/:id/batches", get(list_batches).post(create_batch))
        .route("/api/inventory/transactions", get(list_transactions).post(create_transaction))
        .with_state(pool)
}

#[derive(Deserialize)]
struct ItemQuery { category_id: Option<i64>, low_stock: Option<bool> }
#[derive(Deserialize)]
struct TxQuery { item_id: Option<i64>, page: Option<i64>, page_size: Option<i64> }

async fn list_categories(State(pool): State<DbPool>, user: AuthedUser) -> Result<Json<ApiResponse<Vec<crate::models::inventory::InventoryCategory>>>> {
    user.require("inventory:read")?;
    Ok(Json(ApiResponse::ok(repo::inventory_repo::list_categories(&pool)?)))
}

async fn create_category(State(pool): State<DbPool>, user: AuthedUser, Json(body): Json<CategoryCreate>) -> Result<Json<ApiResponse<crate::models::inventory::InventoryCategory>>> {
    user.require("inventory:write")?;
    Ok(Json(ApiResponse::ok(inventory_service::create_category(&pool, &body, &user.username)?)))
}

async fn update_category(State(pool): State<DbPool>, user: AuthedUser, Path(id): Path<i64>, Json(body): Json<CategoryUpdate>) -> Result<Json<ApiResponse<crate::models::inventory::InventoryCategory>>> {
    user.require("inventory:write")?;
    Ok(Json(ApiResponse::ok(inventory_service::update_category(&pool, id, &body, &user.username)?)))
}

async fn delete_category(State(pool): State<DbPool>, user: AuthedUser, Path(id): Path<i64>) -> Result<Json<ApiResponse<()>>> {
    user.require("inventory:write")?;
    inventory_service::delete_category(&pool, id, &user.username)?;
    Ok(Json(ApiResponse::ok_msg("分类已删除")))
}

async fn list_items(State(pool): State<DbPool>, user: AuthedUser, Query(q): Query<ItemQuery>) -> Result<Json<ApiResponse<Vec<crate::models::inventory::ItemResponse>>>> {
    user.require("inventory:read")?;
    Ok(Json(ApiResponse::ok(repo::inventory_repo::list_items(&pool, q.category_id, q.low_stock.unwrap_or(false))?)))
}

async fn get_item(State(pool): State<DbPool>, user: AuthedUser, Path(id): Path<i64>) -> Result<Json<ApiResponse<crate::models::inventory::ItemResponse>>> {
    user.require("inventory:read")?;
    Ok(Json(ApiResponse::ok(repo::inventory_repo::get_item(&pool, id)?)))
}

async fn create_item(State(pool): State<DbPool>, user: AuthedUser, Json(body): Json<ItemCreate>) -> Result<Json<ApiResponse<crate::models::inventory::ItemResponse>>> {
    user.require("inventory:write")?;
    Ok(Json(ApiResponse::ok(inventory_service::create_item(&pool, &body, &user.username, &user.username)?)))
}

async fn update_item(State(pool): State<DbPool>, user: AuthedUser, Path(id): Path<i64>, Json(body): Json<ItemUpdate>) -> Result<Json<ApiResponse<crate::models::inventory::ItemResponse>>> {
    user.require("inventory:write")?;
    Ok(Json(ApiResponse::ok(inventory_service::update_item(&pool, id, &body, &user.username)?)))
}

async fn delete_item(State(pool): State<DbPool>, user: AuthedUser, Path(id): Path<i64>) -> Result<Json<ApiResponse<()>>> {
    user.require("inventory:write")?;
    inventory_service::delete_item(&pool, id, &user.username)?;
    Ok(Json(ApiResponse::ok_msg("物料已删除")))
}

async fn list_batches(State(pool): State<DbPool>, user: AuthedUser, Path(id): Path<i64>) -> Result<Json<ApiResponse<Vec<crate::models::inventory::InventoryBatch>>>> {
    user.require("inventory:read")?;
    Ok(Json(ApiResponse::ok(repo::inventory_repo::list_batches(&pool, id)?)))
}

async fn create_batch(State(pool): State<DbPool>, user: AuthedUser, Path(id): Path<i64>, Json(mut body): Json<BatchCreate>) -> Result<Json<ApiResponse<crate::models::inventory::InventoryBatch>>> {
    user.require("inventory:write")?;
    body.item_id = id;
    Ok(Json(ApiResponse::ok(inventory_service::create_batch(&pool, &body, &user.username)?)))
}

async fn list_transactions(State(pool): State<DbPool>, user: AuthedUser, Query(q): Query<TxQuery>) -> Result<Json<ApiResponse<crate::models::PaginatedResponse<crate::models::inventory::TransactionResponse>>>> {
    user.require("inventory:read")?;
    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(50).min(500);
    let (items, total) = inventory_service::list_transactions(&pool, q.item_id, page, page_size)?;
    Ok(Json(ApiResponse::ok(crate::models::PaginatedResponse { items, total, page, page_size })))
}

async fn create_transaction(State(pool): State<DbPool>, user: AuthedUser, Json(body): Json<TransactionCreate>) -> Result<Json<ApiResponse<()>>> {
    user.require("inventory:write")?;
    match body.tx_type.as_str() {
        "out" | "scrap" => {
            inventory_service::create_out(&pool, body.item_id, &body.tx_type, body.quantity, &user.username, &user.username, &user.role, &user.username)?;
            Ok(Json(ApiResponse::ok_msg("出库申请已提交，等待审批")))
        }
        "in" => {
            let batch = BatchCreate {
                item_id: body.item_id, batch_no: String::new(), quantity: body.quantity, unit_price: 0.0,
                produced_at: None, expiry_date: None, source_type: "manual".into(), source_id: body.related_id,
            };
            inventory_service::create_batch(&pool, &batch, &user.username)?;
            Ok(Json(ApiResponse::ok_msg("入库成功")))
        }
        _ => Err(crate::error::AppError::Validation("不支持的流水类型".into())),
    }
}
