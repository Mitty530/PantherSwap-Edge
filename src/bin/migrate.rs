// Migration management CLI tool
// Provides command-line interface for database migration operations

use anyhow::Result;
use clap::{Parser, Subcommand};
use pantherswap_edge::config::Settings;
use pantherswap_edge::database::Database;
use tracing::{info, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "migrate")]
#[command(about = "PantherSwap Edge Database Migration Tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run pending migrations
    Up,
    /// Show migration status
    Status,
    /// Validate database schema
    Validate,
    /// Reset database (WARNING: destroys all data)
    Reset {
        #[arg(long, help = "Confirm reset operation")]
        confirm: bool,
    },
    /// Show database information
    Info,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    setup_logging()?;

    let cli = Cli::parse();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Load configuration
    let settings = Settings::load()?;
    info!("Configuration loaded successfully");

    // Connect to database
    let database = Database::new(&settings.database.url).await?;
    info!("Connected to database");

    match cli.command {
        Commands::Up => {
            info!("Running database migrations...");
            database.run_migrations().await?;
            info!("✅ Migrations completed successfully");
        }
        Commands::Status => {
            let status = database.migration_status().await?;
            println!("{}", status);
        }
        Commands::Validate => {
            let validation = database.validate_schema().await?;
            println!("{}", validation);
            if !validation.valid {
                std::process::exit(1);
            }
        }
        Commands::Reset { confirm } => {
            if !confirm {
                eprintln!("❌ Reset operation requires --confirm flag");
                eprintln!("WARNING: This will destroy ALL data in the database!");
                std::process::exit(1);
            }

            println!("🚨 DANGER: Resetting database...");
            let migration_manager = database.migration_manager();
            migration_manager.reset().await?;
            println!("✅ Database reset completed");
        }
        Commands::Info => {
            show_database_info(&database).await?;
        }
    }

    Ok(())
}

async fn show_database_info(database: &Database) -> Result<()> {
    println!("🗄️  PantherSwap Edge Database Information");
    println!("==========================================");

    // Health check
    let healthy = database.health_check().await?;
    println!("Health: {}", if healthy { "✅ Healthy" } else { "❌ Unhealthy" });

    // Migration status
    let status = database.migration_status().await?;
    println!("\n📊 Migration Status:");
    println!("  Applied migrations: {}", status.applied_migrations);
    println!("  Pending migrations: {}", status.pending_migrations);
    println!("  Last migration: {}", status.last_migration.as_deref().unwrap_or("None"));
    println!("  Database ready: {}", if status.database_ready { "✅ Yes" } else { "❌ No" });

    // Schema validation
    let validation = database.validate_schema().await?;
    println!("\n🔍 Schema Validation:");
    if validation.valid {
        println!("  Status: ✅ Valid");
    } else {
        println!("  Status: ❌ Invalid");
        for issue in &validation.issues {
            println!("  Issue: {}", issue);
        }
    }

    // TimescaleDB information
    if let Ok(hypertables) = get_hypertables_info(database).await {
        println!("\n⏰ TimescaleDB Hypertables:");
        for table in hypertables {
            println!("  📈 {}", table);
        }
    }

    Ok(())
}

async fn get_hypertables_info(database: &Database) -> Result<Vec<String>> {
    let hypertables = sqlx::query_scalar::<_, String>(
        "SELECT table_name FROM timescaledb_information.hypertables
         WHERE schema_name = 'public'
         ORDER BY table_name"
    )
    .fetch_all(&database.pool)
    .await?;

    // Also check if TimescaleDB extension is available
    let timescale_available = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM pg_extension WHERE extname = 'timescaledb')"
    )
    .fetch_one(&database.pool)
    .await
    .unwrap_or(false);

    if !timescale_available {
        println!("  ⚠️  TimescaleDB extension not found!");
    }

    Ok(hypertables)
}

fn setup_logging() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "migrate=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(())
}
