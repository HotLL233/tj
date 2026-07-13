use crate::error::AppError;
use crate::db::DbPool;
use crate::models::rd_record::{RdRecordResponse, RdSampleUpdate};
use crate::models::record::{RecordCreate, RecordUpdate};
use crate::repo;

/// Validate and create a work record
pub fn create_record(pool: &DbPool, input: &RecordCreate) -> Result<RdRecordResponse, AppError> {
    if input.quantity <= 0 {
        return Err(AppError::Validation("数量必须大于0".into()));
    }
    // Verify project exists (returns error if not found)
    repo::project_repo::get_by_id(pool, input.project_id)?;
    repo::rd_record_repo::create(pool, input)
}

/// Update a work record (with change detection and deleted check)
pub fn update_record(pool: &DbPool, id: i64, input: &RecordUpdate, user_name: &str) -> Result<RdRecordResponse, AppError> {
    let old = repo::rd_record_repo::get_by_id(pool, id)?;

    if old.deleted_at.is_some() {
        return Err(AppError::Validation("记录已被删除，无法编辑".into()));
    }

    let un_changed = input.user_name.as_ref().map_or(true, |u| u == &old.user_name);
    let qty_changed = input.quantity.map_or(true, |q| q == old.quantity);
    let dt_changed = input.recorded_at.as_ref().map_or(true, |d| d == &old.recorded_at);
    if un_changed && qty_changed && dt_changed {
        return Err(AppError::Validation("没有需要更新的字段".into()));
    }

    repo::rd_record_repo::update(pool, id, input, user_name)
}

/// Soft-delete a single record
pub fn delete_record(pool: &DbPool, id: i64, user_name: &str) -> Result<(), AppError> {
    repo::rd_record_repo::soft_delete(pool, id, user_name)
}

/// Mark a record as sampled
pub fn sample(pool: &DbPool, id: i64, body: &RdSampleUpdate) -> Result<RdRecordResponse, AppError> {
    repo::rd_record_repo::sample(pool, id, body)
}
