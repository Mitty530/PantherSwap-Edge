use pantherswap_edge::config::Settings;
use pantherswap_edge::database::Database;
use pantherswap_edge::testing::{
    AlpacaIntegrationTestSuite, TestConfiguration, PerformanceTargets
};
use pantherswap_edge::utils::Result;
use std::env;
use tracing::{info, error, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    info!("🚀 PantherSwap Edge - Alpaca Integration Test Suite");
    info!("================================================");

    // Check for test mode argument
    let args: Vec<String> = env::args().collect();
    let test_mode = args.get(1).map(|s| s.as_str()).unwrap_or("full");

    match test_mode {
        "quick" => run_quick_test().await,
        "connectivity" => run_connectivity_test().await,
        "performance" => run_performance_test().await,
        "full" => run_full_test_suite().await,
        _ => {
            info!("Usage: {} [quick|connectivity|performance|full]", args[0]);
            info!("  quick        - Quick connectivity and basic functionality test");
            info!("  connectivity - Test Alpaca API connectivity only");
            info!("  performance  - Focus on performance validation");
            info!("  full         - Complete integration test suite (default)");
            Ok(())
        }
    }
}

async fn run_quick_test() -> Result<()> {
    info!("🏃 Running quick Alpaca integration test...");

    let settings = Settings::load()?;
    let database = Database::new(&settings.database).await?;

    let test_config = TestConfiguration {
        test_symbols: vec!["AAPL".to_string(), "SPY".to_string()],
        test_duration_seconds: 60, // 1 minute
        max_test_orders: 2,
        enable_live_data_test: true,
        enable_order_execution_test: false, // Skip for quick test
        enable_database_logging_test: true,
        enable_performance_validation: false,
        test_budget_usd: 100.0,
        performance_targets: PerformanceTargets {
            max_execution_latency_ms: 10.0,
            max_ai_inference_latency_ms: 100.0,
            min_throughput_tps: 100.0, // Lower target for quick test
            min_data_quality_score: 0.8,
            max_error_rate_percent: 5.0,
        },
    };

    let mut test_suite = AlpacaIntegrationTestSuite::new(settings, database)
        .await?
        .with_config(test_config);

    test_suite.initialize_components().await?;

    // Run connectivity and market data tests only
    let connectivity_results = test_suite.run_connectivity_tests().await?;
    let market_data_results = test_suite.run_market_data_tests().await?;

    info!("📊 Quick Test Results:");
    info!("   Connectivity: {}", if connectivity_results.alpaca_api_connection { "✅" } else { "❌" });
    info!("   Market Data: {}", if market_data_results.real_time_quotes { "✅" } else { "❌" });
    info!("   Data Quality: {:.2}", market_data_results.data_quality_score);

    let overall_success = connectivity_results.alpaca_api_connection && 
                         market_data_results.real_time_quotes;

    if overall_success {
        info!("✅ Quick test PASSED - Alpaca integration is functional");
    } else {
        error!("❌ Quick test FAILED - Issues detected with Alpaca integration");
    }

    Ok(())
}

async fn run_connectivity_test() -> Result<()> {
    info!("🔌 Running Alpaca connectivity test...");

    let settings = Settings::load()?;
    let database = Database::new(&settings.database).await?;

    let mut test_suite = AlpacaIntegrationTestSuite::new(settings, database).await?;
    test_suite.initialize_components().await?;

    let results = test_suite.run_connectivity_tests().await?;

    info!("📊 Connectivity Test Results:");
    info!("   API Connection: {}", if results.alpaca_api_connection { "✅" } else { "❌" });
    info!("   Market Data: {}", if results.market_data_connection { "✅" } else { "❌" });
    info!("   Trading API: {}", if results.trading_api_connection { "✅" } else { "❌" });
    info!("   Account Access: {}", if results.account_access { "✅" } else { "❌" });
    info!("   Authentication: {}", if results.authentication_valid { "✅" } else { "❌" });
    info!("   Market Status: {}", if results.market_status_check { "✅" } else { "❌" });

    let overall_success = results.alpaca_api_connection && 
                         results.account_access && 
                         results.authentication_valid;

    if overall_success {
        info!("✅ Connectivity test PASSED");
    } else {
        error!("❌ Connectivity test FAILED");
    }

    Ok(())
}

async fn run_performance_test() -> Result<()> {
    info!("⚡ Running Alpaca performance test...");

    let settings = Settings::load()?;
    let database = Database::new(&settings.database).await?;

    let test_config = TestConfiguration {
        test_symbols: vec![
            "AAPL".to_string(), "MSFT".to_string(), "GOOGL".to_string(),
            "TSLA".to_string(), "SPY".to_string(), "QQQ".to_string()
        ],
        test_duration_seconds: 300, // 5 minutes
        max_test_orders: 20,
        enable_live_data_test: true,
        enable_order_execution_test: true,
        enable_database_logging_test: true,
        enable_performance_validation: true,
        test_budget_usd: 1000.0,
        performance_targets: PerformanceTargets {
            max_execution_latency_ms: 10.0,
            max_ai_inference_latency_ms: 100.0,
            min_throughput_tps: 1000.0,
            min_data_quality_score: 0.9,
            max_error_rate_percent: 1.0,
        },
    };

    let mut test_suite = AlpacaIntegrationTestSuite::new(settings, database)
        .await?
        .with_config(test_config);

    test_suite.initialize_components().await?;

    let performance_results = test_suite.run_performance_tests().await?;

    info!("📊 Performance Test Results:");
    info!("   Execution Latency: {:.1}ms (target: ≤10ms) {}", 
        performance_results.measured_execution_latency_ms,
        if performance_results.execution_latency_target_met { "✅" } else { "❌" });
    info!("   AI Inference: {:.1}ms (target: ≤100ms) {}", 
        performance_results.measured_ai_inference_latency_ms,
        if performance_results.ai_inference_latency_target_met { "✅" } else { "❌" });
    info!("   Throughput: {:.1} TPS (target: ≥1000 TPS) {}", 
        performance_results.measured_throughput_tps,
        if performance_results.throughput_target_met { "✅" } else { "❌" });
    info!("   Error Rate: {:.1}% (target: ≤1%) {}", 
        performance_results.measured_error_rate_percent,
        if performance_results.error_rate_target_met { "✅" } else { "❌" });

    let all_targets_met = performance_results.execution_latency_target_met &&
                         performance_results.ai_inference_latency_target_met &&
                         performance_results.throughput_target_met &&
                         performance_results.error_rate_target_met;

    if all_targets_met {
        info!("✅ Performance test PASSED - All targets met");
    } else {
        error!("❌ Performance test FAILED - Some targets not met");
    }

    Ok(())
}

async fn run_full_test_suite() -> Result<()> {
    info!("🎯 Running complete Alpaca integration test suite...");

    let settings = Settings::load()?;
    let database = Database::new(&settings.database).await?;

    let mut test_suite = AlpacaIntegrationTestSuite::new(settings, database).await?;
    test_suite.initialize_components().await?;

    let results = test_suite.run_complete_test_suite().await?;

    // Generate comprehensive report
    let report = test_suite.generate_test_report(&results);
    
    // Save report to file
    let report_filename = format!("alpaca_integration_test_report_{}.md", 
        chrono::Utc::now().format("%Y%m%d_%H%M%S"));
    
    if let Err(e) = std::fs::write(&report_filename, &report) {
        error!("Failed to save report to {}: {}", report_filename, e);
    } else {
        info!("📄 Detailed report saved to: {}", report_filename);
    }

    // Print summary
    info!("📊 Test Suite Summary:");
    info!("   Duration: {:.1} seconds", results.duration_seconds);
    info!("   Total Tests: {}", results.test_summary.total_tests);
    info!("   Passed: {} ✅", results.test_summary.passed_tests);
    info!("   Failed: {} ❌", results.test_summary.failed_tests);
    info!("   Success Rate: {:.1}%", results.test_summary.success_rate * 100.0);

    if results.overall_success {
        info!("🎉 OVERALL RESULT: ✅ PASS");
        info!("🚀 Alpaca integration is ready for production use!");
    } else {
        error!("💥 OVERALL RESULT: ❌ FAIL");
        error!("⚠️  Issues detected - review the detailed report before production deployment");
    }

    Ok(())
}
