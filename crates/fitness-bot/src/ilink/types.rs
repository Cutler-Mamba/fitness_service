use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ILinkResponse {
    pub code: i32,
    pub msg: Option<String>,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetUpdatesResponse {
    pub messages: Vec<IlinkMessage>,
    pub has_more: Option<bool>,
    pub next_buf: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IlinkMessage {
    pub msg_id: String,
    pub from_user_id: String,
    pub to_user_id: Option<String>,
    pub msg_type: Option<i32>,
    pub content: Option<String>,
    pub context_token: Option<String>,
    #[serde(rename = "createTime")]
    pub create_time: Option<i64>,
    pub push_content: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SendMessageRequest {
    pub to_user_id: String,
    pub msg_type: i32,
    pub content: String,
    pub context_token: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SendMessageResponse {
    pub msg_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TypingRequest {
    pub to_user_id: String,
    pub command: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetConfigResponse {
    pub typing_ticket: Option<String>,
}

pub const MSG_TYPE_TEXT: i32 = 1;
pub const MSG_TYPE_IMAGE: i32 = 2;
pub const MSG_TYPE_VOICE: i32 = 3;
pub const MSG_TYPE_VIDEO: i32 = 4;
pub const MSG_TYPE_FILE: i32 = 5;

pub const MAX_TEXT_LENGTH: usize = 4000;
pub const POLL_TIMEOUT_SECS: u64 = 35;
pub const CHUNK_DELAY_MS: u64 = 300;
