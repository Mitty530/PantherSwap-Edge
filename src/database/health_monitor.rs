// Comprehensive database health monitoring for PantherSwap Edge
// Provides real-time health checks, performance monitoring, and alerting

use crate::utils::Result;
use sqlx::{PgPool, Row};
use chrono::{DateTime, Utc};
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use tracing::{info, warn, error};
use tokio::time::interval;

/// Comprehensive database health monitor
pub struct DatabaseHealthMonitor {
    pool: PgPool,
    config: HealthMonitorConfig,
    metrics_history: Vec<HealthMetrics>,
    alert_thresholds: AlertThresholds,
}

#[derive(Debug, Clone)]
pub struct HealthMonitorConfig {
    pub check_interval_seconds: u64,
    pub metrics_retention_hours: i64,
    pub enable_continuous_monitoring: bool,
    pub enable_alerting: bool,
    pub max_history_size: usize,
}

impl Default for HealthMonitorConfig {
    fn default() -> Self {
        Self {
            check_interval_seconds: 30,
            metrics_retention_hours: 24,
            enable_continuous_monitoring: true,
            enable_alerting: true,
            max_history_size: 2880, // 24 hours at 30-second intervals
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    pub timestamp: DateTime<Utc>,
    pub connectivity: ConnectivityMetrics,
    pub performance: PerformanceMetrics,
    pub resource_usage: ResourceUsageMetrics,
    pub database_stats: DatabaseStatistics,
    pub timescale_metrics: Option<TimescaleMetrics>,
    pub overall_health_score: f64,
    pub alerts: Vec<HealthAlert>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectivityMetrics {
    pub is_connected: bool,
    pub connection_time_ms: u64,
    pub active_connections: u32,
    pub idle_connections: u32,
    pub max_connections: u32,
    pub connection_utilization_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub query_response_time_ms: u64,
    pub transactions_per_second: f64,
    pub cache_hit_ratio: f64,
    pub index_usage_ratio: f64,
    pub slow_queries_count: i64,
    pub blocked_queries_count: i64,
    pub deadlocks_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageMetrics {
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub disk_usage_percent: f64,
    pub disk_io_read_mb_per_sec: f64,
    pub disk_io_write_mb_per_sec: f64,
    pub network_io_mb_per_sec: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStatistics {
    pub total_size_mb: f64,
    pub table_count: i64,
    pub index_count: i64,
    pub total_transactions: i64,
    pub committed_transactions: i64,
    pub rolled_back_transactions: i64,
    pub blocks_read: i64,
    pub blocks_hit: i64,
    pub tuples_inserted: i64,
    pub tuples_updated: i64,
    pub tuples_deleted: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimescaleMetrics {
    pub hypertables_count: i64,
    pub chunks_count: i64,
    pub compression_ratio: f64,
    pub compressed_chunks: i64,
    pub total_chunks: i64,
    pub retention_policy_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthAlert {
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub metric_value: f64,
    pub threshold: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertType {
    Connectivity,
    Performance,
    ResourceUsage,
    DatabaseSize,
    SlowQueries,
    ConnectionPool,
    TimescaleDB,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

#[derive(Debug, Clone)]
pub struct AlertThresholds {
    pub connection_time_warning_ms: u64,
    pub connection_time_critical_ms: u64,
    pub connection_utilization_warning_percent: f64,
    pub connection_utilization_critical_percent: f64,
    pub query_response_warning_ms: u64,
    pub query_response_critical_ms: u64,
    pub cache_hit_ratio_warning_percent: f64,
    pub slow_queries_warning_count: i64,
    pub slow_queries_critical_count: i64,
    pub cpu_usage_warning_percent: f64,
    pub cpu_usage_critical_percent: f64,
    pub memory_usage_warning_percent: f64,
    pub memory_usage_critical_percent: f64,
    pub disk_usage_warning_percent: f64,
    pub disk_usage_critical_percent: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            connection_time_warning_ms: 100,
            connection_time_critical_ms: 500,
            connection_utilization_warning_percent: 80.0,
            connection_utilization_critical_percent: 95.0,
            query_response_warning_ms: 50,
            query_response_critical_ms: 200,
            cache_hit_ratio_warning_percent: 90.0,
            slow_queries_warning_count: 5,
            slow_queries_critical_count: 20,
            cpu_usage_warning_percent: 70.0,
            cpu_usage_critical_percent: 90.0,
            memory_usage_warning_percent: 80.0,
            memory_usage_critical_percent: 95.0,
            disk_usage_warning_percent: 85.0,
            disk_usage_critical_percent: 95.0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthReport {
    pub timestamp: DateTime<Utc>,
    pub overall_status: HealthStatus,
    pub overall_score: f64,
    pub current_metrics: HealthMetrics,
    pub trends: HealthTrends,
    pub recommendations: Vec<String>,
    pub critical_alerts: Vec<HealthAlert>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Emergency,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthTrends {
    pub performance_trend: TrendDirection,
    pub resource_usage_trend: TrendDirection,
    pub connection_trend: TrendDirection,
    pub error_rate_trend: TrendDirection,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Stable,
    Degrading,
    Unknown,
}

impl DatabaseHealthMonitor {
    /// Create a new database health monitor
    pub fn new(pool: PgPool, config: HealthMonitorConfig) -> Self {
        Self {
            pool,
            config,
            metrics_history: Vec::new(),
            alert_thresholds: AlertThresholds::default(),
        }
    }

    /// Create with default configuration
    pub fn with_defaults(pool: PgPool) -> Self {
        Self::new(pool, HealthMonitorConfig::default())
    }

    /// Start continuous health monitoring
    pub async fn start_monitoring(&self) -> Result<()> {
        if !self.config.enable_continuous_monitoring {
            return Ok(());
        }

        let pool = self.pool.clone();
        let config = self.config.clone();
        let thresholds = self.alert_thresholds.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(config.check_interval_seconds));
            
            loop {
                interval.tick().await;
                
                match Self::collect_health_metrics(&pool, &thresholds).await {
                    Ok(metrics) => {
                        // Log health status
                        info!("Health check completed - Score: {:.2}, Alerts: {}", 
                              metrics.overall_health_score, metrics.alerts.len());
                        
                        // Log critical alerts
                        for alert in &metrics.alerts {
                            match alert.severity {
                                AlertSeverity::Critical | AlertSeverity::Emergency => {
                                    error!("CRITICAL ALERT: {:?} - {}", alert.alert_type, alert.message);
                                }
                                AlertSeverity::Warning => {
                                    warn!("WARNING: {:?} - {}", alert.alert_type, alert.message);
                                }
                                AlertSeverity::Info => {
                                    info!("INFO: {:?} - {}", alert.alert_type, alert.message);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Health monitoring error: {}", e);
                    }
                }
            }
        });

        info!("Started continuous database health monitoring (interval: {}s)", 
              self.config.check_interval_seconds);
        Ok(())
    }

    /// Perform comprehensive health check
    pub async fn health_check(&mut self) -> Result<HealthReport> {
        let metrics = Self::collect_health_metrics(&self.pool, &self.alert_thresholds).await?;
        
        // Store metrics in history
        self.metrics_history.push(metrics.clone());
        
        // Trim history if needed
        if self.metrics_history.len() > self.config.max_history_size {
            self.metrics_history.remove(0);
        }
        
        // Calculate trends
        let trends = self.calculate_trends();
        
        // Determine overall status
        let overall_status = self.determine_overall_status(&metrics);
        
        // Generate recommendations
        let recommendations = self.generate_recommendations(&metrics, &trends);
        
        // Filter critical alerts
        let critical_alerts = metrics.alerts.iter()
            .filter(|alert| matches!(alert.severity, AlertSeverity::Critical | AlertSeverity::Emergency))
            .cloned()
            .collect();

        Ok(HealthReport {
            timestamp: Utc::now(),
            overall_status,
            overall_score: metrics.overall_health_score,
            current_metrics: metrics,
            trends,
            recommendations,
            critical_alerts,
        })
    }

    /// Collect comprehensive health metrics
    async fn collect_health_metrics(pool: &PgPool, thresholds: &AlertThresholds) -> Result<HealthMetrics> {
        let _start_time = Instant::now();
        let mut alerts = Vec::new();

        // Test connectivity
        let connectivity_start = Instant::now();
        let connectivity_test = sqlx::query("SELECT 1 as health_check")
            .fetch_one(pool)
            .await;
        let connection_time = connectivity_start.elapsed();
        let is_connected = connectivity_test.is_ok();

        // Connection pool metrics
        let pool_size = pool.size() as u32;
        let idle_connections = pool.num_idle() as u32;
        let active_connections = pool_size - idle_connections;
        let max_connections = pool.options().get_max_connections();
        let connection_utilization = if max_connections > 0 {
            (pool_size as f64 / max_connections as f64) * 100.0
        } else {
            0.0
        };

        // Check connection alerts
        if connection_time.as_millis() as u64 > thresholds.connection_time_critical_ms {
            alerts.push(HealthAlert {
                alert_type: AlertType::Connectivity,
                severity: AlertSeverity::Critical,
                message: format!("Connection time {}ms exceeds critical threshold", connection_time.as_millis()),
                metric_value: connection_time.as_millis() as f64,
                threshold: thresholds.connection_time_critical_ms as f64,
                timestamp: Utc::now(),
            });
        } else if connection_time.as_millis() as u64 > thresholds.connection_time_warning_ms {
            alerts.push(HealthAlert {
                alert_type: AlertType::Connectivity,
                severity: AlertSeverity::Warning,
                message: format!("Connection time {}ms exceeds warning threshold", connection_time.as_millis()),
                metric_value: connection_time.as_millis() as f64,
                threshold: thresholds.connection_time_warning_ms as f64,
                timestamp: Utc::now(),
            });
        }

        if connection_utilization > thresholds.connection_utilization_critical_percent {
            alerts.push(HealthAlert {
                alert_type: AlertType::ConnectionPool,
                severity: AlertSeverity::Critical,
                message: format!("Connection utilization {:.1}% is critically high", connection_utilization),
                metric_value: connection_utilization,
                threshold: thresholds.connection_utilization_critical_percent,
                timestamp: Utc::now(),
            });
        }

        let connectivity = ConnectivityMetrics {
            is_connected,
            connection_time_ms: connection_time.as_millis() as u64,
            active_connections,
            idle_connections,
            max_connections,
            connection_utilization_percent: connection_utilization,
        };

        // Performance metrics
        let performance = Self::collect_performance_metrics(pool, thresholds, &mut alerts).await?;
        
        // Resource usage metrics
        let resource_usage = Self::collect_resource_metrics(pool, thresholds, &mut alerts).await?;
        
        // Database statistics
        let database_stats = Self::collect_database_statistics(pool).await?;
        
        // TimescaleDB metrics (if available)
        let timescale_metrics = Self::collect_timescale_metrics(pool).await.ok();
        
        // Calculate overall health score
        let overall_health_score = Self::calculate_health_score(&connectivity, &performance, &resource_usage);

        Ok(HealthMetrics {
            timestamp: Utc::now(),
            connectivity,
            performance,
            resource_usage,
            database_stats,
            timescale_metrics,
            overall_health_score,
            alerts,
        })
    }

    /// Collect performance metrics
    async fn collect_performance_metrics(
        pool: &PgPool,
        thresholds: &AlertThresholds,
        alerts: &mut Vec<HealthAlert>
    ) -> Result<PerformanceMetrics> {
        // Query response time test
        let query_start = Instant::now();
        let _ = sqlx::query("SELECT COUNT(*) FROM pg_stat_activity")
            .fetch_one(pool)
            .await?;
        let query_response_time = query_start.elapsed();

        // Check query response time alerts
        if query_response_time.as_millis() as u64 > thresholds.query_response_critical_ms {
            alerts.push(HealthAlert {
                alert_type: AlertType::Performance,
                severity: AlertSeverity::Critical,
                message: format!("Query response time {}ms is critically slow", query_response_time.as_millis()),
                metric_value: query_response_time.as_millis() as f64,
                threshold: thresholds.query_response_critical_ms as f64,
                timestamp: Utc::now(),
            });
        }

        // Database statistics for performance
        let stats = sqlx::query(
            r#"
            SELECT
                xact_commit,
                xact_rollback,
                blks_read,
                blks_hit,
                tup_returned,
                tup_fetched
            FROM pg_stat_database
            WHERE datname = current_database()
            "#
        )
        .fetch_optional(pool)
        .await?;

        let (transactions_per_second, cache_hit_ratio) = if let Some(row) = stats {
            let commits: Option<i64> = row.get("xact_commit");
            let rollbacks: Option<i64> = row.get("xact_rollback");
            let blocks_read: Option<i64> = row.get("blks_read");
            let blocks_hit: Option<i64> = row.get("blks_hit");

            let total_transactions = commits.unwrap_or(0) + rollbacks.unwrap_or(0);
            let tps = total_transactions as f64 / 60.0; // Approximate TPS

            let total_blocks = blocks_read.unwrap_or(0) + blocks_hit.unwrap_or(0);
            let hit_ratio = if total_blocks > 0 {
                (blocks_hit.unwrap_or(0) as f64 / total_blocks as f64) * 100.0
            } else {
                100.0
            };

            (tps, hit_ratio)
        } else {
            (0.0, 100.0)
        };

        // Check cache hit ratio
        if cache_hit_ratio < thresholds.cache_hit_ratio_warning_percent {
            alerts.push(HealthAlert {
                alert_type: AlertType::Performance,
                severity: AlertSeverity::Warning,
                message: format!("Cache hit ratio {:.1}% is below optimal", cache_hit_ratio),
                metric_value: cache_hit_ratio,
                threshold: thresholds.cache_hit_ratio_warning_percent,
                timestamp: Utc::now(),
            });
        }

        // Slow queries
        let slow_queries = sqlx::query(
            r#"
            SELECT COUNT(*) as slow_count
            FROM pg_stat_activity
            WHERE state = 'active'
            AND query_start < NOW() - INTERVAL '5 seconds'
            AND query NOT LIKE '%pg_stat%'
            "#
        )
        .fetch_one(pool)
        .await?;

        let slow_count: i64 = slow_queries.get("slow_count");

        if slow_count > thresholds.slow_queries_critical_count {
            alerts.push(HealthAlert {
                alert_type: AlertType::Performance,
                severity: AlertSeverity::Critical,
                message: format!("Critical number of slow queries: {}", slow_count),
                metric_value: slow_count as f64,
                threshold: thresholds.slow_queries_critical_count as f64,
                timestamp: Utc::now(),
            });
        } else if slow_count > thresholds.slow_queries_warning_count {
            alerts.push(HealthAlert {
                alert_type: AlertType::Performance,
                severity: AlertSeverity::Warning,
                message: format!("High number of slow queries: {}", slow_count),
                metric_value: slow_count as f64,
                threshold: thresholds.slow_queries_warning_count as f64,
                timestamp: Utc::now(),
            });
        }

        // Blocked queries
        let blocked_queries = sqlx::query(
            r#"
            SELECT COUNT(*) as blocked_count
            FROM pg_stat_activity
            WHERE wait_event_type = 'Lock'
            "#
        )
        .fetch_one(pool)
        .await?;

        let blocked_count: i64 = blocked_queries.get("blocked_count");

        // Index usage ratio (simplified)
        let index_usage = sqlx::query(
            r#"
            SELECT
                COALESCE(
                    100.0 * SUM(idx_scan) / NULLIF(SUM(seq_scan + idx_scan), 0),
                    100.0
                ) as index_usage_ratio
            FROM pg_stat_user_tables
            "#
        )
        .fetch_optional(pool)
        .await?;

        let index_usage_ratio = if let Some(row) = index_usage {
            row.get::<Option<f64>, _>("index_usage_ratio").unwrap_or(100.0)
        } else {
            100.0
        };

        // Deadlocks (from pg_stat_database)
        let deadlock_stats = sqlx::query(
            r#"
            SELECT deadlocks
            FROM pg_stat_database
            WHERE datname = current_database()
            "#
        )
        .fetch_optional(pool)
        .await?;

        let deadlocks_count = if let Some(row) = deadlock_stats {
            row.get::<Option<i64>, _>("deadlocks").unwrap_or(0)
        } else {
            0
        };

        Ok(PerformanceMetrics {
            query_response_time_ms: query_response_time.as_millis() as u64,
            transactions_per_second,
            cache_hit_ratio,
            index_usage_ratio,
            slow_queries_count: slow_count,
            blocked_queries_count: blocked_count,
            deadlocks_count,
        })
    }

    /// Collect resource usage metrics
    async fn collect_resource_metrics(
        pool: &PgPool,
        thresholds: &AlertThresholds,
        alerts: &mut Vec<HealthAlert>
    ) -> Result<ResourceUsageMetrics> {
        // Database size and disk usage
        let disk_stats = sqlx::query(
            r#"
            SELECT
                pg_database_size(current_database()) as db_size_bytes,
                pg_size_pretty(pg_database_size(current_database())) as db_size_pretty
            "#
        )
        .fetch_one(pool)
        .await?;

        let db_size_bytes: i64 = disk_stats.get("db_size_bytes");
        let db_size_mb = db_size_bytes as f64 / (1024.0 * 1024.0);

        // System resource metrics (simplified - in production use system monitoring)
        let cpu_usage = Self::get_cpu_usage_estimate(pool).await.unwrap_or(0.0);
        let memory_usage = Self::get_memory_usage_estimate(pool).await.unwrap_or(0.0);

        // Disk usage estimation based on database size
        let disk_usage = if db_size_mb > 10000.0 { 75.0 } else { 50.0 }; // Simplified estimation

        // Check resource usage alerts
        if cpu_usage > thresholds.cpu_usage_critical_percent {
            alerts.push(HealthAlert {
                alert_type: AlertType::ResourceUsage,
                severity: AlertSeverity::Critical,
                message: format!("CPU usage {:.1}% is critically high", cpu_usage),
                metric_value: cpu_usage,
                threshold: thresholds.cpu_usage_critical_percent,
                timestamp: Utc::now(),
            });
        }

        if memory_usage > thresholds.memory_usage_critical_percent {
            alerts.push(HealthAlert {
                alert_type: AlertType::ResourceUsage,
                severity: AlertSeverity::Critical,
                message: format!("Memory usage {:.1}% is critically high", memory_usage),
                metric_value: memory_usage,
                threshold: thresholds.memory_usage_critical_percent,
                timestamp: Utc::now(),
            });
        }

        // I/O statistics
        let io_stats = sqlx::query(
            r#"
            SELECT
                blks_read,
                blks_hit
            FROM pg_stat_database
            WHERE datname = current_database()
            "#
        )
        .fetch_optional(pool)
        .await?;

        let (disk_io_read, disk_io_write) = if let Some(row) = io_stats {
            let blocks_read: Option<i64> = row.get("blks_read");
            let blocks_hit: Option<i64> = row.get("blks_hit");

            // Convert blocks to MB (8KB blocks)
            let read_mb = (blocks_read.unwrap_or(0) * 8) as f64 / 1024.0;
            let write_mb = (blocks_hit.unwrap_or(0) * 8) as f64 / 1024.0;

            (read_mb / 60.0, write_mb / 60.0) // Per second estimate
        } else {
            (0.0, 0.0)
        };

        Ok(ResourceUsageMetrics {
            cpu_usage_percent: cpu_usage,
            memory_usage_percent: memory_usage,
            disk_usage_percent: disk_usage,
            disk_io_read_mb_per_sec: disk_io_read,
            disk_io_write_mb_per_sec: disk_io_write,
            network_io_mb_per_sec: 0.0, // Would need system-level monitoring
        })
    }

    /// Collect database statistics
    async fn collect_database_statistics(pool: &PgPool) -> Result<DatabaseStatistics> {
        let stats = sqlx::query(
            r#"
            SELECT
                pg_database_size(current_database()) as total_size_bytes,
                (SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public') as table_count,
                (SELECT COUNT(*) FROM pg_indexes WHERE schemaname = 'public') as index_count,
                xact_commit + xact_rollback as total_transactions,
                xact_commit as committed_transactions,
                xact_rollback as rolled_back_transactions,
                blks_read,
                blks_hit,
                tup_inserted,
                tup_updated,
                tup_deleted
            FROM pg_stat_database
            WHERE datname = current_database()
            "#
        )
        .fetch_one(pool)
        .await?;

        Ok(DatabaseStatistics {
            total_size_mb: stats.get::<i64, _>("total_size_bytes") as f64 / (1024.0 * 1024.0),
            table_count: stats.get::<Option<i64>, _>("table_count").unwrap_or(0),
            index_count: stats.get::<Option<i64>, _>("index_count").unwrap_or(0),
            total_transactions: stats.get::<Option<i64>, _>("total_transactions").unwrap_or(0),
            committed_transactions: stats.get::<Option<i64>, _>("committed_transactions").unwrap_or(0),
            rolled_back_transactions: stats.get::<Option<i64>, _>("rolled_back_transactions").unwrap_or(0),
            blocks_read: stats.get::<Option<i64>, _>("blks_read").unwrap_or(0),
            blocks_hit: stats.get::<Option<i64>, _>("blks_hit").unwrap_or(0),
            tuples_inserted: stats.get::<Option<i64>, _>("tup_inserted").unwrap_or(0),
            tuples_updated: stats.get::<Option<i64>, _>("tup_updated").unwrap_or(0),
            tuples_deleted: stats.get::<Option<i64>, _>("tup_deleted").unwrap_or(0),
        })
    }

    /// Collect TimescaleDB-specific metrics
    async fn collect_timescale_metrics(pool: &PgPool) -> Result<TimescaleMetrics> {
        // Check if TimescaleDB is available
        let timescale_check = sqlx::query(
            "SELECT EXISTS(SELECT 1 FROM pg_extension WHERE extname = 'timescaledb') as has_timescale"
        )
        .fetch_one(pool)
        .await?;

        let has_timescale: bool = timescale_check.get("has_timescale");

        if !has_timescale {
            return Ok(TimescaleMetrics {
                hypertables_count: 0,
                chunks_count: 0,
                compression_ratio: 0.0,
                compressed_chunks: 0,
                total_chunks: 0,
                retention_policy_active: false,
            });
        }

        // Get hypertable information
        let hypertable_stats = sqlx::query(
            r#"
            SELECT
                COUNT(*) as hypertables_count
            FROM timescaledb_information.hypertables
            "#
        )
        .fetch_optional(pool)
        .await?;

        let hypertables_count = if let Some(row) = hypertable_stats {
            row.get::<i64, _>("hypertables_count")
        } else {
            0
        };

        // Get chunk information
        let chunk_stats = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total_chunks,
                COUNT(CASE WHEN compressed_chunk_id IS NOT NULL THEN 1 END) as compressed_chunks
            FROM timescaledb_information.chunks
            "#
        )
        .fetch_optional(pool)
        .await?;

        let (total_chunks, compressed_chunks) = if let Some(row) = chunk_stats {
            (
                row.get::<i64, _>("total_chunks"),
                row.get::<i64, _>("compressed_chunks")
            )
        } else {
            (0, 0)
        };

        let compression_ratio = if total_chunks > 0 {
            (compressed_chunks as f64 / total_chunks as f64) * 100.0
        } else {
            0.0
        };

        // Check for retention policies
        let retention_check = sqlx::query(
            r#"
            SELECT COUNT(*) as policy_count
            FROM timescaledb_information.jobs
            WHERE proc_name = 'policy_retention'
            "#
        )
        .fetch_optional(pool)
        .await?;

        let retention_policy_active = if let Some(row) = retention_check {
            row.get::<i64, _>("policy_count") > 0
        } else {
            false
        };

        Ok(TimescaleMetrics {
            hypertables_count,
            chunks_count: total_chunks,
            compression_ratio,
            compressed_chunks,
            total_chunks,
            retention_policy_active,
        })
    }

    /// Calculate overall health score
    fn calculate_health_score(
        connectivity: &ConnectivityMetrics,
        performance: &PerformanceMetrics,
        resource_usage: &ResourceUsageMetrics,
    ) -> f64 {
        let mut score: f64 = 100.0;

        // Connectivity score (25% weight)
        if !connectivity.is_connected {
            score -= 50.0;
        } else {
            if connectivity.connection_time_ms > 500 {
                score -= 15.0;
            } else if connectivity.connection_time_ms > 100 {
                score -= 5.0;
            }

            if connectivity.connection_utilization_percent > 95.0 {
                score -= 10.0;
            } else if connectivity.connection_utilization_percent > 80.0 {
                score -= 5.0;
            }
        }

        // Performance score (35% weight)
        if performance.query_response_time_ms > 200 {
            score -= 20.0;
        } else if performance.query_response_time_ms > 50 {
            score -= 10.0;
        }

        if performance.cache_hit_ratio < 80.0 {
            score -= 15.0;
        } else if performance.cache_hit_ratio < 90.0 {
            score -= 5.0;
        }

        if performance.slow_queries_count > 20 {
            score -= 15.0;
        } else if performance.slow_queries_count > 5 {
            score -= 5.0;
        }

        // Resource usage score (40% weight)
        if resource_usage.cpu_usage_percent > 90.0 {
            score -= 20.0;
        } else if resource_usage.cpu_usage_percent > 70.0 {
            score -= 10.0;
        }

        if resource_usage.memory_usage_percent > 95.0 {
            score -= 15.0;
        } else if resource_usage.memory_usage_percent > 80.0 {
            score -= 8.0;
        }

        if resource_usage.disk_usage_percent > 95.0 {
            score -= 10.0;
        } else if resource_usage.disk_usage_percent > 85.0 {
            score -= 5.0;
        }

        score.max(0.0).min(100.0)
    }

    /// Calculate health trends based on historical data
    fn calculate_trends(&self) -> HealthTrends {
        if self.metrics_history.len() < 2 {
            return HealthTrends {
                performance_trend: TrendDirection::Unknown,
                resource_usage_trend: TrendDirection::Unknown,
                connection_trend: TrendDirection::Unknown,
                error_rate_trend: TrendDirection::Unknown,
            };
        }

        let recent_count = (self.metrics_history.len() / 4).max(2);
        let recent_metrics = &self.metrics_history[self.metrics_history.len() - recent_count..];
        let older_metrics = &self.metrics_history[..recent_count];

        // Performance trend
        let recent_perf_avg = recent_metrics.iter()
            .map(|m| m.performance.query_response_time_ms as f64)
            .sum::<f64>() / recent_metrics.len() as f64;
        let older_perf_avg = older_metrics.iter()
            .map(|m| m.performance.query_response_time_ms as f64)
            .sum::<f64>() / older_metrics.len() as f64;

        let performance_trend = if recent_perf_avg < older_perf_avg * 0.9 {
            TrendDirection::Improving
        } else if recent_perf_avg > older_perf_avg * 1.1 {
            TrendDirection::Degrading
        } else {
            TrendDirection::Stable
        };

        // Resource usage trend
        let recent_resource_avg = recent_metrics.iter()
            .map(|m| (m.resource_usage.cpu_usage_percent + m.resource_usage.memory_usage_percent) / 2.0)
            .sum::<f64>() / recent_metrics.len() as f64;
        let older_resource_avg = older_metrics.iter()
            .map(|m| (m.resource_usage.cpu_usage_percent + m.resource_usage.memory_usage_percent) / 2.0)
            .sum::<f64>() / older_metrics.len() as f64;

        let resource_usage_trend = if recent_resource_avg < older_resource_avg * 0.9 {
            TrendDirection::Improving
        } else if recent_resource_avg > older_resource_avg * 1.1 {
            TrendDirection::Degrading
        } else {
            TrendDirection::Stable
        };

        // Connection trend
        let recent_conn_avg = recent_metrics.iter()
            .map(|m| m.connectivity.connection_utilization_percent)
            .sum::<f64>() / recent_metrics.len() as f64;
        let older_conn_avg = older_metrics.iter()
            .map(|m| m.connectivity.connection_utilization_percent)
            .sum::<f64>() / older_metrics.len() as f64;

        let connection_trend = if recent_conn_avg < older_conn_avg * 0.9 {
            TrendDirection::Improving
        } else if recent_conn_avg > older_conn_avg * 1.1 {
            TrendDirection::Degrading
        } else {
            TrendDirection::Stable
        };

        // Error rate trend (based on alerts)
        let recent_error_avg = recent_metrics.iter()
            .map(|m| m.alerts.len() as f64)
            .sum::<f64>() / recent_metrics.len() as f64;
        let older_error_avg = older_metrics.iter()
            .map(|m| m.alerts.len() as f64)
            .sum::<f64>() / older_metrics.len() as f64;

        let error_rate_trend = if recent_error_avg < older_error_avg * 0.9 {
            TrendDirection::Improving
        } else if recent_error_avg > older_error_avg * 1.1 {
            TrendDirection::Degrading
        } else {
            TrendDirection::Stable
        };

        HealthTrends {
            performance_trend,
            resource_usage_trend,
            connection_trend,
            error_rate_trend,
        }
    }

    /// Determine overall health status
    fn determine_overall_status(&self, metrics: &HealthMetrics) -> HealthStatus {
        let critical_alerts = metrics.alerts.iter()
            .filter(|alert| matches!(alert.severity, AlertSeverity::Critical | AlertSeverity::Emergency))
            .count();

        let warning_alerts = metrics.alerts.iter()
            .filter(|alert| matches!(alert.severity, AlertSeverity::Warning))
            .count();

        if critical_alerts > 0 || metrics.overall_health_score < 50.0 {
            if critical_alerts > 3 || metrics.overall_health_score < 25.0 {
                HealthStatus::Emergency
            } else {
                HealthStatus::Critical
            }
        } else if warning_alerts > 0 || metrics.overall_health_score < 80.0 {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        }
    }

    /// Generate health recommendations
    fn generate_recommendations(&self, metrics: &HealthMetrics, trends: &HealthTrends) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Connection recommendations
        if metrics.connectivity.connection_utilization_percent > 90.0 {
            recommendations.push("Consider increasing the maximum connection pool size".to_string());
        }

        if metrics.connectivity.connection_time_ms > 100 {
            recommendations.push("Database connection latency is high - check network and database performance".to_string());
        }

        // Performance recommendations
        if metrics.performance.cache_hit_ratio < 90.0 {
            recommendations.push("Cache hit ratio is low - consider increasing shared_buffers or optimizing queries".to_string());
        }

        if metrics.performance.slow_queries_count > 5 {
            recommendations.push("High number of slow queries detected - review query performance and add indexes".to_string());
        }

        if metrics.performance.index_usage_ratio < 80.0 {
            recommendations.push("Low index usage ratio - consider adding indexes for frequently queried columns".to_string());
        }

        // Resource recommendations
        if metrics.resource_usage.cpu_usage_percent > 80.0 {
            recommendations.push("High CPU usage - consider optimizing queries or scaling database resources".to_string());
        }

        if metrics.resource_usage.memory_usage_percent > 85.0 {
            recommendations.push("High memory usage - monitor for memory leaks and consider increasing available memory".to_string());
        }

        if metrics.resource_usage.disk_usage_percent > 85.0 {
            recommendations.push("High disk usage - consider implementing data retention policies or adding storage".to_string());
        }

        // TimescaleDB recommendations
        if let Some(ts_metrics) = &metrics.timescale_metrics {
            if ts_metrics.compression_ratio < 50.0 && ts_metrics.total_chunks > 100 {
                recommendations.push("Low compression ratio - consider enabling compression policies for older data".to_string());
            }

            if !ts_metrics.retention_policy_active && ts_metrics.total_chunks > 1000 {
                recommendations.push("No retention policy active - consider implementing data retention to manage storage".to_string());
            }
        }

        // Trend-based recommendations
        match trends.performance_trend {
            TrendDirection::Degrading => {
                recommendations.push("Performance is degrading over time - investigate recent changes and optimize queries".to_string());
            }
            _ => {}
        }

        match trends.resource_usage_trend {
            TrendDirection::Degrading => {
                recommendations.push("Resource usage is increasing over time - plan for capacity scaling".to_string());
            }
            _ => {}
        }

        if recommendations.is_empty() {
            recommendations.push("Database health is optimal - continue monitoring".to_string());
        }

        recommendations
    }

    /// Get CPU usage estimate (simplified)
    async fn get_cpu_usage_estimate(pool: &PgPool) -> Result<f64> {
        // In a real implementation, this would use system monitoring
        // For now, estimate based on active connections and query load
        let active_stats = sqlx::query(
            "SELECT COUNT(*) as active_count FROM pg_stat_activity WHERE state = 'active'"
        )
        .fetch_one(pool)
        .await?;

        let active_count: i64 = active_stats.get("active_count");

        // Simple estimation: more active connections = higher CPU usage
        let estimated_cpu = (active_count as f64 * 10.0).min(100.0);
        Ok(estimated_cpu)
    }

    /// Get memory usage estimate (simplified)
    async fn get_memory_usage_estimate(pool: &PgPool) -> Result<f64> {
        // In a real implementation, this would use system monitoring
        // For now, estimate based on database size and connections
        let size_stats = sqlx::query(
            "SELECT pg_database_size(current_database()) as db_size"
        )
        .fetch_one(pool)
        .await?;

        let db_size: i64 = size_stats.get("db_size");
        let size_gb = db_size as f64 / (1024.0 * 1024.0 * 1024.0);

        // Simple estimation: larger database = higher memory usage
        let estimated_memory = (size_gb * 20.0 + 30.0).min(100.0);
        Ok(estimated_memory)
    }

    /// Get historical metrics
    pub fn get_metrics_history(&self) -> &[HealthMetrics] {
        &self.metrics_history
    }

    /// Clear metrics history
    pub fn clear_history(&mut self) {
        self.metrics_history.clear();
    }

    /// Update alert thresholds
    pub fn update_thresholds(&mut self, thresholds: AlertThresholds) {
        self.alert_thresholds = thresholds;
    }

    /// Get current alert thresholds
    pub fn get_thresholds(&self) -> &AlertThresholds {
        &self.alert_thresholds
    }
}
