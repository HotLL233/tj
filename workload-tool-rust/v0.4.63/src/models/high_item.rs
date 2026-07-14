use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone)]
pub struct HighItem {
    pub id: i64,
    pub name: String,
    pub sort_order: i64,
    pub is_active: i64,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct HighItemCreate {
    pub name: String,
    pub sort_order: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct HighItemUpdate {
    pub name: Option<String>,
    pub sort_order: Option<i64>,
    pub is_active: Option<i64>,
}
