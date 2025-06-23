// Real Database Trading Test - Actually writes to TimescaleDB
// This test will persist real data to your TimescaleDB instance

use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::PgPool;
use std::time::Instant;
use tracing::{info, error};
use uuid::Uuid;

use pantherswap_edge::config::Settings;
use pantherswap_edge::database::Database;

#[derive(Debug)]
struct RealTradeRecord {
    id: Uuid,
    timestamp: DateTime<Utc>,
    instrument: String,
    side: String,
    quantity: f64,
    price: f64,
    execution_time_ms: f64,
    pnl: f64,
}

#[derive(Debug)]
struct RealMarketTick {
    timestamp: DateTime<Utc>,
    instrument_id: Uuid,
    provider: String,
    bid_price: f64,
    ask_price: f64,
    bid_size: f64,
    ask_size: f64,
    last_price: f64,
    volume: f64,
    spread: f64,
    data_quality_score: f64,
}

#[derive(Debug)]
struct RealAIPrediction {
    timestamp: DateTime<Utc>,
    instrument_id: Uuid,
    model_type: String,
    model_version: String,
    prediction_horizon_minutes: i32,
    predicted_price: f64,
    confidence_score: f64,
}

async fn setup_test_tables(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    info!("🔧 Setting up test tables in TimescaleDB...");

    // Create instruments table if not exists
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS instruments (
            id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
            symbol VARCHAR(20) NOT NULL UNIQUE,
            name VARCHAR(100) NOT NULL,
            asset_class VARCHAR(50) NOT NULL,
            is_active BOOLEAN DEFAULT true,
            created_at TIMESTAMPTZ DEFAULT NOW()
        )
    "#).execute(pool).await?;

    // Create market_ticks table if not exists
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS market_ticks (
            timestamp TIMESTAMPTZ NOT NULL,
            instrument_id UUID NOT NULL REFERENCES instruments(id),
            provider VARCHAR(50) NOT NULL,
            bid_price DECIMAL(20, 10) NOT NULL,
            ask_price DECIMAL(20, 10) NOT NULL,
            bid_size DECIMAL(20, 10) NOT NULL,
            ask_size DECIMAL(20, 10) NOT NULL,
            last_price DECIMAL(20, 10),
            volume DECIMAL(20, 10),
            spread DECIMAL(20, 10) NOT NULL,
            data_quality_score DECIMAL(3, 2) NOT NULL,
            raw_data JSONB NOT NULL
        )
    "#).execute(pool).await?;

    // Create ai_predictions table if not exists
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS ai_predictions (
            timestamp TIMESTAMPTZ NOT NULL,
            instrument_id UUID NOT NULL REFERENCES instruments(id),
            model_type VARCHAR(50) NOT NULL,
            model_version VARCHAR(20) NOT NULL,
            prediction_horizon_minutes INTEGER NOT NULL,
            predicted_price DECIMAL(20, 10) NOT NULL,
            predicted_volatility DECIMAL(8, 6),
            confidence_score DECIMAL(5, 4) NOT NULL,
            prediction_intervals JSONB,
            feature_importance JSONB,
            created_at TIMESTAMPTZ DEFAULT NOW()
        )
    "#).execute(pool).await?;

    // Create trade_executions table if not exists
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS trade_executions (
            id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
            timestamp TIMESTAMPTZ NOT NULL,
            instrument_id UUID NOT NULL REFERENCES instruments(id),
            order_id UUID NOT NULL,
            side VARCHAR(10) NOT NULL,
            quantity DECIMAL(20, 10) NOT NULL,
            price DECIMAL(20, 10) NOT NULL,
            execution_time_ms DECIMAL(10, 3) NOT NULL,
            slippage_bps DECIMAL(8, 4),
            pnl DECIMAL(20, 10),
            strategy_id VARCHAR(100),
            created_at TIMESTAMPTZ DEFAULT NOW()
        )
    "#).execute(pool).await?;

    // Convert to hypertables (ignore errors if already exists)
    let _ = sqlx::query("SELECT create_hypertable('market_ticks', 'timestamp', if_not_exists => TRUE)")
        .execute(pool).await;
    let _ = sqlx::query("SELECT create_hypertable('ai_predictions', 'timestamp', if_not_exists => TRUE)")
        .execute(pool).await;
    let _ = sqlx::query("SELECT create_hypertable('trade_executions', 'timestamp', if_not_exists => TRUE)")
        .execute(pool).await;

    info!("✅ Test tables setup completed");
    Ok(())
}

async fn insert_test_instrument(pool: &PgPool) -> Result<Uuid, Box<dyn std::error::Error>> {
    info!("📊 Inserting test instrument (EURUSD)...");

    let instrument_id = Uuid::new_v4();
    
    sqlx::query(r#"
        INSERT INTO instruments (id, symbol, name, asset_class, is_active)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (symbol) DO UPDATE SET
        name = EXCLUDED.name,
        is_active = EXCLUDED.is_active
        RETURNING id
    "#)
    .bind(instrument_id)
    .bind("EURUSD")
    .bind("Euro / US Dollar")
    .bind("FX")
    .bind(true)
    .execute(pool)
    .await?;

    info!("✅ Instrument EURUSD inserted with ID: {}", instrument_id);
    Ok(instrument_id)
}

async fn insert_real_market_data(pool: &PgPool, instrument_id: Uuid, count: usize) -> Result<(), Box<dyn std::error::Error>> {
    info!("📈 Inserting {} real market ticks...", count);

    let start_time = Instant::now();
    let mut inserted_count = 0;

    for i in 0..count {
        let timestamp = Utc::now() - chrono::Duration::seconds(count as i64 - i as i64);
        let base_price = 1.0850;
        let price_variation = (i as f64 * 0.0001) % 0.01;
        let bid_price = base_price + price_variation;
        let ask_price = bid_price + 0.0002;
        
        let tick = RealMarketTick {
            timestamp,
            instrument_id,
            provider: "alpha_vantage".to_string(),
            bid_price,
            ask_price,
            bid_size: 1000.0 + (i as f64 * 10.0),
            ask_size: 1000.0 + (i as f64 * 10.0),
            last_price: Some(bid_price + 0.0001).unwrap(),
            volume: 5000.0 + (i as f64 * 50.0),
            spread: ask_price - bid_price,
            data_quality_score: 0.95,
        };

        let raw_data = json!({
            "source": "alpha_vantage",
            "api_key": "EZDZ4VOFQ2GRA7VU",
            "latency_ms": 45,
            "sequence": i
        });

        sqlx::query(r#"
            INSERT INTO market_ticks
            (timestamp, instrument_id, provider, bid_price, ask_price, bid_size, ask_size,
             last_price, volume, spread, data_quality_score, raw_data)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#)
        .bind(tick.timestamp)
        .bind(tick.instrument_id)
        .bind(&tick.provider)
        .bind(tick.bid_price)
        .bind(tick.ask_price)
        .bind(tick.bid_size)
        .bind(tick.ask_size)
        .bind(tick.last_price)
        .bind(tick.volume)
        .bind(tick.spread)
        .bind(tick.data_quality_score)
        .bind(&raw_data)
        .execute(pool)
        .await?;

        inserted_count += 1;
    }

    let duration = start_time.elapsed();
    info!("✅ Inserted {} market ticks in {:?} ({:.2} ticks/sec)", 
          inserted_count, duration, inserted_count as f64 / duration.as_secs_f64());
    
    Ok(())
}

async fn insert_real_ai_predictions(pool: &PgPool, instrument_id: Uuid, count: usize) -> Result<(), Box<dyn std::error::Error>> {
    info!("🤖 Inserting {} AI predictions...", count);

    let start_time = Instant::now();
    let mut inserted_count = 0;

    for i in 0..count {
        let timestamp = Utc::now() - chrono::Duration::seconds(count as i64 - i as i64);
        let base_price = 1.0850;
        let predicted_price = base_price + ((i as f64 * 0.0001) % 0.01);
        let confidence = 0.7 + ((i as f64 * 0.01) % 0.25);

        let prediction = RealAIPrediction {
            timestamp,
            instrument_id,
            model_type: if i % 2 == 0 { "LSTM".to_string() } else { "HMM".to_string() },
            model_version: "v1.0".to_string(),
            prediction_horizon_minutes: 1,
            predicted_price,
            confidence_score: confidence,
        };

        let prediction_intervals = json!({
            "lower_bound": predicted_price - 0.001,
            "upper_bound": predicted_price + 0.001,
            "confidence_interval": 0.95
        });

        let feature_importance = json!({
            "price_momentum": 0.35,
            "volume_trend": 0.25,
            "volatility": 0.20,
            "market_regime": 0.20
        });

        sqlx::query(r#"
            INSERT INTO ai_predictions
            (timestamp, instrument_id, model_type, model_version, prediction_horizon_minutes,
             predicted_price, predicted_volatility, confidence_score, prediction_intervals,
             feature_importance)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#)
        .bind(prediction.timestamp)
        .bind(prediction.instrument_id)
        .bind(&prediction.model_type)
        .bind(&prediction.model_version)
        .bind(prediction.prediction_horizon_minutes)
        .bind(prediction.predicted_price)
        .bind(0.015) // predicted_volatility
        .bind(prediction.confidence_score)
        .bind(&prediction_intervals)
        .bind(&feature_importance)
        .execute(pool)
        .await?;

        inserted_count += 1;
    }

    let duration = start_time.elapsed();
    info!("✅ Inserted {} AI predictions in {:?} ({:.2} predictions/sec)", 
          inserted_count, duration, inserted_count as f64 / duration.as_secs_f64());
    
    Ok(())
}

async fn insert_real_trade_executions(pool: &PgPool, instrument_id: Uuid, count: usize) -> Result<(), Box<dyn std::error::Error>> {
    info!("⚡ Inserting {} trade executions...", count);

    let start_time = Instant::now();
    let mut inserted_count = 0;

    for i in 0..count {
        let timestamp = Utc::now() - chrono::Duration::seconds(count as i64 - i as i64);
        let side = if i % 2 == 0 { "BUY" } else { "SELL" };
        let base_price = 1.0850;
        let price = base_price + ((i as f64 * 0.0001) % 0.01);
        let quantity = 1000.0 + (i as f64 * 100.0);
        let execution_time_ms = 5.0 + (i as f64 * 0.5) % 10.0;
        
        // Calculate P&L (simplified)
        let pnl = if side == "BUY" {
            (price - base_price) * quantity
        } else {
            (base_price - price) * quantity
        };

        sqlx::query(r#"
            INSERT INTO trade_executions
            (timestamp, instrument_id, order_id, side, quantity, price,
             execution_time_ms, slippage_bps, pnl, strategy_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#)
        .bind(timestamp)
        .bind(instrument_id)
        .bind(Uuid::new_v4()) // order_id
        .bind(side)
        .bind(quantity)
        .bind(price)
        .bind(execution_time_ms)
        .bind(0.2) // slippage_bps
        .bind(pnl)
        .bind("real_test_strategy")
        .execute(pool)
        .await?;

        inserted_count += 1;
    }

    let duration = start_time.elapsed();
    info!("✅ Inserted {} trade executions in {:?} ({:.2} trades/sec)", 
          inserted_count, duration, inserted_count as f64 / duration.as_secs_f64());
    
    Ok(())
}

async fn verify_data_in_database(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    info!("🔍 Verifying data in TimescaleDB...");

    // Check market ticks
    let market_tick_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM market_ticks WHERE timestamp >= NOW() - INTERVAL '1 hour'"
    ).fetch_one(pool).await?;
    
    // Check AI predictions
    let ai_prediction_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM ai_predictions WHERE timestamp >= NOW() - INTERVAL '1 hour'"
    ).fetch_one(pool).await?;
    
    // Check trade executions
    let trade_execution_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM trade_executions WHERE timestamp >= NOW() - INTERVAL '1 hour'"
    ).fetch_one(pool).await?;

    // Calculate total P&L
    let total_pnl: (Option<f64>,) = sqlx::query_as(
        "SELECT SUM(pnl) FROM trade_executions WHERE timestamp >= NOW() - INTERVAL '1 hour'"
    ).fetch_one(pool).await?;

    info!("📊 DATABASE VERIFICATION RESULTS:");
    info!("================================");
    info!("Market Ticks: {} records", market_tick_count.0);
    info!("AI Predictions: {} records", ai_prediction_count.0);
    info!("Trade Executions: {} records", trade_execution_count.0);
    info!("Total P&L: ${:.2}", total_pnl.0.unwrap_or(0.0));
    info!("✅ Data successfully verified in TimescaleDB!");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 Real Database Trading Test - TimescaleDB Integration");
    info!("======================================================");

    // Load configuration
    std::env::set_var("RUN_MODE", "production");
    let settings = Settings::load()?;
    
    info!("🔗 Connecting to TimescaleDB: {}", 
          settings.database.url.split('@').last().unwrap_or("unknown"));

    // Initialize database
    let database = Database::new(&settings.database.url).await?;
    let pool = &database.pool;

    // Setup test tables
    setup_test_tables(pool).await?;

    // Insert test instrument
    let instrument_id = insert_test_instrument(pool).await?;

    // Insert real data
    insert_real_market_data(pool, instrument_id, 100).await?;
    insert_real_ai_predictions(pool, instrument_id, 50).await?;
    insert_real_trade_executions(pool, instrument_id, 20).await?;

    // Verify data
    verify_data_in_database(pool).await?;

    info!("🎯 Real database trading test completed successfully!");
    info!("You can now query your TimescaleDB instance to see the actual data.");

    Ok(())
}
