#!/usr/bin/env rust-script
//! Simple Database Storage Test for PantherSwap Edge
//! Tests basic database operations without complex constraints

use std::env;
use std::time::Instant;
use sqlx::PgPool;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();
    
    println!("💾 PantherSwap Edge Simple Storage Test");
    println!("=======================================");

    let database_url = env::var("PANTHERSWAP_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://tsdbadmin:sz2eu577bgqi5767@jqrbtbc5nw.w0mq2s13iy.tsdb.cloud.timescale.com:35762/tsdb?sslmode=require".to_string());

    let pool = PgPool::connect(&database_url).await?;
    
    // Test 1: Create simple test table
    println!("\n📊 Creating test table...");
    let create_result = create_test_table(&pool).await;
    match create_result {
        Ok(_) => println!("✅ Test table created successfully"),
        Err(e) => println!("❌ Test table creation failed: {}", e),
    }

    // Test 2: Insert test data
    println!("\n📝 Inserting test data...");
    let insert_result = insert_test_data(&pool).await;
    match insert_result {
        Ok(latency) => println!("✅ Data inserted successfully ({:.2}ms average)", latency),
        Err(e) => println!("❌ Data insertion failed: {}", e),
    }

    // Test 3: Query test data
    println!("\n🔍 Querying test data...");
    let query_result = query_test_data(&pool).await;
    match query_result {
        Ok(count) => println!("✅ Data queried successfully ({} records)", count),
        Err(e) => println!("❌ Data query failed: {}", e),
    }

    // Test 4: Performance test
    println!("\n⚡ Running performance test...");
    let perf_result = performance_test(&pool).await;
    match perf_result {
        Ok((insert_tps, query_latency)) => {
            println!("✅ Performance test completed:");
            println!("   Insert TPS: {:.2}", insert_tps);
            println!("   Query Latency: {:.2}ms", query_latency);
        }
        Err(e) => println!("❌ Performance test failed: {}", e),
    }

    // Test 5: TimescaleDB features
    println!("\n⏰ Testing TimescaleDB features...");
    let timescale_result = test_timescale_features(&pool).await;
    match timescale_result {
        Ok(_) => println!("✅ TimescaleDB features working"),
        Err(e) => println!("❌ TimescaleDB features failed: {}", e),
    }

    println!("\n🎯 Simple Storage Test Complete");
    Ok(())
}

async fn create_test_table(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    // Drop table if exists
    sqlx::query("DROP TABLE IF EXISTS test_market_data CASCADE")
        .execute(pool)
        .await?;

    // Create table
    sqlx::query(
        r#"
        CREATE TABLE test_market_data (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            timestamp TIMESTAMPTZ NOT NULL,
            symbol VARCHAR(20) NOT NULL,
            price DECIMAL(20,8) NOT NULL,
            volume DECIMAL(20,8) NOT NULL,
            provider VARCHAR(50) NOT NULL,
            data_quality DECIMAL(5,4) NOT NULL,
            metadata JSONB,
            created_at TIMESTAMPTZ DEFAULT NOW()
        )
        "#
    )
    .execute(pool)
    .await?;

    // Try to create hypertable (may fail if not TimescaleDB, but that's OK)
    let _ = sqlx::query(
        "SELECT create_hypertable('test_market_data', 'timestamp', if_not_exists => TRUE)"
    )
    .execute(pool)
    .await;

    Ok(())
}

async fn insert_test_data(pool: &PgPool) -> Result<f64, Box<dyn std::error::Error>> {
    let symbols = vec!["AAPL", "MSFT", "GOOGL", "TSLA", "NVDA"];
    let mut total_latency = 0.0;
    let count = symbols.len();

    for symbol in symbols {
        let start = Instant::now();
        
        sqlx::query(
            r#"
            INSERT INTO test_market_data (timestamp, symbol, price, volume, provider, data_quality, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#
        )
        .bind(Utc::now())
        .bind(symbol)
        .bind(150.50 + (symbol.len() as f64 * 10.0))
        .bind(100000.0)
        .bind("test_provider")
        .bind(0.95)
        .bind(serde_json::json!({"test": true, "symbol": symbol}))
        .execute(pool)
        .await?;
        
        total_latency += start.elapsed().as_millis() as f64;
    }

    Ok(total_latency / count as f64)
}

async fn query_test_data(pool: &PgPool) -> Result<i64, Box<dyn std::error::Error>> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM test_market_data WHERE timestamp > NOW() - INTERVAL '1 hour'"
    )
    .fetch_one(pool)
    .await?;

    Ok(count)
}

async fn performance_test(pool: &PgPool) -> Result<(f64, f64), Box<dyn std::error::Error>> {
    // Insert performance test
    let insert_start = Instant::now();
    let batch_size = 100;
    
    for i in 0..batch_size {
        sqlx::query(
            r#"
            INSERT INTO test_market_data (timestamp, symbol, price, volume, provider, data_quality, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#
        )
        .bind(Utc::now())
        .bind(format!("PERF{}", i))
        .bind(100.0 + i as f64)
        .bind(50000.0)
        .bind("performance_test")
        .bind(0.98)
        .bind(serde_json::json!({"batch": i}))
        .execute(pool)
        .await?;
    }
    
    let insert_duration = insert_start.elapsed().as_secs_f64();
    let insert_tps = batch_size as f64 / insert_duration;

    // Query performance test
    let query_start = Instant::now();
    let _results: Vec<(String, rust_decimal::Decimal)> = sqlx::query_as(
        "SELECT symbol, price FROM test_market_data ORDER BY timestamp DESC LIMIT 50"
    )
    .fetch_all(pool)
    .await?;
    
    let query_latency = query_start.elapsed().as_millis() as f64;

    Ok((insert_tps, query_latency))
}

async fn test_timescale_features(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    // Test time-bucket aggregation (TimescaleDB specific)
    let _result: Option<(chrono::DateTime<Utc>, rust_decimal::Decimal)> = sqlx::query_as(
        r#"
        SELECT
            time_bucket('1 minute', timestamp) as bucket,
            AVG(price) as avg_price
        FROM test_market_data
        WHERE timestamp > NOW() - INTERVAL '1 hour'
        GROUP BY bucket
        ORDER BY bucket DESC
        LIMIT 1
        "#
    )
    .fetch_optional(pool)
    .await?;

    // Test data retention (if configured)
    let retention_info: Option<String> = sqlx::query_scalar(
        "SELECT retention_policy FROM timescaledb_information.jobs WHERE hypertable_name = 'test_market_data' LIMIT 1"
    )
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    if retention_info.is_some() {
        println!("   Retention policy: {:?}", retention_info);
    }

    Ok(())
}
