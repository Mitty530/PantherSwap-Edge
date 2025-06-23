use pantherswap_edge::database::Database;
use tracing::{info, error};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 Setting up database tables for trading execution logging");

    // Get database URL from environment or use default
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://tsdbadmin:sz2eu577bgqi5767@jqrbtbc5nw.w0mq2s13iy.tsdb.cloud.timescale.com:35762/tsdb?sslmode=require".to_string());

    info!("📊 Connecting to database: {}", database_url.split('@').last().unwrap_or("unknown"));

    // Create database connection
    let database = Database::new_production(&database_url).await?;
    info!("✅ Database connection established");

    // Create orders table
    info!("🔄 Creating orders table...");
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS orders (
            id UUID PRIMARY KEY,
            instrument_id UUID NOT NULL,
            side VARCHAR(10) NOT NULL,
            quantity DECIMAL(20, 10) NOT NULL,
            filled_quantity DECIMAL(20, 10) NOT NULL DEFAULT 0,
            remaining_quantity DECIMAL(20, 10) NOT NULL,
            order_type VARCHAR(20) NOT NULL,
            price DECIMAL(20, 10),
            stop_price DECIMAL(20, 10),
            time_in_force VARCHAR(10) NOT NULL,
            execution_style VARCHAR(20) NOT NULL,
            status VARCHAR(20) NOT NULL,
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL,
            strategy_name VARCHAR(100),
            metadata JSONB
        )
    "#)
    .execute(&database.pool)
    .await?;

    // Create fills table
    info!("🔄 Creating fills table...");
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS fills (
            id UUID PRIMARY KEY,
            order_id UUID NOT NULL,
            quantity DECIMAL(20, 10) NOT NULL,
            price DECIMAL(20, 10) NOT NULL,
            timestamp TIMESTAMPTZ NOT NULL,
            commission DECIMAL(20, 10) NOT NULL DEFAULT 0,
            venue VARCHAR(50) NOT NULL DEFAULT 'internal',
            liquidity_flag VARCHAR(20) NOT NULL DEFAULT 'unknown',
            metadata JSONB
        )
    "#)
    .execute(&database.pool)
    .await?;

    // Create position_updates table
    info!("🔄 Creating position_updates table...");
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS position_updates (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            timestamp TIMESTAMPTZ NOT NULL,
            instrument_id UUID NOT NULL,
            strategy_name VARCHAR(100) NOT NULL,
            size DECIMAL(20, 10) NOT NULL,
            entry_price DECIMAL(20, 10) NOT NULL,
            entry_time TIMESTAMPTZ NOT NULL,
            stop_loss DECIMAL(20, 10),
            take_profit DECIMAL(20, 10),
            unrealized_pnl DECIMAL(20, 10) NOT NULL DEFAULT 0,
            realized_pnl DECIMAL(20, 10) NOT NULL DEFAULT 0,
            risk_score DECIMAL(5, 4) NOT NULL,
            metadata JSONB
        )
    "#)
    .execute(&database.pool)
    .await?;

    // Create pnl_records table
    info!("🔄 Creating pnl_records table...");
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS pnl_records (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            timestamp TIMESTAMPTZ NOT NULL,
            instrument_id UUID NOT NULL,
            strategy_name VARCHAR(100) NOT NULL,
            realized_pnl DECIMAL(20, 10) NOT NULL,
            unrealized_pnl DECIMAL(20, 10) NOT NULL,
            total_pnl DECIMAL(20, 10) NOT NULL,
            trade_count INTEGER NOT NULL DEFAULT 0,
            win_rate DECIMAL(5, 4) NOT NULL DEFAULT 0,
            sharpe_ratio DECIMAL(8, 6),
            max_drawdown DECIMAL(8, 6),
            metadata JSONB
        )
    "#)
    .execute(&database.pool)
    .await?;

    // Create indexes
    info!("🔄 Creating indexes...");
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_orders_instrument_id ON orders (instrument_id)")
        .execute(&database.pool)
        .await?;
    
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_fills_order_id ON fills (order_id)")
        .execute(&database.pool)
        .await?;
    
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_fills_timestamp ON fills (timestamp DESC)")
        .execute(&database.pool)
        .await?;

    info!("✅ Database tables created successfully");

    // Test table creation
    info!("🔄 Testing table creation...");
    let table_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public' AND table_name IN ('orders', 'fills', 'position_updates', 'pnl_records')"
    )
    .fetch_one(&database.pool)
    .await?;

    info!("📊 Created {} tables successfully", table_count);

    if table_count == 4 {
        info!("🎉 All trading execution logging tables are ready!");
    } else {
        error!("❌ Some tables may not have been created properly");
    }

    Ok(())
}
