//! 库存管理数据访问层：分类 / 物料 / 批次 / 流水。
//! 写操作统一在事务内完成业务 SQL + 审计（module='inventory'）。

use crate::db::DbPool;
use crate::error::Result;
use crate::models::inventory::{
    BatchCreate, CategoryCreate, CategoryUpdate, InventoryBatch, InventoryCategory,
    InventoryTransaction, ItemCreate, ItemResponse, ItemUpdate, TransactionResponse,
};
use crate::repo::audit_repo;

fn map_category(row: &rusqlite::Row) -> rusqlite::Result<InventoryCategory> {
    Ok(InventoryCategory {
        id: row.get(0)?,
        name: row.get(1)?,
        parent_id: row.get(2)?,
        sort_order: row.get(3)?,
    })
}

fn map_item(row: &rusqlite::Row) -> rusqlite::Result<ItemResponse> {
    Ok(ItemResponse {
        id: row.get(0)?,
        name: row.get(1)?,
        brand: row.get(2)?,
        unit: row.get(3)?,
        category_id: row.get(4)?,
        category_name: row.get(5)?,
        tags: row.get(6)?,
        location: row.get(7)?,
        spec: row.get(8)?,
        safety_stock: row.get(9)?,
        expiry_threshold_days: row.get(10)?,
        current_quantity: row.get(11)?,
        created_by: row.get(12)?,
        created_at: row.get(13)?,
    })
}

fn map_batch(row: &rusqlite::Row) -> rusqlite::Result<InventoryBatch> {
    Ok(InventoryBatch {
        id: row.get(0)?,
        item_id: row.get(1)?,
        batch_no: row.get(2)?,
        quantity: row.get(3)?,
        unit_price: row.get(4)?,
        produced_at: row.get(5)?,
        expiry_date: row.get(6)?,
        source_type: row.get(7)?,
        source_id: row.get(8)?,
        created_at: row.get(9)?,
        deleted_at: row.get(10)?,
    })
}

fn map_tx(row: &rusqlite::Row) -> rusqlite::Result<TransactionResponse> {
    Ok(TransactionResponse {
        id: row.get(0)?,
        item_id: row.get(1)?,
        item_name: row.get(2)?,
        batch_id: row.get(3)?,
        tx_type: row.get(4)?,
        quantity: row.get(5)?,
        applicant: row.get(6)?,
        approver: row.get(7)?,
        approval_task_id: row.get(8)?,
        related_id: row.get(9)?,
        note: row.get(10)?,
        created_by: row.get(11)?,
        created_at: row.get(12)?,
    })
}

// ── 分类 ──

pub fn list_categories(pool: &DbPool) -> Result<Vec<InventoryCategory>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare("SELECT id, name, parent_id, sort_order FROM inventory_categories ORDER BY sort_order, id")?;
    let rows = stmt.query_map([], map_category)?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

pub fn create_category(pool: &DbPool, body: &CategoryCreate, actor: &str) -> Result<InventoryCategory> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO inventory_categories (name, parent_id, sort_order) VALUES (?1,?2,?3)",
        rusqlite::params!(body.name, body.parent_id, body.sort_order),
    )?;
    let id = tx.last_insert_rowid();
    audit_repo::log_with_module_on_conn(&tx, "create", "inventory_categories", Some(id), actor, &format!("新增分类：{}", body.name), "inventory", None, None)?;
    tx.commit()?;
    conn.query_row("SELECT id, name, parent_id, sort_order FROM inventory_categories WHERE id=?1", [id], map_category)
        .map_err(|e| e.into())
}

pub fn update_category(pool: &DbPool, id: i64, body: &CategoryUpdate, actor: &str) -> Result<InventoryCategory> {
    let mut conn = pool.get()?;
    let mut sets: Vec<String> = vec![];
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
    if let Some(ref v) = body.name { sets.push("name=?1".into()); params.push(Box::new(v.clone())); }
    if let Some(v) = body.parent_id { sets.push(format!("parent_id=?{}", params.len() + 1)); params.push(Box::new(v)); }
    if let Some(v) = body.sort_order { sets.push(format!("sort_order=?{}", params.len() + 1)); params.push(Box::new(v)); }
    if sets.is_empty() {
        return Err(crate::error::AppError::Validation("没有需要更新的字段".into()));
    }
    params.push(Box::new(id));
    let sql = format!("UPDATE inventory_categories SET {} WHERE id=?{}", sets.join(","), params.len());
    let tx = conn.transaction()?;
    tx.execute(&sql, rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())))?;
    audit_repo::log_with_module_on_conn(&tx, "update", "inventory_categories", Some(id), actor, "更新分类", "inventory", None, None)?;
    tx.commit()?;
    conn.query_row("SELECT id, name, parent_id, sort_order FROM inventory_categories WHERE id=?1", [id], map_category)
        .map_err(|e| e.into())
}

pub fn delete_category(pool: &DbPool, id: i64, actor: &str) -> Result<()> {
    let mut conn = pool.get()?;
    let used: i64 = conn.query_row("SELECT COUNT(*) FROM inventory_items WHERE category_id=?1 AND deleted_at IS NULL", [id], |r| r.get(0))?;
    if used > 0 {
        return Err(crate::error::AppError::Conflict("该分类下仍有物料，无法删除".into()));
    }
    let tx = conn.transaction()?;
    tx.execute("DELETE FROM inventory_categories WHERE id=?1", [id])?;
    audit_repo::log_with_module_on_conn(&tx, "delete", "inventory_categories", Some(id), actor, "删除分类", "inventory", None, None)?;
    tx.commit()?;
    Ok(())
}

// ── 物料 ──

const ITEM_SELECT: &str = "SELECT i.id, i.name, i.brand, i.unit, i.category_id, COALESCE(c.name,''), i.tags, i.location, i.spec, i.safety_stock, i.expiry_threshold_days, COALESCE((SELECT SUM(quantity) FROM inventory_batches b WHERE b.item_id=i.id AND b.deleted_at IS NULL),0), i.created_by, i.created_at FROM inventory_items i LEFT JOIN inventory_categories c ON c.id=i.category_id";

pub fn list_items(pool: &DbPool, category_id: Option<i64>, low_stock: bool) -> Result<Vec<ItemResponse>> {
    let conn = pool.get()?;
    let mut sql = format!("{} WHERE i.deleted_at IS NULL", ITEM_SELECT);
    if let Some(cid) = category_id { sql.push_str(&format!(" AND i.category_id={}", cid)); }
    if low_stock { sql.push_str(" AND COALESCE((SELECT SUM(quantity) FROM inventory_batches b WHERE b.item_id=i.id AND b.deleted_at IS NULL),0) <= i.safety_stock"); }
    sql.push_str(" ORDER BY i.id DESC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], map_item)?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

pub fn get_item(pool: &DbPool, id: i64) -> Result<ItemResponse> {
    let conn = pool.get()?;
    conn.query_row(&format!("{} WHERE i.id=?1 AND i.deleted_at IS NULL", ITEM_SELECT), [id], map_item)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => crate::error::AppError::NotFound("物料不存在".into()),
            _ => e.into(),
        })
}

pub fn create_item(pool: &DbPool, body: &ItemCreate, created_by: &str, actor: &str) -> Result<ItemResponse> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO inventory_items (name, brand, unit, category_id, tags, location, spec, safety_stock, expiry_threshold_days, created_by) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
        rusqlite::params!(body.name, body.brand, body.unit, body.category_id, body.tags, body.location, body.spec, body.safety_stock, body.expiry_threshold_days, created_by),
    )?;
    let id = tx.last_insert_rowid();
    audit_repo::log_with_module_on_conn(&tx, "create", "inventory_items", Some(id), actor, &format!("新增物料：{}", body.name), "inventory", None, None)?;
    tx.commit()?;
    get_item(pool, id)
}

pub fn update_item(pool: &DbPool, id: i64, body: &ItemUpdate, actor: &str) -> Result<ItemResponse> {
    let mut conn = pool.get()?;
    let mut sets: Vec<String> = vec![];
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
    macro_rules! push_opt { ($f:expr, $v:expr) => { if let Some(ref v) = $v { sets.push(format!("{}=?{}", $f, params.len() + 1)); params.push(Box::new(v.clone())); } } }
    push_opt!("name", body.name);
    push_opt!("brand", body.brand);
    push_opt!("unit", body.unit);
    if let Some(v) = body.category_id { sets.push(format!("category_id=?{}", params.len() + 1)); params.push(Box::new(v)); }
    push_opt!("tags", body.tags);
    push_opt!("location", body.location);
    push_opt!("spec", body.spec);
    if let Some(v) = body.safety_stock { sets.push(format!("safety_stock=?{}", params.len() + 1)); params.push(Box::new(v)); }
    if let Some(v) = body.expiry_threshold_days { sets.push(format!("expiry_threshold_days=?{}", params.len() + 1)); params.push(Box::new(v)); }
    if sets.is_empty() {
        return Err(crate::error::AppError::Validation("没有需要更新的字段".into()));
    }
    params.push(Box::new(id));
    let sql = format!("UPDATE inventory_items SET {} WHERE id=?{}", sets.join(","), params.len());
    let tx = conn.transaction()?;
    tx.execute(&sql, rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())))?;
    audit_repo::log_with_module_on_conn(&tx, "update", "inventory_items", Some(id), actor, "更新物料", "inventory", None, None)?;
    tx.commit()?;
    get_item(pool, id)
}

pub fn soft_delete_item(pool: &DbPool, id: i64, actor: &str) -> Result<()> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    let n = tx.execute("UPDATE inventory_items SET deleted_at=datetime('now','localtime') WHERE id=?1 AND deleted_at IS NULL", [id])?;
    if n == 0 {
        return Err(crate::error::AppError::NotFound("物料不存在".into()));
    }
    audit_repo::log_with_module_on_conn(&tx, "delete", "inventory_items", Some(id), actor, "删除物料", "inventory", None, None)?;
    tx.commit()?;
    Ok(())
}

// ── 批次 ──

pub fn list_batches(pool: &DbPool, item_id: i64) -> Result<Vec<InventoryBatch>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT id, item_id, batch_no, quantity, unit_price, produced_at, expiry_date, source_type, source_id, created_at, deleted_at \
         FROM inventory_batches WHERE item_id=?1 AND deleted_at IS NULL ORDER BY id DESC",
    )?;
    let rows = stmt.query_map([item_id], map_batch)?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

/// 手动入库：新增批次并记一笔 'in' 流水（同一事务 + 审计）。
pub fn create_batch(pool: &DbPool, body: &BatchCreate, actor: &str) -> Result<InventoryBatch> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO inventory_batches (item_id, batch_no, quantity, unit_price, produced_at, expiry_date, source_type, source_id) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        rusqlite::params!(body.item_id, body.batch_no, body.quantity, body.unit_price, body.produced_at, body.expiry_date, body.source_type, body.source_id),
    )?;
    let batch_id = tx.last_insert_rowid();
    tx.execute(
        "INSERT INTO inventory_transactions (item_id, batch_id, tx_type, quantity, created_by, note) VALUES (?1,?2,'in',?3,?4,'手动入库')",
        rusqlite::params!(body.item_id, batch_id, body.quantity, actor),
    )?;
    audit_repo::log_with_module_on_conn(&tx, "create", "inventory_batches", Some(batch_id), actor, &format!("入库批次：+{}", body.quantity), "inventory", None, None)?;
    tx.commit()?;
    conn.query_row(
        "SELECT id, item_id, batch_no, quantity, unit_price, produced_at, expiry_date, source_type, source_id, created_at, deleted_at FROM inventory_batches WHERE id=?1",
        [batch_id], map_batch,
    ).map_err(|e| e.into())
}

// ── 流水 ──

pub fn list_transactions(pool: &DbPool, item_id: Option<i64>, page: i64, page_size: i64) -> Result<(Vec<TransactionResponse>, i64)> {
    let conn = pool.get()?;
    let mut where_sql = String::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
    if let Some(iid) = item_id { where_sql = format!(" WHERE t.item_id={}", iid); }
    let count: i64 = if where_sql.is_empty() {
        conn.query_row("SELECT COUNT(*) FROM inventory_transactions t", [], |r| r.get(0))?
    } else {
        conn.query_row(&format!("SELECT COUNT(*) FROM inventory_transactions t{}", where_sql), [], |r| r.get(0))?
    };
    let offset = (page - 1) * page_size;
    let sql = format!(
        "SELECT t.id, t.item_id, i.name, t.batch_id, t.tx_type, t.quantity, t.applicant, t.approver, t.approval_task_id, t.related_id, t.note, t.created_by, t.created_at \
         FROM inventory_transactions t LEFT JOIN inventory_items i ON i.id=t.item_id{} ORDER BY t.id DESC LIMIT ?1 OFFSET ?2",
        where_sql
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([page_size, offset], map_tx)?;
    Ok((rows.collect::<std::result::Result<Vec<_>, _>>()?, count))
}

// ── 跨事务 / 跨模块辅助（供 service 在审批回调中调用） ──

/// 在已有连接上计算物料实时库存（批次 quantity 之和）。
pub fn current_quantity_on_conn(conn: &rusqlite::Connection, item_id: i64) -> Result<f64> {
    let v: Option<f64> = conn.query_row(
        "SELECT SUM(quantity) FROM inventory_batches WHERE item_id=?1 AND deleted_at IS NULL",
        [item_id],
        |r| r.get(0),
    )?;
    Ok(v.unwrap_or(0.0))
}

/// 在已有连接上插入一条出库/报废流水（不立即扣减库存，待审批通过后由 apply_out 扣减）。
pub fn insert_out_transaction_on_conn(
    conn: &rusqlite::Connection,
    item_id: i64,
    tx_type: &str,
    quantity: f64,
    applicant: &str,
    created_by: &str,
    approval_task_id: Option<i64>,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO inventory_transactions (item_id, tx_type, quantity, applicant, created_by, approval_task_id, note) VALUES (?1,?2,?3,?4,?5,?6,'待审批出库')",
        rusqlite::params!(item_id, tx_type, quantity, applicant, created_by, approval_task_id),
    )?;
    Ok(conn.last_insert_rowid())
}

/// 池级别插入出库/报废流水（approval_task_id 可空）。返回流水 ID。
pub fn create_out_transaction(
    pool: &DbPool,
    item_id: i64,
    tx_type: &str,
    quantity: f64,
    applicant: &str,
    created_by: &str,
    approval_task_id: Option<i64>,
) -> Result<i64> {
    let mut conn = pool.get()?;
    conn.execute(
        "INSERT INTO inventory_transactions (item_id, tx_type, quantity, applicant, created_by, approval_task_id, note) VALUES (?1,?2,?3,?4,?5,?6,'待审批出库')",
        rusqlite::params!(item_id, tx_type, quantity, applicant, created_by, approval_task_id),
    )?;
    Ok(conn.last_insert_rowid())
}

/// 将出库流水与审批任务关联（待审批阶段调用）。
pub fn link_transaction_task(pool: &DbPool, tx_id: i64, task_id: i64) -> Result<()> {
    let mut conn = pool.get()?;
    conn.execute("UPDATE inventory_transactions SET approval_task_id=?1 WHERE id=?2", rusqlite::params!(task_id, tx_id))?;
    Ok(())
}

/// FIFO 扣减库存（按到期日升序、id 升序）。在已有连接上执行。
pub fn reduce_stock_fifo_on_conn(conn: &rusqlite::Connection, item_id: i64, mut qty: f64) -> Result<()> {
    let mut stmt = conn.prepare(
        "SELECT id, quantity FROM inventory_batches WHERE item_id=?1 AND deleted_at IS NULL AND quantity>0 ORDER BY expiry_date ASC, id ASC",
    )?;
    let rows = stmt.query_map([item_id], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, f64>(1)?)))?;
    let batches: Vec<(i64, f64)> = rows.collect::<std::result::Result<Vec<_>, _>>()?;
    for (bid, avail) in batches {
        if qty <= 0.0 { break; }
        let take = avail.min(qty);
        let remain = avail - take;
        conn.execute("UPDATE inventory_batches SET quantity=?1 WHERE id=?2", rusqlite::params!(remain, bid))?;
        qty -= take;
    }
    if qty > 0.0001 {
        return Err(crate::error::AppError::Validation("库存不足，无法完成出库".into()));
    }
    Ok(())
}

/// 审批通过后的出库落地：扣减库存并标记审批人（独立事务 + 审计）。
pub fn apply_out(pool: &DbPool, tx_id: i64, approver: &str, actor: &str) -> Result<()> {
    let mut conn = pool.get()?;
    let (item_id, qty, tx_type): (i64, f64, String) = conn.query_row(
        "SELECT item_id, quantity, tx_type FROM inventory_transactions WHERE id=?1",
        [tx_id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => crate::error::AppError::NotFound("流水不存在".into()),
        _ => e.into(),
    })?;
    let tx = conn.transaction()?;
    reduce_stock_fifo_on_conn(&tx, item_id, qty)?;
    tx.execute(
        "UPDATE inventory_transactions SET approver=?1, note='审批通过出库' WHERE id=?2",
        rusqlite::params!(approver, tx_id),
    )?;
    audit_repo::log_with_module_on_conn(&tx, "approval", "inventory_transactions", Some(tx_id), actor, &format!("审批通过{}：{}", tx_type, qty), "inventory", None, None)?;
    tx.commit()?;
    Ok(())
}

/// 采购到货回写库存：按名称+规格查找或新建物料，生成批次 + 'in' 流水（独立事务 + 审计）。
/// 供采购单审批通过时调用。
pub fn receive_purchase(
    pool: &DbPool,
    item_name: &str,
    spec: &str,
    quantity: f64,
    unit_price: f64,
    order_id: i64,
    actor: &str,
) -> Result<()> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    // 查找或新建物料
    let item_id: i64 = match tx.query_row(
        "SELECT id FROM inventory_items WHERE name=?1 AND (spec IS NULL OR spec='' OR spec=?2) AND deleted_at IS NULL LIMIT 1",
        rusqlite::params!(item_name, spec),
        |r| r.get::<_, i64>(0),
    ) {
        Ok(id) => id,
        Err(_) => {
            tx.execute(
                "INSERT INTO inventory_items (name, spec, unit, created_by) VALUES (?1,?2,'个',?3)",
                rusqlite::params!(item_name, spec, actor),
            )?;
            tx.last_insert_rowid()
        }
    };
    tx.execute(
        "INSERT INTO inventory_batches (item_id, batch_no, quantity, unit_price, source_type, source_id) VALUES (?1,'',?2,?3,'purchase',?4)",
        rusqlite::params!(item_id, quantity, unit_price, order_id),
    )?;
    let batch_id = tx.last_insert_rowid();
    tx.execute(
        "INSERT INTO inventory_transactions (item_id, batch_id, tx_type, quantity, created_by, related_id, note) VALUES (?1,?2,'in',?3,?4,?5,'采购到货入库')",
        rusqlite::params!(item_id, batch_id, quantity, actor, order_id),
    )?;
    audit_repo::log_with_module_on_conn(&tx, "create", "inventory_batches", Some(batch_id), actor, &format!("采购到货入库：{} ×{}", item_name, quantity), "inventory", None, None)?;
    tx.commit()?;
    Ok(())
}

/// 在已有连接（事务）上执行采购到货回写库存：按名称+规格查找或新建物料，生成批次 + 'in' 流水。
/// 由调用方统一管理事务，避免与订单状态变更形成嵌套/分散事务。
pub fn receive_purchase_on_conn(
    conn: &rusqlite::Connection,
    item_name: &str,
    spec: &str,
    quantity: f64,
    unit_price: f64,
    order_id: i64,
    actor: &str,
) -> Result<()> {
    let item_id: i64 = match conn.query_row(
        "SELECT id FROM inventory_items WHERE name=?1 AND (spec IS NULL OR spec='' OR spec=?2) AND deleted_at IS NULL LIMIT 1",
        rusqlite::params!(item_name, spec),
        |r| r.get::<_, i64>(0),
    ) {
        Ok(id) => id,
        Err(_) => {
            conn.execute(
                "INSERT INTO inventory_items (name, spec, unit, created_by) VALUES (?1,?2,'个',?3)",
                rusqlite::params!(item_name, spec, actor),
            )?;
            conn.last_insert_rowid()
        }
    };
    conn.execute(
        "INSERT INTO inventory_batches (item_id, batch_no, quantity, unit_price, source_type, source_id) VALUES (?1,'',?2,?3,'purchase',?4)",
        rusqlite::params!(item_id, quantity, unit_price, order_id),
    )?;
    let batch_id = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO inventory_transactions (item_id, batch_id, tx_type, quantity, created_by, related_id, note) VALUES (?1,?2,'in',?3,?4,?5,'采购到货入库')",
        rusqlite::params!(item_id, batch_id, quantity, actor, order_id),
    )?;
    audit_repo::log_with_module_on_conn(conn, "create", "inventory_batches", Some(batch_id), actor, &format!("采购到货入库：{} ×{}", item_name, quantity), "inventory", None, None)?;
    Ok(())
}
