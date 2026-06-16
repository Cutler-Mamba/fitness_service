use axum::{Json, Router, extract::State, routing::post};
use fitness_core::{config::FeishuConfig, error::AppError};
use serde_json::{Value, json};
use std::sync::Arc;
use tracing::info;

pub fn feishu_router(config: Arc<FeishuConfig>) -> Router {
    Router::new()
        .route("/event", post(handle_event))
        .route("/challenge", post(handle_challenge))
        .with_state(config)
}

async fn handle_challenge(
    State(_config): State<Arc<FeishuConfig>>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, AppError> {
    if let Some(challenge) = body.get("challenge").and_then(|v| v.as_str()) {
        info!("Feishu URL challenge received");
        return Ok(Json(json!({ "challenge": challenge })));
    }
    Err(AppError::BadRequest("Invalid challenge request".into()))
}

#[derive(serde::Deserialize)]
struct FeishuEvent {
    header: Option<FeishuEventHeader>,
}

#[derive(serde::Deserialize)]
struct FeishuEventHeader {
    token: Option<String>,
}

async fn handle_event(
    State(config): State<Arc<FeishuConfig>>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, AppError> {
    info!(
        "Feishu event received: {}",
        serde_json::to_string(&body).unwrap_or_default()
    );

    if let Some(challenge) = body.get("challenge").and_then(|v| v.as_str()) {
        return Ok(Json(json!({ "challenge": challenge })));
    }

    if let Ok(event) = serde_json::from_value::<FeishuEvent>(body) {
        if let Some(header) = &event.header {
            if let Some(token) = &header.token {
                if token != &config.verification_token {
                    return Err(AppError::Forbidden("Invalid verification token".into()));
                }
            }
        }
    }

    Ok(Json(json!({ "code": 0 })))
}
