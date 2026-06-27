use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{Transaction, params};
use crate::models::{AuditAction, AppError};

/// 在同一事务内执行业务闭包并写入审计日志。
/// - `action` 为业务动作标识（Create、Update、Delete 等）。
/// - `description` 为对该动作的简短说明。
/// - `user_name` 可选，记录发起人。
/// - `f` 为业务闭包，接受事务引用 `&Transaction` 并返回 `Result<T, AppError>`。
/// 成功时事务提交并写入审计日志，失败时自动回滚且不写日志。
pub fn execute_with_audit<F, T>(
    pool: &Pool<SqliteConnectionManager>,
    action: AuditAction,
    description: &str,
    user_name: Option<&str>,
    f: F,
) -> Result<T, AppError>
where
    F: FnOnce(&Transaction) -> Result<T, AppError>,
{
    let mut conn = pool.get()?; // 获取连接
    let tx = conn.transaction()?; // 开启事务

    // 业务逻辑执行
    let result = f(&tx)?;

    // 写入审计日志（同事务）
    tx.execute(
        "INSERT INTO audit_log (action, description, user_name, created_at) VALUES (?1, ?2, ?3, datetime('now','localtime'))",
        params![action.to_str(), description, user_name],
    )?;

    // 提交事务
    tx.commit()?;
    Ok(result)
}
