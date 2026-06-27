use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportRecord {
    pub project_name: String,
    pub group_name: String,
    pub recorded_at: String,     // YYYY-MM-DD
    pub batch_no: String,
    pub quantity: i64,
    pub user_name: Option<String>,
    pub extra_info: Option<String>,
}
