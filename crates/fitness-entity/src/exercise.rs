use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "exercises")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub plan_id: Option<Uuid>,
    pub name: String,
    pub sets: i32,
    pub reps: String,
    pub rest_seconds: i32,
    pub notes: Option<String>,
    pub order_index: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
