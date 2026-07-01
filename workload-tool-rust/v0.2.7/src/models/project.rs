use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ProjectResponse {
    pub id: i64, pub group_id: i64, pub group_name: String,
    pub name: String, pub full_name: String, pub notes: String,
    pub sort_order: i64, pub is_active: bool,
    pub coefficient: f64, pub method_type: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ProjectCreate {
    pub group_id: i64, pub name: String,
    pub sort_order: Option<i64>, pub coefficient: Option<f64>,
    pub method_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProjectUpdate {
    pub name: Option<String>, pub full_name: Option<String>,
    pub notes: Option<String>, pub sort_order: Option<i64>,
    pub is_active: Option<bool>, pub coefficient: Option<f64>,
    pub method_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MethodImportItem {
    pub group_name: String, pub name: String,
    pub coefficient: f64, pub method_type: String,
}
