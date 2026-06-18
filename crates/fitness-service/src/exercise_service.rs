use sea_orm::DatabaseConnection;

pub struct ExerciseService {
    #[allow(dead_code)]
    db: DatabaseConnection,
}

impl ExerciseService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
