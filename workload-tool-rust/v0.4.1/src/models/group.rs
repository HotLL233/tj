use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct GroupResponse {
    pub id: i64,
    pub name: String,
    pub sort_order: i64,
    pub created_at: String,
    pub project_count: i64,
    pub project_names: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GroupCreate {
    pub name: String,
    pub sort_order: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct GroupUpdate {
    pub name: Option<String>,
    pub sort_order: Option<i64>,
}
