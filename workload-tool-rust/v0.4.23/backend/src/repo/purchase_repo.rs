//! 采购管理数据访问层：供应商 / 采购申请 / 采购单。
//! 写操作统一在事务内完成业务 SQL + 审计（module='purchase'）。

use chrono::Local;
use crate::db::DbPool;
use crate::error::Result;
use crate::models::purchase::{OrderCreate, OrderItemCreate, OrderResponse, PurchaseOrder, PurchaseOrderItem, PurchaseRequisition, RequisitionCreate};
use crate::models::supplier::{Supplier, SupplierCreate, SupplierUpdate};
use crate::repo::audit_repo;

fn map_supplier(row: &rusqlite::Row) -> rusqlite::Result<Supplier> {
    Ok(Supplier {
        id: row.get(0)?, name: row.get(1)?, contact: row.get(2)?, phone: row.get(3)?,
        email: row.get(4)?, qualification: row.get(5)?, status: row.get(6)?, notes: row.get(7)?,
        created_at: row.get(8)?, deleted_at: row.get(9)?,
    })
}

fn map_requisition(row: &rusqlite::Row) -> rusqlite::Result<PurchaseRequisition> {
    Ok(PurchaseRequisition {
        id: row.get(0)?, requester: row.get(1)?, item_name: row.get(2)?, spec: row.get(3)?,
        quantity: row.get(4)?, unit: row.get(5)?, purpose: row.get(6)?, expected_supplier: row.get(7)?,
        status: row.get(8)?, approval_task_id: row.get(9)?, created_by: row.get(10)?, created_at: row.get(11)?,
        deleted_at: row.get(12)?,
    })
}

fn map_order(row: &rusqlite::Row) -> rusqlite::Result<PurchaseOrder> {
    Ok(PurchaseOrder {
        id: row.get(0)?, order_no: row.get(1)?, supplier_id: row.get(2)?, requisition_ids: row.get(3)?,
        total_amount: row.get(4)?, status: row.get(5)?, approval_task_id: row.get(6)?, sent_at: row.get(7)?,
        received_at: row.get(8)?, note: row.get(9)?, created_by: row.get(10)?, created_at: row.get(11)?,
        deleted_at: row.get(12)?,
    })
}

fn map_order_item(row: &rusqlite::Row) -> rusqlite::Result<PurchaseOrderItem> {
    Ok(PurchaseOrderItem {
        id: row.get(0)?, order_id: row.get(1)?, item_name: row.get(2)?, spec: row.get(3)?,
        quantity: row.get(4)?, unit_price: row.get(5)?, amount: row.get(6)?, requisition_id: row.get(7)?,
    })
}

// ── 供应商 ──

pub fn list_suppliers(pool: &DbPool) -> Result<Vec<Supplier>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare("SELECT id, name, contact, phone, email, qualification, status, notes, created_at, deleted_at FROM suppliers WHERE deleted_at IS NULL ORDER BY id DESC")?;
    let rows = stmt.query_map([], map_supplier)?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

pub fn get_supplier(pool: &DbPool, id: i64) -> Result<Supplier> {
    let conn = pool.get()?;
    conn.query_row("SELECT id, name, contact, phone, email, qualification, status, notes, created_at, deleted_at FROM suppliers WHERE id=?1 AND deleted_at IS NULL", [id], map_supplier)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => crate::error::AppError::NotFound("供应商不存在".into()),
            _ => e.into(),
        })
}

pub fn create_supplier(pool: &DbPool, body: &SupplierCreate, actor: &str) -> Result<Supplier> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO suppliers (name, contact, phone, email, qualification, notes) VALUES (?1,?2,?3,?4,?5,?6)",
        rusqlite::params!(body.name, body.contact, body.phone, body.email, body.qualification, body.notes),
    )?;
    let id = tx.last_insert_rowid();
    audit_repo::log_with_module_on_conn(&tx, "create", "suppliers", Some(id), actor, &format!("新增供应商：{}", body.name), "purchase", None, None)?;
    tx.commit()?;
    get_supplier(pool, id)
}

pub fn update_supplier(pool: &DbPool, id: i64, body: &SupplierUpdate, actor: &str) -> Result<Supplier> {
    let mut conn = pool.get()?;
    let mut sets: Vec<String> = vec![];
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
    macro_rules! push_opt { ($f:expr, $v:expr) => { if let Some(ref v) = $v { sets.push(format!("{}=?{}", $f, params.len() + 1)); params.push(Box::new(v.clone())); } } }
    push_opt!("name", body.name);
    push_opt!("contact", body.contact);
    push_opt!("phone", body.phone);
    push_opt!("email", body.email);
    push_opt!("qualification", body.qualification);
    push_opt!("status", body.status);
    push_opt!("notes", body.notes);
    if sets.is_empty() {
        return Err(crate::error::AppError::Validation("没有需要更新的字段".into()));
    }
    params.push(Box::new(id));
    let sql = format!("UPDATE suppliers SET {} WHERE id=?{}", sets.join(","), params.len());
    let tx = conn.transaction()?;
    tx.execute(&sql, rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())))?;
    audit_repo::log_with_module_on_conn(&tx, "update", "suppliers", Some(id), actor, "更新供应商", "purchase", None, None)?;
    tx.commit()?;
    get_supplier(pool, id)
}

pub fn soft_delete_supplier(pool: &DbPool, id: i64, actor: &str) -> Result<()> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    let n = tx.execute("UPDATE suppliers SET deleted_at=datetime('now','localtime') WHERE id=?1 AND deleted_at IS NULL", [id])?;
    if n == 0 {
        return Err(crate::error::AppError::NotFound("供应商不存在".into()));
    }
    audit_repo::log_with_module_on_conn(&tx, "delete", "suppliers", Some(id), actor, "删除供应商", "purchase", None, None)?;
    tx.commit()?;
    Ok(())
}

// ── 采购申请 ──

pub fn list_requisitions(pool: &DbPool, status: Option<&str>, applicant: Option<&str>) -> Result<Vec<PurchaseRequisition>> {
    let conn = pool.get()?;
    let mut sql = "SELECT id, requester, item_name, spec, quantity, unit, purpose, expected_supplier, status, approval_task_id, created_by, created_at, deleted_at FROM purchase_requisitions WHERE deleted_at IS NULL".to_string();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
    if let Some(s) = status { sql.push_str(&format!(" AND status=?{}", params.len() + 1)); params.push(Box::new(s.to_string())); }
    if let Some(a) = applicant { sql.push_str(&format!(" AND requester=?{}", params.len() + 1)); params.push(Box::new(a.to_string())); }
    sql.push_str(" ORDER BY id DESC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())), map_requisition)?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

pub fn get_requisition(pool: &DbPool, id: i64) -> Result<PurchaseRequisition> {
    let conn = pool.get()?;
    conn.query_row("SELECT id, requester, item_name, spec, quantity, unit, purpose, expected_supplier, status, approval_task_id, created_by, created_at, deleted_at FROM purchase_requisitions WHERE id=?1 AND deleted_at IS NULL", [id], map_requisition)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => crate::error::AppError::NotFound("采购申请不存在".into()),
            _ => e.into(),
        })
}

pub fn create_requisition(pool: &DbPool, body: &RequisitionCreate, applicant: &str, created_by: &str, status: &str, actor: &str) -> Result<i64> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO purchase_requisitions (requester, item_name, spec, quantity, unit, purpose, expected_supplier, status, created_by) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        rusqlite::params!(applicant, body.item_name, body.spec, body.quantity, body.unit, body.purpose, body.expected_supplier, status, created_by),
    )?;
    let id = tx.last_insert_rowid();
    audit_repo::log_with_module_on_conn(&tx, "create", "purchase_requisitions", Some(id), actor, &format!("提交采购申请：{} ×{}（{}）", body.item_name, body.quantity, status), "purchase", None, None)?;
    tx.commit()?;
    Ok(id)
}

pub fn set_requisition_status(pool: &DbPool, id: i64, status: &str, _actor: &str) -> Result<()> {
    let mut conn = pool.get()?;
    conn.execute("UPDATE purchase_requisitions SET status=?1 WHERE id=?2 AND deleted_at IS NULL", rusqlite::params!(status, id))?;
    Ok(())
}

// ── 采购单 ──

pub fn list_orders(pool: &DbPool, status: Option<&str>) -> Result<Vec<OrderResponse>> {
    let conn = pool.get()?;
    let mut sql = "SELECT o.id, o.order_no, o.supplier_id, COALESCE(s.name,''), o.requisition_ids, o.total_amount, o.status, o.approval_task_id, o.sent_at, o.received_at, o.note, o.created_by, o.created_at \
                   FROM purchase_orders o LEFT JOIN suppliers s ON s.id=o.supplier_id WHERE o.deleted_at IS NULL".to_string();
    if let Some(st) = status { sql.push_str(&format!(" AND o.status='{}'", st.replace('\'', "''"))); }
    sql.push_str(" ORDER BY o.id DESC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| {
        Ok(OrderResponse {
            id: row.get(0)?, order_no: row.get(1)?, supplier_id: row.get(2)?, supplier_name: row.get(3)?,
            requisition_ids: row.get(4)?, total_amount: row.get(5)?, status: row.get(6)?, approval_task_id: row.get(7)?,
            sent_at: row.get(8)?, received_at: row.get(9)?, note: row.get(10)?, created_by: row.get(11)?, created_at: row.get(12)?,
            items: vec![],
        })
    })?;
    let mut out: Vec<OrderResponse> = rows.collect::<std::result::Result<Vec<_>, _>>()?;
    for o in out.iter_mut() {
        o.items = list_order_items(pool, o.id)?;
    }
    Ok(out)
}

pub fn get_order(pool: &DbPool, id: i64) -> Result<OrderResponse> {
    let conn = pool.get()?;
    let order = conn.query_row(
        "SELECT o.id, o.order_no, o.supplier_id, COALESCE(s.name,''), o.requisition_ids, o.total_amount, o.status, o.approval_task_id, o.sent_at, o.received_at, o.note, o.created_by, o.created_at \
         FROM purchase_orders o LEFT JOIN suppliers s ON s.id=o.supplier_id WHERE o.id=?1 AND o.deleted_at IS NULL",
        [id],
        |row| Ok(PurchaseOrder {
            id: row.get(0)?, order_no: row.get(1)?, supplier_id: row.get(2)?, requisition_ids: row.get(3)?,
            total_amount: row.get(4)?, status: row.get(5)?, approval_task_id: row.get(6)?, sent_at: row.get(7)?,
            received_at: row.get(8)?, note: row.get(9)?, created_by: row.get(10)?, created_at: row.get(11)?, deleted_at: None,
        }),
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => crate::error::AppError::NotFound("采购单不存在".into()),
        _ => e.into(),
    })?;
    let items = list_order_items(pool, id)?;
    Ok(OrderResponse {
        id: order.id, order_no: order.order_no, supplier_id: order.supplier_id, supplier_name: String::new(),
        requisition_ids: order.requisition_ids, total_amount: order.total_amount, status: order.status,
        approval_task_id: order.approval_task_id, sent_at: order.sent_at, received_at: order.received_at,
        note: order.note, created_by: order.created_by, created_at: order.created_at, items,
    })
}

fn list_order_items(pool: &DbPool, order_id: i64) -> Result<Vec<PurchaseOrderItem>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare("SELECT id, order_id, item_name, spec, quantity, unit_price, amount, requisition_id FROM purchase_order_items WHERE order_id=?1 ORDER BY id")?;
    let rows = stmt.query_map([order_id], map_order_item)?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

pub fn create_order(pool: &DbPool, body: &OrderCreate, created_by: &str, status: &str, actor: &str) -> Result<i64> {
    let mut conn = pool.get()?;
    let order_no = format!("PO{}", Local::now().format("%Y%m%d%H%M%S"));
    let mut total: f64 = 0.0;
    for it in &body.items {
        total += it.quantity * it.unit_price;
    }
    let req_ids = body.requisition_ids.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(",");
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO purchase_orders (order_no, supplier_id, requisition_ids, total_amount, status, created_by) VALUES (?1,?2,?3,?4,?5,?6)",
        rusqlite::params!(order_no, body.supplier_id, req_ids, total, status, created_by),
    )?;
    let order_id = tx.last_insert_rowid();
    for it in &body.items {
        let amount = it.quantity * it.unit_price;
        tx.execute(
            "INSERT INTO purchase_order_items (order_id, item_name, spec, quantity, unit_price, amount, requisition_id) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            rusqlite::params!(order_id, it.item_name, it.spec, it.quantity, it.unit_price, amount, it.requisition_id),
        )?;
    }
    audit_repo::log_with_module_on_conn(&tx, "create", "purchase_orders", Some(order_id), actor, &format!("创建采购单：{}（{}，合计 {}）", order_no, status, total), "purchase", None, None)?;
    tx.commit()?;
    Ok(order_id)
}

pub fn set_order_status(pool: &DbPool, id: i64, status: &str, actor: &str) -> Result<()> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    tx.execute("UPDATE purchase_orders SET status=?1 WHERE id=?2 AND deleted_at IS NULL", rusqlite::params!(status, id))?;
    audit_repo::log_with_module_on_conn(&tx, "update", "purchase_orders", Some(id), actor, &format!("采购单状态变更：{}", status), "purchase", None, None)?;
    tx.commit()?;
    Ok(())
}

pub fn mark_order_received(pool: &DbPool, id: i64, actor: &str) -> Result<()> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    tx.execute("UPDATE purchase_orders SET received_at=datetime('now','localtime'), status='已收货' WHERE id=?1 AND deleted_at IS NULL", [id])?;
    audit_repo::log_with_module_on_conn(&tx, "update", "purchase_orders", Some(id), actor, "采购单到货", "purchase", None, None)?;
    tx.commit()?;
    Ok(())
}

/// 在已有连接（事务）上设置采购单状态并写审计（module='purchase'）。
/// 供采购审批通过回写时调用，与库存回写处于同一事务。
pub fn set_order_status_on_conn(conn: &rusqlite::Connection, id: i64, status: &str, actor: &str) -> Result<()> {
    conn.execute("UPDATE purchase_orders SET status=?1 WHERE id=?2 AND deleted_at IS NULL", rusqlite::params!(status, id))?;
    audit_repo::log_with_module_on_conn(conn, "update", "purchase_orders", Some(id), actor, &format!("采购单状态变更：{}", status), "purchase", None, None)?;
    Ok(())
}

/// 在已有连接（事务）上读取采购单明细（用于审批通过回写库存，避免跨连接读取）。
fn list_order_items_on_conn(conn: &rusqlite::Connection, order_id: i64) -> Result<Vec<PurchaseOrderItem>> {
    let mut stmt = conn.prepare("SELECT id, order_id, item_name, spec, quantity, unit_price, amount, requisition_id FROM purchase_order_items WHERE order_id=?1 ORDER BY id")?;
    let rows = stmt.query_map([order_id], map_order_item)?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

/// 在已有连接（事务）上读取采购单及其明细（供采购审批通过回写时调用）。
pub fn get_order_on_conn(conn: &rusqlite::Connection, id: i64) -> Result<OrderResponse> {
    let order = conn.query_row(
        "SELECT o.id, o.order_no, o.supplier_id, COALESCE(s.name,''), o.requisition_ids, o.total_amount, o.status, o.approval_task_id, o.sent_at, o.received_at, o.note, o.created_by, o.created_at \
         FROM purchase_orders o LEFT JOIN suppliers s ON s.id=o.supplier_id WHERE o.id=?1 AND o.deleted_at IS NULL",
        [id],
        |row| Ok(PurchaseOrder {
            id: row.get(0)?, order_no: row.get(1)?, supplier_id: row.get(2)?, requisition_ids: row.get(3)?,
            total_amount: row.get(4)?, status: row.get(5)?, approval_task_id: row.get(6)?, sent_at: row.get(7)?,
            received_at: row.get(8)?, note: row.get(9)?, created_by: row.get(10)?, created_at: row.get(11)?, deleted_at: None,
        }),
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => crate::error::AppError::NotFound("采购单不存在".into()),
        _ => e.into(),
    })?;
    let items = list_order_items_on_conn(conn, id)?;
    Ok(OrderResponse {
        id: order.id, order_no: order.order_no, supplier_id: order.supplier_id, supplier_name: String::new(),
        requisition_ids: order.requisition_ids, total_amount: order.total_amount, status: order.status,
        approval_task_id: order.approval_task_id, sent_at: order.sent_at, received_at: order.received_at,
        note: order.note, created_by: order.created_by, created_at: order.created_at, items,
    })
}
