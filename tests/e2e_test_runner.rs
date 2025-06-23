// End-to-End Test Runner for PantherSwap Edge
// Comprehensive testing with real Alpha Vantage market data
// Run with: cargo test --test e2e_test_runner

use std::time::{Duration, Instant};
use std::env;
use tokio::time::sleep;
use tracing::{info, warn, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use serde_json;

mod e2e_comprehensive_test;
mod e2e_test_implementations;
mod common;

use e2e_comprehensive_test::{E2ETestOrchestrator, E2ETestResults, TestStatus};

/// Main end-to-end test that runs the comprehensive testing campaign
#[tokio::test]
async fn run_comprehensive_e2e_testing_campaign() {
    // Initialize logging
    init_test_logging();
    
    info!("🚀 Starting PantherSwap Edge Comprehensive End-to-End Testing Campaign");
    info!("=" .repeat(80));
    info!("📋 Test Configuration:");
    info!("  • Real Alpha Vantage API Integration (Key: EZDZ4VOFQ2GRA7VU)");
    info!("  • TimescaleDB Production Database");
    info!("  • Autonomous Trading Operations");
    info!("  • Performance Benchmarking (<10ms execution, <100ms AI inference, >1000 TPS)");
    info!("  • Competitive Analysis vs Industry Standards");
    info!("=" .repeat(80));
    
    // Set environment for testing
    env::set_var("RUN_MODE", "e2e_testing");
    env::set_var("PANTHERSWAP_MARKET_DATA_ALPHA_VANTAGE_API_KEY", "EZDZ4VOFQ2GRA7VU");
    env::set_var("RUST_LOG", "info");
    
    let test_start_time = Instant::now();
    
    // Initialize test orchestrator
    let orchestrator = match E2ETestOrchestrator::new().await {
        Ok(orchestrator) => {
            info!("✅ Test orchestrator initialized successfully");
            orchestrator
        }
        Err(e) => {
            error!("❌ Failed to initialize test orchestrator: {}", e);
            panic!("Cannot proceed with testing");
        }
    };
    
    // Run comprehensive testing campaign
    let results = match orchestrator.run_comprehensive_test().await {
        Ok(results) => {
            info!("✅ Comprehensive testing campaign completed");
            results
        }
        Err(e) => {
            error!("❌ Comprehensive testing campaign failed: {}", e);
            panic!("Testing campaign failed");
        }
    };
    
    let total_test_duration = test_start_time.elapsed();
    
    // Generate and display comprehensive test report
    generate_comprehensive_test_report(&results, total_test_duration);
    
    // Validate results against production readiness criteria
    validate_production_readiness(&results);
    
    // Save results to file for analysis
    save_test_results_to_file(&results).await;
    
    info!("🎯 End-to-End Testing Campaign Completed Successfully!");
}

/// Test autonomous trading operations specifically
#[tokio::test]
async fn test_autonomous_trading_operations() {
    init_test_logging();
    
    info!("🤖 Testing Autonomous Trading Operations");
    
    env::set_var("RUN_MODE", "e2e_testing");
    env::set_var("PANTHERSWAP_MARKET_DATA_ALPHA_VANTAGE_API_KEY", "EZDZ4VOFQ2GRA7VU");
    
    let orchestrator = E2ETestOrchestrator::new().await
        .expect("Failed to initialize test orchestrator");
    
    // Test autonomous trading specifically
    let autonomous_results = orchestrator.test_autonomous_trading().await
        .expect("Autonomous trading test failed");
    
    info!("🤖 Autonomous Trading Results:");
    info!("  • AI Signal Generation Success Rate: {:.2}%", autonomous_results.ai_signal_generation_success_rate * 100.0);
    info!("  • Autonomous Order Execution Count: {}", autonomous_results.autonomous_order_execution_count);
    info!("  • Decision Accuracy: {:.2}%", autonomous_results.autonomous_decision_accuracy * 100.0);
    info!("  • Portfolio Management Effectiveness: {:.2}%", autonomous_results.portfolio_management_effectiveness * 100.0);
    info!("  • Real-time Adaptation Score: {:.2}%", autonomous_results.real_time_adaptation_score * 100.0);
    info!("  • Status: {:?}", autonomous_results.status);
    
    // Assert autonomous trading meets requirements
    assert!(matches!(autonomous_results.status, TestStatus::Passed | TestStatus::PartiallyPassed));
    assert!(autonomous_results.ai_signal_generation_success_rate > 0.7);
    assert!(autonomous_results.autonomous_decision_accuracy > 0.6);
}

/// Test real-time market data integration with Alpha Vantage
#[tokio::test]
async fn test_real_time_market_data_integration() {
    init_test_logging();
    
    info!("📊 Testing Real-Time Market Data Integration");
    
    env::set_var("RUN_MODE", "e2e_testing");
    env::set_var("PANTHERSWAP_MARKET_DATA_ALPHA_VANTAGE_API_KEY", "EZDZ4VOFQ2GRA7VU");
    
    let orchestrator = E2ETestOrchestrator::new().await
        .expect("Failed to initialize test orchestrator");
    
    // Test market data integration
    let market_data_results = orchestrator.test_market_data_integration().await
        .expect("Market data integration test failed");
    
    info!("📊 Market Data Integration Results:");
    info!("  • Alpha Vantage Connectivity Score: {:.2}%", market_data_results.alpha_vantage_connectivity_score * 100.0);
    info!("  • Data Quality Score: {:.2}%", market_data_results.data_quality_score * 100.0);
    info!("  • Real-time Processing Latency: {:.2}ms", market_data_results.real_time_processing_latency_ms);
    info!("  • Data Consistency Score: {:.2}%", market_data_results.data_consistency_score * 100.0);
    info!("  • Pipeline Reliability Score: {:.2}%", market_data_results.pipeline_reliability_score * 100.0);
    info!("  • Status: {:?}", market_data_results.status);
    
    // Assert market data integration meets requirements
    assert!(matches!(market_data_results.status, TestStatus::Passed | TestStatus::PartiallyPassed));
    assert!(market_data_results.alpha_vantage_connectivity_score > 0.8);
    assert!(market_data_results.real_time_processing_latency_ms < 1000.0);
}

/// Test performance benchmarks against targets
#[tokio::test]
async fn test_performance_benchmarks() {
    init_test_logging();
    
    info!("🏃 Testing Performance Benchmarks");
    
    env::set_var("RUN_MODE", "e2e_testing");
    env::set_var("PANTHERSWAP_MARKET_DATA_ALPHA_VANTAGE_API_KEY", "EZDZ4VOFQ2GRA7VU");
    
    let orchestrator = E2ETestOrchestrator::new().await
        .expect("Failed to initialize test orchestrator");
    
    // Test performance benchmarks
    let performance_results = orchestrator.test_performance_benchmarks().await
        .expect("Performance benchmark test failed");
    
    info!("🏃 Performance Benchmark Results:");
    info!("  • Order Execution Latency: {:.2}ms (Target: <10ms)", performance_results.order_execution_latency_ms);
    info!("  • AI Inference Latency: {:.2}ms (Target: <100ms)", performance_results.ai_inference_latency_ms);
    info!("  • Throughput: {:.2} TPS (Target: >1000 TPS)", performance_results.throughput_tps);
    info!("  • Uptime: {:.2}% (Target: >99.9%)", performance_results.uptime_percentage);
    info!("  • Error Rate: {:.2}% (Target: <0.1%)", performance_results.error_rate_percentage);
    info!("  • Meets Performance Targets: {}", performance_results.meets_performance_targets);
    info!("  • Status: {:?}", performance_results.status);
    
    // Assert performance meets targets
    assert!(performance_results.order_execution_latency_ms < 10.0, 
           "Order execution latency {} exceeds 10ms target", performance_results.order_execution_latency_ms);
    assert!(performance_results.ai_inference_latency_ms < 100.0, 
           "AI inference latency {} exceeds 100ms target", performance_results.ai_inference_latency_ms);
    assert!(performance_results.throughput_tps > 1000.0, 
           "Throughput {} below 1000 TPS target", performance_results.throughput_tps);
}

/// Test competitive analysis
#[tokio::test]
async fn test_competitive_analysis() {
    init_test_logging();
    
    info!("🏆 Testing Competitive Analysis");
    
    env::set_var("RUN_MODE", "e2e_testing");
    env::set_var("PANTHERSWAP_MARKET_DATA_ALPHA_VANTAGE_API_KEY", "EZDZ4VOFQ2GRA7VU");
    
    let orchestrator = E2ETestOrchestrator::new().await
        .expect("Failed to initialize test orchestrator");
    
    // Test competitive analysis
    let competitive_results = orchestrator.test_competitive_analysis().await
        .expect("Competitive analysis test failed");
    
    info!("🏆 Competitive Analysis Results:");
    info!("  • Execution Speed vs Industry: {:.2}x", competitive_results.execution_speed_vs_industry);
    info!("  • Trading Accuracy vs Industry: {:.2}x", competitive_results.trading_accuracy_vs_industry);
    info!("  • Risk Management vs Industry: {:.2}x", competitive_results.risk_management_vs_industry);
    info!("  • Profitability vs Industry: {:.2}x", competitive_results.profitability_vs_industry);
    info!("  • Overall Competitive Score: {:.2}%", competitive_results.overall_competitive_score * 100.0);
    info!("  • Industry Ranking Percentile: {:.1}%", competitive_results.industry_ranking_percentile);
    info!("  • Status: {:?}", competitive_results.status);
    
    // Assert competitive positioning
    assert!(competitive_results.overall_competitive_score > 0.7, 
           "Overall competitive score {} below 70% threshold", competitive_results.overall_competitive_score);
    assert!(competitive_results.industry_ranking_percentile > 50.0, 
           "Industry ranking percentile {} below 50%", competitive_results.industry_ranking_percentile);
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

/// Generate comprehensive test report
fn generate_comprehensive_test_report(results: &E2ETestResults, duration: Duration) {
    info!("📋 COMPREHENSIVE END-TO-END TEST REPORT");
    info!("=" .repeat(80));
    info!("🕒 Test Duration: {:.2} seconds", duration.as_secs_f64());
    info!("🆔 Test ID: {}", results.test_id);
    info!("📊 Overall Score: {:.2}%", results.overall_score);
    info!("✅ Status: {:?}", results.pass_fail_status);
    info!("");
    
    info!("📈 DETAILED RESULTS:");
    info!("  🤖 Autonomous Trading: {:?}", results.autonomous_trading.status);
    info!("  📋 Order Management: {:?}", results.order_management.status);
    info!("  ⚡ Execution Scenarios: {:?}", results.execution_scenarios.status);
    info!("  📊 Market Data Integration: {:?}", results.market_data_integration.status);
    info!("  🔧 Backend Integration: {:?}", results.backend_integration.status);
    info!("  🏃 Performance Benchmarks: {:?}", results.performance_benchmarks.status);
    info!("  📈 Trading Analytics: {:?}", results.trading_analytics.status);
    info!("  🛡️ System Reliability: {:?}", results.system_reliability.status);
    info!("  🏆 Competitive Analysis: {:?}", results.competitive_analysis.status);
    info!("");
    
    info!("💡 RECOMMENDATIONS:");
    for (i, recommendation) in results.recommendations.iter().enumerate() {
        info!("  {}. {}", i + 1, recommendation);
    }
    info!("=" .repeat(80));
}

/// Validate production readiness
fn validate_production_readiness(results: &E2ETestResults) {
    info!("🔍 PRODUCTION READINESS VALIDATION");
    
    let is_production_ready = matches!(results.pass_fail_status, TestStatus::Passed) && 
                             results.overall_score >= 80.0;
    
    if is_production_ready {
        info!("✅ SYSTEM IS PRODUCTION READY");
        info!("  • All critical tests passed");
        info!("  • Performance targets met");
        info!("  • Overall score above 80%");
    } else {
        warn!("⚠️ SYSTEM REQUIRES IMPROVEMENTS BEFORE PRODUCTION");
        warn!("  • Some critical tests failed or overall score below 80%");
        warn!("  • Review recommendations and address issues");
    }
}

/// Save test results to file
async fn save_test_results_to_file(results: &E2ETestResults) {
    let filename = format!("e2e_test_results_{}.json", results.test_id);
    
    match serde_json::to_string_pretty(results) {
        Ok(json_results) => {
            if let Err(e) = tokio::fs::write(&filename, json_results).await {
                warn!("Failed to save test results to file {}: {}", filename, e);
            } else {
                info!("📄 Test results saved to: {}", filename);
            }
        }
        Err(e) => {
            warn!("Failed to serialize test results: {}", e);
        }
    }
}
