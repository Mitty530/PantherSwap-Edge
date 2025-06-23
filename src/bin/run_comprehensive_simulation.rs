// Comprehensive Live Trading Simulation Runner
// Executes the full-system live trading simulation with IG Trading API

use anyhow::Result;
use std::time::Duration;
use tracing::{info, error, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use serde_json;

use pantherswap_edge::testing::comprehensive_live_simulation::{
    ComprehensiveLiveSimulator, 
    ComprehensiveLiveSimulationConfig,
    PerformanceTargets,
    IGTradingSimulationConfig,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "pantherswap_edge=info,run_comprehensive_simulation=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("🚀 Starting PantherSwap Edge Comprehensive Live Trading Simulation");
    info!("📊 This simulation will test the complete system with real IG Trading market data");

    // Create simulation configuration
    let config = ComprehensiveLiveSimulationConfig {
        initial_capital: 100_000.0,
        simulation_duration: Duration::from_secs(600), // 10 minutes
        target_symbols: vec![
            "AAPL".to_string(),
            "MSFT".to_string(), 
            "GOOGL".to_string(),
            "TSLA".to_string(),
            "NVDA".to_string(),
        ],
        max_positions: 5,
        risk_per_trade: 0.02, // 2% risk per trade
        enable_ai_trading: true,
        enable_performance_monitoring: true,
        performance_targets: PerformanceTargets {
            ai_inference_latency_ms: 100,      // <100ms target
            order_execution_latency_ms: 10,    // <10ms target
            system_throughput_tps: 1000,       // >1000 TPS target
            database_latency_ms: 10,           // <10ms target
            uptime_percentage: 99.9,           // >99.9% target
            ai_accuracy_threshold: 90.0,       // >90% target
        },
        ig_trading_config: IGTradingSimulationConfig {
            api_key: "3ded3ba7db96187488bf8773b86bdf3e8fc42e9b".to_string(),
            security_token: "1206a1630c34bcc90fdcc1b62fc5920fa7ed3a216ae09933430d3de2c6bcf6CD01112".to_string(),
            cst: "48417021199921da08b95b210d8f9492c36614232983a9f1f3e1a8f0748ce33CC01113".to_string(),
            demo_mode: true,
            rate_limit_per_minute: 100,
        },
    };

    info!("📋 Simulation Configuration:");
    info!("   💰 Initial Capital: ${:.2}", config.initial_capital);
    info!("   ⏱️  Duration: {} seconds", config.simulation_duration.as_secs());
    info!("   📈 Target Symbols: {:?}", config.target_symbols);
    info!("   🎯 Performance Targets:");
    info!("      - AI Inference: <{}ms", config.performance_targets.ai_inference_latency_ms);
    info!("      - Order Execution: <{}ms", config.performance_targets.order_execution_latency_ms);
    info!("      - System Throughput: >{}TPS", config.performance_targets.system_throughput_tps);
    info!("      - Database Latency: <{}ms", config.performance_targets.database_latency_ms);
    info!("      - Uptime: >{}%", config.performance_targets.uptime_percentage);
    info!("      - AI Accuracy: >{}%", config.performance_targets.ai_accuracy_threshold);

    // Initialize the comprehensive simulator
    info!("🔧 Initializing comprehensive live trading simulator...");
    let mut simulator = match ComprehensiveLiveSimulator::new(config).await {
        Ok(sim) => {
            info!("✅ Simulator initialized successfully");
            sim
        }
        Err(e) => {
            error!("❌ Failed to initialize simulator: {}", e);
            return Err(e.into());
        }
    };

    // Validate system components
    info!("🔍 Validating system components...");
    match simulator.validate_system_components().await {
        Ok(true) => {
            info!("✅ All system components validated successfully");
        }
        Ok(false) => {
            warn!("⚠️  Some system components failed validation - proceeding with caution");
        }
        Err(e) => {
            error!("❌ System validation failed: {}", e);
            return Err(e.into());
        }
    }

    // Initialize trading components
    info!("🚀 Initializing trading components...");
    if let Err(e) = simulator.initialize_trading_components().await {
        error!("❌ Failed to initialize trading components: {}", e);
        return Err(e.into());
    }
    info!("✅ Trading components initialized successfully");

    // Execute the comprehensive simulation
    info!("🎯 Starting comprehensive live trading simulation...");
    info!("📊 The simulation will run for {} minutes with real-time market data", 
          simulator.config.simulation_duration.as_secs() / 60);
    
    let simulation_result = match simulator.execute_simulation().await {
        Ok(report) => {
            info!("✅ Comprehensive simulation completed successfully!");
            report
        }
        Err(e) => {
            error!("❌ Simulation failed: {}", e);
            return Err(e.into());
        }
    };

    // Display comprehensive results
    info!("📊 COMPREHENSIVE SIMULATION RESULTS");
    info!("=====================================");
    
    // Execution Summary
    info!("📋 EXECUTION SUMMARY:");
    info!("   🆔 Simulation ID: {}", simulation_result.simulation_id);
    info!("   ⏱️  Duration: {:.2} seconds", simulation_result.execution_summary.duration_seconds);
    info!("   📈 Symbols Traded: {:?}", simulation_result.execution_summary.symbols_traded);
    info!("   🔢 Total Operations: {}", simulation_result.execution_summary.total_operations);
    info!("   ✅ Success Rate: {:.2}%", simulation_result.execution_summary.success_rate);
    info!("   📊 Overall Status: {}", simulation_result.execution_summary.overall_status);

    // Trading Performance
    info!("💰 TRADING PERFORMANCE:");
    info!("   💵 Total P&L: ${:.2}", simulation_result.trading_performance.total_pnl);
    info!("   📈 Return: {:.2}%", simulation_result.trading_performance.return_percentage);
    info!("   📉 Max Drawdown: {:.2}%", simulation_result.trading_performance.max_drawdown);
    info!("   📊 Sharpe Ratio: {:.2}", simulation_result.trading_performance.sharpe_ratio);
    info!("   🎯 Win Rate: {:.2}%", simulation_result.trading_performance.win_rate);
    info!("   ⚡ Avg Execution Latency: {:.2}ms", simulation_result.trading_performance.avg_execution_latency_ms);
    info!("   📊 Trades/Minute: {:.2}", simulation_result.trading_performance.trades_per_minute);

    // AI Performance
    info!("🤖 AI PERFORMANCE:");
    info!("   🔮 Total Predictions: {}", simulation_result.ai_performance.total_predictions);
    info!("   ⚡ Avg Inference Latency: {:.2}ms", simulation_result.ai_performance.avg_inference_latency_ms);
    info!("   🎯 Accuracy Score: {:.2}%", simulation_result.ai_performance.accuracy_score);
    info!("   🧠 Prediction Confidence: {:.2}%", simulation_result.ai_performance.prediction_confidence * 100.0);

    // System Performance
    info!("⚙️  SYSTEM PERFORMANCE:");
    info!("   🚀 Avg Throughput: {:.2} TPS", simulation_result.system_performance.avg_throughput_tps);
    info!("   🔥 Peak Throughput: {:.2} TPS", simulation_result.system_performance.peak_throughput_tps);
    info!("   💻 Avg CPU: {:.2}%", simulation_result.system_performance.avg_cpu_utilization);
    info!("   🧠 Peak Memory: {} MB", simulation_result.system_performance.peak_memory_usage_mb);
    info!("   ⚠️  Error Rate: {:.2}%", simulation_result.system_performance.error_rate);
    info!("   ⏰ Uptime: {:.2}%", simulation_result.system_performance.uptime_percentage);

    // Database Performance
    info!("🗄️  DATABASE PERFORMANCE:");
    info!("   📊 Total Operations: {}", simulation_result.database_performance.total_operations);
    info!("   ⚡ Avg Latency: {:.2}ms", simulation_result.database_performance.avg_latency_ms);
    info!("   🔥 Max Latency: {:.2}ms", simulation_result.database_performance.max_latency_ms);

    // Market Data Performance
    info!("📡 MARKET DATA PERFORMANCE:");
    info!("   📊 Total Updates: {}", simulation_result.market_data_performance.total_updates);
    info!("   ⚡ Avg Latency: {:.2}ms", simulation_result.market_data_performance.avg_latency_ms);
    info!("   🎯 Data Quality: {:.2}%", simulation_result.market_data_performance.data_quality_score);
    info!("   ✅ API Success Rate: {:.2}%", simulation_result.market_data_performance.api_success_rate);

    // Performance Validation
    info!("🎯 PERFORMANCE VALIDATION:");
    info!("   🤖 AI Inference Target Met: {}", if simulation_result.performance_validation.ai_inference_target_met { "✅" } else { "❌" });
    info!("   ⚡ Execution Latency Target Met: {}", if simulation_result.performance_validation.execution_latency_target_met { "✅" } else { "❌" });
    info!("   🚀 Throughput Target Met: {}", if simulation_result.performance_validation.throughput_target_met { "✅" } else { "❌" });
    info!("   🗄️  Database Latency Target Met: {}", if simulation_result.performance_validation.database_latency_target_met { "✅" } else { "❌" });
    info!("   ⏰ Uptime Target Met: {}", if simulation_result.performance_validation.uptime_target_met { "✅" } else { "❌" });
    info!("   🎯 AI Accuracy Target Met: {}", if simulation_result.performance_validation.ai_accuracy_target_met { "✅" } else { "❌" });
    info!("   🏆 OVERALL TARGETS MET: {}", if simulation_result.performance_validation.overall_targets_met { "✅ YES" } else { "❌ NO" });

    // Production Readiness Score
    info!("🏆 PRODUCTION READINESS SCORE: {:.1}/100", simulation_result.production_readiness_score);

    // Recommendations
    info!("💡 RECOMMENDATIONS:");
    for (i, recommendation) in simulation_result.recommendations.iter().enumerate() {
        info!("   {}. {}", i + 1, recommendation);
    }

    // Save detailed report to file
    let report_json = serde_json::to_string_pretty(&simulation_result)?;
    let report_filename = format!("comprehensive_simulation_report_{}.json", simulation_result.simulation_id);
    std::fs::write(&report_filename, report_json)?;
    info!("📄 Detailed report saved to: {}", report_filename);

    // Final summary
    if simulation_result.performance_validation.overall_targets_met {
        info!("🎉 SIMULATION SUCCESS: All performance targets met!");
        info!("🚀 System is ready for production deployment");
    } else {
        warn!("⚠️  SIMULATION PARTIAL SUCCESS: Some targets not met");
        warn!("🔧 Review recommendations before production deployment");
    }

    info!("✅ Comprehensive live trading simulation completed successfully!");
    
    Ok(())
}
