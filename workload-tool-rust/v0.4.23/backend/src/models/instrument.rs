use serde::{Deserialize, Serialize};

/// 仪器档案（软删：deleted_at 非空表示已删除）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instrument {
    pub id: i64,
    pub name: String,
    pub model: String,
    pub location: String,
    pub manager: String,
    pub status: String,
    pub photo_path: String,
    pub qr_code_path: String,
    pub notes: String,
    pub created_by: String,
    pub created_at: String,
    pub deleted_at: Option<String>,
}

/// 新建仪器请求（created_by 由服务端填充）
#[derive(Debug, Deserialize)]
pub struct InstrumentCreate {
    pub name: String,
    #[serde(default)] pub model: String,
    #[serde(default)] pub location: String,
    #[serde(default)] pub manager: String,
    #[serde(default = "default_status")] pub status: String,
    #[serde(default)] pub notes: String,
}

/// 更新仪器请求（全可选）
#[derive(Debug, Deserialize)]
pub struct InstrumentUpdate {
    pub name: Option<String>,
    pub model: Option<String>,
    pub location: Option<String>,
    pub manager: Option<String>,
    pub status: Option<String>,
    pub notes: Option<String>,
}

/// 对外返回的仪器（含预约/保养计数，便于前端展示）
#[derive(Debug, Serialize)]
pub struct InstrumentResponse {
    pub id: i64,
    pub name: String,
    pub model: String,
    pub location: String,
    pub manager: String,
    pub status: String,
    pub photo_path: String,
    pub qr_code_path: String,
    pub notes: String,
    pub created_by: String,
    pub created_at: String,
}

/// 仪器预约
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentBooking {
    pub id: i64,
    pub instrument_id: i64,
    pub applicant: String,
    pub start_time: String,
    pub end_time: String,
    pub purpose: String,
    pub status: String,
    pub approver: Option<String>,
    pub approved_at: Option<String>,
    pub approver_note: String,
    pub created_at: String,
    pub deleted_at: Option<String>,
}

/// 提交预约请求
#[derive(Debug, Deserialize)]
pub struct BookingCreate {
    pub instrument_id: i64,
    pub applicant: String,
    pub start_time: String,
    pub end_time: String,
    #[serde(default)] pub purpose: String,
}

/// 审批预约请求（approve / reject）
#[derive(Debug, Deserialize)]
pub struct BookingDecision {
    pub decision: String, // "approve" | "reject"
    #[serde(default)] pub note: String,
}

/// 预约对外返回（含仪器名）
#[derive(Debug, Serialize)]
pub struct BookingResponse {
    pub id: i64,
    pub instrument_id: i64,
    pub instrument_name: String,
    pub applicant: String,
    pub start_time: String,
    pub end_time: String,
    pub purpose: String,
    pub status: String,
    pub approver: Option<String>,
    pub approved_at: Option<String>,
    pub approver_note: String,
    pub created_at: String,
}

/// 仪器保养记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentMaintenance {
    pub id: i64,
    pub instrument_id: i64,
    pub maintainer: String,
    pub maintained_at: String,
    pub content: String,
    pub cost: f64,
    pub created_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MaintenanceCreate {
    pub instrument_id: i64,
    pub maintainer: String,
    pub maintained_at: String,
    #[serde(default)] pub content: String,
    #[serde(default)] pub cost: f64,
}

#[derive(Debug, Serialize)]
pub struct MaintenanceResponse {
    pub id: i64,
    pub instrument_id: i64,
    pub instrument_name: String,
    pub maintainer: String,
    pub maintained_at: String,
    pub content: String,
    pub cost: f64,
    pub created_at: String,
}

fn default_status() -> String { "正常".to_string() }
