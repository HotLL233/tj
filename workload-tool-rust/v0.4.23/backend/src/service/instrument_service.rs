//! 仪器业务服务：档案 / 预约（含审批提交）/ 保养。

use crate::db::DbPool;
use crate::error::Result;
use crate::models::instrument::{BookingCreate, BookingResponse, InstrumentCreate, InstrumentResponse, InstrumentUpdate, MaintenanceCreate};
use crate::repo;
use crate::service::approval_service;

/// 新建仪器档案。
pub fn create_instrument(pool: &DbPool, body: &InstrumentCreate, created_by: &str, actor: &str) -> Result<InstrumentResponse> {
    repo::instrument_repo::create(pool, body, created_by, actor)
}

pub fn update_instrument(pool: &DbPool, id: i64, body: &InstrumentUpdate, actor: &str) -> Result<InstrumentResponse> {
    repo::instrument_repo::update(pool, id, body, actor)
}

pub fn delete_instrument(pool: &DbPool, id: i64, actor: &str) -> Result<()> {
    repo::instrument_repo::soft_delete(pool, id, actor)
}

/// 提交仪器预约：落库后进入审批流；无匹配规则则自动通过。
pub fn submit_booking(
    pool: &DbPool,
    body: &BookingCreate,
    applicant_role: &str,
    actor: &str,
) -> Result<BookingResponse> {
    let booking_id = repo::instrument_repo::create_booking(pool, body, "待审批", actor)?;
    let instrument = repo::instrument_repo::get(pool, body.instrument_id).ok();
    let title = format!(
        "仪器预约：{}（{}）",
        instrument.map(|i| i.name).unwrap_or_default(),
        body.applicant
    );
    let outcome = approval_service::submit_for_approval(
        pool, "instrument_booking", booking_id, &title, &body.applicant, applicant_role, None, None,
    )?;
    if outcome.auto_approved {
        repo::instrument_repo::set_booking_status(pool, booking_id, "已通过", "system", "自动通过（无匹配审批规则）", "system")?;
    }
    repo::instrument_repo::list_bookings(pool, Some(body.instrument_id), None, None)?
        .into_iter().find(|b| b.id == booking_id)
        .ok_or_else(|| crate::error::AppError::Internal("预约创建后无法读取".into()))
}

/// 登记仪器保养。
pub fn add_maintenance(pool: &DbPool, body: &MaintenanceCreate, actor: &str) -> Result<i64> {
    repo::instrument_repo::create_maintenance(pool, body, actor)
}
