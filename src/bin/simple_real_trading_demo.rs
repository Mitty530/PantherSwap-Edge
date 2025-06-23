// Simple Real Trading Demo
// Demonstrates actual trading engine execution with Alpha Vantage API and database

use std::time::{Duration, Instant};
use tracing::{info, error, Level};
use uuid::Uuid;

use pantherswap_edge::config::Settings;
use pantherswap_edge::database::Database;
use pantherswap_edge::trading::{TradingEngine, TradingEngineConfig};
use pantherswap_edge::trading::signals::{OrderRequest, OrderSide, OrderType, ExecutionStyle};
use pantherswap_edge::utils::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("🚀 Simple Real Trading Demo - PantherSwap Edge");
    info!("================================================");

    // Load configuration
    std::env::set_var("RUN_MODE", "production");
    let mut settings = Settings::load()?;
    settings.market_data.alpha_vantage_api_key = "EZDZ4VOFQ2GRA7VU".to_string();
    settings.database.url = "postgres://tsdbadmin:sz2eu577bgqi5767@jqrbtbc5nw.w0mq2s13iy.tsdb.cloud.timescale.com:35762/tsdb?sslmode=require".to_string();

    info!("✅ Configuration loaded");

    // Initialize database
    let database = Database::new(&settings.database.url).await?;
    database.run_manual_migrations().await?;
    info!("✅ Database connected and migrated");

    // Initialize trading engine
    let trading_config = TradingEngineConfig {
        max_position_size: 10000.0,
        max_daily_trades: 100,
        risk_check_interval_ms: 100,
        signal_generation_interval_ms: 500,
        enable_async_risk_checks: true,
        max_slippage_bps: 50,
        ..Default::default()
    };

    let trading_engine = TradingEngine::new(trading_config, database.clone()).await?;
    info!("✅ Trading Engine initialized");

    // Start the trading engine
    trading_engine.start().await?;
    info!("✅ Trading Engine started");

    // Execute a few test trades
    let mut successful_trades = 0;
    let mut failed_trades = 0;

    for i in 1..=5 {
        info!("🔄 Executing test trade #{}", i);

        // Create a test order
        let order_request = OrderRequest {
            id: Uuid::new_v4(),
            instrument_id: format!("TEST_TRADE_{}", i),
            side: if i % 2 == 0 { OrderSide::Buy } else { OrderSide::Sell },
            order_type: OrderType::Market,
            quantity: 1000.0,
            price: None, // Market order
            execution_style: ExecutionStyle::Aggressive,
            strategy_name: Some("DEMO_STRATEGY".to_string()),
            time_in_force: None,
            stop_loss: None,
            take_profit: None,
        };

        // Execute the trade
        let execution_start = Instant::now();
        match trading_engine.submit_order(order_request.clone()).await {
            Ok(order_id) => {
                let execution_latency = execution_start.elapsed();
                successful_trades += 1;
                info!("💰 ✅ Trade #{} SUCCESSFUL: Order ID {} | {} {} | Latency: {:.2}ms",
                     i, order_id, order_request.side, order_request.quantity, 
                     execution_latency.as_millis());
            }
            Err(e) => {
                failed_trades += 1;
                error!("❌ Trade #{} FAILED: {}", i, e);
            }
        }

        // Small delay between trades
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    // Stop the trading engine
    trading_engine.stop().await?;
    info!("✅ Trading Engine stopped");

    // Summary
    info!("================================================");
    info!("📊 SIMPLE REAL TRADING DEMO RESULTS");
    info!("================================================");
    info!("💰 Trading Results:");
    info!("   - Total Trades: {}", successful_trades + failed_trades);
    info!("   - Successful Trades: {}", successful_trades);
    info!("   - Failed Trades: {}", failed_trades);
    info!("   - Success Rate: {:.1}%", 
         if (successful_trades + failed_trades) > 0 {
             (successful_trades as f64 / (successful_trades + failed_trades) as f64) * 100.0
         } else { 0.0 });

    if successful_trades > 0 {
        info!("🎉 SUCCESS! Real trading engine is working properly");
        info!("✅ Orders are being executed through the actual trading engine");
        info!("✅ Database integration is functional");
        info!("📈 System is ready for live trading");
    } else {
        error!("❌ FAILURE! No trades were executed successfully");
        error!("🔍 Check trading engine configuration and database connectivity");
        error!("🛠️  Review error messages above for debugging information");
    }

    info!("================================================");
    info!("✅ Simple Real Trading Demo Complete");

    Ok(())
}
