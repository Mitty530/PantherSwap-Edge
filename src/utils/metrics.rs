use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct PerformanceMetrics {
    pub trading_cycles_total: AtomicU64,
    pub trading_cycles_successful: AtomicU64,
    pub signals_generated_total: AtomicU64,
    pub market_data_ticks_processed: AtomicU64,
    pub api_requests_total: AtomicU64,
    pub database_queries_total: AtomicU64,
    pub average_cycle_time_ms: Arc<RwLock<f64>>,
    pub error_counts: Arc<RwLock<HashMap<String, u64>>>,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            trading_cycles_total: AtomicU64::new(0),
            trading_cycles_successful: AtomicU64::new(0),
            signals_generated_total: AtomicU64::new(0),
            market_data_ticks_processed: AtomicU64::new(0),
            api_requests_total: AtomicU64::new(0),
            database_queries_total: AtomicU64::new(0),
            average_cycle_time_ms: Arc::new(RwLock::new(0.0)),
            error_counts: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn record_trading_cycle(&self, duration: Duration, successful: bool) {
        self.trading_cycles_total.fetch_add(1, Ordering::Relaxed);
        if successful {
            self.trading_cycles_successful.fetch_add(1, Ordering::Relaxed);
        }
        
        // Update average cycle time (simplified exponential moving average)
        tokio::spawn({
            let avg_time = Arc::clone(&self.average_cycle_time_ms);
            let duration_ms = duration.as_millis() as f64;
            
            async move {
                let mut avg = avg_time.write().await;
                *avg = if *avg == 0.0 {
                    duration_ms
                } else {
                    0.9 * *avg + 0.1 * duration_ms  // EMA with alpha = 0.1
                };
            }
        });
    }
    
    pub fn record_signals_generated(&self, count: u64) {
        self.signals_generated_total.fetch_add(count, Ordering::Relaxed);
    }
    
    pub fn record_ticks_processed(&self, count: u64) {
        self.market_data_ticks_processed.fetch_add(count, Ordering::Relaxed);
    }
    
    pub fn record_api_request(&self) {
        self.api_requests_total.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn record_database_query(&self) {
        self.database_queries_total.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn record_error(&self, error_type: String) {
        tokio::spawn({
            let error_counts = Arc::clone(&self.error_counts);
            
            async move {
                let mut counts = error_counts.write().await;
                *counts.entry(error_type).or_insert(0) += 1;
            }
        });
    }
    
    pub async fn get_summary(&self) -> MetricsSummary {
        let avg_cycle_time = *self.average_cycle_time_ms.read().await;
        let error_counts = self.error_counts.read().await.clone();
        
        MetricsSummary {
            trading_cycles_total: self.trading_cycles_total.load(Ordering::Relaxed),
            trading_cycles_successful: self.trading_cycles_successful.load(Ordering::Relaxed),
            signals_generated_total: self.signals_generated_total.load(Ordering::Relaxed),
            ticks_processed_total: self.market_data_ticks_processed.load(Ordering::Relaxed),
            api_requests_total: self.api_requests_total.load(Ordering::Relaxed),
            database_queries_total: self.database_queries_total.load(Ordering::Relaxed),
            average_cycle_time_ms: avg_cycle_time,
            error_counts,
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct MetricsSummary {
    pub trading_cycles_total: u64,
    pub trading_cycles_successful: u64,
    pub signals_generated_total: u64,
    pub ticks_processed_total: u64,
    pub api_requests_total: u64,
    pub database_queries_total: u64,
    pub average_cycle_time_ms: f64,
    pub error_counts: HashMap<String, u64>,
}

impl MetricsSummary {
    pub fn success_rate(&self) -> f64 {
        if self.trading_cycles_total > 0 {
            self.trading_cycles_successful as f64 / self.trading_cycles_total as f64
        } else {
            0.0
        }
    }
}
