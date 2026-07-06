use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct RecordResponse {
    pub id: i64,
    pub project_id: i64,
    pub project_name: String,
    pub group_name: String,
    pub user_name: String,
    pub quantity: i32,
    pub recorded_at: String,
    pub created_at: String,
    pub deleted_at: Option<String>,
    /// 关联的方法名（取项目关联的第一个方法）
    pub method_name: Option<String>,
    /// 关联的方法类型名（取项目关联方法的所有类型，逗号分隔）
    pub method_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RecordCreate {
    pub project_id: i64,
    pub user_name: String,
    pub quantity: i32,
    pub recorded_at: String,
}

#[derive(Debug, Deserialize)]
pub struct RecordUpdate {
    pub user_name: Option<String>,
    pub quantity: Option<i32>,
    pub recorded_at: Option<String>,
}
