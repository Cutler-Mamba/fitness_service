use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(M20250101_000001CreateUserTable)]
    }
}

pub struct M20250101_000001CreateUserTable;

impl MigrationName for M20250101_000001CreateUserTable {
    fn name(&self) -> &str {
        "m20250101_000001_create_user_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for M20250101_000001CreateUserTable {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                sea_orm_migration::sea_query::Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(User::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(User::Phone).string().unique_key())
                    .col(ColumnDef::new(User::Email).string().unique_key())
                    .col(ColumnDef::new(User::PasswordHash).string())
                    .col(ColumnDef::new(User::Nickname).string().not_null())
                    .col(ColumnDef::new(User::Avatar).string())
                    .col(ColumnDef::new(User::FitnessLevel).string())
                    .col(ColumnDef::new(User::Gender).string())
                    .col(ColumnDef::new(User::BirthDate).date())
                    .col(ColumnDef::new(User::Height).decimal())
                    .col(ColumnDef::new(User::Weight).decimal())
                    .col(ColumnDef::new(User::FeishuOpenId).string().unique_key())
                    .col(
                        ColumnDef::new(User::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(User::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                sea_orm_migration::sea_query::Table::drop()
                    .table(User::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
    Phone,
    Email,
    PasswordHash,
    Nickname,
    Avatar,
    FitnessLevel,
    Gender,
    BirthDate,
    Height,
    Weight,
    FeishuOpenId,
    CreatedAt,
    UpdatedAt,
}
