// Demo of database query functions (using simple query functions)
// Run with: DATABASE_URL="..." cargo run --example query_demo

use pantherswap_edge::database::{Database, types::*};
use pantherswap_edge::config::Settings;
use chrono::Utc;
use uuid::Uuid;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 PantherSwap Edge Database Query Demo");
    
    // Load configuration
    let settings = Settings::load()?;
    
    // Connect to database
    let database = Database::new(&settings.database.url).await?;
    println!("✅ Connected to database");
    
    // Get simple query manager
    let query_manager = database.simple_query_manager();
    
    // Test health check
    let health = query_manager.health_check().await?;
    println!("📊 Database health: {}", if health { "✅ Healthy" } else { "❌ Unhealthy" });
    
    // Create a test instrument
    let instrument = Instrument {
        id: Uuid::new_v4(), // This will be overwritten by the database
        symbol: "BTC-USD".to_string(),
        name: "Bitcoin USD".to_string(),
        instrument_type: "crypto".to_string(),
        base_currency: "BTC".to_string(),
        quote_currency: "USD".to_string(),
        tick_size: 0.01,
        lot_size: 0.001,
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    // Test instrument operations
    println!("\n📈 Testing Instrument Operations:");
    
    // Check if instrument already exists
    match query_manager.get_instrument_by_symbol("BTC-USD").await? {
        Some(existing) => {
            println!("✅ Found existing instrument: {} ({})", existing.name, existing.id);
        }
        None => {
            // Insert new instrument
            let instrument_id = query_manager.insert_instrument(&instrument).await?;
            println!("✅ Inserted new instrument: {} ({})", instrument.name, instrument_id);
        }
    }
    
    // Get the instrument to use for further tests
    let btc_instrument = query_manager.get_instrument_by_symbol("BTC-USD").await?
        .expect("BTC-USD instrument should exist");
    
    // Test market data operations
    println!("\n📊 Testing Market Data Operations:");
    
    let market_tick = MarketTick {
        timestamp: Utc::now(),
        instrument_id: btc_instrument.id,
        provider: "demo_provider".to_string(),
        bid_price: 45000.50,
        ask_price: 45001.50,
        bid_size: 1.5,
        ask_size: 2.0,
        last_price: Some(45001.00),
        volume: Some(100.0),
        spread: 1.00,
        data_quality_score: 0.95,
        raw_data: json!({"source": "demo", "quality": "high"}),
    };
    
    // Insert market tick
    query_manager.insert_market_tick(&market_tick).await?;
    println!("✅ Inserted market tick for {}", btc_instrument.symbol);
    
    // Get latest market tick
    if let Some(latest_tick) = query_manager.get_latest_market_tick(btc_instrument.id).await? {
        println!("✅ Retrieved latest tick: ${:.2} @ {}", 
                 latest_tick.last_price.unwrap_or(0.0), 
                 latest_tick.timestamp.format("%H:%M:%S"));
    }
    
    // Test AI prediction operations
    println!("\n🤖 Testing AI Prediction Operations:");
    
    let ai_prediction = AIPrediction {
        timestamp: Utc::now(),
        instrument_id: btc_instrument.id,
        model_type: "lstm".to_string(),
        model_version: "v1.0".to_string(),
        prediction_horizon_minutes: 60,
        predicted_price: 45500.00,
        predicted_volatility: Some(0.02),
        confidence_score: 0.85,
        prediction_intervals: Some(json!({"lower": 44500, "upper": 46500})),
        feature_importance: Some(json!({"volume": 0.3, "price": 0.7})),
        created_at: Utc::now(),
    };
    
    query_manager.insert_ai_prediction(&ai_prediction).await?;
    println!("✅ Inserted AI prediction: ${:.2} (confidence: {:.1}%)", 
             ai_prediction.predicted_price, 
             ai_prediction.confidence_score * 100.0);
    
    // Test trading signal operations
    println!("\n⚡ Testing Trading Signal Operations:");
    
    let trading_signal = TradingSignal {
        timestamp: Utc::now(),
        instrument_id: btc_instrument.id,
        strategy_type: "momentum".to_string(),
        signal_type: "BUY".to_string(),
        confidence_score: 0.78,
        target_price: Some(46000.00),
        stop_loss: Some(44000.00),
        take_profit: Some(47000.00),
        position_size: 0.1,
        risk_score: 0.25,
        time_horizon: Some(chrono::Duration::hours(4)),
        metadata: json!({"strategy": "momentum", "timeframe": "4h"}),
        created_at: Utc::now(),
    };
    
    query_manager.insert_trading_signal(&trading_signal).await?;
    println!("✅ Inserted trading signal: {} {} (confidence: {:.1}%)", 
             trading_signal.signal_type,
             btc_instrument.symbol,
             trading_signal.confidence_score * 100.0);
    
    // Test analytical queries
    println!("\n📈 Testing Analytical Queries:");
    
    let end_time = Utc::now();
    let start_time = end_time - chrono::Duration::hours(24);
    
    let ohlcv_data = query_manager.get_ohlcv_data(
        btc_instrument.id,
        "1 hour",
        start_time,
        end_time
    ).await?;
    
    println!("✅ Retrieved {} OHLCV data points for the last 24 hours", ohlcv_data.len());
    
    if let Some((time, open, high, low, close, volume)) = ohlcv_data.first() {
        println!("   Latest: O:{:.2} H:{:.2} L:{:.2} C:{:.2} V:{:.2} @ {}", 
                 open, high, low, close, volume, time.format("%H:%M"));
    }
    
    println!("\n🎉 Database query demo completed successfully!");
    println!("✅ All query functions are working correctly");
    
    Ok(())
}
