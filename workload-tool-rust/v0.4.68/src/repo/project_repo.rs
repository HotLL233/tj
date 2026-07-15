use crate::db::DbPool;
use crate::error::{Result, AppError};
use crate::models::project::*;
use crate::repo::audit_repo;

// ── helpers ──

fn parse_comma_i64(s: &str) -> Vec<i64> {
    if s.is_empty() { return vec![]; }
    s.split(',').filter_map(|x| x.trim().parse().ok()).collect()
}

fn parse_comma_str(s: &str) -> Vec<String> {
    if s.is_empty() { return vec![]; }
    s.split(',').map(|x| x.trim().to_string()).collect()
}

const PROJ_SQL: &str =
    "SELECT p.id, p.name, COALESCE(p.notes,''), \
     COALESCE(p.full_name,''), p.sort_order, p.is_active, \
     COALESCE(GROUP_CONCAT(DISTINCT pll.group_id), '') as lab_ids_str, \
     COALESCE(GROUP_CONCAT(DISTINCT pg.name), '') as lab_names_str, \
     COALESCE(GROUP_CONCAT(DISTINCT pml.method_id), '') as method_ids_str, \
     COALESCE(GROUP_CONCAT(DISTINCT m.name), '') as method_names_str, \
     p.created_at, \
     p.high_item \
     FROM projects p \
     LEFT JOIN project_lab_links pll ON p.id = pll.project_id \
     LEFT JOIN project_groups pg ON pll.group_id = pg.id AND pg.name != '研发项目' \
     LEFT JOIN project_method_links pml ON p.id = pml.project_id \
     LEFT JOIN methods m ON pml.method_id = m.id";

fn row_to_project(row: &rusqlite::Row) -> rusqlite::Result<ProjectResponse> {
    let lab_ids_str: String = row.get::<_, String>(6).unwrap_or_default();
    let lab_names_str: String = row.get::<_, String>(7).unwrap_or_default();
    let method_ids_str: String = row.get::<_, String>(8).unwrap_or_default();
    let method_names_str: String = row.get::<_, String>(9).unwrap_or_default();
    Ok(ProjectResponse {
        id: row.get(0)?,
        name: row.get(1)?,
        notes: row.get(2)?,
        full_name: row.get::<_, String>(3).unwrap_or_default(),
        sort_order: row.get::<_, i64>(4).unwrap_or(0),
        is_active: row.get::<_, i64>(5).unwrap_or(1) != 0,
        lab_ids: parse_comma_i64(&lab_ids_str),
        lab_names: parse_comma_str(&lab_names_str),
        method_ids: parse_comma_i64(&method_ids_str),
        method_names: parse_comma_str(&method_names_str),
        created_at: row.get(10)?,
        high_item: row.get::<_, Option<String>>(11).unwrap_or(None),
    })
}

// ── CRUD ──

pub fn list(pool: &DbPool, group_id: Option<i64>, _active_only: bool, _method_type: Option<&str>) -> Result<Vec<ProjectResponse>> {
    let conn = pool.get()?;
    let mut sql = format!("{} WHERE 1=1", PROJ_SQL);
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
    if let Some(gid) = group_id {
        if gid > 0 {
            // Filter projects that have this lab link
            sql.push_str(" AND p.id IN (SELECT project_id FROM project_lab_links WHERE group_id=?)");
            params.push(Box::new(gid));
        }
    }
    sql.push_str(" GROUP BY p.id ORDER BY p.id");
    let mut stmt = conn.prepare(&sql)?;
    let refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let rows: Vec<ProjectResponse> = stmt.query_map(
        rusqlite::params_from_iter(refs.iter()),
        |row| row_to_project(row),
    )?.collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn get_by_id(pool: &DbPool, id: i64) -> Result<ProjectResponse> {
    let conn = pool.get()?;
    let sql = format!("{} WHERE p.id=?1 GROUP BY p.id", PROJ_SQL);
    conn.query_row(&sql, [id], |row| row_to_project(row))
        .map_err(|e| AppError::NotFound(format!("项目不存在: {}", e)))
}

pub fn create(pool: &DbPool, body: &ProjectCreate) -> Result<ProjectResponse> {
    let conn = pool.get()?;
    // 取第一个 lab_id 作为 group_id（项目必须关联至少一个实验室）
    let group_id = body.lab_ids
        .as_ref()
        .and_then(|ids| ids.first().copied())
        .ok_or_else(|| AppError::Validation("请至少选择一个实验室".into()))?;
    let nt = body.notes.as_deref().unwrap_or("");
    let fnm = body.full_name.as_deref().unwrap_or("");
    let so = body.sort_order.unwrap_or(0);
    let ia = body.is_active.unwrap_or(true) as i64;
    conn.execute(
        "INSERT INTO projects (group_id, name, full_name, notes, sort_order, is_active, high_item) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![group_id, body.name, fnm, nt, so, ia, body.high_item],
    )?;
    let pid = conn.last_insert_rowid();
    audit_repo::log(pool, "create", "projects", Some(pid), "system", &format!("创建项目「{}」", body.name))?;

    // Insert lab links
    if let Some(ref lab_ids) = body.lab_ids {
        for lid in lab_ids {
            conn.execute(
                "INSERT OR IGNORE INTO project_lab_links (project_id, group_id) VALUES (?1,?2)",
                rusqlite::params![pid, lid],
            )?;
        }
    }

    // Insert method links
    if let Some(ref method_ids) = body.method_ids {
        for mid in method_ids {
            conn.execute(
                "INSERT OR IGNORE INTO project_method_links (project_id, method_id) VALUES (?1,?2)",
                rusqlite::params![pid, mid],
            )?;
        }
    }

    get_by_id(pool, pid)
}

pub fn update(pool: &DbPool, id: i64, body: &ProjectUpdate) -> Result<ProjectResponse> {
    let conn = pool.get()?;
    if let Some(ref n) = body.name {
        conn.execute("UPDATE projects SET name=?1 WHERE id=?2", (n, id))?;
    }
    if let Some(ref n) = body.full_name {
        conn.execute("UPDATE projects SET full_name=?1 WHERE id=?2", (n, id))?;
    }
    if let Some(ref n) = body.notes {
        conn.execute("UPDATE projects SET notes=?1 WHERE id=?2", (n, id))?;
    }
    if let Some(s) = body.sort_order {
        conn.execute("UPDATE projects SET sort_order=?1 WHERE id=?2", (s, id))?;
    }
    if let Some(a) = body.is_active {
        conn.execute("UPDATE projects SET is_active=?1 WHERE id=?2", (a as i64, id))?;
    }
    if let Some(ref hi) = body.high_item {
        let hi_trim = hi.trim().to_string();
        let val = if hi_trim.is_empty() { None } else { Some(hi_trim) };
        conn.execute("UPDATE projects SET high_item=?1 WHERE id=?2", rusqlite::params![val, id])?;
    }
    // Replace lab links
    if let Some(ref lab_ids) = body.lab_ids {
        conn.execute("DELETE FROM project_lab_links WHERE project_id=?1", [id])?;
        for lid in lab_ids {
            conn.execute(
                "INSERT OR IGNORE INTO project_lab_links (project_id, group_id) VALUES (?1,?2)",
                rusqlite::params![id, lid],
            )?;
        }
    }
    // Replace method links
    if let Some(ref method_ids) = body.method_ids {
        conn.execute("DELETE FROM project_method_links WHERE project_id=?1", [id])?;
        for mid in method_ids {
            conn.execute(
                "INSERT OR IGNORE INTO project_method_links (project_id, method_id) VALUES (?1,?2)",
                rusqlite::params![id, mid],
            )?;
        }
    }
    audit_repo::log(pool, "update", "projects", Some(id), "system", &format!("编辑项目「{}」", body.name.as_deref().unwrap_or("")))?;
    get_by_id(pool, id)
}

pub fn delete(pool: &DbPool, id: i64) -> Result<()> {
    let conn = pool.get()?;
    let wr_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM work_records WHERE project_id=?1 AND deleted_at IS NULL", [id], |r| r.get(0),
    )?;
    if wr_count > 0 { return Err(AppError::Validation(format!("有{}条工作记录，无法删除", wr_count))); }
    let proj = get_by_id(pool, id)?;
    audit_repo::log(pool, "delete", "projects", Some(id), "system", &format!("删除项目「{}」", proj.name))?;
    // CASCADE cleans up project_lab_links, project_method_links
    conn.execute("DELETE FROM projects WHERE id=?1", [id])?;
    Ok(())
}

// ── 批量系数 ──

pub fn batch_coefficient(pool: &DbPool, group_id: i64, coefficient: f64) -> Result<i64> {
    let conn = pool.get()?;
    // Update coefficient for projects linked to this group
    let count = conn.execute(
        "UPDATE projects SET coefficient=?1 WHERE id IN (SELECT project_id FROM project_lab_links WHERE group_id=?2)",
        rusqlite::params![coefficient, group_id],
    )?;
    Ok(count as i64)
}
