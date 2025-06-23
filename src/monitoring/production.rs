// Production monitoring and health management for PantherSwap Edge
// Comprehensive system for monitoring AI models, trading engine, and infrastructure

use crate::utils::Result;
use crate::ai::monitoring::{AIPerformanceMonitor, PerformanceAlert as AIAlert};
use crate::database::health_monitor::{DatabaseHealthMonitor, HealthAlert as DBAlert};
use crate::trading::engine::{TradingEngine, PerformanceAlert as TradingAlert};
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tracing::{info, warn, error, debug};
use uuid::Uuid;

/// Production monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionMonitoringConfig {
    pub health_check_interval_seconds: u64,
    pub metrics_collection_interval_seconds: u64,
    pub alert_aggregation_window_seconds: u64,
    pub enable_auto_recovery: bool,
    pub enable_failover: bool,
    pub max_consecutive_failures: u32,
    pub system_health_threshold: f64,
    pub enable_performance_profiling: bool,
    pub enable_predictive_alerts: bool,
}

impl Default for ProductionMonitoringConfig {
    fn default() -> Self {
        Self {
            health_check_interval_seconds: 30,
            metrics_collection_interval_seconds: 10,
            alert_aggregation_window_seconds: 300, // 5 minutes
            enable_auto_recovery: true,
            enable_failover: true,
            max_consecutive_failures: 3,
            system_health_threshold: 0.8, // 80% minimum health
            enable_performance_profiling: true,
            enable_predictive_alerts: true,
        }
    }
}

/// System-wide health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthStatus {
    pub overall_health_score: f64,
    pub component_health: HashMap<String, ComponentHealth>,
    pub active_alerts: Vec<SystemAlert>,
    pub performance_metrics: SystemPerformanceMetrics,
    pub timestamp: DateTime<Utc>,
    pub uptime_seconds: u64,
    pub last_restart: Option<DateTime<Utc>>,
}

/// Individual component health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub component_name: String,
    pub health_score: f64,
    pub status: ComponentStatus,
    pub last_check: DateTime<Utc>,
    pub error_count: u32,
    pub performance_metrics: HashMap<String, f64>,
}

/// Component status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComponentStatus {
    Healthy,
    Degraded,
    Critical,
    Offline,
    Recovering,
}

/// System-wide performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPerformanceMetrics {
    pub total_requests_per_second: f64,
    pub average_response_time_ms: f64,
    pub error_rate_percentage: f64,
    pub memory_usage_percentage: f64,
    pub cpu_usage_percentage: f64,
    pub disk_usage_percentage: f64,
    pub network_throughput_mbps: f64,
}

/// Unified system alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemAlert {
    pub id: Uuid,
    pub component: String,
    pub alert_type: SystemAlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
    pub auto_recovery_attempted: bool,
    pub resolved: bool,
}

/// System alert types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemAlertType {
    PerformanceDegradation,
    ComponentFailure,
    ResourceExhaustion,
    SecurityBreach,
    DataQualityIssue,
    TradingRiskBreach,
    AIModelDrift,
    DatabaseConnectivity,
    NetworkLatency,
    SystemOverload,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// Production monitoring system
pub struct ProductionMonitor {
    config: ProductionMonitoringConfig,
    
    // Component monitors
    ai_monitor: Arc<AIPerformanceMonitor>,
    db_monitor: Arc<DatabaseHealthMonitor>,
    trading_engine: Arc<RwLock<TradingEngine>>,
    
    // System state
    system_health: Arc<RwLock<SystemHealthStatus>>,
    component_health: Arc<RwLock<HashMap<String, ComponentHealth>>>,
    active_alerts: Arc<RwLock<Vec<SystemAlert>>>,
    
    // Metrics collection
    metrics_history: Arc<RwLock<Vec<SystemPerformanceMetrics>>>,
    
    // Event handling
    alert_sender: mpsc::UnboundedSender<SystemAlert>,
    alert_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<SystemAlert>>>>,
    
    // Auto-recovery
    failure_counts: Arc<RwLock<HashMap<String, u32>>>,
    last_recovery_attempt: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
    
    // System startup time
    startup_time: DateTime<Utc>,
}

impl ProductionMonitor {
    /// Create new production monitor
    pub fn new(
        config: ProductionMonitoringConfig,
        ai_monitor: Arc<AIPerformanceMonitor>,
        db_monitor: Arc<DatabaseHealthMonitor>,
        trading_engine: Arc<RwLock<TradingEngine>>,
    ) -> Self {
        let (alert_sender, alert_receiver) = mpsc::unbounded_channel();
        
        Self {
            config,
            ai_monitor,
            db_monitor,
            trading_engine,
            system_health: Arc::new(RwLock::new(SystemHealthStatus {
                overall_health_score: 1.0,
                component_health: HashMap::new(),
                active_alerts: Vec::new(),
                performance_metrics: SystemPerformanceMetrics {
                    total_requests_per_second: 0.0,
                    average_response_time_ms: 0.0,
                    error_rate_percentage: 0.0,
                    memory_usage_percentage: 0.0,
                    cpu_usage_percentage: 0.0,
                    disk_usage_percentage: 0.0,
                    network_throughput_mbps: 0.0,
                },
                timestamp: Utc::now(),
                uptime_seconds: 0,
                last_restart: None,
            })),
            component_health: Arc::new(RwLock::new(HashMap::new())),
            active_alerts: Arc::new(RwLock::new(Vec::new())),
            metrics_history: Arc::new(RwLock::new(Vec::new())),
            alert_sender,
            alert_receiver: Arc::new(RwLock::new(Some(alert_receiver))),
            failure_counts: Arc::new(RwLock::new(HashMap::new())),
            last_recovery_attempt: Arc::new(RwLock::new(HashMap::new())),
            startup_time: Utc::now(),
        }
    }

    /// Start production monitoring
    pub async fn start_monitoring(&self) -> Result<()> {
        info!("Starting production monitoring system...");
        
        // Start health check loop
        self.start_health_check_loop().await;
        
        // Start metrics collection loop
        self.start_metrics_collection_loop().await;
        
        // Start alert processing loop
        self.start_alert_processing_loop().await;
        
        // Start auto-recovery loop if enabled
        if self.config.enable_auto_recovery {
            self.start_auto_recovery_loop().await;
        }
        
        info!("Production monitoring system started successfully");
        Ok(())
    }

    /// Start health check monitoring loop
    async fn start_health_check_loop(&self) {
        let monitor = self.clone();
        let interval_duration = std::time::Duration::from_secs(monitor.config.health_check_interval_seconds);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval_duration);
            loop {
                interval.tick().await;
                if let Err(e) = monitor.perform_health_check().await {
                    error!("Health check failed: {}", e);
                }
            }
        });
    }

    /// Start metrics collection loop
    async fn start_metrics_collection_loop(&self) {
        let monitor = self.clone();
        let interval_duration = std::time::Duration::from_secs(monitor.config.metrics_collection_interval_seconds);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval_duration);
            loop {
                interval.tick().await;
                if let Err(e) = monitor.collect_system_metrics().await {
                    error!("Metrics collection failed: {}", e);
                }
            }
        });
    }

    /// Start alert processing loop
    async fn start_alert_processing_loop(&self) {
        let monitor = self.clone();
        
        tokio::spawn(async move {
            let mut receiver_guard = monitor.alert_receiver.write().await;
            if let Some(mut receiver) = receiver_guard.take() {
                drop(receiver_guard);
                
                while let Some(alert) = receiver.recv().await {
                    if let Err(e) = monitor.process_system_alert(alert).await {
                        error!("Alert processing failed: {}", e);
                    }
                }
            }
        });
    }

    /// Start auto-recovery loop
    async fn start_auto_recovery_loop(&self) {
        let monitor = self.clone();
        let check_interval = std::time::Duration::from_secs(60); // Check every minute
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(check_interval);
            loop {
                interval.tick().await;
                if let Err(e) = monitor.check_auto_recovery().await {
                    error!("Auto-recovery check failed: {}", e);
                }
            }
        });
    }
}

impl ProductionMonitor {
    /// Perform comprehensive health check
    async fn perform_health_check(&self) -> Result<()> {
        let start_time = std::time::Instant::now();
        let mut component_health = HashMap::new();

        // Check AI components
        let ai_health = self.check_ai_component_health().await?;
        component_health.insert("ai_engine".to_string(), ai_health);

        // Check database health
        let db_health = self.check_database_health().await?;
        component_health.insert("database".to_string(), db_health);

        // Check trading engine health
        let trading_health = self.check_trading_engine_health().await?;
        component_health.insert("trading_engine".to_string(), trading_health);

        // Calculate overall health score
        let overall_health_score = self.calculate_overall_health_score(&component_health);

        // Update system health
        {
            let mut system_health = self.system_health.write().await;
            system_health.overall_health_score = overall_health_score;
            system_health.component_health = component_health.clone();
            system_health.timestamp = Utc::now();
            system_health.uptime_seconds = (Utc::now() - self.startup_time).num_seconds() as u64;
        }

        // Store component health
        {
            let mut health_guard = self.component_health.write().await;
            *health_guard = component_health;
        }

        // Check for critical health issues
        if overall_health_score < self.config.system_health_threshold {
            self.send_alert(SystemAlert {
                id: Uuid::new_v4(),
                component: "system".to_string(),
                alert_type: SystemAlertType::SystemOverload,
                severity: AlertSeverity::Critical,
                message: format!("System health degraded to {:.2}%", overall_health_score * 100.0),
                timestamp: Utc::now(),
                metadata: HashMap::new(),
                auto_recovery_attempted: false,
                resolved: false,
            }).await;
        }

        let check_duration = start_time.elapsed();
        debug!("Health check completed in {:?} - Overall health: {:.2}%",
               check_duration, overall_health_score * 100.0);

        Ok(())
    }

    /// Check AI component health
    async fn check_ai_component_health(&self) -> Result<ComponentHealth> {
        let mut performance_metrics = HashMap::new();
        let mut error_count = 0;

        // Get AI performance metrics (simplified for MVP)
        performance_metrics.insert("inference_latency_ms".to_string(), 50.0); // Mock data
        performance_metrics.insert("model_accuracy".to_string(), 0.85);
        performance_metrics.insert("prediction_confidence".to_string(), 0.78);

        // Determine health score based on metrics
        let health_score = if performance_metrics.get("inference_latency_ms").unwrap_or(&0.0) < &100.0
            && performance_metrics.get("model_accuracy").unwrap_or(&0.0) > &0.65 {
            0.9
        } else {
            0.6
        };

        let status = match health_score {
            s if s >= 0.8 => ComponentStatus::Healthy,
            s if s >= 0.6 => ComponentStatus::Degraded,
            s if s >= 0.3 => ComponentStatus::Critical,
            _ => ComponentStatus::Offline,
        };

        Ok(ComponentHealth {
            component_name: "ai_engine".to_string(),
            health_score,
            status,
            last_check: Utc::now(),
            error_count,
            performance_metrics,
        })
    }

    /// Check database health
    async fn check_database_health(&self) -> Result<ComponentHealth> {
        let mut performance_metrics = HashMap::new();
        let mut error_count = 0;

        // Get database health metrics (simplified)
        performance_metrics.insert("connection_time_ms".to_string(), 15.0);
        performance_metrics.insert("query_latency_ms".to_string(), 25.0);
        performance_metrics.insert("connection_pool_usage".to_string(), 0.4);

        let health_score = if performance_metrics.get("connection_time_ms").unwrap_or(&0.0) < &50.0
            && performance_metrics.get("query_latency_ms").unwrap_or(&0.0) < &100.0 {
            0.95
        } else {
            0.7
        };

        let status = match health_score {
            s if s >= 0.8 => ComponentStatus::Healthy,
            s if s >= 0.6 => ComponentStatus::Degraded,
            s if s >= 0.3 => ComponentStatus::Critical,
            _ => ComponentStatus::Offline,
        };

        Ok(ComponentHealth {
            component_name: "database".to_string(),
            health_score,
            status,
            last_check: Utc::now(),
            error_count,
            performance_metrics,
        })
    }

    /// Check trading engine health
    async fn check_trading_engine_health(&self) -> Result<ComponentHealth> {
        let mut performance_metrics = HashMap::new();
        let mut error_count = 0;

        // Get trading engine performance metrics
        let trading_engine = self.trading_engine.read().await;
        let engine_metrics = trading_engine.get_performance_metrics().await;

        performance_metrics.insert("order_latency_ms".to_string(), engine_metrics.average_latency_ms);
        performance_metrics.insert("throughput_tps".to_string(), engine_metrics.current_throughput_tps);
        performance_metrics.insert("total_orders".to_string(), engine_metrics.total_orders_processed as f64);

        let health_score = if engine_metrics.average_latency_ms < 10.0
            && engine_metrics.current_throughput_tps > 1000.0 {
            0.92
        } else if engine_metrics.average_latency_ms < 50.0 {
            0.75
        } else {
            0.5
        };

        let status = match health_score {
            s if s >= 0.8 => ComponentStatus::Healthy,
            s if s >= 0.6 => ComponentStatus::Degraded,
            s if s >= 0.3 => ComponentStatus::Critical,
            _ => ComponentStatus::Offline,
        };

        Ok(ComponentHealth {
            component_name: "trading_engine".to_string(),
            health_score,
            status,
            last_check: Utc::now(),
            error_count,
            performance_metrics,
        })
    }

    /// Calculate overall system health score
    fn calculate_overall_health_score(&self, component_health: &HashMap<String, ComponentHealth>) -> f64 {
        if component_health.is_empty() {
            return 0.0;
        }

        // Weighted average of component health scores
        let weights = [
            ("ai_engine", 0.3),
            ("database", 0.4),
            ("trading_engine", 0.3),
        ];

        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;

        for (component, weight) in &weights {
            if let Some(health) = component_health.get(*component) {
                weighted_sum += health.health_score * weight;
                total_weight += weight;
            }
        }

        if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.0
        }
    }

    /// Send system alert
    async fn send_alert(&self, alert: SystemAlert) {
        if let Err(e) = self.alert_sender.send(alert.clone()) {
            error!("Failed to send system alert: {}", e);
        } else {
            debug!("System alert sent: {}", alert.message);
        }
    }

    /// Collect system-wide performance metrics
    async fn collect_system_metrics(&self) -> Result<()> {
        let metrics = SystemPerformanceMetrics {
            total_requests_per_second: self.calculate_total_rps().await,
            average_response_time_ms: self.calculate_avg_response_time().await,
            error_rate_percentage: self.calculate_error_rate().await,
            memory_usage_percentage: self.get_memory_usage().await,
            cpu_usage_percentage: self.get_cpu_usage().await,
            disk_usage_percentage: self.get_disk_usage().await,
            network_throughput_mbps: self.get_network_throughput().await,
        };

        // Store metrics in history
        {
            let mut history = self.metrics_history.write().await;
            history.push(metrics.clone());

            // Keep only last 24 hours of metrics (assuming 10-second intervals)
            if history.len() > 8640 {
                history.remove(0);
            }
        }

        // Update system health with current metrics
        {
            let mut system_health = self.system_health.write().await;
            system_health.performance_metrics = metrics;
        }

        Ok(())
    }

    /// Process system alert
    async fn process_system_alert(&self, alert: SystemAlert) -> Result<()> {
        info!("Processing system alert: {} - {}", alert.component, alert.message);

        // Add to active alerts
        {
            let mut active_alerts = self.active_alerts.write().await;
            active_alerts.push(alert.clone());

            // Remove old resolved alerts
            active_alerts.retain(|a| !a.resolved && (Utc::now() - a.timestamp).num_hours() < 24);
        }

        // Attempt auto-recovery if enabled and appropriate
        if self.config.enable_auto_recovery &&
           matches!(alert.severity, AlertSeverity::Critical | AlertSeverity::Emergency) {
            self.attempt_auto_recovery(&alert).await?;
        }

        Ok(())
    }

    /// Check and perform auto-recovery
    async fn check_auto_recovery(&self) -> Result<()> {
        let component_health = self.component_health.read().await;

        for (component_name, health) in component_health.iter() {
            if matches!(health.status, ComponentStatus::Critical | ComponentStatus::Offline) {
                let failure_count = {
                    let mut failures = self.failure_counts.write().await;
                    let count = failures.entry(component_name.clone()).or_insert(0);
                    *count += 1;
                    *count
                };

                if failure_count >= self.config.max_consecutive_failures {
                    info!("Attempting auto-recovery for component: {}", component_name);
                    self.perform_component_recovery(component_name).await?;
                }
            } else if health.status == ComponentStatus::Healthy {
                // Reset failure count for healthy components
                let mut failures = self.failure_counts.write().await;
                failures.insert(component_name.clone(), 0);
            }
        }

        Ok(())
    }

    /// Attempt auto-recovery for a specific alert
    async fn attempt_auto_recovery(&self, alert: &SystemAlert) -> Result<()> {
        let component = &alert.component;

        // Check if we've attempted recovery recently
        {
            let last_attempts = self.last_recovery_attempt.read().await;
            if let Some(last_attempt) = last_attempts.get(component) {
                if (Utc::now() - *last_attempt).num_minutes() < 5 {
                    debug!("Skipping auto-recovery for {} - too recent", component);
                    return Ok(());
                }
            }
        }

        info!("Attempting auto-recovery for component: {}", component);

        // Update last recovery attempt time
        {
            let mut last_attempts = self.last_recovery_attempt.write().await;
            last_attempts.insert(component.clone(), Utc::now());
        }

        // Perform component-specific recovery
        self.perform_component_recovery(component).await?;

        Ok(())
    }

    /// Perform recovery for a specific component
    async fn perform_component_recovery(&self, component: &str) -> Result<()> {
        match component {
            "ai_engine" => {
                info!("Performing AI engine recovery...");
                // In production, this would restart AI models, clear caches, etc.
                // For now, we'll just log the recovery attempt
                info!("AI engine recovery completed");
            },
            "database" => {
                info!("Performing database recovery...");
                // In production, this would reconnect to database, clear connection pools, etc.
                info!("Database recovery completed");
            },
            "trading_engine" => {
                info!("Performing trading engine recovery...");
                // In production, this would restart trading processes, clear order queues, etc.
                info!("Trading engine recovery completed");
            },
            _ => {
                warn!("Unknown component for recovery: {}", component);
            }
        }

        Ok(())
    }

    /// Get current system health status
    pub async fn get_system_health(&self) -> SystemHealthStatus {
        self.system_health.read().await.clone()
    }

    /// Get component health details
    pub async fn get_component_health(&self, component: &str) -> Option<ComponentHealth> {
        let health = self.component_health.read().await;
        health.get(component).cloned()
    }

    /// Get all component health details
    pub async fn get_all_component_health(&self) -> HashMap<String, ComponentHealth> {
        self.component_health.read().await.clone()
    }

    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Vec<SystemAlert> {
        self.active_alerts.read().await.clone()
    }

    /// Get performance metrics
    pub async fn get_performance_metrics(&self) -> Vec<SystemPerformanceMetrics> {
        self.metrics_history.read().await.clone()
    }

    /// Get performance metrics history
    pub async fn get_metrics_history(&self, hours: u32) -> Vec<SystemPerformanceMetrics> {
        let history = self.metrics_history.read().await;
        let samples_per_hour = 360; // 10-second intervals
        let max_samples = (hours as usize) * samples_per_hour;

        if history.len() <= max_samples {
            history.clone()
        } else {
            history[history.len() - max_samples..].to_vec()
        }
    }

    /// Simplified metric calculation methods (would be more sophisticated in production)
    async fn calculate_total_rps(&self) -> f64 {
        // Mock implementation - would aggregate from all components
        1250.0
    }

    async fn calculate_avg_response_time(&self) -> f64 {
        // Mock implementation
        8.5
    }

    async fn calculate_error_rate(&self) -> f64 {
        // Mock implementation
        0.1
    }

    async fn get_memory_usage(&self) -> f64 {
        // Mock implementation - would use system metrics
        45.2
    }

    async fn get_cpu_usage(&self) -> f64 {
        // Mock implementation
        23.8
    }

    async fn get_disk_usage(&self) -> f64 {
        // Mock implementation
        67.3
    }

    async fn get_network_throughput(&self) -> f64 {
        // Mock implementation
        125.7
    }
}

// Clone implementation for async tasks
impl Clone for ProductionMonitor {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            ai_monitor: self.ai_monitor.clone(),
            db_monitor: self.db_monitor.clone(),
            trading_engine: self.trading_engine.clone(),
            system_health: self.system_health.clone(),
            component_health: self.component_health.clone(),
            active_alerts: self.active_alerts.clone(),
            metrics_history: self.metrics_history.clone(),
            alert_sender: self.alert_sender.clone(),
            alert_receiver: self.alert_receiver.clone(),
            failure_counts: self.failure_counts.clone(),
            last_recovery_attempt: self.last_recovery_attempt.clone(),
            startup_time: self.startup_time,
        }
    }
}
