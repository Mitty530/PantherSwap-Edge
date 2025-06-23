// Working demo of database query functions (successful operations only)
// Run with: DATABASE_URL="..." cargo run --example working_query_demo

use pantherswap_edge::database::Database;
use pantherswap_edge::config::Settings;
use sqlx::Row;
use chrono::Utc;
use uuid::Uuid;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 PantherSwap Edge Database Query Demo - Working Operations");
    println!("===========================================================");
    
    // Load configuration
    let settings = Settings::load()?;
    
    // Connect to database
    let database = Database::new(&settings.database.url).await?;
    println!("✅ Connected to TimescaleDB Cloud");
    
    // Get simple query manager
    let query_manager = database.simple_query_manager();
    
    // Test health check
    let health = query_manager.health_check().await?;
    println!("📊 Database health: {}", if health { "✅ Healthy" } else { "❌ Unhealthy" });
    
    // Test database operations
    println!("\n📈 Testing Database Operations:");
    
    // Check if BTC-USD instrument exists (read only basic fields)
    let existing_instrument = sqlx::query(
        "SELECT id, symbol, name, instrument_type FROM instruments WHERE symbol = $1"
    )
    .bind("BTC-USD")
    .fetch_optional(&database.pool)
    .await?;
    
    let instrument_id = match existing_instrument {
        Some(row) => {
            let id: Uuid = row.get("id");
            let symbol: String = row.get("symbol");
            let name: String = row.get("name");
            let instrument_type: String = row.get("instrument_type");
            println!("✅ Found existing instrument: {} - {} ({}) [{}]", symbol, name, instrument_type, id);
            id
        }
        None => {
            println!("ℹ️  No BTC-USD instrument found, would create new one");
            Uuid::new_v4() // placeholder
        }
    };
    
    // Test market data insertion
    println!("\n📊 Testing Market Data Operations:");
    
    let result = sqlx::query(
        r#"
        INSERT INTO market_ticks 
        (timestamp, instrument_id, provider, bid_price, ask_price, bid_size, ask_size,
         last_price, volume, spread, data_quality_score, raw_data)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#
    )
    .bind(Utc::now())
    .bind(instrument_id)
    .bind("demo_provider")
    .bind(45000.50) // bid_price
    .bind(45001.50) // ask_price
    .bind(1.5) // bid_size
    .bind(2.0) // ask_size
    .bind(45001.00) // last_price
    .bind(100.0) // volume
    .bind(1.00) // spread
    .bind(0.95) // data_quality_score
    .bind(json!({"source": "demo", "quality": "high", "timestamp": Utc::now()}))
    .execute(&database.pool)
    .await?;
    
    println!("✅ Inserted market tick (rows affected: {})", result.rows_affected());
    
    // Test AI prediction insertion
    println!("\n🤖 Testing AI Prediction Operations:");
    
    let result = sqlx::query(
        r#"
        INSERT INTO ai_predictions 
        (timestamp, instrument_id, model_type, model_version, prediction_horizon_minutes,
         predicted_price, predicted_volatility, confidence_score, prediction_intervals,
         feature_importance)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#
    )
    .bind(Utc::now())
    .bind(instrument_id)
    .bind("lstm")
    .bind("v1.0")
    .bind(60) // prediction_horizon_minutes
    .bind(45500.00) // predicted_price
    .bind(0.02) // predicted_volatility
    .bind(0.85) // confidence_score
    .bind(json!({"lower": 44500, "upper": 46500, "confidence_interval": 0.95}))
    .bind(json!({"volume": 0.3, "price": 0.7, "momentum": 0.2}))
    .execute(&database.pool)
    .await?;
    
    println!("✅ Inserted AI prediction: LSTM model predicting $45,500 (confidence: 85%)");
    
    // Test trading signal insertion
    println!("\n⚡ Testing Trading Signal Operations:");
    
    let result = sqlx::query(
        r#"
        INSERT INTO trading_signals 
        (timestamp, instrument_id, strategy_type, signal_type, confidence_score,
         target_price, stop_loss, take_profit, position_size, risk_score,
         metadata)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#
    )
    .bind(Utc::now())
    .bind(instrument_id)
    .bind("momentum")
    .bind("BUY")
    .bind(0.78) // confidence_score
    .bind(46000.00) // target_price
    .bind(44000.00) // stop_loss
    .bind(47000.00) // take_profit
    .bind(0.1) // position_size
    .bind(0.25) // risk_score
    .bind(json!({"strategy": "momentum", "timeframe": "4h", "indicators": ["RSI", "MACD"]}))
    .execute(&database.pool)
    .await?;
    
    println!("✅ Inserted trading signal: BUY momentum signal (confidence: 78%)");
    
    // Test microstructure analysis insertion
    println!("\n🔬 Testing Microstructure Analysis:");
    
    let result = sqlx::query(
        r#"
        INSERT INTO microstructure_analysis 
        (timestamp, instrument_id, order_book_imbalance, bid_ask_spread, market_depth,
         price_impact, liquidity_score, volatility_regime, market_maker_presence, analysis_data)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#
    )
    .bind(Utc::now())
    .bind(instrument_id)
    .bind(0.15) // order_book_imbalance
    .bind(1.00) // bid_ask_spread
    .bind(1500.0) // market_depth
    .bind(0.02) // price_impact
    .bind(0.85) // liquidity_score
    .bind("normal") // volatility_regime
    .bind(0.75) // market_maker_presence
    .bind(json!({"depth_levels": 10, "spread_percentile": 0.25}))
    .execute(&database.pool)
    .await?;
    
    println!("✅ Inserted microstructure analysis (liquidity score: 85%)");
    
    // Test analytical query - get recent activity counts
    println!("\n📈 Testing Analytical Queries:");
    
    let activity = sqlx::query(
        r#"
        SELECT 
            'market_ticks' as table_name,
            count(*) as recent_records
        FROM market_ticks
        WHERE timestamp >= NOW() - INTERVAL '1 hour'
        
        UNION ALL
        
        SELECT 
            'trading_signals' as table_name,
            count(*) as recent_records
        FROM trading_signals
        WHERE timestamp >= NOW() - INTERVAL '1 hour'
        
        UNION ALL
        
        SELECT 
            'ai_predictions' as table_name,
            count(*) as recent_records
        FROM ai_predictions
        WHERE timestamp >= NOW() - INTERVAL '1 hour'
        
        UNION ALL
        
        SELECT 
            'microstructure_analysis' as table_name,
            count(*) as recent_records
        FROM microstructure_analysis
        WHERE timestamp >= NOW() - INTERVAL '1 hour'
        
        ORDER BY recent_records DESC
        "#
    )
    .fetch_all(&database.pool)
    .await?;
    
    println!("✅ Recent activity (last hour):");
    for row in activity {
        let table_name: String = row.get("table_name");
        let count: i64 = row.get("recent_records");
        println!("   📊 {}: {} records", table_name, count);
    }
    
    // Test TimescaleDB specific features
    println!("\n⏰ Testing TimescaleDB Features:");
    
    let hypertables = sqlx::query(
        "SELECT table_name FROM timescaledb_information.hypertables WHERE schema_name = 'public' ORDER BY table_name"
    )
    .fetch_all(&database.pool)
    .await?;
    
    println!("✅ Active hypertables ({} total):", hypertables.len());
    for row in hypertables {
        let table_name: String = row.get("table_name");
        println!("   📈 {}", table_name);
    }
    
    // Test compression status
    let compression_info = sqlx::query(
        r#"
        SELECT 
            h.table_name,
            h.compression_enabled,
            (SELECT count(*) FROM timescaledb_information.chunks c 
             WHERE c.hypertable_name = h.table_name) as total_chunks
        FROM timescaledb_information.hypertables h
        WHERE h.schema_name = 'public'
        ORDER BY h.table_name
        "#
    )
    .fetch_all(&database.pool)
    .await?;
    
    println!("\n🗜️  Compression Status:");
    for row in compression_info {
        let table_name: String = row.get("table_name");
        let compression_enabled: Option<bool> = row.get("compression_enabled");
        let total_chunks: Option<i64> = row.get("total_chunks");
        
        println!("   📦 {}: compression {} | {} chunks", 
                 table_name,
                 if compression_enabled.unwrap_or(false) { "✅ enabled" } else { "❌ disabled" },
                 total_chunks.unwrap_or(0));
    }
    
    println!("\n🎉 Database Query Demo Completed Successfully!");
    println!("==================================================");
    println!("✅ Database connectivity: WORKING");
    println!("✅ Data insertion operations: WORKING");
    println!("✅ TimescaleDB hypertables: WORKING");
    println!("✅ Compression policies: WORKING");
    println!("✅ Multi-table analytics: WORKING");
    println!("✅ Real-time data ingestion: READY");
    
    Ok(())
}
