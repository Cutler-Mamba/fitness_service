use axum::{Json, Router, extract::State, routing::post};
use fitness_core::{config::WechatConfig, error::AppError};
use serde_json::{Value, json};
use std::sync::Arc;
use tracing::info;

pub fn wechat_router(config: Arc<WechatConfig>) -> Router {
    Router::new()
        .route("/event", post(handle_event))
        .route("/challenge", post(handle_challenge))
        .with_state(config)
}

async fn handle_challenge(
    State(_config): State<Arc<WechatConfig>>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, AppError> {
    if let Some(challenge) = body.get("challenge").and_then(|v| v.as_str()) {
        info!("WeChat URL challenge received");
        return Ok(Json(json!({ "challenge": challenge })));
    }
    Err(AppError::BadRequest("Invalid challenge request".into()))
}

#[derive(serde::Deserialize)]
struct WechatEvent {
    token: Option<String>,
    user_id: Option<String>,
    text: Option<String>,
}

async fn handle_event(
    State(config): State<Arc<WechatConfig>>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, AppError> {
    info!(
        "WeChat event received: {}",
        serde_json::to_string(&body).unwrap_or_default()
    );

    if let Some(challenge) = body.get("challenge").and_then(|v| v.as_str()) {
        return Ok(Json(json!({ "challenge": challenge })));
    }

    let event: WechatEvent = serde_json::from_value(body.clone())
        .map_err(|_| AppError::BadRequest("Invalid event format".into()))?;

    if let Some(token) = &event.token {
        if token != &config.verification_token {
            return Err(AppError::Forbidden("Invalid verification token".into()));
        }
    }

    if let Some(user_id) = &event.user_id {
        if let Some(text) = &event.text {
            info!("WeChat message from {}: {}", user_id, text);
        }
    }

    Ok(Json(json!({ "code": 0 })))
}
