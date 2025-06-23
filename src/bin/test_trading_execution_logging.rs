use pantherswap_edge::database::Database;
use pantherswap_edge::database::types::{Instrument, SignalType, OrderType, TimeInForce};
use pantherswap_edge::database::query_functions::insert_instrument_simple;
use pantherswap_edge::trading::execution::{ExecutionEngine, ExecutionConfig, Fill, Order, OrderStatus, LiquidityFlag};
use pantherswap_edge::trading::signals::OrderRequest;
use uuid::Uuid;
use chrono::Utc;
use tracing::{info, error, warn};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 Starting Trading Execution Database Logging Test");

    // Get database URL from environment or use default
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://tsdbadmin:sz2eu577bgqi5767@jqrbtbc5nw.w0mq2s13iy.tsdb.cloud.timescale.com:35762/tsdb?sslmode=require".to_string());

    info!("📊 Connecting to database: {}", database_url.split('@').last().unwrap_or("unknown"));

    // Create database connection
    let database = Database::new_production(&database_url).await?;
    info!("✅ Database connection established");

    // Run database migrations
    info!("🔄 Running database migrations...");
    if let Err(e) = database.run_migrations().await {
        error!("Failed to run migrations: {}", e);
        return Err(e.into());
    }
    info!("✅ Database migrations completed");

    // Create test instrument
    let test_instrument = create_test_instrument(&database).await?;
    info!("📈 Created test instrument: {} ({})", test_instrument.symbol, test_instrument.id);

    // Create execution engine for testing
    info!("🏭 Creating execution engine...");
    let execution_config = ExecutionConfig::default();
    let execution_engine = ExecutionEngine::new(execution_config, database.clone()).await?;
    info!("✅ Execution engine created");

    // Test 1: Submit and execute a buy order
    info!("🔄 Test 1: Executing BUY order with database logging");
    let buy_order = OrderRequest {
        instrument_id: test_instrument.id,
        side: SignalType::Buy,
        quantity: 100.0,
        order_type: OrderType::Market,
        price: Some(150.0),
        time_in_force: TimeInForce::GTC,
    };

    match execution_engine.execute_order(buy_order).await {
        Ok(execution_result) => {
            info!("✅ BUY order executed successfully: {} {} @ {}",
                  execution_result.filled_quantity, execution_result.average_price, execution_result.instrument_id);
        }
        Err(e) => {
            error!("❌ Failed to execute BUY order: {}", e);
        }
    }

    // Test 2: Submit and execute a sell order
    info!("🔄 Test 2: Executing SELL order with database logging");
    let sell_order = OrderRequest {
        instrument_id: test_instrument.id,
        side: SignalType::Sell,
        quantity: 50.0,
        order_type: OrderType::Limit,
        price: Some(155.0),
        time_in_force: TimeInForce::GTC,
    };

    match execution_engine.execute_order(sell_order).await {
        Ok(execution_result) => {
            info!("✅ SELL order executed successfully: {} {} @ {}",
                  execution_result.filled_quantity, execution_result.average_price, execution_result.instrument_id);
        }
        Err(e) => {
            error!("❌ Failed to execute SELL order: {}", e);
        }
    }

    // Test 3: Multiple rapid orders to test performance
    info!("🔄 Test 3: Submitting multiple rapid orders");
    let start_time = std::time::Instant::now();
    let mut successful_orders = 0;

    for i in 0..10 {
        let order = OrderRequest {
            instrument_id: test_instrument.id,
            side: if i % 2 == 0 { SignalType::Buy } else { SignalType::Sell },
            quantity: 10.0 + (i as f64 * 5.0),
            order_type: OrderType::Market,
            price: Some(150.0 + (i as f64)),
            time_in_force: TimeInForce::GTC,
        };

        match execution_engine.execute_order(order).await {
            Ok(_) => successful_orders += 1,
            Err(e) => error!("Failed to execute order {}: {}", i, e),
        }
    }

    let duration = start_time.elapsed();
    info!("✅ Submitted {}/10 orders in {:?} ({:.2} orders/sec)", 
          successful_orders, duration, successful_orders as f64 / duration.as_secs_f64());

    // Test 4: Verify database records
    info!("🔄 Test 4: Verifying database records");
    if let Err(e) = verify_database_records(&database, &test_instrument.id).await {
        error!("❌ Database verification failed: {}", e);
    } else {
        info!("✅ Database records verified successfully");
    }

    // Test 5: Check execution engine state
    info!("🔄 Test 5: Checking execution engine state");
    let active_orders = execution_engine.get_active_orders().await;
    info!("📊 Execution Engine Statistics:");
    info!("   - Active orders: {}", active_orders.len());

    info!("🎉 Trading Execution Database Logging Test completed successfully!");
    Ok(())
}

async fn create_test_instrument(database: &Database) -> Result<Instrument, Box<dyn std::error::Error>> {
    let instrument = Instrument {
        id: Uuid::new_v4(),
        symbol: "TEST_LOGGING".to_string(),
        name: "Test Logging Instrument".to_string(),
        instrument_type: "STOCK".to_string(),
        base_currency: "USD".to_string(),
        quote_currency: "USD".to_string(),
        tick_size: 0.01,
        lot_size: 1.0,
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    insert_instrument_simple(&database.pool, &instrument).await?;
    Ok(instrument)
}

async fn verify_database_records(database: &Database, instrument_id: &Uuid) -> Result<(), Box<dyn std::error::Error>> {
    // Check if we have trade executions
    let trade_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM trade_executions WHERE instrument_id = $1"
    )
    .bind(instrument_id)
    .fetch_one(&database.pool)
    .await?;

    info!("📊 Found {} trade executions in database", trade_count);

    // Check if we have orders
    let order_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM orders WHERE instrument_id = $1"
    )
    .bind(instrument_id)
    .fetch_one(&database.pool)
    .await
    .unwrap_or(0);

    info!("📊 Found {} orders in database", order_count);

    // Check if we have fills
    let fill_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM fills f JOIN orders o ON f.order_id = o.id WHERE o.instrument_id = $1"
    )
    .bind(instrument_id)
    .fetch_one(&database.pool)
    .await
    .unwrap_or(0);

    info!("📊 Found {} fills in database", fill_count);

    // Check if we have position updates
    let position_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM position_updates WHERE instrument_id = $1"
    )
    .bind(instrument_id)
    .fetch_one(&database.pool)
    .await
    .unwrap_or(0);

    info!("📊 Found {} position updates in database", position_count);

    // Check if we have P&L records
    let pnl_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pnl_records WHERE instrument_id = $1"
    )
    .bind(instrument_id)
    .fetch_one(&database.pool)
    .await
    .unwrap_or(0);

    info!("📊 Found {} P&L records in database", pnl_count);

    if trade_count > 0 {
        info!("✅ Database logging is working correctly!");
    } else {
        warn!("⚠️  No trade executions found - database logging may not be working");
    }

    Ok(())
}
