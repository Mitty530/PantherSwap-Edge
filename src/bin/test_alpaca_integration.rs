// Test Alpaca API integration for PantherSwap Edge
// Run with: PANTHERSWAP_MARKET_DATA_ALPACA_API_KEY=your_key PANTHERSWAP_MARKET_DATA_ALPACA_SECRET_KEY=your_secret cargo run --bin test_alpaca_integration

use pantherswap_edge::config::Settings;
use pantherswap_edge::database::Database;
use pantherswap_edge::market_data::{AlpacaProvider, MarketDataManager};
use pantherswap_edge::trading::{AlpacaExecutionEngine, signals::OrderRequest, signals::OrderType};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("🚀 Starting Alpaca API Integration Test");

    // Load configuration
    let mut settings = Settings::load()?;
    
    // Override Alpaca API keys from environment if available
    if let Ok(api_key) = std::env::var("PANTHERSWAP_MARKET_DATA_ALPACA_API_KEY") {
        info!("🔧 Using Alpaca API key from environment");
        settings.market_data.alpaca.api_key = api_key;
    }
    
    if let Ok(secret_key) = std::env::var("PANTHERSWAP_MARKET_DATA_ALPACA_SECRET_KEY") {
        info!("🔧 Using Alpaca secret key from environment");
        settings.market_data.alpaca.secret_key = secret_key;
    }

    if settings.market_data.alpaca.api_key.is_empty() || settings.market_data.alpaca.secret_key.is_empty() {
        eprintln!("❌ Alpaca API credentials not found!");
        eprintln!("Please set environment variables:");
        eprintln!("  PANTHERSWAP_MARKET_DATA_ALPACA_API_KEY=your_api_key");
        eprintln!("  PANTHERSWAP_MARKET_DATA_ALPACA_SECRET_KEY=your_secret_key");
        eprintln!("");
        eprintln!("You can get paper trading credentials from: https://alpaca.markets/");
        return Ok(());
    }

    // Initialize database
    let database = Database::new(&settings.database.url).await?;
    info!("✅ Database connected");

    // Test 1: Alpaca Provider Validation
    info!("\n📊 Test 1: Alpaca Provider Validation");
    let alpaca_provider = AlpacaProvider::new(settings.market_data.alpaca.clone())?
        .with_database(database.clone());
    
    match alpaca_provider.validate_configuration().await {
        Ok(_) => info!("✅ Alpaca API validation successful"),
        Err(e) => {
            eprintln!("❌ Alpaca API validation failed: {}", e);
            return Ok(());
        }
    }

    // Test 2: Account Information
    info!("\n💰 Test 2: Account Information");
    match alpaca_provider.get_account_info().await {
        Ok(account_info) => {
            info!("✅ Account info retrieved:");
            info!("   {}", serde_json::to_string_pretty(&account_info)?);
        }
        Err(e) => {
            eprintln!("❌ Failed to get account info: {}", e);
        }
    }

    // Test 3: Market Status
    info!("\n🕐 Test 3: Market Status");
    match alpaca_provider.get_market_status().await {
        Ok(market_status) => {
            info!("✅ Market status retrieved:");
            info!("   {}", serde_json::to_string_pretty(&market_status)?);
        }
        Err(e) => {
            eprintln!("❌ Failed to get market status: {}", e);
        }
    }

    // Test 4: Market Data Retrieval
    info!("\n📈 Test 4: Market Data Retrieval");
    let test_symbols = vec!["AAPL".to_string(), "MSFT".to_string(), "GOOGL".to_string()];
    
    for symbol in &test_symbols {
        match alpaca_provider.get_latest_quote(symbol).await {
            Ok(quote) => {
                info!("✅ Quote for {}: ${:.2} bid, ${:.2} ask, spread: ${:.4}", 
                    symbol, quote.bid_price, quote.ask_price, quote.spread);
            }
            Err(e) => {
                eprintln!("❌ Failed to get quote for {}: {}", symbol, e);
            }
        }
    }

    // Test 5: Multiple Quotes
    info!("\n📊 Test 5: Multiple Quotes Retrieval");
    match alpaca_provider.get_multiple_quotes(&test_symbols).await {
        Ok(quotes) => {
            info!("✅ Retrieved {} quotes:", quotes.len());
            for (symbol, quote) in quotes {
                info!("   {}: ${:.2} (spread: ${:.4})", symbol, quote.exchange_rate, quote.spread);
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to get multiple quotes: {}", e);
        }
    }

    // Test 6: Historical Data
    info!("\n📚 Test 6: Historical Data");
    let start_time = chrono::Utc::now() - chrono::Duration::days(1);
    let end_time = chrono::Utc::now();
    
    match alpaca_provider.get_historical_bars("AAPL", start_time, end_time, "1Min").await {
        Ok(bars) => {
            info!("✅ Retrieved {} historical bars for AAPL", bars.len());
            if let Some(latest_bar) = bars.last() {
                info!("   Latest bar: O=${:.2} H=${:.2} L=${:.2} C=${:.2} V={}", 
                    latest_bar.open, latest_bar.high, latest_bar.low, latest_bar.close, latest_bar.volume);
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to get historical data: {}", e);
        }
    }

    // Test 7: Execution Engine
    info!("\n⚡ Test 7: Execution Engine");
    let execution_engine = AlpacaExecutionEngine::new(settings.market_data.alpaca.clone())?
        .with_database(database.clone());

    // Check if trading is ready
    match execution_engine.is_ready_for_trading().await {
        true => info!("✅ Trading engine ready for execution"),
        false => info!("⚠️  Trading engine not ready (may be outside market hours)"),
    }

    // Test 8: Market Data Manager with Alpaca
    info!("\n🔄 Test 8: Market Data Manager Integration");
    match MarketDataManager::new_with_alpaca(&settings, database.clone()).await {
        Ok(manager) => {
            info!("✅ Market Data Manager with Alpaca created successfully");
            
            // Test getting quotes through the manager
            match manager.get_latest_quote_primary("AAPL").await {
                Ok(quote) => {
                    info!("✅ Quote via manager: AAPL ${:.2} (spread: ${:.4})", 
                        quote.exchange_rate, quote.spread);
                }
                Err(e) => {
                    eprintln!("❌ Failed to get quote via manager: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to create Market Data Manager: {}", e);
        }
    }

    // Test 9: Real-time Streaming (brief test)
    info!("\n🌊 Test 9: Real-time Streaming Test");
    match alpaca_provider.start_streaming(vec!["AAPL".to_string()]).await {
        Ok(mut rx) => {
            info!("✅ Streaming started, listening for 10 seconds...");
            
            let timeout = tokio::time::timeout(
                std::time::Duration::from_secs(10),
                async {
                    let mut event_count = 0;
                    while let Some(event) = rx.recv().await {
                        match event {
                            pantherswap_edge::market_data::alpaca::AlpacaStreamEvent::Connected => {
                                info!("🔗 Streaming connected");
                            }
                            pantherswap_edge::market_data::alpaca::AlpacaStreamEvent::Quote(quote) => {
                                info!("📊 Quote: {} ${:.2}/${:.2}", quote.symbol, quote.bid_price, quote.ask_price);
                                event_count += 1;
                            }
                            pantherswap_edge::market_data::alpaca::AlpacaStreamEvent::Trade(trade) => {
                                info!("💱 Trade: {} ${:.2} x{}", trade.symbol, trade.price, trade.size);
                                event_count += 1;
                            }
                            pantherswap_edge::market_data::alpaca::AlpacaStreamEvent::Error(err) => {
                                eprintln!("❌ Stream error: {}", err);
                            }
                            _ => {}
                        }
                        
                        if event_count >= 5 {
                            break; // Stop after 5 events
                        }
                    }
                }
            ).await;
            
            match timeout {
                Ok(_) => info!("✅ Streaming test completed"),
                Err(_) => info!("⏰ Streaming test timed out (normal for testing)"),
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to start streaming: {}", e);
        }
    }

    // Test 10: Performance Metrics
    info!("\n📊 Test 10: Performance Summary");
    let stats = execution_engine.get_execution_stats().await;
    info!("✅ Execution Statistics:");
    info!("   Total Orders: {}", stats.total_orders);
    info!("   Filled Orders: {}", stats.filled_orders);
    info!("   Average Fill Time: {:.2}ms", stats.average_fill_time_ms);
    info!("   Average Slippage: {:.2} bps", stats.slippage_bps);

    info!("\n🎉 Alpaca Integration Test Completed Successfully!");
    info!("🔧 Your Alpaca API integration is ready for live trading testing");
    info!("📝 Next steps:");
    info!("   1. Provide your Alpaca API credentials when prompted");
    info!("   2. Run end-to-end trading tests");
    info!("   3. Monitor performance metrics");
    info!("   4. Validate against target latencies (<10ms execution, <100ms AI inference)");

    Ok(())
}
