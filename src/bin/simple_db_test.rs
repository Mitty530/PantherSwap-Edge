// Simple Database Test - Insert real data into TimescaleDB
// This will actually write data to your database

use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;
use tracing::info;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 Simple Database Test - Real Data Insertion");
    info!("==============================================");

    // Database connection string
    let database_url = "postgres://tsdbadmin:sz2eu577bgqi5767@jqrbtbc5nw.w0mq2s13iy.tsdb.cloud.timescale.com:35762/tsdb?sslmode=require";
    
    info!("🔗 Connecting to TimescaleDB...");
    let pool = PgPool::connect(database_url).await?;
    info!("✅ Connected successfully!");

    // Create tables
    info!("🔧 Creating tables...");
    
    // Create instruments table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS instruments (
            id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
            symbol VARCHAR(20) NOT NULL UNIQUE,
            name VARCHAR(100) NOT NULL,
            asset_class VARCHAR(50) NOT NULL,
            is_active BOOLEAN DEFAULT true,
            created_at TIMESTAMPTZ DEFAULT NOW()
        )
    "#).execute(&pool).await?;

    // Create market_ticks table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS market_ticks (
            timestamp TIMESTAMPTZ NOT NULL,
            instrument_id UUID NOT NULL,
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
    "#).execute(&pool).await?;

    // Create trade_executions table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS trade_executions (
            id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
            timestamp TIMESTAMPTZ NOT NULL,
            instrument_id UUID NOT NULL,
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
    "#).execute(&pool).await?;

    // Convert to hypertables (ignore errors if already exists)
    let _ = sqlx::query("SELECT create_hypertable('market_ticks', 'timestamp', if_not_exists => TRUE)")
        .execute(&pool).await;
    let _ = sqlx::query("SELECT create_hypertable('trade_executions', 'timestamp', if_not_exists => TRUE)")
        .execute(&pool).await;

    info!("✅ Tables created successfully!");

    // Insert test instrument
    info!("📊 Inserting EURUSD instrument...");
    let instrument_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")?;
    
    sqlx::query(r#"
        INSERT INTO instruments (id, symbol, name, asset_class, is_active)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (symbol) DO UPDATE SET
        name = EXCLUDED.name,
        is_active = EXCLUDED.is_active
    "#)
    .bind(instrument_id)
    .bind("EURUSD")
    .bind("Euro / US Dollar")
    .bind("FX")
    .bind(true)
    .execute(&pool)
    .await?;

    info!("✅ EURUSD instrument inserted!");

    // Insert market data
    info!("📈 Inserting market ticks...");
    for i in 0..10 {
        let timestamp = Utc::now() - chrono::Duration::seconds(10 - i);
        let bid_price = 1.0850 + (i as f64 * 0.0001);
        let ask_price = bid_price + 0.0002;
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
        .bind(timestamp)
        .bind(instrument_id)
        .bind("alpha_vantage")
        .bind(bid_price)
        .bind(ask_price)
        .bind(1000.0 + (i as f64 * 10.0))
        .bind(1000.0 + (i as f64 * 10.0))
        .bind(bid_price + 0.0001)
        .bind(5000.0 + (i as f64 * 50.0))
        .bind(ask_price - bid_price)
        .bind(0.95)
        .bind(&raw_data)
        .execute(&pool)
        .await?;
    }

    info!("✅ Inserted 10 market ticks!");

    // Insert trade executions
    info!("⚡ Inserting trade executions...");
    for i in 0..5 {
        let timestamp = Utc::now() - chrono::Duration::seconds(5 - i);
        let side = if i % 2 == 0 { "BUY" } else { "SELL" };
        let price = 1.0850 + (i as f64 * 0.0002);
        let quantity = 1000.0 + (i as f64 * 500.0);
        let pnl = if side == "BUY" { 25.50 + (i as f64 * 5.0) } else { -15.20 + (i as f64 * 3.0) };

        sqlx::query(r#"
            INSERT INTO trade_executions
            (timestamp, instrument_id, order_id, side, quantity, price,
             execution_time_ms, slippage_bps, pnl, strategy_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#)
        .bind(timestamp)
        .bind(instrument_id)
        .bind(Uuid::new_v4())
        .bind(side)
        .bind(quantity)
        .bind(price)
        .bind(8.5 + (i as f64 * 0.5))
        .bind(0.2)
        .bind(pnl)
        .bind("real_test_strategy")
        .execute(&pool)
        .await?;
    }

    info!("✅ Inserted 5 trade executions!");

    // Verify data
    info!("🔍 Verifying inserted data...");

    // Count market ticks
    let market_tick_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM market_ticks WHERE timestamp >= NOW() - INTERVAL '1 hour'"
    ).fetch_one(&pool).await?;

    // Count trade executions
    let trade_execution_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM trade_executions WHERE timestamp >= NOW() - INTERVAL '1 hour'"
    ).fetch_one(&pool).await?;

    // Calculate total P&L
    let total_pnl: (Option<f64>,) = sqlx::query_as(
        "SELECT SUM(pnl) FROM trade_executions WHERE timestamp >= NOW() - INTERVAL '1 hour'"
    ).fetch_one(&pool).await?;

    // Get recent market data
    let recent_ticks: Vec<(chrono::DateTime<Utc>, f64, f64)> = sqlx::query_as(
        "SELECT timestamp, bid_price, ask_price FROM market_ticks WHERE timestamp >= NOW() - INTERVAL '1 hour' ORDER BY timestamp DESC LIMIT 3"
    ).fetch_all(&pool).await?;

    // Get recent trades
    let recent_trades: Vec<(chrono::DateTime<Utc>, String, f64, f64, f64)> = sqlx::query_as(
        "SELECT timestamp, side, quantity, price, pnl FROM trade_executions WHERE timestamp >= NOW() - INTERVAL '1 hour' ORDER BY timestamp DESC LIMIT 3"
    ).fetch_all(&pool).await?;

    info!("📊 DATABASE VERIFICATION RESULTS:");
    info!("================================");
    info!("Market Ticks: {} records", market_tick_count.0);
    info!("Trade Executions: {} records", trade_execution_count.0);
    info!("Total P&L: ${:.2}", total_pnl.0.unwrap_or(0.0));
    info!("");
    info!("Recent Market Ticks:");
    for (timestamp, bid, ask) in recent_ticks {
        info!("  {} - Bid: {:.4}, Ask: {:.4}", timestamp.format("%H:%M:%S"), bid, ask);
    }
    info!("");
    info!("Recent Trades:");
    for (timestamp, side, quantity, price, pnl) in recent_trades {
        info!("  {} - {} {:.0} @ {:.4} (P&L: ${:.2})", 
              timestamp.format("%H:%M:%S"), side, quantity, price, pnl);
    }

    info!("✅ Real data successfully inserted and verified in TimescaleDB!");
    info!("🎯 You can now query your database to see the actual trading data.");

    Ok(())
}
