use crate::error::AppError;
use crate::db::DbPool;
use crate::models::sample::{SampleRecordResponse, SampleRecordCreate, SampleRecordUpdate};
use crate::repo;

/// Create sample record with validation.
pub fn create(pool: &DbPool, input: &SampleRecordCreate) -> Result<SampleRecordResponse, AppError> {
    if input.sample_count <= 0 {
        return Err(AppError::Validation("样品数量必须大于0".into()));
    }
    if input.sample_name.trim().is_empty() {
        return Err(AppError::Validation("样品名称不能为空".into()));
    }
    // Verify project exists
    repo::project_repo::get_by_id(pool, input.project_id)?;
    repo::sample_repo::create(pool, input)
}

/// Update sample record with deleted check.
pub fn update(pool: &DbPool, id: i64, input: &SampleRecordUpdate) -> Result<SampleRecordResponse, AppError> {
    let old = repo::sample_repo::get_by_id(pool, id)?;
    if old.deleted_at.is_some() {
        return Err(AppError::Validation("记录已被删除，无法编辑".into()));
    }
    repo::sample_repo::update(pool, id, input)
}

/// Soft-delete sample record.
pub fn delete(pool: &DbPool, id: i64) -> Result<(), AppError> {
    repo::sample_repo::soft_delete(pool, id)
}
