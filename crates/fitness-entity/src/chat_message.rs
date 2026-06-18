use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "chat_messages")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub session_id: Uuid,
    pub role: String,
    pub content: String,
    pub token_count: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
