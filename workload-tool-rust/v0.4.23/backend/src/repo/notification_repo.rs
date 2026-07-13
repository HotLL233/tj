//! 通知 / 站内信数据访问层（module='notification'）。

use crate::db::DbPool;
use crate::error::Result;
use crate::models::notification::{Notification, NotificationCreate, NotificationResponse};
use crate::repo::audit_repo;

fn map(row: &rusqlite::Row) -> rusqlite::Result<Notification> {
    Ok(Notification {
        id: row.get(0)?, recipient: row.get(1)?, sender: row.get(2)?, title: row.get(3)?,
        content: row.get(4)?, link: row.get(5)?, module: row.get(6)?, is_read: row.get(7)?,
        created_at: row.get(8)?,
    })
}

fn to_response(n: &Notification) -> NotificationResponse {
    NotificationResponse {
        id: n.id, recipient: n.recipient.clone(), sender: n.sender.clone(), title: n.title.clone(),
        content: n.content.clone(), link: n.link.clone(), module: n.module.clone(), is_read: n.is_read,
        created_at: n.created_at.clone(),
    }
}

/// 发送通知（单条）。审计 module='notification'。
pub fn create(pool: &DbPool, body: &NotificationCreate, actor: &str) -> Result<Notification> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO notifications (recipient, sender, title, content, link, module) VALUES (?1,?2,?3,?4,?5,?6)",
        rusqlite::params!(body.recipient, body.sender, body.title, body.content, body.link, body.module),
    )?;
    let id = tx.last_insert_rowid();
    audit_repo::log_with_module_on_conn(&tx, "create", "notifications", Some(id), actor, &format!("发送通知：{} → {}", body.recipient, body.title), "notification", None, None)?;
    tx.commit()?;
    conn.query_row("SELECT id, recipient, sender, title, content, link, module, is_read, created_at FROM notifications WHERE id=?1", [id], map)
        .map_err(|e| e.into())
}

/// 批量发送给一组接收人（同一内容）。
pub fn create_batch(pool: &DbPool, recipients: &[String], sender: &str, title: &str, content: &str, link: &str, module: &str, actor: &str) -> Result<()> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    for r in recipients {
        tx.execute(
            "INSERT INTO notifications (recipient, sender, title, content, link, module) VALUES (?1,?2,?3,?4,?5,?6)",
            rusqlite::params!(r, sender, title, content, link, module),
        )?;
    }
    audit_repo::log_with_module_on_conn(&tx, "create", "notifications", None, actor, &format!("群发通知：{} 人，{}", recipients.len(), title), "notification", None, None)?;
    tx.commit()?;
    Ok(())
}

pub fn list_for(pool: &DbPool, recipient: &str, unread_only: bool) -> Result<Vec<NotificationResponse>> {
    let conn = pool.get()?;
    let mut sql = "SELECT id, recipient, sender, title, content, link, module, is_read, created_at FROM notifications WHERE recipient=?1".to_string();
    if unread_only { sql.push_str(" AND is_read=0"); }
    sql.push_str(" ORDER BY id DESC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([recipient], map)?;
    let items: Vec<Notification> = rows.collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(items.iter().map(to_response).collect())
}

pub fn unread_count(pool: &DbPool, recipient: &str) -> Result<i64> {
    let conn = pool.get()?;
    conn.query_row("SELECT COUNT(*) FROM notifications WHERE recipient=?1 AND is_read=0", [recipient], |r| r.get(0))
        .map_err(|e| e.into())
}

pub fn mark_read(pool: &DbPool, id: i64, recipient: &str) -> Result<()> {
    let mut conn = pool.get()?;
    let n = conn.execute("UPDATE notifications SET is_read=1 WHERE id=?1 AND recipient=?2", rusqlite::params!(id, recipient))?;
    if n == 0 {
        return Err(crate::error::AppError::NotFound("通知不存在".into()));
    }
    Ok(())
}

pub fn mark_all_read(pool: &DbPool, recipient: &str) -> Result<i64> {
    let mut conn = pool.get()?;
    let n = conn.execute("UPDATE notifications SET is_read=1 WHERE recipient=?1 AND is_read=0", [recipient])?;
    Ok(n as i64)
}
