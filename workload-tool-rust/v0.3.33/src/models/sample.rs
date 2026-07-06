use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct SampleRecordResponse {
    pub id: i64,
    pub project_id: i64,
    pub project_name: String,
    pub group_id: i64,
    pub group_name: String,
    pub user_name: String,
    pub sample_name: String,
    pub sample_count: i32,
    pub unit: String,
    pub batch_no: String,
    pub notes: String,
    pub submitted_at: String,
    pub created_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SampleRecordCreate {
    pub project_id: i64,
    pub user_name: String,
    pub sample_name: String,
    pub sample_count: i32,
    pub unit: Option<String>,
    pub batch_no: Option<String>,
    pub notes: Option<String>,
    pub submitted_at: String,
}

#[derive(Debug, Deserialize)]
pub struct SampleRecordUpdate {
    pub sample_name: Option<String>,
    pub sample_count: Option<i32>,
    pub unit: Option<String>,
    pub batch_no: Option<String>,
    pub notes: Option<String>,
    pub submitted_at: Option<String>,
}
