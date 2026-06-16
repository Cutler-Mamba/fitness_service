use axum::{Json, Router, extract::State, http::HeaderMap, response::IntoResponse, routing::get};
use fitness_core::error::AppError;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use crate::{ApiState, middleware::auth::extract_user_id};

pub fn user_routes(state: Arc<ApiState>) -> Router {
    Router::new()
        .route("/me", get(get_me).put(update_me))
        .with_state(state)
}

async fn get_me(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let user_id = auth_from_headers(&headers, &state.config.auth.jwt_secret)?;
    let user = state.user_service.find_by_id(user_id).await?;
    Ok(Json(json!({
        "id": user.id,
        "nickname": user.nickname,
        "email": user.email,
        "phone": user.phone,
        "avatar": user.avatar,
        "fitness_level": user.fitness_level,
        "gender": user.gender,
        "birth_date": user.birth_date,
        "height": user.height,
        "weight": user.weight,
        "created_at": user.created_at,
        "updated_at": user.updated_at,
    })))
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub nickname: Option<String>,
    pub avatar: Option<String>,
    pub fitness_level: Option<String>,
    pub gender: Option<String>,
    pub birth_date: Option<String>,
    pub height: Option<f64>,
    pub weight: Option<f64>,
}

async fn update_me(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = auth_from_headers(&headers, &state.config.auth.jwt_secret)?;

    let birth_date = req
        .birth_date
        .as_deref()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

    let height = req
        .height
        .map(|h| rust_decimal::Decimal::from_str_exact(&h.to_string()).unwrap_or_default());
    let weight = req
        .weight
        .map(|w| rust_decimal::Decimal::from_str_exact(&w.to_string()).unwrap_or_default());

    let user = state
        .user_service
        .update_profile(
            user_id,
            req.nickname,
            req.avatar,
            req.fitness_level,
            req.gender,
            birth_date,
            height,
            weight,
        )
        .await?;

    Ok(Json(json!({
        "id": user.id,
        "nickname": user.nickname,
        "email": user.email,
        "phone": user.phone,
        "avatar": user.avatar,
        "fitness_level": user.fitness_level,
        "gender": user.gender,
        "birth_date": user.birth_date,
        "height": user.height,
        "weight": user.weight,
        "updated_at": user.updated_at,
    })))
}

fn auth_from_headers(headers: &HeaderMap, jwt_secret: &str) -> Result<uuid::Uuid, AppError> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".into()))?;
    extract_user_id(auth_header, jwt_secret)
}
