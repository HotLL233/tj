use serde::{Deserialize, Serialize};

/// 站内通知 / 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: i64,
    pub recipient: String,
    pub sender: String,
    pub title: String,
    pub content: String,
    pub link: String,
    pub module: String,
    pub is_read: i32,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct NotificationCreate {
    pub recipient: String,
    #[serde(default = "default_sender")] pub sender: String,
    pub title: String,
    #[serde(default)] pub content: String,
    #[serde(default)] pub link: String,
    #[serde(default = "default_module")] pub module: String,
}

#[derive(Debug, Serialize)]
pub struct NotificationResponse {
    pub id: i64,
    pub recipient: String,
    pub sender: String,
    pub title: String,
    pub content: String,
    pub link: String,
    pub module: String,
    pub is_read: i32,
    pub created_at: String,
}

fn default_sender() -> String { "system".to_string() }
fn default_module() -> String { "system".to_string() }
