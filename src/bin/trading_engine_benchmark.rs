// Comprehensive Trading Engine Performance Benchmark
// Tests async processing, adaptive batching, and lock-free optimizations

use anyhow::Result;
use pantherswap_edge::config::Settings;
use pantherswap_edge::database::Database;
use pantherswap_edge::trading::engine::{TradingEngine, create_optimized_trading_engine};
use pantherswap_edge::trading::signals::{OrderRequest, SignalType};
use pantherswap_edge::utils::performance_profiler::{PerformanceProfiler, ProfilerConfig};
use chrono::Utc;
use std::sync::Arc;
use std::time::{Instant, Duration};
use tokio::time::sleep;
use tracing::{info, warn, error};
use uuid::Uuid;
use rand::Rng;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    setup_logging()?;
    
    info!("🚀 Starting PantherSwap Edge Trading Engine Performance Benchmark");
    
    // Load configuration
    dotenvy::dotenv().ok();
    let settings = Settings::load()?;
    
    // Initialize database
    let database = Database::new(&settings.database.url).await?;
    info!("✅ Database connected");
    
    // Initialize performance profiler
    let profiler_config = ProfilerConfig {
        sampling_interval_ms: 25, // High frequency sampling
        baseline_duration_minutes: 1, // Short baseline for testing
        latency_sample_size: 1000,
        enable_detailed_profiling: true,
        enable_memory_profiling: true,
        enable_cpu_profiling: true,
        target_order_latency_ms: 10.0,
        target_ai_latency_ms: 100.0,
        target_throughput_tps: 2000.0, // High target
    };
    
    let profiler = PerformanceProfiler::new(profiler_config, database.clone()).await?;
    info!("✅ Performance profiler initialized");
    
    // Run comprehensive benchmark suite
    run_benchmark_suite(&profiler, &database).await?;
    
    info!("🎯 Trading Engine Performance Benchmark Complete");
    Ok(())
}

async fn run_benchmark_suite(profiler: &PerformanceProfiler, database: &Database) -> Result<()> {
    info!("📊 Starting comprehensive benchmark suite...");
    
    // 1. Baseline Performance Test
    info!("🔍 Running baseline performance test...");
    run_baseline_performance_test(profiler, database).await?;
    
    // 2. Async Processing Benchmark
    info!("⚡ Running async processing benchmark...");
    run_async_processing_benchmark(profiler, database).await?;
    
    // 3. Adaptive Batching Benchmark
    info!("📦 Running adaptive batching benchmark...");
    run_adaptive_batching_benchmark(profiler, database).await?;
    
    // 4. Lock-Free Structures Benchmark
    info!("🔓 Running lock-free structures benchmark...");
    run_lock_free_benchmark(profiler, database).await?;
    
    // 5. High-Throughput Stress Test
    info!("🔥 Running high-throughput stress test...");
    run_stress_test(profiler, database).await?;
    
    // 6. Latency Validation Test
    info!("⏱️ Running latency validation test...");
    run_latency_validation_test(profiler, database).await?;
    
    // Generate final report
    generate_benchmark_report(profiler).await?;
    
    Ok(())
}

async fn run_baseline_performance_test(profiler: &PerformanceProfiler, database: &Database) -> Result<()> {
    info!("  Testing baseline trading engine performance...");
    
    // Create standard trading engine
    let engine = TradingEngine::new(
        pantherswap_edge::trading::engine::TradingEngineConfig::default(),
        database.clone()
    ).await?;
    
    let test_orders = generate_test_orders(1000);
    let start_time = Instant::now();
    
    for order in test_orders {
        let order_start = Instant::now();
        
        if let Err(e) = engine.submit_order(order).await {
            error!("Baseline order failed: {}", e);
        }
        
        let latency_ms = order_start.elapsed().as_micros() as f64 / 1000.0;
        profiler.record_order_latency(latency_ms).await;
    }
    
    let total_time = start_time.elapsed();
    let throughput = 1000.0 / total_time.as_secs_f64();
    
    info!("  ✅ Baseline: {:.2} TPS, {:.2}s total", throughput, total_time.as_secs_f64());
    Ok(())
}

async fn run_async_processing_benchmark(profiler: &PerformanceProfiler, database: &Database) -> Result<()> {
    info!("  Testing async processing optimizations...");
    
    // Create optimized trading engine
    let engine = create_optimized_trading_engine(database.clone()).await?;
    
    // Start async processing pipeline
    engine.start_async_order_pipeline().await?;
    
    let test_orders = generate_test_orders(2000);
    let start_time = Instant::now();
    
    // Submit orders concurrently
    let mut handles = Vec::new();
    for order in test_orders {
        let engine_clone = engine.clone();
        let profiler_clone = profiler.clone();
        
        let handle = tokio::spawn(async move {
            let order_start = Instant::now();
            
            let result = engine_clone.process_order_optimized(order).await;
            
            let latency_ms = order_start.elapsed().as_micros() as f64 / 1000.0;
            profiler_clone.record_order_latency(latency_ms).await;
            
            result
        });
        
        handles.push(handle);
    }
    
    // Wait for all orders to complete
    let mut successful = 0;
    for handle in handles {
        match handle.await {
            Ok(Ok(_)) => successful += 1,
            Ok(Err(e)) => error!("Async order failed: {}", e),
            Err(e) => error!("Task join failed: {}", e),
        }
    }
    
    let total_time = start_time.elapsed();
    let throughput = successful as f64 / total_time.as_secs_f64();
    
    info!("  ✅ Async Processing: {:.2} TPS, {}/{} success", throughput, successful, 2000);
    Ok(())
}

async fn run_adaptive_batching_benchmark(profiler: &PerformanceProfiler, database: &Database) -> Result<()> {
    info!("  Testing adaptive batching system...");
    
    let engine = create_optimized_trading_engine(database.clone()).await?;
    let test_orders = generate_test_orders(3000);
    let start_time = Instant::now();
    
    // Submit orders through adaptive batching
    for order in test_orders {
        let order_start = Instant::now();
        
        if let Err(e) = engine.submit_order_adaptive(order).await {
            error!("Adaptive batch order failed: {}", e);
        }
        
        let latency_ms = order_start.elapsed().as_micros() as f64 / 1000.0;
        profiler.record_order_latency(latency_ms).await;
    }
    
    let total_time = start_time.elapsed();
    let throughput = 3000.0 / total_time.as_secs_f64();
    
    // Get adaptive batching stats
    if let Some(stats) = engine.get_adaptive_batching_stats().await {
        info!("  📊 Adaptive Batching Stats:");
        info!("    Current batch size: {}", stats.current_batch_size);
        info!("    Average batch size: {}", stats.average_batch_size);
        info!("    Average latency: {:.2}ms", stats.average_latency_ms);
        info!("    Current load factor: {:.2}", stats.current_load_factor);
    }
    
    info!("  ✅ Adaptive Batching: {:.2} TPS", throughput);
    Ok(())
}

async fn run_lock_free_benchmark(profiler: &PerformanceProfiler, database: &Database) -> Result<()> {
    info!("  Testing lock-free data structures...");
    
    let engine = create_optimized_trading_engine(database.clone()).await?;
    let test_orders = generate_test_orders(5000);
    let start_time = Instant::now();
    
    // Submit orders through lock-free queue
    for order in test_orders {
        let order_start = Instant::now();
        
        if let Err(e) = engine.submit_order_lock_free(order).await {
            error!("Lock-free order failed: {}", e);
        }
        
        let latency_ms = order_start.elapsed().as_micros() as f64 / 1000.0;
        profiler.record_order_latency(latency_ms).await;
    }
    
    let total_time = start_time.elapsed();
    let throughput = 5000.0 / total_time.as_secs_f64();
    
    // Get lock-free stats
    if let Some((queue_depth, queue_capacity)) = engine.get_lock_free_queue_stats() {
        info!("  📊 Lock-Free Queue: {}/{} depth", queue_depth, queue_capacity);
    }
    
    if let Some(pool_stats) = engine.get_memory_pool_stats() {
        info!("  📊 Memory Pool: {}", pool_stats);
    }
    
    info!("  ✅ Lock-Free: {:.2} TPS", throughput);
    Ok(())
}

async fn run_stress_test(profiler: &PerformanceProfiler, database: &Database) -> Result<()> {
    info!("  Running high-throughput stress test...");
    
    let engine = create_optimized_trading_engine(database.clone()).await?;
    let concurrent_tasks = 50;
    let orders_per_task = 200;
    let total_orders = concurrent_tasks * orders_per_task;
    
    let start_time = Instant::now();
    let mut handles = Vec::new();
    
    for task_id in 0..concurrent_tasks {
        let engine_clone = engine.clone();
        let profiler_clone = profiler.clone();
        
        let handle = tokio::spawn(async move {
            let task_orders = generate_test_orders(orders_per_task);
            let mut successful = 0;
            
            for order in task_orders {
                let order_start = Instant::now();
                
                // Use all optimization techniques
                let result = if task_id % 3 == 0 {
                    engine_clone.submit_order_adaptive(order).await
                } else if task_id % 3 == 1 {
                    engine_clone.submit_order_lock_free(order).await
                } else {
                    engine_clone.process_order_optimized(order).await.map(|_| ())
                };
                
                if result.is_ok() {
                    successful += 1;
                }
                
                let latency_ms = order_start.elapsed().as_micros() as f64 / 1000.0;
                profiler_clone.record_order_latency(latency_ms).await;
                
                // Small delay to simulate realistic load
                sleep(Duration::from_micros(100)).await;
            }
            
            successful
        });
        
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    let mut total_successful = 0;
    for handle in handles {
        match handle.await {
            Ok(successful) => total_successful += successful,
            Err(e) => error!("Stress test task failed: {}", e),
        }
    }
    
    let total_time = start_time.elapsed();
    let throughput = total_successful as f64 / total_time.as_secs_f64();
    
    info!("  ✅ Stress Test: {:.2} TPS, {}/{} success", throughput, total_successful, total_orders);
    Ok(())
}

async fn run_latency_validation_test(profiler: &PerformanceProfiler, database: &Database) -> Result<()> {
    info!("  Validating latency requirements (<10ms)...");
    
    let engine = create_optimized_trading_engine(database.clone()).await?;
    let test_orders = generate_test_orders(1000);
    let mut latencies = Vec::new();
    
    for order in test_orders {
        let start = Instant::now();
        
        if let Err(e) = engine.process_order_optimized(order).await {
            error!("Latency test order failed: {}", e);
            continue;
        }
        
        let latency_ms = start.elapsed().as_micros() as f64 / 1000.0;
        latencies.push(latency_ms);
        profiler.record_order_latency(latency_ms).await;
    }
    
    if !latencies.is_empty() {
        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p50 = latencies[latencies.len() / 2];
        let p95 = latencies[(latencies.len() * 95) / 100];
        let p99 = latencies[(latencies.len() * 99) / 100];
        let avg = latencies.iter().sum::<f64>() / latencies.len() as f64;
        
        info!("  📊 Latency Statistics:");
        info!("    Average: {:.2}ms", avg);
        info!("    P50: {:.2}ms", p50);
        info!("    P95: {:.2}ms", p95);
        info!("    P99: {:.2}ms", p99);
        
        let target_met = p95 < 10.0;
        if target_met {
            info!("  ✅ Latency target met (P95 < 10ms)");
        } else {
            warn!("  ⚠️ Latency target missed (P95 >= 10ms)");
        }
    }
    
    Ok(())
}

async fn generate_benchmark_report(profiler: &PerformanceProfiler) -> Result<()> {
    info!("📋 Generating final benchmark report...");
    
    let baseline_metrics = profiler.get_baseline_metrics().await?;
    
    info!("🎯 PERFORMANCE BENCHMARK RESULTS");
    info!("================================");
    info!("Order Execution Latency:");
    info!("  Average: {:.2}ms", baseline_metrics.trading_metrics.order_execution_latency_ms.avg_ms);
    info!("  P95: {:.2}ms", baseline_metrics.trading_metrics.order_execution_latency_ms.p95_ms);
    info!("  P99: {:.2}ms", baseline_metrics.trading_metrics.order_execution_latency_ms.p99_ms);
    info!("");
    info!("Throughput:");
    info!("  Orders/sec: {:.2}", baseline_metrics.trading_metrics.throughput_orders_per_second);
    info!("  Signals/sec: {:.2}", baseline_metrics.trading_metrics.throughput_signals_per_second);
    info!("");
    info!("System Performance:");
    info!("  CPU Usage: {:.1}%", baseline_metrics.system_metrics.cpu_usage_percent);
    info!("  Memory Usage: {:.1}%", baseline_metrics.system_metrics.memory_usage_percent);
    info!("  Error Rate: {:.2}%", baseline_metrics.trading_metrics.error_rate_percent);
    
    Ok(())
}

fn generate_test_orders(count: usize) -> Vec<OrderRequest> {
    let mut rng = rand::thread_rng();
    let mut orders = Vec::with_capacity(count);
    
    for _ in 0..count {
        let order = OrderRequest {
            instrument_id: Uuid::new_v4(),
            signal_type: if rng.gen_bool(0.5) { SignalType::Buy } else { SignalType::Sell },
            quantity: rng.gen_range(100.0..10000.0),
            confidence: rng.gen_range(0.7..1.0),
            timestamp: Utc::now(),
            strategy_name: Some("benchmark_test".to_string()),
            metadata: serde_json::json!({"test": true}),
        };
        orders.push(order);
    }
    
    orders
}

fn setup_logging() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info,sqlx=warn,hyper=warn")
        .init();
    Ok(())
}
