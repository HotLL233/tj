use serde::{Deserialize, Serialize};

/// 采购申请
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseRequisition {
    pub id: i64,
    pub requester: String,
    pub item_name: String,
    pub spec: String,
    pub quantity: f64,
    pub unit: String,
    pub purpose: String,
    pub expected_supplier: String,
    pub status: String,
    pub approval_task_id: Option<i64>,
    pub created_by: String,
    pub created_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RequisitionCreate {
    pub item_name: String,
    #[serde(default)] pub spec: String,
    pub quantity: f64,
    #[serde(default)] pub unit: String,
    #[serde(default)] pub purpose: String,
    #[serde(default)] pub expected_supplier: String,
}

/// 采购单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseOrder {
    pub id: i64,
    pub order_no: String,
    pub supplier_id: Option<i64>,
    pub requisition_ids: String,
    pub total_amount: f64,
    pub status: String,
    pub approval_task_id: Option<i64>,
    pub sent_at: Option<String>,
    pub received_at: Option<String>,
    pub note: String,
    pub created_by: String,
    pub created_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OrderCreate {
    pub supplier_id: Option<i64>,
    #[serde(default)] pub requisition_ids: Vec<i64>,
    pub items: Vec<OrderItemCreate>,
    #[serde(default)] pub note: String,
}

#[derive(Debug, Deserialize)]
pub struct OrderItemCreate {
    pub item_name: String,
    #[serde(default)] pub spec: String,
    pub quantity: f64,
    #[serde(default)] pub unit_price: f64,
    /// 关联采购申请ID（可选，用于耗材溯源）
    #[serde(default)] pub requisition_id: Option<i64>,
}

/// 采购单明细
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseOrderItem {
    pub id: i64,
    pub order_id: i64,
    pub item_name: String,
    pub spec: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub amount: f64,
    pub requisition_id: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct OrderResponse {
    pub id: i64,
    pub order_no: String,
    pub supplier_id: Option<i64>,
    pub supplier_name: String,
    pub requisition_ids: String,
    pub total_amount: f64,
    pub status: String,
    pub approval_task_id: Option<i64>,
    pub sent_at: Option<String>,
    pub received_at: Option<String>,
    pub note: String,
    pub created_by: String,
    pub created_at: String,
    pub items: Vec<PurchaseOrderItem>,
}

/// 审批动作（approve / reject），供审批中心复用
#[derive(Debug, Deserialize)]
pub struct ApprovalDecision {
    pub decision: String,
    #[serde(default)] pub note: String,
}
