//! 审批流数据访问层：审批规则 / 审批任务。
//! 规则匹配：biz_type 命中 + 申请人/角色/对象条件全部满足；多条命中取 priority 最小者。
//! 写操作统一在事务内完成业务 SQL + 审计（module='approval'）。

use crate::db::DbPool;
use crate::error::Result;
use crate::models::approval::{ApprovalRule, ApprovalRuleCreate, ApprovalRuleUpdate, ApprovalTask};
use crate::repo::audit_repo;

fn map_rule(row: &rusqlite::Row) -> rusqlite::Result<ApprovalRule> {
    Ok(ApprovalRule {
        id: row.get(0)?, biz_type: row.get(1)?, name: row.get(2)?, applicant_role: row.get(3)?,
        applicant: row.get(4)?, object_type: row.get(5)?, object_value: row.get(6)?, approver_role: row.get(7)?,
        approver: row.get(8)?, priority: row.get(9)?, is_active: row.get(10)?, created_at: row.get(11)?,
    })
}

fn map_task(row: &rusqlite::Row) -> rusqlite::Result<ApprovalTask> {
    Ok(ApprovalTask {
        id: row.get(0)?, biz_type: row.get(1)?, biz_id: row.get(2)?, title: row.get(3)?,
        applicant: row.get(4)?, approver: row.get(5)?, approver_role: row.get(6)?, status: row.get(7)?,
        rule_id: row.get(8)?, decision_note: row.get(9)?, decided_at: row.get(10)?, created_at: row.get(11)?,
    })
}

// ── 规则 ──

pub fn list_rules(pool: &DbPool, biz_type: Option<&str>) -> Result<Vec<ApprovalRule>> {
    let conn = pool.get()?;
    let mut sql = "SELECT id, biz_type, name, applicant_role, applicant, object_type, object_value, approver_role, approver, priority, is_active, created_at FROM approval_rules".to_string();
    if let Some(bt) = biz_type { sql.push_str(&format!(" WHERE biz_type='{}'", bt.replace('\'', "''"))); }
    sql.push_str(" ORDER BY priority ASC, id DESC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], map_rule)?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

pub fn get_rule(pool: &DbPool, id: i64) -> Result<ApprovalRule> {
    let conn = pool.get()?;
    conn.query_row("SELECT id, biz_type, name, applicant_role, applicant, object_type, object_value, approver_role, approver, priority, is_active, created_at FROM approval_rules WHERE id=?1", [id], map_rule)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => crate::error::AppError::NotFound("审批规则不存在".into()),
            _ => e.into(),
        })
}

pub fn create_rule(pool: &DbPool, body: &ApprovalRuleCreate, actor: &str) -> Result<ApprovalRule> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO approval_rules (biz_type, name, applicant_role, applicant, object_type, object_value, approver_role, approver, priority, is_active) \
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,1)",
        rusqlite::params!(body.biz_type, body.name, body.applicant_role, body.applicant, body.object_type, body.object_value, body.approver_role, body.approver, body.priority),
    )?;
    let id = tx.last_insert_rowid();
    audit_repo::log_with_module_on_conn(&tx, "create", "approval_rules", Some(id), actor, &format!("新增审批规则：{} / {}", body.biz_type, body.name), "approval", None, None)?;
    tx.commit()?;
    get_rule(pool, id)
}

pub fn update_rule(pool: &DbPool, id: i64, body: &ApprovalRuleUpdate, actor: &str) -> Result<ApprovalRule> {
    let mut conn = pool.get()?;
    let mut sets: Vec<String> = vec![];
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
    macro_rules! push_opt { ($f:expr, $v:expr) => { if let Some(ref v) = $v { sets.push(format!("{}=?{}", $f, params.len() + 1)); params.push(Box::new(v.clone())); } } }
    push_opt!("biz_type", body.biz_type);
    push_opt!("name", body.name);
    if let Some(v) = &body.applicant_role { sets.push(format!("applicant_role=?{}", params.len() + 1)); params.push(Box::new(v)); }
    if let Some(v) = &body.applicant { sets.push(format!("applicant=?{}", params.len() + 1)); params.push(Box::new(v)); }
    if let Some(v) = &body.object_type { sets.push(format!("object_type=?{}", params.len() + 1)); params.push(Box::new(v)); }
    if let Some(v) = &body.object_value { sets.push(format!("object_value=?{}", params.len() + 1)); params.push(Box::new(v)); }
    if let Some(v) = &body.approver_role { sets.push(format!("approver_role=?{}", params.len() + 1)); params.push(Box::new(v)); }
    if let Some(v) = &body.approver { sets.push(format!("approver=?{}", params.len() + 1)); params.push(Box::new(v)); }
    if let Some(v) = body.priority { sets.push(format!("priority=?{}", params.len() + 1)); params.push(Box::new(v)); }
    if let Some(v) = body.is_active { sets.push(format!("is_active=?{}", params.len() + 1)); params.push(Box::new(v)); }
    if sets.is_empty() {
        return Err(crate::error::AppError::Validation("没有需要更新的字段".into()));
    }
    params.push(Box::new(id));
    let sql = format!("UPDATE approval_rules SET {} WHERE id=?{}", sets.join(","), params.len());
    let tx = conn.transaction()?;
    tx.execute(&sql, rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())))?;
    audit_repo::log_with_module_on_conn(&tx, "update", "approval_rules", Some(id), actor, "更新审批规则", "approval", None, None)?;
    tx.commit()?;
    get_rule(pool, id)
}

pub fn delete_rule(pool: &DbPool, id: i64, actor: &str) -> Result<()> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    tx.execute("DELETE FROM approval_rules WHERE id=?1", [id])?;
    audit_repo::log_with_module_on_conn(&tx, "delete", "approval_rules", Some(id), actor, "删除审批规则", "approval", None, None)?;
    tx.commit()?;
    Ok(())
}

/// 查找匹配的审批规则：biz_type 命中且所有已配置条件均满足，取 priority 最小者。无匹配返回 None（调用方默认通过）。
pub fn find_matching_rule(
    pool: &DbPool,
    biz_type: &str,
    applicant: &str,
    applicant_role: &str,
    object_type: Option<&str>,
    object_value: Option<&str>,
) -> Result<Option<ApprovalRule>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT id, biz_type, name, applicant_role, applicant, object_type, object_value, approver_role, approver, priority, is_active, created_at \
         FROM approval_rules WHERE biz_type=?1 AND is_active=1 ORDER BY priority ASC, id DESC",
    )?;
    let rows = stmt.query_map([biz_type], map_rule)?;
    let rules: Vec<ApprovalRule> = rows.collect::<std::result::Result<Vec<_>, _>>()?;
    let mut best: Option<ApprovalRule> = None;
    for r in rules {
        // applicant_role 条件
        if let Some(ref role) = r.applicant_role {
            if role != applicant_role { continue; }
        }
        // applicant 条件
        if let Some(ref a) = r.applicant {
            if a != applicant { continue; }
        }
        // object 条件
        if let Some(ref ot) = r.object_type {
            if Some(ot.as_str()) != object_type { continue; }
            if r.object_value.as_deref() != object_value { continue; }
        }
        // 命中：取 priority 最小（列表已排序，取第一个命中即可）
        best = Some(r);
        break;
    }
    Ok(best)
}

// ── 任务 ──

pub fn create_task(
    pool: &DbPool,
    biz_type: &str,
    biz_id: i64,
    title: &str,
    applicant: &str,
    approver: Option<&str>,
    approver_role: Option<&str>,
    rule_id: Option<i64>,
) -> Result<i64> {
    let mut conn = pool.get()?;
    conn.execute(
        "INSERT INTO approval_tasks (biz_type, biz_id, title, applicant, approver, approver_role, status, rule_id) VALUES (?1,?2,?3,?4,?5,?6,'待审批',?7)",
        rusqlite::params!(biz_type, biz_id, title, applicant, approver, approver_role, rule_id),
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_task(pool: &DbPool, id: i64) -> Result<ApprovalTask> {
    let conn = pool.get()?;
    conn.query_row(
        "SELECT id, biz_type, biz_id, title, applicant, approver, approver_role, status, rule_id, decision_note, decided_at, created_at FROM approval_tasks WHERE id=?1",
        [id], map_task,
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => crate::error::AppError::NotFound("审批任务不存在".into()),
        _ => e.into(),
    })
}

/// 列出审批任务（可选过滤）。approver/approver_role 用于审批中心「待我审批」。
pub fn list_tasks(
    pool: &DbPool,
    biz_type: Option<&str>,
    status: Option<&str>,
    approver: Option<&str>,
    approver_role: Option<&str>,
    applicant: Option<&str>,
) -> Result<Vec<ApprovalTask>> {
    let conn = pool.get()?;
    let mut sql = "SELECT id, biz_type, biz_id, title, applicant, approver, approver_role, status, rule_id, decision_note, decided_at, created_at FROM approval_tasks WHERE 1=1".to_string();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
    if let Some(bt) = biz_type { sql.push_str(&format!(" AND biz_type='{}'", bt.replace('\'', "''"))); }
    if let Some(s) = status { sql.push_str(&format!(" AND status='{}'", s.replace('\'', "''"))); }
    if let Some(a) = approver { sql.push_str(&format!(" AND approver='{}'", a.replace('\'', "''"))); }
    if let Some(ar) = approver_role { sql.push_str(&format!(" AND approver_role='{}'", ar.replace('\'', "''"))); }
    if let Some(ap) = applicant { sql.push_str(&format!(" AND applicant='{}'", ap.replace('\'', "''"))); }
    sql.push_str(" ORDER BY id DESC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())), map_task)?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

pub fn set_task_status(pool: &DbPool, id: i64, status: &str, approver: Option<&str>, note: &str) -> Result<()> {
    let mut conn = pool.get()?;
    conn.execute(
        "UPDATE approval_tasks SET status=?1, approver=?2, decision_note=?3, decided_at=datetime('now','localtime') WHERE id=?4",
        rusqlite::params!(status, approver, note, id),
    )?;
    Ok(())
}
