// Order Book Management Testing for PantherSwap Edge
// Comprehensive tests for order placement, modification, cancellation, and state management
// Run with: cargo test --test order_book_management_tests

use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio::time::sleep;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use tracing::{info, warn, error, debug};

use pantherswap_edge::trading::{
    TradingEngine, TradingEngineConfig, Order, OrderType, OrderSide, OrderStatus,
    OrderManager, OrderBook, Fill, MarketData
};
use pantherswap_edge::config::Settings;
use pantherswap_edge::database::Database;
use pantherswap_edge::ai::AIEngine;

mod common;
use common::*;

/// Order book management test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookTestResults {
    pub test_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub order_placement_tests: OrderPlacementResults,
    pub order_modification_tests: OrderModificationResults,
    pub order_cancellation_tests: OrderCancellationResults,
    pub order_book_state_tests: OrderBookStateResults,
    pub order_types_tests: OrderTypesResults,
    pub performance_metrics: OrderBookPerformanceMetrics,
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
pub struct OrderPlacementResults {
    pub market_orders_success_rate: f64,
    pub limit_orders_success_rate: f64,
    pub stop_loss_orders_success_rate: f64,
    pub take_profit_orders_success_rate: f64,
    pub average_placement_latency_ms: f64,
    pub total_orders_placed: u64,
    pub status: TestStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderModificationResults {
    pub price_modification_success_rate: f64,
    pub quantity_modification_success_rate: f64,
    pub stop_loss_modification_success_rate: f64,
    pub take_profit_modification_success_rate: f64,
    pub average_modification_latency_ms: f64,
    pub total_modifications: u64,
    pub status: TestStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCancellationResults {
    pub immediate_cancellation_success_rate: f64,
    pub partial_fill_cancellation_success_rate: f64,
    pub bulk_cancellation_success_rate: f64,
    pub average_cancellation_latency_ms: f64,
    pub total_cancellations: u64,
    pub status: TestStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookStateResults {
    pub state_consistency_score: f64,
    pub real_time_updates_accuracy: f64,
    pub order_matching_accuracy: f64,
    pub price_level_integrity_score: f64,
    pub volume_tracking_accuracy: f64,
    pub status: TestStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderTypesResults {
    pub market_order_execution_quality: f64,
    pub limit_order_execution_quality: f64,
    pub stop_loss_trigger_accuracy: f64,
    pub take_profit_trigger_accuracy: f64,
    pub stop_limit_execution_quality: f64,
    pub iceberg_order_handling: f64,
    pub status: TestStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookPerformanceMetrics {
    pub order_processing_throughput_ops: f64,
    pub memory_usage_efficiency: f64,
    pub cpu_utilization_under_load: f64,
    pub latency_percentiles: LatencyPercentiles,
    pub error_rate_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyPercentiles {
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub p99_9_ms: f64,
}

/// Order book management test orchestrator
pub struct OrderBookTestOrchestrator {
    trading_engine: TradingEngine,
    test_id: Uuid,
    start_time: DateTime<Utc>,
}

impl OrderBookTestOrchestrator {
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
            database,
            ai_engine,
        ).await?;
        
        Ok(Self {
            trading_engine,
            test_id: Uuid::new_v4(),
            start_time: Utc::now(),
        })
    }

    /// Run comprehensive order book management tests
    pub async fn run_comprehensive_order_book_tests(&self) -> Result<OrderBookTestResults, Box<dyn std::error::Error>> {
        info!("🚀 Starting comprehensive order book management tests");
        info!("Test ID: {}", self.test_id);
        
        // Run all test categories
        let order_placement_tests = self.test_order_placement().await?;
        let order_modification_tests = self.test_order_modification().await?;
        let order_cancellation_tests = self.test_order_cancellation().await?;
        let order_book_state_tests = self.test_order_book_state().await?;
        let order_types_tests = self.test_order_types().await?;
        let performance_metrics = self.measure_performance_metrics().await?;
        
        // Calculate overall score
        let overall_score = self.calculate_overall_score(
            &order_placement_tests,
            &order_modification_tests,
            &order_cancellation_tests,
            &order_book_state_tests,
            &order_types_tests,
        );
        
        // Determine pass/fail status
        let pass_fail_status = self.determine_pass_fail_status(overall_score);
        
        let results = OrderBookTestResults {
            test_id: self.test_id,
            timestamp: Utc::now(),
            order_placement_tests,
            order_modification_tests,
            order_cancellation_tests,
            order_book_state_tests,
            order_types_tests,
            performance_metrics,
            overall_score,
            pass_fail_status,
        };
        
        info!("✅ Order book management tests completed");
        info!("Overall Score: {:.2}%", results.overall_score);
        info!("Status: {:?}", results.pass_fail_status);
        
        Ok(results)
    }

    /// Test order placement functionality
    async fn test_order_placement(&self) -> Result<OrderPlacementResults, Box<dyn std::error::Error>> {
        info!("📋 Testing order placement functionality...");
        
        let mut placement_latencies = Vec::new();
        let mut market_orders_success = 0;
        let mut limit_orders_success = 0;
        let mut stop_loss_orders_success = 0;
        let mut take_profit_orders_success = 0;
        let total_orders_per_type = 50;
        
        // Test market orders
        for i in 0..total_orders_per_type {
            let start_time = Instant::now();
            
            let order = self.create_test_market_order(OrderSide::Buy, 100.0).await?;
            let result = self.place_order(order).await;
            
            let latency = start_time.elapsed().as_micros() as f64 / 1000.0;
            placement_latencies.push(latency);
            
            if result.is_ok() {
                market_orders_success += 1;
            }
            
            debug!("Market order {}: {:?}, latency: {:.2}ms", i + 1, result.is_ok(), latency);
            
            // Small delay to avoid overwhelming the system
            sleep(Duration::from_millis(10)).await;
        }
        
        // Test limit orders
        for i in 0..total_orders_per_type {
            let start_time = Instant::now();
            
            let order = self.create_test_limit_order(OrderSide::Buy, 100.0, 1.1234).await?;
            let result = self.place_order(order).await;
            
            let latency = start_time.elapsed().as_micros() as f64 / 1000.0;
            placement_latencies.push(latency);
            
            if result.is_ok() {
                limit_orders_success += 1;
            }
            
            debug!("Limit order {}: {:?}, latency: {:.2}ms", i + 1, result.is_ok(), latency);
            sleep(Duration::from_millis(10)).await;
        }
        
        // Test stop-loss orders
        for i in 0..total_orders_per_type {
            let start_time = Instant::now();
            
            let order = self.create_test_stop_loss_order(OrderSide::Sell, 100.0, 1.1200).await?;
            let result = self.place_order(order).await;
            
            let latency = start_time.elapsed().as_micros() as f64 / 1000.0;
            placement_latencies.push(latency);
            
            if result.is_ok() {
                stop_loss_orders_success += 1;
            }
            
            debug!("Stop-loss order {}: {:?}, latency: {:.2}ms", i + 1, result.is_ok(), latency);
            sleep(Duration::from_millis(10)).await;
        }
        
        // Test take-profit orders
        for i in 0..total_orders_per_type {
            let start_time = Instant::now();
            
            let order = self.create_test_take_profit_order(OrderSide::Sell, 100.0, 1.1300).await?;
            let result = self.place_order(order).await;
            
            let latency = start_time.elapsed().as_micros() as f64 / 1000.0;
            placement_latencies.push(latency);
            
            if result.is_ok() {
                take_profit_orders_success += 1;
            }
            
            debug!("Take-profit order {}: {:?}, latency: {:.2}ms", i + 1, result.is_ok(), latency);
            sleep(Duration::from_millis(10)).await;
        }
        
        // Calculate results
        let market_orders_success_rate = market_orders_success as f64 / total_orders_per_type as f64;
        let limit_orders_success_rate = limit_orders_success as f64 / total_orders_per_type as f64;
        let stop_loss_orders_success_rate = stop_loss_orders_success as f64 / total_orders_per_type as f64;
        let take_profit_orders_success_rate = take_profit_orders_success as f64 / total_orders_per_type as f64;
        let average_placement_latency_ms = placement_latencies.iter().sum::<f64>() / placement_latencies.len() as f64;
        let total_orders_placed = (total_orders_per_type * 4) as u64;
        
        let status = if market_orders_success_rate > 0.95 && 
                        limit_orders_success_rate > 0.95 && 
                        stop_loss_orders_success_rate > 0.90 && 
                        take_profit_orders_success_rate > 0.90 {
            TestStatus::Passed
        } else if market_orders_success_rate > 0.80 && limit_orders_success_rate > 0.80 {
            TestStatus::PartiallyPassed
        } else {
            TestStatus::Failed
        };
        
        info!("📋 Order placement test results:");
        info!("  • Market orders success rate: {:.2}%", market_orders_success_rate * 100.0);
        info!("  • Limit orders success rate: {:.2}%", limit_orders_success_rate * 100.0);
        info!("  • Stop-loss orders success rate: {:.2}%", stop_loss_orders_success_rate * 100.0);
        info!("  • Take-profit orders success rate: {:.2}%", take_profit_orders_success_rate * 100.0);
        info!("  • Average placement latency: {:.2}ms", average_placement_latency_ms);
        info!("  • Total orders placed: {}", total_orders_placed);
        info!("  • Status: {:?}", status);
        
        Ok(OrderPlacementResults {
            market_orders_success_rate,
            limit_orders_success_rate,
            stop_loss_orders_success_rate,
            take_profit_orders_success_rate,
            average_placement_latency_ms,
            total_orders_placed,
            status,
        })
    }

    /// Test order modification functionality
    async fn test_order_modification(&self) -> Result<OrderModificationResults, Box<dyn std::error::Error>> {
        info!("✏️ Testing order modification functionality...");

        let mut modification_latencies = Vec::new();
        let mut price_modifications_success = 0;
        let mut quantity_modifications_success = 0;
        let mut stop_loss_modifications_success = 0;
        let mut take_profit_modifications_success = 0;
        let total_modifications_per_type = 30;

        // Test price modifications
        for i in 0..total_modifications_per_type {
            // First place an order
            let order = self.create_test_limit_order(OrderSide::Buy, 100.0, 1.1234).await?;
            let order_id = self.place_order(order).await?;

            // Then modify its price
            let start_time = Instant::now();
            let result = self.modify_order_price(order_id, 1.1240).await;
            let latency = start_time.elapsed().as_micros() as f64 / 1000.0;
            modification_latencies.push(latency);

            if result.is_ok() {
                price_modifications_success += 1;
            }

            debug!("Price modification {}: {:?}, latency: {:.2}ms", i + 1, result.is_ok(), latency);
            sleep(Duration::from_millis(10)).await;
        }

        // Test quantity modifications
        for i in 0..total_modifications_per_type {
            let order = self.create_test_limit_order(OrderSide::Buy, 100.0, 1.1234).await?;
            let order_id = self.place_order(order).await?;

            let start_time = Instant::now();
            let result = self.modify_order_quantity(order_id, 150.0).await;
            let latency = start_time.elapsed().as_micros() as f64 / 1000.0;
            modification_latencies.push(latency);

            if result.is_ok() {
                quantity_modifications_success += 1;
            }

            debug!("Quantity modification {}: {:?}, latency: {:.2}ms", i + 1, result.is_ok(), latency);
            sleep(Duration::from_millis(10)).await;
        }

        // Test stop-loss modifications
        for i in 0..total_modifications_per_type {
            let order = self.create_test_stop_loss_order(OrderSide::Sell, 100.0, 1.1200).await?;
            let order_id = self.place_order(order).await?;

            let start_time = Instant::now();
            let result = self.modify_stop_loss_price(order_id, 1.1190).await;
            let latency = start_time.elapsed().as_micros() as f64 / 1000.0;
            modification_latencies.push(latency);

            if result.is_ok() {
                stop_loss_modifications_success += 1;
            }

            debug!("Stop-loss modification {}: {:?}, latency: {:.2}ms", i + 1, result.is_ok(), latency);
            sleep(Duration::from_millis(10)).await;
        }

        // Test take-profit modifications
        for i in 0..total_modifications_per_type {
            let order = self.create_test_take_profit_order(OrderSide::Sell, 100.0, 1.1300).await?;
            let order_id = self.place_order(order).await?;

            let start_time = Instant::now();
            let result = self.modify_take_profit_price(order_id, 1.1310).await;
            let latency = start_time.elapsed().as_micros() as f64 / 1000.0;
            modification_latencies.push(latency);

            if result.is_ok() {
                take_profit_modifications_success += 1;
            }

            debug!("Take-profit modification {}: {:?}, latency: {:.2}ms", i + 1, result.is_ok(), latency);
            sleep(Duration::from_millis(10)).await;
        }

        // Calculate results
        let price_modification_success_rate = price_modifications_success as f64 / total_modifications_per_type as f64;
        let quantity_modification_success_rate = quantity_modifications_success as f64 / total_modifications_per_type as f64;
        let stop_loss_modification_success_rate = stop_loss_modifications_success as f64 / total_modifications_per_type as f64;
        let take_profit_modification_success_rate = take_profit_modifications_success as f64 / total_modifications_per_type as f64;
        let average_modification_latency_ms = modification_latencies.iter().sum::<f64>() / modification_latencies.len() as f64;
        let total_modifications = (total_modifications_per_type * 4) as u64;

        let status = if price_modification_success_rate > 0.90 &&
                        quantity_modification_success_rate > 0.90 &&
                        stop_loss_modification_success_rate > 0.85 &&
                        take_profit_modification_success_rate > 0.85 {
            TestStatus::Passed
        } else if price_modification_success_rate > 0.75 && quantity_modification_success_rate > 0.75 {
            TestStatus::PartiallyPassed
        } else {
            TestStatus::Failed
        };

        info!("✏️ Order modification test results:");
        info!("  • Price modification success rate: {:.2}%", price_modification_success_rate * 100.0);
        info!("  • Quantity modification success rate: {:.2}%", quantity_modification_success_rate * 100.0);
        info!("  • Stop-loss modification success rate: {:.2}%", stop_loss_modification_success_rate * 100.0);
        info!("  • Take-profit modification success rate: {:.2}%", take_profit_modification_success_rate * 100.0);
        info!("  • Average modification latency: {:.2}ms", average_modification_latency_ms);
        info!("  • Total modifications: {}", total_modifications);
        info!("  • Status: {:?}", status);

        Ok(OrderModificationResults {
            price_modification_success_rate,
            quantity_modification_success_rate,
            stop_loss_modification_success_rate,
            take_profit_modification_success_rate,
            average_modification_latency_ms,
            total_modifications,
            status,
        })
    }

    /// Test order cancellation functionality
    async fn test_order_cancellation(&self) -> Result<OrderCancellationResults, Box<dyn std::error::Error>> {
        info!("❌ Testing order cancellation functionality...");

        let mut cancellation_latencies = Vec::new();
        let mut immediate_cancellations_success = 0;
        let mut partial_fill_cancellations_success = 0;
        let mut bulk_cancellations_success = 0;
        let total_cancellations_per_type = 30;

        // Test immediate cancellations (orders that haven't been filled)
        for i in 0..total_cancellations_per_type {
            let order = self.create_test_limit_order(OrderSide::Buy, 100.0, 1.1000).await?; // Far from market
            let order_id = self.place_order(order).await?;

            // Small delay to ensure order is placed
            sleep(Duration::from_millis(5)).await;

            let start_time = Instant::now();
            let result = self.cancel_order(order_id).await;
            let latency = start_time.elapsed().as_micros() as f64 / 1000.0;
            cancellation_latencies.push(latency);

            if result.is_ok() {
                immediate_cancellations_success += 1;
            }

            debug!("Immediate cancellation {}: {:?}, latency: {:.2}ms", i + 1, result.is_ok(), latency);
            sleep(Duration::from_millis(10)).await;
        }

        // Test partial fill cancellations (simulate partially filled orders)
        for i in 0..total_cancellations_per_type {
            let order = self.create_test_limit_order(OrderSide::Buy, 200.0, 1.1235).await?; // Near market
            let order_id = self.place_order(order).await?;

            // Simulate partial fill
            self.simulate_partial_fill(order_id, 50.0).await?;

            let start_time = Instant::now();
            let result = self.cancel_order(order_id).await;
            let latency = start_time.elapsed().as_micros() as f64 / 1000.0;
            cancellation_latencies.push(latency);

            if result.is_ok() {
                partial_fill_cancellations_success += 1;
            }

            debug!("Partial fill cancellation {}: {:?}, latency: {:.2}ms", i + 1, result.is_ok(), latency);
            sleep(Duration::from_millis(10)).await;
        }

        // Test bulk cancellations
        let bulk_test_iterations = 10;
        for i in 0..bulk_test_iterations {
            // Place multiple orders
            let mut order_ids = Vec::new();
            for _ in 0..5 {
                let order = self.create_test_limit_order(OrderSide::Buy, 100.0, 1.1000).await?;
                let order_id = self.place_order(order).await?;
                order_ids.push(order_id);
            }

            let start_time = Instant::now();
            let result = self.cancel_orders_bulk(order_ids).await;
            let latency = start_time.elapsed().as_micros() as f64 / 1000.0;
            cancellation_latencies.push(latency);

            if result.is_ok() {
                bulk_cancellations_success += 1;
            }

            debug!("Bulk cancellation {}: {:?}, latency: {:.2}ms", i + 1, result.is_ok(), latency);
            sleep(Duration::from_millis(20)).await;
        }

        // Calculate results
        let immediate_cancellation_success_rate = immediate_cancellations_success as f64 / total_cancellations_per_type as f64;
        let partial_fill_cancellation_success_rate = partial_fill_cancellations_success as f64 / total_cancellations_per_type as f64;
        let bulk_cancellation_success_rate = bulk_cancellations_success as f64 / bulk_test_iterations as f64;
        let average_cancellation_latency_ms = cancellation_latencies.iter().sum::<f64>() / cancellation_latencies.len() as f64;
        let total_cancellations = (total_cancellations_per_type * 2 + bulk_test_iterations) as u64;

        let status = if immediate_cancellation_success_rate > 0.95 &&
                        partial_fill_cancellation_success_rate > 0.90 &&
                        bulk_cancellation_success_rate > 0.90 {
            TestStatus::Passed
        } else if immediate_cancellation_success_rate > 0.85 && partial_fill_cancellation_success_rate > 0.80 {
            TestStatus::PartiallyPassed
        } else {
            TestStatus::Failed
        };

        info!("❌ Order cancellation test results:");
        info!("  • Immediate cancellation success rate: {:.2}%", immediate_cancellation_success_rate * 100.0);
        info!("  • Partial fill cancellation success rate: {:.2}%", partial_fill_cancellation_success_rate * 100.0);
        info!("  • Bulk cancellation success rate: {:.2}%", bulk_cancellation_success_rate * 100.0);
        info!("  • Average cancellation latency: {:.2}ms", average_cancellation_latency_ms);
        info!("  • Total cancellations: {}", total_cancellations);
        info!("  • Status: {:?}", status);

        Ok(OrderCancellationResults {
            immediate_cancellation_success_rate,
            partial_fill_cancellation_success_rate,
            bulk_cancellation_success_rate,
            average_cancellation_latency_ms,
            total_cancellations,
            status,
        })
    }

    /// Test order book state management
    async fn test_order_book_state(&self) -> Result<OrderBookStateResults, Box<dyn std::error::Error>> {
        info!("📊 Testing order book state management...");

        // Test state consistency
        let state_consistency_score = self.test_state_consistency().await?;

        // Test real-time updates
        let real_time_updates_accuracy = self.test_real_time_updates().await?;

        // Test order matching
        let order_matching_accuracy = self.test_order_matching().await?;

        // Test price level integrity
        let price_level_integrity_score = self.test_price_level_integrity().await?;

        // Test volume tracking
        let volume_tracking_accuracy = self.test_volume_tracking().await?;

        let status = if state_consistency_score > 0.95 &&
                        real_time_updates_accuracy > 0.90 &&
                        order_matching_accuracy > 0.95 {
            TestStatus::Passed
        } else if state_consistency_score > 0.85 && real_time_updates_accuracy > 0.80 {
            TestStatus::PartiallyPassed
        } else {
            TestStatus::Failed
        };

        info!("📊 Order book state test results:");
        info!("  • State consistency score: {:.2}%", state_consistency_score * 100.0);
        info!("  • Real-time updates accuracy: {:.2}%", real_time_updates_accuracy * 100.0);
        info!("  • Order matching accuracy: {:.2}%", order_matching_accuracy * 100.0);
        info!("  • Price level integrity score: {:.2}%", price_level_integrity_score * 100.0);
        info!("  • Volume tracking accuracy: {:.2}%", volume_tracking_accuracy * 100.0);
        info!("  • Status: {:?}", status);

        Ok(OrderBookStateResults {
            state_consistency_score,
            real_time_updates_accuracy,
            order_matching_accuracy,
            price_level_integrity_score,
            volume_tracking_accuracy,
            status,
        })
    }

    /// Test different order types
    async fn test_order_types(&self) -> Result<OrderTypesResults, Box<dyn std::error::Error>> {
        info!("🔄 Testing different order types...");

        // Test market order execution
        let market_order_execution_quality = self.test_market_order_execution().await?;

        // Test limit order execution
        let limit_order_execution_quality = self.test_limit_order_execution().await?;

        // Test stop-loss trigger accuracy
        let stop_loss_trigger_accuracy = self.test_stop_loss_triggers().await?;

        // Test take-profit trigger accuracy
        let take_profit_trigger_accuracy = self.test_take_profit_triggers().await?;

        // Test stop-limit execution
        let stop_limit_execution_quality = self.test_stop_limit_execution().await?;

        // Test iceberg order handling
        let iceberg_order_handling = self.test_iceberg_orders().await?;

        let status = if market_order_execution_quality > 0.90 &&
                        limit_order_execution_quality > 0.85 &&
                        stop_loss_trigger_accuracy > 0.85 {
            TestStatus::Passed
        } else if market_order_execution_quality > 0.80 && limit_order_execution_quality > 0.75 {
            TestStatus::PartiallyPassed
        } else {
            TestStatus::Failed
        };

        info!("🔄 Order types test results:");
        info!("  • Market order execution quality: {:.2}%", market_order_execution_quality * 100.0);
        info!("  • Limit order execution quality: {:.2}%", limit_order_execution_quality * 100.0);
        info!("  • Stop-loss trigger accuracy: {:.2}%", stop_loss_trigger_accuracy * 100.0);
        info!("  • Take-profit trigger accuracy: {:.2}%", take_profit_trigger_accuracy * 100.0);
        info!("  • Stop-limit execution quality: {:.2}%", stop_limit_execution_quality * 100.0);
        info!("  • Iceberg order handling: {:.2}%", iceberg_order_handling * 100.0);
        info!("  • Status: {:?}", status);

        Ok(OrderTypesResults {
            market_order_execution_quality,
            limit_order_execution_quality,
            stop_loss_trigger_accuracy,
            take_profit_trigger_accuracy,
            stop_limit_execution_quality,
            iceberg_order_handling,
            status,
        })
    }

    /// Measure performance metrics
    async fn measure_performance_metrics(&self) -> Result<OrderBookPerformanceMetrics, Box<dyn std::error::Error>> {
        info!("⚡ Measuring order book performance metrics...");

        // Measure throughput
        let order_processing_throughput_ops = self.measure_order_processing_throughput().await?;

        // Measure memory efficiency
        let memory_usage_efficiency = self.measure_memory_usage_efficiency().await?;

        // Measure CPU utilization
        let cpu_utilization_under_load = self.measure_cpu_utilization().await?;

        // Measure latency percentiles
        let latency_percentiles = self.measure_latency_percentiles().await?;

        // Measure error rate
        let error_rate_percentage = self.measure_error_rate().await?;

        info!("⚡ Performance metrics results:");
        info!("  • Order processing throughput: {:.2} ops/sec", order_processing_throughput_ops);
        info!("  • Memory usage efficiency: {:.2}%", memory_usage_efficiency * 100.0);
        info!("  • CPU utilization under load: {:.2}%", cpu_utilization_under_load * 100.0);
        info!("  • P50 latency: {:.2}ms", latency_percentiles.p50_ms);
        info!("  • P95 latency: {:.2}ms", latency_percentiles.p95_ms);
        info!("  • P99 latency: {:.2}ms", latency_percentiles.p99_ms);
        info!("  • Error rate: {:.2}%", error_rate_percentage);

        Ok(OrderBookPerformanceMetrics {
            order_processing_throughput_ops,
            memory_usage_efficiency,
            cpu_utilization_under_load,
            latency_percentiles,
            error_rate_percentage,
        })
    }

    /// Calculate overall score
    fn calculate_overall_score(
        &self,
        placement: &OrderPlacementResults,
        modification: &OrderModificationResults,
        cancellation: &OrderCancellationResults,
        state: &OrderBookStateResults,
        types: &OrderTypesResults,
    ) -> f64 {
        let placement_score = self.get_test_score(&placement.status) * 0.25;
        let modification_score = self.get_test_score(&modification.status) * 0.20;
        let cancellation_score = self.get_test_score(&cancellation.status) * 0.20;
        let state_score = self.get_test_score(&state.status) * 0.20;
        let types_score = self.get_test_score(&types.status) * 0.15;

        (placement_score + modification_score + cancellation_score + state_score + types_score) * 100.0
    }

    /// Determine pass/fail status
    fn determine_pass_fail_status(&self, overall_score: f64) -> TestStatus {
        if overall_score >= 85.0 {
            TestStatus::Passed
        } else if overall_score >= 70.0 {
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
