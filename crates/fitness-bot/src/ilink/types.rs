use serde::{Deserialize, Serialize};

pub const ILINK_APP_ID: &str = "bot";
pub const ILINK_APP_CLIENT_VERSION: u32 = 131584;
pub const CHANNEL_VERSION: &str = "2.2.0";

pub const EP_GET_UPDATES: &str = "ilink/bot/getupdates";
pub const EP_SEND_MESSAGE: &str = "ilink/bot/sendmessage";
pub const EP_SEND_TYPING: &str = "ilink/bot/sendtyping";
pub const EP_GET_CONFIG: &str = "ilink/bot/getconfig";
pub const EP_GET_BOT_QR: &str = "ilink/bot/get_bot_qrcode";
pub const EP_GET_QR_STATUS: &str = "ilink/bot/get_qrcode_status";

pub const MSG_TYPE_USER: i32 = 1;
pub const MSG_TYPE_BOT: i32 = 2;
pub const MSG_STATE_FINISH: i32 = 2;

pub const ITEM_TEXT: i32 = 1;
pub const ITEM_IMAGE: i32 = 2;
pub const ITEM_VOICE: i32 = 3;
pub const ITEM_FILE: i32 = 4;
pub const ITEM_VIDEO: i32 = 5;

pub const TYPING_START: i32 = 1;
pub const TYPING_STOP: i32 = 2;

pub const SESSION_EXPIRED_ERRCODE: i32 = -14;

pub const MAX_MESSAGE_LENGTH: usize = 2000;
pub const LONG_POLL_TIMEOUT_MS: u64 = 35_000;
pub const API_TIMEOUT_MS: u64 = 15_000;
pub const CONFIG_TIMEOUT_MS: u64 = 10_000;
pub const QR_TIMEOUT_MS: u64 = 35_000;
pub const MAX_CONSECUTIVE_FAILURES: u32 = 3;
pub const RETRY_DELAY_SECS: u64 = 2;
pub const BACKOFF_DELAY_SECS: u64 = 30;
pub const SESSION_PAUSE_SECS: u64 = 600;
pub const TYPING_TTL_SECONDS: f64 = 600.0;

#[derive(Debug, Clone, Serialize)]
pub struct BaseInfo {
    pub channel_version: String,
}

impl Default for BaseInfo {
    fn default() -> Self {
        Self {
            channel_version: CHANNEL_VERSION.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiRequest<T: Serialize> {
    #[serde(flatten)]
    pub payload: T,
    pub base_info: BaseInfo,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetUpdatesResponse {
    #[serde(default)]
    pub ret: i32,
    #[serde(default)]
    pub errcode: i32,
    #[serde(default)]
    pub errmsg: Option<String>,
    #[serde(default)]
    pub msgs: Vec<IlinkMessage>,
    #[serde(default)]
    pub get_updates_buf: String,
    #[serde(default)]
    pub longpolling_timeout_ms: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IlinkMessage {
    pub message_id: Option<String>,
    pub from_user_id: Option<String>,
    pub to_user_id: Option<String>,
    pub item_list: Option<Vec<MessageItem>>,
    pub context_token: Option<String>,
    pub create_time: Option<i64>,
    pub message_type: Option<i32>,
    pub message_state: Option<i32>,
    pub room_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessageItem {
    #[serde(rename = "type")]
    pub item_type: Option<i32>,
    pub text_item: Option<TextItem>,
    pub voice_item: Option<VoiceItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TextItem {
    pub text: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VoiceItem {
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SendMessagePayload {
    pub msg: OutboundMessage,
}

#[derive(Debug, Clone, Serialize)]
pub struct OutboundMessage {
    pub from_user_id: String,
    pub to_user_id: String,
    pub client_id: String,
    pub message_type: i32,
    pub message_state: i32,
    pub item_list: Vec<OutboundItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_token: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OutboundItem {
    #[serde(rename = "type")]
    pub item_type: i32,
    pub text_item: OutboundTextItem,
}

#[derive(Debug, Clone, Serialize)]
pub struct OutboundTextItem {
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TypingPayload {
    pub ilink_user_id: String,
    pub typing_ticket: String,
    pub status: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct GetConfigPayload {
    pub ilink_user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_token: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetConfigResponse {
    pub typing_ticket: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QrCodeResponse {
    pub qrcode: Option<String>,
    pub qrcode_img_content: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QrStatusResponse {
    pub status: Option<String>,
    pub ilink_bot_id: Option<String>,
    pub bot_token: Option<String>,
    pub baseurl: Option<String>,
    pub ilink_user_id: Option<String>,
    pub redirect_host: Option<String>,
}

pub fn extract_text(msg: &IlinkMessage) -> Option<String> {
    let items = msg.item_list.as_ref()?;
    for item in items {
        if item.item_type == Some(ITEM_TEXT) {
            if let Some(text_item) = &item.text_item {
                if let Some(text) = &text_item.text {
                    if !text.is_empty() {
                        return Some(text.clone());
                    }
                }
            }
        }
        if item.item_type == Some(ITEM_VOICE) {
            if let Some(voice) = &item.voice_item {
                if let Some(text) = &voice.text {
                    if !text.is_empty() {
                        return Some(text.clone());
                    }
                }
            }
        }
    }
    None
}
