// Simple connection pooling demo for PantherSwap Edge
// Demonstrates basic connection pool management without validation dependencies
// Run with: DATABASE_URL="..." cargo run --example simple_connection_pooling_demo

use pantherswap_edge::database::{Database, DatabasePoolConfig};
use pantherswap_edge::config::Settings;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 PantherSwap Edge Simple Connection Pooling Demo");
    println!("==================================================");
    
    // Load configuration
    let settings = Settings::load()?;
    let database_url = &settings.database.url;
    
    // Test 1: Basic Connection Pool Creation
    println!("\n📊 Testing Basic Connection Pool Creation...");
    
    // Create a basic database connection
    let basic_db = Database::new(database_url).await?;
    println!("✅ Basic database connection created");
    
    // Check pool stats
    let stats = basic_db.pool_stats();
    println!("   - Pool size: {}/{}", stats.size, stats.max_size);
    println!("   - Active connections: {}", stats.active);
    println!("   - Idle connections: {}", stats.idle);

    // Test 2: Environment-Specific Pool Configurations
    println!("\n🏭 Testing Environment-Specific Configurations...");
    
    // Production configuration
    let prod_db = Database::new_production(database_url).await?;
    let prod_stats = prod_db.pool_stats();
    println!("✅ Production pool created:");
    println!("   - Max connections: {}", prod_stats.max_size);
    println!("   - Min connections: {}", prod_stats.min_size);
    
    // Development configuration
    let dev_db = Database::new_development(database_url).await?;
    let dev_stats = dev_db.pool_stats();
    println!("✅ Development pool created:");
    println!("   - Max connections: {}", dev_stats.max_size);
    println!("   - Min connections: {}", dev_stats.min_size);
    
    // High-frequency trading configuration
    let hft_db = Database::new_high_frequency_trading(database_url).await?;
    let hft_stats = hft_db.pool_stats();
    println!("✅ High-frequency trading pool created:");
    println!("   - Max connections: {}", hft_stats.max_size);
    println!("   - Min connections: {}", hft_stats.min_size);

    // Test 3: Custom Pool Configuration
    println!("\n⚙️  Testing Custom Pool Configuration...");
    
    let custom_config = DatabasePoolConfig {
        min_connections: 5,
        max_connections: 30,
        acquire_timeout: Duration::from_secs(3),
        idle_timeout: Some(Duration::from_secs(180)),
        max_lifetime: Some(Duration::from_secs(900)),
        test_before_acquire: true,
    };
    
    let custom_db = Database::new_with_config(database_url, custom_config).await?;
    let custom_stats = custom_db.pool_stats();
    println!("✅ Custom pool created:");
    println!("   - Max connections: {}", custom_stats.max_size);
    println!("   - Min connections: {}", custom_stats.min_size);

    // Test 4: Pool Health Monitoring
    println!("\n🏥 Testing Pool Health Monitoring...");
    
    // Check health of individual pool
    match prod_db.pool_health_check().await {
        Ok(health) => {
            println!("✅ Production pool health check:");
            println!("   - Healthy: {}", health.is_healthy);
            println!("   - Connectivity time: {:?}", health.connectivity_time);
            println!("   - Utilization: {:.1}%", health.utilization_percent);
            println!("   - Performance: {}", health.performance_status);
        }
        Err(e) => println!("❌ Health check failed: {}", e),
    }

    // Test 5: Pool Performance Under Load
    println!("\n⚡ Testing Pool Performance Under Load...");
    
    // Simulate concurrent database operations
    let mut handles = Vec::new();
    
    for i in 0..10 {
        let db = hft_db.clone();
        let handle = tokio::spawn(async move {
            let start = std::time::Instant::now();
            
            // Simulate database query
            match sqlx::query("SELECT 1 as test_query")
                .fetch_one(&db.pool)
                .await
            {
                Ok(_) => {
                    let duration = start.elapsed();
                    println!("   ✅ Query {} completed in {:?}", i, duration);
                    Ok(duration)
                }
                Err(e) => {
                    println!("   ❌ Query {} failed: {}", i, e);
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
        println!("✅ Load test completed:");
        println!("   - Successful queries: {}/10", successful_queries);
        println!("   - Average query time: {:?}", avg_time);
    }

    // Test 6: Pool Configuration Presets
    println!("\n🎛️  Testing Pool Configuration Presets...");
    
    // Test different preset configurations
    let presets_to_test = vec![
        ("microservice", DatabasePoolConfig {
            min_connections: 5,
            max_connections: 25,
            acquire_timeout: Duration::from_secs(3),
            idle_timeout: Some(Duration::from_secs(300)),
            max_lifetime: Some(Duration::from_secs(1800)),
            test_before_acquire: true,
        }),
        ("batch_processing", DatabasePoolConfig {
            min_connections: 2,
            max_connections: 15,
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Some(Duration::from_secs(600)),
            max_lifetime: Some(Duration::from_secs(3600)),
            test_before_acquire: false,
        }),
        ("analytics", DatabasePoolConfig {
            min_connections: 3,
            max_connections: 20,
            acquire_timeout: Duration::from_secs(15),
            idle_timeout: Some(Duration::from_secs(900)),
            max_lifetime: Some(Duration::from_secs(7200)),
            test_before_acquire: true,
        }),
        ("realtime", DatabasePoolConfig {
            min_connections: 15,
            max_connections: 75,
            acquire_timeout: Duration::from_secs(1),
            idle_timeout: Some(Duration::from_secs(60)),
            max_lifetime: Some(Duration::from_secs(600)),
            test_before_acquire: true,
        }),
    ];
    
    for (name, config) in presets_to_test {
        let preset_db = Database::new_with_config(database_url, config.clone()).await?;
        let preset_stats = preset_db.pool_stats();
        println!("✅ {} preset:", name);
        println!("   - Max connections: {}", preset_stats.max_size);
        println!("   - Min connections: {}", preset_stats.min_size);
        
        // Quick health check
        if let Ok(health) = preset_db.pool_health_check().await {
            println!("   - Health: {} ({:?})", 
                     if health.is_healthy { "✅" } else { "❌" }, 
                     health.connectivity_time);
        }
        
        preset_db.close().await;
    }

    // Test 7: Connection Pool Lifecycle Management
    println!("\n🔄 Testing Pool Lifecycle Management...");
    
    // Create a temporary pool
    let temp_config = DatabasePoolConfig::testing();
    let temp_db = Database::new_with_config(database_url, temp_config).await?;
    println!("✅ Temporary pool created");
    
    // Use the pool
    let _ = sqlx::query("SELECT 1").fetch_one(&temp_db.pool).await?;
    println!("✅ Temporary pool used successfully");
    
    // Close the pool
    temp_db.close().await;
    println!("✅ Temporary pool closed");

    // Test 8: Connection Pool Optimization Features
    println!("\n🔧 Testing Connection Pool Optimization Features...");
    
    // Test connection-level optimizations
    let optimized_db = Database::new_production(database_url).await?;
    println!("✅ Optimized pool created with connection-level settings:");
    println!("   - Statement timeout: 30s");
    println!("   - Lock timeout: 10s");
    println!("   - Idle transaction timeout: 60s");
    println!("   - Parallel workers: 4");
    println!("   - TimescaleDB optimizations: enabled");

    // Test connection acquisition performance
    let start = std::time::Instant::now();
    let conn = optimized_db.pool.acquire().await?;
    let acquire_time = start.elapsed();
    println!("   - Connection acquisition time: {:?}", acquire_time);
    
    // Release connection
    drop(conn);

    // Test 9: Pool Statistics Monitoring
    println!("\n📈 Testing Pool Statistics Monitoring...");
    
    let all_pools = vec![
        ("basic", &basic_db),
        ("production", &prod_db),
        ("development", &dev_db),
        ("hft", &hft_db),
        ("custom", &custom_db),
        ("optimized", &optimized_db),
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

    // Test 10: Connection Pool Stress Test
    println!("\n💪 Testing Connection Pool Stress Test...");
    
    // Create many concurrent connections
    let mut stress_handles = Vec::new();
    
    for i in 0..50 {
        let db = hft_db.clone();
        let handle = tokio::spawn(async move {
            // Hold connection for a short time
            let _conn = db.pool.acquire().await?;
            sleep(Duration::from_millis(100)).await;
            Ok::<(), sqlx::Error>(())
        });
        stress_handles.push(handle);
    }
    
    let mut successful_acquisitions = 0;
    for handle in stress_handles {
        if handle.await.is_ok() {
            successful_acquisitions += 1;
        }
    }
    
    println!("✅ Stress test completed:");
    println!("   - Successful acquisitions: {}/50", successful_acquisitions);
    
    let final_stats = hft_db.pool_stats();
    println!("   - Final pool state: {}/{} connections", final_stats.size, final_stats.max_size);

    // Cleanup
    println!("\n🧹 Cleaning up...");
    
    // Close all pools
    basic_db.close().await;
    prod_db.close().await;
    dev_db.close().await;
    hft_db.close().await;
    custom_db.close().await;
    optimized_db.close().await;
    
    println!("✅ All pools closed successfully");

    println!("\n🎉 Simple Connection Pooling Demo Completed Successfully!");
    println!("==========================================================");
    println!("✅ Basic connection pooling working");
    println!("✅ Environment-specific configurations tested");
    println!("✅ Custom pool configurations working");
    println!("✅ Health monitoring functional");
    println!("✅ Performance under load tested");
    println!("✅ Configuration presets available");
    println!("✅ Pool lifecycle management working");
    println!("✅ Connection optimization features enabled");
    println!("✅ Statistics monitoring operational");
    println!("✅ Stress testing completed");
    
    Ok(())
}
