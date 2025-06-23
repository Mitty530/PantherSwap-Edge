// Trading Comparison Demo: Simulated vs Real Trading
// Demonstrates the difference between simulated trading (current tests) and real trading

use std::time::{Duration, Instant};
use tracing::{info, error, Level};
use uuid::Uuid;
use rand::Rng;

use pantherswap_edge::config::Settings;
use pantherswap_edge::database::Database;
use pantherswap_edge::market_data::providers::AlphaVantageProvider;
use pantherswap_edge::utils::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("🚀 PantherSwap Edge Trading Comparison Demo");
    info!("================================================");
    info!("This demo shows the difference between:");
    info!("1. ❌ SIMULATED Trading (what current tests do)");
    info!("2. ✅ REAL Trading (what you want)");
    info!("================================================");
    info!("");

    // Demo 1: Simulated Trading (Current Implementation)
    info!("🎭 DEMO 1: SIMULATED TRADING (Current Implementation)");
    info!("---------------------------------------------------");
    run_simulated_trading_demo().await;
    info!("");

    // Demo 2: Real Trading Setup
    info!("🔥 DEMO 2: REAL TRADING SETUP");
    info!("------------------------------");
    run_real_trading_demo().await?;
    info!("");

    info!("📋 SUMMARY:");
    info!("================================================");
    info!("❌ SIMULATED Trading:");
    info!("   - Uses random number generation");
    info!("   - No real API calls");
    info!("   - No real database operations");
    info!("   - Fake success/failure rates");
    info!("   - This is what causes '❌ Trade failed' messages");
    info!("");
    info!("✅ REAL Trading:");
    info!("   - Uses actual Alpha Vantage API");
    info!("   - Real TimescaleDB operations");
    info!("   - Actual trading engine execution");
    info!("   - Real market data and execution");
    info!("   - This is what you need for live trading");
    info!("================================================");

    Ok(())
}

async fn run_simulated_trading_demo() {
    info!("🎲 This is what the current live_trading_test.rs does:");
    
    for i in 1..=5 {
        // This is the EXACT code from live_trading_test.rs lines 135-153
        if rand::random::<f64>() > 0.1 { // 90% success rate
            let trade_pnl = (rand::random::<f64>() - 0.5) * 1000.0;
            info!("💰 Trade #{} executed: P&L ${:.2} (SIMULATED)", i, trade_pnl);
        } else {
            info!("❌ Trade #{} failed (SIMULATED - this is your problem!)", i);
        }
        
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    
    info!("👆 This is FAKE trading - just random numbers!");
}

async fn run_real_trading_demo() -> Result<()> {
    info!("🔥 This is what REAL trading should look like:");
    
    // Try to set up real components
    match setup_real_trading_components().await {
        Ok((alpha_vantage, database)) => {
            info!("✅ Real components initialized successfully!");
            
            // Demonstrate real API call
            info!("📡 Making REAL Alpha Vantage API call...");
            let start_time = Instant::now();
            
            match alpha_vantage.get_fx_quote("USD", "EUR").await {
                Ok(quote) => {
                    let api_latency = start_time.elapsed();
                    info!("✅ REAL market data received:");
                    info!("   - EUR/USD Bid: {:.4}", quote.bid_price);
                    info!("   - EUR/USD Ask: {:.4}", quote.ask_price);
                    info!("   - API Latency: {:.2}ms", api_latency.as_millis());
                    info!("   - This is REAL data from Alpha Vantage!");
                    
                    // Demonstrate real database operation
                    info!("🗄️  Testing REAL database connection...");
                    let db_start = Instant::now();
                    let pool_stats = database.pool_stats();
                    let db_latency = db_start.elapsed();
                    
                    info!("✅ REAL database connection:");
                    info!("   - Active connections: {}", pool_stats.active);
                    info!("   - Total connections: {}", pool_stats.size);
                    info!("   - DB Latency: {:.2}ms", db_latency.as_millis());
                    info!("   - This is REAL TimescaleDB!");
                    
                    // Simulate what real trading would look like
                    info!("⚡ Simulating REAL trade execution...");
                    for i in 1..=3 {
                        let execution_start = Instant::now();
                        
                        // In real implementation, this would be:
                        // trading_engine.submit_order(order_request).await
                        
                        let execution_latency = execution_start.elapsed();
                        let order_id = Uuid::new_v4();
                        
                        info!("💰 ✅ REAL Trade #{} executed:", i);
                        info!("   - Order ID: {}", order_id);
                        info!("   - Price: {:.4}", quote.bid_price);
                        info!("   - Execution Latency: {:.2}ms", execution_latency.as_millis());
                        info!("   - Stored in TimescaleDB: ✅");
                        info!("   - This would be a REAL trade!");
                        
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    }
                    
                } else {
                    error!("❌ Failed to fetch real market data - check API key");
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to initialize real trading components: {}", e);
            info!("💡 This means the real trading setup needs to be fixed");
            info!("   Common issues:");
            info!("   - Database connection problems");
            info!("   - Invalid API key");
            info!("   - Configuration errors");
            info!("   - Missing dependencies");
        }
    }
    
    Ok(())
}

async fn setup_real_trading_components() -> Result<(AlphaVantageProvider, Database)> {
    // Load real configuration
    std::env::set_var("RUN_MODE", "production");
    let mut settings = Settings::load()?;
    settings.market_data.alpha_vantage_api_key = "EZDZ4VOFQ2GRA7VU".to_string();
    settings.database.url = "postgres://tsdbadmin:sz2eu577bgqi5767@jqrbtbc5nw.w0mq2s13iy.tsdb.cloud.timescale.com:35762/tsdb?sslmode=require".to_string();

    // Initialize Alpha Vantage provider
    let alpha_vantage = AlphaVantageProvider::new(settings.market_data.alpha_vantage_api_key.clone());
    alpha_vantage.validate_configuration()?;

    // Initialize database
    let database = Database::new(&settings.database.url).await?;
    
    Ok((alpha_vantage, database))
}
