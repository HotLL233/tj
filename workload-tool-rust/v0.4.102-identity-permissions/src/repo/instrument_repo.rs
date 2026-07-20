use crate::db::DbPool;
use crate::error::{AppError, Result};
use crate::models::instrument::{InstrumentCreate, InstrumentResponse, InstrumentUpdate};
use crate::repo::audit_repo;

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<InstrumentResponse> {
    Ok(InstrumentResponse {
        id: row.get(0)?,
        code: row.get(1)?,
        name: row.get(2)?,
        instrument_type: row.get(3)?,
        is_active: row.get::<_, i64>(4).unwrap_or(1) != 0,
        notes: row.get(5)?,
        created_at: row.get(6)?,
    })
}

pub fn list(pool: &DbPool) -> Result<Vec<InstrumentResponse>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(
        "SELECT id,code,COALESCE(name,''),instrument_type,is_active,COALESCE(notes,''),created_at
         FROM instruments ORDER BY instrument_type,code,id",
    )?;
    let rows = stmt
        .query_map([], map_row)?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn get(pool: &DbPool, id: i64) -> Result<InstrumentResponse> {
    let conn = pool.get()?;
    conn.query_row(
        "SELECT id,code,COALESCE(name,''),instrument_type,is_active,COALESCE(notes,''),created_at FROM instruments WHERE id=?1",
        [id],
        map_row,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound("仪器不存在".into()),
        _ => e.into(),
    })
}

fn validate(code: &str, instrument_type: &str) -> Result<(String, String)> {
    let code = code.trim().to_string();
    let instrument_type = instrument_type.trim().to_string();
    if code.is_empty() {
        return Err(AppError::Validation("仪器编号不能为空".into()));
    }
    if instrument_type.is_empty() {
        return Err(AppError::Validation("仪器类型不能为空".into()));
    }
    Ok((code, instrument_type))
}

pub fn create(pool: &DbPool, body: &InstrumentCreate, operator: &str) -> Result<InstrumentResponse> {
    let conn = pool.get()?;
    let (code, instrument_type) = validate(&body.code, &body.instrument_type)?;
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM instruments WHERE code=?1",
        [&code],
        |row| row.get(0),
    )?;
    if exists > 0 {
        return Err(AppError::Validation(format!("仪器编号已存在：{code}")));
    }
    let name = body.name.as_deref().unwrap_or("").trim();
    if !name.is_empty() {
        let duplicate_name: i64 = conn.query_row("SELECT COUNT(*) FROM instruments WHERE name=?1", [name], |row| row.get(0))?;
        if duplicate_name > 0 { return Err(AppError::Validation(format!("仪器名称已存在：{name}"))); }
    }
    conn.execute(
        "INSERT INTO instruments(code,name,instrument_type,is_active,notes) VALUES(?1,?2,?3,?4,?5)",
        rusqlite::params![
            code,
            name,
            instrument_type,
            body.is_active.unwrap_or(true) as i64,
            body.notes.as_deref().unwrap_or("")
        ],
    )?;
    let id = conn.last_insert_rowid();
    audit_repo::log(
        pool,
        "create",
        "instruments",
        Some(id),
        operator,
        &format!("创建仪器“{}”", body.code.trim()),
    )?;
    get(pool, id)
}

pub fn update(pool: &DbPool, id: i64, body: &InstrumentUpdate, operator: &str) -> Result<InstrumentResponse> {
    let current = get(pool, id)?;
    let code = body.code.as_deref().unwrap_or(&current.code);
    let instrument_type = body
        .instrument_type
        .as_deref()
        .unwrap_or(&current.instrument_type);
    let (code, instrument_type) = validate(code, instrument_type)?;
    let conn = pool.get()?;
    let duplicate: i64 = conn.query_row(
        "SELECT COUNT(*) FROM instruments WHERE code=?1 AND id<>?2",
        rusqlite::params![code, id],
        |row| row.get(0),
    )?;
    if duplicate > 0 {
        return Err(AppError::Validation(format!("仪器编号已存在：{code}")));
    }
    let name = body.name.as_deref().unwrap_or(&current.name).trim();
    if !name.is_empty() {
        let duplicate_name: i64 = conn.query_row("SELECT COUNT(*) FROM instruments WHERE name=?1 AND id<>?2", rusqlite::params![name, id], |row| row.get(0))?;
        if duplicate_name > 0 { return Err(AppError::Validation(format!("仪器名称已存在：{name}"))); }
    }
    conn.execute(
        "UPDATE instruments SET code=?1,name=?2,instrument_type=?3,is_active=?4,notes=?5 WHERE id=?6",
        rusqlite::params![
            code,
            name,
            instrument_type,
            body.is_active.unwrap_or(current.is_active) as i64,
            body.notes.as_deref().unwrap_or(&current.notes),
            id
        ],
    )?;
    audit_repo::log(
        pool,
        "update",
        "instruments",
        Some(id),
        operator,
        &format!("编辑仪器“{}”", current.code),
    )?;
    get(pool, id)
}

pub fn delete(pool: &DbPool, id: i64, operator: &str) -> Result<()> {
    let current = get(pool, id)?;
    let conn = pool.get()?;
    let used: i64 = conn.query_row(
        "SELECT COUNT(*) FROM methods WHERE instrument_id=?1",
        [id],
        |row| row.get(0),
    )?;
    if used > 0 {
        return Err(AppError::Validation(format!(
            "仪器被 {used} 个方法实例使用，无法删除"
        )));
    }
    conn.execute("DELETE FROM instruments WHERE id=?1", [id])?;
    audit_repo::log(
        pool,
        "delete",
        "instruments",
        Some(id),
        operator,
        &format!("删除仪器“{}”", current.code),
    )?;
    Ok(())
}
