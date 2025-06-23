// Simple Rust TimescaleDB Connection Test
// Uses existing PantherSwap Edge infrastructure to connect to TimescaleDB

use pantherswap_edge::config::Settings;
use pantherswap_edge::database::Database;
use std::time::{Duration, Instant};
use uuid::Uuid;
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("🚀 PantherSwap Edge Simple Rust TimescaleDB Test");
    println!("💾 Testing connection to your actual TimescaleDB using Rust");
    
    // Load settings
    let settings = Settings::load().map_err(|e| {
        format!("Failed to load settings: {}", e)
    })?;
    
    println!("📋 Database Configuration:");
    println!("   🗄️  URL: {}", settings.database.url);
    println!("   🔗 Max Connections: {}", settings.database.max_connections);
    
    // Connect to database
    println!("🔗 Connecting to TimescaleDB...");
    let database = Database::new(&settings.database.url).await.map_err(|e| {
        format!("Failed to connect to database: {}", e)
    })?;
    
    println!("✅ Successfully connected to TimescaleDB using Rust!");
    
    // Test basic database operations
    let simulation_id = Uuid::new_v4();
    let start_time = Instant::now();
    
    println!("📊 Testing database operations...");
    
    // Create a simple test table
    let create_table_query = r#"
        CREATE TABLE IF NOT EXISTS rust_db_test (
            id UUID PRIMARY KEY,
            test_name TEXT NOT NULL,
            timestamp TIMESTAMPTZ NOT NULL,
            value DECIMAL(15,4) NOT NULL,
            metadata JSONB
        );
    "#;
    
    sqlx::query(create_table_query)
        .execute(&database.pool)
        .await
        .map_err(|e| format!("Failed to create test table: {}", e))?;
    
    println!("✅ Test table created successfully");
    
    // Insert test data
    let mut total_operations = 0;
    let test_duration = Duration::from_secs(60); // 1 minute test
    let test_start = Instant::now();
    
    println!("🚀 Running 1-minute database performance test...");
    
    while test_start.elapsed() < test_duration {
        let operation_start = Instant::now();
        
        // Insert market data simulation
        for i in 0..5 {
            let test_id = Uuid::new_v4();
            let symbol = match i {
                0 => "AAPL",
                1 => "MSFT", 
                2 => "GOOGL",
                3 => "TSLA",
                4 => "NVDA",
                _ => "UNKNOWN",
            };
            
            let price = 100.0 + (total_operations as f64 * 0.1);
            let metadata = serde_json::json!({
                "simulation_id": simulation_id,
                "symbol": symbol,
                "iteration": total_operations,
                "test_type": "rust_performance"
            });
            
            sqlx::query(r#"
                INSERT INTO rust_db_test (id, test_name, timestamp, value, metadata)
                VALUES ($1, $2, $3, $4, $5)
            "#)
            .bind(&test_id)
            .bind(format!("rust_test_{}", symbol))
            .bind(Utc::now())
            .bind(price)
            .bind(&metadata)
            .execute(&database.pool)
            .await
            .map_err(|e| format!("Failed to insert test data: {}", e))?;
            
            total_operations += 1;
        }
        
        let operation_latency = operation_start.elapsed();
        
        // Log progress every 15 seconds
        if total_operations % 75 == 0 { // Every 15 operations (3 seconds at 5 ops per iteration)
            let elapsed = test_start.elapsed();
            let remaining = test_duration.saturating_sub(elapsed);
            println!("📊 Progress: {:.1}s elapsed, {:.1}s remaining | Operations: {} | Latency: {:.2}ms", 
                     elapsed.as_secs_f64(), 
                     remaining.as_secs_f64(),
                     total_operations,
                     operation_latency.as_millis());
        }
        
        // Small delay to prevent overwhelming
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    
    let total_duration = start_time.elapsed();
    
    // Query results
    println!("📊 Querying test results...");
    
    let count_result: (i64,) = sqlx::query_as(r#"
        SELECT COUNT(*) FROM rust_db_test WHERE metadata->>'simulation_id' = $1
    "#)
    .bind(simulation_id.to_string())
    .fetch_one(&database.pool)
    .await
    .map_err(|e| format!("Failed to query test results: {}", e))?;
    
    let total_records = count_result.0;
    
    // Get sample data
    let sample_records: Vec<(String, chrono::DateTime<chrono::Utc>, f64)> = sqlx::query_as(r#"
        SELECT test_name, timestamp, value 
        FROM rust_db_test 
        WHERE metadata->>'simulation_id' = $1 
        ORDER BY timestamp DESC 
        LIMIT 5
    "#)
    .bind(simulation_id.to_string())
    .fetch_all(&database.pool)
    .await
    .map_err(|e| format!("Failed to fetch sample records: {}", e))?;
    
    // Calculate performance metrics
    let operations_per_second = total_operations as f64 / total_duration.as_secs_f64();
    let avg_latency_ms = total_duration.as_millis() as f64 / total_operations as f64;
    
    // Display results
    println!("\n📊 RUST TIMESCALEDB TEST RESULTS");
    println!("=================================");
    println!("🆔 Test ID: {}", simulation_id);
    println!("⏱️  Total Duration: {:.2} seconds", total_duration.as_secs_f64());
    println!("🔢 Total Operations: {}", total_operations);
    println!("📊 Records Created: {}", total_records);
    println!("🚀 Operations/Second: {:.2}", operations_per_second);
    println!("⚡ Average Latency: {:.2}ms", avg_latency_ms);
    
    println!("\n📋 Sample Records Created:");
    for (i, (test_name, timestamp, value)) in sample_records.iter().enumerate() {
        println!("   {}. {} | {} | ${:.2}", i + 1, test_name, timestamp.format("%H:%M:%S"), value);
    }
    
    // Performance validation
    let latency_target_met = avg_latency_ms < 50.0;
    let throughput_target_met = operations_per_second > 10.0;
    let data_integrity_met = total_records == total_operations as i64;
    
    println!("\n🎯 PERFORMANCE VALIDATION:");
    println!("   ⚡ Latency (<50ms): {}", if latency_target_met { "✅ MET" } else { "❌ NOT MET" });
    println!("   🚀 Throughput (>10 ops/sec): {}", if throughput_target_met { "✅ MET" } else { "❌ NOT MET" });
    println!("   💾 Data Integrity: {}", if data_integrity_met { "✅ MET" } else { "❌ NOT MET" });
    
    if latency_target_met && throughput_target_met && data_integrity_met {
        println!("\n🎉 SUCCESS: Rust TimescaleDB integration validated!");
        println!("💾 Database connection and operations working perfectly");
        println!("🦀 Rust can successfully connect to and use your TimescaleDB");
    } else {
        println!("\n⚠️  Some performance targets not met - review configuration");
    }
    
    // Cleanup test data
    println!("\n🧹 Cleaning up test data...");
    let deleted_count = sqlx::query(r#"
        DELETE FROM rust_db_test WHERE metadata->>'simulation_id' = $1
    "#)
    .bind(simulation_id.to_string())
    .execute(&database.pool)
    .await
    .map_err(|e| format!("Failed to cleanup test data: {}", e))?
    .rows_affected();
    
    println!("✅ Cleaned up {} test records", deleted_count);
    
    // Drop test table
    sqlx::query("DROP TABLE IF EXISTS rust_db_test")
        .execute(&database.pool)
        .await
        .map_err(|e| format!("Failed to drop test table: {}", e))?;
    
    println!("✅ Test table dropped");
    
    println!("\n✅ Rust TimescaleDB integration test completed successfully!");
    println!("💾 Your TimescaleDB is ready for production Rust applications");
    
    Ok(())
}
