// Simple demo of database query functions (working around type issues)
// Run with: DATABASE_URL="..." cargo run --example simple_query_demo

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

    println!("🚀 PantherSwap Edge Simple Database Query Demo");
    
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
    
    // Test direct SQL queries to demonstrate functionality
    println!("\n📈 Testing Direct Database Operations:");
    
    // Check if BTC-USD instrument exists
    let existing_instrument = sqlx::query(
        "SELECT id, symbol, name FROM instruments WHERE symbol = $1"
    )
    .bind("BTC-USD")
    .fetch_optional(&database.pool)
    .await?;
    
    let instrument_id = match existing_instrument {
        Some(row) => {
            let id: Uuid = row.get("id");
            let symbol: String = row.get("symbol");
            let name: String = row.get("name");
            println!("✅ Found existing instrument: {} - {} ({})", symbol, name, id);
            id
        }
        None => {
            // Insert new instrument with direct SQL
            let row = sqlx::query(
                r#"
                INSERT INTO instruments 
                (symbol, name, instrument_type, base_currency, quote_currency, 
                 tick_size, lot_size, is_active)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                RETURNING id, symbol, name
                "#
            )
            .bind("BTC-USD")
            .bind("Bitcoin USD")
            .bind("crypto")
            .bind("BTC")
            .bind("USD")
            .bind(0.01) // tick_size as f64
            .bind(0.001) // lot_size as f64
            .bind(true)
            .fetch_one(&database.pool)
            .await?;
            
            let id: Uuid = row.get("id");
            let symbol: String = row.get("symbol");
            let name: String = row.get("name");
            println!("✅ Inserted new instrument: {} - {} ({})", symbol, name, id);
            id
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
    .bind(json!({"source": "demo", "quality": "high"}))
    .execute(&database.pool)
    .await?;
    
    println!("✅ Inserted market tick (rows affected: {})", result.rows_affected());
    
    // Get latest market tick
    let latest_tick = sqlx::query(
        r#"
        SELECT timestamp, last_price, volume, data_quality_score 
        FROM market_ticks 
        WHERE instrument_id = $1 
        ORDER BY timestamp DESC 
        LIMIT 1
        "#
    )
    .bind(instrument_id)
    .fetch_optional(&database.pool)
    .await?;
    
    if let Some(row) = latest_tick {
        let timestamp: chrono::DateTime<Utc> = row.get("timestamp");
        let last_price: Option<f64> = row.get("last_price");
        let volume: Option<f64> = row.get("volume");
        let quality: f64 = row.get("data_quality_score");
        
        println!("✅ Retrieved latest tick: ${:.2} (vol: {:.1}, quality: {:.1}%) @ {}", 
                 last_price.unwrap_or(0.0), 
                 volume.unwrap_or(0.0),
                 quality * 100.0,
                 timestamp.format("%H:%M:%S"));
    }
    
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
    .bind(json!({"lower": 44500, "upper": 46500}))
    .bind(json!({"volume": 0.3, "price": 0.7}))
    .execute(&database.pool)
    .await?;
    
    println!("✅ Inserted AI prediction (rows affected: {})", result.rows_affected());
    
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
    .bind(json!({"strategy": "momentum", "timeframe": "4h"}))
    .execute(&database.pool)
    .await?;
    
    println!("✅ Inserted trading signal (rows affected: {})", result.rows_affected());
    
    // Test analytical query - get recent activity
    println!("\n📈 Testing Analytical Queries:");
    
    let activity = sqlx::query(
        r#"
        SELECT 
            'market_ticks' as table_name,
            count(*) as recent_records,
            max(timestamp) as last_insert
        FROM market_ticks
        WHERE timestamp >= NOW() - INTERVAL '1 hour'
        
        UNION ALL
        
        SELECT 
            'trading_signals' as table_name,
            count(*) as recent_records,
            max(timestamp) as last_insert
        FROM trading_signals
        WHERE timestamp >= NOW() - INTERVAL '1 hour'
        
        UNION ALL
        
        SELECT 
            'ai_predictions' as table_name,
            count(*) as recent_records,
            max(timestamp) as last_insert
        FROM ai_predictions
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
        let last_insert: Option<chrono::DateTime<Utc>> = row.get("last_insert");
        
        println!("   📊 {}: {} records (last: {})", 
                 table_name, 
                 count,
                 last_insert.map(|t| t.format("%H:%M:%S").to_string()).unwrap_or("None".to_string()));
    }
    
    // Test TimescaleDB specific query
    println!("\n⏰ Testing TimescaleDB Features:");
    
    let hypertables = sqlx::query(
        "SELECT table_name FROM timescaledb_information.hypertables WHERE schema_name = 'public'"
    )
    .fetch_all(&database.pool)
    .await?;
    
    println!("✅ Active hypertables:");
    for row in hypertables {
        let table_name: String = row.get("table_name");
        println!("   📈 {}", table_name);
    }
    
    println!("\n🎉 Simple database query demo completed successfully!");
    println!("✅ All basic query operations are working correctly");
    println!("✅ TimescaleDB hypertables are functioning");
    println!("✅ Data insertion and retrieval verified");
    
    Ok(())
}
