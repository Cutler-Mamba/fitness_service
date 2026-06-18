use std::sync::Arc;
use std::time::{Duration, Instant};

use fitness_core::config::IlinkConfig;
use fitness_core::error::AppError;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::engine::BotEngine;
use crate::ilink::{IlinkHttpClient, IlinkSession, MAX_TEXT_LENGTH};
use crate::platform::{IncomingMessage, MessagingPlatform};

const MAX_CONSECUTIVE_ERRORS: u32 = 3;
const BACKOFF_SECS: u64 = 30;
const RETRY_DELAY_SECS: u64 = 2;
const SESSION_PAUSE_SECS: u64 = 600;
const TYPING_CACHE_MINUTES: u64 = 10;

pub struct ILinkPlatform {
    client: IlinkHttpClient,
    session: Arc<IlinkSession>,
    engine: RwLock<Option<Arc<BotEngine>>>,
    typing_cache: RwLock<TypingCache>,
    name_str: String,
}

struct TypingCache {
    ticket: Option<String>,
    last_fetch: Instant,
    ttl: Duration,
}

impl TypingCache {
    fn new() -> Self {
        Self {
            ticket: None,
            last_fetch: Instant::now(),
            ttl: Duration::from_secs(TYPING_CACHE_MINUTES * 60),
        }
    }

    fn is_valid(&self) -> bool {
        self.ticket.is_some() && self.last_fetch.elapsed() < self.ttl
    }
}

impl ILinkPlatform {
    pub fn new(config: &IlinkConfig) -> Self {
        let client = IlinkHttpClient::new(config);
        let session = Arc::new(IlinkSession::new(&config.context_dir, &config.account_id));

        Self {
            client,
            session,
            engine: RwLock::new(None),
            typing_cache: RwLock::new(TypingCache::new()),
            name_str: "iLink".to_string(),
        }
    }

    pub fn session(&self) -> Arc<IlinkSession> {
        self.session.clone()
    }

    async fn get_typing_ticket(&self) -> Option<String> {
        {
            let cache = self.typing_cache.read().await;
            if cache.is_valid() {
                return cache.ticket.clone();
            }
        }

        match self.client.get_config().await {
            Ok(config) => {
                let mut cache = self.typing_cache.write().await;
                cache.ticket = config.typing_ticket;
                cache.last_fetch = Instant::now();
                cache.ticket.clone()
            }
            Err(e) => {
                warn!("Failed to get typing ticket: {}", e);
                None
            }
        }
    }

    async fn set_typing(&self, to_user_id: &str, is_typing: bool) {
        let ticket = self.get_typing_ticket().await;
        if let Err(e) = self
            .client
            .send_typing(to_user_id, ticket.as_deref(), is_typing)
            .await
        {
            debug!("Failed to set typing indicator: {}", e);
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
        let mut cursor: Option<String> = self.session.get_poll_cursor();

        loop {
            match self.client.get_updates(cursor.as_deref()).await {
                Ok(updates) => {
                    consecutive_errors = 0;

                    if let Some(next_buf) = &updates.next_buf {
                        if !next_buf.is_empty() {
                            cursor = Some(next_buf.clone());
                            self.session.set_poll_cursor(next_buf.clone());
                        }
                    }

                    if !updates.messages.is_empty() {
                        debug!("Received {} messages", updates.messages.len());

                        if let Some(engine) = self.engine.read().await.as_ref() {
                            for msg in &updates.messages {
                                if let Some(text) = extract_text(msg) {
                                    let incoming = IncomingMessage {
                                        user_id: msg.from_user_id.clone(),
                                        text,
                                        msg_id: msg.msg_id.clone(),
                                        context_token: msg.context_token.clone(),
                                    };

                                    let engine_clone = engine.clone();
                                    let platform: Arc<dyn MessagingPlatform> = self.clone();

                                    self.set_typing(&incoming.user_id, true).await;

                                    tokio::spawn(async move {
                                        engine_clone
                                            .handle_message(platform.as_ref(), incoming)
                                            .await;
                                    });
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    consecutive_errors += 1;
                    let err_str = e.to_string();

                    if err_str.contains("errcode=-14") || err_str.contains("code=-14") {
                        warn!(
                            "iLink session expired (errcode=-14), pausing {}s",
                            SESSION_PAUSE_SECS
                        );
                        tokio::time::sleep(Duration::from_secs(SESSION_PAUSE_SECS)).await;
                        consecutive_errors = 0;
                        continue;
                    }

                    if consecutive_errors <= 2 {
                        warn!(
                            "iLink poll error ({}/{}): {}, retrying in {}s",
                            consecutive_errors, MAX_CONSECUTIVE_ERRORS, e, RETRY_DELAY_SECS
                        );
                        tokio::time::sleep(Duration::from_secs(RETRY_DELAY_SECS)).await;
                    } else {
                        error!(
                            "iLink poll persistent error ({} consecutive), backing off {}s",
                            consecutive_errors, BACKOFF_SECS
                        );
                        tokio::time::sleep(Duration::from_secs(BACKOFF_SECS)).await;
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
        let chunks = crate::engine::split_text(text, MAX_TEXT_LENGTH);
        let total = chunks.len();

        for (i, chunk) in chunks.iter().enumerate() {
            let ctx = if i == 0 { context_token } else { None };
            self.client.send_text(user_id, chunk, ctx).await?;

            if total > 1 && i < total - 1 {
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

fn extract_text(msg: &crate::ilink::IlinkMessage) -> Option<String> {
    if let Some(push_content) = &msg.push_content {
        if !push_content.is_empty() {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(push_content) {
                if let Some(text) = parsed.get("text").and_then(|v| v.as_str()) {
                    return Some(text.to_string());
                }
            }
            return Some(push_content.clone());
        }
    }

    msg.content.clone().filter(|c| !c.is_empty())
}
