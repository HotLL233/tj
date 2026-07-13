use crate::db::DbPool;
use crate::error::Result;
use crate::models::group::{GroupCreate, GroupResponse, GroupUpdate};
use crate::repo::audit_repo;

pub fn list(pool: &DbPool) -> Result<Vec<GroupResponse>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT g.id, g.name, g.sort_order, g.created_at,
                COUNT(pll.project_id) AS project_count,
                (SELECT GROUP_CONCAT(p.name) FROM project_lab_links pll2
                 JOIN projects p ON p.id = pll2.project_id
                 WHERE pll2.group_id = g.id) AS project_names,
                (SELECT COUNT(*) FROM rd_work_records wr2
                 JOIN projects p2 ON wr2.project_id = p2.id
                 JOIN project_lab_links pll3 ON pll3.project_id = p2.id
                 WHERE pll3.group_id = g.id AND wr2.deleted_at IS NULL AND wr2.status = '待取样') AS rd_record_count
         FROM project_groups g
         LEFT JOIN project_lab_links pll ON pll.group_id = g.id
         WHERE g.name != '研发项目'
         GROUP BY g.id ORDER BY g.sort_order"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(GroupResponse {
            id: row.get(0)?, name: row.get(1)?, sort_order: row.get(2)?,
            created_at: row.get(3)?, project_count: row.get(4)?,
            project_names: row.get(5)?, rd_record_count: row.get(6)?,
        })
    })?;
    rows.collect::<std::result::Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn get_by_id(pool: &DbPool, id: i64) -> Result<GroupResponse> {
    let conn = pool.get()?;
    conn.query_row(
        "SELECT g.id, g.name, g.sort_order, g.created_at, COUNT(pll.project_id),
                (SELECT GROUP_CONCAT(p.name) FROM project_lab_links pll2
                 JOIN projects p ON p.id = pll2.project_id
                 WHERE pll2.group_id = g.id) AS project_names,
                (SELECT COUNT(*) FROM rd_work_records wr2
                 JOIN projects p2 ON wr2.project_id = p2.id
                 JOIN project_lab_links pll3 ON pll3.project_id = p2.id
                 WHERE pll3.group_id = g.id AND wr2.deleted_at IS NULL AND wr2.status = '待取样') AS rd_record_count
         FROM project_groups g LEFT JOIN project_lab_links pll ON pll.group_id = g.id
         WHERE g.id = ?1 GROUP BY g.id",
        [id],
        |row| Ok(GroupResponse {
            id: row.get(0)?, name: row.get(1)?, sort_order: row.get(2)?,
            created_at: row.get(3)?, project_count: row.get(4)?,
            project_names: row.get(5)?, rd_record_count: row.get(6)?,
        }),
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => crate::error::AppError::NotFound("分组不存在".into()),
        _ => e.into(),
    })
}

pub fn create(pool: &DbPool, body: &GroupCreate) -> Result<GroupResponse> {
    let conn = pool.get()?;
    conn.execute(
        "INSERT INTO project_groups (name, sort_order) VALUES (?1, ?2)",
        (&body.name, body.sort_order.unwrap_or(0)),
    )?;
    let id = conn.last_insert_rowid();
    audit_repo::log(pool, "create", "project_groups", Some(id), "system", &format!("创建实验室「{}」", body.name))?;
    get_by_id(pool, id)
}

pub fn update(pool: &DbPool, id: i64, body: &GroupUpdate) -> Result<GroupResponse> {
    let conn = pool.get()?;
    if let Some(ref name) = body.name {
        conn.execute("UPDATE project_groups SET name=?1 WHERE id=?2", (name, id))?;
    }
    if let Some(so) = body.sort_order {
        conn.execute("UPDATE project_groups SET sort_order=?1 WHERE id=?2", (so, id))?;
    }
    audit_repo::log(pool, "update", "project_groups", Some(id), "system", &format!("编辑实验室「{}」", body.name.as_deref().unwrap_or("")))?;
    get_by_id(pool, id)
}

pub fn delete(pool: &DbPool, id: i64) -> Result<()> {
    let g = get_by_id(pool, id)?;
    if g.project_count > 0 {
        return Err(crate::error::AppError::Conflict("分组下还有项目，无法删除".into()));
    }
    let conn = pool.get()?;
    let default_gid: i64 = conn.query_row(
        "SELECT id FROM project_groups WHERE name='研发项目'", [], |r| r.get(0),
    )?;
    conn.execute("UPDATE projects SET group_id=?1 WHERE group_id=?2",
        rusqlite::params![default_gid, id])?;
    conn.execute("DELETE FROM project_groups WHERE id=?1", [id])?;
    audit_repo::log(pool, "delete", "project_groups", Some(id), "system", &format!("删除实验室「{}」", g.name))?;
    Ok(())
}
