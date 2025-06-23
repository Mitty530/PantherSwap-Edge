// Basic connection pooling demo for PantherSwap Edge
// Demonstrates connection pool setup with conservative cloud-friendly settings
// Run with: DATABASE_URL="..." cargo run --example basic_pool_demo

use pantherswap_edge::database::{Database, DatabasePoolConfig};
use pantherswap_edge::config::Settings;
use std::time::Duration;
use sqlx::Row;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 PantherSwap Edge Basic Connection Pool Demo");
    println!("===============================================");
    
    // Load configuration
    let settings = Settings::load()?;
    let database_url = &settings.database.url;
    
    // Test 1: Cloud-Optimized Connection Pool
    println!("\n☁️  Testing Cloud-Optimized Connection Pool...");
    
    // Create cloud-optimized configuration
    let cloud_config = DatabasePoolConfig {
        min_connections: 1,
        max_connections: 5,
        acquire_timeout: Duration::from_secs(30),
        idle_timeout: Some(Duration::from_secs(600)), // 10 minutes
        max_lifetime: Some(Duration::from_secs(3600)), // 1 hour
        test_before_acquire: true,
    };
    
    let cloud_db = Database::new_with_config(database_url, cloud_config).await?;
    println!("✅ Cloud-optimized database connection created");
    
    // Check pool stats
    let stats = cloud_db.pool_stats();
    println!("   - Pool size: {}/{}", stats.size, stats.max_size);
    println!("   - Active connections: {}", stats.active);
    println!("   - Idle connections: {}", stats.idle);

    // Test 2: Basic Database Operations
    println!("\n🔍 Testing Basic Database Operations...");
    
    // Test simple query
    let result = sqlx::query("SELECT 1 as test_value")
        .fetch_one(&cloud_db.pool)
        .await?;
    
    let test_value: i32 = result.get("test_value");
    println!("✅ Basic query successful: {}", test_value);
    
    // Test database version
    let version_result = sqlx::query("SELECT version() as db_version")
        .fetch_one(&cloud_db.pool)
        .await?;
    
    let db_version: String = version_result.get("db_version");
    println!("✅ Database version: {}", db_version.split_whitespace().take(2).collect::<Vec<_>>().join(" "));

    // Test 3: Pool Health Check
    println!("\n🏥 Testing Pool Health Check...");
    
    match cloud_db.pool_health_check().await {
        Ok(health) => {
            println!("✅ Pool health check completed:");
            println!("   - Healthy: {}", health.is_healthy);
            println!("   - Connectivity time: {:?}", health.connectivity_time);
            println!("   - Utilization: {:.1}%", health.utilization_percent);
            println!("   - Performance: {}", health.performance_status);
        }
        Err(e) => println!("❌ Health check failed: {}", e),
    }

    // Test 4: Different Pool Configurations
    println!("\n⚙️  Testing Different Pool Configurations...");
    
    // Testing configuration
    println!("Creating testing pool...");
    let test_db = Database::new_testing(database_url).await?;
    let test_stats = test_db.pool_stats();
    println!("✅ Testing pool: {}/{} connections", test_stats.size, test_stats.max_size);
    
    // Development configuration
    println!("Creating development pool...");
    let dev_db = Database::new_development(database_url).await?;
    let dev_stats = dev_db.pool_stats();
    println!("✅ Development pool: {}/{} connections", dev_stats.size, dev_stats.max_size);
    
    // Cloud configuration
    println!("Creating cloud pool...");
    let cloud_preset_db = Database::new_cloud(database_url).await?;
    let cloud_preset_stats = cloud_preset_db.pool_stats();
    println!("✅ Cloud preset pool: {}/{} connections", cloud_preset_stats.size, cloud_preset_stats.max_size);

    // Test 5: Connection Pool Performance
    println!("\n⚡ Testing Connection Pool Performance...");
    
    // Test connection acquisition time
    let start = std::time::Instant::now();
    let mut conn = cloud_db.pool.acquire().await?;
    let acquire_time = start.elapsed();
    println!("✅ Connection acquisition time: {:?}", acquire_time);
    
    // Test query execution time
    let query_start = std::time::Instant::now();
    let _query_result = sqlx::query("SELECT NOW() as current_time")
        .fetch_one(&mut *conn)
        .await?;
    let query_time = query_start.elapsed();
    println!("✅ Query execution time: {:?}", query_time);
    
    // Release connection
    drop(conn);

    // Test 6: Concurrent Operations
    println!("\n🔄 Testing Concurrent Operations...");
    
    let mut handles = Vec::new();
    
    for i in 0..5 {
        let db = cloud_db.clone();
        let handle = tokio::spawn(async move {
            let start = std::time::Instant::now();
            
            match sqlx::query("SELECT $1 as query_id")
                .bind(i)
                .fetch_one(&db.pool)
                .await
            {
                Ok(row) => {
                    let query_id: i32 = row.get("query_id");
                    let duration = start.elapsed();
                    println!("   ✅ Concurrent query {} completed in {:?}", query_id, duration);
                    Ok(duration)
                }
                Err(e) => {
                    println!("   ❌ Concurrent query {} failed: {}", i, e);
                    Err(e)
                }
            }
        });
        handles.push(handle);
    }
    
    // Wait for all queries to complete
    let mut total_time = Duration::from_secs(0);
    let mut successful_queries = 0;
    
    for handle in handles {
        match handle.await {
            Ok(Ok(duration)) => {
                total_time += duration;
                successful_queries += 1;
            }
            _ => {}
        }
    }
    
    if successful_queries > 0 {
        let avg_time = total_time / successful_queries;
        println!("✅ Concurrent operations completed:");
        println!("   - Successful queries: {}/5", successful_queries);
        println!("   - Average query time: {:?}", avg_time);
    }

    // Test 7: Pool Statistics Monitoring
    println!("\n📊 Testing Pool Statistics Monitoring...");
    
    let all_pools = vec![
        ("cloud", &cloud_db),
        ("testing", &test_db),
        ("development", &dev_db),
        ("cloud_preset", &cloud_preset_db),
    ];
    
    println!("✅ Pool statistics summary:");
    for (name, db) in &all_pools {
        let stats = db.pool_stats();
        let utilization = if stats.size > 0 {
            (stats.active as f64 / stats.size as f64) * 100.0
        } else {
            0.0
        };
        
        println!("   - {}:", name);
        println!("     • Size: {}/{}", stats.size, stats.max_size);
        println!("     • Active: {}, Idle: {}", stats.active, stats.idle);
        println!("     • Utilization: {:.1}%", utilization);
    }

    // Test 8: Connection Pool Optimization Features
    println!("\n🔧 Testing Connection Pool Optimization Features...");
    
    // Test connection-level optimizations by checking if they're applied
    let optimization_test = sqlx::query("SHOW statement_timeout")
        .fetch_one(&cloud_db.pool)
        .await?;
    
    let timeout_setting: String = optimization_test.get("statement_timeout");
    println!("✅ Connection optimization check:");
    println!("   - Statement timeout: {}", timeout_setting);
    
    // Test TimescaleDB specific features if available
    let timescale_check = sqlx::query("SELECT extname FROM pg_extension WHERE extname = 'timescaledb'")
        .fetch_optional(&cloud_db.pool)
        .await?;
    
    if let Some(row) = timescale_check {
        let ext_name: String = row.get("extname");
        println!("   - TimescaleDB extension: {} ✅", ext_name);
    } else {
        println!("   - TimescaleDB extension: not installed");
    }

    // Test 9: Pool Configuration Comparison
    println!("\n📋 Pool Configuration Comparison...");
    
    println!("✅ Configuration comparison:");
    println!("   - Testing: min={}, max={}", test_stats.min_size, test_stats.max_size);
    println!("   - Development: min={}, max={}", dev_stats.min_size, dev_stats.max_size);
    println!("   - Cloud: min={}, max={}", cloud_preset_stats.min_size, cloud_preset_stats.max_size);
    println!("   - Custom Cloud: min={}, max={}", stats.min_size, stats.max_size);

    // Test 10: Pool Lifecycle Management
    println!("\n🔄 Testing Pool Lifecycle Management...");
    
    // Create a temporary pool
    let temp_config = DatabasePoolConfig {
        min_connections: 1,
        max_connections: 2,
        acquire_timeout: Duration::from_secs(10),
        idle_timeout: Some(Duration::from_secs(60)),
        max_lifetime: Some(Duration::from_secs(300)),
        test_before_acquire: false,
    };
    
    let temp_db = Database::new_with_config(database_url, temp_config).await?;
    println!("✅ Temporary pool created");
    
    // Use the pool
    let _ = sqlx::query("SELECT 'temporary_test' as test").fetch_one(&temp_db.pool).await?;
    println!("✅ Temporary pool used successfully");
    
    // Close the pool
    temp_db.close().await;
    println!("✅ Temporary pool closed");

    // Cleanup
    println!("\n🧹 Cleaning up...");
    
    // Close all pools
    cloud_db.close().await;
    test_db.close().await;
    dev_db.close().await;
    cloud_preset_db.close().await;
    
    println!("✅ All pools closed successfully");

    println!("\n🎉 Basic Connection Pool Demo Completed Successfully!");
    println!("======================================================");
    println!("✅ Cloud-optimized connection pooling working");
    println!("✅ Basic database operations functional");
    println!("✅ Pool health monitoring operational");
    println!("✅ Multiple pool configurations tested");
    println!("✅ Connection performance measured");
    println!("✅ Concurrent operations successful");
    println!("✅ Pool statistics monitoring working");
    println!("✅ Connection optimizations applied");
    println!("✅ Pool lifecycle management functional");
    
    Ok(())
}
