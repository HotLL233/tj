use crate::db::DbPool;
use crate::error::{AppError, Result};
use crate::models::sample_info_column::*;
use rusqlite::params;

pub fn list_all(pool: &DbPool) -> Result<Vec<SampleInfoColumn>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT c.id, c.field_key, c.label, c.data_type, c.is_predefined, c.is_required, c.is_active, \
         c.width, c.sort_order, c.options, c.show_in_list, c.show_in_export, c.show_in_form, \
         c.type_key, c.created_at, c.updated_at, \
         COALESCE((SELECT GROUP_CONCAT(v.type_key) FROM sample_info_column_visibility v WHERE v.column_id=c.id), '') AS visible_types \
         FROM sample_info_columns c ORDER BY c.sort_order ASC"
    )?;
    let rows = stmt.query_map([], |row| {
        let vt: String = row.get(16).unwrap_or_default();
        Ok(SampleInfoColumn {
            id: row.get(0)?,
            field_key: row.get(1)?,
            label: row.get(2)?,
            data_type: row.get(3)?,
            is_predefined: row.get::<_, i64>(4)? != 0,
            is_required: row.get::<_, i64>(5)? != 0,
            is_active: row.get::<_, i64>(6)? != 0,
            width: row.get(7)?,
            sort_order: row.get(8)?,
            options: row.get(9)?,
            show_in_list: row.get::<_, i64>(10)? != 0,
            show_in_export: row.get::<_, i64>(11)? != 0,
            show_in_form: row.get::<_, i64>(12)? != 0,
            type_key: row.get(13)?,
            created_at: row.get(14)?,
            updated_at: row.get(15)?,
            visible_types: vt.split(',').filter(|s| !s.is_empty()).map(|s| s.to_string()).collect(),
        })
    })?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

pub fn list_active(pool: &DbPool) -> Result<Vec<SampleInfoColumn>> {
    let items = list_all(pool)?;
    Ok(items.into_iter().filter(|c| c.is_active).collect())
}

/// 按检测类型加载启用的列（自定义列 + 该类型可见的预置列）
pub fn list_active_by_type(pool: &DbPool, type_key: &str) -> Result<Vec<SampleInfoColumn>> {
    let conn = pool.get()?;
    let sql = "
        -- 自定义列（该类型专属）
        SELECT c.id, c.field_key, c.label, c.data_type, c.is_predefined, c.is_required, c.is_active,
               c.width, c.sort_order, c.options, c.show_in_list, c.show_in_export, c.show_in_form,
               c.type_key, c.created_at, c.updated_at
        FROM sample_info_columns c
        WHERE c.is_active = 1 AND c.type_key = ?1 AND c.is_predefined = 0

        UNION ALL

        -- 预置列：LEFT JOIN visibility，无 visibility 行时默认可见
        SELECT c.id, c.field_key, c.label, c.data_type, c.is_predefined, c.is_required, c.is_active,
               c.width, c.sort_order, c.options, c.show_in_list, c.show_in_export, c.show_in_form,
               c.type_key, c.created_at, c.updated_at
        FROM sample_info_columns c
        LEFT JOIN sample_info_column_visibility v ON v.column_id = c.id AND v.type_key = ?1
        WHERE c.is_active = 1 AND c.is_predefined = 1
          AND (v.is_visible = 1 OR v.is_visible IS NULL)

        ORDER BY sort_order ASC
    ";
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([type_key], |row| {
        Ok(SampleInfoColumn {
            id: row.get(0)?,
            field_key: row.get(1)?,
            label: row.get(2)?,
            data_type: row.get(3)?,
            is_predefined: row.get::<_, i64>(4)? != 0,
            is_required: row.get::<_, i64>(5)? != 0,
            is_active: row.get::<_, i64>(6)? != 0,
            width: row.get(7)?,
            sort_order: row.get(8)?,
            options: row.get(9)?,
            show_in_list: row.get::<_, i64>(10)? != 0,
            show_in_export: row.get::<_, i64>(11)? != 0,
            show_in_form: row.get::<_, i64>(12)? != 0,
            type_key: row.get(13)?,
            created_at: row.get(14)?,
            updated_at: row.get(15)?,
            visible_types: vec![],
        })
    })?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

/// 管理页专用：所有列 + visibility 信息（LEFT JOIN）
pub fn list_all_with_visibility(pool: &DbPool, type_key: &str) -> Result<Vec<(SampleInfoColumn, bool)>> {
    let conn = pool.get()?;
    let sql = "
        SELECT c.id, c.field_key, c.label, c.data_type, c.is_predefined, c.is_required, c.is_active,
               c.width, c.sort_order, c.options, c.show_in_list, c.show_in_export, c.show_in_form,
               c.type_key, c.created_at, c.updated_at,
               COALESCE(v.is_visible, 0) as visible
        FROM sample_info_columns c
        LEFT JOIN sample_info_column_visibility v ON v.column_id = c.id AND v.type_key = ?1
        ORDER BY c.is_predefined DESC, c.sort_order ASC
    ";
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([type_key], |row| {
        let col = SampleInfoColumn {
            id: row.get(0)?,
            field_key: row.get(1)?,
            label: row.get(2)?,
            data_type: row.get(3)?,
            is_predefined: row.get::<_, i64>(4)? != 0,
            is_required: row.get::<_, i64>(5)? != 0,
            is_active: row.get::<_, i64>(6)? != 0,
            width: row.get(7)?,
            sort_order: row.get(8)?,
            options: row.get(9)?,
            show_in_list: row.get::<_, i64>(10)? != 0,
            show_in_export: row.get::<_, i64>(11)? != 0,
            show_in_form: row.get::<_, i64>(12)? != 0,
            type_key: row.get(13)?,
            created_at: row.get(14)?,
            updated_at: row.get(15)?,
            visible_types: vec![],
        };
        let visible: i64 = row.get(16)?;
        Ok((col, visible != 0))
    })?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

pub fn create(pool: &DbPool, data: &ColumnCreate) -> Result<SampleInfoColumn> {
    let conn = pool.get()?;
    let field_key = &data.field_key;
    let label = &data.label;
    let data_type = &data.data_type;
    let width = data.width.unwrap_or(100);
    let sort_order = data.sort_order.unwrap_or(0);
    let is_required = data.is_required.unwrap_or(false);
    let show_in_list = data.show_in_list.unwrap_or(true);
    let show_in_export = data.show_in_export.unwrap_or(true);
    let show_in_form = data.show_in_form.unwrap_or(true);

    conn.execute(
        "INSERT INTO sample_info_columns (field_key, label, data_type, type_key, is_predefined, is_required, is_active, width, sort_order, options, show_in_list, show_in_export, show_in_form) \
         VALUES (?1, ?2, ?3, ?4, 0, ?5, 1, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            field_key, label, data_type, data.type_key,
            is_required as i64, width, sort_order, data.options,
            show_in_list as i64, show_in_export as i64, show_in_form as i64,
        ],
    )?;
    let id = conn.last_insert_rowid();
    list_all(pool)?
        .into_iter()
        .find(|c| c.id == id)
        .ok_or_else(|| AppError::Internal("创建列失败".into()))
}

pub fn update(pool: &DbPool, id: i64, data: &ColumnUpdate) -> Result<SampleInfoColumn> {
    let conn = pool.get()?;

    // 先检查是否存在
    let existing = list_all(pool)?
        .into_iter()
        .find(|c| c.id == id)
        .ok_or_else(|| AppError::NotFound("列不存在".into()))?;

    let label = data.label.as_deref().unwrap_or(&existing.label);
    let data_type = data.data_type.as_deref().unwrap_or(&existing.data_type);
    let is_active = data.is_active.unwrap_or(existing.is_active) as i64;
    let is_required = data.is_required.unwrap_or(existing.is_required) as i64;
    let width = data.width.unwrap_or(existing.width);
    let options = data.options.as_deref().or(existing.options.as_deref());
    let show_in_list = data.show_in_list.unwrap_or(existing.show_in_list) as i64;
    let show_in_export = data.show_in_export.unwrap_or(existing.show_in_export) as i64;
    let show_in_form = data.show_in_form.unwrap_or(existing.show_in_form) as i64;

    conn.execute(
        "UPDATE sample_info_columns SET label=?1, data_type=?2, is_active=?3, is_required=?4, \
         width=?5, options=?6, show_in_list=?7, show_in_export=?8, show_in_form=?9, \
         updated_at=datetime('now','localtime') WHERE id=?10",
        params![
            label, data_type, is_active, is_required, width, options,
            show_in_list, show_in_export, show_in_form, id,
        ],
    )?;

    list_all(pool)?
        .into_iter()
        .find(|c| c.id == id)
        .ok_or_else(|| AppError::Internal("更新列失败".into()))
}

pub fn soft_delete(pool: &DbPool, id: i64) -> Result<()> {
    let conn = pool.get()?;

    // 检查是否预置字段
    let col = list_all(pool)?
        .into_iter()
        .find(|c| c.id == id)
        .ok_or_else(|| AppError::NotFound("列不存在".into()))?;

    if col.is_predefined {
        return Err(AppError::Validation("预置字段不可删除".into()));
    }

    conn.execute("DELETE FROM sample_info_columns WHERE id=?1", params![id])?;
    Ok(())
}

pub fn reorder(pool: &DbPool, data: &ColumnReorder) -> Result<Vec<SampleInfoColumn>> {
    let conn = pool.get()?;
    for item in &data.ids {
        conn.execute(
            "UPDATE sample_info_columns SET sort_order=?1, updated_at=datetime('now','localtime') WHERE id=?2",
            params![item.sort_order, item.id],
        )?;
    }
    list_all(pool)
}

pub fn set_visibility(pool: &DbPool, column_id: i64, type_keys: &[String]) -> Result<SampleInfoColumn> {
    let conn = pool.get()?;
    // 清空旧 visibility
    conn.execute("DELETE FROM sample_info_column_visibility WHERE column_id=?1", params![column_id])?;
    // 插入新 visibility
    for tk in type_keys {
        conn.execute(
            "INSERT INTO sample_info_column_visibility (column_id, type_key, is_visible) VALUES (?1, ?2, 1)",
            params![column_id, tk],
        )?;
    }
    list_all(pool)?
        .into_iter()
        .find(|c| c.id == column_id)
        .ok_or_else(|| AppError::NotFound("列不存在".into()))
}
