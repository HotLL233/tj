use serde::{Deserialize, Serialize};

/// 供应商
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Supplier {
    pub id: i64,
    pub name: String,
    pub contact: String,
    pub phone: String,
    pub email: String,
    pub qualification: String,
    pub status: String,
    pub notes: String,
    pub created_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SupplierCreate {
    pub name: String,
    #[serde(default)] pub contact: String,
    #[serde(default)] pub phone: String,
    #[serde(default)] pub email: String,
    #[serde(default)] pub qualification: String,
    #[serde(default)] pub notes: String,
}

#[derive(Debug, Deserialize)]
pub struct SupplierUpdate {
    pub name: Option<String>,
    pub contact: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub qualification: Option<String>,
    pub status: Option<String>,
    pub notes: Option<String>,
}
