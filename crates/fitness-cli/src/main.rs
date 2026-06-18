use clap::{Parser, Subcommand};
use fitness_core::config::FitnessConfig;
use fitness_service::TenantService;
use sea_orm::Database;

#[derive(Parser)]
#[command(name = "fitness-cli")]
#[command(about = "AI Fitness Assistant CLI management tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Tenant {
        #[command(subcommand)]
        action: TenantAction,
    },
}

#[derive(Subcommand)]
enum TenantAction {
    Create {
        wechat_user_id: String,
        #[arg(long)]
        nickname: Option<String>,
    },
    List,
    Status {
        wechat_user_id: String,
        status: String,
    },
    Show {
        wechat_user_id: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let config = FitnessConfig::from_env()?;
    let db = Database::connect(&config.database.url).await?;
    let tenant_service = TenantService::new(db);

    let cli = Cli::parse();

    match cli.command {
        Commands::Tenant { action } => match action {
            TenantAction::Create {
                wechat_user_id,
                nickname,
            } => {
                let tenant = tenant_service
                    .create(&wechat_user_id, nickname.as_deref())
                    .await?;
                println!(
                    "Tenant created: id={}, wechat_user_id={}, status={}",
                    tenant.id, tenant.wechat_user_id, tenant.status
                );
            }
            TenantAction::List => {
                let tenants = tenant_service.list_all().await?;
                if tenants.is_empty() {
                    println!("No tenants found.");
                } else {
                    println!(
                        "{:<36} {:<32} {:<20} {:<10}",
                        "ID", "WeChat User ID", "Nickname", "Status"
                    );
                    println!("{}", "-".repeat(100));
                    for t in &tenants {
                        println!(
                            "{:<36} {:<32} {:<20} {:<10}",
                            t.id,
                            t.wechat_user_id,
                            t.nickname.as_deref().unwrap_or("-"),
                            t.status
                        );
                    }
                }
            }
            TenantAction::Status {
                wechat_user_id,
                status,
            } => {
                let tenant = tenant_service.set_status(&wechat_user_id, &status).await?;
                println!(
                    "Tenant status updated: wechat_user_id={}, status={}",
                    tenant.wechat_user_id, tenant.status
                );
            }
            TenantAction::Show { wechat_user_id } => {
                match tenant_service.find_by_wechat_id(&wechat_user_id).await? {
                    Some(t) => {
                        println!("ID:             {}", t.id);
                        println!("WeChat User ID: {}", t.wechat_user_id);
                        println!("Nickname:       {}", t.nickname.as_deref().unwrap_or("-"));
                        println!("Status:         {}", t.status);
                        println!(
                            "Daily Quota:    {}",
                            t.daily_quota
                                .map_or("unlimited".to_string(), |q| q.to_string())
                        );
                        println!("Created:        {}", t.created_at);
                        println!("Updated:        {}", t.updated_at);
                    }
                    None => println!("Tenant not found: {}", wechat_user_id),
                }
            }
        },
    }

    Ok(())
}
