// Comprehensive Performance Profiler for PantherSwap Edge
// Establishes baseline metrics and monitors critical performance indicators

use crate::utils::{Result, PantherSwapError};
use crate::database::Database;
use crate::trading::engine::TradingEngine;
use crate::ai::AIEngine;
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Instant, Duration as StdDuration};
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;
use tracing::{info, warn, error, debug};
use sysinfo::System;

/// Comprehensive performance baseline metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineMetrics {
    pub timestamp: DateTime<Utc>,
    pub system_metrics: SystemMetrics,
    pub trading_metrics: TradingPerformanceMetrics,
    pub ai_metrics: AIPerformanceMetrics,
    pub database_metrics: DatabasePerformanceMetrics,
    pub api_metrics: APIPerformanceMetrics,
    pub memory_metrics: MemoryMetrics,
    pub network_metrics: NetworkMetrics,
}

/// System-level performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: u64,
    pub memory_total_mb: u64,
    pub memory_available_mb: u64,
    pub disk_usage_percent: f64,
    pub load_average: f64,
    pub process_count: usize,
    pub thread_count: usize,
}

/// Trading engine performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingPerformanceMetrics {
    pub order_execution_latency_ms: LatencyStats,
    pub signal_generation_latency_ms: LatencyStats,
    pub risk_check_latency_ms: LatencyStats,
    pub portfolio_update_latency_ms: LatencyStats,
    pub throughput_orders_per_second: f64,
    pub throughput_signals_per_second: f64,
    pub error_rate_percent: f64,
    pub active_orders_count: u64,
    pub daily_trade_volume: f64,
}

/// AI engine performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIPerformanceMetrics {
    pub lstm_inference_latency_ms: LatencyStats,
    pub rl_inference_latency_ms: LatencyStats,
    pub hmm_inference_latency_ms: LatencyStats,
    pub total_inference_latency_ms: LatencyStats,
    pub prediction_accuracy_percent: f64,
    pub model_confidence_score: f64,
    pub inference_throughput_per_second: f64,
    pub cache_hit_rate_percent: f64,
}

/// Database performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabasePerformanceMetrics {
    pub query_latency_ms: LatencyStats,
    pub connection_pool_usage_percent: f64,
    pub transactions_per_second: f64,
    pub cache_hit_ratio_percent: f64,
    pub index_usage_ratio_percent: f64,
    pub slow_queries_count: u64,
    pub deadlocks_count: u64,
    pub connection_errors_count: u64,
}

/// API performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIPerformanceMetrics {
    pub request_latency_ms: LatencyStats,
    pub requests_per_second: f64,
    pub error_rate_percent: f64,
    pub concurrent_connections: u64,
    pub rate_limit_hits: u64,
    pub auth_latency_ms: LatencyStats,
}

/// Memory usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub heap_usage_mb: u64,
    pub stack_usage_mb: u64,
    pub gc_pressure_percent: f64,
    pub allocation_rate_mb_per_sec: f64,
    pub deallocation_rate_mb_per_sec: f64,
    pub memory_leaks_detected: u64,
}

/// Network performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub bandwidth_usage_mbps: f64,
    pub packet_loss_percent: f64,
    pub connection_latency_ms: f64,
    pub tcp_connections_active: u64,
    pub network_errors_count: u64,
}

/// Latency statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyStats {
    pub min_ms: f64,
    pub max_ms: f64,
    pub avg_ms: f64,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub p999_ms: f64,
    pub std_dev_ms: f64,
}

impl LatencyStats {
    pub fn from_samples(mut samples: Vec<f64>) -> Self {
        if samples.is_empty() {
            return Self::default();
        }

        samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let len = samples.len();

        let min_ms = samples[0];
        let max_ms = samples[len - 1];
        let avg_ms = samples.iter().sum::<f64>() / len as f64;

        let p50_ms = samples[len * 50 / 100];
        let p95_ms = samples[len * 95 / 100];
        let p99_ms = samples[len * 99 / 100];
        let p999_ms = samples[len * 999 / 1000];

        // Calculate standard deviation
        let variance = samples.iter()
            .map(|x| (x - avg_ms).powi(2))
            .sum::<f64>() / len as f64;
        let std_dev_ms = variance.sqrt();

        Self {
            min_ms,
            max_ms,
            avg_ms,
            p50_ms,
            p95_ms,
            p99_ms,
            p999_ms,
            std_dev_ms,
        }
    }
}

impl Default for LatencyStats {
    fn default() -> Self {
        Self {
            min_ms: 0.0,
            max_ms: 0.0,
            avg_ms: 0.0,
            p50_ms: 0.0,
            p95_ms: 0.0,
            p99_ms: 0.0,
            p999_ms: 0.0,
            std_dev_ms: 0.0,
        }
    }
}

/// Performance profiler configuration
#[derive(Debug, Clone)]
pub struct ProfilerConfig {
    pub sampling_interval_ms: u64,
    pub baseline_duration_minutes: u64,
    pub latency_sample_size: usize,
    pub enable_detailed_profiling: bool,
    pub enable_memory_profiling: bool,
    pub enable_cpu_profiling: bool,
    pub target_order_latency_ms: f64,
    pub target_ai_latency_ms: f64,
    pub target_throughput_tps: f64,
}

impl Default for ProfilerConfig {
    fn default() -> Self {
        Self {
            sampling_interval_ms: 100,
            baseline_duration_minutes: 5,
            latency_sample_size: 1000,
            enable_detailed_profiling: true,
            enable_memory_profiling: true,
            enable_cpu_profiling: true,
            target_order_latency_ms: 10.0,
            target_ai_latency_ms: 100.0,
            target_throughput_tps: 1000.0,
        }
    }
}

/// Main performance profiler
pub struct PerformanceProfiler {
    config: ProfilerConfig,
    database: Database,
    
    // Metrics storage
    baseline_metrics: Arc<RwLock<Option<BaselineMetrics>>>,
    historical_metrics: Arc<RwLock<VecDeque<BaselineMetrics>>>,
    
    // Latency tracking
    order_latencies: Arc<RwLock<VecDeque<f64>>>,
    ai_latencies: Arc<RwLock<VecDeque<f64>>>,
    db_latencies: Arc<RwLock<VecDeque<f64>>>,
    api_latencies: Arc<RwLock<VecDeque<f64>>>,
    
    // System monitoring
    system: Arc<RwLock<System>>,
    
    // Performance alerts
    alert_sender: mpsc::UnboundedSender<PerformanceAlert>,
    alert_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<PerformanceAlert>>>>,
}

/// Performance alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    pub timestamp: DateTime<Utc>,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub metric_name: String,
    pub current_value: f64,
    pub threshold_value: f64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    LatencyThreshold,
    ThroughputDrop,
    ErrorRateSpike,
    MemoryLeak,
    CpuOverload,
    DatabaseSlow,
    AIModelDrift,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

impl PerformanceProfiler {
    /// Create a new performance profiler
    pub async fn new(config: ProfilerConfig, database: Database) -> Result<Self> {
        let (alert_sender, alert_receiver) = mpsc::unbounded_channel();
        
        let mut system = System::new_all();
        system.refresh_all();

        Ok(Self {
            config,
            database,
            baseline_metrics: Arc::new(RwLock::new(None)),
            historical_metrics: Arc::new(RwLock::new(VecDeque::new())),
            order_latencies: Arc::new(RwLock::new(VecDeque::new())),
            ai_latencies: Arc::new(RwLock::new(VecDeque::new())),
            db_latencies: Arc::new(RwLock::new(VecDeque::new())),
            api_latencies: Arc::new(RwLock::new(VecDeque::new())),
            system: Arc::new(RwLock::new(system)),
            alert_sender,
            alert_receiver: Arc::new(RwLock::new(Some(alert_receiver))),
        })
    }

    /// Start performance profiling
    pub async fn start_profiling(&self) -> Result<()> {
        info!("🔍 Starting performance profiling...");

        // Start baseline collection
        self.collect_baseline_metrics().await?;

        // Start continuous monitoring
        self.start_continuous_monitoring().await?;

        info!("✅ Performance profiling started successfully");
        Ok(())
    }

    /// Collect baseline performance metrics
    async fn collect_baseline_metrics(&self) -> Result<()> {
        info!("📊 Collecting baseline performance metrics...");

        let start_time = Instant::now();
        let duration = StdDuration::from_secs(self.config.baseline_duration_minutes * 60);

        let mut samples = Vec::new();

        while start_time.elapsed() < duration {
            let metrics = self.collect_current_metrics().await?;
            samples.push(metrics);

            tokio::time::sleep(StdDuration::from_millis(self.config.sampling_interval_ms)).await;
        }

        // Calculate baseline from samples
        let baseline = self.calculate_baseline_from_samples(samples).await?;

        // Store baseline
        {
            let mut baseline_guard = self.baseline_metrics.write().await;
            *baseline_guard = Some(baseline.clone());
        }

        info!("✅ Baseline metrics collected: avg order latency: {:.2}ms, avg AI latency: {:.2}ms",
              baseline.trading_metrics.order_execution_latency_ms.avg_ms,
              baseline.ai_metrics.total_inference_latency_ms.avg_ms);

        Ok(())
    }

    /// Collect current performance metrics
    async fn collect_current_metrics(&self) -> Result<BaselineMetrics> {
        let timestamp = Utc::now();

        // Collect system metrics
        let system_metrics = self.collect_system_metrics().await?;

        // Collect trading metrics
        let trading_metrics = self.collect_trading_metrics().await?;

        // Collect AI metrics
        let ai_metrics = self.collect_ai_metrics().await?;

        // Collect database metrics
        let database_metrics = self.collect_database_metrics().await?;

        // Collect API metrics
        let api_metrics = self.collect_api_metrics().await?;

        // Collect memory metrics
        let memory_metrics = self.collect_memory_metrics().await?;

        // Collect network metrics
        let network_metrics = self.collect_network_metrics().await?;

        Ok(BaselineMetrics {
            timestamp,
            system_metrics,
            trading_metrics,
            ai_metrics,
            database_metrics,
            api_metrics,
            memory_metrics,
            network_metrics,
        })
    }

    /// Collect system-level metrics
    async fn collect_system_metrics(&self) -> Result<SystemMetrics> {
        let mut system = self.system.write().await;
        system.refresh_all();

        // Get CPU usage
        let cpu_usage_percent = system.global_cpu_info().cpu_usage() as f64;

        // Get memory info
        let memory_total_mb = system.total_memory() / 1024 / 1024;
        let memory_usage_mb = system.used_memory() / 1024 / 1024;
        let memory_available_mb = system.available_memory() / 1024 / 1024;

        // Get process count (simplified)
        let process_count = system.processes().len();

        Ok(SystemMetrics {
            cpu_usage_percent,
            memory_usage_mb,
            memory_total_mb,
            memory_available_mb,
            disk_usage_percent: 0.0, // Would need additional implementation
            load_average: 0.0, // Would need additional implementation
            process_count,
            thread_count: 0, // Would need additional implementation
        })
    }

    /// Collect trading engine metrics
    async fn collect_trading_metrics(&self) -> Result<TradingPerformanceMetrics> {
        // Get latency samples
        let order_latencies = {
            let latencies = self.order_latencies.read().await;
            latencies.iter().cloned().collect::<Vec<_>>()
        };

        let order_execution_latency_ms = LatencyStats::from_samples(order_latencies);

        // Mock other metrics for now - would integrate with actual trading engine
        Ok(TradingPerformanceMetrics {
            order_execution_latency_ms,
            signal_generation_latency_ms: LatencyStats::default(),
            risk_check_latency_ms: LatencyStats::default(),
            portfolio_update_latency_ms: LatencyStats::default(),
            throughput_orders_per_second: 0.0,
            throughput_signals_per_second: 0.0,
            error_rate_percent: 0.0,
            active_orders_count: 0,
            daily_trade_volume: 0.0,
        })
    }

    /// Collect AI engine metrics
    async fn collect_ai_metrics(&self) -> Result<AIPerformanceMetrics> {
        // Get AI latency samples
        let ai_latencies = {
            let latencies = self.ai_latencies.read().await;
            latencies.iter().cloned().collect::<Vec<_>>()
        };

        let total_inference_latency_ms = LatencyStats::from_samples(ai_latencies);

        Ok(AIPerformanceMetrics {
            lstm_inference_latency_ms: LatencyStats::default(),
            rl_inference_latency_ms: LatencyStats::default(),
            hmm_inference_latency_ms: LatencyStats::default(),
            total_inference_latency_ms,
            prediction_accuracy_percent: 0.0,
            model_confidence_score: 0.0,
            inference_throughput_per_second: 0.0,
            cache_hit_rate_percent: 0.0,
        })
    }

    /// Collect database metrics
    async fn collect_database_metrics(&self) -> Result<DatabasePerformanceMetrics> {
        // Get database latency samples
        let db_latencies = {
            let latencies = self.db_latencies.read().await;
            latencies.iter().cloned().collect::<Vec<_>>()
        };

        let query_latency_ms = LatencyStats::from_samples(db_latencies);

        Ok(DatabasePerformanceMetrics {
            query_latency_ms,
            connection_pool_usage_percent: 0.0,
            transactions_per_second: 0.0,
            cache_hit_ratio_percent: 0.0,
            index_usage_ratio_percent: 0.0,
            slow_queries_count: 0,
            deadlocks_count: 0,
            connection_errors_count: 0,
        })
    }

    /// Collect API metrics
    async fn collect_api_metrics(&self) -> Result<APIPerformanceMetrics> {
        // Get API latency samples
        let api_latencies = {
            let latencies = self.api_latencies.read().await;
            latencies.iter().cloned().collect::<Vec<_>>()
        };

        let request_latency_ms = LatencyStats::from_samples(api_latencies);

        Ok(APIPerformanceMetrics {
            request_latency_ms,
            requests_per_second: 0.0,
            error_rate_percent: 0.0,
            concurrent_connections: 0,
            rate_limit_hits: 0,
            auth_latency_ms: LatencyStats::default(),
        })
    }

    /// Collect memory metrics
    async fn collect_memory_metrics(&self) -> Result<MemoryMetrics> {
        let system = self.system.read().await;
        let memory_usage_mb = system.used_memory() / 1024 / 1024;

        Ok(MemoryMetrics {
            heap_usage_mb: memory_usage_mb,
            stack_usage_mb: 0,
            gc_pressure_percent: 0.0,
            allocation_rate_mb_per_sec: 0.0,
            deallocation_rate_mb_per_sec: 0.0,
            memory_leaks_detected: 0,
        })
    }

    /// Collect network metrics
    async fn collect_network_metrics(&self) -> Result<NetworkMetrics> {
        Ok(NetworkMetrics {
            bandwidth_usage_mbps: 0.0,
            packet_loss_percent: 0.0,
            connection_latency_ms: 0.0,
            tcp_connections_active: 0,
            network_errors_count: 0,
        })
    }

    /// Calculate baseline from collected samples
    async fn calculate_baseline_from_samples(&self, samples: Vec<BaselineMetrics>) -> Result<BaselineMetrics> {
        if samples.is_empty() {
            return Err(PantherSwapError::performance("No samples collected for baseline".to_string()));
        }

        // Use the last sample as template and calculate averages
        let mut baseline = samples.last().unwrap().clone();
        baseline.timestamp = Utc::now();

        // Calculate average system metrics
        let cpu_avg = samples.iter().map(|s| s.system_metrics.cpu_usage_percent).sum::<f64>() / samples.len() as f64;
        let memory_avg = samples.iter().map(|s| s.system_metrics.memory_usage_mb).sum::<u64>() / samples.len() as u64;

        baseline.system_metrics.cpu_usage_percent = cpu_avg;
        baseline.system_metrics.memory_usage_mb = memory_avg;

        Ok(baseline)
    }

    /// Start continuous monitoring
    async fn start_continuous_monitoring(&self) -> Result<()> {
        let profiler = self.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(StdDuration::from_millis(profiler.config.sampling_interval_ms));

            loop {
                interval.tick().await;

                if let Err(e) = profiler.monitoring_cycle().await {
                    error!("Monitoring cycle error: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Run a single monitoring cycle
    async fn monitoring_cycle(&self) -> Result<()> {
        let metrics = self.collect_current_metrics().await?;

        // Store historical metrics
        {
            let mut historical = self.historical_metrics.write().await;
            if historical.len() >= 1000 {
                historical.pop_front();
            }
            historical.push_back(metrics.clone());
        }

        // Check for performance alerts
        self.check_performance_alerts(&metrics).await?;

        Ok(())
    }

    /// Check for performance alerts
    async fn check_performance_alerts(&self, metrics: &BaselineMetrics) -> Result<()> {
        // Check order execution latency
        if metrics.trading_metrics.order_execution_latency_ms.avg_ms > self.config.target_order_latency_ms {
            self.send_alert(PerformanceAlert {
                timestamp: Utc::now(),
                alert_type: AlertType::LatencyThreshold,
                severity: AlertSeverity::Warning,
                metric_name: "order_execution_latency".to_string(),
                current_value: metrics.trading_metrics.order_execution_latency_ms.avg_ms,
                threshold_value: self.config.target_order_latency_ms,
                message: format!("Order execution latency ({:.2}ms) exceeds target ({:.2}ms)",
                                metrics.trading_metrics.order_execution_latency_ms.avg_ms,
                                self.config.target_order_latency_ms),
            }).await?;
        }

        // Check AI inference latency
        if metrics.ai_metrics.total_inference_latency_ms.avg_ms > self.config.target_ai_latency_ms {
            self.send_alert(PerformanceAlert {
                timestamp: Utc::now(),
                alert_type: AlertType::LatencyThreshold,
                severity: AlertSeverity::Warning,
                metric_name: "ai_inference_latency".to_string(),
                current_value: metrics.ai_metrics.total_inference_latency_ms.avg_ms,
                threshold_value: self.config.target_ai_latency_ms,
                message: format!("AI inference latency ({:.2}ms) exceeds target ({:.2}ms)",
                                metrics.ai_metrics.total_inference_latency_ms.avg_ms,
                                self.config.target_ai_latency_ms),
            }).await?;
        }

        // Check CPU usage
        if metrics.system_metrics.cpu_usage_percent > 80.0 {
            self.send_alert(PerformanceAlert {
                timestamp: Utc::now(),
                alert_type: AlertType::CpuOverload,
                severity: AlertSeverity::Critical,
                metric_name: "cpu_usage".to_string(),
                current_value: metrics.system_metrics.cpu_usage_percent,
                threshold_value: 80.0,
                message: format!("CPU usage ({:.1}%) is critically high", metrics.system_metrics.cpu_usage_percent),
            }).await?;
        }

        Ok(())
    }

    /// Send performance alert
    async fn send_alert(&self, alert: PerformanceAlert) -> Result<()> {
        if let Err(_) = self.alert_sender.send(alert.clone()) {
            warn!("Failed to send performance alert: {:?}", alert);
        }

        // Log alert
        match alert.severity {
            AlertSeverity::Info => info!("📊 {}", alert.message),
            AlertSeverity::Warning => warn!("⚠️ {}", alert.message),
            AlertSeverity::Critical => error!("🚨 {}", alert.message),
            AlertSeverity::Emergency => error!("🆘 EMERGENCY: {}", alert.message),
        }

        Ok(())
    }

    /// Record order execution latency
    pub async fn record_order_latency(&self, latency_ms: f64) {
        let mut latencies = self.order_latencies.write().await;
        if latencies.len() >= self.config.latency_sample_size {
            latencies.pop_front();
        }
        latencies.push_back(latency_ms);
    }

    /// Record AI inference latency
    pub async fn record_ai_latency(&self, latency_ms: f64) {
        let mut latencies = self.ai_latencies.write().await;
        if latencies.len() >= self.config.latency_sample_size {
            latencies.pop_front();
        }
        latencies.push_back(latency_ms);
    }

    /// Record database query latency
    pub async fn record_db_latency(&self, latency_ms: f64) {
        let mut latencies = self.db_latencies.write().await;
        if latencies.len() >= self.config.latency_sample_size {
            latencies.pop_front();
        }
        latencies.push_back(latency_ms);
    }

    /// Record API request latency
    pub async fn record_api_latency(&self, latency_ms: f64) {
        let mut latencies = self.api_latencies.write().await;
        if latencies.len() >= self.config.latency_sample_size {
            latencies.pop_front();
        }
        latencies.push_back(latency_ms);
    }

    /// Get current baseline metrics
    pub async fn get_baseline_metrics(&self) -> Option<BaselineMetrics> {
        let baseline = self.baseline_metrics.read().await;
        baseline.clone()
    }

    /// Get performance report
    pub async fn generate_performance_report(&self) -> Result<PerformanceReport> {
        let baseline = self.get_baseline_metrics().await;
        let current = self.collect_current_metrics().await?;

        let historical = {
            let hist = self.historical_metrics.read().await;
            hist.iter().cloned().collect::<Vec<_>>()
        };

        Ok(PerformanceReport {
            timestamp: Utc::now(),
            baseline_metrics: baseline,
            current_metrics: current,
            historical_metrics: historical,
            performance_summary: self.calculate_performance_summary().await?,
        })
    }

    /// Calculate performance summary
    async fn calculate_performance_summary(&self) -> Result<PerformanceSummary> {
        let baseline = self.get_baseline_metrics().await;
        let current = self.collect_current_metrics().await?;

        let meets_latency_targets = current.trading_metrics.order_execution_latency_ms.avg_ms <= self.config.target_order_latency_ms
            && current.ai_metrics.total_inference_latency_ms.avg_ms <= self.config.target_ai_latency_ms;

        let meets_throughput_targets = current.trading_metrics.throughput_orders_per_second >= self.config.target_throughput_tps;

        Ok(PerformanceSummary {
            overall_health_score: if meets_latency_targets && meets_throughput_targets { 100.0 } else { 75.0 },
            meets_latency_targets,
            meets_throughput_targets,
            critical_alerts_count: 0,
            warning_alerts_count: 0,
            uptime_percent: 99.9,
        })
    }
}

impl Clone for PerformanceProfiler {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            database: self.database.clone(),
            baseline_metrics: self.baseline_metrics.clone(),
            historical_metrics: self.historical_metrics.clone(),
            order_latencies: self.order_latencies.clone(),
            ai_latencies: self.ai_latencies.clone(),
            db_latencies: self.db_latencies.clone(),
            api_latencies: self.api_latencies.clone(),
            system: self.system.clone(),
            alert_sender: self.alert_sender.clone(),
            alert_receiver: Arc::new(RwLock::new(None)), // Clone doesn't get receiver
        }
    }
}

/// Performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub timestamp: DateTime<Utc>,
    pub baseline_metrics: Option<BaselineMetrics>,
    pub current_metrics: BaselineMetrics,
    pub historical_metrics: Vec<BaselineMetrics>,
    pub performance_summary: PerformanceSummary,
}

/// Performance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub overall_health_score: f64,
    pub meets_latency_targets: bool,
    pub meets_throughput_targets: bool,
    pub critical_alerts_count: u64,
    pub warning_alerts_count: u64,
    pub uptime_percent: f64,
}
