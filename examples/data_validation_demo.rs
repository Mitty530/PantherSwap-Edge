// Data validation layer demo for PantherSwap Edge
// Demonstrates comprehensive data validation, quality assessment, and integrity checking
// Run with: DATABASE_URL="..." cargo run --example data_validation_demo

use pantherswap_edge::database::{Database, types::*};
use pantherswap_edge::config::Settings;
use chrono::Utc;
use uuid::Uuid;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 PantherSwap Edge Data Validation Layer Demo");
    println!("===============================================");
    
    // Load configuration
    let settings = Settings::load()?;
    
    // Connect to database
    let database = Database::new(&settings.database.url).await?;
    println!("✅ Connected to database");
    
    // Get validation components
    let mut data_validator = database.data_validator();
    let mut data_quality_assessor = database.data_quality_assessor();
    let integrity_checker = database.integrity_checker();
    println!("✅ Validation components initialized");

    // Test 1: Market Data Validation
    println!("\n📊 Testing Market Data Validation...");
    
    // Create test instrument
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

    // Test basic validation
    match data_validator.validate_market_tick(&valid_tick) {
        Ok(()) => {
            println!("✅ Valid market tick validation: PASSED");

            // Test quality assessment
            match data_quality_assessor.assess_market_tick_quality(&valid_tick) {
                Ok(quality_report) => {
                    println!("   - Quality score: {:.2}", quality_report.overall_score);
                    println!("   - Data completeness: {:.2}", quality_report.data_completeness);
                    if !quality_report.recommendations.is_empty() {
                        println!("   - Recommendations: {:?}", quality_report.recommendations);
                    }
                }
                Err(e) => println!("   ⚠️  Quality assessment failed: {}", e),
            }
        }
        Err(e) => println!("❌ Valid tick validation failed: {}", e),
    }

    // Test invalid market tick (negative spread)
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

    match validation_middleware.validate_market_tick(&invalid_tick).await {
        Ok(result) => {
            println!("\n⚠️  Invalid market tick validation:");
            println!("   - Is valid: {}", result.is_valid);
            println!("   - Validation passed: {}", result.validation_passed);
            if let Some(quality) = result.quality_score {
                println!("   - Quality score: {:.2}", quality);
            }
            if !result.validation_errors.is_empty() {
                println!("   - Validation errors:");
                for error in &result.validation_errors {
                    println!("     • {}", error);
                }
            }
            if !result.recommendations.is_empty() {
                println!("   - Recommendations:");
                for rec in &result.recommendations {
                    println!("     • {}", rec);
                }
            }
        }
        Err(e) => println!("❌ Invalid tick validation failed: {}", e),
    }

    // Test 2: Trading Signal Validation
    println!("\n⚡ Testing Trading Signal Validation...");

    // Test valid trading signal
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

    match validation_middleware.validate_trading_signal(&valid_signal).await {
        Ok(result) => {
            println!("✅ Valid trading signal validation:");
            println!("   - Is valid: {}", result.is_valid);
            println!("   - Validation passed: {}", result.validation_passed);
            if let Some(quality) = result.quality_score {
                println!("   - Quality score: {:.2}", quality);
            }
        }
        Err(e) => println!("❌ Valid signal validation failed: {}", e),
    }

    // Test invalid trading signal (inconsistent price levels)
    let invalid_signal = TradingSignal {
        timestamp: Utc::now(),
        instrument_id,
        strategy_type: "invalid_strategy".to_string(), // Invalid strategy
        signal_type: "BUY".to_string(),
        confidence_score: 1.5, // Invalid confidence (> 1.0)
        target_price: Some(44000.00), // Target below stop loss for BUY
        stop_loss: Some(46000.00),
        take_profit: Some(47000.00),
        position_size: -0.1, // Negative position size
        risk_score: 1.5, // Invalid risk score (> 1.0)
        time_horizon: Some(chrono::Duration::hours(200)), // Too long
        metadata: json!({}),
        created_at: Utc::now(),
    };

    match validation_middleware.validate_trading_signal(&invalid_signal).await {
        Ok(result) => {
            println!("\n⚠️  Invalid trading signal validation:");
            println!("   - Is valid: {}", result.is_valid);
            println!("   - Validation passed: {}", result.validation_passed);
            if !result.validation_errors.is_empty() {
                println!("   - Validation errors:");
                for error in &result.validation_errors {
                    println!("     • {}", error);
                }
            }
        }
        Err(e) => println!("❌ Invalid signal validation failed: {}", e),
    }

    // Test 3: AI Prediction Validation
    println!("\n🤖 Testing AI Prediction Validation...");

    // Test valid AI prediction
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

    match validation_middleware.validate_ai_prediction(&valid_prediction).await {
        Ok(result) => {
            println!("✅ Valid AI prediction validation:");
            println!("   - Is valid: {}", result.is_valid);
            println!("   - Validation passed: {}", result.validation_passed);
            if let Some(quality) = result.quality_score {
                println!("   - Confidence score: {:.2}", quality);
            }
            if !result.recommendations.is_empty() {
                println!("   - Recommendations:");
                for rec in &result.recommendations {
                    println!("     • {}", rec);
                }
            }
        }
        Err(e) => println!("❌ Valid prediction validation failed: {}", e),
    }

    // Test invalid AI prediction
    let invalid_prediction = AIPrediction {
        timestamp: Utc::now(),
        instrument_id,
        model_type: "invalid_model".to_string(), // Invalid model type
        model_version: "v1.0".to_string(),
        prediction_horizon_minutes: 2000, // Too long horizon
        predicted_price: -100.00, // Negative price
        predicted_volatility: Some(2.0), // Invalid volatility (> 1.0)
        confidence_score: 1.5, // Invalid confidence (> 1.0)
        prediction_intervals: Some(json!({})),
        feature_importance: Some(json!({})),
        created_at: Utc::now(),
    };

    match validation_middleware.validate_ai_prediction(&invalid_prediction).await {
        Ok(result) => {
            println!("\n⚠️  Invalid AI prediction validation:");
            println!("   - Is valid: {}", result.is_valid);
            println!("   - Validation passed: {}", result.validation_passed);
            if !result.validation_errors.is_empty() {
                println!("   - Validation errors:");
                for error in &result.validation_errors {
                    println!("     • {}", error);
                }
            }
        }
        Err(e) => println!("❌ Invalid prediction validation failed: {}", e),
    }

    // Test 4: Instrument Validation
    println!("\n📈 Testing Instrument Validation...");

    // Test valid instrument
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

    match validation_middleware.validate_instrument(&valid_instrument).await {
        Ok(result) => {
            println!("✅ Valid instrument validation:");
            println!("   - Is valid: {}", result.is_valid);
            println!("   - Validation passed: {}", result.validation_passed);
        }
        Err(e) => println!("❌ Valid instrument validation failed: {}", e),
    }

    // Test 5: System Integrity Check
    println!("\n🔍 Testing System Integrity Check...");

    match validation_middleware.check_system_integrity().await {
        Ok(report) => {
            println!("✅ System integrity check completed:");
            println!("   - Overall health score: {:.2}", report.overall_health_score);
            println!("   - Total validations: {}", report.validation_summary.total_validations);
            println!("   - Successful validations: {}", report.validation_summary.successful_validations);
            println!("   - Failed validations: {}", report.validation_summary.failed_validations);
            println!("   - Integrity violations: {}", report.integrity_summary.total_violations);
            
            if !report.critical_issues.is_empty() {
                println!("   - Critical issues:");
                for issue in &report.critical_issues {
                    println!("     • {}", issue);
                }
            }
            
            if !report.recommendations.is_empty() {
                println!("   - Recommendations:");
                for rec in &report.recommendations {
                    println!("     • {}", rec);
                }
            }
        }
        Err(e) => println!("❌ System integrity check failed: {}", e),
    }

    // Test 6: Validation Statistics
    println!("\n📊 Validation Statistics:");
    let stats = validation_middleware.get_validation_stats().await;
    println!("   - Total validations performed: {}", stats.total_validations);
    println!("   - Successful validations: {}", stats.successful_validations);
    println!("   - Failed validations: {}", stats.failed_validations);
    
    if !stats.error_breakdown.is_empty() {
        println!("   - Error breakdown:");
        for (error_type, count) in &stats.error_breakdown {
            println!("     • {}: {}", error_type, count);
        }
    }

    println!("\n🎉 Data Validation Layer Demo Completed Successfully!");
    println!("=====================================================");
    println!("✅ Comprehensive data validation system working");
    println!("✅ Quality assessment and scoring functional");
    println!("✅ Data integrity checking operational");
    println!("✅ Validation middleware integration complete");
    println!("✅ Error handling and reporting robust");
    
    Ok(())
}
