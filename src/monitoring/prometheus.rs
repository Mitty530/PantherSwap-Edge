use prometheus::{
    Counter, Gauge, Histogram, IntCounter, IntGauge, Registry, Encoder, TextEncoder,
    HistogramOpts, Opts, CounterVec, GaugeVec, HistogramVec, IntCounterVec, IntGaugeVec,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tracing::{info, warn, error};

/// Prometheus metrics manager
pub struct PrometheusMetrics {
    registry: Registry,
    
    // HTTP metrics
    http_requests_total: CounterVec,
    http_request_duration: HistogramVec,
    http_response_size: HistogramVec,
    
    // Trading metrics
    orders_total: CounterVec,
    order_execution_duration: HistogramVec,
    active_positions: IntGaugeVec,
    portfolio_value: GaugeVec,
    pnl_total: GaugeVec,
    
    // AI metrics
    ai_inference_duration: HistogramVec,
    ai_predictions_total: CounterVec,
    ai_model_accuracy: GaugeVec,
    ai_cache_hits: CounterVec,
    
    // Database metrics
    db_connections_active: IntGauge,
    db_connections_idle: IntGauge,
    db_query_duration: HistogramVec,
    db_queries_total: CounterVec,
    
    // System metrics
    memory_usage: Gauge,
    cpu_usage: Gauge,
    disk_usage: GaugeVec,
    network_bytes: CounterVec,
    
    // Business metrics
    trading_signals_generated: CounterVec,
    risk_alerts_total: CounterVec,
    system_health_score: Gauge,
    uptime_seconds: Counter,
    
    // Performance metrics
    latency_p50: Gauge,
    latency_p95: Gauge,
    latency_p99: Gauge,
    throughput_tps: Gauge,
    error_rate: Gauge,
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub endpoint_path: String,
    pub collection_interval_seconds: u64,
    pub retention_days: u32,
    pub enable_detailed_metrics: bool,
    pub enable_custom_metrics: bool,
    pub labels: HashMap<String, String>,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint_path: "/metrics".to_string(),
            collection_interval_seconds: 15,
            retention_days: 30,
            enable_detailed_metrics: true,
            enable_custom_metrics: true,
            labels: HashMap::new(),
        }
    }
}

impl PrometheusMetrics {
    /// Create new Prometheus metrics manager
    pub fn new(config: &MetricsConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let registry = Registry::new();
        
        // HTTP metrics
        let http_requests_total = CounterVec::new(
            Opts::new("http_requests_total", "Total number of HTTP requests")
                .namespace("pantherswap_edge"),
            &["method", "endpoint", "status_code"]
        )?;
        
        let http_request_duration = HistogramVec::new(
            HistogramOpts::new("http_request_duration_seconds", "HTTP request duration in seconds")
                .namespace("pantherswap_edge")
                .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]),
            &["method", "endpoint"]
        )?;
        
        let http_response_size = HistogramVec::new(
            HistogramOpts::new("http_response_size_bytes", "HTTP response size in bytes")
                .namespace("pantherswap_edge")
                .buckets(vec![100.0, 1000.0, 10000.0, 100000.0, 1000000.0]),
            &["method", "endpoint"]
        )?;
        
        // Trading metrics
        let orders_total = CounterVec::new(
            Opts::new("orders_total", "Total number of orders")
                .namespace("pantherswap_edge"),
            &["instrument", "side", "status"]
        )?;
        
        let order_execution_duration = HistogramVec::new(
            HistogramOpts::new("order_execution_duration_seconds", "Order execution duration in seconds")
                .namespace("pantherswap_edge")
                .buckets(vec![0.001, 0.002, 0.005, 0.01, 0.02, 0.05, 0.1, 0.2, 0.5]),
            &["instrument", "execution_type"]
        )?;
        
        let active_positions = IntGaugeVec::new(
            Opts::new("active_positions", "Number of active positions")
                .namespace("pantherswap_edge"),
            &["instrument", "side"]
        )?;
        
        let portfolio_value = GaugeVec::new(
            Opts::new("portfolio_value_usd", "Portfolio value in USD")
                .namespace("pantherswap_edge"),
            &["account", "currency"]
        )?;
        
        let pnl_total = GaugeVec::new(
            Opts::new("pnl_total_usd", "Total profit and loss in USD")
                .namespace("pantherswap_edge"),
            &["account", "period"]
        )?;
        
        // AI metrics
        let ai_inference_duration = HistogramVec::new(
            HistogramOpts::new("ai_inference_duration_seconds", "AI inference duration in seconds")
                .namespace("pantherswap_edge")
                .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5]),
            &["model_type", "instrument"]
        )?;
        
        let ai_predictions_total = CounterVec::new(
            Opts::new("ai_predictions_total", "Total number of AI predictions")
                .namespace("pantherswap_edge"),
            &["model_type", "instrument", "prediction_type"]
        )?;
        
        let ai_model_accuracy = GaugeVec::new(
            Opts::new("ai_model_accuracy", "AI model accuracy score")
                .namespace("pantherswap_edge"),
            &["model_type", "instrument", "time_horizon"]
        )?;
        
        let ai_cache_hits = CounterVec::new(
            Opts::new("ai_cache_hits_total", "Total number of AI cache hits")
                .namespace("pantherswap_edge"),
            &["cache_type", "hit_miss"]
        )?;
        
        // Database metrics
        let db_connections_active = IntGauge::new(
            "db_connections_active", "Number of active database connections"
        )?;
        
        let db_connections_idle = IntGauge::new(
            "db_connections_idle", "Number of idle database connections"
        )?;
        
        let db_query_duration = HistogramVec::new(
            HistogramOpts::new("db_query_duration_seconds", "Database query duration in seconds")
                .namespace("pantherswap_edge")
                .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0]),
            &["query_type", "table"]
        )?;
        
        let db_queries_total = CounterVec::new(
            Opts::new("db_queries_total", "Total number of database queries")
                .namespace("pantherswap_edge"),
            &["query_type", "table", "status"]
        )?;
        
        // System metrics
        let memory_usage = Gauge::new(
            "memory_usage_bytes", "Memory usage in bytes"
        )?;
        
        let cpu_usage = Gauge::new(
            "cpu_usage_percent", "CPU usage percentage"
        )?;
        
        let disk_usage = GaugeVec::new(
            Opts::new("disk_usage_bytes", "Disk usage in bytes")
                .namespace("pantherswap_edge"),
            &["mount_point", "device"]
        )?;
        
        let network_bytes = CounterVec::new(
            Opts::new("network_bytes_total", "Total network bytes")
                .namespace("pantherswap_edge"),
            &["direction", "interface"]
        )?;
        
        // Business metrics
        let trading_signals_generated = CounterVec::new(
            Opts::new("trading_signals_generated_total", "Total trading signals generated")
                .namespace("pantherswap_edge"),
            &["signal_type", "instrument", "confidence_level"]
        )?;
        
        let risk_alerts_total = CounterVec::new(
            Opts::new("risk_alerts_total", "Total risk alerts")
                .namespace("pantherswap_edge"),
            &["alert_type", "severity"]
        )?;
        
        let system_health_score = Gauge::new(
            "system_health_score", "Overall system health score (0-100)"
        )?;
        
        let uptime_seconds = Counter::new(
            "uptime_seconds_total", "Total uptime in seconds"
        )?;
        
        // Performance metrics
        let latency_p50 = Gauge::new(
            "latency_p50_seconds", "50th percentile latency in seconds"
        )?;
        
        let latency_p95 = Gauge::new(
            "latency_p95_seconds", "95th percentile latency in seconds"
        )?;
        
        let latency_p99 = Gauge::new(
            "latency_p99_seconds", "99th percentile latency in seconds"
        )?;
        
        let throughput_tps = Gauge::new(
            "throughput_transactions_per_second", "Throughput in transactions per second"
        )?;
        
        let error_rate = Gauge::new(
            "error_rate_percent", "Error rate percentage"
        )?;
        
        // Register all metrics
        registry.register(Box::new(http_requests_total.clone()))?;
        registry.register(Box::new(http_request_duration.clone()))?;
        registry.register(Box::new(http_response_size.clone()))?;
        registry.register(Box::new(orders_total.clone()))?;
        registry.register(Box::new(order_execution_duration.clone()))?;
        registry.register(Box::new(active_positions.clone()))?;
        registry.register(Box::new(portfolio_value.clone()))?;
        registry.register(Box::new(pnl_total.clone()))?;
        registry.register(Box::new(ai_inference_duration.clone()))?;
        registry.register(Box::new(ai_predictions_total.clone()))?;
        registry.register(Box::new(ai_model_accuracy.clone()))?;
        registry.register(Box::new(ai_cache_hits.clone()))?;
        registry.register(Box::new(db_connections_active.clone()))?;
        registry.register(Box::new(db_connections_idle.clone()))?;
        registry.register(Box::new(db_query_duration.clone()))?;
        registry.register(Box::new(db_queries_total.clone()))?;
        registry.register(Box::new(memory_usage.clone()))?;
        registry.register(Box::new(cpu_usage.clone()))?;
        registry.register(Box::new(disk_usage.clone()))?;
        registry.register(Box::new(network_bytes.clone()))?;
        registry.register(Box::new(trading_signals_generated.clone()))?;
        registry.register(Box::new(risk_alerts_total.clone()))?;
        registry.register(Box::new(system_health_score.clone()))?;
        registry.register(Box::new(uptime_seconds.clone()))?;
        registry.register(Box::new(latency_p50.clone()))?;
        registry.register(Box::new(latency_p95.clone()))?;
        registry.register(Box::new(latency_p99.clone()))?;
        registry.register(Box::new(throughput_tps.clone()))?;
        registry.register(Box::new(error_rate.clone()))?;
        
        Ok(Self {
            registry,
            http_requests_total,
            http_request_duration,
            http_response_size,
            orders_total,
            order_execution_duration,
            active_positions,
            portfolio_value,
            pnl_total,
            ai_inference_duration,
            ai_predictions_total,
            ai_model_accuracy,
            ai_cache_hits,
            db_connections_active,
            db_connections_idle,
            db_query_duration,
            db_queries_total,
            memory_usage,
            cpu_usage,
            disk_usage,
            network_bytes,
            trading_signals_generated,
            risk_alerts_total,
            system_health_score,
            uptime_seconds,
            latency_p50,
            latency_p95,
            latency_p99,
            throughput_tps,
            error_rate,
        })
    }

    /// Record HTTP request metrics
    pub fn record_http_request(&self, method: &str, endpoint: &str, status_code: u16, duration: f64, response_size: f64) {
        self.http_requests_total
            .with_label_values(&[method, endpoint, &status_code.to_string()])
            .inc();
        
        self.http_request_duration
            .with_label_values(&[method, endpoint])
            .observe(duration);
        
        self.http_response_size
            .with_label_values(&[method, endpoint])
            .observe(response_size);
    }

    /// Record order execution metrics
    pub fn record_order_execution(&self, instrument: &str, side: &str, status: &str, duration: f64, execution_type: &str) {
        self.orders_total
            .with_label_values(&[instrument, side, status])
            .inc();
        
        self.order_execution_duration
            .with_label_values(&[instrument, execution_type])
            .observe(duration);
    }

    /// Record AI inference metrics
    pub fn record_ai_inference(&self, model_type: &str, instrument: &str, duration: f64, prediction_type: &str) {
        self.ai_inference_duration
            .with_label_values(&[model_type, instrument])
            .observe(duration);
        
        self.ai_predictions_total
            .with_label_values(&[model_type, instrument, prediction_type])
            .inc();
    }

    /// Update system health score
    pub fn update_system_health(&self, score: f64) {
        self.system_health_score.set(score);
    }

    /// Update performance metrics
    pub fn update_performance_metrics(&self, p50: f64, p95: f64, p99: f64, tps: f64, error_rate: f64) {
        self.latency_p50.set(p50);
        self.latency_p95.set(p95);
        self.latency_p99.set(p99);
        self.throughput_tps.set(tps);
        self.error_rate.set(error_rate);
    }

    /// Export metrics in Prometheus format
    pub fn export_metrics(&self) -> Result<String, Box<dyn std::error::Error>> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }

    /// Get registry for custom metrics
    pub fn registry(&self) -> &Registry {
        &self.registry
    }
}
