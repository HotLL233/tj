use crate::db::DbPool;
use crate::error::Result;
use crate::models::record::{RecordCreate, RecordResponse, RecordUpdate};
use crate::repo::audit_repo;

pub fn list(
    pool: &DbPool, project_id: Option<i64>, group_id: Option<i64>,
    user_name: Option<&str>,
    start: Option<&str>, end: Option<&str>, page: i64, page_size: i64,
    include_deleted: bool,
) -> Result<(Vec<RecordResponse>, i64)> {
    let conn = pool.get()?;
    let mut where_clauses = vec![];
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];

    if !include_deleted { where_clauses.push("wr.deleted_at IS NULL".to_string()); }
    if let Some(pid) = project_id { where_clauses.push(format!("wr.project_id={}", pid)); }
    if let Some(gid) = group_id { where_clauses.push(format!("EXISTS (SELECT 1 FROM project_lab_links pll WHERE pll.project_id = p.id AND pll.group_id = {})", gid)); }
    if let Some(un) = user_name { where_clauses.push("wr.user_name=?1".to_string()); params.push(Box::new(un.to_string())); }
    if let Some(s) = start { let idx = params.len() + 1; where_clauses.push(format!("wr.recorded_at>=?{}", idx)); params.push(Box::new(s.to_string())); }
    if let Some(e) = end { let idx = params.len() + 1; where_clauses.push(format!("wr.recorded_at<=?{}", idx)); params.push(Box::new(format!("{}T23:59:59", e))); }

    let where_sql = if where_clauses.is_empty() { String::new() } else { format!("WHERE {}", where_clauses.join(" AND ")) };

    // v0.3.18 修复：用 LEFT JOIN 替代 group_concat 子查询
    // 注意：为了保持记录不重复，这里保留 group_concat 用于列表展示
    // 导出 Excel 已在 export_data.rs 中修复为分开统计
    let sql = format!(
        "SELECT wr.id, wr.project_id, wr.method_id, p.name,
                COALESCE((SELECT group_concat(pg.name)
                 FROM project_lab_links pll
                 JOIN project_groups pg ON pll.group_id = pg.id AND pg.name != '研发项目'
                 WHERE pll.project_id = p.id), '未知') AS group_name,
                wr.user_name, wr.quantity,
                wr.recorded_at, wr.created_at, wr.deleted_at,
                COALESCE(NULLIF(m.full_name,''), NULLIF(m.name,'')) AS method_name,
                (SELECT group_concat(DISTINCT mt.name)
                 FROM method_type_links mtl
                 JOIN method_types mt ON mtl.method_type_id = mt.id
                 WHERE mtl.method_id = wr.method_id) AS method_type
         FROM work_records wr
         JOIN projects p ON wr.project_id = p.id
         LEFT JOIN methods m ON wr.method_id = m.id
         {} ORDER BY wr.recorded_at DESC
         LIMIT {} OFFSET {}",
        where_sql, page_size, (page - 1) * page_size
    );

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(
        rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
        |row| Ok(RecordResponse {
            id: row.get(0)?, project_id: row.get(1)?, method_id: row.get(2)?,
            project_name: row.get(3)?, group_name: row.get(4)?, user_name: row.get(5)?,
            quantity: row.get(6)?, recorded_at: row.get(7)?, created_at: row.get(8)?,
            deleted_at: row.get(9)?, method_name: row.get(10)?, method_type: row.get(11)?,
        }),
    )?;
    let items: Vec<RecordResponse> = rows.collect::<std::result::Result<Vec<_>, _>>()?;

    // Get count
    let count: i64 = conn.query_row(
        &format!("SELECT COUNT(DISTINCT wr.id) FROM work_records wr JOIN projects p ON wr.project_id = p.id {}", where_sql),
        rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())),
        |r| r.get(0),
    )?;

    Ok((items, count))
}

/// Internal helper: query a single record on an existing connection.
/// Avoids grabbing a separate pool connection when called inside a transaction.
/// v0.3.18: 保留 group_concat 用于显示（列表界面需要显示所有关联实验室）
fn get_by_id_on_conn(conn: &rusqlite::Connection, id: i64) -> Result<RecordResponse> {
    conn.query_row(
        "SELECT wr.id, wr.project_id, wr.method_id, p.name,
                COALESCE((SELECT group_concat(pg.name)
                 FROM project_lab_links pll
                 JOIN project_groups pg ON pll.group_id = pg.id AND pg.name != '研发项目'
                 WHERE pll.project_id = p.id), '未知') AS group_name,
                wr.user_name, wr.quantity,
                wr.recorded_at, wr.created_at, wr.deleted_at,
                COALESCE(NULLIF(m.full_name,''), NULLIF(m.name,'')) AS method_name,
                (SELECT group_concat(DISTINCT mt.name)
                 FROM method_type_links mtl
                 JOIN method_types mt ON mtl.method_type_id = mt.id
                 WHERE mtl.method_id = wr.method_id) AS method_type
         FROM work_records wr
         JOIN projects p ON wr.project_id = p.id
         LEFT JOIN methods m ON wr.method_id = m.id
         WHERE wr.id=?1", [id],
        |row| Ok(RecordResponse {
            id: row.get(0)?, project_id: row.get(1)?, method_id: row.get(2)?,
            project_name: row.get(3)?, group_name: row.get(4)?, user_name: row.get(5)?,
            quantity: row.get(6)?, recorded_at: row.get(7)?, created_at: row.get(8)?,
            deleted_at: row.get(9)?, method_name: row.get(10)?, method_type: row.get(11)?,
        }),
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => crate::error::AppError::NotFound("记录不存在".into()),
        _ => e.into(),
    })
}

pub fn get_by_id(pool: &DbPool, id: i64) -> Result<RecordResponse> {
    let conn = pool.get()?;
    get_by_id_on_conn(&conn, id)
}

pub fn create(pool: &DbPool, body: &RecordCreate) -> Result<RecordResponse> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO work_records (project_id, method_id, user_name, quantity, recorded_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params!(body.project_id, body.method_id, &body.user_name, body.quantity, &body.recorded_at),
    )?;
    let id = tx.last_insert_rowid();
    audit_repo::log_on_conn(&tx, "create", "work_records", Some(id), &body.user_name, "创建记录")?;
    tx.commit()?;
    get_by_id_on_conn(&conn, id)
}

pub fn update(pool: &DbPool, id: i64, body: &RecordUpdate, user_name: &str) -> Result<RecordResponse> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;

    // Existence + soft-delete check on same connection (no extra pool round-trip)
    let existing = get_by_id_on_conn(&tx, id)?;
    if existing.deleted_at.is_some() {
        return Err(crate::error::AppError::Validation("记录已被删除，无法编辑".into()));
    }

    let mut updated = false;
    if let Some(ref un) = body.user_name {
        let rows = tx.execute("UPDATE work_records SET user_name=?1 WHERE id=?2", (un, id))?;
        if rows == 0 {
            return Err(crate::error::AppError::NotFound("记录不存在".into()));
        }
        updated = true;
    }
    if let Some(q) = body.quantity {
        let rows = tx.execute("UPDATE work_records SET quantity=?1 WHERE id=?2", (q, id))?;
        if rows == 0 {
            return Err(crate::error::AppError::NotFound("记录不存在".into()));
        }
        updated = true;
    }
    if let Some(ref dt) = body.recorded_at {
        let rows = tx.execute("UPDATE work_records SET recorded_at=?1 WHERE id=?2", (dt, id))?;
        if rows == 0 {
            return Err(crate::error::AppError::NotFound("记录不存在".into()));
        }
        updated = true;
    }
    if !updated {
        return Err(crate::error::AppError::Validation("没有需要更新的字段".into()));
    }
    audit_repo::log_on_conn(&tx, "update", "work_records", Some(id), user_name, "编辑记录")?;
    tx.commit()?;
    get_by_id_on_conn(&conn, id)
}

pub fn soft_delete(pool: &DbPool, id: i64, user_name: &str) -> Result<()> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    let deleted: Option<String> = tx.query_row(
        "SELECT deleted_at FROM work_records WHERE id=?1", [id], |r| r.get(0)
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => crate::error::AppError::NotFound("记录不存在".into()),
        _ => e.into(),
    })?;
    if deleted.is_some() { return Err(crate::error::AppError::Validation("记录已被删除".into())); }
    let rows = tx.execute("UPDATE work_records SET deleted_at=datetime('now','localtime') WHERE id=?1", [id])?;
    if rows == 0 {
        return Err(crate::error::AppError::NotFound("记录不存在".into()));
    }
    audit_repo::log_on_conn(&tx, "delete", "work_records", Some(id), user_name, "软删除记录")?;
    tx.commit()?;
    Ok(())
}

pub fn restore(pool: &DbPool, id: i64, user_name: &str) -> Result<RecordResponse> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;

    // Verify record exists and is indeed in a deleted state
    let deleted: Option<String> = tx.query_row(
        "SELECT deleted_at FROM work_records WHERE id=?1", [id], |r| r.get(0)
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => crate::error::AppError::NotFound("记录不存在".into()),
        _ => e.into(),
    })?;
    if deleted.is_none() {
        return Err(crate::error::AppError::Validation("记录未被删除，无需恢复".into()));
    }

    let rows = tx.execute("UPDATE work_records SET deleted_at=NULL WHERE id=?1", [id])?;
    if rows == 0 {
        return Err(crate::error::AppError::NotFound("记录不存在".into()));
    }
    audit_repo::log_on_conn(&tx, "restore", "work_records", Some(id), user_name, "恢复记录")?;
    tx.commit()?;
    get_by_id_on_conn(&conn, id)
}

pub fn delete_by_user(pool: &DbPool, user_name: &str, start: Option<&str>, end: Option<&str>) -> Result<i64> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    let mut sql = "UPDATE work_records SET deleted_at=datetime('now','localtime') WHERE user_name=?1 AND deleted_at IS NULL".to_string();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(user_name.to_string())];
    if let Some(s) = start { let i = params.len()+1; sql.push_str(&format!(" AND recorded_at>=?{}",i)); params.push(Box::new(s.to_string())); }
    if let Some(e) = end { let i = params.len()+1; sql.push_str(&format!(" AND recorded_at<=?{}",i)); params.push(Box::new(format!("{}T23:59:59",e))); }
    let count = tx.execute(&sql, rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())))?;
    audit_repo::log_on_conn(&tx, "batch_delete", "work_records", None, user_name, &format!("批量删除 {} 条", count))?;
    tx.commit()?;
    Ok(count as i64)
}
