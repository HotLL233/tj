//! 仪器管理数据访问层：仪器档案 / 预约 / 保养。
//! 写操作统一在事务内完成业务 SQL + 审计（module='instrument'）。

use crate::db::DbPool;
use crate::error::Result;
use crate::models::instrument::{
    BookingCreate, BookingResponse, InstrumentBooking, InstrumentCreate, InstrumentMaintenance,
    InstrumentResponse, InstrumentUpdate, MaintenanceCreate, MaintenanceResponse,
};
use crate::repo::audit_repo;

fn map_instrument(row: &rusqlite::Row) -> rusqlite::Result<InstrumentResponse> {
    Ok(InstrumentResponse {
        id: row.get(0)?,
        name: row.get(1)?,
        model: row.get(2)?,
        location: row.get(3)?,
        manager: row.get(4)?,
        status: row.get(5)?,
        photo_path: row.get(6)?,
        qr_code_path: row.get(7)?,
        notes: row.get(8)?,
        created_by: row.get(9)?,
        created_at: row.get(10)?,
    })
}

fn map_booking(row: &rusqlite::Row) -> rusqlite::Result<BookingResponse> {
    Ok(BookingResponse {
        id: row.get(0)?,
        instrument_id: row.get(1)?,
        instrument_name: row.get(2)?,
        applicant: row.get(3)?,
        start_time: row.get(4)?,
        end_time: row.get(5)?,
        purpose: row.get(6)?,
        status: row.get(7)?,
        approver: row.get(8)?,
        approved_at: row.get(9)?,
        approver_note: row.get(10)?,
        created_at: row.get(11)?,
    })
}

fn map_maintenance(row: &rusqlite::Row) -> rusqlite::Result<MaintenanceResponse> {
    Ok(MaintenanceResponse {
        id: row.get(0)?,
        instrument_id: row.get(1)?,
        instrument_name: row.get(2)?,
        maintainer: row.get(3)?,
        maintained_at: row.get(4)?,
        content: row.get(5)?,
        cost: row.get(6)?,
        created_at: row.get(7)?,
    })
}

const INSTRUMENT_COLS: &str = "id, name, model, location, manager, status, photo_path, qr_code_path, notes, created_by, created_at";

pub fn list(pool: &DbPool) -> Result<Vec<InstrumentResponse>> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare(&format!(
        "SELECT {} FROM instruments WHERE deleted_at IS NULL ORDER BY id DESC",
        INSTRUMENT_COLS
    ))?;
    let rows = stmt.query_map([], map_instrument)?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

pub fn get(pool: &DbPool, id: i64) -> Result<InstrumentResponse> {
    let conn = pool.get()?;
    conn.query_row(
        &format!("SELECT {} FROM instruments WHERE id=?1 AND deleted_at IS NULL", INSTRUMENT_COLS),
        [id],
        map_instrument,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => crate::error::AppError::NotFound("仪器不存在".into()),
        _ => e.into(),
    })
}

pub fn create(pool: &DbPool, body: &InstrumentCreate, created_by: &str, actor: &str) -> Result<InstrumentResponse> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO instruments (name, model, location, manager, status, notes, created_by) VALUES (?1,?2,?3,?4,?5,?6,?7)",
        rusqlite::params!(body.name, body.model, body.location, body.manager, body.status, body.notes, created_by),
    )?;
    let id = tx.last_insert_rowid();
    audit_repo::log_with_module_on_conn(
        &tx, "create", "instruments", Some(id), actor,
        &format!("新增仪器：{}", body.name), "instrument", None, None,
    )?;
    tx.commit()?;
    get(pool, id)
}

pub fn update(pool: &DbPool, id: i64, body: &InstrumentUpdate, actor: &str) -> Result<InstrumentResponse> {
    let mut conn = pool.get()?;
    let current = conn.query_row(
        "SELECT name FROM instruments WHERE id=?1 AND deleted_at IS NULL",
        [id],
        |r| r.get::<_, String>(0),
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => crate::error::AppError::NotFound("仪器不存在".into()),
        _ => e.into(),
    })?;
    let mut sets: Vec<String> = vec![];
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
    if let Some(ref v) = body.name { sets.push("name=?1".into()); params.push(Box::new(v.clone())); }
    if let Some(ref v) = body.model { sets.push(format!("model=?{}", params.len() + 1)); params.push(Box::new(v.clone())); }
    if let Some(ref v) = body.location { sets.push(format!("location=?{}", params.len() + 1)); params.push(Box::new(v.clone())); }
    if let Some(ref v) = body.manager { sets.push(format!("manager=?{}", params.len() + 1)); params.push(Box::new(v.clone())); }
    if let Some(ref v) = body.status { sets.push(format!("status=?{}", params.len() + 1)); params.push(Box::new(v.clone())); }
    if let Some(ref v) = body.notes { sets.push(format!("notes=?{}", params.len() + 1)); params.push(Box::new(v.clone())); }
    if sets.is_empty() {
        return Err(crate::error::AppError::Validation("没有需要更新的字段".into()));
    }
    params.push(Box::new(id));
    let sql = format!("UPDATE instruments SET {} WHERE id=?{}", sets.join(","), params.len());
    let tx = conn.transaction()?;
    tx.execute(&sql, rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())))?;
    audit_repo::log_with_module_on_conn(
        &tx, "update", "instruments", Some(id), actor,
        &format!("更新仪器：{}", current), "instrument", None, None,
    )?;
    tx.commit()?;
    get(pool, id)
}

pub fn soft_delete(pool: &DbPool, id: i64, actor: &str) -> Result<()> {
    let mut conn = pool.get()?;
    let name: Option<String> = conn
        .query_row("SELECT name FROM instruments WHERE id=?1 AND deleted_at IS NULL", [id], |r| r.get(0))
        .ok();
    let tx = conn.transaction()?;
    let n = tx.execute("UPDATE instruments SET deleted_at=datetime('now','localtime') WHERE id=?1 AND deleted_at IS NULL", [id])?;
    if n == 0 {
        return Err(crate::error::AppError::NotFound("仪器不存在".into()));
    }
    audit_repo::log_with_module_on_conn(
        &tx, "delete", "instruments", Some(id), actor,
        &format!("删除仪器：{}", name.unwrap_or_default()), "instrument", None, None,
    )?;
    tx.commit()?;
    Ok(())
}

pub fn set_qr_code(pool: &DbPool, id: i64, path: &str) -> Result<()> {
    let mut conn = pool.get()?;
    conn.execute("UPDATE instruments SET qr_code_path=?1 WHERE id=?2", rusqlite::params!(path, id))?;
    Ok(())
}

// ── 预约 ──

pub fn list_bookings(
    pool: &DbPool,
    instrument_id: Option<i64>,
    status: Option<&str>,
    applicant: Option<&str>,
) -> Result<Vec<BookingResponse>> {
    let conn = pool.get()?;
    let mut sql = "SELECT b.id, b.instrument_id, i.name, b.applicant, b.start_time, b.end_time, b.purpose, b.status, b.approver, b.approved_at, b.approver_note, b.created_at \
                   FROM instrument_bookings b LEFT JOIN instruments i ON i.id=b.instrument_id WHERE b.deleted_at IS NULL".to_string();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];
    if let Some(iid) = instrument_id { sql.push_str(" AND b.instrument_id=?1"); params.push(Box::new(iid)); }
    if let Some(s) = status { sql.push_str(&format!(" AND b.status=?{}", params.len() + 1)); params.push(Box::new(s.to_string())); }
    if let Some(a) = applicant { sql.push_str(&format!(" AND b.applicant=?{}", params.len() + 1)); params.push(Box::new(a.to_string())); }
    sql.push_str(" ORDER BY b.start_time DESC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())), map_booking)?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

pub fn get_booking(pool: &DbPool, id: i64) -> Result<InstrumentBooking> {
    let conn = pool.get()?;
    conn.query_row(
        "SELECT id, instrument_id, applicant, start_time, end_time, purpose, status, approver, approved_at, approver_note, created_at, deleted_at \
         FROM instrument_bookings WHERE id=?1",
        [id],
        |row| Ok(InstrumentBooking {
            id: row.get(0)?, instrument_id: row.get(1)?, applicant: row.get(2)?, start_time: row.get(3)?,
            end_time: row.get(4)?, purpose: row.get(5)?, status: row.get(6)?, approver: row.get(7)?,
            approved_at: row.get(8)?, approver_note: row.get(9)?, created_at: row.get(10)?, deleted_at: row.get(11)?,
        }),
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => crate::error::AppError::NotFound("预约不存在".into()),
        _ => e.into(),
    })
}

pub fn create_booking(pool: &DbPool, body: &BookingCreate, status: &str, actor: &str) -> Result<i64> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO instrument_bookings (instrument_id, applicant, start_time, end_time, purpose, status) VALUES (?1,?2,?3,?4,?5,?6)",
        rusqlite::params!(body.instrument_id, body.applicant, body.start_time, body.end_time, body.purpose, status),
    )?;
    let id = tx.last_insert_rowid();
    audit_repo::log_with_module_on_conn(
        &tx, "create", "instrument_bookings", Some(id), actor,
        &format!("提交仪器预约（状态：{}）", status), "instrument", None, None,
    )?;
    tx.commit()?;
    Ok(id)
}

pub fn set_booking_status(
    pool: &DbPool,
    id: i64,
    status: &str,
    approver: &str,
    note: &str,
    actor: &str,
) -> Result<()> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    tx.execute(
        "UPDATE instrument_bookings SET status=?1, approver=?2, approver_note=?3, approved_at=datetime('now','localtime') WHERE id=?4 AND deleted_at IS NULL",
        rusqlite::params!(status, approver, note, id),
    )?;
    audit_repo::log_with_module_on_conn(
        &tx, "approval", "instrument_bookings", Some(id), actor,
        &format!("审批仪器预约：{}（{}）", status, note), "instrument", None, None,
    )?;
    tx.commit()?;
    Ok(())
}

// ── 保养 ──

pub fn list_maintenances(pool: &DbPool, instrument_id: Option<i64>) -> Result<Vec<MaintenanceResponse>> {
    let conn = pool.get()?;
    let mut sql = "SELECT m.id, m.instrument_id, i.name, m.maintainer, m.maintained_at, m.content, m.cost, m.created_at \
                   FROM instrument_maintenances m LEFT JOIN instruments i ON i.id=m.instrument_id WHERE m.deleted_at IS NULL".to_string();
    if let Some(iid) = instrument_id { sql.push_str(&format!(" AND m.instrument_id={}", iid)); }
    sql.push_str(" ORDER BY m.maintained_at DESC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], map_maintenance)?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

pub fn create_maintenance(pool: &DbPool, body: &MaintenanceCreate, actor: &str) -> Result<i64> {
    let mut conn = pool.get()?;
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO instrument_maintenances (instrument_id, maintainer, maintained_at, content, cost) VALUES (?1,?2,?3,?4,?5)",
        rusqlite::params!(body.instrument_id, body.maintainer, body.maintained_at, body.content, body.cost),
    )?;
    let id = tx.last_insert_rowid();
    audit_repo::log_with_module_on_conn(
        &tx, "create", "instrument_maintenances", Some(id), actor,
        &format!("登记仪器保养：{}", body.maintainer), "instrument", None, None,
    )?;
    tx.commit()?;
    Ok(id)
}
