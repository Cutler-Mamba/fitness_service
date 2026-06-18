use axum::Router;
use std::{net::SocketAddr, sync::Arc};
use tracing::info;

mod state;

use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "fitness_app=info,fitness_core=info,tower_http=info".into()),
        )
        .init();

    info!("Starting AI Fitness Assistant...");

    let app_state = AppState::new().await?;
    let config = app_state.config.clone();

    let handler_state = Arc::new(fitness_handler::ApiState::new(
        app_state.user_service,
        app_state.tenant_service,
        app_state.ai_service,
        config.clone(),
    ));

    let api_router = fitness_handler::api_router(handler_state);
    let wechat_router = fitness_bot::wechat_router(Arc::new(config.wechat.clone()));

    let app = Router::new()
        .merge(api_router)
        .nest("/api/v1/wechat", wechat_router);

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;

    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
