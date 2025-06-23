// Buy/Sell Execution Scenario Tests for PantherSwap Edge
// Comprehensive tests for long/short positions, order types, slippage handling, and execution quality
// Run with: cargo test --test execution_scenario_tests

use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio::time::sleep;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use tracing::{info, warn, error, debug};

use pantherswap_edge::trading::{
    TradingEngine, TradingEngineConfig, Order, OrderType, OrderSide, OrderStatus,
    Fill, MarketData, ExecutionEngine
};
use pantherswap_edge::config::Settings;
use pantherswap_edge::database::Database;
use pantherswap_edge::ai::AIEngine;

mod common;
use common::*;

/// Execution scenario test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionScenarioTestResults {
    pub test_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub long_position_tests: LongPositionResults,
    pub short_position_tests: ShortPositionResults,
    pub order_type_tests: OrderTypeExecutionResults,
    pub slippage_tests: SlippageHandlingResults,
    pub execution_quality_tests: ExecutionQualityResults,
    pub market_condition_tests: MarketConditionResults,
    pub performance_metrics: ExecutionPerformanceMetrics,
    pub overall_score: f64,
    pub pass_fail_status: TestStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestStatus {
    Passed,
    Failed,
    PartiallyPassed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongPositionResults {
    pub market_buy_execution_quality: f64,
    pub limit_buy_execution_quality: f64,
    pub stop_loss_protection_effectiveness: f64,
    pub take_profit_execution_accuracy: f64,
    pub position_sizing_accuracy: f64,
    pub average_execution_latency_ms: f64,
    pub total_long_positions_tested: u64,
    pub status: TestStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortPositionResults {
    pub market_sell_execution_quality: f64,
    pub limit_sell_execution_quality: f64,
    pub stop_loss_protection_effectiveness: f64,
    pub take_profit_execution_accuracy: f64,
    pub position_sizing_accuracy: f64,
    pub average_execution_latency_ms: f64,
    pub total_short_positions_tested: u64,
    pub status: TestStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderTypeExecutionResults {
    pub market_order_fill_rate: f64,
    pub limit_order_fill_rate: f64,
    pub stop_order_trigger_accuracy: f64,
    pub stop_limit_execution_quality: f64,
    pub iceberg_order_stealth_score: f64,
    pub twap_execution_quality: f64,
    pub vwap_execution_quality: f64,
    pub status: TestStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlippageHandlingResults {
    pub positive_slippage_capture_rate: f64,
    pub negative_slippage_mitigation_rate: f64,
    pub average_slippage_bps: f64,
    pub slippage_prediction_accuracy: f64,
    pub dynamic_slippage_adjustment_effectiveness: f64,
    pub status: TestStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionQualityResults {
    pub price_improvement_rate: f64,
    pub fill_ratio_optimization: f64,
    pub timing_optimization_score: f64,
    pub liquidity_detection_accuracy: f64,
    pub market_impact_minimization: f64,
    pub execution_cost_efficiency: f64,
    pub status: TestStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketConditionResults {
    pub normal_market_performance: f64,
    pub high_volatility_performance: f64,
    pub low_liquidity_performance: f64,
    pub trending_market_performance: f64,
    pub sideways_market_performance: f64,
    pub stress_test_performance: f64,
    pub status: TestStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPerformanceMetrics {
    pub orders_per_second_capacity: f64,
    pub execution_latency_percentiles: LatencyPercentiles,
    pub memory_efficiency_score: f64,
    pub cpu_utilization_under_load: f64,
    pub error_rate_percentage: f64,
    pub recovery_time_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyPercentiles {
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub p99_9_ms: f64,
}

/// Execution scenario test orchestrator
pub struct ExecutionScenarioTestOrchestrator {
    trading_engine: TradingEngine,
    execution_engine: ExecutionEngine,
    test_id: Uuid,
    start_time: DateTime<Utc>,
}

impl ExecutionScenarioTestOrchestrator {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Load settings
        let settings = Settings::load()?;
        
        // Initialize database
        let database = Database::new(&settings.database.url).await?;
        
        // Initialize AI engine
        let ai_engine = AIEngine::new(database.clone()).await?;
        
        // Initialize trading engine
        let trading_config = TradingEngineConfig::default();
        let trading_engine = TradingEngine::new(
            trading_config,
            database.clone(),
            ai_engine,
        ).await?;
        
        // Initialize execution engine
        let execution_config = pantherswap_edge::trading::ExecutionConfig::default();
        let execution_engine = ExecutionEngine::new(execution_config, database).await?;
        
        Ok(Self {
            trading_engine,
            execution_engine,
            test_id: Uuid::new_v4(),
            start_time: Utc::now(),
        })
    }

    /// Run comprehensive execution scenario tests
    pub async fn run_comprehensive_execution_tests(&self) -> Result<ExecutionScenarioTestResults, Box<dyn std::error::Error>> {
        info!("🚀 Starting comprehensive execution scenario tests");
        info!("Test ID: {}", self.test_id);
        
        // Run all test categories
        let long_position_tests = self.test_long_position_execution().await?;
        let short_position_tests = self.test_short_position_execution().await?;
        let order_type_tests = self.test_order_type_execution().await?;
        let slippage_tests = self.test_slippage_handling().await?;
        let execution_quality_tests = self.test_execution_quality().await?;
        let market_condition_tests = self.test_market_conditions().await?;
        let performance_metrics = self.measure_execution_performance().await?;
        
        // Calculate overall score
        let overall_score = self.calculate_overall_score(
            &long_position_tests,
            &short_position_tests,
            &order_type_tests,
            &slippage_tests,
            &execution_quality_tests,
            &market_condition_tests,
        );
        
        // Determine pass/fail status
        let pass_fail_status = self.determine_pass_fail_status(overall_score);
        
        let results = ExecutionScenarioTestResults {
            test_id: self.test_id,
            timestamp: Utc::now(),
            long_position_tests,
            short_position_tests,
            order_type_tests,
            slippage_tests,
            execution_quality_tests,
            market_condition_tests,
            performance_metrics,
            overall_score,
            pass_fail_status,
        };
        
        info!("✅ Execution scenario tests completed");
        info!("Overall Score: {:.2}%", results.overall_score);
        info!("Status: {:?}", results.pass_fail_status);
        
        Ok(results)
    }

    /// Test long position execution scenarios
    async fn test_long_position_execution(&self) -> Result<LongPositionResults, Box<dyn std::error::Error>> {
        info!("📈 Testing long position execution scenarios...");
        
        let mut execution_latencies = Vec::new();
        let test_iterations = 50;
        
        // Test market buy orders
        let mut market_buy_scores = Vec::new();
        for i in 0..test_iterations {
            let start_time = Instant::now();
            
            let order = self.create_market_buy_order(100.0).await?;
            let execution_result = self.execute_order_with_quality_measurement(order).await?;
            
            let latency = start_time.elapsed().as_micros() as f64 / 1000.0;
            execution_latencies.push(latency);
            market_buy_scores.push(execution_result.execution_quality);
            
            debug!("Market buy {} - Quality: {:.2}, Latency: {:.2}ms", i + 1, execution_result.execution_quality, latency);
            sleep(Duration::from_millis(20)).await;
        }
        
        // Test limit buy orders
        let mut limit_buy_scores = Vec::new();
        for i in 0..test_iterations {
            let start_time = Instant::now();
            
            let order = self.create_limit_buy_order(100.0, 1.1230).await?;
            let execution_result = self.execute_order_with_quality_measurement(order).await?;
            
            let latency = start_time.elapsed().as_micros() as f64 / 1000.0;
            execution_latencies.push(latency);
            limit_buy_scores.push(execution_result.execution_quality);
            
            debug!("Limit buy {} - Quality: {:.2}, Latency: {:.2}ms", i + 1, execution_result.execution_quality, latency);
            sleep(Duration::from_millis(20)).await;
        }
        
        // Test stop-loss protection
        let stop_loss_effectiveness = self.test_stop_loss_protection_long().await?;
        
        // Test take-profit execution
        let take_profit_accuracy = self.test_take_profit_execution_long().await?;
        
        // Test position sizing accuracy
        let position_sizing_accuracy = self.test_position_sizing_accuracy_long().await?;
        
        // Calculate results
        let market_buy_execution_quality = market_buy_scores.iter().sum::<f64>() / market_buy_scores.len() as f64;
        let limit_buy_execution_quality = limit_buy_scores.iter().sum::<f64>() / limit_buy_scores.len() as f64;
        let average_execution_latency_ms = execution_latencies.iter().sum::<f64>() / execution_latencies.len() as f64;
        let total_long_positions_tested = (test_iterations * 2) as u64;
        
        let status = if market_buy_execution_quality > 0.85 && 
                        limit_buy_execution_quality > 0.80 && 
                        stop_loss_effectiveness > 0.90 {
            TestStatus::Passed
        } else if market_buy_execution_quality > 0.75 && limit_buy_execution_quality > 0.70 {
            TestStatus::PartiallyPassed
        } else {
            TestStatus::Failed
        };
        
        info!("📈 Long position execution results:");
        info!("  • Market buy execution quality: {:.2}%", market_buy_execution_quality * 100.0);
        info!("  • Limit buy execution quality: {:.2}%", limit_buy_execution_quality * 100.0);
        info!("  • Stop-loss protection effectiveness: {:.2}%", stop_loss_effectiveness * 100.0);
        info!("  • Take-profit execution accuracy: {:.2}%", take_profit_accuracy * 100.0);
        info!("  • Position sizing accuracy: {:.2}%", position_sizing_accuracy * 100.0);
        info!("  • Average execution latency: {:.2}ms", average_execution_latency_ms);
        info!("  • Total long positions tested: {}", total_long_positions_tested);
        info!("  • Status: {:?}", status);
        
        Ok(LongPositionResults {
            market_buy_execution_quality,
            limit_buy_execution_quality,
            stop_loss_protection_effectiveness: stop_loss_effectiveness,
            take_profit_execution_accuracy: take_profit_accuracy,
            position_sizing_accuracy,
            average_execution_latency_ms,
            total_long_positions_tested,
            status,
        })
    }

    /// Test short position execution scenarios
    async fn test_short_position_execution(&self) -> Result<ShortPositionResults, Box<dyn std::error::Error>> {
        info!("📉 Testing short position execution scenarios...");

        let mut execution_latencies = Vec::new();
        let test_iterations = 50;

        // Test market sell orders
        let mut market_sell_scores = Vec::new();
        for i in 0..test_iterations {
            let start_time = Instant::now();

            let order = self.create_market_sell_order(100.0).await?;
            let execution_result = self.execute_order_with_quality_measurement(order).await?;

            let latency = start_time.elapsed().as_micros() as f64 / 1000.0;
            execution_latencies.push(latency);
            market_sell_scores.push(execution_result.execution_quality);

            debug!("Market sell {} - Quality: {:.2}, Latency: {:.2}ms", i + 1, execution_result.execution_quality, latency);
            sleep(Duration::from_millis(20)).await;
        }

        // Test limit sell orders
        let mut limit_sell_scores = Vec::new();
        for i in 0..test_iterations {
            let start_time = Instant::now();

            let order = self.create_limit_sell_order(100.0, 1.1240).await?;
            let execution_result = self.execute_order_with_quality_measurement(order).await?;

            let latency = start_time.elapsed().as_micros() as f64 / 1000.0;
            execution_latencies.push(latency);
            limit_sell_scores.push(execution_result.execution_quality);

            debug!("Limit sell {} - Quality: {:.2}, Latency: {:.2}ms", i + 1, execution_result.execution_quality, latency);
            sleep(Duration::from_millis(20)).await;
        }

        // Test stop-loss protection for short positions
        let stop_loss_effectiveness = self.test_stop_loss_protection_short().await?;

        // Test take-profit execution for short positions
        let take_profit_accuracy = self.test_take_profit_execution_short().await?;

        // Test position sizing accuracy for short positions
        let position_sizing_accuracy = self.test_position_sizing_accuracy_short().await?;

        // Calculate results
        let market_sell_execution_quality = market_sell_scores.iter().sum::<f64>() / market_sell_scores.len() as f64;
        let limit_sell_execution_quality = limit_sell_scores.iter().sum::<f64>() / limit_sell_scores.len() as f64;
        let average_execution_latency_ms = execution_latencies.iter().sum::<f64>() / execution_latencies.len() as f64;
        let total_short_positions_tested = (test_iterations * 2) as u64;

        let status = if market_sell_execution_quality > 0.85 &&
                        limit_sell_execution_quality > 0.80 &&
                        stop_loss_effectiveness > 0.90 {
            TestStatus::Passed
        } else if market_sell_execution_quality > 0.75 && limit_sell_execution_quality > 0.70 {
            TestStatus::PartiallyPassed
        } else {
            TestStatus::Failed
        };

        info!("📉 Short position execution results:");
        info!("  • Market sell execution quality: {:.2}%", market_sell_execution_quality * 100.0);
        info!("  • Limit sell execution quality: {:.2}%", limit_sell_execution_quality * 100.0);
        info!("  • Stop-loss protection effectiveness: {:.2}%", stop_loss_effectiveness * 100.0);
        info!("  • Take-profit execution accuracy: {:.2}%", take_profit_accuracy * 100.0);
        info!("  • Position sizing accuracy: {:.2}%", position_sizing_accuracy * 100.0);
        info!("  • Average execution latency: {:.2}ms", average_execution_latency_ms);
        info!("  • Total short positions tested: {}", total_short_positions_tested);
        info!("  • Status: {:?}", status);

        Ok(ShortPositionResults {
            market_sell_execution_quality,
            limit_sell_execution_quality,
            stop_loss_protection_effectiveness: stop_loss_effectiveness,
            take_profit_execution_accuracy: take_profit_accuracy,
            position_sizing_accuracy,
            average_execution_latency_ms,
            total_short_positions_tested,
            status,
        })
    }

    /// Test order type execution quality
    async fn test_order_type_execution(&self) -> Result<OrderTypeExecutionResults, Box<dyn std::error::Error>> {
        info!("🔄 Testing order type execution quality...");

        // Test market order fill rate
        let market_order_fill_rate = self.test_market_order_fill_rate().await?;

        // Test limit order fill rate
        let limit_order_fill_rate = self.test_limit_order_fill_rate().await?;

        // Test stop order trigger accuracy
        let stop_order_trigger_accuracy = self.test_stop_order_triggers().await?;

        // Test stop-limit execution quality
        let stop_limit_execution_quality = self.test_stop_limit_execution_quality().await?;

        // Test iceberg order stealth
        let iceberg_order_stealth_score = self.test_iceberg_order_stealth().await?;

        // Test TWAP execution quality
        let twap_execution_quality = self.test_twap_execution().await?;

        // Test VWAP execution quality
        let vwap_execution_quality = self.test_vwap_execution().await?;

        let status = if market_order_fill_rate > 0.95 &&
                        limit_order_fill_rate > 0.85 &&
                        stop_order_trigger_accuracy > 0.90 {
            TestStatus::Passed
        } else if market_order_fill_rate > 0.85 && limit_order_fill_rate > 0.75 {
            TestStatus::PartiallyPassed
        } else {
            TestStatus::Failed
        };

        info!("🔄 Order type execution results:");
        info!("  • Market order fill rate: {:.2}%", market_order_fill_rate * 100.0);
        info!("  • Limit order fill rate: {:.2}%", limit_order_fill_rate * 100.0);
        info!("  • Stop order trigger accuracy: {:.2}%", stop_order_trigger_accuracy * 100.0);
        info!("  • Stop-limit execution quality: {:.2}%", stop_limit_execution_quality * 100.0);
        info!("  • Iceberg order stealth score: {:.2}%", iceberg_order_stealth_score * 100.0);
        info!("  • TWAP execution quality: {:.2}%", twap_execution_quality * 100.0);
        info!("  • VWAP execution quality: {:.2}%", vwap_execution_quality * 100.0);
        info!("  • Status: {:?}", status);

        Ok(OrderTypeExecutionResults {
            market_order_fill_rate,
            limit_order_fill_rate,
            stop_order_trigger_accuracy,
            stop_limit_execution_quality,
            iceberg_order_stealth_score,
            twap_execution_quality,
            vwap_execution_quality,
            status,
        })
    }

    /// Test slippage handling
    async fn test_slippage_handling(&self) -> Result<SlippageHandlingResults, Box<dyn std::error::Error>> {
        info!("💨 Testing slippage handling...");

        // Test positive slippage capture
        let positive_slippage_capture_rate = self.test_positive_slippage_capture().await?;

        // Test negative slippage mitigation
        let negative_slippage_mitigation_rate = self.test_negative_slippage_mitigation().await?;

        // Measure average slippage
        let average_slippage_bps = self.measure_average_slippage().await?;

        // Test slippage prediction accuracy
        let slippage_prediction_accuracy = self.test_slippage_prediction().await?;

        // Test dynamic slippage adjustment
        let dynamic_slippage_adjustment_effectiveness = self.test_dynamic_slippage_adjustment().await?;

        let status = if positive_slippage_capture_rate > 0.70 &&
                        negative_slippage_mitigation_rate > 0.80 &&
                        average_slippage_bps < 2.0 {
            TestStatus::Passed
        } else if negative_slippage_mitigation_rate > 0.70 && average_slippage_bps < 3.0 {
            TestStatus::PartiallyPassed
        } else {
            TestStatus::Failed
        };

        info!("💨 Slippage handling results:");
        info!("  • Positive slippage capture rate: {:.2}%", positive_slippage_capture_rate * 100.0);
        info!("  • Negative slippage mitigation rate: {:.2}%", negative_slippage_mitigation_rate * 100.0);
        info!("  • Average slippage: {:.2} bps", average_slippage_bps);
        info!("  • Slippage prediction accuracy: {:.2}%", slippage_prediction_accuracy * 100.0);
        info!("  • Dynamic adjustment effectiveness: {:.2}%", dynamic_slippage_adjustment_effectiveness * 100.0);
        info!("  • Status: {:?}", status);

        Ok(SlippageHandlingResults {
            positive_slippage_capture_rate,
            negative_slippage_mitigation_rate,
            average_slippage_bps,
            slippage_prediction_accuracy,
            dynamic_slippage_adjustment_effectiveness,
            status,
        })
    }

    /// Test execution quality metrics
    async fn test_execution_quality(&self) -> Result<ExecutionQualityResults, Box<dyn std::error::Error>> {
        info!("⭐ Testing execution quality metrics...");

        // Test price improvement rate
        let price_improvement_rate = self.test_price_improvement().await?;

        // Test fill ratio optimization
        let fill_ratio_optimization = self.test_fill_ratio_optimization().await?;

        // Test timing optimization
        let timing_optimization_score = self.test_timing_optimization().await?;

        // Test liquidity detection accuracy
        let liquidity_detection_accuracy = self.test_liquidity_detection().await?;

        // Test market impact minimization
        let market_impact_minimization = self.test_market_impact_minimization().await?;

        // Test execution cost efficiency
        let execution_cost_efficiency = self.test_execution_cost_efficiency().await?;

        let status = if price_improvement_rate > 0.60 &&
                        fill_ratio_optimization > 0.85 &&
                        timing_optimization_score > 0.80 {
            TestStatus::Passed
        } else if price_improvement_rate > 0.40 && fill_ratio_optimization > 0.75 {
            TestStatus::PartiallyPassed
        } else {
            TestStatus::Failed
        };

        info!("⭐ Execution quality results:");
        info!("  • Price improvement rate: {:.2}%", price_improvement_rate * 100.0);
        info!("  • Fill ratio optimization: {:.2}%", fill_ratio_optimization * 100.0);
        info!("  • Timing optimization score: {:.2}%", timing_optimization_score * 100.0);
        info!("  • Liquidity detection accuracy: {:.2}%", liquidity_detection_accuracy * 100.0);
        info!("  • Market impact minimization: {:.2}%", market_impact_minimization * 100.0);
        info!("  • Execution cost efficiency: {:.2}%", execution_cost_efficiency * 100.0);
        info!("  • Status: {:?}", status);

        Ok(ExecutionQualityResults {
            price_improvement_rate,
            fill_ratio_optimization,
            timing_optimization_score,
            liquidity_detection_accuracy,
            market_impact_minimization,
            execution_cost_efficiency,
            status,
        })
    }

    /// Test performance under different market conditions
    async fn test_market_conditions(&self) -> Result<MarketConditionResults, Box<dyn std::error::Error>> {
        info!("🌊 Testing performance under different market conditions...");

        // Test normal market conditions
        let normal_market_performance = self.test_normal_market_execution().await?;

        // Test high volatility conditions
        let high_volatility_performance = self.test_high_volatility_execution().await?;

        // Test low liquidity conditions
        let low_liquidity_performance = self.test_low_liquidity_execution().await?;

        // Test trending market conditions
        let trending_market_performance = self.test_trending_market_execution().await?;

        // Test sideways market conditions
        let sideways_market_performance = self.test_sideways_market_execution().await?;

        // Test stress conditions
        let stress_test_performance = self.test_stress_conditions().await?;

        let status = if normal_market_performance > 0.85 &&
                        high_volatility_performance > 0.75 &&
                        low_liquidity_performance > 0.70 {
            TestStatus::Passed
        } else if normal_market_performance > 0.75 && high_volatility_performance > 0.65 {
            TestStatus::PartiallyPassed
        } else {
            TestStatus::Failed
        };

        info!("🌊 Market condition results:");
        info!("  • Normal market performance: {:.2}%", normal_market_performance * 100.0);
        info!("  • High volatility performance: {:.2}%", high_volatility_performance * 100.0);
        info!("  • Low liquidity performance: {:.2}%", low_liquidity_performance * 100.0);
        info!("  • Trending market performance: {:.2}%", trending_market_performance * 100.0);
        info!("  • Sideways market performance: {:.2}%", sideways_market_performance * 100.0);
        info!("  • Stress test performance: {:.2}%", stress_test_performance * 100.0);
        info!("  • Status: {:?}", status);

        Ok(MarketConditionResults {
            normal_market_performance,
            high_volatility_performance,
            low_liquidity_performance,
            trending_market_performance,
            sideways_market_performance,
            stress_test_performance,
            status,
        })
    }

    /// Measure execution performance metrics
    async fn measure_execution_performance(&self) -> Result<ExecutionPerformanceMetrics, Box<dyn std::error::Error>> {
        info!("⚡ Measuring execution performance metrics...");

        // Measure orders per second capacity
        let orders_per_second_capacity = self.measure_orders_per_second().await?;

        // Measure latency percentiles
        let execution_latency_percentiles = self.measure_execution_latency_percentiles().await?;

        // Measure memory efficiency
        let memory_efficiency_score = self.measure_execution_memory_efficiency().await?;

        // Measure CPU utilization
        let cpu_utilization_under_load = self.measure_execution_cpu_utilization().await?;

        // Measure error rate
        let error_rate_percentage = self.measure_execution_error_rate().await?;

        // Measure recovery time
        let recovery_time_ms = self.measure_execution_recovery_time().await?;

        info!("⚡ Execution performance metrics:");
        info!("  • Orders per second capacity: {:.0}", orders_per_second_capacity);
        info!("  • P50 execution latency: {:.2}ms", execution_latency_percentiles.p50_ms);
        info!("  • P95 execution latency: {:.2}ms", execution_latency_percentiles.p95_ms);
        info!("  • P99 execution latency: {:.2}ms", execution_latency_percentiles.p99_ms);
        info!("  • Memory efficiency: {:.2}%", memory_efficiency_score * 100.0);
        info!("  • CPU utilization: {:.2}%", cpu_utilization_under_load * 100.0);
        info!("  • Error rate: {:.2}%", error_rate_percentage);
        info!("  • Recovery time: {:.2}ms", recovery_time_ms);

        Ok(ExecutionPerformanceMetrics {
            orders_per_second_capacity,
            execution_latency_percentiles,
            memory_efficiency_score,
            cpu_utilization_under_load,
            error_rate_percentage,
            recovery_time_ms,
        })
    }

    /// Calculate overall score
    fn calculate_overall_score(
        &self,
        long_position: &LongPositionResults,
        short_position: &ShortPositionResults,
        order_types: &OrderTypeExecutionResults,
        slippage: &SlippageHandlingResults,
        execution_quality: &ExecutionQualityResults,
        market_conditions: &MarketConditionResults,
    ) -> f64 {
        let long_score = self.get_test_score(&long_position.status) * 0.20;
        let short_score = self.get_test_score(&short_position.status) * 0.20;
        let order_types_score = self.get_test_score(&order_types.status) * 0.20;
        let slippage_score = self.get_test_score(&slippage.status) * 0.15;
        let quality_score = self.get_test_score(&execution_quality.status) * 0.15;
        let market_score = self.get_test_score(&market_conditions.status) * 0.10;

        (long_score + short_score + order_types_score + slippage_score + quality_score + market_score) * 100.0
    }

    /// Determine pass/fail status
    fn determine_pass_fail_status(&self, overall_score: f64) -> TestStatus {
        if overall_score >= 80.0 {
            TestStatus::Passed
        } else if overall_score >= 65.0 {
            TestStatus::PartiallyPassed
        } else {
            TestStatus::Failed
        }
    }

    fn get_test_score(&self, status: &TestStatus) -> f64 {
        match status {
            TestStatus::Passed => 1.0,
            TestStatus::PartiallyPassed => 0.7,
            TestStatus::Failed => 0.0,
        }
    }
}
