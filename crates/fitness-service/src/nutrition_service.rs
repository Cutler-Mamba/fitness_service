use sea_orm::DatabaseConnection;

pub struct NutritionService {
    #[allow(dead_code)]
    db: DatabaseConnection,
}

impl NutritionService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
