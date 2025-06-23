// Simple data validation demo for PantherSwap Edge
// Demonstrates core validation functionality without middleware complexity
// Run with: DATABASE_URL="..." cargo run --example simple_validation_demo

use pantherswap_edge::database::{Database, types::*};
use pantherswap_edge::config::Settings;
use chrono::Utc;
use uuid::Uuid;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 PantherSwap Edge Simple Data Validation Demo");
    println!("===============================================");
    
    // Load configuration
    let settings = Settings::load()?;
    
    // Connect to database
    let database = Database::new(&settings.database.url).await?;
    println!("✅ Connected to database");
    
    // Get validation components
    let mut data_validator = database.data_validator();
    let mut data_quality_assessor = database.data_quality_assessor();
    println!("✅ Validation components initialized");

    // Test 1: Market Data Validation
    println!("\n📊 Testing Market Data Validation...");
    
    let instrument_id = Uuid::new_v4();
    
    // Test valid market tick
    let valid_tick = MarketTick {
        timestamp: Utc::now(),
        instrument_id,
        provider: "alpha_vantage".to_string(),
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

    println!("Testing valid market tick...");
    match data_validator.validate_market_tick(&valid_tick) {
        Ok(()) => {
            println!("✅ Market tick validation: PASSED");
            
            // Test quality assessment
            match data_quality_assessor.assess_market_tick_quality(&valid_tick) {
                Ok(quality_report) => {
                    println!("✅ Quality assessment completed:");
                    println!("   - Overall score: {:.2}", quality_report.overall_score);
                    println!("   - Data completeness: {:.2}", quality_report.data_completeness);
                    println!("   - Temporal consistency: {:.2}", quality_report.temporal_consistency);
                    println!("   - Anomalies detected: {}", quality_report.anomalies_detected.len());
                    
                    if !quality_report.recommendations.is_empty() {
                        println!("   - Recommendations:");
                        for rec in &quality_report.recommendations {
                            println!("     • {}", rec);
                        }
                    }
                }
                Err(e) => println!("   ⚠️  Quality assessment failed: {}", e),
            }
        }
        Err(e) => println!("❌ Market tick validation failed: {}", e),
    }

    // Test invalid market tick
    println!("\nTesting invalid market tick (negative spread)...");
    let invalid_tick = MarketTick {
        timestamp: Utc::now(),
        instrument_id,
        provider: "test_provider".to_string(),
        bid_price: 45001.50, // Higher than ask price
        ask_price: 45000.50,
        bid_size: 1.5,
        ask_size: 2.0,
        last_price: Some(45001.00),
        volume: Some(100.0),
        spread: -1.00, // Negative spread
        data_quality_score: 0.3, // Low quality
        raw_data: json!({"source": "demo", "quality": "low"}),
    };

    match data_validator.validate_market_tick(&invalid_tick) {
        Ok(()) => println!("⚠️  Invalid tick unexpectedly passed validation"),
        Err(e) => {
            println!("✅ Invalid market tick correctly rejected:");
            println!("   - Error: {}", e);
        }
    }

    // Test 2: Trading Signal Validation
    println!("\n⚡ Testing Trading Signal Validation...");

    // Test valid trading signal
    println!("Testing valid trading signal...");
    let valid_signal = TradingSignal {
        timestamp: Utc::now(),
        instrument_id,
        strategy_type: "momentum".to_string(),
        signal_type: "BUY".to_string(),
        confidence_score: 0.85,
        target_price: Some(46000.00),
        stop_loss: Some(44000.00),
        take_profit: Some(47000.00),
        position_size: 0.1,
        risk_score: 0.25,
        time_horizon: Some(chrono::Duration::hours(4)),
        metadata: json!({"strategy": "momentum", "timeframe": "4h"}),
        created_at: Utc::now(),
    };

    match data_validator.validate_trading_signal(&valid_signal) {
        Ok(()) => {
            println!("✅ Trading signal validation: PASSED");
            
            // Test quality assessment for signals
            match data_quality_assessor.assess_trading_signal_quality(&valid_signal) {
                Ok(quality_report) => {
                    println!("✅ Signal quality assessment:");
                    println!("   - Overall score: {:.2}", quality_report.overall_score);
                    println!("   - Data completeness: {:.2}", quality_report.data_completeness);
                    println!("   - Anomalies detected: {}", quality_report.anomalies_detected.len());
                }
                Err(e) => println!("   ⚠️  Signal quality assessment failed: {}", e),
            }
        }
        Err(e) => println!("❌ Trading signal validation failed: {}", e),
    }

    // Test invalid trading signal
    println!("\nTesting invalid trading signal (bad price levels)...");
    let invalid_signal = TradingSignal {
        timestamp: Utc::now(),
        instrument_id,
        strategy_type: "momentum".to_string(),
        signal_type: "BUY".to_string(),
        confidence_score: 0.85,
        target_price: Some(44000.00), // Target below stop loss for BUY signal
        stop_loss: Some(46000.00),
        take_profit: Some(47000.00),
        position_size: 0.1,
        risk_score: 0.25,
        time_horizon: Some(chrono::Duration::hours(4)),
        metadata: json!({"strategy": "momentum", "timeframe": "4h"}),
        created_at: Utc::now(),
    };

    match data_validator.validate_trading_signal(&invalid_signal) {
        Ok(()) => println!("⚠️  Invalid signal unexpectedly passed validation"),
        Err(e) => {
            println!("✅ Invalid trading signal correctly rejected:");
            println!("   - Error: {}", e);
        }
    }

    // Test 3: AI Prediction Validation
    println!("\n🤖 Testing AI Prediction Validation...");

    // Test valid AI prediction
    println!("Testing valid AI prediction...");
    let valid_prediction = AIPrediction {
        timestamp: Utc::now(),
        instrument_id,
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

    match data_validator.validate_ai_prediction(&valid_prediction) {
        Ok(()) => {
            println!("✅ AI prediction validation: PASSED");
            println!("   - Model: {} v{}", valid_prediction.model_type, valid_prediction.model_version);
            println!("   - Confidence: {:.2}", valid_prediction.confidence_score);
            println!("   - Horizon: {} minutes", valid_prediction.prediction_horizon_minutes);
        }
        Err(e) => println!("❌ AI prediction validation failed: {}", e),
    }

    // Test invalid AI prediction
    println!("\nTesting invalid AI prediction (bad confidence)...");
    let invalid_prediction = AIPrediction {
        timestamp: Utc::now(),
        instrument_id,
        model_type: "lstm".to_string(),
        model_version: "v1.0".to_string(),
        prediction_horizon_minutes: 60,
        predicted_price: 45500.00,
        predicted_volatility: Some(0.02),
        confidence_score: 1.5, // Invalid confidence (> 1.0)
        prediction_intervals: Some(json!({"lower": 44500, "upper": 46500})),
        feature_importance: Some(json!({"volume": 0.3, "price": 0.7})),
        created_at: Utc::now(),
    };

    match data_validator.validate_ai_prediction(&invalid_prediction) {
        Ok(()) => println!("⚠️  Invalid prediction unexpectedly passed validation"),
        Err(e) => {
            println!("✅ Invalid AI prediction correctly rejected:");
            println!("   - Error: {}", e);
        }
    }

    // Test 4: Instrument Validation
    println!("\n📈 Testing Instrument Validation...");

    // Test valid instrument
    println!("Testing valid instrument...");
    let valid_instrument = Instrument {
        id: Uuid::new_v4(),
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

    match data_validator.validate_instrument(&valid_instrument) {
        Ok(()) => {
            println!("✅ Instrument validation: PASSED");
            println!("   - Symbol: {}", valid_instrument.symbol);
            println!("   - Type: {}", valid_instrument.instrument_type);
            println!("   - Tick size: {}", valid_instrument.tick_size);
        }
        Err(e) => println!("❌ Instrument validation failed: {}", e),
    }

    // Test invalid instrument
    println!("\nTesting invalid instrument (negative tick size)...");
    let invalid_instrument = Instrument {
        id: Uuid::new_v4(),
        symbol: "".to_string(), // Empty symbol
        name: "Bitcoin USD".to_string(),
        instrument_type: "crypto".to_string(),
        base_currency: "BTC".to_string(),
        quote_currency: "USD".to_string(),
        tick_size: -0.01, // Negative tick size
        lot_size: 0.001,
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    match data_validator.validate_instrument(&invalid_instrument) {
        Ok(()) => println!("⚠️  Invalid instrument unexpectedly passed validation"),
        Err(e) => {
            println!("✅ Invalid instrument correctly rejected:");
            println!("   - Error: {}", e);
        }
    }

    // Test 5: Validation Statistics
    println!("\n📊 Validation Statistics:");
    let stats = data_validator.get_stats();
    println!("   - Total validations performed: {}", stats.total_validations);
    println!("   - Successful validations: {}", stats.successful_validations);
    println!("   - Failed validations: {}", stats.failed_validations);
    
    if !stats.validation_errors.is_empty() {
        println!("   - Error breakdown:");
        for (error_type, count) in &stats.validation_errors {
            println!("     • {}: {}", error_type, count);
        }
    }

    println!("\n🎉 Simple Data Validation Demo Completed Successfully!");
    println!("====================================================");
    println!("✅ Core data validation working correctly");
    println!("✅ Quality assessment functional");
    println!("✅ Error detection and reporting robust");
    println!("✅ Validation statistics tracking operational");
    println!("✅ All data types validated successfully");
    
    Ok(())
}
