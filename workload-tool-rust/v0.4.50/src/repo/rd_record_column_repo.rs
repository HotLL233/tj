use crate::db::DbPool;
use crate::error::{AppError, Result};
use crate::models::rd_record_column::{RdRecordColumn, RdRecordColumnUpdate};

pub fn list_all(pool: &DbPool) -> Result<Vec<RdRecordColumn>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT id, name, label, data_type, width, sort_order, is_predefined, \
         show_in_list, show_in_form, created_at, updated_at \
         FROM rd_record_columns ORDER BY sort_order ASC"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(RdRecordColumn {
            id: row.get(0)?,
            name: row.get(1)?,
            label: row.get(2)?,
            data_type: row.get(3)?,
            width: row.get(4)?,
            sort_order: row.get(5)?,
            is_predefined: row.get::<_, i64>(6)? != 0,
            show_in_list: row.get::<_, i64>(7)? != 0,
            show_in_form: row.get::<_, i64>(8)? != 0,
            created_at: row.get(9)?,
            updated_at: row.get(10)?,
        })
    })?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

pub fn list_active_in_list(pool: &DbPool) -> Result<Vec<RdRecordColumn>> {
    let all = list_all(pool)?;
    Ok(all.into_iter().filter(|c| c.show_in_list).collect())
}

pub fn list_active_in_form(pool: &DbPool) -> Result<Vec<RdRecordColumn>> {
    let all = list_all(pool)?;
    Ok(all.into_iter().filter(|c| c.show_in_form).collect())
}

pub fn update(pool: &DbPool, id: i64, data: &RdRecordColumnUpdate) -> Result<RdRecordColumn> {
    let conn = pool.get()?;

    let existing = list_all(pool)?
        .into_iter()
        .find(|c| c.id == id)
        .ok_or_else(|| AppError::NotFound("列不存在".into()))?;

    let width = data.width.unwrap_or(existing.width);
    let show_in_list = data.show_in_list.unwrap_or(existing.show_in_list) as i64;
    let show_in_form = data.show_in_form.unwrap_or(existing.show_in_form) as i64;

    conn.execute(
        "UPDATE rd_record_columns SET width=?1, show_in_list=?2, show_in_form=?3, updated_at=datetime('now','localtime') WHERE id=?4",
        rusqlite::params![width, show_in_list, show_in_form, id],
    )?;

    list_all(pool)?
        .into_iter()
        .find(|c| c.id == id)
        .ok_or_else(|| AppError::Internal("更新列失败".into()))
}
