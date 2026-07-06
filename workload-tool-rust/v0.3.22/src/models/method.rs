use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct MethodResponse {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub coefficient: f64,
    pub amount: f64,
    pub notes: String,
    pub is_active: bool,
    pub type_ids: Vec<i64>,
    pub type_names: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct MethodCreate {
    pub name: String,
    pub full_name: Option<String>,
    pub coefficient: Option<f64>,
    pub amount: Option<f64>,
    pub notes: Option<String>,
    pub type_ids: Option<Vec<i64>>,
}

#[derive(Debug, Deserialize)]
pub struct MethodUpdate {
    pub name: Option<String>,
    pub full_name: Option<String>,
    pub coefficient: Option<f64>,
    pub amount: Option<f64>,
    pub notes: Option<String>,
    pub is_active: Option<bool>,
    pub type_ids: Option<Vec<i64>>,
}
