use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: String,
    pub email: String,
    pub password: String,
    pub email_password: String,
    pub smtp_server: String,
    pub smtp_port: u16,
    pub last_login_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserSession {
    pub account_id: String,
    pub cookies: Option<String>,
    pub local_storage: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationCode {
    pub code: String,
    pub timestamp: i64,
    pub from: String,
    pub subject: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EmailStatus {
    Idle,
    Connecting,
    Connected,
    Receiving,
    Error,
    Stopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailReceiverStatus {
    pub status: EmailStatus,
    pub error_message: Option<String>,
    pub last_check_time: Option<i64>,
    pub codes_count: usize,
}