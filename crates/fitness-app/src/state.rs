use fitness_core::config::FitnessConfig;
use fitness_service::UserService;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tracing::info;

pub struct AppState {
    pub config: Arc<FitnessConfig>,
    #[allow(dead_code)]
    pub db: DatabaseConnection,
    pub user_service: UserService,
}

impl AppState {
    pub async fn new() -> Result<Self, anyhow::Error> {
        dotenvy::dotenv().ok();
        let config = Arc::new(FitnessConfig::from_env()?);

        info!("Connecting to database...");
        let db: DatabaseConnection = sea_orm::Database::connect(&config.database.url).await?;

        let user_service = UserService::new(db.clone());

        Ok(Self {
            config,
            db,
            user_service,
        })
    }
}
