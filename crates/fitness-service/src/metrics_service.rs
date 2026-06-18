use sea_orm::DatabaseConnection;

pub struct MetricsService {
    #[allow(dead_code)]
    db: DatabaseConnection,
}

impl MetricsService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
