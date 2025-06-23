// Database connection pool factory for PantherSwap Edge
// Provides easy creation of optimized connection pools for different use cases

use crate::utils::Result;
use crate::database::{Database, DatabasePoolConfig};
use crate::database::connection_pool::{ConnectionPoolManager, PoolConfig};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Connection pool factory for managing multiple database connections
pub struct PoolFactory {
    pools: Arc<RwLock<HashMap<String, Database>>>,
    pool_managers: Arc<RwLock<HashMap<String, ConnectionPoolManager>>>,
}

impl PoolFactory {
    /// Create a new pool factory
    pub fn new() -> Self {
        Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
            pool_managers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create or get a database connection pool by name and configuration
    pub async fn get_or_create_pool(
        &self,
        name: &str,
        database_url: &str,
        config: DatabasePoolConfig,
    ) -> Result<Database> {
        let mut pools = self.pools.write().await;
        
        if let Some(existing_pool) = pools.get(name) {
            info!("Reusing existing database pool: {}", name);
            return Ok(existing_pool.clone());
        }

        info!("Creating new database pool: {} with config: {:?}", name, config);
        let database = Database::new_with_config(database_url, config).await?;
        pools.insert(name.to_string(), database.clone());
        
        Ok(database)
    }

    /// Create or get an advanced connection pool manager
    pub async fn get_or_create_pool_manager(
        &self,
        name: &str,
        database_url: &str,
        config: Option<PoolConfig>,
    ) -> Result<ConnectionPoolManager> {
        let mut managers = self.pool_managers.write().await;
        
        if let Some(existing_manager) = managers.get(name) {
            info!("Reusing existing pool manager: {}", name);
            return Ok(existing_manager.clone());
        }

        info!("Creating new pool manager: {}", name);
        let manager = ConnectionPoolManager::new(database_url, config).await?;
        managers.insert(name.to_string(), manager.clone());
        
        Ok(manager)
    }

    /// Get a production-ready database pool
    pub async fn production_pool(&self, database_url: &str) -> Result<Database> {
        self.get_or_create_pool(
            "production",
            database_url,
            DatabasePoolConfig::production(),
        ).await
    }

    /// Get a development database pool
    pub async fn development_pool(&self, database_url: &str) -> Result<Database> {
        self.get_or_create_pool(
            "development",
            database_url,
            DatabasePoolConfig::development(),
        ).await
    }

    /// Get a testing database pool
    pub async fn testing_pool(&self, database_url: &str) -> Result<Database> {
        self.get_or_create_pool(
            "testing",
            database_url,
            DatabasePoolConfig::testing(),
        ).await
    }

    /// Get a high-frequency trading optimized pool
    pub async fn hft_pool(&self, database_url: &str) -> Result<Database> {
        self.get_or_create_pool(
            "hft",
            database_url,
            DatabasePoolConfig::high_frequency_trading(),
        ).await
    }

    /// Get a cloud-optimized database pool
    pub async fn cloud_pool(&self, database_url: &str) -> Result<Database> {
        self.get_or_create_pool(
            "cloud",
            database_url,
            DatabasePoolConfig::cloud(),
        ).await
    }

    /// Create a custom pool with specific configuration
    pub async fn custom_pool(
        &self,
        name: &str,
        database_url: &str,
        config: DatabasePoolConfig,
    ) -> Result<Database> {
        self.get_or_create_pool(name, database_url, config).await
    }

    /// Get pool statistics for all managed pools
    pub async fn get_all_pool_stats(&self) -> HashMap<String, crate::database::ConnectionPoolStats> {
        let pools = self.pools.read().await;
        let mut stats = HashMap::new();
        
        for (name, database) in pools.iter() {
            stats.insert(name.clone(), database.pool_stats());
        }
        
        stats
    }

    /// Perform health check on all managed pools
    pub async fn health_check_all(&self) -> HashMap<String, bool> {
        let pools = self.pools.read().await;
        let mut health_status = HashMap::new();
        
        for (name, database) in pools.iter() {
            match database.pool_health_check().await {
                Ok(status) => {
                    health_status.insert(name.clone(), status.is_healthy);
                }
                Err(_) => {
                    health_status.insert(name.clone(), false);
                }
            }
        }
        
        health_status
    }

    /// Close a specific pool
    pub async fn close_pool(&self, name: &str) -> Result<()> {
        let mut pools = self.pools.write().await;
        
        if let Some(database) = pools.remove(name) {
            database.close().await;
            info!("Closed database pool: {}", name);
        } else {
            warn!("Attempted to close non-existent pool: {}", name);
        }
        
        Ok(())
    }

    /// Close all managed pools
    pub async fn close_all(&self) {
        let mut pools = self.pools.write().await;
        let mut managers = self.pool_managers.write().await;
        
        for (name, database) in pools.drain() {
            database.close().await;
            info!("Closed database pool: {}", name);
        }
        
        managers.clear();
        info!("All database pools closed");
    }

    /// Get the number of managed pools
    pub async fn pool_count(&self) -> usize {
        self.pools.read().await.len()
    }

    /// List all managed pool names
    pub async fn list_pools(&self) -> Vec<String> {
        self.pools.read().await.keys().cloned().collect()
    }
}

impl Default for PoolFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Global pool factory instance for easy access
static GLOBAL_POOL_FACTORY: once_cell::sync::Lazy<PoolFactory> = 
    once_cell::sync::Lazy::new(|| PoolFactory::new());

/// Get the global pool factory instance
pub fn global_pool_factory() -> &'static PoolFactory {
    &GLOBAL_POOL_FACTORY
}

/// Convenience functions for common pool operations
pub mod convenience {
    use super::*;

    /// Get a production database pool using the global factory
    pub async fn production_database(database_url: &str) -> Result<Database> {
        global_pool_factory().production_pool(database_url).await
    }

    /// Get a development database pool using the global factory
    pub async fn development_database(database_url: &str) -> Result<Database> {
        global_pool_factory().development_pool(database_url).await
    }

    /// Get a testing database pool using the global factory
    pub async fn testing_database(database_url: &str) -> Result<Database> {
        global_pool_factory().testing_pool(database_url).await
    }

    /// Get a high-frequency trading database pool using the global factory
    pub async fn hft_database(database_url: &str) -> Result<Database> {
        global_pool_factory().hft_pool(database_url).await
    }

    /// Get a cloud database pool using the global factory
    pub async fn cloud_database(database_url: &str) -> Result<Database> {
        global_pool_factory().cloud_pool(database_url).await
    }

    /// Create a custom database pool using the global factory
    pub async fn custom_database(
        name: &str,
        database_url: &str,
        config: DatabasePoolConfig,
    ) -> Result<Database> {
        global_pool_factory().custom_pool(name, database_url, config).await
    }

    /// Get health status of all pools
    pub async fn health_check_all() -> HashMap<String, bool> {
        global_pool_factory().health_check_all().await
    }

    /// Get statistics for all pools
    pub async fn all_pool_stats() -> HashMap<String, crate::database::ConnectionPoolStats> {
        global_pool_factory().get_all_pool_stats().await
    }

    /// Close all pools (useful for application shutdown)
    pub async fn shutdown_all_pools() {
        global_pool_factory().close_all().await;
    }
}

/// Pool configuration presets for common scenarios
pub mod presets {
    use super::*;

    /// Configuration for microservices with moderate load
    pub fn microservice() -> DatabasePoolConfig {
        DatabasePoolConfig {
            min_connections: 5,
            max_connections: 25,
            acquire_timeout: std::time::Duration::from_secs(3),
            idle_timeout: Some(std::time::Duration::from_secs(300)),
            max_lifetime: Some(std::time::Duration::from_secs(1800)),
            test_before_acquire: true,
        }
    }

    /// Configuration for batch processing workloads
    pub fn batch_processing() -> DatabasePoolConfig {
        DatabasePoolConfig {
            min_connections: 2,
            max_connections: 15,
            acquire_timeout: std::time::Duration::from_secs(30),
            idle_timeout: Some(std::time::Duration::from_secs(600)),
            max_lifetime: Some(std::time::Duration::from_secs(3600)),
            test_before_acquire: false,
        }
    }

    /// Configuration for analytics workloads
    pub fn analytics() -> DatabasePoolConfig {
        DatabasePoolConfig {
            min_connections: 3,
            max_connections: 20,
            acquire_timeout: std::time::Duration::from_secs(15),
            idle_timeout: Some(std::time::Duration::from_secs(900)),
            max_lifetime: Some(std::time::Duration::from_secs(7200)),
            test_before_acquire: true,
        }
    }

    /// Configuration for real-time applications
    pub fn realtime() -> DatabasePoolConfig {
        DatabasePoolConfig {
            min_connections: 15,
            max_connections: 75,
            acquire_timeout: std::time::Duration::from_secs(1),
            idle_timeout: Some(std::time::Duration::from_secs(60)),
            max_lifetime: Some(std::time::Duration::from_secs(600)),
            test_before_acquire: true,
        }
    }
}
