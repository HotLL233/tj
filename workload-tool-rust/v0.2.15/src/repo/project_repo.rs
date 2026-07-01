use rusqlite::Connection;
use std::collections::BTreeMap;
use crate::db::DbPool;
use crate::error::{Result, AppError};
use crate::models::project::*;

pub fn row_to_project(row: &rusqlite::Row) -> rusqlite::Result<ProjectResponse> {
    Ok(ProjectResponse {
        id: row.get(0)?, group_id: row.get(1)?, group_name: row.get(2)?, name: row.get(3)?, full_name: row.get(4)?,
        notes: row.get(5)?, sort_order: row.get(6)?, is_active: row.get::<_, i64>(7).unwrap_or(1) != 0,
        coefficient: row.get::<_, f64>(8).unwrap_or(1.0), method_type: row.get(9)?, parent_id: 0,
        methods: vec![], created_at: row.get(10)?,
    })
}

const PROJ_SQL: &str =
    "SELECT p.id, p.group_id, pg.name, p.name, COALESCE(p.full_name,''), COALESCE(p.notes,''), \
     p.sort_order, p.is_active, COALESCE(p.coefficient,1.0), p.method_type, \
     COALESCE(p.created_at,'') \
     FROM projects p JOIN project_groups pg ON p.group_id=pg.id WHERE 1=1";

pub fn list(pool: &DbPool, group_id: Option<i64>, active_only: bool, method_type: Option<&str>) -> Result<Vec<ProjectResponse>> {
    let conn = pool.get()?;
    let mut sql = PROJ_SQL.to_string();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
    if let Some(gid) = group_id { if gid > 0 { sql.push_str(" AND p.group_id=?"); params.push(Box::new(gid)); } }
    if active_only { sql.push_str(" AND p.is_active=1"); }
    if let Some(mt) = method_type { if mt != "全部" && !mt.is_empty() { sql.push_str(" AND p.method_type=?"); params.push(Box::new(mt.to_string())); } }
    sql.push_str(" ORDER BY pg.sort_order, p.sort_order");
    let mut stmt = conn.prepare(&sql)?;
    let refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let rows: Vec<ProjectResponse> = stmt.query_map(rusqlite::params_from_iter(refs.iter()), |row| row_to_project(row))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn get_by_id(pool: &DbPool, id: i64) -> Result<ProjectResponse> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(&format!("{} AND p.id=?", PROJ_SQL))?;
    stmt.query_row([id], |row| row_to_project(row)).map_err(|e| AppError::NotFound(format!("项目不存在: {}", e)))
}

pub fn create(pool: &DbPool, body: &ProjectCreate) -> Result<ProjectResponse> {
    let conn = pool.get()?;
    let so = body.sort_order.unwrap_or(0);
    let cf = body.coefficient.unwrap_or(1.0);
    let mt = body.method_type.as_deref().unwrap_or("其他");
    conn.execute("INSERT INTO projects (group_id, name, sort_order, coefficient, method_type) VALUES (?1,?2,?3,?4,?5)",
        rusqlite::params![body.group_id, body.name, so, cf, mt])?;
    get_by_id(pool, conn.last_insert_rowid())
}

/// v0.2.15: 按列导入 — 每列独立，列头=分组名，列值=条目
/// items: Vec<(group_name, item_name, method_type)>
/// item_name 为空 → 只建分组，不建项目
pub fn batch_import_by_column(
    conn: &Connection,
    items: &[(String, String, String)],
) -> Result<ImportSummary> {
    let mut group_count = 0usize;
    let mut project_count = 0usize;
    let mut method_count = 0usize;
    let mut type_counter: BTreeMap<String, usize> = BTreeMap::new();

    for (group_name, item_name, method_type) in items {
        let existed: i64 = conn.query_row(
            "SELECT COUNT(*) FROM project_groups WHERE name=?1",
            rusqlite::params![group_name], |r| r.get(0),
        ).unwrap_or(0);
        conn.execute("INSERT OR IGNORE INTO project_groups (name) VALUES (?1)",
            rusqlite::params![group_name])?;
        if existed == 0 { group_count += 1; }

        let gid: i64 = conn.query_row(
            "SELECT id FROM project_groups WHERE name=?1",
            rusqlite::params![group_name], |r| r.get(0),
        )?;

        if item_name.is_empty() { continue; }

        let existing: Option<i64> = conn.query_row(
            "SELECT id FROM projects WHERE name=?1 AND group_id=?2",
            rusqlite::params![item_name, gid], |r| r.get(0),
        ).ok();

        if existing.is_none() {
            conn.execute(
                "INSERT INTO projects (group_id, name, method_type) VALUES (?1,?2,?3)",
                rusqlite::params![gid, item_name, method_type],
            )?;
            project_count += 1;
            method_count += 1;
        }
        *type_counter.entry(method_type.clone()).or_insert(0) += 1;
    }

    Ok(ImportSummary {
        total_methods: method_count,
        total_projects: project_count,
        total_groups: group_count,
        by_type: type_counter.into_iter()
            .map(|(k, v)| TypeCount { method_type: k, count: v }).collect(),
    })
}

/// v0.2.13: 按列名精准分类导入(已按实验室+研发项目+方法名去重)
/// grouped: { (实验室, 研发项目): [(类型, 方法名), ...] }
/// 实验室/研发项目独立存, 方法按类型存(去重)
pub fn batch_import_card_classified(
    conn: &Connection,
    grouped: &BTreeMap<(String, String), Vec<(String, String)>>,
) -> Result<ImportSummary> {
    let mut group_count = 0usize;
    let mut project_count = 0usize;
    let mut method_count = 0usize;
    let mut type_counter: BTreeMap<String, usize> = BTreeMap::new();

    for ((lab, proj), methods) in grouped {
        // 1. 实验室
        conn.execute("INSERT OR IGNORE INTO project_groups (name) VALUES (?1)", rusqlite::params![lab]).ok();
        let gid: i64 = conn.query_row("SELECT id FROM project_groups WHERE name=?1", rusqlite::params![lab], |r| r.get(0))?;
        group_count += 1;

        // 2. 研发项目(独立存为 projects, method_type='研发项目')
        let existing_proj: Option<i64> = conn.query_row(
            "SELECT id FROM projects WHERE name=?1 AND group_id=?2 AND method_type='研发项目'",
            rusqlite::params![proj, gid], |r| r.get(0)).ok();
        if existing_proj.is_none() {
            conn.execute("INSERT INTO projects (group_id, name, method_type) VALUES (?1,?2,'研发项目')",
                rusqlite::params![gid, proj])?;
            project_count += 1;
        }

        // 3. 方法(按类型, 唯一约束 name+group_id+method_type)
        for (mtype, mname) in methods {
            let existing: Option<i64> = conn.query_row(
                "SELECT id FROM projects WHERE name=?1 AND group_id=?2 AND method_type=?3",
                rusqlite::params![mname, gid, mtype], |r| r.get(0)).ok();
            if existing.is_none() {
                conn.execute("INSERT INTO projects (group_id, name, method_type, full_name) VALUES (?1,?2,?3,?4)",
                    rusqlite::params![gid, mname, mtype, format!("{}/{}", lab, proj)])?;
                method_count += 1;
            }
        }
        // 统计各类型数量
        for (mtype, _) in methods {
            *type_counter.entry(mtype.clone()).or_insert(0) += 1;
        }
    }

    Ok(ImportSummary {
        total_methods: method_count,
        total_projects: project_count,
        total_groups: group_count,
        by_type: type_counter.into_iter().map(|(k, v)| TypeCount { method_type: k, count: v }).collect(),
    })
}

/// 扁平导入: 按列名精准分类存储
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
            "SELECT id FROM projects WHERE name=?1 AND group_id=?2 AND method_type='研发项目'",
            rusqlite::params![item.project_name, gid], |r| r.get(0)).ok();
        if existing_proj.is_none() {
            conn.execute("INSERT INTO projects (group_id, name, method_type) VALUES (?1,?2,'研发项目')",
                rusqlite::params![gid, item.project_name])?;
            project_count += 1;
        }

        // 3. 方法 → projects (独立存储, full_name 记录来源)
        let full_name = format!("{}/{}", item.group_name, item.project_name);
        let existing: Option<i64> = conn.query_row(
            "SELECT id FROM projects WHERE name=?1 AND group_id=?2 AND method_type=?3",
            rusqlite::params![item.method_name, gid, item.method_type], |r| r.get(0)).ok();
        if let Some(pid) = existing {
            conn.execute("UPDATE projects SET full_name=?1, coefficient=?2 WHERE id=?3",
                rusqlite::params![full_name, item.coefficient, pid])?;
        } else {
            conn.execute("INSERT INTO projects (group_id, name, full_name, method_type, coefficient) VALUES (?1,?2,?3,?4,?5)",
                rusqlite::params![gid, item.method_name, full_name, item.method_type, item.coefficient])?;
        }
        method_count += 1;
        *type_counter.entry(item.method_type.clone()).or_insert(0) += 1;
    }

    Ok(ImportSummary {
        total_methods: method_count, total_projects: project_count, total_groups: group_count,
        by_type: type_counter.into_iter().map(|(k, v)| TypeCount { method_type: k, count: v }).collect(),
    })
}

pub fn update(pool: &DbPool, id: i64, body: &ProjectUpdate) -> Result<ProjectResponse> {
    let conn = pool.get()?;
    if let Some(ref n) = body.name { conn.execute("UPDATE projects SET name=?1 WHERE id=?2", (n, id))?; }
    if let Some(ref f) = body.full_name { conn.execute("UPDATE projects SET full_name=?1 WHERE id=?2", (f, id))?; }
    if let Some(ref n) = body.notes { conn.execute("UPDATE projects SET notes=?1 WHERE id=?2", (n, id))?; }
    if let Some(s) = body.sort_order { conn.execute("UPDATE projects SET sort_order=?1 WHERE id=?2", (s, id))?; }
    if let Some(a) = body.is_active { conn.execute("UPDATE projects SET is_active=?1 WHERE id=?2", (a as i64, id))?; }
    if let Some(c) = body.coefficient { conn.execute("UPDATE projects SET coefficient=?1 WHERE id=?2", (c, id))?; }
    if let Some(ref mt) = body.method_type { conn.execute("UPDATE projects SET method_type=?1 WHERE id=?2", (mt, id))?; }
    get_by_id(pool, id)
}

pub fn batch_coefficient(pool: &DbPool, group_id: i64, coefficient: f64) -> Result<i64> {
    let conn = pool.get()?;
    let count = conn.execute("UPDATE projects SET coefficient=?1 WHERE group_id=?2", rusqlite::params![coefficient, group_id])?;
    Ok(count as i64)
}

pub fn delete(pool: &DbPool, id: i64) -> Result<()> {
    let conn = pool.get()?;
    let wr_count: i64 = conn.query_row("SELECT COUNT(*) FROM work_records WHERE project_id=?1 AND deleted_at IS NULL", [id], |r| r.get(0))?;
    if wr_count > 0 { return Err(AppError::Validation(format!("有{}条工作记录，无法删除", wr_count))); }
    conn.execute("DELETE FROM projects WHERE id=?1", [id])?;
    Ok(())
}

// ── 方法类型 ──

pub fn list_method_types(pool: &DbPool) -> Result<Vec<MethodType>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare("SELECT id, name, sort_order FROM method_types ORDER BY sort_order")?;
    let rows: Vec<MethodType> = stmt.query_map([], |row| Ok(MethodType { id: row.get(0)?, name: row.get(1)?, sort_order: row.get(2)? }))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn create_method_type(pool: &DbPool, body: &MethodTypeCreate) -> Result<MethodType> {
    let conn = pool.get()?;
    let so = body.sort_order.unwrap_or(10);
    conn.execute("INSERT INTO method_types (name, sort_order) VALUES (?1,?2)", rusqlite::params![body.name, so])?;
    Ok(MethodType { id: conn.last_insert_rowid(), name: body.name.clone(), sort_order: so })
}

pub fn update_method_type(pool: &DbPool, id: i64, body: &MethodTypeUpdate) -> Result<MethodType> {
    let conn = pool.get()?;
    if let Some(ref n) = body.name { conn.execute("UPDATE method_types SET name=?1 WHERE id=?2", (n, id))?; }
    if let Some(s) = body.sort_order { conn.execute("UPDATE method_types SET sort_order=?1 WHERE id=?2", (s, id))?; }
    let mt = conn.query_row("SELECT id, name, sort_order FROM method_types WHERE id=?", [id], |r|
        Ok(MethodType { id: r.get(0)?, name: r.get(1)?, sort_order: r.get(2)? }))?;
    Ok(mt)
}

pub fn delete_method_type(pool: &DbPool, id: i64) -> Result<()> {
    let conn = pool.get()?;
    let mt_name: String = conn.query_row("SELECT name FROM method_types WHERE id=?", [id], |r| r.get(0))?;
    let used: i64 = conn.query_row("SELECT COUNT(*) FROM projects WHERE method_type=?", [&mt_name], |r| r.get(0))?;
    if used > 0 { return Err(AppError::Validation(format!("类型'{}'被{}个项目使用", mt_name, used))); }
    conn.execute("DELETE FROM method_types WHERE id=?", [id])?;
    Ok(())
}
