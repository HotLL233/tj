use crate::db::DbPool;
use crate::error::{AppError, Result};
use crate::models::role::{
    is_valid_permission, Role, RoleCreate, RolePermissionSet, RoleUpdate, RoleWithPermissions,
};

/// 角色基础信息 SQL
const ROLE_SELECT: &str = "SELECT id, name, description, is_system, sort_order FROM roles";

/// 查询单个角色（不含权限点）
fn get_role_on_conn(conn: &rusqlite::Connection, id: i64) -> Result<Role> {
    conn.query_row(
        &format!("{} WHERE id=?1", ROLE_SELECT),
        [id],
        |row| {
            Ok(Role {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                is_system: row.get(3)?,
                sort_order: row.get(4)?,
            })
        },
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound("角色不存在".into()),
        _ => e.into(),
    })
}

/// 查询单个角色（按名称，供种子匹配）
fn get_role_by_name_on_conn(conn: &rusqlite::Connection, name: &str) -> Result<Option<Role>> {
    let mut stmt = conn.prepare(&format!("{} WHERE name=?1", ROLE_SELECT))?;
    let mut rows = stmt.query_map([name], |row| {
        Ok(Role {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            is_system: row.get(3)?,
            sort_order: row.get(4)?,
        })
    })?;
    match rows.next() {
        Some(Ok(r)) => Ok(Some(r)),
        Some(Err(e)) => Err(e.into()),
        None => Ok(None),
    }
}

/// 查询角色权限点集合
fn get_permissions_on_conn(conn: &rusqlite::Connection, role_id: i64) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT permission_key FROM role_permissions WHERE role_id=?1 ORDER BY permission_key")?;
    let rows = stmt.query_map([role_id], |row| row.get::<_, String>(0))?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

/// 聚合角色 + 权限点
fn with_permissions(conn: &rusqlite::Connection, role: Role) -> Result<RoleWithPermissions> {
    let permissions = get_permissions_on_conn(conn, role.id)?;
    Ok(RoleWithPermissions {
        id: role.id,
        name: role.name,
        description: role.description,
        is_system: role.is_system,
        sort_order: role.sort_order,
        permissions,
    })
}

/// 列出全部角色（含聚合权限点），按 sort_order, id 排序
pub fn list_with_permissions(pool: &DbPool) -> Result<Vec<RoleWithPermissions>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(&format!("{} ORDER BY sort_order, id", ROLE_SELECT))?;
    let roles = stmt.query_map([], |row| {
        Ok(Role {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            is_system: row.get(3)?,
            sort_order: row.get(4)?,
        })
    })?;
    let roles: Vec<Role> = roles.collect::<std::result::Result<Vec<_>, _>>()?;
    let mut out = Vec::with_capacity(roles.len());
    for r in roles {
        out.push(with_permissions(&conn, r)?);
    }
    Ok(out)
}

/// 列出全部角色（仅基础信息），供用户编辑下拉使用
pub fn list_all(pool: &DbPool) -> Result<Vec<Role>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(&format!("{} ORDER BY sort_order, id", ROLE_SELECT))?;
    let roles = stmt.query_map([], |row| {
        Ok(Role {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            is_system: row.get(3)?,
            sort_order: row.get(4)?,
        })
    })?;
    Ok(roles.collect::<std::result::Result<Vec<_>, _>>()?)
}

/// 查询角色权限点
pub fn get_permissions(pool: &DbPool, role_id: i64) -> Result<Vec<String>> {
    let conn = pool.get()?;
    get_permissions_on_conn(&conn, role_id)
}

/// 新建角色并写入权限点（校验权限点合法性）
pub fn create(pool: &DbPool, body: &RoleCreate) -> Result<RoleWithPermissions> {
    let mut conn = pool.get()?;
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM roles WHERE name=?1",
        [&body.name],
        |r| r.get(0),
    )?;
    if exists > 0 {
        return Err(AppError::Conflict(format!("角色名「{}」已存在", body.name)));
    }
    for p in &body.permissions {
        if !is_valid_permission(p) {
            return Err(AppError::Validation(format!("非法权限点: {}", p)));
        }
    }
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO roles (name, description, is_system, sort_order) VALUES (?1,?2,0,?3)",
        rusqlite::params![body.name, body.description, body.sort_order],
    )?;
    let id = tx.last_insert_rowid();
    for p in &body.permissions {
        tx.execute(
            "INSERT INTO role_permissions (role_id, permission_key) VALUES (?1,?2)",
            rusqlite::params![id, p],
        )?;
    }
    tx.commit()?;
    let role = get_role_on_conn(&conn, id)?;
    with_permissions(&conn, role)
}

/// 更新角色基础信息（系统角色不可改名）
pub fn update(pool: &DbPool, id: i64, body: &RoleUpdate) -> Result<RoleWithPermissions> {
    let mut conn = pool.get()?;
    let role = get_role_on_conn(&conn, id)?;
    let mut sets: Vec<String> = vec![];
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
    if let Some(ref n) = body.name {
        if role.is_system == 1 {
            return Err(AppError::Forbidden("系统角色不可改名".into()));
        }
        if !n.is_empty() && n != &role.name {
            // 唯一性校验
            let conflict: i64 = conn.query_row(
                "SELECT COUNT(*) FROM roles WHERE name=?1 AND id<>?2",
                rusqlite::params![n, id],
                |r| r.get(0),
            )?;
            if conflict > 0 {
                return Err(AppError::Conflict(format!("角色名「{}」已存在", n)));
            }
            sets.push("name=?1".to_string());
            params.push(Box::new(n.clone()));
        }
    }
    if let Some(ref d) = body.description {
        sets.push(format!("description=?{}", params.len() + 1));
        params.push(Box::new(d.clone()));
    }
    if let Some(so) = body.sort_order {
        sets.push(format!("sort_order=?{}", params.len() + 1));
        params.push(Box::new(so));
    }
    if sets.is_empty() {
        return Err(AppError::Validation("没有需要更新的字段".into()));
    }
    params.push(Box::new(id));
    let sql = format!(
        "UPDATE roles SET {} WHERE id=?{}",
        sets.join(","),
        params.len()
    );
    conn.execute(&sql, rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())))?;
    let role = get_role_on_conn(&conn, id)?;
    with_permissions(&conn, role)
}

/// 删除角色（系统角色禁止删除；有关联用户时禁止删除）
pub fn delete(pool: &DbPool, id: i64) -> Result<()> {
    let mut conn = pool.get()?;
    let role = get_role_on_conn(&conn, id)?;
    if role.is_system == 1 {
        return Err(AppError::Forbidden("系统角色不可删除".into()));
    }
    let used: i64 = conn.query_row(
        "SELECT
            (SELECT COUNT(*) FROM users WHERE role_id=?1) +
            (SELECT COUNT(*) FROM user_roles WHERE role_id=?1)",
        [id],
        |r| r.get(0),
    )?;
    if used > 0 {
        return Err(AppError::Conflict("该角色下仍有用户，无法删除".into()));
    }
    let tx = conn.transaction()?;
    tx.execute("DELETE FROM role_permissions WHERE role_id=?1", [id])?;
    tx.execute("DELETE FROM roles WHERE id=?1", [id])?;
    tx.commit()?;
    Ok(())
}

/// 整体替换角色权限点
pub fn set_permissions(pool: &DbPool, role_id: i64, body: &RolePermissionSet) -> Result<RoleWithPermissions> {
    let mut conn = pool.get()?;
    // 校验角色存在
    let _ = get_role_on_conn(&conn, role_id)?;
    for p in &body.permissions {
        if !is_valid_permission(p) {
            return Err(AppError::Validation(format!("非法权限点: {}", p)));
        }
    }
    let tx = conn.transaction()?;
    tx.execute("DELETE FROM role_permissions WHERE role_id=?1", [role_id])?;
    for p in &body.permissions {
        tx.execute(
            "INSERT INTO role_permissions (role_id, permission_key) VALUES (?1,?2)",
            rusqlite::params![role_id, p],
        )?;
    }
    tx.commit()?;
    let role = get_role_on_conn(&conn, role_id)?;
    with_permissions(&conn, role)
}
