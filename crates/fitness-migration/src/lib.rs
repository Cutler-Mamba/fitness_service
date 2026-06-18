use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(M20250101_000001CreateUserTable),
            Box::new(M20250101_000002CreateTenantTable),
            Box::new(M20250101_000003CreateWorkoutPlanTable),
            Box::new(M20250101_000004CreateExerciseTable),
            Box::new(M20250101_000005CreateWorkoutLogTable),
            Box::new(M20250101_000006CreateNutritionLogTable),
            Box::new(M20250101_000007CreateBodyMetricsTable),
            Box::new(M20250101_000008CreateChatSessionTable),
            Box::new(M20250101_000009CreateChatMessageTable),
        ]
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

pub struct M20250101_000002CreateTenantTable;

impl MigrationName for M20250101_000002CreateTenantTable {
    fn name(&self) -> &str {
        "m20250101_000002_create_tenant_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for M20250101_000002CreateTenantTable {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                sea_orm_migration::sea_query::Table::create()
                    .table(Tenant::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Tenant::Id).uuid().not_null().primary_key())
                    .col(
                        ColumnDef::new(Tenant::WechatUserId)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Tenant::Nickname).string())
                    .col(
                        ColumnDef::new(Tenant::Status)
                            .string()
                            .not_null()
                            .default("disabled"),
                    )
                    .col(ColumnDef::new(Tenant::DailyQuota).integer())
                    .col(
                        ColumnDef::new(Tenant::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Tenant::UpdatedAt)
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
                    .table(Tenant::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Tenant {
    Table,
    Id,
    WechatUserId,
    Nickname,
    Status,
    DailyQuota,
    CreatedAt,
    UpdatedAt,
}

pub struct M20250101_000003CreateWorkoutPlanTable;

impl MigrationName for M20250101_000003CreateWorkoutPlanTable {
    fn name(&self) -> &str {
        "m20250101_000003_create_workout_plan_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for M20250101_000003CreateWorkoutPlanTable {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                sea_orm_migration::sea_query::Table::create()
                    .table(WorkoutPlan::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(WorkoutPlan::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(WorkoutPlan::UserId).uuid().not_null())
                    .col(ColumnDef::new(WorkoutPlan::Name).string().not_null())
                    .col(ColumnDef::new(WorkoutPlan::Goal).string().not_null())
                    .col(ColumnDef::new(WorkoutPlan::Difficulty).string())
                    .col(
                        ColumnDef::new(WorkoutPlan::DurationWeeks)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(WorkoutPlan::Schedule).json())
                    .col(ColumnDef::new(WorkoutPlan::Tips).json())
                    .col(
                        ColumnDef::new(WorkoutPlan::Status)
                            .string()
                            .not_null()
                            .default("active"),
                    )
                    .col(
                        ColumnDef::new(WorkoutPlan::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(WorkoutPlan::UpdatedAt)
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
                    .table(WorkoutPlan::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum WorkoutPlan {
    Table,
    Id,
    UserId,
    Name,
    Goal,
    Difficulty,
    DurationWeeks,
    Schedule,
    Tips,
    Status,
    CreatedAt,
    UpdatedAt,
}

pub struct M20250101_000004CreateExerciseTable;

impl MigrationName for M20250101_000004CreateExerciseTable {
    fn name(&self) -> &str {
        "m20250101_000004_create_exercise_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for M20250101_000004CreateExerciseTable {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                sea_orm_migration::sea_query::Table::create()
                    .table(Exercise::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Exercise::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Exercise::PlanId).uuid())
                    .col(ColumnDef::new(Exercise::Name).string().not_null())
                    .col(ColumnDef::new(Exercise::Sets).integer().not_null())
                    .col(ColumnDef::new(Exercise::Reps).string().not_null())
                    .col(ColumnDef::new(Exercise::RestSeconds).integer().not_null())
                    .col(ColumnDef::new(Exercise::Notes).string())
                    .col(ColumnDef::new(Exercise::OrderIndex).integer().not_null())
                    .col(
                        ColumnDef::new(Exercise::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Exercise::UpdatedAt)
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
                    .table(Exercise::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Exercise {
    Table,
    Id,
    PlanId,
    Name,
    Sets,
    Reps,
    RestSeconds,
    Notes,
    OrderIndex,
    CreatedAt,
    UpdatedAt,
}

pub struct M20250101_000005CreateWorkoutLogTable;

impl MigrationName for M20250101_000005CreateWorkoutLogTable {
    fn name(&self) -> &str {
        "m20250101_000005_create_workout_log_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for M20250101_000005CreateWorkoutLogTable {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                sea_orm_migration::sea_query::Table::create()
                    .table(WorkoutLog::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(WorkoutLog::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(WorkoutLog::UserId).uuid().not_null())
                    .col(ColumnDef::new(WorkoutLog::PlanId).uuid())
                    .col(ColumnDef::new(WorkoutLog::ExerciseName).string().not_null())
                    .col(ColumnDef::new(WorkoutLog::SetsCompleted).integer())
                    .col(ColumnDef::new(WorkoutLog::RepsCompleted).string())
                    .col(ColumnDef::new(WorkoutLog::WeightKg).decimal())
                    .col(ColumnDef::new(WorkoutLog::DurationSeconds).integer())
                    .col(ColumnDef::new(WorkoutLog::Notes).string())
                    .col(
                        ColumnDef::new(WorkoutLog::LoggedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(WorkoutLog::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(WorkoutLog::UpdatedAt)
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
                    .table(WorkoutLog::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum WorkoutLog {
    Table,
    Id,
    UserId,
    PlanId,
    ExerciseName,
    SetsCompleted,
    RepsCompleted,
    WeightKg,
    DurationSeconds,
    Notes,
    LoggedAt,
    CreatedAt,
    UpdatedAt,
}

pub struct M20250101_000006CreateNutritionLogTable;

impl MigrationName for M20250101_000006CreateNutritionLogTable {
    fn name(&self) -> &str {
        "m20250101_000006_create_nutrition_log_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for M20250101_000006CreateNutritionLogTable {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                sea_orm_migration::sea_query::Table::create()
                    .table(NutritionLog::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(NutritionLog::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(NutritionLog::UserId).uuid().not_null())
                    .col(ColumnDef::new(NutritionLog::MealType).string().not_null())
                    .col(ColumnDef::new(NutritionLog::FoodName).string().not_null())
                    .col(ColumnDef::new(NutritionLog::Amount).decimal())
                    .col(ColumnDef::new(NutritionLog::Unit).string())
                    .col(ColumnDef::new(NutritionLog::Calories).decimal())
                    .col(ColumnDef::new(NutritionLog::ProteinG).decimal())
                    .col(ColumnDef::new(NutritionLog::CarbsG).decimal())
                    .col(ColumnDef::new(NutritionLog::FatG).decimal())
                    .col(ColumnDef::new(NutritionLog::Notes).string())
                    .col(
                        ColumnDef::new(NutritionLog::LoggedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(NutritionLog::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(NutritionLog::UpdatedAt)
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
                    .table(NutritionLog::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum NutritionLog {
    Table,
    Id,
    UserId,
    MealType,
    FoodName,
    Amount,
    Unit,
    Calories,
    ProteinG,
    CarbsG,
    FatG,
    Notes,
    LoggedAt,
    CreatedAt,
    UpdatedAt,
}

pub struct M20250101_000007CreateBodyMetricsTable;

impl MigrationName for M20250101_000007CreateBodyMetricsTable {
    fn name(&self) -> &str {
        "m20250101_000007_create_body_metrics_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for M20250101_000007CreateBodyMetricsTable {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                sea_orm_migration::sea_query::Table::create()
                    .table(BodyMetrics::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(BodyMetrics::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(BodyMetrics::UserId).uuid().not_null())
                    .col(ColumnDef::new(BodyMetrics::WeightKg).decimal())
                    .col(ColumnDef::new(BodyMetrics::BodyFatPct).decimal())
                    .col(ColumnDef::new(BodyMetrics::MuscleMassKg).decimal())
                    .col(ColumnDef::new(BodyMetrics::WaistCm).decimal())
                    .col(ColumnDef::new(BodyMetrics::HipCm).decimal())
                    .col(ColumnDef::new(BodyMetrics::ChestCm).decimal())
                    .col(ColumnDef::new(BodyMetrics::Notes).string())
                    .col(
                        ColumnDef::new(BodyMetrics::MeasuredAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BodyMetrics::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BodyMetrics::UpdatedAt)
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
                    .table(BodyMetrics::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum BodyMetrics {
    Table,
    Id,
    UserId,
    WeightKg,
    BodyFatPct,
    MuscleMassKg,
    WaistCm,
    HipCm,
    ChestCm,
    Notes,
    MeasuredAt,
    CreatedAt,
    UpdatedAt,
}

pub struct M20250101_000008CreateChatSessionTable;

impl MigrationName for M20250101_000008CreateChatSessionTable {
    fn name(&self) -> &str {
        "m20250101_000008_create_chat_session_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for M20250101_000008CreateChatSessionTable {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                sea_orm_migration::sea_query::Table::create()
                    .table(ChatSession::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ChatSession::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ChatSession::UserId).uuid().not_null())
                    .col(ColumnDef::new(ChatSession::Title).string())
                    .col(
                        ColumnDef::new(ChatSession::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ChatSession::UpdatedAt)
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
                    .table(ChatSession::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum ChatSession {
    Table,
    Id,
    UserId,
    Title,
    CreatedAt,
    UpdatedAt,
}

pub struct M20250101_000009CreateChatMessageTable;

impl MigrationName for M20250101_000009CreateChatMessageTable {
    fn name(&self) -> &str {
        "m20250101_000009_create_chat_message_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for M20250101_000009CreateChatMessageTable {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                sea_orm_migration::sea_query::Table::create()
                    .table(ChatMessage::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ChatMessage::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ChatMessage::SessionId).uuid().not_null())
                    .col(ColumnDef::new(ChatMessage::Role).string().not_null())
                    .col(ColumnDef::new(ChatMessage::Content).string().not_null())
                    .col(ColumnDef::new(ChatMessage::TokenCount).integer())
                    .col(
                        ColumnDef::new(ChatMessage::CreatedAt)
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
                    .table(ChatMessage::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum ChatMessage {
    Table,
    Id,
    SessionId,
    Role,
    Content,
    TokenCount,
    CreatedAt,
}
