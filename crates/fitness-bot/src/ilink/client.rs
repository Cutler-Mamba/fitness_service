use std::time::Duration;

use fitness_core::{config::IlinkConfig, error::AppError};
use reqwest::Client;
use tracing::{debug, info, warn};

use super::types::*;

const DEFAULT_ILINK_URL: &str = "https://ilinkai.weixin.qq.com";

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

    fn api_url(&self, path: &str) -> String {
        format!("{}/{}", self.base_url, path.trim_start_matches('/'))
    }

    fn auth_params(&self) -> Vec<(&str, &str)> {
        vec![("token", &self.token)]
    }

    pub async fn get_updates(&self, buf: Option<&str>) -> Result<GetUpdatesResponse, AppError> {
        let mut url = self.api_url("getupdates");

        if let Some(b) = buf {
            url = format!("{}?buf={}", url, b);
        }

        debug!("iLink getupdates polling...");

        let resp = self
            .client
            .get(&url)
            .query(&self.auth_params())
            .send()
            .await
            .map_err(|e| AppError::WechatError(format!("getupdates request failed: {}", e)))?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .map_err(|e| AppError::WechatError(format!("getupdates read body: {}", e)))?;

        if !status.is_success() {
            return Err(AppError::WechatError(format!(
                "getupdates HTTP {}: {}",
                status, body
            )));
        }

        let wrapper: ILinkResponse = serde_json::from_str(&body).map_err(|e| {
            AppError::WechatError(format!("getupdates parse response: {} body: {}", e, body))
        })?;

        if wrapper.code != 0 {
            return Err(AppError::WechatError(format!(
                "getupdates API error code={}: {:?}",
                wrapper.code, wrapper.msg
            )));
        }

        let data = wrapper
            .data
            .ok_or_else(|| AppError::WechatError("getupdates empty data".into()))?;

        serde_json::from_value::<GetUpdatesResponse>(data).map_err(|e| {
            AppError::WechatError(format!("getupdates parse data: {} raw: {}", e, body))
        })
    }

    pub async fn send_text(
        &self,
        to_user_id: &str,
        text: &str,
        context_token: Option<&str>,
    ) -> Result<Option<String>, AppError> {
        let req = SendMessageRequest {
            to_user_id: to_user_id.to_string(),
            msg_type: MSG_TYPE_TEXT,
            content: text.to_string(),
            context_token: context_token.map(|s| s.to_string()),
        };

        let body = serde_json::to_string(&req)
            .map_err(|e| AppError::WechatError(format!("serialize send: {}", e)))?;

        debug!("iLink send to={} len={}", to_user_id, text.len());

        let resp = self
            .client
            .post(self.api_url("send"))
            .query(&self.auth_params())
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .map_err(|e| AppError::WechatError(format!("send request failed: {}", e)))?;

        let status = resp.status();
        let resp_body = resp
            .text()
            .await
            .map_err(|e| AppError::WechatError(format!("send read body: {}", e)))?;

        if !status.is_success() {
            return Err(AppError::WechatError(format!(
                "send HTTP {}: {}",
                status, resp_body
            )));
        }

        let wrapper: ILinkResponse = serde_json::from_str(&resp_body).map_err(|e| {
            AppError::WechatError(format!("send parse response: {} body: {}", e, resp_body))
        })?;

        if wrapper.code != 0 {
            return Err(AppError::WechatError(format!(
                "send API error code={}: {:?}",
                wrapper.code, wrapper.msg
            )));
        }

        if let Some(data) = wrapper.data {
            let send_resp: SendMessageResponse = serde_json::from_value(data)
                .map_err(|e| AppError::WechatError(format!("send parse data: {}", e)))?;
            Ok(send_resp.msg_id)
        } else {
            Ok(None)
        }
    }

    pub async fn get_config(&self) -> Result<GetConfigResponse, AppError> {
        let resp = self
            .client
            .get(self.api_url("getconfig"))
            .query(&self.auth_params())
            .send()
            .await
            .map_err(|e| AppError::WechatError(format!("getconfig request: {}", e)))?;

        let body = resp
            .text()
            .await
            .map_err(|e| AppError::WechatError(format!("getconfig body: {}", e)))?;

        let wrapper: ILinkResponse = serde_json::from_str(&body).map_err(|e| {
            AppError::WechatError(format!("getconfig parse: {} body: {}", e, body))
        })?;

        if wrapper.code != 0 {
            warn!("iLink getconfig returned code={}", wrapper.code);
            return Ok(GetConfigResponse {
                typing_ticket: None,
            });
        }

        if let Some(data) = wrapper.data {
            serde_json::from_value::<GetConfigResponse>(data).map_err(|e| {
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
        to_user_id: &str,
        typing_ticket: Option<&str>,
        is_typing: bool,
    ) -> Result<(), AppError> {
        let ticket = match typing_ticket {
            Some(t) => t,
            None => return Ok(()),
        };

        let command = if is_typing { "Typing" } else { "CancelTyping" };
        let req = TypingRequest {
            to_user_id: to_user_id.to_string(),
            command: command.to_string(),
        };

        let body = serde_json::to_string(&req).map_err(|e| {
            AppError::WechatError(format!("serialize typing: {}", e))
        })?;

        let resp = self
            .client
            .post(self.api_url("send"))
            .query(&self.auth_params())
            .query(&[("typing_ticket", ticket)])
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .map_err(|e| AppError::WechatError(format!("typing request: {}", e)))?;

        if !resp.status().is_success() {
            warn!("iLink typing failed HTTP {}", resp.status());
        }

        Ok(())
    }

    pub async fn verify_token(&self) -> Result<bool, AppError> {
        match self.get_config().await {
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
}
