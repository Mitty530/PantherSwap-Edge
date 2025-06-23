// Simple Trading Engine Test
// Basic test to identify initialization issues

use tracing::{info, error, Level};
use pantherswap_edge::config::Settings;
use pantherswap_edge::database::Database;
use pantherswap_edge::trading::{TradingEngine, TradingEngineConfig};
use pantherswap_edge::utils::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("🔍 Simple Trading Engine Test");
    info!("==============================");

    // Test 1: Configuration
    info!("📋 Testing configuration...");
    std::env::set_var("RUN_MODE", "production");
    let mut settings = match Settings::load() {
        Ok(s) => {
            info!("✅ Configuration loaded successfully");
            s
        }
        Err(e) => {
            error!("❌ Configuration failed: {}", e);
            return Err(e);
        }
    };

    settings.market_data.alpha_vantage_api_key = "EZDZ4VOFQ2GRA7VU".to_string();
    settings.database.url = "postgres://tsdbadmin:sz2eu577bgqi5767@jqrbtbc5nw.w0mq2s13iy.tsdb.cloud.timescale.com:35762/tsdb?sslmode=require".to_string();

    // Test 2: Database
    info!("🗄️  Testing database connection...");
    let database = match Database::new(&settings.database.url).await {
        Ok(db) => {
            info!("✅ Database connected successfully");
            db
        }
        Err(e) => {
            error!("❌ Database connection failed: {}", e);
            return Err(e);
        }
    };

    // Test 3: Database health
    info!("🏥 Testing database health...");
    match database.health_check().await {
        Ok(true) => {
            info!("✅ Database health check passed");
        }
        Ok(false) => {
            error!("❌ Database health check failed");
            return Err(pantherswap_edge::utils::PantherSwapError::DatabaseError("Health check failed".to_string()));
        }
        Err(e) => {
            error!("❌ Database health check error: {}", e);
            return Err(e);
        }
    }

    // Test 4: Database migrations
    info!("🔄 Running database migrations...");
    match database.run_manual_migrations().await {
        Ok(_) => {
            info!("✅ Database migrations completed");
        }
        Err(e) => {
            error!("❌ Database migrations failed: {}", e);
            return Err(e);
        }
    }

    // Test 5: Trading Engine Configuration
    info!("⚙️  Creating trading engine configuration...");
    let trading_config = TradingEngineConfig::default();
    info!("✅ Trading engine configuration created");

    // Test 6: Trading Engine Initialization
    info!("🚀 Initializing trading engine...");
    let trading_engine = match TradingEngine::new(trading_config, database.clone()).await {
        Ok(engine) => {
            info!("✅ Trading engine initialized successfully!");
            engine
        }
        Err(e) => {
            error!("❌ Trading engine initialization failed: {}", e);
            error!("   Error details: {:?}", e);
            return Err(e);
        }
    };

    // Test 7: Trading Engine Start
    info!("▶️  Starting trading engine...");
    match trading_engine.start().await {
        Ok(_) => {
            info!("✅ Trading engine started successfully!");
        }
        Err(e) => {
            error!("❌ Trading engine start failed: {}", e);
            return Err(e);
        }
    }

    // Test 8: Trading Engine Stop
    info!("⏹️  Stopping trading engine...");
    match trading_engine.stop().await {
        Ok(_) => {
            info!("✅ Trading engine stopped successfully!");
        }
        Err(e) => {
            error!("❌ Trading engine stop failed: {}", e);
            return Err(e);
        }
    }

    info!("==============================");
    info!("🎉 ALL TESTS PASSED!");
    info!("✅ Trading Engine is working correctly");
    info!("🚀 Ready for live trading implementation");

    Ok(())
}
