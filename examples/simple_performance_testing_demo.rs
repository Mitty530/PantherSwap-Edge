// Simple database performance testing demo for PantherSwap Edge
// Demonstrates basic performance testing capabilities with cloud-friendly settings
// Run with: DATABASE_URL="..." cargo run --example simple_performance_testing_demo

use pantherswap_edge::database::Database;
use pantherswap_edge::config::Settings;
use std::time::{Duration, Instant};
use sqlx::Row;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("⚡ PantherSwap Edge Simple Performance Testing Demo");
    println!("==================================================");
    
    // Load configuration
    let settings = Settings::load()?;
    let database_url = &settings.database.url;
    
    // Test 1: Basic Connection Performance
    println!("\n🔧 Testing Basic Connection Performance...");
    
    let database = Database::new_cloud(database_url).await?;
    println!("✅ Database connection established");
    
    // Test connection acquisition time
    let start = Instant::now();
    let _conn = database.pool.acquire().await?;
    let connection_time = start.elapsed();
    println!("✅ Connection acquisition time: {:?}", connection_time);
    
    // Test basic query performance
    let query_start = Instant::now();
    let result = sqlx::query("SELECT 1 as test_value")
        .fetch_one(&database.pool)
        .await?;
    let query_time = query_start.elapsed();
    let test_value: i32 = result.get("test_value");
    println!("✅ Basic query performance: {:?} (result: {})", query_time, test_value);

    // Test 2: Latency Measurement
    println!("\n📊 Testing Query Latency Distribution...");
    
    let mut latencies = Vec::new();
    let test_queries = 50;
    
    for i in 0..test_queries {
        let query_start = Instant::now();
        let _ = sqlx::query("SELECT $1 as iteration")
            .bind(i as i32)
            .fetch_one(&database.pool)
            .await?;
        latencies.push(query_start.elapsed());
    }
    
    // Calculate statistics
    let total_time: Duration = latencies.iter().sum();
    let avg_latency = total_time / test_queries;
    let min_latency = latencies.iter().min().unwrap();
    let max_latency = latencies.iter().max().unwrap();
    
    // Sort for percentiles
    let mut sorted_latencies = latencies.clone();
    sorted_latencies.sort();
    let p95_index = (test_queries as f64 * 0.95) as usize;
    let p99_index = (test_queries as f64 * 0.99) as usize;
    let default_duration = Duration::from_millis(0);
    let p95_latency = sorted_latencies.get(p95_index).unwrap_or(&default_duration);
    let p99_latency = sorted_latencies.get(p99_index).unwrap_or(&default_duration);
    
    println!("✅ Latency statistics for {} queries:", test_queries);
    println!("   - Average: {:?}", avg_latency);
    println!("   - Minimum: {:?}", min_latency);
    println!("   - Maximum: {:?}", max_latency);
    println!("   - P95: {:?}", p95_latency);
    println!("   - P99: {:?}", p99_latency);
    
    // Latency distribution
    let under_1ms = latencies.iter().filter(|&d| d < &Duration::from_millis(1)).count();
    let under_5ms = latencies.iter().filter(|&d| d < &Duration::from_millis(5)).count();
    let under_10ms = latencies.iter().filter(|&d| d < &Duration::from_millis(10)).count();
    let under_50ms = latencies.iter().filter(|&d| d < &Duration::from_millis(50)).count();
    let over_50ms = test_queries - under_50ms as u32;
    
    println!("📊 Latency distribution:");
    println!("   - Under 1ms: {} ({:.1}%)", under_1ms, (under_1ms as f64 / test_queries as f64) * 100.0);
    println!("   - Under 5ms: {} ({:.1}%)", under_5ms, (under_5ms as f64 / test_queries as f64) * 100.0);
    println!("   - Under 10ms: {} ({:.1}%)", under_10ms, (under_10ms as f64 / test_queries as f64) * 100.0);
    println!("   - Under 50ms: {} ({:.1}%)", under_50ms, (under_50ms as f64 / test_queries as f64) * 100.0);
    println!("   - Over 50ms: {} ({:.1}%)", over_50ms, (over_50ms as f64 / test_queries as f64) * 100.0);

    // Test 3: Throughput Testing
    println!("\n🚀 Testing Query Throughput...");
    
    let throughput_start = Instant::now();
    let throughput_queries = 100;
    let mut successful_queries = 0;
    let mut failed_queries = 0;
    
    for i in 0..throughput_queries {
        match sqlx::query("SELECT NOW() as timestamp, $1 as query_id")
            .bind(i as i32)
            .fetch_one(&database.pool)
            .await
        {
            Ok(_) => successful_queries += 1,
            Err(_) => failed_queries += 1,
        }
    }
    
    let throughput_duration = throughput_start.elapsed();
    let queries_per_second = successful_queries as f64 / throughput_duration.as_secs_f64();
    
    println!("✅ Throughput test results:");
    println!("   - Total queries: {}", throughput_queries);
    println!("   - Successful: {}", successful_queries);
    println!("   - Failed: {}", failed_queries);
    println!("   - Duration: {:?}", throughput_duration);
    println!("   - Queries per second: {:.2}", queries_per_second);
    println!("   - Success rate: {:.1}%", (successful_queries as f64 / throughput_queries as f64) * 100.0);

    // Test 4: Concurrent Performance
    println!("\n🔄 Testing Concurrent Query Performance...");
    
    let concurrent_tasks = 5;
    let queries_per_task = 20;
    let mut handles = Vec::new();
    
    let concurrent_start = Instant::now();
    
    for task_id in 0..concurrent_tasks {
        let pool = database.pool.clone();
        let handle = tokio::spawn(async move {
            let mut task_latencies = Vec::new();
            let mut task_errors = 0;
            
            for i in 0..queries_per_task {
                let query_start = Instant::now();
                match sqlx::query("SELECT $1 as task_id, $2 as iteration, NOW() as timestamp")
                    .bind(task_id as i32)
                    .bind(i as i32)
                    .fetch_one(&pool)
                    .await
                {
                    Ok(_) => {
                        task_latencies.push(query_start.elapsed());
                    }
                    Err(_) => task_errors += 1,
                }
            }
            
            (task_latencies, task_errors)
        });
        handles.push(handle);
    }
    
    // Collect results from all tasks
    let mut all_concurrent_latencies = Vec::new();
    let mut total_concurrent_errors = 0;
    
    for handle in handles {
        match handle.await {
            Ok((latencies, errors)) => {
                all_concurrent_latencies.extend(latencies);
                total_concurrent_errors += errors;
            }
            Err(_) => total_concurrent_errors += queries_per_task,
        }
    }
    
    let concurrent_duration = concurrent_start.elapsed();
    let concurrent_qps = all_concurrent_latencies.len() as f64 / concurrent_duration.as_secs_f64();
    
    println!("✅ Concurrent performance results:");
    println!("   - Concurrent tasks: {}", concurrent_tasks);
    println!("   - Queries per task: {}", queries_per_task);
    println!("   - Total successful queries: {}", all_concurrent_latencies.len());
    println!("   - Total errors: {}", total_concurrent_errors);
    println!("   - Total duration: {:?}", concurrent_duration);
    println!("   - Concurrent QPS: {:.2}", concurrent_qps);
    
    if !all_concurrent_latencies.is_empty() {
        let concurrent_avg = all_concurrent_latencies.iter().sum::<Duration>() / all_concurrent_latencies.len() as u32;
        println!("   - Average latency: {:?}", concurrent_avg);
    }

    // Test 5: Database Information Queries
    println!("\n📋 Testing Database Information Queries...");
    
    // Database size
    let size_start = Instant::now();
    let size_result = sqlx::query("SELECT pg_database_size(current_database()) as db_size")
        .fetch_one(&database.pool)
        .await?;
    let size_time = size_start.elapsed();
    let db_size: i64 = size_result.get("db_size");
    println!("✅ Database size query: {:?} (size: {:.2} MB)", size_time, db_size as f64 / (1024.0 * 1024.0));
    
    // Connection count
    let conn_start = Instant::now();
    let conn_result = sqlx::query("SELECT COUNT(*) as connection_count FROM pg_stat_activity")
        .fetch_one(&database.pool)
        .await?;
    let conn_time = conn_start.elapsed();
    let connection_count: i64 = conn_result.get("connection_count");
    println!("✅ Connection count query: {:?} (connections: {})", conn_time, connection_count);
    
    // Database version
    let version_start = Instant::now();
    let version_result = sqlx::query("SELECT version() as db_version")
        .fetch_one(&database.pool)
        .await?;
    let version_time = version_start.elapsed();
    let db_version: String = version_result.get("db_version");
    let version_short = db_version.split_whitespace().take(2).collect::<Vec<_>>().join(" ");
    println!("✅ Version query: {:?} (version: {})", version_time, version_short);

    // Test 6: Pool Health and Statistics
    println!("\n🏥 Testing Pool Health and Statistics...");
    
    let pool_stats = database.pool_stats();
    println!("✅ Connection pool statistics:");
    println!("   - Pool size: {}/{}", pool_stats.size, pool_stats.max_size);
    println!("   - Active connections: {}", pool_stats.active);
    println!("   - Idle connections: {}", pool_stats.idle);
    println!("   - Utilization: {:.1}%", (pool_stats.active as f64 / pool_stats.size as f64) * 100.0);
    
    let pool_health = database.pool_health_check().await?;
    println!("✅ Pool health check:");
    println!("   - Status: {}", if pool_health.is_healthy { "Healthy ✅" } else { "Unhealthy ❌" });
    println!("   - Connectivity time: {:?}", pool_health.connectivity_time);
    println!("   - Performance rating: {}", pool_health.performance_status);

    // Test 7: Performance Assessment
    println!("\n📊 Performance Assessment...");
    
    // Calculate overall performance metrics
    let overall_avg_latency = if !latencies.is_empty() {
        latencies.iter().sum::<Duration>() / latencies.len() as u32
    } else {
        Duration::from_millis(0)
    };
    
    // Performance classification
    let latency_score = if overall_avg_latency < Duration::from_millis(5) {
        100
    } else if overall_avg_latency < Duration::from_millis(10) {
        80
    } else if overall_avg_latency < Duration::from_millis(50) {
        60
    } else {
        40
    };
    
    let throughput_score = if queries_per_second > 100.0 {
        100
    } else if queries_per_second > 50.0 {
        80
    } else if queries_per_second > 20.0 {
        60
    } else {
        40
    };
    
    let reliability_score = if failed_queries == 0 {
        100
    } else if failed_queries < throughput_queries / 10 {
        80
    } else {
        40
    };
    
    let overall_score = (latency_score + throughput_score + reliability_score) / 3;
    
    println!("✅ Performance assessment:");
    println!("   - Latency score: {}/100", latency_score);
    println!("   - Throughput score: {}/100", throughput_score);
    println!("   - Reliability score: {}/100", reliability_score);
    println!("   - Overall score: {}/100", overall_score);
    
    let performance_grade = if overall_score >= 90 {
        "🚀 EXCELLENT"
    } else if overall_score >= 80 {
        "✅ GOOD"
    } else if overall_score >= 60 {
        "⚠️  ACCEPTABLE"
    } else {
        "❌ NEEDS IMPROVEMENT"
    };
    
    println!("   - Performance grade: {}", performance_grade);

    // Test 8: Recommendations
    println!("\n💡 Performance Recommendations...");
    
    let mut recommendations = Vec::new();
    
    if overall_avg_latency > Duration::from_millis(10) {
        recommendations.push("Consider optimizing query performance or database configuration");
    }
    
    if queries_per_second < 50.0 {
        recommendations.push("Consider increasing connection pool size or optimizing queries");
    }
    
    if failed_queries > 0 {
        recommendations.push("Investigate query failures and improve error handling");
    }
    
    if pool_stats.active as f64 / pool_stats.size as f64 > 0.8 {
        recommendations.push("Consider increasing connection pool size for better performance");
    }
    
    if pool_health.connectivity_time > Duration::from_millis(100) {
        recommendations.push("High connection latency detected - check network and database performance");
    }
    
    if recommendations.is_empty() {
        recommendations.push("Database performance is optimal - no immediate improvements needed");
    }
    
    for (i, recommendation) in recommendations.iter().enumerate() {
        println!("   {}. {}", i + 1, recommendation);
    }

    // Cleanup
    println!("\n🧹 Cleaning up...");
    
    // Close database connection
    database.close().await;
    println!("✅ Database connection closed");

    println!("\n🎉 Simple Performance Testing Demo Completed Successfully!");
    println!("==========================================================");
    println!("✅ Basic connection performance measured");
    println!("✅ Query latency distribution analyzed");
    println!("✅ Throughput testing completed");
    println!("✅ Concurrent performance evaluated");
    println!("✅ Database information queries tested");
    println!("✅ Pool health and statistics verified");
    println!("✅ Performance assessment generated");
    println!("✅ Recommendations provided");
    
    Ok(())
}
