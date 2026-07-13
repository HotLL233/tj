use serde::{Deserialize, Serialize};

/// 库存分类
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryCategory {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub sort_order: i32,
}

#[derive(Debug, Deserialize)]
pub struct CategoryCreate {
    pub name: String,
    #[serde(default)] pub parent_id: Option<i64>,
    #[serde(default)] pub sort_order: i32,
}

#[derive(Debug, Deserialize)]
pub struct CategoryUpdate {
    pub name: Option<String>,
    pub parent_id: Option<Option<i64>>,
    pub sort_order: Option<i32>,
}

/// 库存物料
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub id: i64,
    pub name: String,
    pub brand: String,
    pub unit: String,
    pub category_id: Option<i64>,
    pub tags: String,
    pub location: String,
    pub spec: String,
    pub safety_stock: f64,
    pub expiry_threshold_days: i32,
    pub created_by: String,
    pub created_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ItemCreate {
    pub name: String,
    #[serde(default)] pub brand: String,
    #[serde(default = "default_unit")] pub unit: String,
    #[serde(default)] pub category_id: Option<i64>,
    #[serde(default)] pub tags: String,
    #[serde(default)] pub location: String,
    #[serde(default)] pub spec: String,
    #[serde(default)] pub safety_stock: f64,
    #[serde(default)] pub expiry_threshold_days: i32,
}

#[derive(Debug, Deserialize)]
pub struct ItemUpdate {
    pub name: Option<String>,
    pub brand: Option<String>,
    pub unit: Option<String>,
    pub category_id: Option<Option<i64>>,
    pub tags: Option<String>,
    pub location: Option<String>,
    pub spec: Option<String>,
    pub safety_stock: Option<f64>,
    pub expiry_threshold_days: Option<i32>,
}

/// 物料对外返回（含实时库存与分类名）
#[derive(Debug, Serialize)]
pub struct ItemResponse {
    pub id: i64,
    pub name: String,
    pub brand: String,
    pub unit: String,
    pub category_id: Option<i64>,
    pub category_name: String,
    pub tags: String,
    pub location: String,
    pub spec: String,
    pub safety_stock: f64,
    pub expiry_threshold_days: i32,
    pub current_quantity: f64,
    pub created_by: String,
    pub created_at: String,
}

/// 批次（同一物料可有多批次，含效期/单价）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryBatch {
    pub id: i64,
    pub item_id: i64,
    pub batch_no: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub produced_at: Option<String>,
    pub expiry_date: Option<String>,
    pub source_type: String,
    pub source_id: Option<i64>,
    pub created_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BatchCreate {
    pub item_id: i64,
    #[serde(default)] pub batch_no: String,
    pub quantity: f64,
    #[serde(default)] pub unit_price: f64,
    #[serde(default)] pub produced_at: Option<String>,
    #[serde(default)] pub expiry_date: Option<String>,
    /// 入库来源：purchase / manual 等
    #[serde(default = "default_source")] pub source_type: String,
    #[serde(default)] pub source_id: Option<i64>,
}

/// 库存流水（in 入库 / out 出库 / adjust 调整 / scrap 报废）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryTransaction {
    pub id: i64,
    pub item_id: i64,
    pub batch_id: Option<i64>,
    pub tx_type: String,
    pub quantity: f64,
    pub applicant: String,
    pub approver: String,
    pub approval_task_id: Option<i64>,
    pub related_id: Option<i64>,
    pub note: String,
    pub created_by: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct TransactionCreate {
    pub item_id: i64,
    /// in / out / adjust / scrap
    pub tx_type: String,
    pub quantity: f64,
    #[serde(default)] pub batch_id: Option<i64>,
    #[serde(default)] pub note: String,
    #[serde(default)] pub related_id: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct TransactionResponse {
    pub id: i64,
    pub item_id: i64,
    pub item_name: String,
    pub batch_id: Option<i64>,
    pub tx_type: String,
    pub quantity: f64,
    pub applicant: String,
    pub approver: String,
    pub approval_task_id: Option<i64>,
    pub related_id: Option<i64>,
    pub note: String,
    pub created_by: String,
    pub created_at: String,
}

fn default_unit() -> String { "个".to_string() }
fn default_source() -> String { "manual".to_string() }
