// Trading Engine Initialization Diagnostic Tool
// Identifies specific initialization failures step by step

use std::time::Instant;
use tracing::{info, error, warn, Level};

use pantherswap_edge::config::Settings;
use pantherswap_edge::database::Database;
use pantherswap_edge::trading::{TradingEngine, TradingEngineConfig};
use pantherswap_edge::trading::execution::{ExecutionEngine, ExecutionConfig};
use pantherswap_edge::trading::risk::{RiskManager, RiskManagerConfig};
use pantherswap_edge::trading::portfolio::{PortfolioManager, PortfolioConfig};
use pantherswap_edge::trading::signals::SignalGenerator;
use pantherswap_edge::microstructure::MicrostructureEngine;
use pantherswap_edge::ai::AIEngine;
use pantherswap_edge::utils::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_target(false)
        .init();

    info!("🔍 Trading Engine Initialization Diagnostic");
    info!("============================================");

    // Step 1: Test Configuration Loading
    info!("📋 Step 1: Testing Configuration Loading...");
    let config_start = Instant::now();
    
    match test_configuration_loading().await {
        Ok(_) => {
            info!("✅ Configuration loading: SUCCESS ({:.2}ms)", config_start.elapsed().as_millis());
        }
        Err(e) => {
            error!("❌ Configuration loading: FAILED - {}", e);
            return Err(e);
        }
    }

    // Step 2: Test Database Connection
    info!("🗄️  Step 2: Testing Database Connection...");
    let db_start = Instant::now();
    
    let database = match test_database_connection().await {
        Ok(db) => {
            info!("✅ Database connection: SUCCESS ({:.2}ms)", db_start.elapsed().as_millis());
            db
        }
        Err(e) => {
            error!("❌ Database connection: FAILED - {}", e);
            return Err(e);
        }
    };

    // Step 3: Test Individual Component Initialization
    info!("🔧 Step 3: Testing Individual Component Initialization...");
    
    // Test ExecutionEngine
    info!("⚡ Testing ExecutionEngine initialization...");
    let exec_start = Instant::now();
    match test_execution_engine_init(database.clone()).await {
        Ok(_) => {
            info!("✅ ExecutionEngine: SUCCESS ({:.2}ms)", exec_start.elapsed().as_millis());
        }
        Err(e) => {
            error!("❌ ExecutionEngine: FAILED - {}", e);
            return Err(e);
        }
    }

    // Test RiskManager
    info!("🛡️  Testing RiskManager initialization...");
    let risk_start = Instant::now();
    match test_risk_manager_init() {
        Ok(_) => {
            info!("✅ RiskManager: SUCCESS ({:.2}ms)", risk_start.elapsed().as_millis());
        }
        Err(e) => {
            error!("❌ RiskManager: FAILED - {}", e);
            return Err(e);
        }
    }

    // Test PortfolioManager
    info!("💼 Testing PortfolioManager initialization...");
    let portfolio_start = Instant::now();
    match test_portfolio_manager_init(database.clone()).await {
        Ok(_) => {
            info!("✅ PortfolioManager: SUCCESS ({:.2}ms)", portfolio_start.elapsed().as_millis());
        }
        Err(e) => {
            error!("❌ PortfolioManager: FAILED - {}", e);
            return Err(e);
        }
    }

    // Test SignalGenerator
    info!("📡 Testing SignalGenerator initialization...");
    let signal_start = Instant::now();
    match test_signal_generator_init() {
        Ok(_) => {
            info!("✅ SignalGenerator: SUCCESS ({:.2}ms)", signal_start.elapsed().as_millis());
        }
        Err(e) => {
            error!("❌ SignalGenerator: FAILED - {}", e);
            return Err(e);
        }
    }

    // Test MicrostructureEngine
    info!("🧬 Testing MicrostructureEngine initialization...");
    let micro_start = Instant::now();
    match test_microstructure_engine_init().await {
        Ok(_) => {
            info!("✅ MicrostructureEngine: SUCCESS ({:.2}ms)", micro_start.elapsed().as_millis());
        }
        Err(e) => {
            error!("❌ MicrostructureEngine: FAILED - {}", e);
            return Err(e);
        }
    }

    // Test AIEngine
    info!("🤖 Testing AIEngine initialization...");
    let ai_start = Instant::now();
    match test_ai_engine_init(database.clone()).await {
        Ok(_) => {
            info!("✅ AIEngine: SUCCESS ({:.2}ms)", ai_start.elapsed().as_millis());
        }
        Err(e) => {
            error!("❌ AIEngine: FAILED - {}", e);
            return Err(e);
        }
    }

    // Step 4: Test Full TradingEngine Initialization
    info!("🚀 Step 4: Testing Full TradingEngine Initialization...");
    let engine_start = Instant::now();
    
    match test_full_trading_engine_init(database.clone()).await {
        Ok(_) => {
            info!("✅ Full TradingEngine: SUCCESS ({:.2}ms)", engine_start.elapsed().as_millis());
        }
        Err(e) => {
            error!("❌ Full TradingEngine: FAILED - {}", e);
            return Err(e);
        }
    }

    // Step 5: Test TradingEngine Start/Stop
    info!("▶️  Step 5: Testing TradingEngine Start/Stop...");
    let start_stop_start = Instant::now();
    
    match test_trading_engine_start_stop(database.clone()).await {
        Ok(_) => {
            info!("✅ TradingEngine Start/Stop: SUCCESS ({:.2}ms)", start_stop_start.elapsed().as_millis());
        }
        Err(e) => {
            error!("❌ TradingEngine Start/Stop: FAILED - {}", e);
            return Err(e);
        }
    }

    info!("============================================");
    info!("🎉 ALL TRADING ENGINE DIAGNOSTICS PASSED!");
    info!("✅ Trading Engine initialization is working correctly");
    info!("🚀 Ready for live trading implementation");

    Ok(())
}

async fn test_configuration_loading() -> Result<Settings> {
    std::env::set_var("RUN_MODE", "production");
    let mut settings = Settings::load()?;
    settings.market_data.alpha_vantage_api_key = "EZDZ4VOFQ2GRA7VU".to_string();
    settings.database.url = "postgres://tsdbadmin:sz2eu577bgqi5767@jqrbtbc5nw.w0mq2s13iy.tsdb.cloud.timescale.com:35762/tsdb?sslmode=require".to_string();
    Ok(settings)
}

async fn test_database_connection() -> Result<Database> {
    let settings = test_configuration_loading().await?;
    let database = Database::new(&settings.database.url).await?;
    
    // Test basic database operations
    let health = database.health_check().await?;
    if !health {
        return Err(pantherswap_edge::utils::PantherSwapError::DatabaseError("Health check failed".to_string()));
    }
    
    // Run migrations
    database.run_manual_migrations().await?;
    
    Ok(database)
}

async fn test_execution_engine_init(database: Database) -> Result<()> {
    let config = ExecutionConfig::default();
    let _execution_engine = ExecutionEngine::new(config, database).await?;
    Ok(())
}

fn test_risk_manager_init() -> Result<()> {
    let config = RiskManagerConfig::default();
    let _risk_manager = RiskManager::with_config(config);
    Ok(())
}

async fn test_portfolio_manager_init(database: Database) -> Result<()> {
    let config = PortfolioConfig::default();
    let _portfolio_manager = PortfolioManager::new(config, database).await?;
    Ok(())
}

fn test_signal_generator_init() -> Result<()> {
    let _signal_generator = SignalGenerator::new(0.7);
    Ok(())
}

async fn test_microstructure_engine_init() -> Result<()> {
    let _microstructure_engine = MicrostructureEngine::new().await?;
    Ok(())
}

async fn test_ai_engine_init(database: Database) -> Result<()> {
    let _ai_engine = AIEngine::new(database).await?;
    Ok(())
}

async fn test_full_trading_engine_init(database: Database) -> Result<TradingEngine> {
    let config = TradingEngineConfig::default();
    let trading_engine = TradingEngine::new(config, database).await?;
    Ok(trading_engine)
}

async fn test_trading_engine_start_stop(database: Database) -> Result<()> {
    let trading_engine = test_full_trading_engine_init(database).await?;
    
    // Test start
    trading_engine.start().await?;
    info!("🟢 TradingEngine started successfully");
    
    // Small delay to let it run
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    
    // Test stop
    trading_engine.stop().await?;
    info!("🔴 TradingEngine stopped successfully");
    
    Ok(())
}
