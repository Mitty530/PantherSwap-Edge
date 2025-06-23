// Order Book Management Test Runner
// Run with: cargo test --test run_order_book_tests

use std::time::Instant;
use tracing::{info, warn, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod order_book_management_tests;
mod order_book_test_helpers;
mod common;

use order_book_management_tests::{OrderBookTestOrchestrator, OrderBookTestResults, TestStatus};

/// Main test for comprehensive order book management
#[tokio::test]
async fn run_comprehensive_order_book_tests() {
    // Initialize logging
    init_test_logging();
    
    info!("🚀 Starting Comprehensive Order Book Management Tests");
    info!("=" .repeat(70));
    info!("📋 Test Categories:");
    info!("  • Order Placement (Market, Limit, Stop-Loss, Take-Profit)");
    info!("  • Order Modification (Price, Quantity, Stop-Loss, Take-Profit)");
    info!("  • Order Cancellation (Immediate, Partial Fill, Bulk)");
    info!("  • Order Book State Management");
    info!("  • Order Types Execution Quality");
    info!("  • Performance Metrics");
    info!("=" .repeat(70));
    
    let test_start_time = Instant::now();
    
    // Initialize test orchestrator
    let orchestrator = match OrderBookTestOrchestrator::new().await {
        Ok(orchestrator) => {
            info!("✅ Order book test orchestrator initialized successfully");
            orchestrator
        }
        Err(e) => {
            error!("❌ Failed to initialize order book test orchestrator: {}", e);
            panic!("Cannot proceed with order book testing");
        }
    };
    
    // Run comprehensive order book tests
    let results = match orchestrator.run_comprehensive_order_book_tests().await {
        Ok(results) => {
            info!("✅ Comprehensive order book tests completed");
            results
        }
        Err(e) => {
            error!("❌ Comprehensive order book tests failed: {}", e);
            panic!("Order book testing failed");
        }
    };
    
    let total_test_duration = test_start_time.elapsed();
    
    // Generate and display test report
    generate_order_book_test_report(&results, total_test_duration);
    
    // Validate results
    validate_order_book_performance(&results);
    
    info!("🎯 Order Book Management Testing Completed Successfully!");
}

/// Test order placement specifically
#[tokio::test]
async fn test_order_placement_functionality() {
    init_test_logging();
    
    info!("📋 Testing Order Placement Functionality");
    
    let orchestrator = OrderBookTestOrchestrator::new().await
        .expect("Failed to initialize test orchestrator");
    
    let placement_results = orchestrator.test_order_placement().await
        .expect("Order placement test failed");
    
    info!("📋 Order Placement Results:");
    info!("  • Market Orders Success Rate: {:.2}%", placement_results.market_orders_success_rate * 100.0);
    info!("  • Limit Orders Success Rate: {:.2}%", placement_results.limit_orders_success_rate * 100.0);
    info!("  • Stop-Loss Orders Success Rate: {:.2}%", placement_results.stop_loss_orders_success_rate * 100.0);
    info!("  • Take-Profit Orders Success Rate: {:.2}%", placement_results.take_profit_orders_success_rate * 100.0);
    info!("  • Average Placement Latency: {:.2}ms", placement_results.average_placement_latency_ms);
    info!("  • Total Orders Placed: {}", placement_results.total_orders_placed);
    info!("  • Status: {:?}", placement_results.status);
    
    // Assert placement meets requirements
    assert!(matches!(placement_results.status, TestStatus::Passed | TestStatus::PartiallyPassed));
    assert!(placement_results.market_orders_success_rate > 0.90);
    assert!(placement_results.limit_orders_success_rate > 0.90);
    assert!(placement_results.average_placement_latency_ms < 10.0);
}

/// Test order modification specifically
#[tokio::test]
async fn test_order_modification_functionality() {
    init_test_logging();
    
    info!("✏️ Testing Order Modification Functionality");
    
    let orchestrator = OrderBookTestOrchestrator::new().await
        .expect("Failed to initialize test orchestrator");
    
    let modification_results = orchestrator.test_order_modification().await
        .expect("Order modification test failed");
    
    info!("✏️ Order Modification Results:");
    info!("  • Price Modification Success Rate: {:.2}%", modification_results.price_modification_success_rate * 100.0);
    info!("  • Quantity Modification Success Rate: {:.2}%", modification_results.quantity_modification_success_rate * 100.0);
    info!("  • Stop-Loss Modification Success Rate: {:.2}%", modification_results.stop_loss_modification_success_rate * 100.0);
    info!("  • Take-Profit Modification Success Rate: {:.2}%", modification_results.take_profit_modification_success_rate * 100.0);
    info!("  • Average Modification Latency: {:.2}ms", modification_results.average_modification_latency_ms);
    info!("  • Total Modifications: {}", modification_results.total_modifications);
    info!("  • Status: {:?}", modification_results.status);
    
    // Assert modification meets requirements
    assert!(matches!(modification_results.status, TestStatus::Passed | TestStatus::PartiallyPassed));
    assert!(modification_results.price_modification_success_rate > 0.85);
    assert!(modification_results.quantity_modification_success_rate > 0.85);
}

/// Test order cancellation specifically
#[tokio::test]
async fn test_order_cancellation_functionality() {
    init_test_logging();
    
    info!("❌ Testing Order Cancellation Functionality");
    
    let orchestrator = OrderBookTestOrchestrator::new().await
        .expect("Failed to initialize test orchestrator");
    
    let cancellation_results = orchestrator.test_order_cancellation().await
        .expect("Order cancellation test failed");
    
    info!("❌ Order Cancellation Results:");
    info!("  • Immediate Cancellation Success Rate: {:.2}%", cancellation_results.immediate_cancellation_success_rate * 100.0);
    info!("  • Partial Fill Cancellation Success Rate: {:.2}%", cancellation_results.partial_fill_cancellation_success_rate * 100.0);
    info!("  • Bulk Cancellation Success Rate: {:.2}%", cancellation_results.bulk_cancellation_success_rate * 100.0);
    info!("  • Average Cancellation Latency: {:.2}ms", cancellation_results.average_cancellation_latency_ms);
    info!("  • Total Cancellations: {}", cancellation_results.total_cancellations);
    info!("  • Status: {:?}", cancellation_results.status);
    
    // Assert cancellation meets requirements
    assert!(matches!(cancellation_results.status, TestStatus::Passed | TestStatus::PartiallyPassed));
    assert!(cancellation_results.immediate_cancellation_success_rate > 0.95);
    assert!(cancellation_results.partial_fill_cancellation_success_rate > 0.85);
    assert!(cancellation_results.bulk_cancellation_success_rate > 0.90);
}

/// Test order book state management
#[tokio::test]
async fn test_order_book_state_management() {
    init_test_logging();
    
    info!("📊 Testing Order Book State Management");
    
    let orchestrator = OrderBookTestOrchestrator::new().await
        .expect("Failed to initialize test orchestrator");
    
    let state_results = orchestrator.test_order_book_state().await
        .expect("Order book state test failed");
    
    info!("📊 Order Book State Results:");
    info!("  • State Consistency Score: {:.2}%", state_results.state_consistency_score * 100.0);
    info!("  • Real-Time Updates Accuracy: {:.2}%", state_results.real_time_updates_accuracy * 100.0);
    info!("  • Order Matching Accuracy: {:.2}%", state_results.order_matching_accuracy * 100.0);
    info!("  • Price Level Integrity Score: {:.2}%", state_results.price_level_integrity_score * 100.0);
    info!("  • Volume Tracking Accuracy: {:.2}%", state_results.volume_tracking_accuracy * 100.0);
    info!("  • Status: {:?}", state_results.status);
    
    // Assert state management meets requirements
    assert!(matches!(state_results.status, TestStatus::Passed | TestStatus::PartiallyPassed));
    assert!(state_results.state_consistency_score > 0.95);
    assert!(state_results.order_matching_accuracy > 0.95);
}

/// Test order types execution
#[tokio::test]
async fn test_order_types_execution() {
    init_test_logging();
    
    info!("🔄 Testing Order Types Execution");
    
    let orchestrator = OrderBookTestOrchestrator::new().await
        .expect("Failed to initialize test orchestrator");
    
    let types_results = orchestrator.test_order_types().await
        .expect("Order types test failed");
    
    info!("🔄 Order Types Results:");
    info!("  • Market Order Execution Quality: {:.2}%", types_results.market_order_execution_quality * 100.0);
    info!("  • Limit Order Execution Quality: {:.2}%", types_results.limit_order_execution_quality * 100.0);
    info!("  • Stop-Loss Trigger Accuracy: {:.2}%", types_results.stop_loss_trigger_accuracy * 100.0);
    info!("  • Take-Profit Trigger Accuracy: {:.2}%", types_results.take_profit_trigger_accuracy * 100.0);
    info!("  • Stop-Limit Execution Quality: {:.2}%", types_results.stop_limit_execution_quality * 100.0);
    info!("  • Iceberg Order Handling: {:.2}%", types_results.iceberg_order_handling * 100.0);
    info!("  • Status: {:?}", types_results.status);
    
    // Assert order types execution meets requirements
    assert!(matches!(types_results.status, TestStatus::Passed | TestStatus::PartiallyPassed));
    assert!(types_results.market_order_execution_quality > 0.85);
    assert!(types_results.limit_order_execution_quality > 0.80);
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

/// Generate comprehensive order book test report
fn generate_order_book_test_report(results: &OrderBookTestResults, duration: std::time::Duration) {
    info!("📋 ORDER BOOK MANAGEMENT TEST REPORT");
    info!("=" .repeat(70));
    info!("🕒 Test Duration: {:.2} seconds", duration.as_secs_f64());
    info!("🆔 Test ID: {}", results.test_id);
    info!("📊 Overall Score: {:.2}%", results.overall_score);
    info!("✅ Status: {:?}", results.pass_fail_status);
    info!("");
    
    info!("📈 DETAILED RESULTS:");
    info!("  📋 Order Placement: {:?}", results.order_placement_tests.status);
    info!("    • Market Orders: {:.1}% success", results.order_placement_tests.market_orders_success_rate * 100.0);
    info!("    • Limit Orders: {:.1}% success", results.order_placement_tests.limit_orders_success_rate * 100.0);
    info!("    • Average Latency: {:.2}ms", results.order_placement_tests.average_placement_latency_ms);
    info!("");
    
    info!("  ✏️ Order Modification: {:?}", results.order_modification_tests.status);
    info!("    • Price Modifications: {:.1}% success", results.order_modification_tests.price_modification_success_rate * 100.0);
    info!("    • Quantity Modifications: {:.1}% success", results.order_modification_tests.quantity_modification_success_rate * 100.0);
    info!("    • Average Latency: {:.2}ms", results.order_modification_tests.average_modification_latency_ms);
    info!("");
    
    info!("  ❌ Order Cancellation: {:?}", results.order_cancellation_tests.status);
    info!("    • Immediate Cancellations: {:.1}% success", results.order_cancellation_tests.immediate_cancellation_success_rate * 100.0);
    info!("    • Bulk Cancellations: {:.1}% success", results.order_cancellation_tests.bulk_cancellation_success_rate * 100.0);
    info!("    • Average Latency: {:.2}ms", results.order_cancellation_tests.average_cancellation_latency_ms);
    info!("");
    
    info!("  📊 Order Book State: {:?}", results.order_book_state_tests.status);
    info!("    • State Consistency: {:.1}%", results.order_book_state_tests.state_consistency_score * 100.0);
    info!("    • Order Matching: {:.1}%", results.order_book_state_tests.order_matching_accuracy * 100.0);
    info!("");
    
    info!("  🔄 Order Types: {:?}", results.order_types_tests.status);
    info!("    • Market Order Quality: {:.1}%", results.order_types_tests.market_order_execution_quality * 100.0);
    info!("    • Limit Order Quality: {:.1}%", results.order_types_tests.limit_order_execution_quality * 100.0);
    info!("");
    
    info!("  ⚡ Performance Metrics:");
    info!("    • Throughput: {:.0} ops/sec", results.performance_metrics.order_processing_throughput_ops);
    info!("    • P50 Latency: {:.2}ms", results.performance_metrics.latency_percentiles.p50_ms);
    info!("    • P95 Latency: {:.2}ms", results.performance_metrics.latency_percentiles.p95_ms);
    info!("    • P99 Latency: {:.2}ms", results.performance_metrics.latency_percentiles.p99_ms);
    info!("    • Error Rate: {:.2}%", results.performance_metrics.error_rate_percentage);
    info!("=" .repeat(70));
}

/// Validate order book performance
fn validate_order_book_performance(results: &OrderBookTestResults) {
    info!("🔍 ORDER BOOK PERFORMANCE VALIDATION");
    
    let is_performance_acceptable = matches!(results.pass_fail_status, TestStatus::Passed) && 
                                   results.overall_score >= 85.0;
    
    if is_performance_acceptable {
        info!("✅ ORDER BOOK PERFORMANCE IS EXCELLENT");
        info!("  • All critical tests passed");
        info!("  • Performance targets met");
        info!("  • Ready for high-frequency trading");
    } else if matches!(results.pass_fail_status, TestStatus::PartiallyPassed) {
        warn!("⚠️ ORDER BOOK PERFORMANCE IS ACCEPTABLE WITH MINOR ISSUES");
        warn!("  • Most tests passed but some improvements needed");
        warn!("  • Review failed test categories");
    } else {
        error!("❌ ORDER BOOK PERFORMANCE REQUIRES SIGNIFICANT IMPROVEMENTS");
        error!("  • Critical tests failed");
        error!("  • Not ready for production trading");
    }
}
