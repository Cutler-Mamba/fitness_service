use axum::{Json, Router, extract::State, http::HeaderMap, response::IntoResponse, routing::post};
use fitness_core::{error::AppError, types::FitnessProfile};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

use crate::ApiState;

pub fn ai_routes(state: Arc<ApiState>) -> Router {
    Router::new()
        .route("/chat", post(chat))
        .route("/plan", post(generate_plan))
        .route("/nutrition", post(analyze_nutrition))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub profile: Option<FitnessProfile>,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub reply: String,
}

#[derive(Debug, Deserialize)]
pub struct PlanRequest {
    pub profile: FitnessProfile,
}

#[derive(Debug, Deserialize)]
pub struct NutritionRequest {
    pub food_input: String,
    pub profile: Option<FitnessProfile>,
}

async fn chat(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(req): Json<ChatRequest>,
) -> Result<impl IntoResponse, AppError> {
    let _tenant = verify_tenant(&state, &headers).await?;

    let reply = state
        .ai_service
        .chat(&req.message, req.profile.as_ref())
        .await?;

    Ok(Json(json!({ "reply": reply })))
}

async fn generate_plan(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(req): Json<PlanRequest>,
) -> Result<impl IntoResponse, AppError> {
    let _tenant = verify_tenant(&state, &headers).await?;

    let plan = state.ai_service.generate_plan(&req.profile).await?;

    Ok(Json(serde_json::to_value(plan).unwrap_or_default()))
}

async fn analyze_nutrition(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(req): Json<NutritionRequest>,
) -> Result<impl IntoResponse, AppError> {
    let _tenant = verify_tenant(&state, &headers).await?;

    let analysis = state
        .ai_service
        .analyze_nutrition(&req.food_input, req.profile.as_ref())
        .await?;

    Ok(Json(serde_json::to_value(analysis).unwrap_or_default()))
}

fn extract_wechat_user_id(headers: &HeaderMap) -> Result<String, AppError> {
    headers
        .get("X-Wechat-User-Id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::Unauthorized("Missing X-Wechat-User-Id header".into()))
}

async fn verify_tenant(
    state: &ApiState,
    headers: &HeaderMap,
) -> Result<fitness_entity::tenant::Model, AppError> {
    let wechat_user_id = extract_wechat_user_id(headers)?;

    let tenant = state
        .tenant_service
        .find_by_wechat_id(&wechat_user_id)
        .await?
        .ok_or_else(|| AppError::Forbidden("Tenant not found. Please contact admin.".into()))?;

    if tenant.status != "active" {
        return Err(AppError::Forbidden(
            "Tenant is not active. Please contact admin.".into(),
        ));
    }

    Ok(tenant)
}
