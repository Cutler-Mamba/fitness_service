use axum::Router;
use std::sync::Arc;

use crate::ApiState;

pub fn plan_routes(_state: Arc<ApiState>) -> Router {
    Router::new()
}

pub fn exercise_routes(_state: Arc<ApiState>) -> Router {
    Router::new()
}

pub fn nutrition_routes(_state: Arc<ApiState>) -> Router {
    Router::new()
}

pub fn metrics_routes(_state: Arc<ApiState>) -> Router {
    Router::new()
}
