use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct RdRecordResponse {
    pub id: i64,
    pub project_id: i64,
    pub method_id: Option<i64>,
    pub project_name: String,
    pub group_name: String,
    pub user_name: String,
    pub quantity: i32,
    pub recorded_at: String,
    pub created_at: String,
    pub deleted_at: Option<String>,
    pub method_name: Option<String>,
    pub method_type: Option<String>,
    pub status: String,
    pub sampler: Option<String>,
    pub sampled_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RdSampleUpdate {
    pub sampler: String,
}
