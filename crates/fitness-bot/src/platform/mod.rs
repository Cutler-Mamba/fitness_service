use std::sync::Arc;

use fitness_core::error::AppError;

pub mod ilink_platform;
pub mod wechat_webhook;

#[derive(Debug, Clone)]
pub struct IncomingMessage {
    pub user_id: String,
    pub text: String,
    pub msg_id: String,
    pub context_token: Option<String>,
}

#[async_trait::async_trait]
pub trait MessagingPlatform: Send + Sync {
    async fn start(self: Arc<Self>) -> Result<(), AppError>;

    async fn send_text(
        &self,
        user_id: &str,
        text: &str,
        context_token: Option<&str>,
    ) -> Result<(), AppError>;

    fn name(&self) -> &str;

    fn dedup_key(&self, msg: &IncomingMessage) -> String {
        msg.msg_id.clone()
    }
}
