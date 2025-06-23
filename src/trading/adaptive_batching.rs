// Adaptive Batching System for PantherSwap Edge Trading Engine
// Dynamically adjusts batch sizes based on current load and performance metrics

use crate::trading::signals::OrderRequest;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Instant, Duration};
use tokio::sync::RwLock;
use tracing::{info, debug};
use uuid::Uuid;

/// Configuration for adaptive batching system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveBatchingConfig {
    pub min_batch_size: usize,
    pub max_batch_size: usize,
    pub initial_batch_size: usize,
    pub target_latency_ms: f64,
    pub max_wait_time_ms: u64,
    pub load_threshold_high: f64,
    pub load_threshold_low: f64,
    pub batch_size_increment: usize,
    pub batch_size_decrement: usize,
    pub performance_window_size: usize,
    pub enable_predictive_sizing: bool,
    pub enable_load_balancing: bool,
}

impl Default for AdaptiveBatchingConfig {
    fn default() -> Self {
        Self {
            min_batch_size: 10,
            max_batch_size: 200,
            initial_batch_size: 50,
            target_latency_ms: 8.0, // Target <10ms with buffer
            max_wait_time_ms: 5, // 5ms max wait for batch formation
            load_threshold_high: 0.8, // 80% load threshold
            load_threshold_low: 0.3, // 30% load threshold
            batch_size_increment: 10,
            batch_size_decrement: 5,
            performance_window_size: 100,
            enable_predictive_sizing: true,
            enable_load_balancing: true,
        }
    }
}

/// Batch performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchMetrics {
    pub batch_id: Uuid,
    pub batch_size: usize,
    pub processing_latency_ms: f64,
    pub throughput_tps: f64,
    pub success_rate: f64,
    pub queue_depth_at_start: usize,
    pub timestamp: DateTime<Utc>,
}

/// Load metrics for adaptive sizing
#[derive(Debug)]
pub struct LoadMetrics {
    pub current_queue_depth: AtomicUsize,
    pub orders_per_second: AtomicU64,
    pub average_processing_time_ms: Arc<RwLock<f64>>,
    pub cpu_utilization: Arc<RwLock<f64>>,
    pub memory_pressure: Arc<RwLock<f64>>,
}

/// Adaptive batch processor
pub struct AdaptiveBatchProcessor {
    config: AdaptiveBatchingConfig,
    current_batch_size: AtomicUsize,
    load_metrics: LoadMetrics,
    performance_history: Arc<RwLock<VecDeque<BatchMetrics>>>,
    batch_buffer: Arc<RwLock<Vec<OrderRequest>>>,
    last_batch_time: Arc<RwLock<Instant>>,
    total_batches_processed: AtomicU64,
    optimization_counter: AtomicU64,
}

impl AdaptiveBatchProcessor {
    /// Create new adaptive batch processor
    pub fn new(config: AdaptiveBatchingConfig) -> Self {
        Self {
            current_batch_size: AtomicUsize::new(config.initial_batch_size),
            config,
            load_metrics: LoadMetrics {
                current_queue_depth: AtomicUsize::new(0),
                orders_per_second: AtomicU64::new(0),
                average_processing_time_ms: Arc::new(RwLock::new(0.0)),
                cpu_utilization: Arc::new(RwLock::new(0.0)),
                memory_pressure: Arc::new(RwLock::new(0.0)),
            },
            performance_history: Arc::new(RwLock::new(VecDeque::new())),
            batch_buffer: Arc::new(RwLock::new(Vec::new())),
            last_batch_time: Arc::new(RwLock::new(Instant::now())),
            total_batches_processed: AtomicU64::new(0),
            optimization_counter: AtomicU64::new(0),
        }
    }

    /// Add order to batch buffer and check if batch should be processed
    pub async fn add_order(&self, order: OrderRequest) -> Option<Vec<OrderRequest>> {
        let mut buffer = self.batch_buffer.write().await;
        buffer.push(order);
        
        // Update queue depth
        self.load_metrics.current_queue_depth.store(buffer.len(), Ordering::Relaxed);
        
        let current_batch_size = self.current_batch_size.load(Ordering::Relaxed);
        let last_batch_time = *self.last_batch_time.read().await;
        let max_wait_duration = Duration::from_millis(self.config.max_wait_time_ms);
        
        // Check if batch should be processed
        let should_process = buffer.len() >= current_batch_size 
            || last_batch_time.elapsed() >= max_wait_duration
            || self.should_force_batch_processing().await;
        
        if should_process && !buffer.is_empty() {
            let batch = buffer.drain(..).collect();
            *self.last_batch_time.write().await = Instant::now();
            Some(batch)
        } else {
            None
        }
    }

    /// Record batch performance and adapt batch size
    pub async fn record_batch_performance(&self, metrics: BatchMetrics) {
        // Add to performance history
        {
            let mut history = self.performance_history.write().await;
            history.push_back(metrics.clone());
            
            // Trim history to window size
            if history.len() > self.config.performance_window_size {
                history.pop_front();
            }
        }
        
        // Update load metrics
        *self.load_metrics.average_processing_time_ms.write().await = metrics.processing_latency_ms;
        
        // Increment batch counter
        self.total_batches_processed.fetch_add(1, Ordering::Relaxed);
        
        // Perform adaptive optimization
        if self.optimization_counter.fetch_add(1, Ordering::Relaxed) % 10 == 0 {
            self.optimize_batch_size().await;
        }
        
        debug!("Batch performance recorded: size={}, latency={:.2}ms, throughput={:.2} TPS", 
               metrics.batch_size, metrics.processing_latency_ms, metrics.throughput_tps);
    }

    /// Optimize batch size based on performance history and current load
    async fn optimize_batch_size(&self) {
        let history = self.performance_history.read().await;
        if history.len() < 5 {
            return; // Need more data
        }
        
        let recent_metrics: Vec<_> = history.iter().rev().take(10).collect();
        let avg_latency = recent_metrics.iter().map(|m| m.processing_latency_ms).sum::<f64>() / recent_metrics.len() as f64;
        let avg_throughput = recent_metrics.iter().map(|m| m.throughput_tps).sum::<f64>() / recent_metrics.len() as f64;
        let avg_success_rate = recent_metrics.iter().map(|m| m.success_rate).sum::<f64>() / recent_metrics.len() as f64;
        
        let current_batch_size = self.current_batch_size.load(Ordering::Relaxed);
        let current_load = self.calculate_current_load().await;
        
        let new_batch_size = if self.config.enable_predictive_sizing {
            self.calculate_optimal_batch_size(avg_latency, avg_throughput, current_load).await
        } else {
            self.calculate_reactive_batch_size(avg_latency, current_load).await
        };
        
        if new_batch_size != current_batch_size {
            self.current_batch_size.store(new_batch_size, Ordering::Relaxed);
            info!("Adaptive batching: adjusted batch size from {} to {} (latency={:.2}ms, throughput={:.2} TPS, load={:.2})", 
                  current_batch_size, new_batch_size, avg_latency, avg_throughput, current_load);
        }
    }

    /// Calculate optimal batch size using predictive algorithm
    async fn calculate_optimal_batch_size(&self, avg_latency: f64, avg_throughput: f64, current_load: f64) -> usize {
        let current_size = self.current_batch_size.load(Ordering::Relaxed);
        
        // Predictive model based on latency, throughput, and load
        let latency_factor = if avg_latency > self.config.target_latency_ms {
            0.8 // Reduce batch size if latency is high
        } else if avg_latency < self.config.target_latency_ms * 0.7 {
            1.2 // Increase batch size if latency is low
        } else {
            1.0 // Keep current size
        };
        
        let load_factor = if current_load > self.config.load_threshold_high {
            1.3 // Increase batch size under high load
        } else if current_load < self.config.load_threshold_low {
            0.9 // Decrease batch size under low load
        } else {
            1.0
        };
        
        let throughput_factor = if avg_throughput < 1000.0 {
            1.1 // Increase batch size for low throughput
        } else {
            1.0
        };
        
        let optimal_size = (current_size as f64 * latency_factor * load_factor * throughput_factor) as usize;
        
        optimal_size.clamp(self.config.min_batch_size, self.config.max_batch_size)
    }

    /// Calculate reactive batch size based on simple rules
    async fn calculate_reactive_batch_size(&self, avg_latency: f64, current_load: f64) -> usize {
        let current_size = self.current_batch_size.load(Ordering::Relaxed);
        
        if avg_latency > self.config.target_latency_ms {
            // Latency too high, reduce batch size
            (current_size.saturating_sub(self.config.batch_size_decrement))
                .max(self.config.min_batch_size)
        } else if current_load > self.config.load_threshold_high {
            // High load, increase batch size for better throughput
            (current_size + self.config.batch_size_increment)
                .min(self.config.max_batch_size)
        } else if current_load < self.config.load_threshold_low && avg_latency < self.config.target_latency_ms * 0.5 {
            // Low load and low latency, can increase batch size
            (current_size + self.config.batch_size_increment / 2)
                .min(self.config.max_batch_size)
        } else {
            current_size
        }
    }

    /// Calculate current system load
    async fn calculate_current_load(&self) -> f64 {
        let queue_depth = self.load_metrics.current_queue_depth.load(Ordering::Relaxed) as f64;
        let orders_per_second = self.load_metrics.orders_per_second.load(Ordering::Relaxed) as f64;
        let avg_processing_time = *self.load_metrics.average_processing_time_ms.read().await;
        
        // Normalize load factors
        let queue_load = (queue_depth / self.config.max_batch_size as f64).min(1.0);
        let throughput_load = (orders_per_second / 2000.0).min(1.0); // Normalize to 2000 TPS
        let latency_load = (avg_processing_time / self.config.target_latency_ms).min(1.0);
        
        // Weighted average of load factors
        (queue_load * 0.4 + throughput_load * 0.3 + latency_load * 0.3)
    }

    /// Check if batch processing should be forced
    async fn should_force_batch_processing(&self) -> bool {
        let queue_depth = self.load_metrics.current_queue_depth.load(Ordering::Relaxed);
        let current_load = self.calculate_current_load().await;
        
        // Force processing under high load or large queue
        queue_depth > self.config.max_batch_size || current_load > 0.9
    }

    /// Get current batch size
    pub fn get_current_batch_size(&self) -> usize {
        self.current_batch_size.load(Ordering::Relaxed)
    }

    /// Get performance statistics
    pub async fn get_performance_stats(&self) -> AdaptiveBatchingStats {
        let history = self.performance_history.read().await;
        let total_batches = self.total_batches_processed.load(Ordering::Relaxed);
        
        if history.is_empty() {
            return AdaptiveBatchingStats::default();
        }
        
        let avg_latency = history.iter().map(|m| m.processing_latency_ms).sum::<f64>() / history.len() as f64;
        let avg_throughput = history.iter().map(|m| m.throughput_tps).sum::<f64>() / history.len() as f64;
        let avg_batch_size = history.iter().map(|m| m.batch_size).sum::<usize>() / history.len();
        let current_load = self.calculate_current_load().await;
        
        AdaptiveBatchingStats {
            total_batches_processed: total_batches,
            current_batch_size: self.get_current_batch_size(),
            average_batch_size: avg_batch_size,
            average_latency_ms: avg_latency,
            average_throughput_tps: avg_throughput,
            current_load_factor: current_load,
            queue_depth: self.load_metrics.current_queue_depth.load(Ordering::Relaxed),
        }
    }
}

/// Performance statistics for adaptive batching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveBatchingStats {
    pub total_batches_processed: u64,
    pub current_batch_size: usize,
    pub average_batch_size: usize,
    pub average_latency_ms: f64,
    pub average_throughput_tps: f64,
    pub current_load_factor: f64,
    pub queue_depth: usize,
}

impl Default for AdaptiveBatchingStats {
    fn default() -> Self {
        Self {
            total_batches_processed: 0,
            current_batch_size: 50,
            average_batch_size: 50,
            average_latency_ms: 0.0,
            average_throughput_tps: 0.0,
            current_load_factor: 0.0,
            queue_depth: 0,
        }
    }
}
