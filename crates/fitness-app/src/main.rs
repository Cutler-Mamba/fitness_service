use axum::Router;
use fitness_core::config::WechatChannel;
use fitness_bot::platform::MessagingPlatform;
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
        app_state.user_service.clone(),
        app_state.tenant_service.clone(),
        app_state.ai_service.clone(),
        config.clone(),
    ));

    let api_router = fitness_handler::api_router(handler_state);

    let mut app = Router::new().merge(api_router);

    match &config.wechat.channel {
        WechatChannel::ILink(ilink_config) => {
            if !ilink_config.account_id.is_empty() && !ilink_config.token.is_empty() {
                info!(
                    "Starting iLink long-polling bot (account: {})",
                    ilink_config.account_id
                );

                let ilink_platform = Arc::new(fitness_bot::ILinkPlatform::new(ilink_config));
                let session = ilink_platform.session();

                let bot_engine = Arc::new(fitness_bot::BotEngine::new(
                    app_state.ai_service.clone(),
                    app_state.tenant_service.clone(),
                    session,
                ));

                tokio::spawn(async move {
                    if let Err(e) = ilink_platform.start().await {
                        tracing::error!("iLink platform exited with error: {}", e);
                    }
                });

                drop(bot_engine);
            } else {
                info!(
                    "iLink channel configured but missing credentials (account_id/token), skipping"
                );
            }
        }
        WechatChannel::Webhook(webhook_config) => {
            info!("Starting WeChat webhook mode");
            let webhook_router =
                fitness_bot::wechat_webhook_router(Arc::new(webhook_config.clone()));
            app = app.nest("/api/v1/wechat", webhook_router);
        }
    }

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;

    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
