use rusqlite::Connection;
use crate::models::import_record::ImportRecord;
use crate::error::AppError;

/// Upsert project_group and project, return project_id.
pub fn upsert_project(conn: &Connection, group_name: &str, project_name: &str) -> Result<i64, AppError> {
    // Insert or get group
    let group_id: i64 = conn.query_row(
        "INSERT INTO project_groups (name, sort_order) VALUES (?1, (SELECT COALESCE(MAX(sort_order),0)+1 FROM project_groups)) ON CONFLICT(name) DO UPDATE SET name=name RETURNING id",
        [group_name],
        |r| r.get(0),
    ).unwrap_or_else(|_| {
        conn.query_row("SELECT id FROM project_groups WHERE name=?1", [group_name], |r| r.get(0)).unwrap()
    });

    // Insert or get project
    let project_id: i64 = conn.query_row(
        "INSERT INTO projects (group_id, name, sort_order) VALUES (?1, ?2, (SELECT COALESCE(MAX(sort_order),0)+1 FROM projects WHERE group_id=?1)) ON CONFLICT(name, group_id) DO UPDATE SET name=name RETURNING id",
        rusqlite::params![group_id, project_name],
        |r| r.get(0),
    ).unwrap_or_else(|_| {
        conn.query_row("SELECT id FROM projects WHERE group_id=?1 AND name=?2", rusqlite::params![group_id, project_name], |r| r.get(0)).unwrap()
    });

    Ok(project_id)
}

/// Batch import records with transaction and quantity accumulation.
/// Same (project_id, recorded_at, batch_no) → quantities accumulate.
pub fn batch_import(conn: &Connection, records: &[ImportRecord]) -> Result<usize, AppError> {
    let tx = conn.unchecked_transaction()?;
    let mut imported = 0usize;

    for rec in records {
        let project_id = upsert_project(&tx, &rec.group_name, &rec.project_name)?;
        let user_name = rec.user_name.as_deref().unwrap_or("导入");

        // Try UPDATE existing row first
        let updated = tx.execute(
            "UPDATE work_records SET quantity = quantity + ?1 WHERE project_id=?2 AND recorded_at=?3 AND batch_no=?4 AND deleted_at IS NULL",
            rusqlite::params![rec.quantity, project_id, rec.recorded_at, rec.batch_no],
        )?;

        if updated == 0 {
            // INSERT new row
            tx.execute(
                "INSERT INTO work_records (project_id, user_name, quantity, recorded_at, batch_no, extra_info) VALUES (?1,?2,?3,?4,?5,?6)",
                rusqlite::params![project_id, user_name, rec.quantity, rec.recorded_at, rec.batch_no, rec.extra_info],
            )?;
        }
        imported += 1;
    }

    tx.commit()?;
    Ok(imported)
}
