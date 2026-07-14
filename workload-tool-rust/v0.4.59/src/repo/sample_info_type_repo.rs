use crate::db::DbPool;
use crate::error::{AppError, Result};
use crate::models::sample_info_type::{
    SampleInfoType, SampleInfoTypeCreate, SampleInfoTypeUpdate,
};
use crate::repo::audit_repo;

fn row_to_type(row: &rusqlite::Row) -> rusqlite::Result<SampleInfoType> {
    Ok(SampleInfoType {
        id: row.get(0)?,
        type_key: row.get(1)?,
        label: row.get(2)?,
        description: row.get::<_, String>(3).unwrap_or_default(),
        color: row.get::<_, String>(4).unwrap_or_else(|_| "#2e7d32".to_string()),
        sort_order: row.get::<_, i64>(5).unwrap_or(0),
        is_active: row.get::<_, i64>(6).unwrap_or(1),
        created_at: row.get(7)?,
    })
}

/// 列表（仅启用 is_active=1），供门户使用
pub fn list(pool: &DbPool) -> Result<Vec<SampleInfoType>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT id, type_key, label, description, color, sort_order, is_active, created_at \
         FROM sample_info_types WHERE is_active=1 ORDER BY sort_order ASC, id ASC",
    )?;
    let rows = stmt.query_map([], |row| row_to_type(row))?;
    let items = rows.collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(items)
}

/// 列表（含软删 is_active=0），供管理页使用
pub fn list_all(pool: &DbPool) -> Result<Vec<SampleInfoType>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT id, type_key, label, description, color, sort_order, is_active, created_at \
         FROM sample_info_types ORDER BY sort_order ASC, id ASC",
    )?;
    let rows = stmt.query_map([], |row| row_to_type(row))?;
    let items = rows.collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(items)
}

/// 创建（事务 + 审计）；type_key 唯一冲突返回 Conflict
pub fn create(pool: &DbPool, data: &SampleInfoTypeCreate) -> Result<SampleInfoType> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;

    let label = data.label.trim().to_string();
    let type_key = data.type_key.trim().to_string();
    if type_key.is_empty() || label.is_empty() {
        return Err(AppError::Validation("type_key 与 label 不能为空".into()));
    }

    // 唯一性校验
    let exists: i64 = tx.query_row(
        "SELECT COUNT(*) FROM sample_info_types WHERE type_key=?1",
        [&type_key],
        |r| r.get(0),
    )?;
    if exists > 0 {
        return Err(AppError::Conflict(format!("类型标识「{}」已存在", type_key)));
    }

    tx.execute(
        "INSERT INTO sample_info_types (type_key, label, description, color, sort_order) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            type_key,
            label,
            data.description.clone().unwrap_or_default(),
            data.color.clone().unwrap_or_else(|| "#2e7d32".to_string()),
            data.sort_order.unwrap_or(0),
        ],
    )?;

    let id = tx.last_insert_rowid();
    let detail = format!("创建检测类型#{}：{}（{}）", id, label, type_key);
    audit_repo::log_on_conn_with_module(
        &tx, "create", "sample_info_types", Some(id),
        "system", &detail, "sample_info",
    )?;
    tx.commit()?;

    let item = get_by_id(&conn, id)?;
    Ok(item)
}

/// 更新（事务 + 审计）
pub fn update(pool: &DbPool, id: i64, data: &SampleInfoTypeUpdate) -> Result<SampleInfoType> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    let existing = get_by_id(&tx, id)?;

    let mut changes: Vec<String> = vec![];

    if let Some(ref tk) = data.type_key {
        let tk = tk.trim().to_string();
        if !tk.is_empty() && tk != existing.type_key {
            // 唯一性校验（排除自身）
            let conflict: i64 = tx.query_row(
                "SELECT COUNT(*) FROM sample_info_types WHERE type_key=?1 AND id<>?2",
                rusqlite::params![tk, id],
                |r| r.get(0),
            )?;
            if conflict > 0 {
                return Err(AppError::Conflict(format!("类型标识「{}」已存在", tk)));
            }
            changes.push(format!("标识 {} → {}", existing.type_key, tk));
            tx.execute("UPDATE sample_info_types SET type_key=?1 WHERE id=?2", rusqlite::params![tk, id])?;
        }
    }
    if let Some(ref l) = data.label {
        let l = l.trim().to_string();
        if !l.is_empty() && l != existing.label {
            changes.push(format!("名称 {} → {}", existing.label, l));
            tx.execute("UPDATE sample_info_types SET label=?1 WHERE id=?2", rusqlite::params![l, id])?;
        }
    }
    if let Some(ref d) = data.description {
        if d != &existing.description {
            tx.execute("UPDATE sample_info_types SET description=?1 WHERE id=?2", rusqlite::params![d, id])?;
            changes.push("描述已更新".into());
        }
    }
    if let Some(ref c) = data.color {
        if c != &existing.color {
            tx.execute("UPDATE sample_info_types SET color=?1 WHERE id=?2", rusqlite::params![c, id])?;
            changes.push("颜色已更新".into());
        }
    }
    if let Some(so) = data.sort_order {
        if so != existing.sort_order {
            tx.execute("UPDATE sample_info_types SET sort_order=?1 WHERE id=?2", rusqlite::params![so, id])?;
            changes.push(format!("排序 {} → {}", existing.sort_order, so));
        }
    }
    if let Some(ia) = data.is_active {
        if ia != existing.is_active {
            tx.execute("UPDATE sample_info_types SET is_active=?1 WHERE id=?2", rusqlite::params![ia, id])?;
            changes.push(format!("启用 {}", ia));
        }
    }

    if changes.is_empty() {
        drop(tx);
    } else {
        let detail = format!("修改检测类型#{}：{}", id, changes.join("，"));
        audit_repo::log_on_conn_with_module(
            &tx, "update", "sample_info_types", Some(id),
            "system", &detail, "sample_info",
        )?;
        tx.commit()?;
    }

    let item = get_by_id(&conn, id)?;
    Ok(item)
}

/// 软删除：is_active=0（事务 + 审计）
pub fn soft_delete(pool: &DbPool, id: i64) -> Result<()> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    let existing = get_by_id(&tx, id)?;

    tx.execute("UPDATE sample_info_types SET is_active=0 WHERE id=?1", [id])?;
    let detail = format!("删除检测类型#{}：{}（{}）", id, existing.label, existing.type_key);
    audit_repo::log_on_conn_with_module(
        &tx, "delete", "sample_info_types", Some(id),
        "system", &detail, "sample_info",
    )?;
    tx.commit()?;
    Ok(())
}

/// 永久删除：DELETE FROM（仅从回收站）
pub fn permanent_delete(pool: &DbPool, id: i64) -> Result<()> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    tx.execute("DELETE FROM sample_info_column_visibility WHERE type_key IN (SELECT type_key FROM sample_info_types WHERE id=?1)", [id])?;
    tx.execute("DELETE FROM sample_info_types WHERE id=?1", [id])?;
    let detail = format!("永久删除检测类型#{}", id);
    audit_repo::log_on_conn_with_module(
        &tx, "delete", "sample_info_types", Some(id),
        "system", &detail, "sample_info",
    )?;
    tx.commit()?;
    Ok(())
}

fn get_by_id(conn: &rusqlite::Connection, id: i64) -> Result<SampleInfoType> {
    conn.query_row(
        "SELECT id, type_key, label, description, color, sort_order, is_active, created_at \
         FROM sample_info_types WHERE id=?1",
        [id],
        |row| row_to_type(row),
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound("检测类型不存在".into()),
        _ => e.into(),
    })
}
