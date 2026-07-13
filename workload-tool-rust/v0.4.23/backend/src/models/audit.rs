use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AuditLogResponse {
    pub id: i64,
    pub action: String,
    pub table_name: String,
    pub record_id: Option<i64>,
    pub user_name: String,
    pub detail: String,
    pub module: String,
    /// 字段级 diff：变更前 JSON（新增于 v0.4.21）
    pub before_json: Option<String>,
    /// 字段级 diff：变更后 JSON（新增于 v0.4.21）
    pub after_json: Option<String>,
    pub created_at: String,
}
