use sea_orm::DatabaseConnection;

pub struct PlanService {
    #[allow(dead_code)]
    db: DatabaseConnection,
}

impl PlanService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
