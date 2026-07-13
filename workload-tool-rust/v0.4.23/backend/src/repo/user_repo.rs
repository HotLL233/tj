use crate::db::DbPool;
use crate::error::Result;
use crate::models::user::{User, UserPublic, UserCreate, UserUpdate};

/// 按用户名查询用户（不存在返回 Unauthorized，供登录失败统一 401）。
pub fn get_by_username(pool: &DbPool, username: &str) -> Result<User> {
    let conn = pool.get()?;
    conn.query_row(
        "SELECT id, username, display_name, password_hash, role_id, lab_id, is_active, must_change_password, created_at, updated_at \
         FROM users WHERE username=?1",
        [username],
        |row| Ok(User {
            id: row.get(0)?, username: row.get(1)?, display_name: row.get(2)?, password_hash: row.get(3)?,
            role_id: row.get(4)?, lab_id: row.get(5)?, is_active: row.get(6)?, must_change_password: row.get(7)?,
            created_at: row.get(8)?, updated_at: row.get(9)?,
        }),
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => crate::error::AppError::Unauthorized("账号或密码错误".into()),
        _ => e.into(),
    })
}

/// 按主键查询用户（不存在返回 NotFound）。
pub fn get_by_id(pool: &DbPool, id: i64) -> Result<User> {
    let conn = pool.get()?;
    conn.query_row(
        "SELECT id, username, display_name, password_hash, role_id, lab_id, is_active, must_change_password, created_at, updated_at \
         FROM users WHERE id=?1",
        [id],
        |row| Ok(User {
            id: row.get(0)?, username: row.get(1)?, display_name: row.get(2)?, password_hash: row.get(3)?,
            role_id: row.get(4)?, lab_id: row.get(5)?, is_active: row.get(6)?, must_change_password: row.get(7)?,
            created_at: row.get(8)?, updated_at: row.get(9)?,
        }),
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => crate::error::AppError::NotFound("用户不存在".into()),
        _ => e.into(),
    })
}

/// 列出全部用户（含角色名）。
pub fn list(pool: &DbPool) -> Result<Vec<UserPublic>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT u.id, u.username, u.display_name, u.role_id, COALESCE(r.name,''), u.lab_id, u.is_active, u.must_change_password, u.created_at, u.updated_at \
         FROM users u LEFT JOIN roles r ON r.id = u.role_id ORDER BY u.id",
    )?;
    let rows = stmt.query_map([], |row| Ok(UserPublic {
        id: row.get(0)?, username: row.get(1)?, display_name: row.get(2)?, role_id: row.get(3)?,
        role_name: row.get::<_, String>(4).unwrap_or_default(),
        lab_id: row.get(5)?, is_active: row.get(6)?, must_change_password: row.get(7)?,
        created_at: row.get(8)?, updated_at: row.get(9)?,
    }))?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

/// 新建用户（校验用户名唯一），password_hash 由调用方预先 argon2 哈希。
pub fn create(pool: &DbPool, body: &UserCreate, password_hash: &str) -> Result<UserPublic> {
    let mut conn = pool.get()?;
    let exists: i64 = conn.query_row("SELECT COUNT(*) FROM users WHERE username=?1", [body.username.clone()], |r| r.get(0))?;
    if exists > 0 {
        return Err(crate::error::AppError::Conflict("用户名已存在".into()));
    }
    let display_name = body.display_name.clone().unwrap_or_else(|| body.username.clone());
    conn.execute(
        "INSERT INTO users (username, display_name, password_hash, role_id, lab_id, is_active) VALUES (?1,?2,?3,?4,?5,?6)",
        rusqlite::params!(body.username, display_name, password_hash, body.role_id, body.lab_id, body.is_active),
    )?;
    let id = conn.last_insert_rowid();
    get_public_by_id(&conn, id)
}

/// 更新用户资料（display_name / role_id / lab_id / is_active）。
pub fn update(pool: &DbPool, id: i64, body: &UserUpdate) -> Result<UserPublic> {
    let mut conn = pool.get()?;
    let mut sets: Vec<String> = vec![];
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
    if let Some(ref dn) = body.display_name { sets.push("display_name=?1".to_string()); params.push(Box::new(dn.clone())); }
    if let Some(rid) = body.role_id { sets.push(format!("role_id=?{}", params.len() + 1)); params.push(Box::new(rid)); }
    if let Some(lid) = body.lab_id { sets.push(format!("lab_id=?{}", params.len() + 1)); params.push(Box::new(lid)); }
    if let Some(act) = body.is_active { sets.push(format!("is_active=?{}", params.len() + 1)); params.push(Box::new(act)); }
    if sets.is_empty() {
        return Err(crate::error::AppError::Validation("没有需要更新的字段".into()));
    }
    sets.push("updated_at=datetime('now','localtime')".to_string());
    params.push(Box::new(id));
    let sql = format!("UPDATE users SET {} WHERE id=?{}", sets.join(","), params.len());
    conn.execute(&sql, rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())))?;
    get_public_by_id(&conn, id)
}

/// 软删用户（置 is_active=0，不直接 DELETE，保留审计关联）。
pub fn soft_delete(pool: &DbPool, id: i64) -> Result<()> {
    let mut conn = pool.get()?;
    let n = conn.execute("UPDATE users SET is_active=0, updated_at=datetime('now','localtime') WHERE id=?1", [id])?;
    if n == 0 {
        return Err(crate::error::AppError::NotFound("用户不存在".into()));
    }
    Ok(())
}

/// 设置密码哈希，并可选地清除「强制改密」标记。
pub fn set_password(pool: &DbPool, id: i64, password_hash: &str, clear_must_change: bool) -> Result<()> {
    let mut conn = pool.get()?;
    if clear_must_change {
        conn.execute(
            "UPDATE users SET password_hash=?1, must_change_password=0, updated_at=datetime('now','localtime') WHERE id=?2",
            rusqlite::params!(password_hash, id),
        )?;
    } else {
        conn.execute(
            "UPDATE users SET password_hash=?1, updated_at=datetime('now','localtime') WHERE id=?2",
            rusqlite::params!(password_hash, id),
        )?;
    }
    Ok(())
}

/// 查询用户对外视图（含角色名）。
pub fn get_public(pool: &DbPool, id: i64) -> Result<UserPublic> {
    let conn = pool.get()?;
    get_public_by_id(&conn, id)
}

/// 按角色名列出用户名（供按角色群发通知）。
pub fn find_usernames_by_role(pool: &DbPool, role_name: &str) -> Result<Vec<String>> {
    let items = list(pool)?;
    Ok(items.into_iter().filter(|u| u.role_name == role_name).map(|u| u.username).collect())
}

/// 仅查询对外视图（内部复用）。
fn get_public_by_id(conn: &rusqlite::Connection, id: i64) -> Result<UserPublic> {
    conn.query_row(
        "SELECT u.id, u.username, u.display_name, u.role_id, COALESCE(r.name,''), u.lab_id, u.is_active, u.must_change_password, u.created_at, u.updated_at \
         FROM users u LEFT JOIN roles r ON r.id = u.role_id WHERE u.id=?1",
        [id],
        |row| Ok(UserPublic {
            id: row.get(0)?, username: row.get(1)?, display_name: row.get(2)?, role_id: row.get(3)?,
            role_name: row.get::<_, String>(4).unwrap_or_default(),
            lab_id: row.get(5)?, is_active: row.get(6)?, must_change_password: row.get(7)?,
            created_at: row.get(8)?, updated_at: row.get(9)?,
        }),
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => crate::error::AppError::NotFound("用户不存在".into()),
        _ => e.into(),
    })
}
