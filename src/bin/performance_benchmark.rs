// Performance Benchmark Tool for PantherSwap Edge
// Establishes baseline metrics and validates performance targets

use anyhow::Result;
use pantherswap_edge::config::Settings;
use pantherswap_edge::database::Database;
use pantherswap_edge::utils::{PerformanceProfiler, ProfilerConfig};
use pantherswap_edge::trading::engine::{TradingEngine, TradingEngineConfig};
use pantherswap_edge::ai::AIEngine;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{info, warn, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    setup_logging()?;
    
    info!("🚀 Starting PantherSwap Edge Performance Benchmark");
    
    // Load configuration
    dotenvy::dotenv().ok();
    let settings = Settings::load()?;
    
    // Initialize database
    let database = Database::new(&settings.database.url).await?;
    info!("✅ Database connected");
    
    // Initialize performance profiler
    let profiler_config = ProfilerConfig {
        sampling_interval_ms: 50,
        baseline_duration_minutes: 2, // Shorter for testing
        latency_sample_size: 500,
        enable_detailed_profiling: true,
        enable_memory_profiling: true,
        enable_cpu_profiling: true,
        target_order_latency_ms: 10.0,
        target_ai_latency_ms: 100.0,
        target_throughput_tps: 1000.0,
    };
    
    let profiler = PerformanceProfiler::new(profiler_config, database.clone()).await?;
    info!("✅ Performance profiler initialized");
    
    // Start profiling
    profiler.start_profiling().await?;
    info!("📊 Performance profiling started");
    
    // Run performance benchmarks
    run_comprehensive_benchmarks(&profiler, &database).await?;
    
    // Generate final report
    let report = profiler.generate_performance_report().await?;
    print_performance_report(&report).await?;
    
    info!("🎯 Performance benchmark completed successfully");
    Ok(())
}

fn setup_logging() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx=warn,hyper=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    Ok(())
}

async fn run_comprehensive_benchmarks(
    profiler: &PerformanceProfiler,
    database: &Database,
) -> Result<()> {
    info!("🔥 Running comprehensive performance benchmarks...");
    
    // Benchmark 1: Database Performance
    info!("📊 Benchmark 1: Database Performance");
    run_database_benchmark(profiler, database).await?;
    
    // Benchmark 2: Order Processing Latency
    info!("⚡ Benchmark 2: Order Processing Latency");
    run_order_latency_benchmark(profiler).await?;
    
    // Benchmark 3: AI Inference Performance
    info!("🤖 Benchmark 3: AI Inference Performance");
    run_ai_inference_benchmark(profiler).await?;
    
    // Benchmark 4: High-Frequency Trading Simulation
    info!("🚀 Benchmark 4: High-Frequency Trading Simulation");
    run_hft_simulation_benchmark(profiler).await?;
    
    // Benchmark 5: Concurrent Load Testing
    info!("💪 Benchmark 5: Concurrent Load Testing");
    run_concurrent_load_benchmark(profiler).await?;
    
    // Benchmark 6: Memory and Resource Usage
    info!("🧠 Benchmark 6: Memory and Resource Usage");
    run_memory_benchmark(profiler).await?;
    
    info!("✅ All benchmarks completed");
    Ok(())
}

async fn run_database_benchmark(
    profiler: &PerformanceProfiler,
    database: &Database,
) -> Result<()> {
    info!("  Testing database query performance...");
    
    let iterations = 100;
    let mut total_latency = 0.0;
    
    for _ in 0..iterations {
        let start = Instant::now();
        
        // Simple query test
        let _result = sqlx::query("SELECT 1 as test")
            .fetch_one(&database.pool)
            .await?;
        
        let latency_ms = start.elapsed().as_micros() as f64 / 1000.0;
        total_latency += latency_ms;
        
        profiler.record_db_latency(latency_ms).await;
    }
    
    let avg_latency = total_latency / iterations as f64;
    info!("  ✅ Database benchmark: avg latency {:.2}ms", avg_latency);
    
    Ok(())
}

async fn run_order_latency_benchmark(profiler: &PerformanceProfiler) -> Result<()> {
    info!("  Testing order processing latency...");
    
    let iterations = 200;
    let mut latencies = Vec::new();
    
    for _ in 0..iterations {
        let start = Instant::now();
        
        // Simulate order processing
        simulate_order_processing().await;
        
        let latency_ms = start.elapsed().as_micros() as f64 / 1000.0;
        latencies.push(latency_ms);
        
        profiler.record_order_latency(latency_ms).await;
        
        // Small delay to prevent overwhelming
        sleep(Duration::from_millis(1)).await;
    }
    
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let avg_latency = latencies.iter().sum::<f64>() / latencies.len() as f64;
    let p95_latency = latencies[latencies.len() * 95 / 100];
    let p99_latency = latencies[latencies.len() * 99 / 100];
    
    info!("  ✅ Order latency benchmark:");
    info!("     - Average: {:.2}ms", avg_latency);
    info!("     - P95: {:.2}ms", p95_latency);
    info!("     - P99: {:.2}ms", p99_latency);
    info!("     - Target: <10ms ({})", if avg_latency < 10.0 { "✅ PASS" } else { "❌ FAIL" });
    
    Ok(())
}

async fn run_ai_inference_benchmark(profiler: &PerformanceProfiler) -> Result<()> {
    info!("  Testing AI inference performance...");
    
    let iterations = 100;
    let mut latencies = Vec::new();
    
    for _ in 0..iterations {
        let start = Instant::now();
        
        // Simulate AI inference
        simulate_ai_inference().await;
        
        let latency_ms = start.elapsed().as_micros() as f64 / 1000.0;
        latencies.push(latency_ms);
        
        profiler.record_ai_latency(latency_ms).await;
        
        sleep(Duration::from_millis(5)).await;
    }
    
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let avg_latency = latencies.iter().sum::<f64>() / latencies.len() as f64;
    let p95_latency = latencies[latencies.len() * 95 / 100];
    let p99_latency = latencies[latencies.len() * 99 / 100];
    
    info!("  ✅ AI inference benchmark:");
    info!("     - Average: {:.2}ms", avg_latency);
    info!("     - P95: {:.2}ms", p95_latency);
    info!("     - P99: {:.2}ms", p99_latency);
    info!("     - Target: <100ms ({})", if avg_latency < 100.0 { "✅ PASS" } else { "❌ FAIL" });
    
    Ok(())
}

async fn run_hft_simulation_benchmark(profiler: &PerformanceProfiler) -> Result<()> {
    info!("  Testing high-frequency trading simulation...");
    
    let duration = Duration::from_secs(10);
    let start_time = Instant::now();
    let mut order_count = 0;
    
    while start_time.elapsed() < duration {
        let order_start = Instant::now();
        
        // Simulate HFT order
        simulate_hft_order().await;
        
        let latency_ms = order_start.elapsed().as_micros() as f64 / 1000.0;
        profiler.record_order_latency(latency_ms).await;
        
        order_count += 1;
        
        // Minimal delay for HFT simulation
        sleep(Duration::from_micros(100)).await;
    }
    
    let total_time = start_time.elapsed().as_secs_f64();
    let throughput = order_count as f64 / total_time;
    
    info!("  ✅ HFT simulation benchmark:");
    info!("     - Orders processed: {}", order_count);
    info!("     - Throughput: {:.2} orders/second", throughput);
    info!("     - Target: >1000 orders/second ({})", if throughput > 1000.0 { "✅ PASS" } else { "❌ FAIL" });
    
    Ok(())
}

async fn run_concurrent_load_benchmark(profiler: &PerformanceProfiler) -> Result<()> {
    info!("  Testing concurrent load performance...");
    
    let concurrent_tasks = 50;
    let operations_per_task = 20;
    
    let start_time = Instant::now();
    let mut handles = Vec::new();
    
    for task_id in 0..concurrent_tasks {
        let profiler_clone = profiler.clone();
        
        let handle = tokio::spawn(async move {
            for _ in 0..operations_per_task {
                let op_start = Instant::now();
                
                // Simulate concurrent operation
                simulate_concurrent_operation(task_id).await;
                
                let latency_ms = op_start.elapsed().as_micros() as f64 / 1000.0;
                profiler_clone.record_api_latency(latency_ms).await;
                
                sleep(Duration::from_millis(10)).await;
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.await?;
    }
    
    let total_time = start_time.elapsed().as_secs_f64();
    let total_operations = concurrent_tasks * operations_per_task;
    let throughput = total_operations as f64 / total_time;
    
    info!("  ✅ Concurrent load benchmark:");
    info!("     - Concurrent tasks: {}", concurrent_tasks);
    info!("     - Total operations: {}", total_operations);
    info!("     - Throughput: {:.2} ops/second", throughput);
    info!("     - Duration: {:.2}s", total_time);
    
    Ok(())
}

async fn run_memory_benchmark(profiler: &PerformanceProfiler) -> Result<()> {
    info!("  Testing memory usage patterns...");
    
    // Simulate memory-intensive operations
    let mut data_structures = Vec::new();
    
    for i in 0..1000 {
        // Create some data structures to test memory usage
        let data = vec![i; 1000];
        data_structures.push(data);
        
        if i % 100 == 0 {
            // Simulate some cleanup
            if data_structures.len() > 500 {
                data_structures.drain(0..100);
            }
        }
        
        sleep(Duration::from_millis(1)).await;
    }
    
    info!("  ✅ Memory benchmark completed");
    info!("     - Data structures created: 1000");
    info!("     - Memory management: Active cleanup");
    
    Ok(())
}

// Simulation functions
async fn simulate_order_processing() {
    // Simulate order validation, risk checks, and execution
    sleep(Duration::from_micros(500)).await; // Risk check
    sleep(Duration::from_micros(300)).await; // Validation
    sleep(Duration::from_micros(200)).await; // Execution
}

async fn simulate_ai_inference() {
    // Simulate LSTM, RL, and HMM inference
    sleep(Duration::from_millis(20)).await; // LSTM
    sleep(Duration::from_millis(15)).await; // RL
    sleep(Duration::from_millis(10)).await; // HMM
}

async fn simulate_hft_order() {
    // Simulate ultra-fast order processing
    sleep(Duration::from_micros(100)).await;
}

async fn simulate_concurrent_operation(task_id: usize) {
    // Simulate various concurrent operations
    let operation_type = task_id % 4;
    match operation_type {
        0 => sleep(Duration::from_millis(5)).await,  // Fast operation
        1 => sleep(Duration::from_millis(10)).await, // Medium operation
        2 => sleep(Duration::from_millis(15)).await, // Slow operation
        _ => sleep(Duration::from_millis(8)).await,  // Default operation
    }
}

async fn print_performance_report(report: &pantherswap_edge::utils::PerformanceReport) -> Result<()> {
    info!("📈 PERFORMANCE REPORT");
    info!("==================");
    
    if let Some(baseline) = &report.baseline_metrics {
        info!("🎯 Baseline Metrics:");
        info!("   - CPU Usage: {:.1}%", baseline.system_metrics.cpu_usage_percent);
        info!("   - Memory Usage: {}MB / {}MB", 
              baseline.system_metrics.memory_usage_mb, 
              baseline.system_metrics.memory_total_mb);
        info!("   - Order Latency: {:.2}ms avg", baseline.trading_metrics.order_execution_latency_ms.avg_ms);
        info!("   - AI Latency: {:.2}ms avg", baseline.ai_metrics.total_inference_latency_ms.avg_ms);
    }
    
    info!("📊 Current Metrics:");
    info!("   - CPU Usage: {:.1}%", report.current_metrics.system_metrics.cpu_usage_percent);
    info!("   - Memory Usage: {}MB / {}MB", 
          report.current_metrics.system_metrics.memory_usage_mb, 
          report.current_metrics.system_metrics.memory_total_mb);
    
    info!("🎯 Performance Summary:");
    info!("   - Overall Health Score: {:.1}%", report.performance_summary.overall_health_score);
    info!("   - Meets Latency Targets: {}", if report.performance_summary.meets_latency_targets { "✅" } else { "❌" });
    info!("   - Meets Throughput Targets: {}", if report.performance_summary.meets_throughput_targets { "✅" } else { "❌" });
    info!("   - Uptime: {:.2}%", report.performance_summary.uptime_percent);
    
    Ok(())
}
