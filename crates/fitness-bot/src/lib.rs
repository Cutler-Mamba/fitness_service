pub mod command;
pub mod engine;
pub mod ilink;
pub mod platform;

pub use command::parse_command;
pub use engine::BotEngine;
pub use platform::{
    ilink_platform::ILinkPlatform,
    wechat_webhook::wechat_webhook_router,
};
