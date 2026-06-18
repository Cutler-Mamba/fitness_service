use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "nutrition_logs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub meal_type: String,
    pub food_name: String,
    pub amount: Option<Decimal>,
    pub unit: Option<String>,
    pub calories: Option<Decimal>,
    pub protein_g: Option<Decimal>,
    pub carbs_g: Option<Decimal>,
    pub fat_g: Option<Decimal>,
    pub notes: Option<String>,
    pub logged_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
