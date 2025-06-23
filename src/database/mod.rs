pub mod schema;
pub mod types;
// pub mod queries; // Temporarily disabled due to compile-time query issues
pub mod query_functions;
pub mod migrations;
pub mod optimization;
pub mod advanced_indexes;
pub mod connection_pool;
pub mod pool_factory;
pub mod health_monitor;
pub mod alerting;
pub mod performance_testing;
pub mod query_monitor;
pub mod materialized_views;
pub mod pgbouncer_integration;
pub mod connection_cache;
pub mod optimization_validator;

// Temporarily disabled validation modules due to type conversion issues
// pub mod validation;
// pub mod data_quality;
// pub mod integrity;
// pub mod validation_middleware;

use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;
use crate::utils::Result;

// Re-export health monitoring types
pub use health_monitor::{
    DatabaseHealthMonitor, HealthReport, HealthMetrics, HealthStatus,
    HealthMonitorConfig, AlertThresholds, HealthAlert, AlertType, AlertSeverity
};
pub use alerting::{AlertManager, AlertConfig, ProcessedAlert};

// Re-export performance testing types
pub use performance_testing::{
    PerformanceTestManager, PerformanceTestConfig, TestResult, PerformanceMetrics,
    LatencyDistribution, ThroughputMetrics, ErrorMetrics, ResourceUsage,
    TestScenario, PerformanceReport
};

/// Database connection pool configuration for different environments and use cases
#[derive(Debug, Clone)]
pub struct DatabasePoolConfig {
    pub min_connections: u32,
    pub max_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Option<Duration>,
    pub max_lifetime: Option<Duration>,
    pub test_before_acquire: bool,
}

impl Default for DatabasePoolConfig {
    fn default() -> Self {
        Self::production()
    }
}

impl DatabasePoolConfig {
    /// Production configuration optimized for high-frequency trading
    pub fn production() -> Self {
        Self {
            min_connections: 2,
            max_connections: 10,
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Some(Duration::from_secs(300)), // 5 minutes
            max_lifetime: Some(Duration::from_secs(1800)), // 30 minutes
            test_before_acquire: true,
        }
    }

    /// Development configuration with lower resource usage
    pub fn development() -> Self {
        Self {
            min_connections: 2,
            max_connections: 10,
            acquire_timeout: Duration::from_secs(10),
            idle_timeout: Some(Duration::from_secs(600)), // 10 minutes
            max_lifetime: Some(Duration::from_secs(3600)), // 1 hour
            test_before_acquire: true,
        }
    }

    /// Testing configuration with minimal connections
    pub fn testing() -> Self {
        Self {
            min_connections: 1,
            max_connections: 5,
            acquire_timeout: Duration::from_secs(30), // Increased for cloud DB
            idle_timeout: Some(Duration::from_secs(300)),
            max_lifetime: Some(Duration::from_secs(1800)),
            test_before_acquire: false, // Faster for tests
        }
    }

    /// Cloud testing configuration optimized for remote databases
    pub fn cloud_testing() -> Self {
        Self {
            min_connections: 2,
            max_connections: 8,
            acquire_timeout: Duration::from_secs(45), // Generous for cloud latency
            idle_timeout: Some(Duration::from_secs(600)), // 10 minutes
            max_lifetime: Some(Duration::from_secs(3600)), // 1 hour
            test_before_acquire: false, // Skip health checks for speed
        }
    }

    /// High-frequency trading configuration with maximum performance
    pub fn high_frequency_trading() -> Self {
        Self {
            min_connections: 20,
            max_connections: 100,
            acquire_timeout: Duration::from_secs(5),
            idle_timeout: Some(Duration::from_secs(300)), // 5 minutes
            max_lifetime: Some(Duration::from_secs(1800)), // 30 minutes
            test_before_acquire: true,
        }
    }

    /// Cloud environment configuration with conservative settings
    pub fn cloud() -> Self {
        Self {
            min_connections: 5,
            max_connections: 20,
            acquire_timeout: Duration::from_secs(10),
            idle_timeout: Some(Duration::from_secs(600)),
            max_lifetime: Some(Duration::from_secs(3600)),
            test_before_acquire: true,
        }
    }
}

/// Connection pool statistics
#[derive(Debug, Clone)]
pub struct ConnectionPoolStats {
    pub size: u32,
    pub idle: u32,
    pub active: u32,
    pub max_size: u32,
    pub min_size: u32,
}

/// Pool health status information
#[derive(Debug)]
pub struct PoolHealthStatus {
    pub is_healthy: bool,
    pub connectivity_time: Duration,
    pub pool_stats: ConnectionPoolStats,
    pub utilization_percent: f64,
    pub performance_status: String,
}

#[derive(Debug, Clone)]
pub struct Database {
    pub pool: PgPool,
}

impl Database {
    /// Create a new database connection with optimized pool settings
    pub async fn new(database_url: &str) -> Result<Self> {
        Self::new_with_config(database_url, DatabasePoolConfig::default()).await
    }

    /// Create a new database connection with custom pool configuration
    pub async fn new_with_config(database_url: &str, config: DatabasePoolConfig) -> Result<Self> {
        tracing::info!("Creating database connection pool with config: {:?}", config);

        let pool = PgPoolOptions::new()
            .min_connections(config.min_connections)
            .max_connections(config.max_connections)
            .acquire_timeout(config.acquire_timeout)
            .idle_timeout(config.idle_timeout)
            .max_lifetime(config.max_lifetime)
            .test_before_acquire(config.test_before_acquire)
            // Optimize connection settings for trading workloads
            .after_connect(|conn, _meta| {
                Box::pin(async move {
                    // Set connection-level optimizations for trading with reasonable timeouts
                    sqlx::query("SET statement_timeout = '30s'").execute(&mut *conn).await?;
                    sqlx::query("SET lock_timeout = '10s'").execute(&mut *conn).await?;
                    sqlx::query("SET idle_in_transaction_session_timeout = '60s'").execute(&mut *conn).await?;

                    // Enable parallel query execution
                    sqlx::query("SET max_parallel_workers_per_gather = 4").execute(&mut *conn).await?;

                    // Optimize for read-heavy workloads
                    sqlx::query("SET default_transaction_isolation = 'read committed'").execute(&mut *conn).await?;

                    // Enable TimescaleDB optimizations
                    sqlx::query("SET timescaledb.enable_optimizations = 'on'").execute(&mut *conn).await.ok();

                    Ok(())
                })
            })
            .connect(database_url)
            .await?;

        tracing::info!("Database connection pool created successfully with {} max connections", config.max_connections);
        Ok(Self { pool })
    }
    
    pub async fn run_migrations(&self) -> Result<()> {
        tracing::info!("Running database migrations...");

        // Run sqlx migrations from the migrations directory
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await?;

        tracing::info!("Database migrations completed successfully");
        Ok(())
    }

    /// Run migrations manually (fallback for development)
    pub async fn run_manual_migrations(&self) -> Result<()> {
        tracing::info!("Running manual database migrations (fallback)...");

        // Create extensions
        sqlx::query("CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\";")
            .execute(&self.pool)
            .await?;

        // Run schema setup (temporarily disabled)
        // queries::setup_database_schema(&self.pool).await?;

        tracing::info!("Manual database migrations completed successfully");
        Ok(())
    }
    
    pub async fn health_check(&self) -> Result<bool> {
        match sqlx::query("SELECT 1 as health_check")
            .fetch_one(&self.pool)
            .await
        {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Get migration manager for advanced migration operations
    pub fn migration_manager(&self) -> migrations::MigrationManager {
        migrations::MigrationManager::new(self.pool.clone())
    }

    /// Check migration status
    pub async fn migration_status(&self) -> Result<migrations::MigrationStatus> {
        self.migration_manager().status().await
    }

    /// Validate database schema
    pub async fn validate_schema(&self) -> Result<migrations::SchemaValidation> {
        self.migration_manager().validate_schema().await
    }

    /// Get query manager for database operations
    pub fn query_manager(&self) -> query_functions::SimpleQueryManager {
        query_functions::SimpleQueryManager::new(self.pool.clone())
    }

    /// Get simple query manager for basic operations (compiles without database)
    pub fn simple_query_manager(&self) -> query_functions::SimpleQueryManager {
        query_functions::SimpleQueryManager::new(self.pool.clone())
    }

    /// Get optimization manager for database performance tuning
    pub fn optimization_manager(&self) -> optimization::OptimizationManager {
        optimization::OptimizationManager::new(self.pool.clone())
    }

    /// Get advanced index manager for specialized indexing
    pub fn advanced_index_manager(&self) -> advanced_indexes::AdvancedIndexManager {
        advanced_indexes::AdvancedIndexManager::new(self.pool.clone())
    }

    /// Get materialized views manager for analytics optimization
    pub fn materialized_views_manager(&self) -> materialized_views::MaterializedViewsManager {
        materialized_views::MaterializedViewsManager::new(self.pool.clone())
    }

    /// Get PgBouncer integration manager for connection multiplexing
    pub async fn pgbouncer_manager(&self) -> Result<pgbouncer_integration::PgBouncerManager> {
        let config = pgbouncer_integration::PgBouncerConfig::default();
        pgbouncer_integration::PgBouncerManager::new(config).await
    }

    /// Get connection cache manager for burst operations
    pub async fn connection_cache_manager(&self) -> Result<connection_cache::ConnectionCacheManager> {
        let config = connection_cache::CacheConfig::default();
        connection_cache::ConnectionCacheManager::new(self.pool.clone(), config).await
    }

    /// Get optimization validator for performance validation
    pub fn optimization_validator(&self) -> optimization_validator::OptimizationValidator {
        let config = optimization_validator::ValidationConfig::default();
        optimization_validator::OptimizationValidator::new(self.pool.clone(), config)
    }

    /// Get optimization validator with custom configuration
    pub fn optimization_validator_with_config(&self, config: optimization_validator::ValidationConfig) -> optimization_validator::OptimizationValidator {
        optimization_validator::OptimizationValidator::new(self.pool.clone(), config)
    }

    /// Create database connection optimized for production use
    pub async fn new_production(database_url: &str) -> Result<Self> {
        Self::new_with_config(database_url, DatabasePoolConfig::production()).await
    }

    /// Create database connection optimized for development
    pub async fn new_development(database_url: &str) -> Result<Self> {
        Self::new_with_config(database_url, DatabasePoolConfig::development()).await
    }

    /// Create database connection optimized for testing
    pub async fn new_testing(database_url: &str) -> Result<Self> {
        Self::new_with_config(database_url, DatabasePoolConfig::testing()).await
    }

    /// Create database connection optimized for cloud testing with generous timeouts
    pub async fn new_cloud_testing(database_url: &str) -> Result<Self> {
        Self::new_with_config(database_url, DatabasePoolConfig::cloud_testing()).await
    }

    /// Create database connection optimized for high-frequency trading
    pub async fn new_high_frequency_trading(database_url: &str) -> Result<Self> {
        Self::new_with_config(database_url, DatabasePoolConfig::high_frequency_trading()).await
    }

    /// Create database connection optimized for cloud environments
    pub async fn new_cloud(database_url: &str) -> Result<Self> {
        Self::new_with_config(database_url, DatabasePoolConfig::cloud()).await
    }

    /// Create optimized connection pool manager for advanced features
    pub async fn create_optimized_pool(database_url: &str) -> Result<connection_pool::ConnectionPoolManager> {
        connection_pool::ConnectionPoolManager::new(database_url, None).await
    }

    // Validation methods temporarily disabled due to module compilation issues
    // These will be re-enabled once validation module type issues are resolved

    /// Get connection pool statistics
    pub fn pool_stats(&self) -> ConnectionPoolStats {
        let size = self.pool.size() as u32;
        let idle = self.pool.num_idle() as u32;
        let active = if size >= idle { size - idle } else { 0 };

        ConnectionPoolStats {
            size,
            idle,
            active,
            max_size: self.pool.options().get_max_connections(),
            min_size: self.pool.options().get_min_connections(),
        }
    }

    /// Check if the connection pool is healthy
    pub async fn pool_health_check(&self) -> Result<PoolHealthStatus> {
        let start = std::time::Instant::now();

        // Test basic connectivity
        let connectivity_test = sqlx::query("SELECT 1 as health_check")
            .fetch_one(&self.pool)
            .await;

        let connectivity_time = start.elapsed();
        let is_healthy = connectivity_test.is_ok();

        let stats = self.pool_stats();
        let utilization = if stats.size > 0 {
            (stats.active as f64 / stats.size as f64) * 100.0
        } else {
            0.0
        };

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
            pool_stats: stats,
            utilization_percent: utilization,
            performance_status,
        })
    }

    /// Close the connection pool gracefully
    pub async fn close(&self) {
        self.pool.close().await;
        tracing::info!("Database connection pool closed");
    }

    /// Get the global pool factory for managing multiple pools
    pub fn pool_factory() -> &'static pool_factory::PoolFactory {
        pool_factory::global_pool_factory()
    }

    /// Create a comprehensive health monitor for this database
    pub fn health_monitor(&self) -> health_monitor::DatabaseHealthMonitor {
        health_monitor::DatabaseHealthMonitor::with_defaults(self.pool.clone())
    }

    /// Create a health monitor with custom configuration
    pub fn health_monitor_with_config(&self, config: health_monitor::HealthMonitorConfig) -> health_monitor::DatabaseHealthMonitor {
        health_monitor::DatabaseHealthMonitor::new(self.pool.clone(), config)
    }

    /// Perform comprehensive health check
    pub async fn comprehensive_health_check(&self) -> Result<health_monitor::HealthReport> {
        let mut monitor = self.health_monitor();
        monitor.health_check().await
    }

    /// Create an alert manager for database health monitoring
    pub fn alert_manager(&self) -> alerting::AlertManager {
        alerting::AlertManager::with_defaults()
    }

    /// Create an alert manager with custom configuration
    pub fn alert_manager_with_config(&self, config: alerting::AlertConfig) -> alerting::AlertManager {
        alerting::AlertManager::new(config)
    }

    /// Create a performance test manager for this database
    pub fn performance_test_manager(&self) -> performance_testing::PerformanceTestManager {
        performance_testing::PerformanceTestManager::with_trading_defaults(self.pool.clone())
    }

    /// Create a performance test manager with custom configuration
    pub fn performance_test_manager_with_config(&self, config: performance_testing::PerformanceTestConfig) -> performance_testing::PerformanceTestManager {
        performance_testing::PerformanceTestManager::new(self.pool.clone(), config)
    }

    /// Create a query performance monitor for real-time monitoring
    pub fn query_monitor(&self) -> query_monitor::QueryPerformanceMonitor {
        query_monitor::QueryPerformanceMonitor::new(self.pool.clone(), None)
    }

    /// Create a query performance monitor with custom configuration
    pub fn query_monitor_with_config(&self, config: query_monitor::MonitorConfig) -> query_monitor::QueryPerformanceMonitor {
        query_monitor::QueryPerformanceMonitor::new(self.pool.clone(), Some(config))
    }

    // ============================================================================
    // MISSING METHODS FOR COMPATIBILITY
    // ============================================================================

    /// Store market tick data (delegates to query manager)
    pub async fn store_market_tick(&self, tick: &types::MarketTick) -> Result<()> {
        self.query_manager().insert_market_tick(tick).await
    }

    /// Get market ticks by symbol with time range
    pub async fn get_market_ticks_by_symbol(&self, symbol: &str, hours: i64) -> Result<Vec<types::MarketTick>> {
        // First get the instrument by symbol
        if let Some(instrument) = self.query_manager().get_instrument_by_symbol(symbol).await? {
            let end_time = chrono::Utc::now();
            let start_time = end_time - chrono::Duration::hours(hours);
            self.query_manager().get_market_ticks_range(
                instrument.id,
                start_time,
                end_time,
                None
            ).await
        } else {
            Ok(Vec::new())
        }
    }

    /// Get market ticks for instrument with time range
    pub async fn get_market_ticks_for_instrument(
        &self,
        instrument_id: uuid::Uuid,
        start_time: Option<chrono::DateTime<chrono::Utc>>,
        end_time: Option<chrono::DateTime<chrono::Utc>>,
        limit: Option<i64>,
    ) -> Result<Vec<types::MarketTick>> {
        self.query_manager().get_market_ticks_for_instrument(
            instrument_id,
            start_time,
            end_time,
            limit
        ).await
    }

    /// Get latest market ticks
    pub async fn get_latest_market_ticks(
        &self,
        instrument_id: Option<uuid::Uuid>,
        limit: Option<i64>
    ) -> Result<Vec<types::MarketTick>> {
        self.query_manager().get_latest_market_ticks(instrument_id, limit).await
    }

    /// Insert trading signal
    pub async fn insert_trading_signal(&self, signal: &types::TradingSignal) -> Result<()> {
        self.query_manager().insert_trading_signal(signal).await
    }

    /// Insert AI prediction
    pub async fn insert_ai_prediction(&self, prediction: &types::AIPrediction) -> Result<()> {
        self.query_manager().insert_ai_prediction(prediction).await
    }

    /// Get instruments with filters
    pub async fn get_instruments_with_filters(
        &self,
        instrument_type: Option<&str>,
        is_active: Option<bool>,
        base_currency: Option<&str>,
        quote_currency: Option<&str>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<types::Instrument>> {
        self.query_manager().get_instruments_with_filters(
            instrument_type,
            is_active,
            base_currency,
            quote_currency,
            limit,
            offset
        ).await
    }

    /// Get instrument by symbol
    pub async fn get_instrument_by_symbol(&self, symbol: &str) -> Result<Option<types::Instrument>> {
        self.query_manager().get_instrument_by_symbol(symbol).await
    }

    /// Get instrument by ID
    pub async fn get_instrument_by_id(&self, id: uuid::Uuid) -> Result<Option<types::Instrument>> {
        self.query_manager().get_instrument_by_id(id).await
    }

    /// Insert instrument
    pub async fn insert_instrument(&self, instrument: &types::Instrument) -> Result<uuid::Uuid> {
        self.query_manager().insert_instrument(instrument).await
    }

    /// Get active instruments
    pub async fn get_active_instruments(&self) -> Result<Vec<types::Instrument>> {
        self.query_manager().get_active_instruments().await
    }

    /// Batch insert market ticks
    pub async fn batch_insert_market_ticks(&self, ticks: &[types::MarketTick]) -> Result<u64> {
        self.query_manager().batch_insert_market_ticks(ticks).await
    }

    /// Get latest market tick for instrument
    pub async fn get_latest_market_tick(&self, instrument_id: uuid::Uuid) -> Result<Option<types::MarketTick>> {
        self.query_manager().get_latest_market_tick(instrument_id).await
    }

    /// Get OHLCV data
    pub async fn get_ohlcv_data(
        &self,
        instrument_id: uuid::Uuid,
        bucket_size: &str,
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>
    ) -> Result<Vec<(chrono::DateTime<chrono::Utc>, f64, f64, f64, f64, f64)>> {
        self.query_manager().get_ohlcv_data(instrument_id, bucket_size, start_time, end_time).await
    }
}
