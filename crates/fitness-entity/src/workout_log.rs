use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "workout_logs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub plan_id: Option<Uuid>,
    pub exercise_name: String,
    pub sets_completed: Option<i32>,
    pub reps_completed: Option<String>,
    pub weight_kg: Option<Decimal>,
    pub duration_seconds: Option<i32>,
    pub notes: Option<String>,
    pub logged_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
