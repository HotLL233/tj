use serde::{Deserialize, Serialize};

/// 审批规则（匹配条件 + 审批人 / 审批角色）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRule {
    pub id: i64,
    /// 业务类型：instrument_booking / purchase_requisition / purchase_order / inventory_out
    pub biz_type: String,
    pub name: String,
    /// 申请人角色（可选）
    pub applicant_role: Option<String>,
    /// 申请人用户名（可选）
    pub applicant: Option<String>,
    /// 适配对象类型（可选，如 item_id）
    pub object_type: Option<String>,
    /// 适配对象值（可选）
    pub object_value: Option<String>,
    /// 指定审批角色
    pub approver_role: Option<String>,
    /// 指定审批人
    pub approver: Option<String>,
    pub priority: i32,
    pub is_active: i32,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ApprovalRuleCreate {
    pub biz_type: String,
    #[serde(default)] pub name: String,
    #[serde(default)] pub applicant_role: Option<String>,
    #[serde(default)] pub applicant: Option<String>,
    #[serde(default)] pub object_type: Option<String>,
    #[serde(default)] pub object_value: Option<String>,
    #[serde(default)] pub approver_role: Option<String>,
    #[serde(default)] pub approver: Option<String>,
    #[serde(default)] pub priority: i32,
}

#[derive(Debug, Deserialize)]
pub struct ApprovalRuleUpdate {
    pub biz_type: Option<String>,
    pub name: Option<String>,
    pub applicant_role: Option<Option<String>>,
    pub applicant: Option<Option<String>>,
    pub object_type: Option<Option<String>>,
    pub object_value: Option<Option<String>>,
    pub approver_role: Option<Option<String>>,
    pub approver: Option<Option<String>>,
    pub priority: Option<i32>,
    pub is_active: Option<i32>,
}

/// 审批任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalTask {
    pub id: i64,
    pub biz_type: String,
    pub biz_id: i64,
    pub title: String,
    pub applicant: String,
    pub approver: Option<String>,
    pub approver_role: Option<String>,
    pub status: String,
    pub rule_id: Option<i64>,
    pub decision_note: String,
    pub decided_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ApprovalAction {
    /// approve / reject
    pub decision: String,
    #[serde(default)] pub note: String,
}
