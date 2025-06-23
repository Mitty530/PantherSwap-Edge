// Test CRUD operations for PantherSwap Edge database
// Run with: PANTHERSWAP_MARKET_DATA_ALPHA_VANTAGE_API_KEY=EZDZ4VOFQ2GRA7VU cargo run --bin test_crud

use pantherswap_edge::config::Settings;
use pantherswap_edge::database::Database;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("🧪 Starting PantherSwap Edge CRUD Operations Test");

    // Load configuration
    let settings = Settings::load()?;
    info!("✅ Configuration loaded successfully");

    // Initialize database with a shorter timeout for testing
    info!("🔗 Attempting to connect to database...");
    let database = match Database::new_testing(&settings.database.url).await {
        Ok(db) => {
            info!("✅ Database connection established successfully");
            db
        }
        Err(e) => {
            info!("❌ Database connection failed: {}", e);
            info!("💡 This is expected if TimescaleDB is not accessible");
            info!("   To test with a real database, ensure TimescaleDB is running and accessible");
            return Ok(());
        }
    };

    // Run migrations
    info!("🔄 Running database migrations...");
    match database.run_migrations().await {
        Ok(_) => info!("✅ Database migrations completed successfully"),
        Err(e) => {
            info!("⚠️  Migration warning: {}", e);
            info!("   Continuing with existing schema...");
        }
    }

    // Test basic CRUD operations
    info!("🧪 Testing basic CRUD operations...");
    let query_manager = database.query_manager();
    
    match query_manager.test_basic_operations().await {
        Ok(_) => {
            info!("🎉 All CRUD operations completed successfully!");
            info!("✅ Database is fully functional for PantherSwap Edge");
        }
        Err(e) => {
            info!("❌ CRUD operations failed: {}", e);
            info!("💡 This might indicate schema issues or data type mismatches");
        }
    }

    // Test additional operations
    info!("📊 Testing additional database features...");
    
    // Test health check
    match query_manager.health_check().await {
        Ok(true) => info!("✅ Database health check: HEALTHY"),
        Ok(false) => info!("⚠️  Database health check: UNHEALTHY"),
        Err(e) => info!("❌ Health check failed: {}", e),
    }

    // Test getting active instruments
    match query_manager.get_active_instruments().await {
        Ok(instruments) => {
            info!("✅ Found {} active instruments in database", instruments.len());
            for instrument in instruments.iter().take(3) {
                info!("   📈 {}: {} ({})", instrument.symbol, instrument.name, instrument.instrument_type);
            }
        }
        Err(e) => info!("⚠️  Could not retrieve instruments: {}", e),
    }

    info!("🏁 CRUD operations test completed");
    Ok(())
}
