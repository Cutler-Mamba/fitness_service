use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "body_metrics")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub weight_kg: Option<Decimal>,
    pub body_fat_pct: Option<Decimal>,
    pub muscle_mass_kg: Option<Decimal>,
    pub waist_cm: Option<Decimal>,
    pub hip_cm: Option<Decimal>,
    pub chest_cm: Option<Decimal>,
    pub notes: Option<String>,
    pub measured_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
