pub mod middleware;

pub mod ai_handler;
pub mod auth_handler;
pub mod stub_handler;
pub mod user_handler;

use axum::Router;
use fitness_core::config::FitnessConfig;
use fitness_service::{AiService, TenantService, UserService};
use std::sync::Arc;

#[derive(Clone)]
pub struct ApiState {
    pub user_service: Arc<UserService>,
    pub tenant_service: Arc<TenantService>,
    pub ai_service: Arc<AiService>,
    pub config: Arc<FitnessConfig>,
}

impl ApiState {
    pub fn new(
        user_service: UserService,
        tenant_service: TenantService,
        ai_service: AiService,
        config: Arc<FitnessConfig>,
    ) -> Self {
        Self {
            user_service: Arc::new(user_service),
            tenant_service: Arc::new(tenant_service),
            ai_service: Arc::new(ai_service),
            config,
        }
    }
}

pub fn api_router(state: Arc<ApiState>) -> Router {
    Router::new()
        .nest("/api/v1/auth", auth_handler::auth_routes(state.clone()))
        .nest("/api/v1/users", user_handler::user_routes(state.clone()))
        .nest("/api/v1/ai", ai_handler::ai_routes(state.clone()))
        .nest("/api/v1/plans", stub_handler::plan_routes(state.clone()))
        .nest("/api/v1/exercises", stub_handler::exercise_routes(state.clone()))
        .nest("/api/v1/nutrition", stub_handler::nutrition_routes(state.clone()))
        .nest("/api/v1/metrics", stub_handler::metrics_routes(state))
}
