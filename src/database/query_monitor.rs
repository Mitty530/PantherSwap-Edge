// Real-time query performance monitoring for PantherSwap Edge
// Provides comprehensive query performance tracking, alerting, and optimization

use crate::utils::Result;
use sqlx::{PgPool, Row};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use uuid::Uuid;

/// Real-time query performance monitor
#[derive(Clone)]
pub struct QueryPerformanceMonitor {
    pool: PgPool,
    config: MonitorConfig,
    metrics: Arc<RwLock<QueryMetrics>>,
    slow_queries: Arc<RwLock<VecDeque<SlowQuery>>>,
    alerts: Arc<RwLock<Vec<QueryAlert>>>,
}

/// Configuration for query monitoring
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    pub slow_query_threshold_ms: u64,
    pub max_slow_queries_history: usize,
    pub max_alerts_history: usize,
    pub monitoring_interval_seconds: u64,
    pub enable_real_time_alerts: bool,
    pub enable_query_logging: bool,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            slow_query_threshold_ms: 1000, // 1 second
            max_slow_queries_history: 1000,
            max_alerts_history: 500,
            monitoring_interval_seconds: 30,
            enable_real_time_alerts: true,
            enable_query_logging: true,
        }
    }
}

/// Query performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMetrics {
    pub timestamp: DateTime<Utc>,
    pub total_queries: u64,
    pub slow_queries: u64,
    pub failed_queries: u64,
    pub average_execution_time_ms: f64,
    pub p95_execution_time_ms: f64,
    pub p99_execution_time_ms: f64,
    pub queries_per_second: f64,
    pub connection_pool_utilization: f64,
    pub active_connections: u32,
    pub idle_connections: u32,
}

/// Slow query information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlowQuery {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub query: String,
    pub execution_time_ms: u64,
    pub rows_affected: Option<u64>,
    pub connection_id: Option<String>,
    pub error_message: Option<String>,
}

/// Query performance alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryAlert {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub alert_type: QueryAlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub metric_value: f64,
    pub threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryAlertType {
    SlowQuery,
    HighQueryRate,
    ConnectionPoolExhaustion,
    HighFailureRate,
    LongRunningTransaction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

impl QueryPerformanceMonitor {
    /// Create a new query performance monitor
    pub fn new(pool: PgPool, config: Option<MonitorConfig>) -> Self {
        let config = config.unwrap_or_default();
        
        Self {
            pool,
            config,
            metrics: Arc::new(RwLock::new(QueryMetrics {
                timestamp: Utc::now(),
                total_queries: 0,
                slow_queries: 0,
                failed_queries: 0,
                average_execution_time_ms: 0.0,
                p95_execution_time_ms: 0.0,
                p99_execution_time_ms: 0.0,
                queries_per_second: 0.0,
                connection_pool_utilization: 0.0,
                active_connections: 0,
                idle_connections: 0,
            })),
            slow_queries: Arc::new(RwLock::new(VecDeque::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Start real-time monitoring
    pub async fn start_monitoring(&self) -> Result<()> {
        info!("Starting real-time query performance monitoring");
        
        let pool = self.pool.clone();
        let config = self.config.clone();
        let metrics = self.metrics.clone();
        let slow_queries = self.slow_queries.clone();
        let alerts = self.alerts.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                std::time::Duration::from_secs(config.monitoring_interval_seconds)
            );

            loop {
                interval.tick().await;
                
                match Self::collect_metrics(&pool, &config).await {
                    Ok(new_metrics) => {
                        // Update metrics
                        {
                            let mut metrics_guard = metrics.write().await;
                            *metrics_guard = new_metrics.clone();
                        }

                        // Check for alerts
                        if let Ok(new_alerts) = Self::check_alerts(&new_metrics, &config).await {
                            let mut alerts_guard = alerts.write().await;
                            alerts_guard.extend(new_alerts);

                            // Trim alerts history
                            let alerts_len = alerts_guard.len();
                            if alerts_len > config.max_alerts_history {
                                alerts_guard.drain(0..alerts_len - config.max_alerts_history);
                            }
                        }

                        debug!("Query metrics updated: QPS={:.2}, Avg={:.2}ms, Slow={}", 
                               new_metrics.queries_per_second,
                               new_metrics.average_execution_time_ms,
                               new_metrics.slow_queries);
                    }
                    Err(e) => {
                        error!("Failed to collect query metrics: {}", e);
                    }
                }
            }
        });

        info!("Real-time query performance monitoring started");
        Ok(())
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> QueryMetrics {
        self.metrics.read().await.clone()
    }

    /// Get slow queries history
    pub async fn get_slow_queries(&self, limit: Option<usize>) -> Vec<SlowQuery> {
        let slow_queries = self.slow_queries.read().await;
        let limit = limit.unwrap_or(100);
        
        slow_queries.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get recent alerts
    pub async fn get_alerts(&self, limit: Option<usize>) -> Vec<QueryAlert> {
        let alerts = self.alerts.read().await;
        let limit = limit.unwrap_or(50);
        
        alerts.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Record a slow query
    pub async fn record_slow_query(&self, query: String, execution_time_ms: u64) {
        let slow_query = SlowQuery {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            query,
            execution_time_ms,
            rows_affected: None,
            connection_id: None,
            error_message: None,
        };

        let mut slow_queries = self.slow_queries.write().await;
        slow_queries.push_back(slow_query);

        // Trim history
        if slow_queries.len() > self.config.max_slow_queries_history {
            slow_queries.pop_front();
        }

        if self.config.enable_query_logging {
            warn!("Slow query detected: {}ms - {}", execution_time_ms, 
                  slow_queries.back().unwrap().query.chars().take(100).collect::<String>());
        }
    }

    /// Collect current performance metrics
    async fn collect_metrics(pool: &PgPool, config: &MonitorConfig) -> Result<QueryMetrics> {
        let timestamp = Utc::now();
        
        // Get connection pool stats
        let pool_stats = Self::get_pool_stats(pool).await?;
        
        // Get query statistics from pg_stat_statements if available
        let query_stats = Self::get_query_stats(pool).await.unwrap_or_default();
        
        // Get active connections
        let connection_stats = Self::get_connection_stats(pool).await?;

        Ok(QueryMetrics {
            timestamp,
            total_queries: query_stats.total_calls,
            slow_queries: query_stats.slow_queries,
            failed_queries: query_stats.failed_queries,
            average_execution_time_ms: query_stats.avg_time_ms,
            p95_execution_time_ms: query_stats.p95_time_ms,
            p99_execution_time_ms: query_stats.p99_time_ms,
            queries_per_second: query_stats.qps,
            connection_pool_utilization: pool_stats.utilization,
            active_connections: connection_stats.active,
            idle_connections: connection_stats.idle,
        })
    }

    /// Check for performance alerts
    async fn check_alerts(metrics: &QueryMetrics, config: &MonitorConfig) -> Result<Vec<QueryAlert>> {
        let mut alerts = Vec::new();
        
        // Check for slow queries
        if metrics.average_execution_time_ms > config.slow_query_threshold_ms as f64 {
            alerts.push(QueryAlert {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                alert_type: QueryAlertType::SlowQuery,
                severity: AlertSeverity::Warning,
                message: format!("Average query time ({:.2}ms) exceeds threshold ({}ms)", 
                               metrics.average_execution_time_ms, config.slow_query_threshold_ms),
                metric_value: metrics.average_execution_time_ms,
                threshold: config.slow_query_threshold_ms as f64,
            });
        }

        // Check connection pool utilization
        if metrics.connection_pool_utilization > 0.8 {
            alerts.push(QueryAlert {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                alert_type: QueryAlertType::ConnectionPoolExhaustion,
                severity: if metrics.connection_pool_utilization > 0.95 { 
                    AlertSeverity::Critical 
                } else { 
                    AlertSeverity::Warning 
                },
                message: format!("High connection pool utilization: {:.1}%", 
                               metrics.connection_pool_utilization * 100.0),
                metric_value: metrics.connection_pool_utilization,
                threshold: 0.8,
            });
        }

        Ok(alerts)
    }

    /// Get connection pool statistics
    async fn get_pool_stats(pool: &PgPool) -> Result<PoolStats> {
        // Note: sqlx doesn't expose detailed pool stats in current version
        // This is a placeholder for when the API becomes available
        Ok(PoolStats {
            utilization: 0.5, // Placeholder
        })
    }

    /// Get query statistics
    async fn get_query_stats(pool: &PgPool) -> Result<QueryStats> {
        // Try to get stats from pg_stat_statements
        let result = sqlx::query(r#"
            SELECT 
                COALESCE(SUM(calls), 0) as total_calls,
                COALESCE(AVG(mean_exec_time), 0) as avg_time_ms,
                COALESCE(COUNT(*) FILTER (WHERE mean_exec_time > 1000), 0) as slow_queries
            FROM pg_stat_statements 
            WHERE query NOT LIKE '%pg_stat_statements%'
        "#)
        .fetch_optional(pool)
        .await;

        match result {
            Ok(Some(row)) => {
                Ok(QueryStats {
                    total_calls: row.get::<i64, _>("total_calls") as u64,
                    avg_time_ms: row.get::<f64, _>("avg_time_ms"),
                    slow_queries: row.get::<i64, _>("slow_queries") as u64,
                    failed_queries: 0, // Would need additional tracking
                    p95_time_ms: 0.0,  // Would need percentile calculation
                    p99_time_ms: 0.0,  // Would need percentile calculation
                    qps: 0.0,          // Would need time-based calculation
                })
            }
            _ => {
                // Fallback when pg_stat_statements is not available
                Ok(QueryStats::default())
            }
        }
    }

    /// Get connection statistics
    async fn get_connection_stats(pool: &PgPool) -> Result<ConnectionStats> {
        let result = sqlx::query(r#"
            SELECT 
                COUNT(*) FILTER (WHERE state = 'active') as active,
                COUNT(*) FILTER (WHERE state = 'idle') as idle
            FROM pg_stat_activity 
            WHERE datname = current_database()
        "#)
        .fetch_one(pool)
        .await?;

        Ok(ConnectionStats {
            active: result.get::<i64, _>("active") as u32,
            idle: result.get::<i64, _>("idle") as u32,
        })
    }
}

#[derive(Debug, Default)]
struct PoolStats {
    utilization: f64,
}

#[derive(Debug, Default)]
struct QueryStats {
    total_calls: u64,
    avg_time_ms: f64,
    slow_queries: u64,
    failed_queries: u64,
    p95_time_ms: f64,
    p99_time_ms: f64,
    qps: f64,
}

#[derive(Debug)]
struct ConnectionStats {
    active: u32,
    idle: u32,
}
