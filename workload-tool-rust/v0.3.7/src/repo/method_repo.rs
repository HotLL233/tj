use rusqlite::Connection;
use std::collections::BTreeMap;
use crate::db::DbPool;
use crate::error::{Result, AppError};
use crate::repo::audit_repo;
use crate::models::import::ImportMapping;
use crate::models::project::{ImportSummary, TypeCount, MethodImportItem};
use crate::models::method::{MethodResponse, MethodCreate, MethodUpdate};

// ── helpers ──

fn parse_comma_i64(s: &str) -> Vec<i64> {
    if s.is_empty() { return vec![]; }
    s.split(',').filter_map(|x| x.trim().parse().ok()).collect()
}

fn parse_comma_str(s: &str) -> Vec<String> {
    if s.is_empty() { return vec![]; }
    s.split(',').map(|x| x.trim().to_string()).collect()
}

const METHOD_SQL: &str =
    "SELECT m.id, m.name, COALESCE(m.full_name,''), COALESCE(m.coefficient,1.0), \
     COALESCE(m.amount,0.0), \
     COALESCE(m.notes,''), m.is_active, \
     COALESCE(GROUP_CONCAT(DISTINCT mtl.method_type_id), '') as type_ids_str, \
     COALESCE(GROUP_CONCAT(DISTINCT mt.name), '') as type_names_str, \
     m.created_at \
     FROM methods m \
     LEFT JOIN method_type_links mtl ON m.id = mtl.method_id \
     LEFT JOIN method_types mt ON mtl.method_type_id = mt.id";

fn row_to_method(row: &rusqlite::Row) -> rusqlite::Result<MethodResponse> {
    let type_ids_str: String = row.get::<_, String>(7).unwrap_or_default();
    let type_names_str: String = row.get::<_, String>(8).unwrap_or_default();
    Ok(MethodResponse {
        id: row.get(0)?,
        name: row.get(1)?,
        full_name: row.get(2)?,
        coefficient: row.get::<_, f64>(3).unwrap_or(1.0),
        amount: row.get::<_, f64>(4).unwrap_or(0.0),
        notes: row.get(5)?,
        is_active: row.get::<_, i64>(6).unwrap_or(1) != 0,
        type_ids: parse_comma_i64(&type_ids_str),
        type_names: parse_comma_str(&type_names_str),
        created_at: row.get(9)?,
    })
}

// ── CRUD ──

pub fn list(pool: &DbPool, type_filter: Option<i64>) -> Result<Vec<MethodResponse>> {
    let conn = pool.get()?;
    let mut sql = format!("{} GROUP BY m.id", METHOD_SQL);
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
    if let Some(tid) = type_filter {
        if tid > 0 {
            sql.push_str(" HAVING ',' || type_ids_str || ',' LIKE ?1");
            params.push(Box::new(format!("%,{},%", tid)));
        }
    }
    sql.push_str(" ORDER BY m.id");
    let mut stmt = conn.prepare(&sql)?;
    let refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let rows: Vec<MethodResponse> = stmt.query_map(
        rusqlite::params_from_iter(refs.iter()),
        |row| row_to_method(row),
    )?.collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn get_by_id(pool: &DbPool, id: i64) -> Result<MethodResponse> {
    let conn = pool.get()?;
    let sql = format!("{} WHERE m.id=?1 GROUP BY m.id", METHOD_SQL);
    conn.query_row(&sql, [id], |row| row_to_method(row))
        .map_err(|e| AppError::NotFound(format!("方法不存在: {}", e)))
}

pub fn create(pool: &DbPool, body: &MethodCreate) -> Result<MethodResponse> {
    let conn = pool.get()?;
    let fn_ = body.full_name.as_deref().unwrap_or("");
    let cf = body.coefficient.unwrap_or(1.0);
    let amt = body.amount.unwrap_or(0.0);
    let nt = body.notes.as_deref().unwrap_or("");
    conn.execute(
        "INSERT INTO methods (name, full_name, coefficient, amount, notes) VALUES (?1,?2,?3,?4,?5)",
        rusqlite::params![body.name, fn_, cf, amt, nt],
    )?;
    let mid = conn.last_insert_rowid();

    // Insert method_type_links
    if let Some(ref type_ids) = body.type_ids {
        for tid in type_ids {
            conn.execute(
                "INSERT OR IGNORE INTO method_type_links (method_id, method_type_id) VALUES (?1,?2)",
                rusqlite::params![mid, tid],
            )?;
        }
    }

    audit_repo::log(pool, "create", "methods", Some(mid), "system", &format!("创建方法: {}", body.name))?;

    get_by_id(pool, mid)
}

pub fn update(pool: &DbPool, id: i64, body: &MethodUpdate) -> Result<MethodResponse> {
    let conn = pool.get()?;
    if let Some(ref n) = body.name {
        conn.execute("UPDATE methods SET name=?1 WHERE id=?2", (n, id))?;
    }
    if let Some(ref f) = body.full_name {
        conn.execute("UPDATE methods SET full_name=?1 WHERE id=?2", (f, id))?;
    }
    if let Some(c) = body.coefficient {
        conn.execute("UPDATE methods SET coefficient=?1 WHERE id=?2", (c, id))?;
    }
    if let Some(a) = body.amount {
        conn.execute("UPDATE methods SET amount=?1 WHERE id=?2", (a, id))?;
    }
    if let Some(ref n) = body.notes {
        conn.execute("UPDATE methods SET notes=?1 WHERE id=?2", (n, id))?;
    }
    if let Some(a) = body.is_active {
        conn.execute("UPDATE methods SET is_active=?1 WHERE id=?2", (a as i64, id))?;
    }
    // Replace type links
    if let Some(ref type_ids) = body.type_ids {
        conn.execute("DELETE FROM method_type_links WHERE method_id=?1", [id])?;
        for tid in type_ids {
            conn.execute(
                "INSERT OR IGNORE INTO method_type_links (method_id, method_type_id) VALUES (?1,?2)",
                rusqlite::params![id, tid],
            )?;
        }
    }
    audit_repo::log(pool, "update", "methods", Some(id), "system", &format!("更新方法: {}", body.name.as_deref().unwrap_or("")))?;
    get_by_id(pool, id)
}

pub fn delete(pool: &DbPool, id: i64) -> Result<()> {
    let conn = pool.get()?;
    // Check if any project references this method
    let used: i64 = conn.query_row(
        "SELECT COUNT(*) FROM project_method_links WHERE method_id=?1", [id], |r| r.get(0),
    )?;
    if used > 0 {
        return Err(AppError::Validation(format!("该方法被{}个项目关联，无法删除", used)));
    }
    audit_repo::log(pool, "delete", "methods", Some(id), "system", "删除方法")?;
    conn.execute("DELETE FROM method_type_links WHERE method_id=?1", [id])?;
    conn.execute("DELETE FROM methods WHERE id=?1", [id])?;
    Ok(())
}

// ── 方法类型 (method_types 表) ──

pub fn list_method_types(pool: &DbPool) -> Result<Vec<crate::models::project::MethodType>> {
    use crate::models::project::MethodType;
    let conn = pool.get()?;
    let mut stmt = conn.prepare("SELECT id, name, sort_order FROM method_types ORDER BY sort_order")?;
    let rows: Vec<MethodType> = stmt.query_map([], |row| {
        Ok(MethodType { id: row.get(0)?, name: row.get(1)?, sort_order: row.get(2)? })
    })?.collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn create_method_type(pool: &DbPool, body: &crate::models::project::MethodTypeCreate) -> Result<crate::models::project::MethodType> {
    use crate::models::project::MethodType;
    let conn = pool.get()?;
    let so = body.sort_order.unwrap_or(10);
    conn.execute("INSERT INTO method_types (name, sort_order) VALUES (?1,?2)", rusqlite::params![body.name, so])?;
    Ok(MethodType { id: conn.last_insert_rowid(), name: body.name.clone(), sort_order: so })
}

pub fn update_method_type(pool: &DbPool, id: i64, body: &crate::models::project::MethodTypeUpdate) -> Result<crate::models::project::MethodType> {
    use crate::models::project::MethodType;
    let conn = pool.get()?;
    if let Some(ref n) = body.name { conn.execute("UPDATE method_types SET name=?1 WHERE id=?2", (n, id))?; }
    if let Some(s) = body.sort_order { conn.execute("UPDATE method_types SET sort_order=?1 WHERE id=?2", (s, id))?; }
    let mt = conn.query_row("SELECT id, name, sort_order FROM method_types WHERE id=?", [id], |r| {
        Ok(MethodType { id: r.get(0)?, name: r.get(1)?, sort_order: r.get(2)? })
    })?;
    Ok(mt)
}

pub fn delete_method_type(pool: &DbPool, id: i64) -> Result<()> {
    let conn = pool.get()?;
    let mt_name: String = conn.query_row("SELECT name FROM method_types WHERE id=?", [id], |r| r.get(0))?;
    let used: i64 = conn.query_row(
        "SELECT COUNT(*) FROM method_type_links WHERE method_type_id=?", [id], |r| r.get(0),
    )?;
    if used > 0 { return Err(AppError::Validation(format!("类型'{}'被{}个方法使用", mt_name, used))); }
    conn.execute("DELETE FROM method_types WHERE id=?", [id])?;
    Ok(())
}

// ── 导入映射配置 (v0.3.0) ──

/// 从连接加载活跃的导入映射（按优先级排序）
pub fn load_mappings_from_conn(conn: &Connection) -> Result<Vec<ImportMapping>> {
    let mut stmt = conn.prepare(
        "SELECT id, header_pattern, match_mode, target_table, COALESCE(default_type,''), priority, is_active
         FROM import_mappings WHERE is_active=1 ORDER BY priority ASC"
    )?;
    let items = stmt.query_map([], |r| {
        Ok(ImportMapping {
            id: r.get(0)?,
            header_pattern: r.get(1)?,
            match_mode: r.get(2)?,
            target_table: r.get(3)?,
            default_type: r.get(4)?,
            priority: r.get(5)?,
            is_active: r.get::<_, i64>(6)? != 0,
        })
    })?.filter_map(|r| r.ok()).collect();
    Ok(items)
}

/// 从连接池加载活跃的导入映射
pub fn load_mappings(pool: &DbPool) -> Result<Vec<ImportMapping>> {
    let conn = pool.get()?;
    load_mappings_from_conn(&conn)
}

// ── 导入 ──

/// v0.2.17: 按列导入 — 分三路：实验室→project_groups, 研发项目→projects, 方法→methods
pub fn batch_import_column_split(
    conn: &Connection,
    group_names: &[String],
    project_names: &[String],
    method_items: &[(String, String, String)],
) -> Result<ImportSummary> {
    let mut group_count = 0usize;
    let mut project_count = 0usize;
    let mut method_count = 0usize;
    let mut type_counter: BTreeMap<String, usize> = BTreeMap::new();

    // 1. 实验室分组
    for gname in group_names {
        let existed: i64 = conn.query_row(
            "SELECT COUNT(*) FROM project_groups WHERE name=?1",
            rusqlite::params![gname], |r| r.get(0),
        ).unwrap_or(0);
        conn.execute("INSERT OR IGNORE INTO project_groups (name) VALUES (?1)",
            rusqlite::params![gname])?;
        if existed == 0 { group_count += 1; }
    }

    // 2. 研发项目 → projects 表
    // 确保"研发项目"分组存在
    conn.execute("INSERT OR IGNORE INTO project_groups (name) VALUES ('研发项目')", [])?;
    let proj_gid: i64 = conn.query_row(
        "SELECT id FROM project_groups WHERE name='研发项目'",
        [], |r| r.get(0),
    )?;

    for pname in project_names {
        // v0.3.3: 按项目名查重（不限制 group_id，因为不同项目可能关联不同实验室）
        let existing: Option<i64> = conn.query_row(
            "SELECT id FROM projects WHERE name=?1",
            rusqlite::params![pname], |r| r.get(0),
        ).ok();
        if existing.is_none() {
            conn.execute(
                "INSERT INTO projects (group_id, name, method_type) VALUES (?1,?2,'研发项目')",
                rusqlite::params![proj_gid, pname],
            )?;
            let pid = conn.last_insert_rowid();
            project_count += 1;
        }
    }

    // 3. 方法 → methods 表（不创建 project_groups，避免污染实验室标签）
    // v0.3.4: 智能识别 — 若列头含"方法"，从列头提取真实类型名并自动创建 method_type
    for (header, item_name, method_type) in method_items {

        // v0.3.4: 智能提取类型名
        let effective_type: String = if header.contains("方法") {
            let extracted = header.replace("方法", "").trim().to_string();
            if !extracted.is_empty() {
                // 检查并自动创建 method_type（不存在则创建，sort_order=10）
                let mt_exists: Option<i64> = conn.query_row(
                    "SELECT id FROM method_types WHERE name=?1",
                    rusqlite::params![extracted], |r| r.get(0),
                ).ok();
                if mt_exists.is_none() {
                    conn.execute(
                        "INSERT INTO method_types (name, sort_order) VALUES (?1, 10)",
                        rusqlite::params![extracted],
                    )?;
                }
                extracted
            } else {
                method_type.clone()
            }
        } else {
            method_type.clone()
        };

        // Insert method（去重：已存在则跳过）
        let existing: Option<i64> = conn.query_row(
            "SELECT id FROM methods WHERE name=?1",
            rusqlite::params![item_name], |r| r.get(0),
        ).ok();

        let mid = if let Some(pid) = existing {
            pid
        } else {
            conn.execute("INSERT INTO methods (name) VALUES (?1)",
                rusqlite::params![item_name])?;
            method_count += 1;
            conn.last_insert_rowid()
        };

        // 关联类型
        if !effective_type.is_empty() {
            let mt_id: Option<i64> = conn.query_row(
                "SELECT id FROM method_types WHERE name=?1",
                rusqlite::params![effective_type], |r| r.get(0),
            ).ok();
            if let Some(tid) = mt_id {
                conn.execute(
                    "INSERT OR IGNORE INTO method_type_links (method_id, method_type_id) VALUES (?1,?2)",
                    rusqlite::params![mid, tid],
                )?;
            }
        }
        *type_counter.entry(effective_type).or_insert(0) += 1;
    }

    // 审计日志
    if method_count > 0 {
        crate::repo::audit_repo::log_on_conn(conn, "import", "methods", None, "system", &format!("批量导入: {}条方法", method_count)).ok();
    }
    if project_count > 0 {
        crate::repo::audit_repo::log_on_conn(conn, "import", "projects", None, "system", &format!("批量导入: {}个研发项目", project_count)).ok();
    }
    if group_count > 0 {
        crate::repo::audit_repo::log_on_conn(conn, "import", "project_groups", None, "system", &format!("批量导入: {}个实验室分组", group_count)).ok();
    }

    Ok(ImportSummary {
        total_methods: method_count,
        total_projects: project_count,
        total_groups: group_count,
        by_type: type_counter.into_iter()
            .map(|(k, v)| TypeCount { method_type: k, count: v })
            .collect(),
    })
}

/// v0.2.17: 扁平导入
pub fn batch_import_flat(conn: &Connection, items: &[MethodImportItem]) -> Result<ImportSummary> {
    let mut method_count = 0usize;
    let mut project_count = 0usize;
    let mut group_count = 0usize;
    let mut type_counter: BTreeMap<String, usize> = BTreeMap::new();

    for item in items {
        // 1. 实验室 → project_groups
        conn.execute("INSERT OR IGNORE INTO project_groups (name) VALUES (?1)", rusqlite::params![item.group_name]).ok();
        let gid: i64 = conn.query_row("SELECT id FROM project_groups WHERE name=?1", rusqlite::params![item.group_name], |r| r.get(0))?;

        // 2. 研发项目 → projects (method_type='研发项目')
        let existing_proj: Option<i64> = conn.query_row(
            "SELECT id FROM projects WHERE name=?1 AND method_type='研发项目'",
            rusqlite::params![item.project_name], |r| r.get(0)).ok();
        let proj_id = if existing_proj.is_none() {
            conn.execute("INSERT INTO projects (group_id, name, method_type) VALUES (?1,?2,'研发项目')",
                rusqlite::params![gid, item.project_name])?;
            let pid = conn.last_insert_rowid();
            // Link project → lab
            conn.execute("INSERT OR IGNORE INTO project_lab_links (project_id, group_id) VALUES (?1,?2)",
                rusqlite::params![pid, gid])?;
            project_count += 1;
            pid
        } else {
            let pid = existing_proj.unwrap();
            // Ensure link
            conn.execute("INSERT OR IGNORE INTO project_lab_links (project_id, group_id) VALUES (?1,?2)",
                rusqlite::params![pid, gid])?;
            pid
        };

        // 3. 方法 → methods
        let full_name = format!("{}/{}", item.group_name, item.project_name);
        let existing: Option<i64> = conn.query_row(
            "SELECT id FROM methods WHERE name=?1",
            rusqlite::params![item.method_name], |r| r.get(0)).ok();
        let mid = if let Some(pid) = existing {
            conn.execute("UPDATE methods SET full_name=?1, coefficient=?2 WHERE id=?3",
                rusqlite::params![full_name, item.coefficient, pid])?;
            pid
        } else {
            conn.execute("INSERT INTO methods (name, full_name, coefficient) VALUES (?1,?2,?3)",
                rusqlite::params![item.method_name, full_name, item.coefficient])?;
            method_count += 1;
            conn.last_insert_rowid()
        };

        // Link method → type
        if !item.method_type.is_empty() {
            let mt_id: Option<i64> = conn.query_row(
                "SELECT id FROM method_types WHERE name=?1",
                rusqlite::params![item.method_type], |r| r.get(0),
            ).ok();
            if let Some(tid) = mt_id {
                conn.execute(
                    "INSERT OR IGNORE INTO method_type_links (method_id, method_type_id) VALUES (?1,?2)",
                    rusqlite::params![mid, tid],
                )?;
            }
        }

        // Link project → method
        conn.execute(
            "INSERT OR IGNORE INTO project_method_links (project_id, method_id) VALUES (?1,?2)",
            rusqlite::params![proj_id, mid],
        )?;

        *type_counter.entry(item.method_type.clone()).or_insert(0) += 1;
        group_count += 1;
    }

    Ok(ImportSummary {
        total_methods: method_count,
        total_projects: project_count,
        total_groups: group_count,
        by_type: type_counter.into_iter().map(|(k, v)| TypeCount { method_type: k, count: v }).collect(),
    })
}
