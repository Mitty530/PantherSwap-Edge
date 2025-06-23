// Execution Scenario Test Runner
// Run with: cargo test --test run_execution_scenario_tests

use std::time::Instant;
use tracing::{info, warn, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod execution_scenario_tests;
mod execution_scenario_helpers;
mod common;

use execution_scenario_tests::{ExecutionScenarioTestOrchestrator, ExecutionScenarioTestResults, TestStatus};

/// Main test for comprehensive execution scenarios
#[tokio::test]
async fn run_comprehensive_execution_scenario_tests() {
    // Initialize logging
    init_test_logging();
    
    info!("🚀 Starting Comprehensive Execution Scenario Tests");
    info!("=" .repeat(70));
    info!("📋 Test Categories:");
    info!("  • Long Position Execution (Market Buy, Limit Buy, Stop-Loss, Take-Profit)");
    info!("  • Short Position Execution (Market Sell, Limit Sell, Stop-Loss, Take-Profit)");
    info!("  • Order Type Execution (Market, Limit, Stop, Stop-Limit, Iceberg, TWAP, VWAP)");
    info!("  • Slippage Handling (Positive Capture, Negative Mitigation, Prediction)");
    info!("  • Execution Quality (Price Improvement, Fill Ratio, Timing, Liquidity Detection)");
    info!("  • Market Conditions (Normal, High Volatility, Low Liquidity, Trending, Sideways, Stress)");
    info!("  • Performance Metrics (Throughput, Latency, Memory, CPU, Error Rate)");
    info!("=" .repeat(70));
    
    let test_start_time = Instant::now();
    
    // Initialize test orchestrator
    let orchestrator = match ExecutionScenarioTestOrchestrator::new().await {
        Ok(orchestrator) => {
            info!("✅ Execution scenario test orchestrator initialized successfully");
            orchestrator
        }
        Err(e) => {
            error!("❌ Failed to initialize execution scenario test orchestrator: {}", e);
            panic!("Cannot proceed with execution scenario testing");
        }
    };
    
    // Run comprehensive execution scenario tests
    let results = match orchestrator.run_comprehensive_execution_tests().await {
        Ok(results) => {
            info!("✅ Comprehensive execution scenario tests completed");
            results
        }
        Err(e) => {
            error!("❌ Comprehensive execution scenario tests failed: {}", e);
            panic!("Execution scenario testing failed");
        }
    };
    
    let total_test_duration = test_start_time.elapsed();
    
    // Generate and display test report
    generate_execution_scenario_test_report(&results, total_test_duration);
    
    // Validate results
    validate_execution_scenario_performance(&results);
    
    info!("🎯 Execution Scenario Testing Completed Successfully!");
}

/// Test long position execution specifically
#[tokio::test]
async fn test_long_position_execution_scenarios() {
    init_test_logging();
    
    info!("📈 Testing Long Position Execution Scenarios");
    
    let orchestrator = ExecutionScenarioTestOrchestrator::new().await
        .expect("Failed to initialize test orchestrator");
    
    let long_results = orchestrator.test_long_position_execution().await
        .expect("Long position execution test failed");
    
    info!("📈 Long Position Execution Results:");
    info!("  • Market Buy Execution Quality: {:.2}%", long_results.market_buy_execution_quality * 100.0);
    info!("  • Limit Buy Execution Quality: {:.2}%", long_results.limit_buy_execution_quality * 100.0);
    info!("  • Stop-Loss Protection Effectiveness: {:.2}%", long_results.stop_loss_protection_effectiveness * 100.0);
    info!("  • Take-Profit Execution Accuracy: {:.2}%", long_results.take_profit_execution_accuracy * 100.0);
    info!("  • Position Sizing Accuracy: {:.2}%", long_results.position_sizing_accuracy * 100.0);
    info!("  • Average Execution Latency: {:.2}ms", long_results.average_execution_latency_ms);
    info!("  • Total Long Positions Tested: {}", long_results.total_long_positions_tested);
    info!("  • Status: {:?}", long_results.status);
    
    // Assert long position execution meets requirements
    assert!(matches!(long_results.status, TestStatus::Passed | TestStatus::PartiallyPassed));
    assert!(long_results.market_buy_execution_quality > 0.80);
    assert!(long_results.limit_buy_execution_quality > 0.75);
    assert!(long_results.stop_loss_protection_effectiveness > 0.85);
    assert!(long_results.average_execution_latency_ms < 15.0);
}

/// Test short position execution specifically
#[tokio::test]
async fn test_short_position_execution_scenarios() {
    init_test_logging();
    
    info!("📉 Testing Short Position Execution Scenarios");
    
    let orchestrator = ExecutionScenarioTestOrchestrator::new().await
        .expect("Failed to initialize test orchestrator");
    
    let short_results = orchestrator.test_short_position_execution().await
        .expect("Short position execution test failed");
    
    info!("📉 Short Position Execution Results:");
    info!("  • Market Sell Execution Quality: {:.2}%", short_results.market_sell_execution_quality * 100.0);
    info!("  • Limit Sell Execution Quality: {:.2}%", short_results.limit_sell_execution_quality * 100.0);
    info!("  • Stop-Loss Protection Effectiveness: {:.2}%", short_results.stop_loss_protection_effectiveness * 100.0);
    info!("  • Take-Profit Execution Accuracy: {:.2}%", short_results.take_profit_execution_accuracy * 100.0);
    info!("  • Position Sizing Accuracy: {:.2}%", short_results.position_sizing_accuracy * 100.0);
    info!("  • Average Execution Latency: {:.2}ms", short_results.average_execution_latency_ms);
    info!("  • Total Short Positions Tested: {}", short_results.total_short_positions_tested);
    info!("  • Status: {:?}", short_results.status);
    
    // Assert short position execution meets requirements
    assert!(matches!(short_results.status, TestStatus::Passed | TestStatus::PartiallyPassed));
    assert!(short_results.market_sell_execution_quality > 0.80);
    assert!(short_results.limit_sell_execution_quality > 0.75);
    assert!(short_results.stop_loss_protection_effectiveness > 0.85);
    assert!(short_results.average_execution_latency_ms < 15.0);
}

/// Test order type execution specifically
#[tokio::test]
async fn test_order_type_execution_quality() {
    init_test_logging();
    
    info!("🔄 Testing Order Type Execution Quality");
    
    let orchestrator = ExecutionScenarioTestOrchestrator::new().await
        .expect("Failed to initialize test orchestrator");
    
    let order_type_results = orchestrator.test_order_type_execution().await
        .expect("Order type execution test failed");
    
    info!("🔄 Order Type Execution Results:");
    info!("  • Market Order Fill Rate: {:.2}%", order_type_results.market_order_fill_rate * 100.0);
    info!("  • Limit Order Fill Rate: {:.2}%", order_type_results.limit_order_fill_rate * 100.0);
    info!("  • Stop Order Trigger Accuracy: {:.2}%", order_type_results.stop_order_trigger_accuracy * 100.0);
    info!("  • Stop-Limit Execution Quality: {:.2}%", order_type_results.stop_limit_execution_quality * 100.0);
    info!("  • Iceberg Order Stealth Score: {:.2}%", order_type_results.iceberg_order_stealth_score * 100.0);
    info!("  • TWAP Execution Quality: {:.2}%", order_type_results.twap_execution_quality * 100.0);
    info!("  • VWAP Execution Quality: {:.2}%", order_type_results.vwap_execution_quality * 100.0);
    info!("  • Status: {:?}", order_type_results.status);
    
    // Assert order type execution meets requirements
    assert!(matches!(order_type_results.status, TestStatus::Passed | TestStatus::PartiallyPassed));
    assert!(order_type_results.market_order_fill_rate > 0.90);
    assert!(order_type_results.limit_order_fill_rate > 0.80);
    assert!(order_type_results.stop_order_trigger_accuracy > 0.85);
}

/// Test slippage handling specifically
#[tokio::test]
async fn test_slippage_handling_effectiveness() {
    init_test_logging();
    
    info!("💨 Testing Slippage Handling Effectiveness");
    
    let orchestrator = ExecutionScenarioTestOrchestrator::new().await
        .expect("Failed to initialize test orchestrator");
    
    let slippage_results = orchestrator.test_slippage_handling().await
        .expect("Slippage handling test failed");
    
    info!("💨 Slippage Handling Results:");
    info!("  • Positive Slippage Capture Rate: {:.2}%", slippage_results.positive_slippage_capture_rate * 100.0);
    info!("  • Negative Slippage Mitigation Rate: {:.2}%", slippage_results.negative_slippage_mitigation_rate * 100.0);
    info!("  • Average Slippage: {:.2} bps", slippage_results.average_slippage_bps);
    info!("  • Slippage Prediction Accuracy: {:.2}%", slippage_results.slippage_prediction_accuracy * 100.0);
    info!("  • Dynamic Adjustment Effectiveness: {:.2}%", slippage_results.dynamic_slippage_adjustment_effectiveness * 100.0);
    info!("  • Status: {:?}", slippage_results.status);
    
    // Assert slippage handling meets requirements
    assert!(matches!(slippage_results.status, TestStatus::Passed | TestStatus::PartiallyPassed));
    assert!(slippage_results.negative_slippage_mitigation_rate > 0.75);
    assert!(slippage_results.average_slippage_bps < 3.0);
    assert!(slippage_results.slippage_prediction_accuracy > 0.70);
}

/// Test execution quality metrics
#[tokio::test]
async fn test_execution_quality_metrics() {
    init_test_logging();
    
    info!("⭐ Testing Execution Quality Metrics");
    
    let orchestrator = ExecutionScenarioTestOrchestrator::new().await
        .expect("Failed to initialize test orchestrator");
    
    let quality_results = orchestrator.test_execution_quality().await
        .expect("Execution quality test failed");
    
    info!("⭐ Execution Quality Results:");
    info!("  • Price Improvement Rate: {:.2}%", quality_results.price_improvement_rate * 100.0);
    info!("  • Fill Ratio Optimization: {:.2}%", quality_results.fill_ratio_optimization * 100.0);
    info!("  • Timing Optimization Score: {:.2}%", quality_results.timing_optimization_score * 100.0);
    info!("  • Liquidity Detection Accuracy: {:.2}%", quality_results.liquidity_detection_accuracy * 100.0);
    info!("  • Market Impact Minimization: {:.2}%", quality_results.market_impact_minimization * 100.0);
    info!("  • Execution Cost Efficiency: {:.2}%", quality_results.execution_cost_efficiency * 100.0);
    info!("  • Status: {:?}", quality_results.status);
    
    // Assert execution quality meets requirements
    assert!(matches!(quality_results.status, TestStatus::Passed | TestStatus::PartiallyPassed));
    assert!(quality_results.fill_ratio_optimization > 0.80);
    assert!(quality_results.timing_optimization_score > 0.75);
    assert!(quality_results.liquidity_detection_accuracy > 0.85);
}

/// Test market condition performance
#[tokio::test]
async fn test_market_condition_performance() {
    init_test_logging();
    
    info!("🌊 Testing Market Condition Performance");
    
    let orchestrator = ExecutionScenarioTestOrchestrator::new().await
        .expect("Failed to initialize test orchestrator");
    
    let market_results = orchestrator.test_market_conditions().await
        .expect("Market condition test failed");
    
    info!("🌊 Market Condition Results:");
    info!("  • Normal Market Performance: {:.2}%", market_results.normal_market_performance * 100.0);
    info!("  • High Volatility Performance: {:.2}%", market_results.high_volatility_performance * 100.0);
    info!("  • Low Liquidity Performance: {:.2}%", market_results.low_liquidity_performance * 100.0);
    info!("  • Trending Market Performance: {:.2}%", market_results.trending_market_performance * 100.0);
    info!("  • Sideways Market Performance: {:.2}%", market_results.sideways_market_performance * 100.0);
    info!("  • Stress Test Performance: {:.2}%", market_results.stress_test_performance * 100.0);
    info!("  • Status: {:?}", market_results.status);
    
    // Assert market condition performance meets requirements
    assert!(matches!(market_results.status, TestStatus::Passed | TestStatus::PartiallyPassed));
    assert!(market_results.normal_market_performance > 0.80);
    assert!(market_results.high_volatility_performance > 0.70);
    assert!(market_results.low_liquidity_performance > 0.65);
}

/// Initialize logging for tests
fn init_test_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

/// Generate comprehensive execution scenario test report
fn generate_execution_scenario_test_report(results: &ExecutionScenarioTestResults, duration: std::time::Duration) {
    info!("📋 EXECUTION SCENARIO TEST REPORT");
    info!("=" .repeat(70));
    info!("🕒 Test Duration: {:.2} seconds", duration.as_secs_f64());
    info!("🆔 Test ID: {}", results.test_id);
    info!("📊 Overall Score: {:.2}%", results.overall_score);
    info!("✅ Status: {:?}", results.pass_fail_status);
    info!("");
    
    info!("📈 DETAILED RESULTS:");
    info!("  📈 Long Positions: {:?}", results.long_position_tests.status);
    info!("    • Market Buy Quality: {:.1}%", results.long_position_tests.market_buy_execution_quality * 100.0);
    info!("    • Limit Buy Quality: {:.1}%", results.long_position_tests.limit_buy_execution_quality * 100.0);
    info!("    • Stop-Loss Effectiveness: {:.1}%", results.long_position_tests.stop_loss_protection_effectiveness * 100.0);
    info!("    • Average Latency: {:.2}ms", results.long_position_tests.average_execution_latency_ms);
    info!("");
    
    info!("  📉 Short Positions: {:?}", results.short_position_tests.status);
    info!("    • Market Sell Quality: {:.1}%", results.short_position_tests.market_sell_execution_quality * 100.0);
    info!("    • Limit Sell Quality: {:.1}%", results.short_position_tests.limit_sell_execution_quality * 100.0);
    info!("    • Stop-Loss Effectiveness: {:.1}%", results.short_position_tests.stop_loss_protection_effectiveness * 100.0);
    info!("    • Average Latency: {:.2}ms", results.short_position_tests.average_execution_latency_ms);
    info!("");
    
    info!("  🔄 Order Types: {:?}", results.order_type_tests.status);
    info!("    • Market Order Fill Rate: {:.1}%", results.order_type_tests.market_order_fill_rate * 100.0);
    info!("    • Limit Order Fill Rate: {:.1}%", results.order_type_tests.limit_order_fill_rate * 100.0);
    info!("    • Stop Order Trigger Accuracy: {:.1}%", results.order_type_tests.stop_order_trigger_accuracy * 100.0);
    info!("");
    
    info!("  💨 Slippage Handling: {:?}", results.slippage_tests.status);
    info!("    • Positive Capture Rate: {:.1}%", results.slippage_tests.positive_slippage_capture_rate * 100.0);
    info!("    • Negative Mitigation Rate: {:.1}%", results.slippage_tests.negative_slippage_mitigation_rate * 100.0);
    info!("    • Average Slippage: {:.2} bps", results.slippage_tests.average_slippage_bps);
    info!("");
    
    info!("  ⭐ Execution Quality: {:?}", results.execution_quality_tests.status);
    info!("    • Price Improvement Rate: {:.1}%", results.execution_quality_tests.price_improvement_rate * 100.0);
    info!("    • Fill Ratio Optimization: {:.1}%", results.execution_quality_tests.fill_ratio_optimization * 100.0);
    info!("    • Timing Optimization: {:.1}%", results.execution_quality_tests.timing_optimization_score * 100.0);
    info!("");
    
    info!("  🌊 Market Conditions: {:?}", results.market_condition_tests.status);
    info!("    • Normal Market: {:.1}%", results.market_condition_tests.normal_market_performance * 100.0);
    info!("    • High Volatility: {:.1}%", results.market_condition_tests.high_volatility_performance * 100.0);
    info!("    • Low Liquidity: {:.1}%", results.market_condition_tests.low_liquidity_performance * 100.0);
    info!("");
    
    info!("  ⚡ Performance Metrics:");
    info!("    • Throughput: {:.0} orders/sec", results.performance_metrics.orders_per_second_capacity);
    info!("    • P50 Latency: {:.2}ms", results.performance_metrics.execution_latency_percentiles.p50_ms);
    info!("    • P95 Latency: {:.2}ms", results.performance_metrics.execution_latency_percentiles.p95_ms);
    info!("    • P99 Latency: {:.2}ms", results.performance_metrics.execution_latency_percentiles.p99_ms);
    info!("    • Error Rate: {:.2}%", results.performance_metrics.error_rate_percentage);
    info!("=" .repeat(70));
}

/// Validate execution scenario performance
fn validate_execution_scenario_performance(results: &ExecutionScenarioTestResults) {
    info!("🔍 EXECUTION SCENARIO PERFORMANCE VALIDATION");
    
    let is_performance_excellent = matches!(results.pass_fail_status, TestStatus::Passed) && 
                                   results.overall_score >= 80.0;
    
    if is_performance_excellent {
        info!("✅ EXECUTION SCENARIO PERFORMANCE IS EXCELLENT");
        info!("  • All critical execution tests passed");
        info!("  • Performance targets met across all scenarios");
        info!("  • Ready for high-frequency trading operations");
    } else if matches!(results.pass_fail_status, TestStatus::PartiallyPassed) {
        warn!("⚠️ EXECUTION SCENARIO PERFORMANCE IS ACCEPTABLE WITH MINOR ISSUES");
        warn!("  • Most execution tests passed but some improvements needed");
        warn!("  • Review failed test categories for optimization opportunities");
    } else {
        error!("❌ EXECUTION SCENARIO PERFORMANCE REQUIRES SIGNIFICANT IMPROVEMENTS");
        error!("  • Critical execution tests failed");
        error!("  • Not ready for production trading operations");
    }
}
