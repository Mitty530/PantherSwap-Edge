use crate::database::{Database, types::MarketTick};
use crate::market_data::types::{MarketQuote, DataQualityResult};
use crate::utils::Result;
use chrono::{DateTime, Utc, Timelike};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use tracing::{info, error, debug};
use uuid::Uuid;

/// Enhanced configuration for the market data processing pipeline with real-time optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub batch_size: usize,
    pub quality_threshold: f64,
    pub max_age_seconds: i64,
    pub enable_normalization: bool,
    pub enable_filtering: bool,
    pub enable_deduplication: bool,
    pub buffer_size: usize,
    pub enable_real_time_processing: bool,
    pub real_time_flush_interval_ms: u64,
    pub enable_time_series_optimization: bool,
    pub enable_compression: bool,
    pub enable_parallel_processing: bool,
    pub max_concurrent_batches: usize,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            quality_threshold: 0.7,
            max_age_seconds: 300, // 5 minutes
            enable_normalization: true,
            enable_filtering: true,
            enable_deduplication: true,
            buffer_size: 1000,
            enable_real_time_processing: true,
            real_time_flush_interval_ms: 1000, // 1 second for real-time
            enable_time_series_optimization: true,
            enable_compression: true,
            enable_parallel_processing: true,
            max_concurrent_batches: 4,
        }
    }
}

/// Pipeline metrics for monitoring and performance tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineMetrics {
    pub total_processed: u64,
    pub total_filtered: u64,
    pub total_normalized: u64,
    pub total_stored: u64,
    pub total_errors: u64,
    pub average_quality_score: f64,
    pub processing_latency_ms: f64,
    pub last_update: DateTime<Utc>,
    pub throughput_per_second: f64,
}

impl Default for PipelineMetrics {
    fn default() -> Self {
        Self {
            total_processed: 0,
            total_filtered: 0,
            total_normalized: 0,
            total_stored: 0,
            total_errors: 0,
            average_quality_score: 0.0,
            processing_latency_ms: 0.0,
            last_update: Utc::now(),
            throughput_per_second: 0.0,
        }
    }
}

/// Normalized market data structure for internal processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedMarketData {
    pub symbol: String,
    pub instrument_id: Uuid,
    pub provider: String,
    pub timestamp: DateTime<Utc>,
    pub bid_price: f64,
    pub ask_price: f64,
    pub mid_price: f64,
    pub spread: f64,
    pub spread_bps: f64, // Spread in basis points
    pub bid_size: Option<f64>,
    pub ask_size: Option<f64>,
    pub volume: Option<f64>,
    pub quality_score: f64,
    pub quality_issues: Vec<String>,
    pub is_valid: bool,
    pub processing_timestamp: DateTime<Utc>,
    pub raw_data: serde_json::Value,
}

/// Enhanced Market Data Processing Pipeline with real-time capabilities
pub struct MarketDataPipeline {
    config: PipelineConfig,
    database: Database,
    metrics: PipelineMetrics,
    data_buffer: VecDeque<NormalizedMarketData>,
    deduplication_cache: HashMap<String, DateTime<Utc>>,
    quality_scorer: DataQualityScorer,
    normalizer: DataNormalizer,
    filter: DataFilter,
    real_time_buffer: VecDeque<NormalizedMarketData>,
    last_flush_time: DateTime<Utc>,
    time_series_optimizer: TimeSeriesOptimizer,
}

impl MarketDataPipeline {
    pub fn new(database: Database, config: PipelineConfig) -> Self {
        Self {
            config: config.clone(),
            database,
            metrics: PipelineMetrics::default(),
            data_buffer: VecDeque::with_capacity(config.buffer_size),
            deduplication_cache: HashMap::new(),
            quality_scorer: DataQualityScorer::new(),
            normalizer: DataNormalizer::new(),
            filter: DataFilter::new(),
            real_time_buffer: VecDeque::with_capacity(config.buffer_size / 10), // Smaller real-time buffer
            last_flush_time: Utc::now(),
            time_series_optimizer: TimeSeriesOptimizer::new(),
        }
    }

    /// Process a single market quote through the pipeline
    pub async fn process_quote(&mut self, quote: MarketQuote, instrument_id: Uuid) -> Result<bool> {
        let start_time = std::time::Instant::now();
        
        debug!("Processing quote for symbol: {}", quote.symbol);
        
        // Step 1: Normalize the data
        let normalized_data = if self.config.enable_normalization {
            self.normalizer.normalize_quote(quote, instrument_id)?
        } else {
            self.convert_quote_to_normalized(quote, instrument_id)?
        };

        // Step 2: Quality scoring
        let quality_result = self.quality_scorer.assess_quality(&normalized_data)?;
        let mut normalized_data = normalized_data;
        normalized_data.quality_score = quality_result.score;
        normalized_data.quality_issues = quality_result.issues;
        normalized_data.is_valid = quality_result.is_valid;

        // Step 3: Filtering
        if self.config.enable_filtering && !self.filter.should_accept(&normalized_data, &self.config)? {
            self.metrics.total_filtered += 1;
            debug!("Quote filtered out for symbol: {}", normalized_data.symbol);
            return Ok(false);
        }

        // Step 4: Deduplication
        if self.config.enable_deduplication && self.is_duplicate(&normalized_data)? {
            debug!("Duplicate quote detected for symbol: {}", normalized_data.symbol);
            return Ok(false);
        }

        // Step 5: Add to buffer
        self.data_buffer.push_back(normalized_data);
        
        // Step 6: Process buffer if it's full or force flush
        if self.data_buffer.len() >= self.config.batch_size {
            self.flush_buffer().await?;
        }

        // Update metrics
        self.metrics.total_processed += 1;
        self.metrics.processing_latency_ms = start_time.elapsed().as_millis() as f64;
        self.metrics.last_update = Utc::now();

        Ok(true)
    }

    /// Process multiple quotes in batch
    pub async fn process_batch(&mut self, quotes: Vec<(MarketQuote, Uuid)>) -> Result<usize> {
        let mut processed_count = 0;
        
        for (quote, instrument_id) in quotes {
            if self.process_quote(quote, instrument_id).await? {
                processed_count += 1;
            }
        }

        // Force flush if we have data
        if !self.data_buffer.is_empty() {
            self.flush_buffer().await?;
        }

        Ok(processed_count)
    }

    /// Flush the buffer to database with time-series optimization
    pub async fn flush_buffer(&mut self) -> Result<()> {
        if self.data_buffer.is_empty() {
            return Ok(());
        }

        let batch_size = self.data_buffer.len();
        info!("Flushing {} items from pipeline buffer", batch_size);

        // Convert normalized data to MarketTick for database storage
        let mut market_ticks = Vec::with_capacity(batch_size);

        while let Some(normalized_data) = self.data_buffer.pop_front() {
            let market_tick = self.convert_normalized_to_tick(normalized_data)?;
            market_ticks.push(market_tick);
        }

        // Apply time-series optimization
        if self.config.enable_time_series_optimization {
            self.time_series_optimizer.optimize_for_storage(&mut market_ticks)?;
        }

        // Batch insert to database
        let query_manager = self.database.query_manager();
        match query_manager.batch_insert_market_ticks(&market_ticks).await {
            Ok(inserted_count) => {
                self.metrics.total_stored += inserted_count;
                info!("Successfully stored {} optimized market ticks", inserted_count);
            }
            Err(e) => {
                self.metrics.total_errors += 1;
                error!("Failed to store market ticks: {}", e);
                return Err(e);
            }
        }

        Ok(())
    }

    /// Process quote in real-time mode with immediate flushing
    pub async fn process_quote_real_time(&mut self, quote: MarketQuote, instrument_id: Uuid) -> Result<bool> {
        if !self.config.enable_real_time_processing {
            return self.process_quote(quote, instrument_id).await;
        }

        let start_time = std::time::Instant::now();

        // Process through normal pipeline
        let normalized_data = if self.config.enable_normalization {
            self.normalizer.normalize_quote(quote, instrument_id)?
        } else {
            self.convert_quote_to_normalized(quote, instrument_id)?
        };

        // Quality scoring
        let quality_result = self.quality_scorer.assess_quality(&normalized_data)?;
        let mut normalized_data = normalized_data;
        normalized_data.quality_score = quality_result.score;
        normalized_data.quality_issues = quality_result.issues;
        normalized_data.is_valid = quality_result.is_valid;

        // Filtering
        if self.config.enable_filtering && !self.filter.should_accept(&normalized_data, &self.config)? {
            self.metrics.total_filtered += 1;
            return Ok(false);
        }

        // Deduplication
        if self.config.enable_deduplication && self.is_duplicate(&normalized_data)? {
            return Ok(false);
        }

        // Add to real-time buffer
        self.real_time_buffer.push_back(normalized_data);

        // Check if we should flush real-time buffer
        let should_flush = self.real_time_buffer.len() >= (self.config.batch_size / 10) ||
            (Utc::now() - self.last_flush_time).num_milliseconds() >= self.config.real_time_flush_interval_ms as i64;

        if should_flush {
            self.flush_real_time_buffer().await?;
        }

        // Update metrics
        self.metrics.total_processed += 1;
        self.metrics.processing_latency_ms = start_time.elapsed().as_millis() as f64;
        self.metrics.last_update = Utc::now();

        Ok(true)
    }

    /// Flush real-time buffer for immediate storage
    async fn flush_real_time_buffer(&mut self) -> Result<()> {
        if self.real_time_buffer.is_empty() {
            return Ok(());
        }

        let batch_size = self.real_time_buffer.len();
        debug!("Flushing {} items from real-time buffer", batch_size);

        // Convert to market ticks
        let mut market_ticks = Vec::with_capacity(batch_size);
        while let Some(normalized_data) = self.real_time_buffer.pop_front() {
            let market_tick = self.convert_normalized_to_tick(normalized_data)?;
            market_ticks.push(market_tick);
        }

        // Apply time-series optimization
        if self.config.enable_time_series_optimization {
            self.time_series_optimizer.optimize_for_storage(&mut market_ticks)?;
        }

        // Immediate insert to database
        let query_manager = self.database.query_manager();
        match query_manager.batch_insert_market_ticks(&market_ticks).await {
            Ok(inserted_count) => {
                self.metrics.total_stored += inserted_count;
                debug!("Real-time stored {} market ticks", inserted_count);
            }
            Err(e) => {
                self.metrics.total_errors += 1;
                error!("Failed to store real-time market ticks: {}", e);
                return Err(e);
            }
        }

        self.last_flush_time = Utc::now();
        Ok(())
    }

    /// Force flush all buffers
    pub async fn force_flush_all(&mut self) -> Result<()> {
        // Flush real-time buffer first
        if !self.real_time_buffer.is_empty() {
            self.flush_real_time_buffer().await?;
        }

        // Then flush main buffer
        if !self.data_buffer.is_empty() {
            self.flush_buffer().await?;
        }

        Ok(())
    }

    /// Get current pipeline metrics
    pub fn get_metrics(&self) -> &PipelineMetrics {
        &self.metrics
    }

    /// Reset pipeline metrics
    pub fn reset_metrics(&mut self) {
        self.metrics = PipelineMetrics::default();
    }

    /// Get pipeline configuration
    pub fn get_config(&self) -> &PipelineConfig {
        &self.config
    }

    /// Update pipeline configuration
    pub fn update_config(&mut self, config: PipelineConfig) {
        self.config = config;
    }

    /// Check if data is duplicate
    fn is_duplicate(&mut self, data: &NormalizedMarketData) -> Result<bool> {
        let cache_key = format!("{}_{}", data.symbol, data.provider);

        if let Some(last_timestamp) = self.deduplication_cache.get(&cache_key) {
            if data.timestamp <= *last_timestamp {
                return Ok(true);
            }
        }

        // Update cache
        self.deduplication_cache.insert(cache_key, data.timestamp);

        // Clean old entries from cache (keep only last hour)
        let cutoff_time = Utc::now() - chrono::Duration::hours(1);
        self.deduplication_cache.retain(|_, timestamp| *timestamp > cutoff_time);

        Ok(false)
    }

    /// Convert MarketQuote to NormalizedMarketData
    fn convert_quote_to_normalized(&self, quote: MarketQuote, instrument_id: Uuid) -> Result<NormalizedMarketData> {
        let spread = quote.spread.unwrap_or_else(|| quote.ask_price - quote.bid_price);
        let spread_bps = (spread / quote.bid_price) * 10000.0; // Convert to basis points

        Ok(NormalizedMarketData {
            symbol: quote.symbol.clone(),
            instrument_id,
            provider: quote.provider.clone(),
            timestamp: quote.timestamp,
            bid_price: quote.bid_price,
            ask_price: quote.ask_price,
            mid_price: quote.mid_price,
            spread,
            spread_bps,
            bid_size: quote.bid_size,
            ask_size: quote.ask_size,
            volume: quote.volume,
            quality_score: quote.data_quality,
            quality_issues: Vec::new(),
            is_valid: true,
            processing_timestamp: Utc::now(),
            raw_data: serde_json::json!({
                "original_quote": quote,
                "processing_stage": "conversion"
            }),
        })
    }

    /// Convert NormalizedMarketData to MarketTick for database storage
    fn convert_normalized_to_tick(&self, data: NormalizedMarketData) -> Result<MarketTick> {
        Ok(MarketTick {
            timestamp: data.timestamp,
            instrument_id: data.instrument_id,
            provider: data.provider,
            bid_price: data.bid_price,
            ask_price: data.ask_price,
            bid_size: data.bid_size.unwrap_or(0.0),
            ask_size: data.ask_size.unwrap_or(0.0),
            last_price: Some(data.mid_price),
            volume: data.volume,
            spread: data.spread,
            data_quality_score: data.quality_score,
            raw_data: data.raw_data,
            symbol: Some(data.symbol),
            price: Some(data.mid_price),
            bid: Some(data.bid_price),
            ask: Some(data.ask_price),
        })
    }
}

/// Data Quality Scorer for assessing market data quality
pub struct DataQualityScorer {
    // Configuration for quality scoring
}

impl DataQualityScorer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn assess_quality(&self, data: &NormalizedMarketData) -> Result<DataQualityResult> {
        let mut score: f64 = 1.0;
        let mut issues = Vec::new();

        // Check price validity
        if data.bid_price <= 0.0 || data.ask_price <= 0.0 {
            score *= 0.0;
            issues.push("Invalid prices (zero or negative)".to_string());
        }

        // Check spread reasonableness
        if data.spread < 0.0 {
            score *= 0.0;
            issues.push("Negative spread".to_string());
        } else if data.spread_bps > 100.0 {  // More than 1% spread
            score *= 0.5;
            issues.push("Very wide spread".to_string());
        } else if data.spread_bps > 50.0 {   // More than 0.5% spread
            score *= 0.8;
            issues.push("Wide spread".to_string());
        }

        // Check timestamp freshness
        let age_seconds = (Utc::now() - data.timestamp).num_seconds();
        if age_seconds > 3600 {  // Older than 1 hour
            score *= 0.3;
            issues.push("Stale data (>1 hour old)".to_string());
        } else if age_seconds > 300 {  // Older than 5 minutes
            score *= 0.7;
            issues.push("Old data (>5 minutes)".to_string());
        }

        // Check for future timestamps
        if age_seconds < -60 {  // More than 1 minute in future
            score *= 0.5;
            issues.push("Future timestamp".to_string());
        }

        // Check bid-ask relationship
        if data.ask_price <= data.bid_price {
            score *= 0.2;
            issues.push("Ask price not greater than bid price".to_string());
        }

        let is_valid = score >= 0.5 && !issues.iter().any(|issue|
            issue.contains("Invalid prices") || issue.contains("Negative spread")
        );

        Ok(DataQualityResult {
            score: score.max(0.0).min(1.0),
            issues,
            is_valid,
        })
    }
}

/// Data Normalizer for standardizing market data from different providers
pub struct DataNormalizer {
    // Configuration for normalization
}

impl DataNormalizer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn normalize_quote(&self, quote: MarketQuote, instrument_id: Uuid) -> Result<NormalizedMarketData> {
        // Normalize symbol format (remove special characters, uppercase)
        let normalized_symbol = self.normalize_symbol(&quote.symbol);

        // Calculate spread and spread in basis points
        let spread = quote.spread.unwrap_or_else(|| quote.ask_price - quote.bid_price);
        let spread_bps = if quote.bid_price > 0.0 {
            (spread / quote.bid_price) * 10000.0
        } else {
            0.0
        };

        // Normalize prices (round to appropriate precision for forex)
        let bid_price = self.normalize_price(quote.bid_price, 5);
        let ask_price = self.normalize_price(quote.ask_price, 5);
        let mid_price = self.normalize_price(quote.mid_price, 5);

        // Normalize timestamp (ensure UTC and round to seconds)
        let normalized_timestamp = quote.timestamp
            .with_nanosecond(0)
            .unwrap_or(quote.timestamp);

        Ok(NormalizedMarketData {
            symbol: normalized_symbol,
            instrument_id,
            provider: quote.provider.to_lowercase(),
            timestamp: normalized_timestamp,
            bid_price,
            ask_price,
            mid_price,
            spread: self.normalize_price(spread, 5),
            spread_bps: self.normalize_price(spread_bps, 2),
            bid_size: quote.bid_size.map(|size| self.normalize_size(size)),
            ask_size: quote.ask_size.map(|size| self.normalize_size(size)),
            volume: quote.volume.map(|vol| self.normalize_size(vol)),
            quality_score: quote.data_quality,
            quality_issues: Vec::new(),
            is_valid: true,
            processing_timestamp: Utc::now(),
            raw_data: serde_json::json!({
                "original_quote": quote,
                "normalization_applied": true,
                "processing_stage": "normalization"
            }),
        })
    }

    fn normalize_symbol(&self, symbol: &str) -> String {
        symbol.replace("/", "").replace("-", "").replace("_", "").to_uppercase()
    }

    fn normalize_price(&self, price: f64, decimal_places: u32) -> f64 {
        let multiplier = 10_f64.powi(decimal_places as i32);
        (price * multiplier).round() / multiplier
    }

    fn normalize_size(&self, size: f64) -> f64 {
        // Round to 2 decimal places for sizes
        (size * 100.0).round() / 100.0
    }
}

/// Data Filter for filtering market data based on quality and business rules
pub struct DataFilter {
    // Configuration for filtering
}

impl DataFilter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn should_accept(&self, data: &NormalizedMarketData, config: &PipelineConfig) -> Result<bool> {
        // Quality threshold check
        if data.quality_score < config.quality_threshold {
            debug!("Rejecting data due to low quality score: {}", data.quality_score);
            return Ok(false);
        }

        // Age check
        let age_seconds = (Utc::now() - data.timestamp).num_seconds();
        if age_seconds > config.max_age_seconds {
            debug!("Rejecting data due to age: {} seconds", age_seconds);
            return Ok(false);
        }

        // Business rule checks
        if !self.passes_business_rules(data)? {
            debug!("Rejecting data due to business rule violation");
            return Ok(false);
        }

        Ok(true)
    }

    fn passes_business_rules(&self, data: &NormalizedMarketData) -> Result<bool> {
        // Check for reasonable forex price ranges
        if self.is_forex_pair(&data.symbol) {
            // Major forex pairs should be within reasonable ranges
            if data.bid_price < 0.01 || data.bid_price > 1000.0 {
                return Ok(false);
            }

            // Spread should not be more than 1% for major pairs
            if data.spread_bps > 100.0 {
                return Ok(false);
            }
        }

        // Check for price jumps (more than 5% change would be suspicious)
        // This would require historical data comparison in a real implementation

        // Check for minimum spread (should not be zero for real market data)
        if data.spread <= 0.0 {
            return Ok(false);
        }

        Ok(true)
    }

    fn is_forex_pair(&self, symbol: &str) -> bool {
        let major_currencies = ["USD", "EUR", "GBP", "JPY", "AUD", "CAD", "CHF", "NZD"];

        if symbol.len() == 6 {
            let base = &symbol[0..3];
            let quote = &symbol[3..6];
            return major_currencies.contains(&base) && major_currencies.contains(&quote);
        }

        false
    }
}

/// Time Series Optimizer for efficient database storage and indexing
pub struct TimeSeriesOptimizer {
    // Configuration for time-series optimization
}

impl TimeSeriesOptimizer {
    pub fn new() -> Self {
        Self {}
    }

    /// Optimize market ticks for time-series storage
    pub fn optimize_for_storage(&self, ticks: &mut Vec<MarketTick>) -> Result<()> {
        // Sort by timestamp for optimal time-series insertion
        ticks.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        // Remove exact duplicates
        ticks.dedup_by(|a, b| {
            a.timestamp == b.timestamp &&
            a.instrument_id == b.instrument_id &&
            a.provider == b.provider
        });

        // Validate time-series constraints
        self.validate_time_series_constraints(ticks)?;

        Ok(())
    }

    /// Validate time-series constraints for TimescaleDB
    fn validate_time_series_constraints(&self, ticks: &[MarketTick]) -> Result<()> {
        for tick in ticks {
            // Ensure timestamp is not too far in the future
            let now = Utc::now();
            if tick.timestamp > now + chrono::Duration::minutes(5) {
                return Err(crate::utils::errors::PantherSwapError::market_data(
                    format!("Timestamp too far in future: {}", tick.timestamp)
                ));
            }

            // Ensure timestamp is not too old (more than 1 year)
            if tick.timestamp < now - chrono::Duration::days(365) {
                return Err(crate::utils::errors::PantherSwapError::market_data(
                    format!("Timestamp too old: {}", tick.timestamp)
                ));
            }
        }

        Ok(())
    }

    /// Create time-series partitioning hints for optimal performance
    pub fn create_partitioning_hints(&self, ticks: &[MarketTick]) -> HashMap<String, serde_json::Value> {
        let mut hints = HashMap::new();

        if !ticks.is_empty() {
            let min_timestamp = ticks.iter().map(|t| t.timestamp).min().unwrap();
            let max_timestamp = ticks.iter().map(|t| t.timestamp).max().unwrap();

            hints.insert("time_range".to_string(), serde_json::json!({
                "start": min_timestamp,
                "end": max_timestamp
            }));

            let unique_instruments: std::collections::HashSet<_> =
                ticks.iter().map(|t| t.instrument_id).collect();
            hints.insert("instrument_count".to_string(), serde_json::json!(unique_instruments.len()));

            let unique_providers: std::collections::HashSet<_> =
                ticks.iter().map(|t| &t.provider).collect();
            hints.insert("provider_count".to_string(), serde_json::json!(unique_providers.len()));
        }

        hints
    }
}
