//! 采购业务服务：申请 / 采购单（审批 + 到货入库回写）。

use crate::db::DbPool;
use crate::error::Result;
use crate::models::purchase::{OrderCreate, RequisitionCreate};
use crate::repo;
use crate::service::approval_service;

/// 提交采购申请：进入审批流；无匹配规则自动通过。
pub fn submit_requisition(
    pool: &DbPool,
    body: &RequisitionCreate,
    applicant: &str,
    applicant_role: &str,
    created_by: &str,
    actor: &str,
) -> Result<i64> {
    let id = repo::purchase_repo::create_requisition(pool, body, applicant, created_by, "待审批", actor)?;
    let title = format!("采购申请：{} ×{}", body.item_name, body.quantity);
    let outcome = approval_service::submit_for_approval(
        pool, "purchase_requisition", id, &title, applicant, applicant_role, None, None,
    )?;
    if outcome.auto_approved {
        repo::purchase_repo::set_requisition_status(pool, id, "已通过", "system")?;
    }
    Ok(id)
}

/// 创建采购单：进入审批流；无匹配规则自动通过并即时回写库存。
pub fn create_order(
    pool: &DbPool,
    body: &OrderCreate,
    created_by: &str,
    applicant_role: &str,
    actor: &str,
) -> Result<i64> {
    let id = repo::purchase_repo::create_order(pool, body, created_by, "待审批", actor)?;
    let order = repo::purchase_repo::get_order(pool, id)?;
    let title = format!("采购单审批：{}", order.order_no);
    let outcome = approval_service::submit_for_approval(
        pool, "purchase_order", id, &title, created_by, applicant_role, None, None,
    )?;
    if outcome.auto_approved {
        // 订单状态变更 + 每个 item 的库存回写，合并到单一事务（T06 硬要求）。
        let mut conn = pool.get()?;
        let tx = conn.transaction()?;
        repo::purchase_repo::set_order_status_on_conn(&tx, id, "已通过", "system")?;
        for it in order.items {
            repo::inventory_repo::receive_purchase_on_conn(&tx, &it.item_name, &it.spec, it.quantity, it.unit_price, id, "system")?;
        }
        tx.commit()?;
    }
    Ok(id)
}

/// 采购单到货登记。
pub fn receive_order(pool: &DbPool, id: i64, actor: &str) -> Result<()> {
    repo::purchase_repo::mark_order_received(pool, id, actor)
}
