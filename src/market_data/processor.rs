use crate::database::{Database, types::MarketTick};
use crate::market_data::providers::AlphaVantageProvider;
use crate::market_data::types::MarketQuote;
use crate::market_data::pipeline::{MarketDataPipeline, PipelineConfig};
use crate::utils::Result;
use tokio::time::{interval, Duration};
use uuid::Uuid;
use std::collections::HashMap;
use chrono::Utc;
use tracing::{info, warn, error};

/// Data processor for real-time market data collection and storage
#[derive(Clone)]
pub struct DataProcessor {
    database: Database,
    alpha_vantage: AlphaVantageProvider,
    instruments: HashMap<String, Uuid>,
    last_update: HashMap<String, chrono::DateTime<Utc>>,
    update_interval: Duration,
    pipeline: MarketDataPipeline,
}

impl DataProcessor {
    pub fn new(
        database: Database,
        alpha_vantage: AlphaVantageProvider,
        instruments: HashMap<String, Uuid>,
        update_interval_ms: u64,
    ) -> Self {
        let pipeline_config = PipelineConfig::default();
        let pipeline = MarketDataPipeline::new(database.clone(), pipeline_config);

        Self {
            database,
            alpha_vantage,
            instruments,
            last_update: HashMap::new(),
            update_interval: Duration::from_millis(update_interval_ms),
            pipeline,
        }
    }

    /// Start the data collection loop
    pub async fn start_data_collection(&mut self) -> Result<()> {
        info!("Starting market data collection with {} instruments", self.instruments.len());

        // Validate Alpha Vantage configuration
        self.alpha_vantage.validate_configuration()?;

        let mut interval = interval(self.update_interval);

        loop {
            interval.tick().await;

            // Clone the instruments to avoid borrowing issues
            let instruments: Vec<(String, Uuid)> = self.instruments.iter()
                .map(|(k, &v)| (k.clone(), v))
                .collect();

            for (symbol, instrument_id) in instruments {
                if let Err(e) = self.collect_instrument_data(&symbol, instrument_id).await {
                    error!("Error collecting data for {}: {}", symbol, e);
                }

                // Rate limiting: wait between requests to respect Alpha Vantage limits
                tokio::time::sleep(Duration::from_secs(12)).await; // 5 requests per minute max
            }
        }
    }

    /// Collect data for a specific instrument
    async fn collect_instrument_data(&mut self, symbol: &str, instrument_id: Uuid) -> Result<()> {
        // Parse currency pair (e.g., "EURUSD" -> ("EUR", "USD"))
        let (from_currency, to_currency) = self.parse_currency_pair(symbol)?;

        info!("Collecting data for {} ({} -> {})", symbol, from_currency, to_currency);

        // Fetch quote from Alpha Vantage
        let quote = self.alpha_vantage.get_fx_quote(&from_currency, &to_currency).await?;

        // Check if this is new data
        if self.is_new_data(&quote)? {
            // Process through the enhanced pipeline
            match self.pipeline.process_quote(quote, instrument_id).await {
                Ok(processed) => {
                    if processed {
                        // Update last seen timestamp
                        self.last_update.insert(symbol.to_string(), Utc::now());

                        info!(
                            "Successfully processed market data for {} through pipeline",
                            symbol
                        );
                    } else {
                        info!("Market data for {} was filtered out by pipeline", symbol);
                    }
                }
                Err(e) => {
                    error!("Pipeline processing failed for {}: {}", symbol, e);
                }
            }
        } else {
            info!("No new data for {}, skipping", symbol);
        }

        Ok(())
    }

    /// Parse currency pair from symbol (e.g., "EURUSD" -> ("EUR", "USD"))
    fn parse_currency_pair(&self, symbol: &str) -> Result<(String, String)> {
        if symbol.len() != 6 {
            return Err(crate::utils::errors::PantherSwapError::market_data(
                format!("Invalid currency pair format: {}", symbol)
            ));
        }

        let from_currency = symbol[0..3].to_uppercase();
        let to_currency = symbol[3..6].to_uppercase();

        Ok((from_currency, to_currency))
    }

    /// Check if the quote contains new data
    fn is_new_data(&self, quote: &MarketQuote) -> Result<bool> {
        if let Some(last_timestamp) = self.last_update.get(&quote.symbol) {
            Ok(quote.timestamp > *last_timestamp)
        } else {
            Ok(true) // First time seeing this symbol
        }
    }

    /// Convert MarketQuote to MarketTick for database storage
    fn convert_to_market_tick(&self, quote: MarketQuote, instrument_id: Uuid) -> Result<MarketTick> {
        let spread = quote.spread.unwrap_or_else(|| quote.ask_price - quote.bid_price);

        // Clone values to avoid move issues
        let provider = quote.provider.clone();
        let symbol = quote.symbol.clone();
        let mid_price = quote.mid_price;
        let timestamp = quote.timestamp;

        Ok(MarketTick {
            timestamp: quote.timestamp,
            instrument_id,
            provider: quote.provider,
            bid_price: quote.bid_price,
            ask_price: quote.ask_price,
            bid_size: quote.bid_size.unwrap_or(0.0),
            ask_size: quote.ask_size.unwrap_or(0.0),
            last_price: Some(quote.mid_price),
            volume: quote.volume,
            spread,
            data_quality_score: quote.data_quality,
            raw_data: serde_json::json!({
                "provider": provider,
                "symbol": symbol,
                "mid_price": mid_price,
                "timestamp": timestamp.to_rfc3339(),
            }),
            symbol: Some(symbol),
            price: Some(quote.mid_price),
            bid: Some(quote.bid_price),
            ask: Some(quote.ask_price),
        })
    }

    /// Validate market tick data quality
    fn validate_market_tick(&self, tick: &MarketTick) -> Result<bool> {
        // Check data quality score threshold
        if tick.data_quality_score < 0.7 {
            warn!("Data quality score too low: {}", tick.data_quality_score);
            return Ok(false);
        }

        // Check for reasonable prices (basic sanity checks)
        if tick.bid_price <= 0.0 || tick.ask_price <= 0.0 {
            warn!("Invalid prices: bid={}, ask={}", tick.bid_price, tick.ask_price);
            return Ok(false);
        }

        // Check spread is positive
        if tick.spread < 0.0 {
            warn!("Negative spread: {}", tick.spread);
            return Ok(false);
        }

        // Check spread is reasonable (less than 1% for major pairs)
        let spread_pct = (tick.spread / tick.bid_price) * 100.0;
        if spread_pct > 1.0 {
            warn!("Spread too wide: {:.4}%", spread_pct);
            return Ok(false);
        }

        // Check timestamp (shouldn't be too old or in future)
        let now = Utc::now();
        let age_minutes = (now - tick.timestamp).num_minutes();

        if age_minutes > 30 || age_minutes < -1 {
            warn!("Timestamp out of range: {} minutes old", age_minutes);
            return Ok(false);
        }

        Ok(true)
    }

    /// Get statistics about data collection
    pub fn get_stats(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();

        stats.insert("instruments_count".to_string(),
                    serde_json::Value::Number(self.instruments.len().into()));

        stats.insert("last_updates".to_string(),
                    serde_json::to_value(&self.last_update).unwrap_or_default());

        stats.insert("update_interval_ms".to_string(),
                    serde_json::Value::Number((self.update_interval.as_millis() as u64).into()));

        stats
    }

    /// Add a new instrument to track
    pub fn add_instrument(&mut self, symbol: String, instrument_id: Uuid) {
        info!("Adding instrument to data collection: {} ({})", symbol, instrument_id);
        self.instruments.insert(symbol, instrument_id);
    }

    /// Remove an instrument from tracking
    pub fn remove_instrument(&mut self, symbol: &str) {
        info!("Removing instrument from data collection: {}", symbol);
        self.instruments.remove(symbol);
        self.last_update.remove(symbol);
    }

    /// Get pipeline metrics
    pub fn get_pipeline_metrics(&self) -> &crate::market_data::pipeline::PipelineMetrics {
        self.pipeline.get_metrics()
    }

    /// Flush pipeline buffer manually
    pub async fn flush_pipeline(&mut self) -> Result<()> {
        self.pipeline.flush_buffer().await
    }

    /// Update pipeline configuration
    pub fn update_pipeline_config(&mut self, config: PipelineConfig) {
        self.pipeline.update_config(config);
    }

    /// Process multiple quotes in batch through the pipeline
    pub async fn process_batch_quotes(&mut self, quotes: Vec<(MarketQuote, Uuid)>) -> Result<usize> {
        self.pipeline.process_batch(quotes).await
    }

    /// Get comprehensive processor statistics
    pub fn get_comprehensive_stats(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = self.get_stats();

        // Add pipeline metrics
        let pipeline_metrics = self.pipeline.get_metrics();
        stats.insert("pipeline_total_processed".to_string(),
                    serde_json::Value::Number(pipeline_metrics.total_processed.into()));
        stats.insert("pipeline_total_filtered".to_string(),
                    serde_json::Value::Number(pipeline_metrics.total_filtered.into()));
        stats.insert("pipeline_total_stored".to_string(),
                    serde_json::Value::Number(pipeline_metrics.total_stored.into()));
        stats.insert("pipeline_average_quality".to_string(),
                    serde_json::Value::Number(serde_json::Number::from_f64(pipeline_metrics.average_quality_score).unwrap_or_else(|| serde_json::Number::from(0))));
        stats.insert("pipeline_processing_latency_ms".to_string(),
                    serde_json::Value::Number(serde_json::Number::from_f64(pipeline_metrics.processing_latency_ms).unwrap_or_else(|| serde_json::Number::from(0))));
        stats.insert("pipeline_throughput_per_second".to_string(),
                    serde_json::Value::Number(serde_json::Number::from_f64(pipeline_metrics.throughput_per_second).unwrap_or_else(|| serde_json::Number::from(0))));

        stats
    }
}
