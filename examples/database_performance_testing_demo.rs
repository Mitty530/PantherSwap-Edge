// Database performance testing demo for PantherSwap Edge
// Demonstrates comprehensive performance testing, benchmarking, and analysis
// Run with: DATABASE_URL="..." cargo run --example database_performance_testing_demo

use pantherswap_edge::database::{Database, PerformanceTestManager, PerformanceTestConfig, TestScenario};
use pantherswap_edge::config::Settings;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("⚡ PantherSwap Edge Database Performance Testing Demo");
    println!("===================================================");
    
    // Load configuration
    let settings = Settings::load()?;
    let database_url = &settings.database.url;
    
    // Test 1: Basic Performance Test Manager Setup
    println!("\n🔧 Setting up Performance Test Manager...");
    
    let database = Database::new_cloud(database_url).await?;
    println!("✅ Database connection established");
    
    let mut perf_manager = database.performance_test_manager();
    println!("✅ Performance test manager created with trading defaults");
    
    // Display default configuration
    println!("📊 Default Performance Test Configuration:");
    println!("   - Concurrent connections: {}", perf_manager.config.concurrent_connections);
    println!("   - Test duration: {}s", perf_manager.config.test_duration_seconds);
    println!("   - Target latency: {}ms", perf_manager.config.target_latency_ms);
    println!("   - Target throughput: {:.0} QPS", perf_manager.config.target_throughput_qps);

    // Test 2: Basic Connectivity Performance Test
    println!("\n🔍 Running Basic Connectivity Performance Test...");
    
    let connectivity_result = perf_manager.test_basic_connectivity().await?;
    println!("✅ Basic connectivity test completed:");
    println!("   - Test passed: {}", if connectivity_result.passed { "✅" } else { "❌" });
    println!("   - Total queries: {}", connectivity_result.metrics.total_queries);
    println!("   - Successful queries: {}", connectivity_result.metrics.successful_queries);
    println!("   - Average latency: {:.2}ms", connectivity_result.metrics.average_latency_ms);
    println!("   - Median latency: {:.2}ms", connectivity_result.metrics.median_latency_ms);
    println!("   - P95 latency: {:.2}ms", connectivity_result.metrics.p95_latency_ms);
    println!("   - P99 latency: {:.2}ms", connectivity_result.metrics.p99_latency_ms);
    println!("   - Queries per second: {:.2}", connectivity_result.metrics.queries_per_second);
    println!("   - Error rate: {:.2}%", connectivity_result.error_metrics.error_rate_percent);
    
    // Display latency distribution
    println!("📊 Latency Distribution:");
    println!("   - Under 1ms: {}", connectivity_result.latency_distribution.under_1ms);
    println!("   - Under 5ms: {}", connectivity_result.latency_distribution.under_5ms);
    println!("   - Under 10ms: {}", connectivity_result.latency_distribution.under_10ms);
    println!("   - Under 50ms: {}", connectivity_result.latency_distribution.under_50ms);
    println!("   - Under 100ms: {}", connectivity_result.latency_distribution.under_100ms);
    println!("   - Over 500ms: {}", connectivity_result.latency_distribution.over_500ms);

    // Test 3: Connection Pool Performance Test
    println!("\n🏊 Running Connection Pool Performance Test...");
    
    let pool_result = perf_manager.test_connection_pool_performance().await?;
    println!("✅ Connection pool test completed:");
    println!("   - Test passed: {}", if pool_result.passed { "✅" } else { "❌" });
    println!("   - Average latency: {:.2}ms", pool_result.metrics.average_latency_ms);
    println!("   - Peak QPS: {:.2}", pool_result.throughput_metrics.peak_qps);
    println!("   - Sustained QPS: {:.2}", pool_result.throughput_metrics.sustained_qps);
    println!("   - Connection efficiency: {:.1}%", pool_result.throughput_metrics.connection_efficiency);
    println!("   - Peak connections: {}", pool_result.resource_usage.peak_connections);
    println!("   - Pool utilization: {:.1}%", pool_result.resource_usage.connection_utilization_percent);

    // Test 4: Query Performance Under Load
    println!("\n📈 Running Query Performance Under Load Test...");
    
    let load_result = perf_manager.test_query_performance_under_load().await?;
    println!("✅ Query performance under load test completed:");
    println!("   - Test passed: {}", if load_result.passed { "✅" } else { "❌" });
    println!("   - Total queries: {}", load_result.metrics.total_queries);
    println!("   - Average latency: {:.2}ms", load_result.metrics.average_latency_ms);
    println!("   - P95 latency: {:.2}ms", load_result.metrics.p95_latency_ms);
    println!("   - Throughput variance: {:.1}%", load_result.throughput_metrics.throughput_variance);
    println!("   - Error rate: {:.2}%", load_result.error_metrics.error_rate_percent);

    // Test 5: Concurrent Read/Write Performance
    println!("\n🔄 Running Concurrent Read/Write Performance Test...");
    
    let rw_result = perf_manager.test_concurrent_read_write().await?;
    println!("✅ Concurrent read/write test completed:");
    println!("   - Test passed: {}", if rw_result.passed { "✅" } else { "❌" });
    println!("   - Average latency: {:.2}ms", rw_result.metrics.average_latency_ms);
    println!("   - Queries per second: {:.2}", rw_result.metrics.queries_per_second);
    println!("   - Connection efficiency: {:.1}%", rw_result.throughput_metrics.connection_efficiency);

    // Test 6: High-Frequency Trading Simulation
    println!("\n🚀 Running High-Frequency Trading Simulation...");
    
    let hft_result = perf_manager.test_hft_simulation().await?;
    println!("✅ HFT simulation test completed:");
    println!("   - Test passed: {}", if hft_result.passed { "✅" } else { "❌" });
    println!("   - Average latency: {:.2}ms", hft_result.metrics.average_latency_ms);
    println!("   - P95 latency: {:.2}ms", hft_result.metrics.p95_latency_ms);
    println!("   - P99 latency: {:.2}ms", hft_result.metrics.p99_latency_ms);
    println!("   - Peak QPS: {:.2}", hft_result.throughput_metrics.peak_qps);
    println!("   - Throughput variance: {:.1}%", hft_result.throughput_metrics.throughput_variance);
    
    if hft_result.passed {
        println!("   🎯 HFT performance requirements met!");
    } else {
        println!("   ⚠️  HFT performance requirements not met");
        for recommendation in &hft_result.recommendations {
            println!("      • {}", recommendation);
        }
    }

    // Test 7: TimescaleDB Performance
    println!("\n⏰ Running TimescaleDB Performance Test...");
    
    let ts_result = perf_manager.test_timescale_performance().await?;
    println!("✅ TimescaleDB test completed:");
    println!("   - Test passed: {}", if ts_result.passed { "✅" } else { "❌" });
    println!("   - Average latency: {:.2}ms", ts_result.metrics.average_latency_ms);
    println!("   - Queries per second: {:.2}", ts_result.metrics.queries_per_second);
    
    if !ts_result.recommendations.is_empty() {
        println!("💡 TimescaleDB Recommendations:");
        for recommendation in &ts_result.recommendations {
            println!("   • {}", recommendation);
        }
    }

    // Test 8: Stress Testing
    println!("\n💪 Running Stress Test Scenarios...");
    
    let stress_result = perf_manager.test_stress_scenarios().await?;
    println!("✅ Stress test completed:");
    println!("   - Test passed: {}", if stress_result.passed { "✅" } else { "❌" });
    println!("   - Average latency: {:.2}ms", stress_result.metrics.average_latency_ms);
    println!("   - Peak QPS: {:.2}", stress_result.throughput_metrics.peak_qps);
    println!("   - Sustained QPS: {:.2}", stress_result.throughput_metrics.sustained_qps);
    println!("   - Error rate: {:.2}%", stress_result.error_metrics.error_rate_percent);

    // Test 9: Comprehensive Test Suite
    println!("\n🧪 Running Comprehensive Performance Test Suite...");
    
    let suite_results = perf_manager.run_comprehensive_test_suite().await?;
    println!("✅ Comprehensive test suite completed:");
    println!("   - Total tests: {}", suite_results.len());
    
    let passed_tests = suite_results.iter().filter(|r| r.passed).count();
    let failed_tests = suite_results.len() - passed_tests;
    
    println!("   - Passed tests: {} ✅", passed_tests);
    println!("   - Failed tests: {} ❌", failed_tests);
    println!("   - Success rate: {:.1}%", (passed_tests as f64 / suite_results.len() as f64) * 100.0);

    // Test 10: Custom Performance Test Configuration
    println!("\n⚙️  Testing Custom Performance Configuration...");
    
    let custom_config = PerformanceTestConfig {
        concurrent_connections: 20,
        test_duration_seconds: 30,
        target_latency_ms: 2, // Very aggressive target
        target_throughput_qps: 2000.0,
        enable_detailed_metrics: true,
        ..Default::default()
    };
    
    let mut custom_manager = database.performance_test_manager_with_config(custom_config);
    println!("✅ Custom performance test manager created");
    println!("   - Concurrent connections: {}", custom_manager.config.concurrent_connections);
    println!("   - Target latency: {}ms", custom_manager.config.target_latency_ms);
    println!("   - Target throughput: {:.0} QPS", custom_manager.config.target_throughput_qps);
    
    let custom_result = custom_manager.test_basic_connectivity().await?;
    println!("✅ Custom configuration test:");
    println!("   - Test passed: {}", if custom_result.passed { "✅" } else { "❌" });
    println!("   - Average latency: {:.2}ms", custom_result.metrics.average_latency_ms);

    // Test 11: Performance Report Generation
    println!("\n📊 Generating Performance Report...");
    
    let performance_report = perf_manager.generate_performance_report();
    println!("✅ Performance report generated:");
    println!("   - Total tests run: {}", performance_report.total_tests);
    println!("   - Tests passed: {}", performance_report.passed_tests);
    println!("   - Tests failed: {}", performance_report.failed_tests);
    println!("   - Average score: {:.1}%", performance_report.average_score);
    
    println!("💡 Overall Recommendations:");
    for recommendation in &performance_report.recommendations {
        println!("   • {}", recommendation);
    }

    // Test 12: Test Scenario Examples
    println!("\n🎯 Testing Specific Trading Scenarios...");
    
    // Market data ingestion scenario
    let market_data_scenario = TestScenario::MarketDataIngestion {
        ticks_per_second: 1000,
        instruments: 50,
    };
    
    let scenario_result = perf_manager.run_test_scenario(market_data_scenario).await?;
    println!("✅ Market data ingestion scenario:");
    println!("   - Test passed: {}", if scenario_result.passed { "✅" } else { "❌" });
    println!("   - Average latency: {:.2}ms", scenario_result.metrics.average_latency_ms);

    // Test 13: Performance Metrics Summary
    println!("\n📈 Performance Metrics Summary...");
    
    let history = perf_manager.get_results_history();
    if !history.is_empty() {
        let avg_latency = history.iter()
            .map(|r| r.metrics.average_latency_ms)
            .sum::<f64>() / history.len() as f64;
        
        let avg_throughput = history.iter()
            .map(|r| r.metrics.queries_per_second)
            .sum::<f64>() / history.len() as f64;
        
        let overall_success_rate = (history.iter().filter(|r| r.passed).count() as f64 / history.len() as f64) * 100.0;
        
        println!("✅ Overall Performance Summary:");
        println!("   - Tests in history: {}", history.len());
        println!("   - Average latency: {:.2}ms", avg_latency);
        println!("   - Average throughput: {:.2} QPS", avg_throughput);
        println!("   - Overall success rate: {:.1}%", overall_success_rate);
        
        // Performance classification
        let performance_class = if avg_latency < 5.0 && avg_throughput > 1000.0 && overall_success_rate > 90.0 {
            "🚀 EXCELLENT"
        } else if avg_latency < 10.0 && avg_throughput > 500.0 && overall_success_rate > 80.0 {
            "✅ GOOD"
        } else if avg_latency < 50.0 && avg_throughput > 100.0 && overall_success_rate > 70.0 {
            "⚠️  ACCEPTABLE"
        } else {
            "❌ NEEDS IMPROVEMENT"
        };
        
        println!("   - Performance classification: {}", performance_class);
    }

    // Cleanup
    println!("\n🧹 Cleaning up...");
    
    // Clear performance test history
    perf_manager.clear_history();
    println!("✅ Performance test history cleared");
    
    // Close database connection
    database.close().await;
    println!("✅ Database connection closed");

    println!("\n🎉 Database Performance Testing Demo Completed Successfully!");
    println!("===========================================================");
    println!("✅ Basic connectivity performance tested");
    println!("✅ Connection pool performance validated");
    println!("✅ Query performance under load measured");
    println!("✅ Concurrent read/write performance assessed");
    println!("✅ High-frequency trading simulation completed");
    println!("✅ TimescaleDB performance evaluated");
    println!("✅ Stress testing scenarios executed");
    println!("✅ Comprehensive test suite run");
    println!("✅ Custom configuration testing verified");
    println!("✅ Performance reporting generated");
    println!("✅ Trading scenario testing demonstrated");
    println!("✅ Performance metrics analysis completed");
    
    Ok(())
}
