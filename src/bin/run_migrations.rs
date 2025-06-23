use pantherswap_edge::database::Database;
use tracing::{info, error};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 Running database migrations");

    // Get database URL from environment or use default
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://tsdbadmin:sz2eu577bgqi5767@jqrbtbc5nw.w0mq2s13iy.tsdb.cloud.timescale.com:35762/tsdb?sslmode=require".to_string());

    info!("📊 Connecting to database: {}", database_url.split('@').last().unwrap_or("unknown"));

    // Create database connection
    let database = Database::new_production(&database_url).await?;
    info!("✅ Database connection established");

    // Run database migrations
    info!("🔄 Running database migrations...");
    if let Err(e) = database.run_migrations().await {
        error!("Failed to run migrations: {}", e);
        return Err(e.into());
    }
    info!("✅ Database migrations completed successfully");

    Ok(())
}
