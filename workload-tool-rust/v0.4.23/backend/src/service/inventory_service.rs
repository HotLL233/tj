//! 库存业务服务：分类 / 物料 / 批次 / 出库审批。

use crate::db::DbPool;
use crate::error::Result;
use crate::models::inventory::{BatchCreate, CategoryCreate, CategoryUpdate, ItemCreate, ItemResponse, ItemUpdate, TransactionResponse};
use crate::repo;
use crate::service::approval_service;

pub fn create_category(pool: &DbPool, body: &CategoryCreate, actor: &str) -> Result<crate::models::inventory::InventoryCategory> {
    repo::inventory_repo::create_category(pool, body, actor)
}
pub fn update_category(pool: &DbPool, id: i64, body: &CategoryUpdate, actor: &str) -> Result<crate::models::inventory::InventoryCategory> {
    repo::inventory_repo::update_category(pool, id, body, actor)
}
pub fn delete_category(pool: &DbPool, id: i64, actor: &str) -> Result<()> {
    repo::inventory_repo::delete_category(pool, id, actor)
}

pub fn create_item(pool: &DbPool, body: &ItemCreate, created_by: &str, actor: &str) -> Result<ItemResponse> {
    repo::inventory_repo::create_item(pool, body, created_by, actor)
}
pub fn update_item(pool: &DbPool, id: i64, body: &ItemUpdate, actor: &str) -> Result<ItemResponse> {
    repo::inventory_repo::update_item(pool, id, body, actor)
}
pub fn delete_item(pool: &DbPool, id: i64, actor: &str) -> Result<()> {
    repo::inventory_repo::soft_delete_item(pool, id, actor)
}

pub fn create_batch(pool: &DbPool, body: &BatchCreate, actor: &str) -> Result<crate::models::inventory::InventoryBatch> {
    repo::inventory_repo::create_batch(pool, body, actor)
}

/// 出库 / 报废：落库后进入审批流；无匹配规则则立即扣减库存。
pub fn create_out(
    pool: &DbPool,
    item_id: i64,
    tx_type: &str,
    quantity: f64,
    applicant: &str,
    created_by: &str,
    applicant_role: &str,
    actor: &str,
) -> Result<i64> {
    let tx_id = repo::inventory_repo::create_out_transaction(pool, item_id, tx_type, quantity, applicant, created_by, None)?;
    let item = repo::inventory_repo::get_item(pool, item_id).ok();
    let title = format!("出库申请：{} ×{}", item.map(|i| i.name).unwrap_or_default(), quantity);
    let outcome = approval_service::submit_for_approval(
        pool, "inventory_out", tx_id, &title, applicant, applicant_role, None, None,
    )?;
    if outcome.auto_approved {
        repo::inventory_repo::apply_out(pool, tx_id, "system", "system")?;
    } else if let Some(task_id) = outcome.task_id {
        repo::inventory_repo::link_transaction_task(pool, tx_id, task_id)?;
    }
    Ok(tx_id)
}

/// 分页查询库存流水。
pub fn list_transactions(pool: &DbPool, item_id: Option<i64>, page: i64, page_size: i64) -> Result<(Vec<TransactionResponse>, i64)> {
    repo::inventory_repo::list_transactions(pool, item_id, page, page_size)
}
