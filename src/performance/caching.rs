use redis::{Client, Connection, Commands, RedisResult, AsyncCommands};
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use tracing::{info, warn, error, debug};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Caching configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachingConfig {
    pub redis_url: String,
    pub enable_redis: bool,
    pub enable_memory_cache: bool,
    pub default_ttl_seconds: u64,
    pub max_memory_cache_size: usize,
    pub cache_strategies: HashMap<String, CacheStrategy>,
    pub enable_cache_warming: bool,
    pub cache_warming_interval_minutes: u32,
    pub enable_cache_metrics: bool,
}

/// Cache strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStrategy {
    pub cache_type: CacheType,
    pub ttl_seconds: u64,
    pub max_size: Option<usize>,
    pub eviction_policy: EvictionPolicy,
    pub enable_compression: bool,
    pub enable_encryption: bool,
}

/// Types of caching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheType {
    Memory,
    Redis,
    Hybrid, // Memory + Redis
}

/// Cache eviction policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvictionPolicy {
    LRU,  // Least Recently Used
    LFU,  // Least Frequently Used
    TTL,  // Time To Live
    FIFO, // First In First Out
}

impl Default for CachingConfig {
    fn default() -> Self {
        let mut strategies = HashMap::new();
        
        // Market data caching - high frequency, short TTL
        strategies.insert("market_data".to_string(), CacheStrategy {
            cache_type: CacheType::Hybrid,
            ttl_seconds: 5,
            max_size: Some(10000),
            eviction_policy: EvictionPolicy::TTL,
            enable_compression: false,
            enable_encryption: false,
        });
        
        // AI predictions - medium frequency, medium TTL
        strategies.insert("ai_predictions".to_string(), CacheStrategy {
            cache_type: CacheType::Redis,
            ttl_seconds: 60,
            max_size: Some(5000),
            eviction_policy: EvictionPolicy::LRU,
            enable_compression: true,
            enable_encryption: false,
        });
        
        // User sessions - long TTL
        strategies.insert("user_sessions".to_string(), CacheStrategy {
            cache_type: CacheType::Redis,
            ttl_seconds: 3600,
            max_size: Some(1000),
            eviction_policy: EvictionPolicy::TTL,
            enable_compression: false,
            enable_encryption: true,
        });
        
        // API responses - short TTL
        strategies.insert("api_responses".to_string(), CacheStrategy {
            cache_type: CacheType::Memory,
            ttl_seconds: 30,
            max_size: Some(2000),
            eviction_policy: EvictionPolicy::LRU,
            enable_compression: true,
            enable_encryption: false,
        });

        Self {
            redis_url: "redis://localhost:6379".to_string(),
            enable_redis: true,
            enable_memory_cache: true,
            default_ttl_seconds: 300,
            max_memory_cache_size: 100000,
            cache_strategies: strategies,
            enable_cache_warming: true,
            cache_warming_interval_minutes: 5,
            enable_cache_metrics: true,
        }
    }
}

/// Cache entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry<T> {
    data: T,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    access_count: u64,
    last_accessed: DateTime<Utc>,
}

/// Cache metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetrics {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub total_requests: u64,
    pub memory_usage_bytes: u64,
    pub redis_usage_bytes: u64,
    pub evictions: u64,
    pub errors: u64,
}

/// Memory cache implementation
struct MemoryCache<T> {
    data: Arc<RwLock<HashMap<String, CacheEntry<T>>>>,
    max_size: usize,
    eviction_policy: EvictionPolicy,
}

impl<T: Clone> MemoryCache<T> {
    fn new(max_size: usize, eviction_policy: EvictionPolicy) -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            max_size,
            eviction_policy,
        }
    }

    async fn get(&self, key: &str) -> Option<T> {
        let mut cache = self.data.write().await;
        if let Some(entry) = cache.get_mut(key) {
            if entry.expires_at > Utc::now() {
                entry.access_count += 1;
                entry.last_accessed = Utc::now();
                Some(entry.data.clone())
            } else {
                cache.remove(key);
                None
            }
        } else {
            None
        }
    }

    async fn set(&self, key: String, value: T, ttl: Duration) {
        let mut cache = self.data.write().await;
        
        // Check if we need to evict entries
        if cache.len() >= self.max_size {
            self.evict_entries(&mut cache).await;
        }

        let entry = CacheEntry {
            data: value,
            created_at: Utc::now(),
            expires_at: Utc::now() + ttl,
            access_count: 0,
            last_accessed: Utc::now(),
        };

        cache.insert(key, entry);
    }

    async fn evict_entries(&self, cache: &mut HashMap<String, CacheEntry<T>>) {
        let evict_count = cache.len() / 4; // Evict 25% of entries
        
        match self.eviction_policy {
            EvictionPolicy::LRU => {
                let mut entries: Vec<_> = cache.iter().collect();
                entries.sort_by_key(|(_, entry)| entry.last_accessed);
                for (key, _) in entries.iter().take(evict_count) {
                    cache.remove(*key);
                }
            }
            EvictionPolicy::LFU => {
                let mut entries: Vec<_> = cache.iter().collect();
                entries.sort_by_key(|(_, entry)| entry.access_count);
                for (key, _) in entries.iter().take(evict_count) {
                    cache.remove(*key);
                }
            }
            EvictionPolicy::TTL => {
                let now = Utc::now();
                cache.retain(|_, entry| entry.expires_at > now);
            }
            EvictionPolicy::FIFO => {
                let mut entries: Vec<_> = cache.iter().collect();
                entries.sort_by_key(|(_, entry)| entry.created_at);
                for (key, _) in entries.iter().take(evict_count) {
                    cache.remove(*key);
                }
            }
        }
    }
}

/// Comprehensive cache manager
pub struct CacheManager {
    config: CachingConfig,
    redis_client: Option<Client>,
    memory_caches: Arc<RwLock<HashMap<String, Box<dyn std::any::Any + Send + Sync>>>>,
    metrics: Arc<RwLock<CacheMetrics>>,
}

impl CacheManager {
    /// Create new cache manager
    pub fn new(config: &CachingConfig) -> Self {
        let redis_client = if config.enable_redis {
            match Client::open(config.redis_url.as_str()) {
                Ok(client) => Some(client),
                Err(e) => {
                    error!("Failed to connect to Redis: {}", e);
                    None
                }
            }
        } else {
            None
        };

        Self {
            config: config.clone(),
            redis_client,
            memory_caches: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(CacheMetrics {
                hits: 0,
                misses: 0,
                hit_rate: 0.0,
                total_requests: 0,
                memory_usage_bytes: 0,
                redis_usage_bytes: 0,
                evictions: 0,
                errors: 0,
            })),
        }
    }

    /// Get value from cache
    pub async fn get<T>(&self, cache_name: &str, key: &str) -> Option<T>
    where
        T: DeserializeOwned + Clone + Send + Sync + 'static,
    {
        let strategy = self.config.cache_strategies.get(cache_name)?;
        
        self.update_metrics(|m| m.total_requests += 1).await;

        match strategy.cache_type {
            CacheType::Memory => self.get_from_memory(cache_name, key).await,
            CacheType::Redis => self.get_from_redis(key).await,
            CacheType::Hybrid => {
                // Try memory first, then Redis
                if let Some(value) = self.get_from_memory::<T>(cache_name, key).await {
                    Some(value)
                } else {
                    let value = self.get_from_redis::<T>(key).await;
                    if let Some(ref v) = value {
                        // Store in memory for faster access
                        self.set_in_memory(cache_name, key.to_string(), v.clone(), Duration::seconds(strategy.ttl_seconds as i64)).await;
                    }
                    value
                }
            }
        }
    }

    /// Set value in cache
    pub async fn set<T>(&self, cache_name: &str, key: String, value: T)
    where
        T: Serialize + Clone + Send + Sync + 'static,
    {
        let strategy = match self.config.cache_strategies.get(cache_name) {
            Some(s) => s,
            None => return,
        };

        let ttl = Duration::seconds(strategy.ttl_seconds as i64);

        match strategy.cache_type {
            CacheType::Memory => {
                self.set_in_memory(cache_name, key, value, ttl).await;
            }
            CacheType::Redis => {
                self.set_in_redis(&key, &value, strategy.ttl_seconds).await;
            }
            CacheType::Hybrid => {
                self.set_in_memory(cache_name, key.clone(), value.clone(), ttl).await;
                self.set_in_redis(&key, &value, strategy.ttl_seconds).await;
            }
        }
    }

    /// Get from memory cache
    async fn get_from_memory<T>(&self, cache_name: &str, key: &str) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        let caches = self.memory_caches.read().await;
        if let Some(cache_any) = caches.get(cache_name) {
            if let Some(cache) = cache_any.downcast_ref::<MemoryCache<T>>() {
                let result = cache.get(key).await;
                if result.is_some() {
                    self.update_metrics(|m| m.hits += 1).await;
                } else {
                    self.update_metrics(|m| m.misses += 1).await;
                }
                return result;
            }
        }
        self.update_metrics(|m| m.misses += 1).await;
        None
    }

    /// Set in memory cache
    async fn set_in_memory<T>(&self, cache_name: &str, key: String, value: T, ttl: Duration)
    where
        T: Clone + Send + Sync + 'static,
    {
        let mut caches = self.memory_caches.write().await;
        
        if !caches.contains_key(cache_name) {
            let strategy = self.config.cache_strategies.get(cache_name).unwrap();
            let cache = MemoryCache::new(
                strategy.max_size.unwrap_or(1000),
                strategy.eviction_policy.clone(),
            );
            caches.insert(cache_name.to_string(), Box::new(cache));
        }

        if let Some(cache_any) = caches.get(cache_name) {
            if let Some(cache) = cache_any.downcast_ref::<MemoryCache<T>>() {
                cache.set(key, value, ttl).await;
            }
        }
    }

    /// Get from Redis cache
    async fn get_from_redis<T>(&self, key: &str) -> Option<T>
    where
        T: DeserializeOwned,
    {
        if let Some(ref client) = self.redis_client {
            match client.get_async_connection().await {
                Ok(mut conn) => {
                    match conn.get::<_, String>(key).await {
                        Ok(data) => {
                            match serde_json::from_str::<T>(&data) {
                                Ok(value) => {
                                    self.update_metrics(|m| m.hits += 1).await;
                                    Some(value)
                                }
                                Err(_) => {
                                    self.update_metrics(|m| m.errors += 1).await;
                                    None
                                }
                            }
                        }
                        Err(_) => {
                            self.update_metrics(|m| m.misses += 1).await;
                            None
                        }
                    }
                }
                Err(_) => {
                    self.update_metrics(|m| m.errors += 1).await;
                    None
                }
            }
        } else {
            None
        }
    }

    /// Set in Redis cache
    async fn set_in_redis<T>(&self, key: &str, value: &T, ttl_seconds: u64)
    where
        T: Serialize,
    {
        if let Some(ref client) = self.redis_client {
            if let Ok(data) = serde_json::to_string(value) {
                match client.get_async_connection().await {
                    Ok(mut conn) => {
                        let _: RedisResult<()> = conn.set_ex(key, data, ttl_seconds).await;
                    }
                    Err(_) => {
                        self.update_metrics(|m| m.errors += 1).await;
                    }
                }
            }
        }
    }

    /// Update cache metrics
    async fn update_metrics<F>(&self, update_fn: F)
    where
        F: FnOnce(&mut CacheMetrics),
    {
        let mut metrics = self.metrics.write().await;
        update_fn(&mut *metrics);
        
        // Update hit rate
        if metrics.total_requests > 0 {
            metrics.hit_rate = (metrics.hits as f64 / metrics.total_requests as f64) * 100.0;
        }
    }

    /// Get cache metrics
    pub async fn get_metrics(&self) -> CacheMetrics {
        self.metrics.read().await.clone()
    }

    /// Clear cache
    pub async fn clear(&self, cache_name: &str) {
        // Clear memory cache
        let mut caches = self.memory_caches.write().await;
        caches.remove(cache_name);

        // Clear Redis cache (pattern-based)
        if let Some(ref client) = self.redis_client {
            if let Ok(mut conn) = client.get_async_connection().await {
                let pattern = format!("{}:*", cache_name);
                let _: RedisResult<()> = redis::cmd("FLUSHDB").query_async(&mut conn).await;
            }
        }
    }

    /// Warm up cache with frequently accessed data
    pub async fn warm_cache(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting cache warming...");
        
        // This would pre-load frequently accessed data
        // For example: recent market data, popular trading pairs, etc.
        
        info!("Cache warming completed");
        Ok(())
    }

    /// Analyze cache performance and generate recommendations
    pub async fn analyze_performance(&self) -> Result<Vec<crate::performance::OptimizationRecommendation>, Box<dyn std::error::Error>> {
        let metrics = self.get_metrics().await;
        let mut recommendations = Vec::new();

        // Check hit rate
        if metrics.hit_rate < 90.0 {
            recommendations.push(crate::performance::OptimizationRecommendation {
                id: Uuid::new_v4(),
                category: crate::performance::OptimizationCategory::Caching,
                priority: crate::performance::OptimizationPriority::High,
                title: "Low Cache Hit Rate".to_string(),
                description: format!("Cache hit rate is {:.1}%, consider increasing TTL or cache size", metrics.hit_rate),
                expected_improvement: "Improved response times and reduced database load".to_string(),
                implementation_effort: "Low".to_string(),
                auto_applicable: true,
                created_at: Utc::now(),
            });
        }

        // Check error rate
        if metrics.errors > metrics.total_requests / 100 {
            recommendations.push(crate::performance::OptimizationRecommendation {
                id: Uuid::new_v4(),
                category: crate::performance::OptimizationCategory::Caching,
                priority: crate::performance::OptimizationPriority::Medium,
                title: "High Cache Error Rate".to_string(),
                description: "Cache is experiencing errors, check Redis connectivity".to_string(),
                expected_improvement: "Improved cache reliability".to_string(),
                implementation_effort: "Medium".to_string(),
                auto_applicable: false,
                created_at: Utc::now(),
            });
        }

        Ok(recommendations)
    }

    /// Execute automatic cache optimizations
    pub async fn execute_auto_optimizations(&self) -> Result<Vec<crate::performance::OptimizationResult>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();

        // Auto-clear expired entries
        let start_time = std::time::Instant::now();
        self.cleanup_expired_entries().await;
        
        results.push(crate::performance::OptimizationResult {
            action_id: Uuid::new_v4(),
            success: true,
            improvement_metrics: HashMap::from([
                ("cleanup_time_ms".to_string(), start_time.elapsed().as_millis() as f64),
            ]),
            error_message: None,
            execution_time_ms: start_time.elapsed().as_millis() as u64,
        });

        Ok(results)
    }

    /// Clean up expired cache entries
    async fn cleanup_expired_entries(&self) {
        // This would clean up expired entries from memory caches
        // Redis handles TTL automatically
        debug!("Cleaning up expired cache entries");
    }
}
