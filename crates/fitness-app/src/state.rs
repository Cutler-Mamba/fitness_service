use fitness_core::config::FitnessConfig;
use fitness_llm::LlmClient;
use fitness_service::{AiService, TenantService, UserService};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tracing::info;

pub struct AppState {
    pub config: Arc<FitnessConfig>,
    #[allow(dead_code)]
    pub db: DatabaseConnection,
    pub user_service: UserService,
    pub tenant_service: TenantService,
    pub ai_service: AiService,
}

impl AppState {
    pub async fn new() -> Result<Self, anyhow::Error> {
        dotenvy::dotenv().ok();
        let config = Arc::new(FitnessConfig::from_env()?);

        info!("Connecting to database...");
        let db: DatabaseConnection = sea_orm::Database::connect(&config.database.url).await?;

        let user_service = UserService::new(db.clone());
        let tenant_service = TenantService::new(db.clone());

        info!("Initializing LLM client...");
        let llm_client = LlmClient::new(&config.llm);
        let ai_service = AiService::new(llm_client);

        Ok(Self {
            config,
            db,
            user_service,
            tenant_service,
            ai_service,
        })
    }
}
