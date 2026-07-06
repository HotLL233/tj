use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AuditLogResponse {
    pub id: i64,
    pub action: String,
    pub table_name: String,
    pub record_id: Option<i64>,
    pub user_name: String,
    pub detail: String,
    pub created_at: String,
}
