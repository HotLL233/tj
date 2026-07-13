use crate::db::DbPool;
use crate::error::Result;
use crate::models::role::{Role, RoleCreate, RoleUpdate, RoleWithPermissions};
use crate::models::role::PERMISSIONS;

/// 列出全部角色（含聚合的权限点）。
pub fn list(pool: &DbPool) -> Result<Vec<RoleWithPermissions>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare("SELECT id, name, description, is_system, sort_order FROM roles ORDER BY sort_order, id")?;
    let roles = stmt.query_map([], |row| Ok(Role {
        id: row.get(0)?, name: row.get(1)?, description: row.get(2)?, is_system: row.get(3)?, sort_order: row.get(4)?,
    }))?.collect::<std::result::Result<Vec<_>, _>>()?;
    let mut out = Vec::with_capacity(roles.len());
    for r in roles {
        let perms = get_permissions_on_conn(&conn, r.id)?;
        out.push(RoleWithPermissions {
            id: r.id, name: r.name, description: r.description, is_system: r.is_system, sort_order: r.sort_order, permissions: perms,
        });
    }
    Ok(out)
}

/// 按主键查询角色。
pub fn get_by_id(pool: &DbPool, id: i64) -> Result<Role> {
    let conn = pool.get()?;
    conn.query_row(
        "SELECT id, name, description, is_system, sort_order FROM roles WHERE id=?1",
        [id],
        |row| Ok(Role { id: row.get(0)?, name: row.get(1)?, description: row.get(2)?, is_system: row.get(3)?, sort_order: row.get(4)? }),
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => crate::error::AppError::NotFound("角色不存在".into()),
        _ => e.into(),
    })
}

/// 按名称查询角色（用于种子匹配）。
pub fn get_by_name(pool: &DbPool, name: &str) -> Result<Role> {
    let conn = pool.get()?;
    conn.query_row(
        "SELECT id, name, description, is_system, sort_order FROM roles WHERE name=?1",
        [name],
        |row| Ok(Role { id: row.get(0)?, name: row.get(1)?, description: row.get(2)?, is_system: row.get(3)?, sort_order: row.get(4)? }),
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => crate::error::AppError::NotFound("角色不存在".into()),
        _ => e.into(),
    })
}

/// 查询角色名（供登录聚合 Claims 使用）。
pub fn get_name(pool: &DbPool, role_id: i64) -> Result<String> {
    let conn = pool.get()?;
    conn.query_row("SELECT name FROM roles WHERE id=?1", [role_id], |row| row.get::<_, String>(0))
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => crate::error::AppError::NotFound("角色不存在".into()),
            _ => e.into(),
        })
}

/// 获取角色的权限点集合。
pub fn get_permissions(pool: &DbPool, role_id: i64) -> Result<Vec<String>> {
    let conn = pool.get()?;
    get_permissions_on_conn(&conn, role_id)
}

fn get_permissions_on_conn(conn: &rusqlite::Connection, role_id: i64) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT permission FROM role_permissions WHERE role_id=?1 ORDER BY permission")?;
    let rows = stmt.query_map([role_id], |row| row.get::<_, String>(0))?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

/// 新建角色并写入权限点（校验权限点合法性）。
pub fn create(pool: &DbPool, body: &RoleCreate) -> Result<RoleWithPermissions> {
    let mut conn = pool.get()?;
    let exists: i64 = conn.query_row("SELECT COUNT(*) FROM roles WHERE name=?1", [body.name.clone()], |r| r.get(0))?;
    if exists > 0 {
        return Err(crate::error::AppError::Conflict("角色名已存在".into()));
    }
    let valid_set: std::collections::HashSet<&str> = PERMISSIONS.iter().copied().collect();
    for p in &body.permissions {
        if p != "*" && !valid_set.contains(p.as_str()) {
            return Err(crate::error::AppError::Validation(format!("非法权限点: {}", p)));
        }
    }
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO roles (name, description, is_system) VALUES (?1,?2,0)",
        rusqlite::params!(body.name, body.description.clone().unwrap_or_default()),
    )?;
    let id = tx.last_insert_rowid();
    for p in &body.permissions {
        tx.execute("INSERT INTO role_permissions (role_id, permission) VALUES (?1,?2)", rusqlite::params!(id, p))?;
    }
    tx.commit()?;
    get_with_permissions(&conn, id)
}

/// 更新角色基础信息（系统角色不可改名）。
pub fn update(pool: &DbPool, id: i64, body: &RoleUpdate) -> Result<RoleWithPermissions> {
    let mut conn = pool.get()?;
    let role = get_by_id(pool, id)?;
    let mut sets: Vec<String> = vec![];
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
    if let Some(ref n) = body.name {
        if role.is_system == 1 {
            return Err(crate::error::AppError::Forbidden("系统角色不可改名".into()));
        }
        sets.push("name=?1".to_string()); params.push(Box::new(n.clone()));
    }
    if let Some(ref d) = body.description { sets.push(format!("description=?{}", params.len() + 1)); params.push(Box::new(d.clone())); }
    if let Some(so) = body.sort_order { sets.push(format!("sort_order=?{}", params.len() + 1)); params.push(Box::new(so)); }
    if sets.is_empty() {
        return Err(crate::error::AppError::Validation("没有需要更新的字段".into()));
    }
    params.push(Box::new(id));
    let sql = format!("UPDATE roles SET {} WHERE id=?{}", sets.join(","), params.len());
    conn.execute(&sql, rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())))?;
    get_with_permissions(&conn, id)
}

/// 删除角色（系统角色禁止删除）。
pub fn delete(pool: &DbPool, id: i64) -> Result<()> {
    let conn = pool.get()?;
    let role = get_by_id(pool, id)?;
    if role.is_system == 1 {
        return Err(crate::error::AppError::Forbidden("系统角色不可删除".into()));
    }
    // 检查是否仍有用户引用
    let used: i64 = conn.query_row("SELECT COUNT(*) FROM users WHERE role_id=?1", [id], |r| r.get(0))?;
    if used > 0 {
        return Err(crate::error::AppError::Conflict("该角色下仍有用户，无法删除".into()));
    }
    conn.execute("DELETE FROM role_permissions WHERE role_id=?1", [id])?;
    conn.execute("DELETE FROM roles WHERE id=?1", [id])?;
    Ok(())
}

/// 整体替换角色的权限点。
pub fn set_permissions(pool: &DbPool, role_id: i64, perms: &[String]) -> Result<RoleWithPermissions> {
    let mut conn = pool.get()?;
    let valid_set: std::collections::HashSet<&str> = PERMISSIONS.iter().copied().collect();
    for p in perms {
        if p != "*" && !valid_set.contains(p.as_str()) {
            return Err(crate::error::AppError::Validation(format!("非法权限点: {}", p)));
        }
    }
    let tx = conn.transaction()?;
    tx.execute("DELETE FROM role_permissions WHERE role_id=?1", [role_id])?;
    for p in perms {
        tx.execute("INSERT INTO role_permissions (role_id, permission) VALUES (?1,?2)", rusqlite::params!(role_id, p))?;
    }
    tx.commit()?;
    get_with_permissions(&conn, role_id)
}

/// 聚合权限后返回。
fn get_with_permissions(conn: &rusqlite::Connection, id: i64) -> Result<RoleWithPermissions> {
    let role = conn.query_row(
        "SELECT id, name, description, is_system, sort_order FROM roles WHERE id=?1",
        [id],
        |row| Ok(Role { id: row.get(0)?, name: row.get(1)?, description: row.get(2)?, is_system: row.get(3)?, sort_order: row.get(4)? }),
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => crate::error::AppError::NotFound("角色不存在".into()),
        _ => e.into(),
    })?;
    let perms = get_permissions_on_conn(conn, id)?;
    Ok(RoleWithPermissions {
        id: role.id, name: role.name, description: role.description, is_system: role.is_system, sort_order: role.sort_order, permissions: perms,
    })
}
