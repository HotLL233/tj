use crate::db::DbPool;
use crate::error::{AppError, Result};
use crate::models::user::{User, UserCreate, UserUpdate};
use crate::repo::audit_repo;

/// 按 username 查找用户（含部门/实验室名称）
pub fn find_by_username(pool: &DbPool, username: &str) -> Result<Option<User>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT u.id, u.username, u.password, u.division_id, d.name AS division_name, \
                u.group_id, pg.name AS group_name, u.is_admin, u.is_active, \
                u.created_at, u.updated_at \
         FROM users u \
         LEFT JOIN divisions d ON u.division_id = d.id \
         LEFT JOIN project_groups pg ON u.group_id = pg.id \
         WHERE u.username = ?1"
    )?;
    let mut rows = stmt.query_map([username], |row| {
        Ok(User {
            id: row.get(0)?,
            username: row.get(1)?,
            password: row.get(2)?,
            division_id: row.get(3)?,
            division_name: row.get(4)?,
            group_id: row.get(5)?,
            group_name: row.get(6)?,
            is_admin: row.get::<_, i64>(7)? != 0,
            is_active: row.get::<_, i64>(8)? != 0,
            created_at: row.get(9)?,
            updated_at: row.get(10)?,
        })
    })?;
    Ok(rows.next().transpose()?)
}

/// 按 id 查找用户（连接版本，供事务内使用）
fn find_by_id_on_conn(conn: &rusqlite::Connection, id: i64) -> Result<Option<User>> {
    let mut stmt = conn.prepare(
        "SELECT u.id, u.username, u.password, u.division_id, d.name AS division_name, \
                u.group_id, pg.name AS group_name, u.is_admin, u.is_active, \
                u.created_at, u.updated_at \
         FROM users u \
         LEFT JOIN divisions d ON u.division_id = d.id \
         LEFT JOIN project_groups pg ON u.group_id = pg.id \
         WHERE u.id = ?1"
    )?;
    let mut rows = stmt.query_map([id], |row| {
        Ok(User {
            id: row.get(0)?,
            username: row.get(1)?,
            password: row.get(2)?,
            division_id: row.get(3)?,
            division_name: row.get(4)?,
            group_id: row.get(5)?,
            group_name: row.get(6)?,
            is_admin: row.get::<_, i64>(7)? != 0,
            is_active: row.get::<_, i64>(8)? != 0,
            created_at: row.get(9)?,
            updated_at: row.get(10)?,
        })
    })?;
    Ok(rows.next().transpose()?)
}

/// 按 id 查找用户
pub fn find_by_id(pool: &DbPool, id: i64) -> Result<Option<User>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT u.id, u.username, u.password, u.division_id, d.name AS division_name, \
                u.group_id, pg.name AS group_name, u.is_admin, u.is_active, \
                u.created_at, u.updated_at \
         FROM users u \
         LEFT JOIN divisions d ON u.division_id = d.id \
         LEFT JOIN project_groups pg ON u.group_id = pg.id \
         WHERE u.id = ?1"
    )?;
    let mut rows = stmt.query_map([id], |row| {
        Ok(User {
            id: row.get(0)?,
            username: row.get(1)?,
            password: row.get(2)?,
            division_id: row.get(3)?,
            division_name: row.get(4)?,
            group_id: row.get(5)?,
            group_name: row.get(6)?,
            is_admin: row.get::<_, i64>(7)? != 0,
            is_active: row.get::<_, i64>(8)? != 0,
            created_at: row.get(9)?,
            updated_at: row.get(10)?,
        })
    })?;
    Ok(rows.next().transpose()?)
}

/// 获取所有用户列表
pub fn list_all(pool: &DbPool) -> Result<Vec<User>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT u.id, u.username, u.password, u.division_id, d.name AS division_name, \
                u.group_id, pg.name AS group_name, u.is_admin, u.is_active, \
                u.created_at, u.updated_at \
         FROM users u \
         LEFT JOIN divisions d ON u.division_id = d.id \
         LEFT JOIN project_groups pg ON u.group_id = pg.id \
         ORDER BY u.id ASC"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(User {
            id: row.get(0)?,
            username: row.get(1)?,
            password: row.get(2)?,
            division_id: row.get(3)?,
            division_name: row.get(4)?,
            group_id: row.get(5)?,
            group_name: row.get(6)?,
            is_admin: row.get::<_, i64>(7)? != 0,
            is_active: row.get::<_, i64>(8)? != 0,
            created_at: row.get(9)?,
            updated_at: row.get(10)?,
        })
    })?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

/// 创建用户
pub fn create(pool: &DbPool, data: &UserCreate, password_hash: &str) -> Result<User> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;

    // 唯一性校验
    let exists: i64 = tx.query_row(
        "SELECT COUNT(*) FROM users WHERE username = ?1",
        [&data.username],
        |r| r.get(0),
    )?;
    if exists > 0 {
        return Err(AppError::Conflict(format!("用户名「{}」已存在", data.username)));
    }

    tx.execute(
        "INSERT INTO users (username, password, division_id, group_id) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![data.username, password_hash, data.division_id, data.group_id],
    )?;

    let id = tx.last_insert_rowid();
    let detail = format!("注册用户#{}：「{}」", id, data.username);
    audit_repo::log_on_conn_with_module(
        &tx, "create", "users", Some(id),
        &data.username, &detail, "shared",
    )?;
    tx.commit()?;

    find_by_id_on_conn(&conn, id)?.ok_or_else(|| AppError::Internal("创建用户失败".into()))
}

/// 更新用户
pub fn update(pool: &DbPool, id: i64, data: &UserUpdate, user_name: &str) -> Result<User> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;

    let existing = find_by_id_on_conn(&tx, id)?
        .ok_or_else(|| AppError::NotFound("用户不存在".into()))?;

    let mut changes: Vec<String> = vec![];

    if let Some(ref un) = data.username {
        if !un.is_empty() && un != &existing.username {
            // 唯一性校验
            let conflict: i64 = tx.query_row(
                "SELECT COUNT(*) FROM users WHERE username = ?1 AND id <> ?2",
                rusqlite::params![un, id],
                |r| r.get(0),
            )?;
            if conflict > 0 {
                return Err(AppError::Conflict(format!("用户名「{}」已存在", un)));
            }
            tx.execute("UPDATE users SET username = ?1, updated_at = datetime('now','localtime') WHERE id = ?2", rusqlite::params![un, id])?;
            changes.push(format!("用户名 {} → {}", existing.username, un));
        }
    }

    if let Some(ref pw) = data.password {
        if !pw.is_empty() {
            let hash = bcrypt::hash(pw, bcrypt::DEFAULT_COST)
                .map_err(|e| AppError::Internal(format!("密码哈希失败: {}", e)))?;
            tx.execute("UPDATE users SET password = ?1, updated_at = datetime('now','localtime') WHERE id = ?2", rusqlite::params![hash, id])?;
            changes.push("密码已更新".into());
        }
    }

    if let Some(did) = data.division_id {
        let val = did.map(|v| v as i64);
        tx.execute("UPDATE users SET division_id = ?1, updated_at = datetime('now','localtime') WHERE id = ?2", rusqlite::params![val, id])?;
        changes.push("所属部门已更新".into());
    }

    if let Some(gid) = data.group_id {
        let val = gid.map(|v| v as i64);
        tx.execute("UPDATE users SET group_id = ?1, updated_at = datetime('now','localtime') WHERE id = ?2", rusqlite::params![val, id])?;
        changes.push("实验室已更新".into());
    }

    if let Some(admin) = data.is_admin {
        if admin != existing.is_admin {
            tx.execute("UPDATE users SET is_admin = ?1, updated_at = datetime('now','localtime') WHERE id = ?2", rusqlite::params![admin as i64, id])?;
            changes.push(format!("管理员权限 {} → {}", existing.is_admin, admin));
        }
    }

    if let Some(active) = data.is_active {
        if active != existing.is_active {
            tx.execute("UPDATE users SET is_active = ?1, updated_at = datetime('now','localtime') WHERE id = ?2", rusqlite::params![active as i64, id])?;
            changes.push(format!("启用状态 {} → {}", existing.is_active, active));
        }
    }

    if changes.is_empty() {
        drop(tx);
    } else {
        let detail = format!("修改用户#{}「{}」：{}", id, existing.username, changes.join("，"));
        audit_repo::log_on_conn_with_module(
            &tx, "update", "users", Some(id),
            user_name, &detail, "shared",
        )?;
        tx.commit()?;
    }

    find_by_id_on_conn(&conn, id)?.ok_or_else(|| AppError::Internal("更新用户失败".into()))
}

/// 软删除用户（is_active=0）
pub fn soft_delete(pool: &DbPool, id: i64, user_name: &str) -> Result<()> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;

    let existing = find_by_id_on_conn(&tx, id)?
        .ok_or_else(|| AppError::NotFound("用户不存在".into()))?;

    tx.execute("UPDATE users SET is_active = 0, updated_at = datetime('now','localtime') WHERE id = ?1", [id])?;

    let detail = format!("删除用户#{}：「{}」", id, existing.username);
    audit_repo::log_on_conn_with_module(
        &tx, "delete", "users", Some(id),
        user_name, &detail, "shared",
    )?;
    tx.commit()?;
    Ok(())
}
