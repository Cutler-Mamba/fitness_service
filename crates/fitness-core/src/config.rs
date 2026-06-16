use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitnessConfig {
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub auth: AuthConfig,
    pub llm: LlmConfig,
    pub feishu: FeishuConfig,
    pub server: ServerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub access_token_ttl_secs: u64,
    pub refresh_token_ttl_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: String,
    pub api_key: String,
    pub api_base: String,
    pub model: String,
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuConfig {
    pub app_id: String,
    pub app_secret: String,
    pub verification_token: String,
    pub encrypt_key: Option<String>,
    pub webhook_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl FitnessConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(FitnessConfig {
            database: DatabaseConfig {
                url: std::env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "sqlite:./data/fitness.db?mode=rwc".to_string()),
                max_connections: std::env::var("DATABASE_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()?,
            },
            redis: RedisConfig {
                url: std::env::var("REDIS_URL")
                    .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            },
            auth: AuthConfig {
                jwt_secret: std::env::var("JWT_SECRET")
                    .unwrap_or_else(|_| "dev-secret-change-in-production".to_string()),
                access_token_ttl_secs: std::env::var("ACCESS_TOKEN_TTL")
                    .unwrap_or_else(|_| "900".to_string())
                    .parse()?,
                refresh_token_ttl_secs: std::env::var("REFRESH_TOKEN_TTL")
                    .unwrap_or_else(|_| "604800".to_string())
                    .parse()?,
            },
            llm: LlmConfig {
                provider: std::env::var("LLM_PROVIDER").unwrap_or_else(|_| "openai".to_string()),
                api_key: std::env::var("LLM_API_KEY").unwrap_or_default(),
                api_base: std::env::var("LLM_API_BASE")
                    .unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
                model: std::env::var("LLM_MODEL").unwrap_or_else(|_| "gpt-4o".to_string()),
                max_tokens: std::env::var("LLM_MAX_TOKENS")
                    .unwrap_or_else(|_| "4096".to_string())
                    .parse()?,
            },
            feishu: FeishuConfig {
                app_id: std::env::var("FEISHU_APP_ID").unwrap_or_default(),
                app_secret: std::env::var("FEISHU_APP_SECRET").unwrap_or_default(),
                verification_token: std::env::var("FEISHU_VERIFICATION_TOKEN").unwrap_or_default(),
                encrypt_key: std::env::var("FEISHU_ENCRYPT_KEY").ok(),
                webhook_path: std::env::var("FEISHU_WEBHOOK_PATH")
                    .unwrap_or_else(|_| "/api/v1/feishu/event".to_string()),
            },
            server: ServerConfig {
                host: std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: std::env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()?,
            },
        })
    }
}
