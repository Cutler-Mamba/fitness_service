use std::sync::Arc;
use std::time::{Duration, Instant};

use fitness_core::config::IlinkConfig;
use fitness_core::error::AppError;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::engine::BotEngine;
use crate::ilink::client::IlinkHttpClient;
use crate::ilink::session::IlinkSession;
use crate::ilink::types::{
    extract_text, GetUpdatesResponse, MAX_CONSECUTIVE_FAILURES, RETRY_DELAY_SECS,
    BACKOFF_DELAY_SECS, SESSION_EXPIRED_ERRCODE, SESSION_PAUSE_SECS, TYPING_TTL_SECONDS,
};
use crate::platform::{IncomingMessage, MessagingPlatform};

pub struct ILinkPlatform {
    client: IlinkHttpClient,
    session: Arc<IlinkSession>,
    engine: RwLock<Option<Arc<BotEngine>>>,
    typing_ticket: RwLock<Option<(String, Instant)>>,
    name_str: String,
}

impl ILinkPlatform {
    pub fn new(config: &IlinkConfig) -> Self {
        let client = IlinkHttpClient::new(config);
        let session = Arc::new(IlinkSession::new(&config.context_dir, &config.account_id));

        Self {
            client,
            session,
            engine: RwLock::new(None),
            typing_ticket: RwLock::new(None),
            name_str: "iLink".to_string(),
        }
    }

    pub fn session(&self) -> Arc<IlinkSession> {
        self.session.clone()
    }

    pub fn set_engine(&self, engine: Arc<BotEngine>) {
        // Use try_write to avoid deadlock; in practice this is called before the poll loop starts
        if let Ok(mut guard) = self.engine.try_write() {
            *guard = Some(engine);
        }
    }

    async fn get_typing_ticket(&self, user_id: &str) -> Option<String> {
        {
            let cache = self.typing_ticket.read().await;
            if let Some((ticket, ts)) = &*cache {
                if ts.elapsed() < Duration::from_secs_f64(TYPING_TTL_SECONDS) {
                    return Some(ticket.clone());
                }
            }
        }

        match self.client.get_config(user_id, None).await {
            Ok(config) => {
                if let Some(ticket) = config.typing_ticket {
                    let mut cache = self.typing_ticket.write().await;
                    *cache = Some((ticket.clone(), Instant::now()));
                    return Some(ticket);
                }
                None
            }
            Err(e) => {
                warn!("Failed to get typing ticket: {}", e);
                None
            }
        }
    }

    async fn set_typing(&self, to_user_id: &str, is_typing: bool) {
        let ticket = self.get_typing_ticket(to_user_id).await;
        if let Some(t) = ticket {
            let _ = self.client.send_typing(to_user_id, &t, is_typing).await;
        }
    }
}

#[async_trait::async_trait]
impl MessagingPlatform for ILinkPlatform {
    async fn start(self: Arc<Self>) -> Result<(), AppError> {
        info!("iLink platform starting...");

        let verified = self.client.verify_token().await.unwrap_or(false);
        if !verified {
            warn!("iLink token verification failed, will continue polling anyway");
        }

        info!("iLink platform started, entering poll loop");

        let mut consecutive_errors: u32 = 0;
        let mut cursor = self.session.get_poll_cursor();

        loop {
            match self.client.get_updates(cursor.as_deref()).await {
                Ok(updates) => {
                    consecutive_errors = 0;

                    if handle_updates(self.clone(), &updates).await {
                        if !updates.get_updates_buf.is_empty() {
                            cursor = Some(updates.get_updates_buf.clone());
                            self.session
                                .set_poll_cursor(updates.get_updates_buf.clone());
                        }
                    }
                }
                Err(e) => {
                    consecutive_errors += 1;
                    let err_str = e.to_string();

                    if err_str.contains(&format!("errcode={}", SESSION_EXPIRED_ERRCODE))
                        || err_str.contains(&format!("code={}", SESSION_EXPIRED_ERRCODE))
                    {
                        warn!(
                            "iLink session expired, pausing {}s",
                            SESSION_PAUSE_SECS
                        );
                        tokio::time::sleep(Duration::from_secs(SESSION_PAUSE_SECS)).await;
                        consecutive_errors = 0;
                        continue;
                    }

                    if consecutive_errors <= MAX_CONSECUTIVE_FAILURES {
                        warn!(
                            "iLink poll error ({}/{}): {}, retrying in {}s",
                            consecutive_errors,
                            MAX_CONSECUTIVE_FAILURES + 1,
                            e,
                            RETRY_DELAY_SECS
                        );
                        tokio::time::sleep(Duration::from_secs(RETRY_DELAY_SECS)).await;
                    } else {
                        error!(
                            "iLink poll persistent error ({} consecutive), backing off {}s",
                            consecutive_errors, BACKOFF_DELAY_SECS
                        );
                        tokio::time::sleep(Duration::from_secs(BACKOFF_DELAY_SECS)).await;
                        consecutive_errors = 0;
                    }
                }
            }
        }
    }

    async fn send_text(
        &self,
        user_id: &str,
        text: &str,
        context_token: Option<&str>,
    ) -> Result<(), AppError> {
        let chunks = crate::engine::split_text(text, crate::ilink::types::MAX_MESSAGE_LENGTH);

        for (i, chunk) in chunks.iter().enumerate() {
            let ctx = if i == 0 { context_token } else { None };
            self.client.send_text(user_id, chunk, ctx).await?;

            if chunks.len() > 1 && i < chunks.len() - 1 {
                tokio::time::sleep(Duration::from_millis(300)).await;
            }
        }

        self.set_typing(user_id, false).await;

        Ok(())
    }

    fn name(&self) -> &str {
        &self.name_str
    }
}

async fn handle_updates(platform: Arc<ILinkPlatform>, resp: &GetUpdatesResponse) -> bool {
    if resp.msgs.is_empty() {
        return false;
    }

    debug!("Received {} messages", resp.msgs.len());

    if let Some(engine) = platform.engine.read().await.as_ref() {
        for msg in &resp.msgs {
            if let Some(from_user_id) = &msg.from_user_id {
                if from_user_id == "" {
                    continue;
                }

                if let Some(text) = extract_text(msg) {
                    let msg_id = msg
                        .message_id
                        .clone()
                        .unwrap_or_else(|| Uuid::new_v4().to_string());

                    let incoming = IncomingMessage {
                        user_id: from_user_id.clone(),
                        text,
                        msg_id,
                        context_token: msg.context_token.clone(),
                    };

                    let engine_clone = engine.clone();
                    let platform_clone = platform.clone();

                    platform.set_typing(&incoming.user_id, true).await;

                    tokio::spawn(async move {
                        engine_clone
                            .handle_message(platform_clone.as_ref() as &dyn MessagingPlatform, incoming)
                            .await;
                    });
                }
            }
        }
    }

    true
}

use uuid::Uuid;
