use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub phone: Option<String>,
    #[sea_orm(unique)]
    pub email: Option<String>,
    pub password_hash: Option<String>,
    pub nickname: String,
    pub avatar: Option<String>,
    pub fitness_level: Option<String>,
    pub gender: Option<String>,
    pub birth_date: Option<Date>,
    pub height: Option<Decimal>,
    pub weight: Option<Decimal>,
    #[sea_orm(unique)]
    pub feishu_open_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
