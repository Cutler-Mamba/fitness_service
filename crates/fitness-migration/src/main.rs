#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    sea_orm_migration::cli::run_cli(fitness_migration::Migrator).await;
}
