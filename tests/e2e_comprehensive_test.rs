// Comprehensive End-to-End Testing Framework for PantherSwap Edge
// Tests autonomous trading operations with real Alpha Vantage market data

use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio::time::sleep;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use tracing::{info, warn, error, debug};

use pantherswap_edge::config::Settings;
use pantherswap_edge::database::Database;
use pantherswap_edge::market_data::MarketDataManager;
use pantherswap_edge::trading::{TradingEngine, TradingEngineConfig};
use pantherswap_edge::ai::AIEngine;
use pantherswap_edge::api::{AppState, create_app};

mod common;
use common::*;

/// Comprehensive end-to-end test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E2ETestResults {
    pub test_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration_seconds: f64,
    pub autonomous_trading: AutonomousTradingResults,
    pub order_management: OrderManagementResults,
    pub execution_scenarios: ExecutionScenariosResults,
    pub market_data_integration: MarketDataIntegrationResults,
    pub backend_integration: BackendIntegrationResults,
    pub performance_benchmarks: PerformanceBenchmarkResults,
    pub trading_analytics: TradingAnalyticsResults,
    pub system_reliability: SystemReliabilityResults,
    pub competitive_analysis: CompetitiveAnalysisResults,
    pub overall_score: f64,
    pub pass_fail_status: TestStatus,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestStatus {
    Passed,
    Failed,
    PartiallyPassed,
    NotRun,
}

/// Autonomous trading test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomousTradingResults {
    pub ai_signal_generation_success_rate: f64,
    pub autonomous_order_execution_count: u64,
    pub autonomous_decision_accuracy: f64,
    pub portfolio_management_effectiveness: f64,
    pub real_time_adaptation_score: f64,
    pub status: TestStatus,
}

/// Order management test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderManagementResults {
    pub order_placement_success_rate: f64,
    pub order_modification_success_rate: f64,
    pub order_cancellation_success_rate: f64,
    pub order_book_consistency_score: f64,
    pub order_types_tested: Vec<String>,
    pub status: TestStatus,
}

/// Execution scenarios test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionScenariosResults {
    pub long_position_execution_quality: f64,
    pub short_position_execution_quality: f64,
    pub slippage_management_score: f64,
    pub execution_speed_score: f64,
    pub scenarios_tested: Vec<String>,
    pub status: TestStatus,
}

/// Market data integration test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataIntegrationResults {
    pub alpha_vantage_connectivity_score: f64,
    pub data_quality_score: f64,
    pub real_time_processing_latency_ms: f64,
    pub data_consistency_score: f64,
    pub pipeline_reliability_score: f64,
    pub status: TestStatus,
}

/// Backend integration test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendIntegrationResults {
    pub timescaledb_integration_score: f64,
    pub rest_api_integration_score: f64,
    pub trading_engine_integration_score: f64,
    pub ai_models_integration_score: f64,
    pub data_flow_consistency_score: f64,
    pub status: TestStatus,
}

/// Performance benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBenchmarkResults {
    pub order_execution_latency_ms: f64,
    pub ai_inference_latency_ms: f64,
    pub throughput_tps: f64,
    pub uptime_percentage: f64,
    pub error_rate_percentage: f64,
    pub meets_performance_targets: bool,
    pub status: TestStatus,
}

/// Trading analytics results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingAnalyticsResults {
    pub trading_accuracy_percentage: f64,
    pub profitability_score: f64,
    pub sharpe_ratio: f64,
    pub maximum_drawdown_percentage: f64,
    pub win_loss_ratio: f64,
    pub risk_adjusted_returns: f64,
    pub status: TestStatus,
}

/// System reliability results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemReliabilityResults {
    pub uptime_score: f64,
    pub error_recovery_time_ms: f64,
    pub data_consistency_under_load_score: f64,
    pub auto_recovery_effectiveness: f64,
    pub fault_tolerance_score: f64,
    pub status: TestStatus,
}

/// Competitive analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitiveAnalysisResults {
    pub execution_speed_vs_industry: f64,
    pub trading_accuracy_vs_industry: f64,
    pub risk_management_vs_industry: f64,
    pub profitability_vs_industry: f64,
    pub overall_competitive_score: f64,
    pub industry_ranking_percentile: f64,
    pub status: TestStatus,
}

/// Main end-to-end test orchestrator
pub struct E2ETestOrchestrator {
    settings: Settings,
    database: Database,
    test_id: Uuid,
    start_time: DateTime<Utc>,
}

impl E2ETestOrchestrator {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Load testing configuration
        std::env::set_var("RUN_MODE", "e2e_testing");
        let settings = Settings::load()?;
        
        // Initialize database with production settings
        let database = Database::new(&settings.database.url).await?;
        
        // Run migrations if needed
        database.run_manual_migrations().await?;
        
        Ok(Self {
            settings,
            database,
            test_id: Uuid::new_v4(),
            start_time: Utc::now(),
        })
    }

    /// Run comprehensive end-to-end testing campaign
    pub async fn run_comprehensive_test(&self) -> Result<E2ETestResults, Box<dyn std::error::Error>> {
        info!("🚀 Starting comprehensive end-to-end testing campaign");
        info!("Test ID: {}", self.test_id);
        info!("Using real Alpha Vantage API key: EZDZ4VOFQ2GRA7VU");
        
        let start_time = Instant::now();
        
        // Initialize all test results
        let mut results = E2ETestResults {
            test_id: self.test_id,
            start_time: self.start_time,
            end_time: Utc::now(),
            duration_seconds: 0.0,
            autonomous_trading: self.create_default_autonomous_results(),
            order_management: self.create_default_order_results(),
            execution_scenarios: self.create_default_execution_results(),
            market_data_integration: self.create_default_market_data_results(),
            backend_integration: self.create_default_backend_results(),
            performance_benchmarks: self.create_default_performance_results(),
            trading_analytics: self.create_default_analytics_results(),
            system_reliability: self.create_default_reliability_results(),
            competitive_analysis: self.create_default_competitive_results(),
            overall_score: 0.0,
            pass_fail_status: TestStatus::NotRun,
            recommendations: Vec::new(),
        };

        // Run all test phases
        info!("📊 Phase 1: Testing real market data integration...");
        results.market_data_integration = self.test_market_data_integration().await?;
        
        info!("🔧 Phase 2: Testing backend integration...");
        results.backend_integration = self.test_backend_integration().await?;
        
        info!("🤖 Phase 3: Testing autonomous trading operations...");
        results.autonomous_trading = self.test_autonomous_trading().await?;
        
        info!("📋 Phase 4: Testing order management...");
        results.order_management = self.test_order_management().await?;
        
        info!("⚡ Phase 5: Testing execution scenarios...");
        results.execution_scenarios = self.test_execution_scenarios().await?;
        
        info!("🏃 Phase 6: Running performance benchmarks...");
        results.performance_benchmarks = self.test_performance_benchmarks().await?;
        
        info!("📈 Phase 7: Analyzing trading performance...");
        results.trading_analytics = self.test_trading_analytics().await?;
        
        info!("🛡️ Phase 8: Testing system reliability...");
        results.system_reliability = self.test_system_reliability().await?;
        
        info!("🏆 Phase 9: Running competitive analysis...");
        results.competitive_analysis = self.test_competitive_analysis().await?;
        
        // Calculate final results
        results.end_time = Utc::now();
        results.duration_seconds = start_time.elapsed().as_secs_f64();
        results.overall_score = self.calculate_overall_score(&results);
        results.pass_fail_status = self.determine_pass_fail_status(&results);
        results.recommendations = self.generate_recommendations(&results);
        
        info!("✅ Comprehensive end-to-end testing completed");
        info!("Overall Score: {:.2}%", results.overall_score);
        info!("Status: {:?}", results.pass_fail_status);
        
        Ok(results)
    }

    /// Test real market data integration with Alpha Vantage
    async fn test_market_data_integration(&self) -> Result<MarketDataIntegrationResults, Box<dyn std::error::Error>> {
        info!("Testing Alpha Vantage market data integration...");

        let mut market_data_manager = MarketDataManager::new(&self.settings, self.database.clone()).await?;

        let start_time = Instant::now();

        // Test Alpha Vantage connectivity
        let connectivity_score = self.test_alpha_vantage_connectivity().await?;

        // Test data quality
        let data_quality_score = self.test_data_quality().await?;

        // Test real-time processing latency
        let processing_latency = self.measure_real_time_processing_latency().await?;

        // Test data consistency
        let consistency_score = self.test_data_consistency().await?;

        // Test pipeline reliability
        let reliability_score = self.test_pipeline_reliability().await?;

        let status = if connectivity_score > 0.8 && data_quality_score > 0.8 &&
                        processing_latency < 1000.0 && consistency_score > 0.9 {
            TestStatus::Passed
        } else {
            TestStatus::Failed
        };

        Ok(MarketDataIntegrationResults {
            alpha_vantage_connectivity_score: connectivity_score,
            data_quality_score,
            real_time_processing_latency_ms: processing_latency,
            data_consistency_score: consistency_score,
            pipeline_reliability_score: reliability_score,
            status,
        })
    }

    /// Test backend integration between all components
    async fn test_backend_integration(&self) -> Result<BackendIntegrationResults, Box<dyn std::error::Error>> {
        info!("Testing backend integration...");

        // Test TimescaleDB integration
        let timescaledb_score = self.test_timescaledb_integration().await?;

        // Test REST API integration
        let api_score = self.test_rest_api_integration().await?;

        // Test trading engine integration
        let trading_score = self.test_trading_engine_integration().await?;

        // Test AI models integration
        let ai_score = self.test_ai_models_integration().await?;

        // Test data flow consistency
        let data_flow_score = self.test_data_flow_consistency().await?;

        let status = if timescaledb_score > 0.9 && api_score > 0.9 &&
                        trading_score > 0.9 && ai_score > 0.8 {
            TestStatus::Passed
        } else {
            TestStatus::Failed
        };

        Ok(BackendIntegrationResults {
            timescaledb_integration_score: timescaledb_score,
            rest_api_integration_score: api_score,
            trading_engine_integration_score: trading_score,
            ai_models_integration_score: ai_score,
            data_flow_consistency_score: data_flow_score,
            status,
        })
    }

    /// Test autonomous trading operations
    async fn test_autonomous_trading(&self) -> Result<AutonomousTradingResults, Box<dyn std::error::Error>> {
        info!("Testing autonomous trading operations...");

        // Initialize trading engine with AI
        let trading_config = TradingEngineConfig::default();
        let ai_engine = AIEngine::new(self.database.clone()).await?;
        let trading_engine = TradingEngine::new(
            trading_config,
            self.database.clone(),
            ai_engine,
        ).await?;

        // Test AI signal generation
        let signal_success_rate = self.test_ai_signal_generation(&trading_engine).await?;

        // Test autonomous order execution
        let execution_count = self.test_autonomous_order_execution(&trading_engine).await?;

        // Test decision accuracy
        let decision_accuracy = self.test_autonomous_decision_accuracy(&trading_engine).await?;

        // Test portfolio management
        let portfolio_effectiveness = self.test_portfolio_management_effectiveness(&trading_engine).await?;

        // Test real-time adaptation
        let adaptation_score = self.test_real_time_adaptation(&trading_engine).await?;

        let status = if signal_success_rate > 0.8 && decision_accuracy > 0.7 &&
                        portfolio_effectiveness > 0.8 {
            TestStatus::Passed
        } else {
            TestStatus::Failed
        };

        Ok(AutonomousTradingResults {
            ai_signal_generation_success_rate: signal_success_rate,
            autonomous_order_execution_count: execution_count,
            autonomous_decision_accuracy: decision_accuracy,
            portfolio_management_effectiveness,
            real_time_adaptation_score: adaptation_score,
            status,
        })
    }

    /// Test order management functionality
    async fn test_order_management(&self) -> Result<OrderManagementResults, Box<dyn std::error::Error>> {
        info!("Testing order management...");

        // Test order placement
        let placement_success_rate = self.test_order_placement().await?;

        // Test order modification
        let modification_success_rate = self.test_order_modification().await?;

        // Test order cancellation
        let cancellation_success_rate = self.test_order_cancellation().await?;

        // Test order book consistency
        let consistency_score = self.test_order_book_consistency().await?;

        let order_types_tested = vec![
            "Market".to_string(),
            "Limit".to_string(),
            "StopLoss".to_string(),
            "TakeProfit".to_string(),
            "StopLimit".to_string(),
        ];

        let status = if placement_success_rate > 0.95 && modification_success_rate > 0.9 &&
                        cancellation_success_rate > 0.95 && consistency_score > 0.9 {
            TestStatus::Passed
        } else {
            TestStatus::Failed
        };

        Ok(OrderManagementResults {
            order_placement_success_rate: placement_success_rate,
            order_modification_success_rate: modification_success_rate,
            order_cancellation_success_rate: cancellation_success_rate,
            order_book_consistency_score: consistency_score,
            order_types_tested,
            status,
        })
    }

    /// Test execution scenarios
    async fn test_execution_scenarios(&self) -> Result<ExecutionScenariosResults, Box<dyn std::error::Error>> {
        info!("Testing execution scenarios...");

        // Test long position execution
        let long_execution_quality = self.test_long_position_execution().await?;

        // Test short position execution
        let short_execution_quality = self.test_short_position_execution().await?;

        // Test slippage management
        let slippage_score = self.test_slippage_management().await?;

        // Test execution speed
        let speed_score = self.test_execution_speed().await?;

        let scenarios_tested = vec![
            "Normal Market".to_string(),
            "High Volatility".to_string(),
            "Low Liquidity".to_string(),
            "Market Stress".to_string(),
            "Trending Market".to_string(),
            "Sideways Market".to_string(),
        ];

        let status = if long_execution_quality > 0.8 && short_execution_quality > 0.8 &&
                        slippage_score > 0.7 && speed_score > 0.9 {
            TestStatus::Passed
        } else {
            TestStatus::Failed
        };

        Ok(ExecutionScenariosResults {
            long_position_execution_quality,
            short_position_execution_quality,
            slippage_management_score: slippage_score,
            execution_speed_score: speed_score,
            scenarios_tested,
            status,
        })
    }

    /// Test performance benchmarks
    async fn test_performance_benchmarks(&self) -> Result<PerformanceBenchmarkResults, Box<dyn std::error::Error>> {
        info!("Running performance benchmarks...");

        // Measure order execution latency
        let execution_latency = self.measure_order_execution_latency().await?;

        // Measure AI inference latency
        let ai_latency = self.measure_ai_inference_latency().await?;

        // Measure throughput
        let throughput = self.measure_throughput().await?;

        // Measure uptime
        let uptime = self.measure_uptime().await?;

        // Measure error rate
        let error_rate = self.measure_error_rate().await?;

        // Check if performance targets are met
        let meets_targets = execution_latency < 10.0 &&
                           ai_latency < 100.0 &&
                           throughput > 1000.0 &&
                           uptime > 99.9 &&
                           error_rate < 0.1;

        let status = if meets_targets {
            TestStatus::Passed
        } else {
            TestStatus::Failed
        };

        Ok(PerformanceBenchmarkResults {
            order_execution_latency_ms: execution_latency,
            ai_inference_latency_ms: ai_latency,
            throughput_tps: throughput,
            uptime_percentage: uptime,
            error_rate_percentage: error_rate,
            meets_performance_targets: meets_targets,
            status,
        })
    }

    /// Test trading analytics
    async fn test_trading_analytics(&self) -> Result<TradingAnalyticsResults, Box<dyn std::error::Error>> {
        info!("Analyzing trading performance...");

        // Calculate trading accuracy
        let accuracy = self.calculate_trading_accuracy().await?;

        // Calculate profitability
        let profitability = self.calculate_profitability().await?;

        // Calculate Sharpe ratio
        let sharpe_ratio = self.calculate_sharpe_ratio().await?;

        // Calculate maximum drawdown
        let max_drawdown = self.calculate_maximum_drawdown().await?;

        // Calculate win/loss ratio
        let win_loss_ratio = self.calculate_win_loss_ratio().await?;

        // Calculate risk-adjusted returns
        let risk_adjusted_returns = self.calculate_risk_adjusted_returns().await?;

        let status = if accuracy > 60.0 && profitability > 0.0 &&
                        sharpe_ratio > 1.0 && max_drawdown < 10.0 {
            TestStatus::Passed
        } else {
            TestStatus::PartiallyPassed
        };

        Ok(TradingAnalyticsResults {
            trading_accuracy_percentage: accuracy,
            profitability_score: profitability,
            sharpe_ratio,
            maximum_drawdown_percentage: max_drawdown,
            win_loss_ratio,
            risk_adjusted_returns,
            status,
        })
    }

    /// Test system reliability
    async fn test_system_reliability(&self) -> Result<SystemReliabilityResults, Box<dyn std::error::Error>> {
        info!("Testing system reliability...");

        // Test uptime
        let uptime_score = self.test_uptime_reliability().await?;

        // Test error recovery
        let recovery_time = self.test_error_recovery_time().await?;

        // Test data consistency under load
        let consistency_under_load = self.test_data_consistency_under_load().await?;

        // Test auto-recovery
        let auto_recovery = self.test_auto_recovery_effectiveness().await?;

        // Test fault tolerance
        let fault_tolerance = self.test_fault_tolerance().await?;

        let status = if uptime_score > 0.99 && recovery_time < 1000.0 &&
                        consistency_under_load > 0.95 && auto_recovery > 0.9 {
            TestStatus::Passed
        } else {
            TestStatus::Failed
        };

        Ok(SystemReliabilityResults {
            uptime_score,
            error_recovery_time_ms: recovery_time,
            data_consistency_under_load_score: consistency_under_load,
            auto_recovery_effectiveness: auto_recovery,
            fault_tolerance_score: fault_tolerance,
            status,
        })
    }

    /// Test competitive analysis
    async fn test_competitive_analysis(&self) -> Result<CompetitiveAnalysisResults, Box<dyn std::error::Error>> {
        info!("Running competitive analysis...");

        // Compare execution speed vs industry
        let execution_vs_industry = self.compare_execution_speed_vs_industry().await?;

        // Compare trading accuracy vs industry
        let accuracy_vs_industry = self.compare_trading_accuracy_vs_industry().await?;

        // Compare risk management vs industry
        let risk_vs_industry = self.compare_risk_management_vs_industry().await?;

        // Compare profitability vs industry
        let profitability_vs_industry = self.compare_profitability_vs_industry().await?;

        // Calculate overall competitive score
        let overall_score = (execution_vs_industry + accuracy_vs_industry +
                           risk_vs_industry + profitability_vs_industry) / 4.0;

        // Calculate industry ranking percentile
        let ranking_percentile = self.calculate_industry_ranking_percentile(overall_score).await?;

        let status = if overall_score > 0.8 && ranking_percentile > 75.0 {
            TestStatus::Passed
        } else if overall_score > 0.6 && ranking_percentile > 50.0 {
            TestStatus::PartiallyPassed
        } else {
            TestStatus::Failed
        };

        Ok(CompetitiveAnalysisResults {
            execution_speed_vs_industry,
            trading_accuracy_vs_industry: accuracy_vs_industry,
            risk_management_vs_industry: risk_vs_industry,
            profitability_vs_industry,
            overall_competitive_score: overall_score,
            industry_ranking_percentile: ranking_percentile,
            status,
        })
    }

    // Helper methods for creating default results
    fn create_default_autonomous_results(&self) -> AutonomousTradingResults {
        AutonomousTradingResults {
            ai_signal_generation_success_rate: 0.0,
            autonomous_order_execution_count: 0,
            autonomous_decision_accuracy: 0.0,
            portfolio_management_effectiveness: 0.0,
            real_time_adaptation_score: 0.0,
            status: TestStatus::NotRun,
        }
    }

    fn create_default_order_results(&self) -> OrderManagementResults {
        OrderManagementResults {
            order_placement_success_rate: 0.0,
            order_modification_success_rate: 0.0,
            order_cancellation_success_rate: 0.0,
            order_book_consistency_score: 0.0,
            order_types_tested: Vec::new(),
            status: TestStatus::NotRun,
        }
    }

    fn create_default_execution_results(&self) -> ExecutionScenariosResults {
        ExecutionScenariosResults {
            long_position_execution_quality: 0.0,
            short_position_execution_quality: 0.0,
            slippage_management_score: 0.0,
            execution_speed_score: 0.0,
            scenarios_tested: Vec::new(),
            status: TestStatus::NotRun,
        }
    }

    fn create_default_market_data_results(&self) -> MarketDataIntegrationResults {
        MarketDataIntegrationResults {
            alpha_vantage_connectivity_score: 0.0,
            data_quality_score: 0.0,
            real_time_processing_latency_ms: 0.0,
            data_consistency_score: 0.0,
            pipeline_reliability_score: 0.0,
            status: TestStatus::NotRun,
        }
    }

    fn create_default_backend_results(&self) -> BackendIntegrationResults {
        BackendIntegrationResults {
            timescaledb_integration_score: 0.0,
            rest_api_integration_score: 0.0,
            trading_engine_integration_score: 0.0,
            ai_models_integration_score: 0.0,
            data_flow_consistency_score: 0.0,
            status: TestStatus::NotRun,
        }
    }

    fn create_default_performance_results(&self) -> PerformanceBenchmarkResults {
        PerformanceBenchmarkResults {
            order_execution_latency_ms: 0.0,
            ai_inference_latency_ms: 0.0,
            throughput_tps: 0.0,
            uptime_percentage: 0.0,
            error_rate_percentage: 0.0,
            meets_performance_targets: false,
            status: TestStatus::NotRun,
        }
    }

    fn create_default_analytics_results(&self) -> TradingAnalyticsResults {
        TradingAnalyticsResults {
            trading_accuracy_percentage: 0.0,
            profitability_score: 0.0,
            sharpe_ratio: 0.0,
            maximum_drawdown_percentage: 0.0,
            win_loss_ratio: 0.0,
            risk_adjusted_returns: 0.0,
            status: TestStatus::NotRun,
        }
    }

    fn create_default_reliability_results(&self) -> SystemReliabilityResults {
        SystemReliabilityResults {
            uptime_score: 0.0,
            error_recovery_time_ms: 0.0,
            data_consistency_under_load_score: 0.0,
            auto_recovery_effectiveness: 0.0,
            fault_tolerance_score: 0.0,
            status: TestStatus::NotRun,
        }
    }

    fn create_default_competitive_results(&self) -> CompetitiveAnalysisResults {
        CompetitiveAnalysisResults {
            execution_speed_vs_industry: 0.0,
            trading_accuracy_vs_industry: 0.0,
            risk_management_vs_industry: 0.0,
            profitability_vs_industry: 0.0,
            overall_competitive_score: 0.0,
            industry_ranking_percentile: 0.0,
            status: TestStatus::NotRun,
        }
    }

    /// Calculate overall test score
    fn calculate_overall_score(&self, results: &E2ETestResults) -> f64 {
        let scores = vec![
            self.get_test_score(&results.autonomous_trading.status) * 0.15,
            self.get_test_score(&results.order_management.status) * 0.15,
            self.get_test_score(&results.execution_scenarios.status) * 0.15,
            self.get_test_score(&results.market_data_integration.status) * 0.15,
            self.get_test_score(&results.backend_integration.status) * 0.15,
            self.get_test_score(&results.performance_benchmarks.status) * 0.15,
            self.get_test_score(&results.trading_analytics.status) * 0.05,
            self.get_test_score(&results.system_reliability.status) * 0.05,
        ];

        scores.iter().sum::<f64>() * 100.0
    }

    /// Determine overall pass/fail status
    fn determine_pass_fail_status(&self, results: &E2ETestResults) -> TestStatus {
        let critical_tests = vec![
            &results.autonomous_trading.status,
            &results.order_management.status,
            &results.execution_scenarios.status,
            &results.market_data_integration.status,
            &results.backend_integration.status,
            &results.performance_benchmarks.status,
        ];

        let passed_count = critical_tests.iter()
            .filter(|status| matches!(status, TestStatus::Passed))
            .count();

        let failed_count = critical_tests.iter()
            .filter(|status| matches!(status, TestStatus::Failed))
            .count();

        if failed_count == 0 && passed_count >= 5 {
            TestStatus::Passed
        } else if failed_count <= 1 && passed_count >= 4 {
            TestStatus::PartiallyPassed
        } else {
            TestStatus::Failed
        }
    }

    /// Generate recommendations based on test results
    fn generate_recommendations(&self, results: &E2ETestResults) -> Vec<String> {
        let mut recommendations = Vec::new();

        if matches!(results.performance_benchmarks.status, TestStatus::Failed) {
            recommendations.push("Optimize order execution latency - consider async processing improvements".to_string());
            recommendations.push("Enhance AI inference speed - consider model optimization or hardware acceleration".to_string());
        }

        if matches!(results.autonomous_trading.status, TestStatus::Failed) {
            recommendations.push("Improve AI signal generation accuracy - retrain models with more data".to_string());
            recommendations.push("Enhance autonomous decision-making algorithms".to_string());
        }

        if matches!(results.market_data_integration.status, TestStatus::Failed) {
            recommendations.push("Improve Alpha Vantage API integration reliability".to_string());
            recommendations.push("Implement better data quality validation".to_string());
        }

        if results.overall_score < 80.0 {
            recommendations.push("Overall system performance needs improvement before production deployment".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("System is performing well and ready for production deployment".to_string());
        }

        recommendations
    }

    fn get_test_score(&self, status: &TestStatus) -> f64 {
        match status {
            TestStatus::Passed => 1.0,
            TestStatus::PartiallyPassed => 0.7,
            TestStatus::Failed => 0.0,
            TestStatus::NotRun => 0.0,
        }
    }
}
