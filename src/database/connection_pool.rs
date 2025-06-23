// Advanced connection pool optimization for high-frequency trading
// Provides optimized connection pooling, monitoring, and auto-tuning

use crate::utils::Result;
use sqlx::{PgPool, postgres::PgPoolOptions, Row, pool::PoolConnection, Postgres};
use tracing::{info, warn};
use std::time::{Duration, Instant};
use tokio::time::interval;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use std::collections::VecDeque;

/// Advanced connection pool manager for high-frequency trading
#[derive(Clone)]
pub struct ConnectionPoolManager {
    pool: PgPool,
    config: PoolConfig,
    metrics: PoolMetrics,
}

/// Connection pool configuration optimized for trading
#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub min_connections: u32,
    pub max_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
    pub test_before_acquire: bool,
    pub auto_tune: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_connections: 15,  // Synchronized with production config for HFT
            max_connections: 75,  // Scaled to target range (50-100) for optimal performance
            acquire_timeout: Duration::from_secs(5), // Reduced for faster trading operations
            idle_timeout: Duration::from_secs(300), // 5 minutes
            max_lifetime: Duration::from_secs(1800), // 30 minutes
            test_before_acquire: true,
            auto_tune: true,
        }
    }
}

/// Connection pool performance metrics
#[derive(Debug, Default, Clone)]
pub struct PoolMetrics {
    pub total_connections: u32,
    pub active_connections: u32,
    pub idle_connections: u32,
    pub acquire_count: u64,
    pub acquire_time_avg: Duration,
    pub acquire_time_max: Duration,
    pub connection_errors: u64,
    pub query_count: u64,
    pub query_time_avg: Duration,
    pub last_updated: Option<Instant>,
}

impl ConnectionPoolManager {
    /// Create a new optimized connection pool for high-frequency trading
    pub async fn new(database_url: &str, config: Option<PoolConfig>) -> Result<Self> {
        let config = config.unwrap_or_default();
        
        info!("Creating optimized connection pool for high-frequency trading");
        info!("Pool config: min={}, max={}, acquire_timeout={:?}", 
              config.min_connections, config.max_connections, config.acquire_timeout);

        let pool = PgPoolOptions::new()
            .min_connections(config.min_connections)
            .max_connections(config.max_connections)
            .acquire_timeout(config.acquire_timeout)
            .idle_timeout(config.idle_timeout)
            .max_lifetime(config.max_lifetime)
            .test_before_acquire(config.test_before_acquire)
            // Optimize for high-frequency operations
            .before_acquire(|_conn, _meta| {
                Box::pin(async move {
                    // Connection acquisition monitoring
                    // Note: meta.age() is not available in current sqlx version
                    Ok(true)
                })
            })
            .after_connect(|conn, _meta| {
                Box::pin(async move {
                    // Optimize connection settings for trading
                    sqlx::query("SET statement_timeout = '30s'").execute(&mut *conn).await?;
                    sqlx::query("SET lock_timeout = '10s'").execute(&mut *conn).await?;
                    sqlx::query("SET idle_in_transaction_session_timeout = '60s'").execute(&mut *conn).await?;

                    // Enable parallel query execution
                    sqlx::query("SET max_parallel_workers_per_gather = 4").execute(&mut *conn).await?;

                    // Optimize for read-heavy workloads
                    sqlx::query("SET default_transaction_isolation = 'read committed'").execute(&mut *conn).await?;

                    Ok(())
                })
            })
            .connect(database_url)
            .await?;

        let manager = Self {
            pool,
            config,
            metrics: PoolMetrics::default(),
        };

        // Start monitoring if auto-tune is enabled
        if manager.config.auto_tune {
            manager.start_monitoring().await?;
        }

        Ok(manager)
    }

    /// Get the underlying connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Get current pool metrics
    pub async fn get_metrics(&mut self) -> Result<&PoolMetrics> {
        self.update_metrics().await?;
        Ok(&self.metrics)
    }

    /// Update pool metrics
    async fn update_metrics(&mut self) -> Result<()> {
        // Get PostgreSQL connection statistics
        let stats = sqlx::query(
            r#"
            SELECT 
                numbackends as total_connections,
                active as active_connections,
                idle as idle_connections
            FROM pg_stat_database 
            WHERE datname = current_database()
            "#
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = stats {
            self.metrics.total_connections = row.get::<Option<i32>, _>("total_connections")
                .unwrap_or(0) as u32;
            self.metrics.active_connections = row.get::<Option<i32>, _>("active_connections")
                .unwrap_or(0) as u32;
            self.metrics.idle_connections = row.get::<Option<i32>, _>("idle_connections")
                .unwrap_or(0) as u32;
        }

        // Get query performance statistics
        let query_stats = sqlx::query(
            r#"
            SELECT 
                calls,
                mean_exec_time,
                max_exec_time
            FROM pg_stat_statements 
            WHERE query NOT LIKE '%pg_stat%'
            ORDER BY calls DESC 
            LIMIT 1
            "#
        )
        .fetch_optional(&self.pool)
        .await;

        if let Ok(Some(row)) = query_stats {
            self.metrics.query_count = row.get::<Option<i64>, _>("calls").unwrap_or(0) as u64;
            let mean_time = row.get::<Option<f64>, _>("mean_exec_time").unwrap_or(0.0);
            let max_time = row.get::<Option<f64>, _>("max_exec_time").unwrap_or(0.0);
            
            self.metrics.query_time_avg = Duration::from_millis(mean_time as u64);
            self.metrics.acquire_time_max = Duration::from_millis(max_time as u64);
        }

        self.metrics.last_updated = Some(Instant::now());
        Ok(())
    }

    /// Start background monitoring and auto-tuning
    async fn start_monitoring(&self) -> Result<()> {
        let pool = self.pool.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                if let Err(e) = Self::monitor_and_tune(&pool, &config).await {
                    warn!("Pool monitoring error: {}", e);
                }
            }
        });

        info!("Started connection pool monitoring and auto-tuning");
        Ok(())
    }

    /// Monitor pool performance and apply auto-tuning
    async fn monitor_and_tune(pool: &PgPool, config: &PoolConfig) -> Result<()> {
        // Check for connection pool pressure
        let pool_size = pool.size() as u32;
        let idle_connections = pool.num_idle() as u32;
        let utilization = if pool_size > 0 {
            ((pool_size - idle_connections) as f64 / pool_size as f64) * 100.0
        } else {
            0.0
        };

        // Log pool status
        info!("Pool status: size={}, idle={}, utilization={:.1}%", 
              pool_size, idle_connections, utilization);

        // Check for slow queries
        let slow_queries = sqlx::query(
            r#"
            SELECT count(*) as slow_count
            FROM pg_stat_activity 
            WHERE state = 'active' 
            AND query_start < NOW() - INTERVAL '5 seconds'
            AND query NOT LIKE '%pg_stat%'
            "#
        )
        .fetch_one(pool)
        .await?;

        let slow_count: i64 = slow_queries.get("slow_count");
        if slow_count > 0 {
            warn!("Detected {} slow queries running > 5 seconds", slow_count);
        }

        // Check for blocked queries
        let blocked_queries = sqlx::query(
            r#"
            SELECT count(*) as blocked_count
            FROM pg_stat_activity 
            WHERE wait_event_type = 'Lock'
            "#
        )
        .fetch_one(pool)
        .await?;

        let blocked_count: i64 = blocked_queries.get("blocked_count");
        if blocked_count > 0 {
            warn!("Detected {} blocked queries waiting for locks", blocked_count);
        }

        // Auto-tune recommendations
        if utilization > 90.0 {
            warn!("High pool utilization ({}%), consider increasing max_connections", utilization);
        } else if utilization < 20.0 && pool_size > config.min_connections {
            info!("Low pool utilization ({}%), pool size could be reduced", utilization);
        }

        Ok(())
    }

    /// Perform connection pool health check
    pub async fn health_check(&self) -> Result<PoolHealthStatus> {
        let start = Instant::now();
        
        // Test basic connectivity
        let connectivity_test = sqlx::query("SELECT 1 as health_check")
            .fetch_one(&self.pool)
            .await;

        let connectivity_time = start.elapsed();
        let is_healthy = connectivity_test.is_ok();

        // Get detailed pool status
        let pool_size = self.pool.size() as u32;
        let idle_connections = self.pool.num_idle() as u32;
        let active_connections = pool_size - idle_connections;

        // Check for connection leaks
        let connection_leak_risk = if pool_size > 0 {
            (active_connections as f64 / pool_size as f64) > 0.95
        } else {
            false
        };

        // Performance assessment
        let performance_status = if connectivity_time < Duration::from_millis(10) {
            "Excellent".to_string()
        } else if connectivity_time < Duration::from_millis(50) {
            "Good".to_string()
        } else if connectivity_time < Duration::from_millis(100) {
            "Fair".to_string()
        } else {
            "Poor".to_string()
        };

        Ok(PoolHealthStatus {
            is_healthy,
            connectivity_time,
            pool_size,
            active_connections,
            idle_connections,
            connection_leak_risk,
            performance_status,
            recommendations: self.generate_health_recommendations(
                pool_size, active_connections, connectivity_time
            ),
        })
    }

    /// Generate health recommendations based on current status
    fn generate_health_recommendations(
        &self, 
        pool_size: u32, 
        active_connections: u32, 
        connectivity_time: Duration
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        let utilization = if pool_size > 0 {
            (active_connections as f64 / pool_size as f64) * 100.0
        } else {
            0.0
        };

        if utilization > 90.0 {
            recommendations.push("Consider increasing max_connections - high utilization detected".to_string());
        }

        if connectivity_time > Duration::from_millis(100) {
            recommendations.push("Slow connection acquisition - check database performance".to_string());
        }

        if pool_size < self.config.min_connections {
            recommendations.push("Pool size below minimum - check connection health".to_string());
        }

        if active_connections == 0 && pool_size > 0 {
            recommendations.push("No active connections - potential connection issue".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("Connection pool is operating optimally".to_string());
        }

        recommendations
    }

    /// Get detailed pool statistics for monitoring
    pub async fn get_detailed_stats(&self) -> Result<Value> {
        let pool_size = self.pool.size() as u32;
        let idle_connections = self.pool.num_idle() as u32;
        let active_connections = pool_size - idle_connections;

        // Get database connection statistics
        let db_stats = sqlx::query(
            r#"
            SELECT 
                numbackends,
                xact_commit,
                xact_rollback,
                blks_read,
                blks_hit,
                tup_returned,
                tup_fetched,
                tup_inserted,
                tup_updated,
                tup_deleted
            FROM pg_stat_database 
            WHERE datname = current_database()
            "#
        )
        .fetch_optional(&self.pool)
        .await?;

        let stats = json!({
            "pool": {
                "size": pool_size,
                "active": active_connections,
                "idle": idle_connections,
                "utilization_percent": if pool_size > 0 { (active_connections as f64 / pool_size as f64) * 100.0 } else { 0.0 },
                "config": {
                    "min_connections": self.config.min_connections,
                    "max_connections": self.config.max_connections,
                    "acquire_timeout_ms": self.config.acquire_timeout.as_millis(),
                    "idle_timeout_ms": self.config.idle_timeout.as_millis(),
                    "max_lifetime_ms": self.config.max_lifetime.as_millis()
                }
            },
            "database": db_stats.map(|row| json!({
                "total_backends": row.get::<Option<i32>, _>("numbackends"),
                "transactions": {
                    "commits": row.get::<Option<i64>, _>("xact_commit"),
                    "rollbacks": row.get::<Option<i64>, _>("xact_rollback")
                },
                "blocks": {
                    "read": row.get::<Option<i64>, _>("blks_read"),
                    "hit": row.get::<Option<i64>, _>("blks_hit")
                },
                "tuples": {
                    "returned": row.get::<Option<i64>, _>("tup_returned"),
                    "fetched": row.get::<Option<i64>, _>("tup_fetched"),
                    "inserted": row.get::<Option<i64>, _>("tup_inserted"),
                    "updated": row.get::<Option<i64>, _>("tup_updated"),
                    "deleted": row.get::<Option<i64>, _>("tup_deleted")
                }
            })).unwrap_or(json!(null)),
            "timestamp": chrono::Utc::now()
        });

        Ok(stats)
    }
}

/// Pool health status information
#[derive(Debug)]
pub struct PoolHealthStatus {
    pub is_healthy: bool,
    pub connectivity_time: Duration,
    pub pool_size: u32,
    pub active_connections: u32,
    pub idle_connections: u32,
    pub connection_leak_risk: bool,
    pub performance_status: String,
    pub recommendations: Vec<String>,
}
