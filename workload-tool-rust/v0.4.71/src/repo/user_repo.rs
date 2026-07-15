use crate::db::DbPool;
use crate::error::{AppError, Result};
use crate::models::user::{User, UserCreate, UserUpdate};
use crate::repo::audit_repo;

/// 基础用户查询 SQL（含 role_id 列，索引 11）
const USER_SELECT: &str = "SELECT u.id, u.username, u.password, u.division_id, d.name AS division_name, \
        u.group_id, pg.name AS group_name, u.is_admin, u.is_active, \
        u.created_at, u.updated_at, u.role_id \
 FROM users u \
 LEFT JOIN divisions d ON u.division_id = d.id \
 LEFT JOIN project_groups pg ON u.group_id = pg.id";

/// 从一行映射为 User（permissions 暂置空，由调用方按需加载）
fn map_user(row: &rusqlite::Row) -> rusqlite::Result<User> {
    let role_id: Option<i64> = row.get(11)?;
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
        role_id,
        permissions: vec![],
    })
}

/// 加载某角色关联的权限点集合
fn user_perms_on_conn(conn: &rusqlite::Connection, role_id: Option<i64>) -> Result<Vec<String>> {
    let rid = match role_id {
        Some(v) => v,
        None => return Ok(vec![]),
    };
    let mut stmt = conn.prepare(
        "SELECT permission_key FROM role_permissions WHERE role_id=?1 ORDER BY permission_key",
    )?;
    let rows = stmt.query_map([rid], |row| row.get::<_, String>(0))?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

/// 单条查询并回填 permissions
fn find_one<P: rusqlite::Params>(conn: &rusqlite::Connection, sql: &str, params: P) -> Result<Option<User>> {
    // stmt 在内部块结束即释放，避免与下方 user_perms_on_conn 的二次查询产生活动语句冲突
    let user = {
        let mut stmt = conn.prepare(sql)?;
        let mut rows = stmt.query_map(params, map_user)?;
        rows.next().transpose()?
    };
    let mut user = user;
    if let Some(ref mut u) = user {
        u.permissions = user_perms_on_conn(conn, u.role_id)?;
    }
    Ok(user)
}

/// 按 username 查找用户（含部门/实验室名称、角色、权限点）
pub fn find_by_username(pool: &DbPool, username: &str) -> Result<Option<User>> {
    let conn = pool.get()?;
    find_one(
        &conn,
        &format!("{} WHERE u.username = ?1", USER_SELECT),
        [username],
    )
}

/// 按 id 查找用户（连接版本，供事务内使用）
fn find_by_id_on_conn(conn: &rusqlite::Connection, id: i64) -> Result<Option<User>> {
    find_one(
        conn,
        &format!("{} WHERE u.id = ?1", USER_SELECT),
        [id],
    )
}

/// 按 id 查找用户
pub fn find_by_id(pool: &DbPool, id: i64) -> Result<Option<User>> {
    let conn = pool.get()?;
    find_by_id_on_conn(&conn, id)
}

/// 获取所有用户列表（含角色、权限点）
pub fn list_all(pool: &DbPool) -> Result<Vec<User>> {
    let conn = pool.get()?;
    let users = {
        let mut stmt = conn.prepare(&format!("{} ORDER BY u.id ASC", USER_SELECT))?;
        let mut rows = stmt.query_map([], map_user)?;
        let mut v: Vec<User> = vec![];
        while let Some(row) = rows.next() {
            v.push(row?);
        }
        v
    };
    let mut users = users;
    for u in &mut users {
        u.permissions = user_perms_on_conn(&conn, u.role_id)?;
    }
    Ok(users)
}

/// 查询某用户所属角色的权限点（role_id 为 NULL 或角色不存在返回空 vec）
pub fn get_user_permissions(pool: &DbPool, user_id: i64) -> Result<Vec<String>> {
    let conn = pool.get()?;
    let role_id: Option<i64> = match conn.query_row(
        "SELECT role_id FROM users WHERE id=?1",
        [user_id],
        |row| row.get::<_, Option<i64>>(0),
    ) {
        Ok(v) => v,
        Err(rusqlite::Error::QueryReturnedNoRows) => None,
        Err(e) => return Err(e.into()),
    };
    user_perms_on_conn(&conn, role_id)
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
        "INSERT INTO users (username, password, division_id, group_id, role_id) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![data.username, password_hash, data.division_id, data.group_id, data.role_id],
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

    if let Some(rid) = data.role_id {
        tx.execute("UPDATE users SET role_id = ?1, updated_at = datetime('now','localtime') WHERE id = ?2", rusqlite::params![rid, id])?;
        changes.push("角色已更新".into());
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

/// v0.4.65: 彻底删除用户
pub fn permanent_delete(pool: &DbPool, id: i64) -> Result<()> {
    let conn = pool.get()?;
    conn.execute("DELETE FROM users WHERE id=?1", [id])?;
    Ok(())
}
