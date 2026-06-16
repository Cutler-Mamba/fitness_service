pub mod middleware;

pub mod auth_handler;
pub mod user_handler;

use axum::Router;
use fitness_core::config::FitnessConfig;
use fitness_service::UserService;
use std::sync::Arc;

#[derive(Clone)]
pub struct ApiState {
    pub user_service: Arc<UserService>,
    pub config: Arc<FitnessConfig>,
}

impl ApiState {
    pub fn new(user_service: UserService, config: Arc<FitnessConfig>) -> Self {
        Self {
            user_service: Arc::new(user_service),
            config,
        }
    }
}

pub fn api_router(state: Arc<ApiState>) -> Router {
    Router::new()
        .nest("/api/v1/auth", auth_handler::auth_routes(state.clone()))
        .nest("/api/v1/users", user_handler::user_routes(state))
}
