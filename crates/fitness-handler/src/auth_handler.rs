use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use fitness_core::{error::AppError, types::JwtClaims};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::ApiState;

pub fn auth_routes(state: Arc<ApiState>) -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub password: String,
    pub nickname: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub nickname: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub avatar: Option<String>,
    pub fitness_level: Option<String>,
}

impl From<fitness_entity::user::Model> for UserResponse {
    fn from(u: fitness_entity::user::Model) -> Self {
        Self {
            id: u.id,
            nickname: u.nickname,
            email: u.email,
            phone: u.phone,
            avatar: u.avatar,
            fitness_level: u.fitness_level,
        }
    }
}

async fn register(
    State(state): State<Arc<ApiState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user = state
        .user_service
        .register(req.email, req.phone, req.password, req.nickname)
        .await?;

    let access_token = generate_token(user.id, &state.config, false)?;
    let refresh_token = generate_token(user.id, &state.config, true)?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "access_token": access_token,
            "refresh_token": refresh_token,
            "user": UserResponse::from(user),
        })),
    ))
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub account: String,
    pub password: String,
}

async fn login(
    State(state): State<Arc<ApiState>>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user = state
        .user_service
        .login(&req.account, &req.password)
        .await?;

    let access_token = generate_token(user.id, &state.config, false)?;
    let refresh_token = generate_token(user.id, &state.config, true)?;

    Ok(Json(json!({
        "access_token": access_token,
        "refresh_token": refresh_token,
        "user": UserResponse::from(user),
    })))
}

fn generate_token(
    user_id: Uuid,
    config: &fitness_core::config::FitnessConfig,
    is_refresh: bool,
) -> Result<String, AppError> {
    let now = chrono::Utc::now();
    let ttl = if is_refresh {
        config.auth.refresh_token_ttl_secs
    } else {
        config.auth.access_token_ttl_secs
    };

    let exp = (now + chrono::Duration::seconds(ttl as i64)).timestamp() as usize;
    let claims = JwtClaims {
        sub: user_id,
        exp,
        iat: now.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.auth.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Failed to generate token: {}", e)))
}
