use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use fitness_core::{config::IlinkConfig, error::AppError};
use reqwest::Client;
use serde::Serialize;
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::types::*;

const DEFAULT_ILINK_URL: &str = "https://ilinkai.weixin.qq.com";

static UIN_COUNTER: AtomicU64 = AtomicU64::new(0);

fn random_wechat_uin() -> String {
    let val = UIN_COUNTER.fetch_add(1, Ordering::Relaxed);
    use std::io::Write;
    let mut buf = Vec::new();
    write!(&mut buf, "{}", val % 4_000_000_000u64).unwrap();
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(&buf)
}

pub struct IlinkHttpClient {
    client: Client,
    base_url: String,
    #[allow(dead_code)]
    account_id: String,
    token: String,
}

impl IlinkHttpClient {
    pub fn new(config: &IlinkConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(40))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: if config.ilink_url.is_empty() {
                DEFAULT_ILINK_URL.to_string()
            } else {
                config.ilink_url.trim_end_matches('/').to_string()
            },
            account_id: config.account_id.clone(),
            token: config.token.clone(),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    fn api_url(&self, path: &str) -> String {
        format!("{}/{}", self.base_url, path.trim_start_matches('/'))
    }

    fn headers(&self, body: &str) -> Vec<(&str, String)> {
        vec![
            ("Content-Type", "application/json".into()),
            ("AuthorizationType", "ilink_bot_token".into()),
            ("Content-Length", body.len().to_string()),
            ("X-WECHAT-UIN", random_wechat_uin()),
            ("iLink-App-Id", ILINK_APP_ID.into()),
            (
                "iLink-App-ClientVersion",
                ILINK_APP_CLIENT_VERSION.to_string(),
            ),
            ("Authorization", format!("Bearer {}", self.token)),
        ]
    }

    async fn api_post<T: Serialize>(
        &self,
        endpoint: &str,
        payload: &T,
    ) -> Result<reqwest::Response, AppError> {
        let mut body = serde_json::to_value(payload)
            .map_err(|e| AppError::WechatError(format!("serialize: {}", e)))?;
        if let Some(obj) = body.as_object_mut() {
            obj.insert(
                "base_info".into(),
                serde_json::to_value(BaseInfo::default()).unwrap(),
            );
        }
        let body_str = serde_json::to_string(&body)
            .map_err(|e| AppError::WechatError(format!("serialize: {}", e)))?;

        let url = self.api_url(endpoint);
        let headers = self.headers(&body_str);

        let mut req = self.client.post(&url);
        for (k, v) in &headers {
            req = req.header(*k, v);
        }

        req.body(body_str)
            .send()
            .await
            .map_err(|e| AppError::WechatError(format!("{} request failed: {}", endpoint, e)))
    }

    #[allow(dead_code)]
    async fn api_get(&self, endpoint: &str) -> Result<reqwest::Response, AppError> {
        let url = self.api_url(endpoint);
        self.client
            .get(&url)
            .header("iLink-App-Id", ILINK_APP_ID)
            .header(
                "iLink-App-ClientVersion",
                ILINK_APP_CLIENT_VERSION.to_string(),
            )
            .send()
            .await
            .map_err(|e| AppError::WechatError(format!("{} request failed: {}", endpoint, e)))
    }

    async fn check_response(
        resp: reqwest::Response,
        endpoint: &str,
    ) -> Result<String, AppError> {
        let status = resp.status();
        let body = resp
            .text()
            .await
            .map_err(|e| AppError::WechatError(format!("{} read body: {}", endpoint, e)))?;

        if !status.is_success() {
            return Err(AppError::WechatError(format!(
                "{} HTTP {}: {}",
                endpoint,
                status,
                &body[..body.len().min(200)]
            )));
        }

        Ok(body)
    }

    pub async fn get_updates(&self, buf: Option<&str>) -> Result<GetUpdatesResponse, AppError> {
        #[derive(Serialize)]
        struct GetUpdatesPayload {
            get_updates_buf: String,
        }

        let payload = GetUpdatesPayload {
            get_updates_buf: buf.unwrap_or("").to_string(),
        };

        debug!("iLink getupdates polling...");

        let resp = self.api_post(EP_GET_UPDATES, &payload).await?;
        let body = Self::check_response(resp, "getupdates").await?;

        let response: GetUpdatesResponse = serde_json::from_str(&body).map_err(|e| {
            AppError::WechatError(format!("getupdates parse: {} body: {}", e, body))
        })?;

        Ok(response)
    }

    pub async fn send_text(
        &self,
        to_user_id: &str,
        text: &str,
        context_token: Option<&str>,
    ) -> Result<(), AppError> {
        if text.trim().is_empty() {
            return Ok(());
        }

        let msg = OutboundMessage {
            from_user_id: String::new(),
            to_user_id: to_user_id.to_string(),
            client_id: Uuid::new_v4().to_string(),
            message_type: MSG_TYPE_BOT,
            message_state: MSG_STATE_FINISH,
            item_list: vec![OutboundItem {
                item_type: ITEM_TEXT,
                text_item: OutboundTextItem {
                    text: text.to_string(),
                },
            }],
            context_token: context_token.map(|s| s.to_string()),
        };

        let payload = SendMessagePayload { msg };

        debug!("iLink send to={} len={}", to_user_id, text.len());

        let resp = self.api_post(EP_SEND_MESSAGE, &payload).await;
        match resp {
            Ok(resp) => {
                let body = Self::check_response(resp, "sendmessage").await?;
                debug!("iLink send response: {}", &body[..body.len().min(200)]);
                Ok(())
            }
            Err(e) => {
                warn!("iLink send error: {}", e);
                Err(e)
            }
        }
    }

    pub async fn get_config(
        &self,
        user_id: &str,
        context_token: Option<&str>,
    ) -> Result<GetConfigResponse, AppError> {
        let payload = GetConfigPayload {
            ilink_user_id: user_id.to_string(),
            context_token: context_token.map(|s| s.to_string()),
        };

        let resp = self.api_post(EP_GET_CONFIG, &payload).await?;
        let body = Self::check_response(resp, "getconfig").await?;

        let wrapper: serde_json::Value = serde_json::from_str(&body).map_err(|e| {
            AppError::WechatError(format!("getconfig parse: {} body: {}", e, body))
        })?;

        if let Some(data) = wrapper.get("data") {
            serde_json::from_value::<GetConfigResponse>(data.clone()).map_err(|e| {
                AppError::WechatError(format!("getconfig parse data: {}", e))
            })
        } else {
            Ok(GetConfigResponse {
                typing_ticket: None,
            })
        }
    }

    pub async fn send_typing(
        &self,
        user_id: &str,
        typing_ticket: &str,
        is_typing: bool,
    ) -> Result<(), AppError> {
        let payload = TypingPayload {
            ilink_user_id: user_id.to_string(),
            typing_ticket: typing_ticket.to_string(),
            status: if is_typing {
                TYPING_START
            } else {
                TYPING_STOP
            },
        };

        match self.api_post(EP_SEND_TYPING, &payload).await {
            Ok(resp) => {
                let _ = Self::check_response(resp, "sendtyping").await;
                Ok(())
            }
            Err(e) => {
                debug!("iLink typing failed: {}", e);
                Ok(())
            }
        }
    }

    pub async fn verify_token(&self) -> Result<bool, AppError> {
        match self.get_config("", None).await {
            Ok(_) => {
                info!("iLink token verified successfully");
                Ok(true)
            }
            Err(e) => {
                warn!("iLink token verification failed: {}", e);
                Ok(false)
            }
        }
    }

    pub async fn get_qr_code() -> Result<QrCodeResponse, AppError> {
        let url = format!(
            "{}/{}?bot_type=3",
            DEFAULT_ILINK_URL, EP_GET_BOT_QR
        );

        let client = Client::builder()
            .timeout(Duration::from_millis(QR_TIMEOUT_MS))
            .build()
            .map_err(|e| AppError::WechatError(format!("client: {}", e)))?;

        let resp = client
            .get(&url)
            .header("iLink-App-Id", ILINK_APP_ID)
            .header(
                "iLink-App-ClientVersion",
                ILINK_APP_CLIENT_VERSION.to_string(),
            )
            .send()
            .await
            .map_err(|e| AppError::WechatError(format!("qr request: {}", e)))?;

        let body = IlinkHttpClient::check_response(resp, "get_bot_qrcode").await?;

        serde_json::from_str::<QrCodeResponse>(&body)
            .map_err(|e| AppError::WechatError(format!("qr parse: {} body: {}", e, body)))
    }

    pub async fn check_qr_status(qrcode: &str) -> Result<QrStatusResponse, AppError> {
        let url = format!(
            "{}/{}?qrcode={}",
            DEFAULT_ILINK_URL, EP_GET_QR_STATUS, qrcode
        );

        let client = Client::builder()
            .timeout(Duration::from_millis(QR_TIMEOUT_MS))
            .build()
            .map_err(|e| AppError::WechatError(format!("client: {}", e)))?;

        let resp = client
            .get(&url)
            .header("iLink-App-Id", ILINK_APP_ID)
            .header(
                "iLink-App-ClientVersion",
                ILINK_APP_CLIENT_VERSION.to_string(),
            )
            .send()
            .await
            .map_err(|e| AppError::WechatError(format!("qr status request: {}", e)))?;

        let body = IlinkHttpClient::check_response(resp, "get_qrcode_status").await?;

        serde_json::from_str::<QrStatusResponse>(&body)
            .map_err(|e| AppError::WechatError(format!("qr status parse: {} body: {}", e, body)))
    }

    pub fn save_credentials(config_dir: &str, account_id: &str, token: &str, base_url: &str) {
        use std::fs;
        use std::path::Path;

        let dir = Path::new(config_dir);
        fs::create_dir_all(dir).ok();

        let file_path = dir.join(format!("{}.json", account_id));
        let data = serde_json::json!({
            "account_id": account_id,
            "token": token,
            "base_url": base_url,
            "saved_at": chrono::Utc::now().to_rfc3339(),
        });

        if let Ok(json) = serde_json::to_string_pretty(&data) {
            if let Err(e) = fs::write(&file_path, &json) {
                warn!("Failed to save credentials: {}", e);
            }
        }
    }
}
