use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BotCommand {
    Plan { goal: Option<String> },
    Log { content: String },
    Diet { request: Option<String> },
    Stats,
    Help,
    Unknown { text: String },
}

pub fn parse_command(text: &str) -> BotCommand {
    let text = text.trim();

    if text.starts_with("/plan") {
        let goal = text.strip_prefix("/plan").map(|s| s.trim().to_string());
        return BotCommand::Plan { goal };
    }

    if text.starts_with("/log") {
        let content = text.strip_prefix("/log").unwrap_or("").trim().to_string();
        return BotCommand::Log { content };
    }

    if text.starts_with("/diet") {
        let request = text.strip_prefix("/diet").map(|s| s.trim().to_string());
        return BotCommand::Diet { request };
    }

    if text.starts_with("/stats") {
        return BotCommand::Stats;
    }

    if text.starts_with("/help") {
        return BotCommand::Help;
    }

    BotCommand::Unknown {
        text: text.to_string(),
    }
}
