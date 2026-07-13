//! 通知服务：单发 / 按角色群发 / 审批结果通知。

use crate::db::DbPool;
use crate::error::Result;
use crate::models::notification::NotificationCreate;
use crate::repo;

/// 给单个接收人发送通知（actor 默认为 system）。
pub fn notify_user(
    pool: &DbPool,
    recipient: &str,
    sender: &str,
    title: &str,
    content: &str,
    link: &str,
    module: &str,
) -> Result<()> {
    let body = NotificationCreate {
        recipient: recipient.to_string(),
        sender: sender.to_string(),
        title: title.to_string(),
        content: content.to_string(),
        link: link.to_string(),
        module: module.to_string(),
    };
    repo::notification_repo::create(pool, &body, sender)?;
    Ok(())
}

/// 给某角色下所有用户群发通知。
pub fn notify_role(
    pool: &DbPool,
    role_name: &str,
    sender: &str,
    title: &str,
    content: &str,
    link: &str,
    module: &str,
) -> Result<()> {
    let recipients = repo::user_repo::find_usernames_by_role(pool, role_name)?;
    if recipients.is_empty() {
        return Ok(());
    }
    repo::notification_repo::create_batch(pool, &recipients, sender, title, content, link, module, sender)?;
    Ok(())
}
