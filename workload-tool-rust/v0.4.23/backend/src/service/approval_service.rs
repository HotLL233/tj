//! 审批引擎：提交审批（匹配规则 / 默认通过）+ 审批决策（落地业务副作用）。
//!
//! 设计要点：
//! - 无匹配规则 → 默认通过（记录通知，不生成待办任务）。
//! - 命中规则 → 生成待审批任务，并通知指定审批人 / 审批角色。
//! - 决策通过时，按 biz_type 分发到各业务模块落地（仪器预约确认、采购入库回写、库存扣减等）。

use crate::db::DbPool;
use crate::error::Result;
use crate::repo;

/// 提交审批后的结果。
pub struct ApprovalOutcome {
    /// 生成的审批任务 ID（默认通过时为 None）
    pub task_id: Option<i64>,
    /// 是否默认通过（无匹配规则）
    pub auto_approved: bool,
}

/// biz_type → 通知所属模块
fn module_for(biz_type: &str) -> &'static str {
    match biz_type {
        "instrument_booking" => "instrument",
        "purchase_requisition" | "purchase_order" => "purchase",
        "inventory_out" => "inventory",
        _ => "approval",
    }
}

/// 提交一笔业务进入审批流。返回是否需人工审批。
pub fn submit_for_approval(
    pool: &DbPool,
    biz_type: &str,
    biz_id: i64,
    title: &str,
    applicant: &str,
    applicant_role: &str,
    object_type: Option<&str>,
    object_value: Option<&str>,
) -> Result<ApprovalOutcome> {
    let rule = repo::approval_repo::find_matching_rule(pool, biz_type, applicant, applicant_role, object_type, object_value)?;
    match rule {
        None => {
            // 默认通过：通知申请人
            crate::service::notification_service::notify_user(
                pool, applicant, "system",
                &format!("「{}」已自动通过", title),
                "未匹配到审批规则，系统已自动通过。", "", module_for(biz_type),
            )?;
            repo::audit_repo::log_with_module(
                pool, "approval_auto", biz_type, Some(biz_id), applicant,
                &format!("无匹配审批规则，默认通过：{}", title), "approval",
            )?;
            Ok(ApprovalOutcome { task_id: None, auto_approved: true })
        }
        Some(r) => {
            let task_id = repo::approval_repo::create_task(
                pool, biz_type, biz_id, title, applicant,
                r.approver.as_deref(), r.approver_role.as_deref(), Some(r.id),
            )?;
            // 通知审批人
            if let Some(a) = &r.approver {
                crate::service::notification_service::notify_user(
                    pool, a, "system",
                    &format!("待审批：{}", title),
                    &format!("申请人：{}", applicant), "", module_for(biz_type),
                )?;
            }
            if let Some(role) = &r.approver_role {
                crate::service::notification_service::notify_role(
                    pool, role, "system",
                    &format!("待审批：{}", title),
                    &format!("申请人：{}", applicant), "", module_for(biz_type),
                )?;
            }
            repo::audit_repo::log_with_module(
                pool, "approval_submit", biz_type, Some(biz_id), applicant,
                &format!("提交审批（任务 #{}）：{}", task_id, title), "approval",
            )?;
            Ok(ApprovalOutcome { task_id: Some(task_id), auto_approved: false })
        }
    }
}

/// 审批决策：approve 落地业务副作用，reject 仅更新状态；均通知申请人。
pub fn decide(
    pool: &DbPool,
    task_id: i64,
    decision: &str,
    approver: &str,
    note: &str,
) -> Result<()> {
    let task = repo::approval_repo::get_task(pool, task_id)?;
    if task.status != "待审批" {
        return Err(crate::error::AppError::Validation("该审批任务已处理，无法重复操作".into()));
    }
    if decision == "approve" {
        repo::approval_repo::set_task_status(pool, task_id, "已通过", Some(approver), note)?;
        apply_effect(pool, &task.biz_type, task.biz_id, approver)?;
    } else {
        repo::approval_repo::set_task_status(pool, task_id, "已拒绝", Some(approver), note)?;
    }
    // 通知申请人
    let result = if decision == "approve" { "已通过" } else { "已拒绝" };
    crate::service::notification_service::notify_user(
        pool, &task.applicant, "system",
        &format!("审批结果：{}", task.title),
        &format!("{}。审批人：{}；意见：{}", result, approver, note),
        "", module_for(&task.biz_type),
    )?;
    repo::audit_repo::log_with_module(
        pool, "approval_decide", "approval_tasks", Some(task_id), approver,
        &format!("审批决策：{}（{}）", result, note), "approval",
    )?;
    Ok(())
}

/// 按业务类型落地审批通过后的副作用。
fn apply_effect(pool: &DbPool, biz_type: &str, biz_id: i64, approver: &str) -> Result<()> {
    match biz_type {
        "instrument_booking" => {
            repo::instrument_repo::set_booking_status(pool, biz_id, "已通过", approver, "审批通过", approver)?;
        }
        "purchase_requisition" => {
            repo::purchase_repo::set_requisition_status(pool, biz_id, "已通过", approver)?;
        }
        "purchase_order" => {
            // 订单状态变更 + 每个 item 的库存回写，合并到单一事务，保证原子性（T06 硬要求）。
            let mut conn = pool.get()?;
            let tx = conn.transaction()?;
            repo::purchase_repo::set_order_status_on_conn(&tx, biz_id, "已通过", approver)?;
            let order = repo::purchase_repo::get_order_on_conn(&tx, biz_id)?;
            for it in order.items {
                repo::inventory_repo::receive_purchase_on_conn(&tx, &it.item_name, &it.spec, it.quantity, it.unit_price, biz_id, approver)?;
            }
            tx.commit()?;
        }
        "inventory_out" => {
            repo::inventory_repo::apply_out(pool, biz_id, approver, approver)?;
        }
        _ => {}
    }
    Ok(())
}
