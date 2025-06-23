// Performance Validation Module for Trading Engine Optimizations
// Validates that all performance optimizations are working correctly

use crate::trading::engine::create_optimized_trading_engine;
use crate::trading::signals::OrderRequest;
use crate::database::types::SignalType;
use crate::database::Database;
use crate::utils::Result;
use chrono::Utc;
use serde::{Serialize, Deserialize};
use std::time::Instant;
use tracing::info;
use uuid::Uuid;

/// Performance validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceValidationResults {
    pub async_processing_score: f64,
    pub adaptive_batching_score: f64,
    pub lock_free_structures_score: f64,
    pub overall_latency_ms: f64,
    pub overall_throughput_tps: f64,
    pub optimization_effectiveness: f64,
    pub meets_requirements: bool,
    pub recommendations: Vec<String>,
}

/// Performance validation configuration
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    pub test_order_count: usize,
    pub target_latency_ms: f64,
    pub target_throughput_tps: f64,
    pub concurrent_tasks: usize,
    pub validation_timeout_seconds: u64,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            test_order_count: 1000,
            target_latency_ms: 10.0,
            target_throughput_tps: 2000.0,
            concurrent_tasks: 20,
            validation_timeout_seconds: 300, // 5 minutes
        }
    }
}

/// Performance validator
pub struct PerformanceValidator {
    config: ValidationConfig,
    database: Database,
}

impl PerformanceValidator {
    pub fn new(config: ValidationConfig, database: Database) -> Self {
        Self { config, database }
    }

    /// Run comprehensive performance validation
    pub async fn validate_performance(&self) -> Result<PerformanceValidationResults> {
        info!("🔍 Starting comprehensive performance validation...");
        
        let start_time = Instant::now();
        
        // 1. Validate async processing optimizations
        let async_score = self.validate_async_processing().await?;
        
        // 2. Validate adaptive batching system
        let batching_score = self.validate_adaptive_batching().await?;
        
        // 3. Validate lock-free structures
        let lock_free_score = self.validate_lock_free_structures().await?;
        
        // 4. Overall performance test
        let (overall_latency, overall_throughput) = self.validate_overall_performance().await?;
        
        // Calculate optimization effectiveness
        let optimization_effectiveness = (async_score + batching_score + lock_free_score) / 3.0;
        
        // Check if requirements are met
        let meets_latency = overall_latency <= self.config.target_latency_ms;
        let meets_throughput = overall_throughput >= self.config.target_throughput_tps;
        let meets_requirements = meets_latency && meets_throughput && optimization_effectiveness >= 0.8;
        
        // Generate recommendations
        let recommendations = self.generate_recommendations(
            async_score, batching_score, lock_free_score, 
            overall_latency, overall_throughput
        );
        
        let total_time = start_time.elapsed();
        info!("✅ Performance validation completed in {:.2}s", total_time.as_secs_f64());
        
        Ok(PerformanceValidationResults {
            async_processing_score: async_score,
            adaptive_batching_score: batching_score,
            lock_free_structures_score: lock_free_score,
            overall_latency_ms: overall_latency,
            overall_throughput_tps: overall_throughput,
            optimization_effectiveness,
            meets_requirements,
            recommendations,
        })
    }

    /// Validate async processing optimizations
    async fn validate_async_processing(&self) -> Result<f64> {
        info!("  🔄 Validating async processing optimizations...");
        
        let engine = create_optimized_trading_engine(self.database.clone()).await?;
        engine.start_async_order_pipeline().await?;
        
        let test_orders = self.generate_test_orders(500);
        let start_time = Instant::now();
        
        // Test concurrent order processing
        let mut handles = Vec::new();
        for order in test_orders {
            let engine_clone = engine.clone();
            let handle = tokio::spawn(async move {
                let order_start = Instant::now();
                let result = engine_clone.process_order_optimized(order).await;
                let latency = order_start.elapsed().as_millis() as f64;
                (result.is_ok(), latency)
            });
            handles.push(handle);
        }
        
        let mut successful = 0;
        let mut total_latency = 0.0;
        for handle in handles {
            if let Ok(result) = handle.await {
                let (success, latency) = result;
                if success {
                    successful += 1;
                    total_latency += latency;
                }
            }
        }
        
        let processing_time = start_time.elapsed();
        let throughput = successful as f64 / processing_time.as_secs_f64();
        let avg_latency = if successful > 0 { total_latency / successful as f64 } else { 0.0 };
        
        // Score based on throughput and latency
        let throughput_score = (throughput / 1000.0).min(1.0); // Normalize to 1000 TPS
        let latency_score = if avg_latency <= 10.0 { 1.0 } else { 10.0 / avg_latency };
        let success_rate = successful as f64 / 500.0;
        
        let score = (throughput_score * 0.4 + latency_score * 0.4 + success_rate * 0.2).min(1.0);
        
        info!("    Async Processing: {:.2} TPS, {:.2}ms avg latency, {:.1}% success, Score: {:.2}", 
              throughput, avg_latency, success_rate * 100.0, score);
        
        Ok(score)
    }

    /// Validate adaptive batching system
    async fn validate_adaptive_batching(&self) -> Result<f64> {
        info!("  📦 Validating adaptive batching system...");
        
        let engine = create_optimized_trading_engine(self.database.clone()).await?;
        let test_orders = self.generate_test_orders(1000);
        let start_time = Instant::now();
        
        // Submit orders through adaptive batching
        let mut successful = 0;
        for order in test_orders {
            if engine.submit_order_adaptive(order).await.is_ok() {
                successful += 1;
            }
        }
        
        let processing_time = start_time.elapsed();
        let throughput = successful as f64 / processing_time.as_secs_f64();
        
        // Get adaptive batching statistics
        let batching_stats = engine.get_adaptive_batching_stats().await;
        let batch_efficiency = if let Some(stats) = batching_stats {
            let size_efficiency = if stats.average_batch_size > 0 {
                (stats.average_batch_size as f64 / 100.0).min(1.0) // Normalize to 100
            } else {
                0.0
            };
            
            let latency_efficiency = if stats.average_latency_ms <= 10.0 {
                1.0
            } else {
                10.0 / stats.average_latency_ms
            };
            
            (size_efficiency + latency_efficiency) / 2.0
        } else {
            0.0
        };
        
        let throughput_score = (throughput / 1500.0).min(1.0); // Normalize to 1500 TPS
        let success_rate = successful as f64 / 1000.0;
        
        let score = (throughput_score * 0.4 + batch_efficiency * 0.4 + success_rate * 0.2).min(1.0);
        
        info!("    Adaptive Batching: {:.2} TPS, {:.1}% success, Score: {:.2}", 
              throughput, success_rate * 100.0, score);
        
        Ok(score)
    }

    /// Validate lock-free structures
    async fn validate_lock_free_structures(&self) -> Result<f64> {
        info!("  🔓 Validating lock-free structures...");
        
        let engine = create_optimized_trading_engine(self.database.clone()).await?;
        let test_orders = self.generate_test_orders(2000);
        let start_time = Instant::now();
        
        // Test lock-free queue performance
        let mut successful = 0;
        for order in test_orders {
            if engine.submit_order_lock_free(order).await.is_ok() {
                successful += 1;
            }
        }
        
        let processing_time = start_time.elapsed();
        let throughput = successful as f64 / processing_time.as_secs_f64();
        
        // Get lock-free statistics
        let queue_efficiency = if let Some((depth, capacity)) = engine.get_lock_free_queue_stats() {
            if capacity > 0 {
                1.0 - (depth as f64 / capacity as f64) // Lower queue depth is better
            } else {
                0.0
            }
        } else {
            0.0
        };
        
        let memory_efficiency = if let Some(pool_stats) = engine.get_memory_pool_stats() {
            // Parse hit rate from stats string (simplified)
            0.8 // Placeholder - in real implementation, parse the actual hit rate
        } else {
            0.0
        };
        
        let throughput_score = (throughput / 2000.0).min(1.0); // Normalize to 2000 TPS
        let success_rate = successful as f64 / 2000.0;
        
        let score = (throughput_score * 0.3 + queue_efficiency * 0.3 + memory_efficiency * 0.2 + success_rate * 0.2).min(1.0);
        
        info!("    Lock-Free: {:.2} TPS, {:.1}% success, Score: {:.2}", 
              throughput, success_rate * 100.0, score);
        
        Ok(score)
    }

    /// Validate overall performance
    async fn validate_overall_performance(&self) -> Result<(f64, f64)> {
        info!("  🎯 Validating overall performance...");
        
        let engine = create_optimized_trading_engine(self.database.clone()).await?;
        let test_orders = self.generate_test_orders(self.config.test_order_count);
        
        let start_time = Instant::now();
        let mut latencies = Vec::new();
        let mut successful = 0;
        
        // Process orders and measure latency
        for order in test_orders {
            let order_start = Instant::now();
            
            if engine.process_order_optimized(order).await.is_ok() {
                successful += 1;
            }
            
            let latency_ms = order_start.elapsed().as_micros() as f64 / 1000.0;
            latencies.push(latency_ms);
        }
        
        let total_time = start_time.elapsed();
        let throughput = successful as f64 / total_time.as_secs_f64();
        
        // Calculate latency statistics
        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p95_latency = if !latencies.is_empty() {
            latencies[(latencies.len() * 95) / 100]
        } else {
            0.0
        };
        
        info!("    Overall: {:.2} TPS, {:.2}ms P95 latency", throughput, p95_latency);
        
        Ok((p95_latency, throughput))
    }

    /// Generate performance recommendations
    fn generate_recommendations(
        &self,
        async_score: f64,
        batching_score: f64,
        lock_free_score: f64,
        latency: f64,
        throughput: f64,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if async_score < 0.8 {
            recommendations.push("Consider increasing max_concurrent_orders for better async processing".to_string());
        }
        
        if batching_score < 0.8 {
            recommendations.push("Tune adaptive batching parameters for better batch efficiency".to_string());
        }
        
        if lock_free_score < 0.8 {
            recommendations.push("Increase lock-free queue capacity or memory pool size".to_string());
        }
        
        if latency > self.config.target_latency_ms {
            recommendations.push(format!("Latency ({:.2}ms) exceeds target ({:.2}ms) - optimize critical path", 
                                       latency, self.config.target_latency_ms));
        }
        
        if throughput < self.config.target_throughput_tps {
            recommendations.push(format!("Throughput ({:.2} TPS) below target ({:.2} TPS) - scale processing", 
                                       throughput, self.config.target_throughput_tps));
        }
        
        if recommendations.is_empty() {
            recommendations.push("All performance targets met - system is optimally configured".to_string());
        }
        
        recommendations
    }

    /// Generate test orders for validation
    fn generate_test_orders(&self, count: usize) -> Vec<OrderRequest> {
        let mut orders = Vec::with_capacity(count);
        
        for i in 0..count {
            let order = OrderRequest {
                instrument_id: Uuid::new_v4(),
                side: if i % 2 == 0 { SignalType::Buy } else { SignalType::Sell },
                quantity: 1000.0 + (i as f64 * 10.0),
                order_type: crate::database::types::OrderType::Market,
                price: None,
                time_in_force: crate::database::types::TimeInForce::IOC,
            };
            orders.push(order);
        }
        
        orders
    }
}
