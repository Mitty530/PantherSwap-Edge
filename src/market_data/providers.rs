use crate::market_data::types::{
    MarketQuote, AlphaVantageResponse, RateLimitState
};
use crate::database::Database;
use crate::utils::Result;
use reqwest::Client;
use chrono::{DateTime, Utc};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{info, warn};
use uuid::Uuid;
use serde_json::json;
use tokio::time::{sleep, interval};
use std::collections::HashMap;

/// Enhanced Alpha Vantage API provider with database persistence and real-time data collection
#[derive(Debug, Clone)]
pub struct AlphaVantageProvider {
    client: Client,
    api_key: String,
    base_url: String,
    rate_limit: Arc<Mutex<RateLimitState>>,
    database: Option<Database>,
    instruments: Arc<Mutex<HashMap<String, Uuid>>>,
    collection_enabled: bool,
}

impl AlphaVantageProvider {
    /// Create a new Alpha Vantage provider without database integration
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
            api_key,
            base_url: "https://www.alphavantage.co/query".to_string(),
            rate_limit: Arc::new(Mutex::new(RateLimitState::new(5))), // 5 requests per minute
            database: None,
            instruments: Arc::new(Mutex::new(HashMap::new())),
            collection_enabled: false,
        }
    }

    /// Create a new Alpha Vantage provider with database integration for production
    pub fn new_with_database(api_key: String, database: Database) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
            api_key,
            base_url: "https://www.alphavantage.co/query".to_string(),
            rate_limit: Arc::new(Mutex::new(RateLimitState::new(5))), // 5 requests per minute
            database: Some(database),
            instruments: Arc::new(Mutex::new(HashMap::new())),
            collection_enabled: true,
        }
    }

    /// Enable real-time data collection and persistence
    pub fn enable_collection(&mut self) {
        self.collection_enabled = true;
    }

    /// Disable real-time data collection
    pub fn disable_collection(&mut self) {
        self.collection_enabled = false;
    }

    /// Get forex quote for a currency pair with database persistence
    pub async fn get_fx_quote(&self, from_currency: &str, to_currency: &str) -> Result<MarketQuote> {
        // Check rate limiting
        self.wait_for_rate_limit().await?;

        let url = format!(
            "{}?function=CURRENCY_EXCHANGE_RATE&from_currency={}&to_currency={}&apikey={}",
            self.base_url, from_currency, to_currency, self.api_key
        );

        info!("Fetching live forex quote: {} -> {}", from_currency, to_currency);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| crate::utils::errors::PantherSwapError::market_data(
                format!("HTTP request failed: {}", e)
            ))?;

        if !response.status().is_success() {
            return Err(crate::utils::errors::PantherSwapError::market_data(
                format!("HTTP error: {}", response.status())
            ));
        }

        let alpha_response: AlphaVantageResponse = response.json().await
            .map_err(|e| crate::utils::errors::PantherSwapError::market_data(
                format!("Failed to parse JSON response: {}", e)
            ))?;

        // Record the request for rate limiting
        {
            let mut rate_limit = self.rate_limit.lock().unwrap();
            rate_limit.record_request();
        }

        // Check for API errors
        if let Some(error) = alpha_response.error_message {
            return Err(crate::utils::errors::PantherSwapError::market_data(
                format!("Alpha Vantage error: {}", error)
            ));
        }

        if let Some(note) = alpha_response.note {
            if note.contains("API call frequency") {
                warn!("Rate limit warning from Alpha Vantage: {}", note);
                return Err(crate::utils::errors::PantherSwapError::market_data(
                    format!("Rate limit exceeded: {}", note)
                ));
            }
        }

        let exchange_rate = alpha_response.realtime_currency_exchange_rate
            .ok_or_else(|| crate::utils::errors::PantherSwapError::market_data(
                "No exchange rate data in response".to_string()
            ))?;

        let market_quote = self.convert_to_market_quote(exchange_rate, from_currency, to_currency)?;

        // Store in database if collection is enabled
        if self.collection_enabled {
            if let Err(e) = self.store_market_data(&market_quote, from_currency, to_currency).await {
                warn!("Failed to store market data in database: {}", e);
            }
        }

        Ok(market_quote)
    }

    /// Wait for rate limit if necessary
    async fn wait_for_rate_limit(&self) -> Result<()> {
        let wait_duration = {
            let rate_limit = self.rate_limit.lock().unwrap();
            rate_limit.time_until_next_request()
        };

        if let Some(duration) = wait_duration {
            warn!("Rate limit reached, waiting {:?} before next request", duration);
            tokio::time::sleep(duration).await;
        }

        Ok(())
    }

    /// Convert Alpha Vantage response to MarketQuote
    fn convert_to_market_quote(
        &self,
        exchange_rate: crate::market_data::types::AlphaVantageExchangeRate,
        from_currency: &str,
        to_currency: &str,
    ) -> Result<MarketQuote> {
        let timestamp = self.parse_timestamp(&exchange_rate.last_refreshed)?;

        let bid_price = exchange_rate.bid_price.parse::<f64>()
            .map_err(|e| crate::utils::errors::PantherSwapError::market_data(
                format!("Failed to parse bid price: {}", e)
            ))?;

        let ask_price = exchange_rate.ask_price.parse::<f64>()
            .map_err(|e| crate::utils::errors::PantherSwapError::market_data(
                format!("Failed to parse ask price: {}", e)
            ))?;

        let mid_price = exchange_rate.exchange_rate.parse::<f64>()
            .map_err(|e| crate::utils::errors::PantherSwapError::market_data(
                format!("Failed to parse exchange rate: {}", e)
            ))?;

        let spread = ask_price - bid_price;
        let data_quality = self.assess_data_quality(&exchange_rate);

        Ok(MarketQuote {
            symbol: format!("{}{}", from_currency, to_currency),
            provider: "alpha_vantage".to_string(),
            timestamp,
            bid_price,
            ask_price,
            mid_price,
            bid_size: None, // Alpha Vantage doesn't provide size data
            ask_size: None,
            volume: None,
            spread: Some(spread),
            data_quality,
        })
    }

    /// Parse timestamp from Alpha Vantage format
    fn parse_timestamp(&self, timestamp_str: &str) -> Result<DateTime<Utc>> {
        // Alpha Vantage format: "2023-12-18 16:30:00"
        let parsed = chrono::NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S")
            .map_err(|e| crate::utils::errors::PantherSwapError::market_data(
                format!("Failed to parse timestamp '{}': {}", timestamp_str, e)
            ))?;

        Ok(DateTime::<Utc>::from_naive_utc_and_offset(parsed, Utc))
    }

    /// Assess data quality based on various factors
    fn assess_data_quality(&self, exchange_rate: &crate::market_data::types::AlphaVantageExchangeRate) -> f64 {
        let mut score: f64 = 1.0;

        // Check if bid/ask prices are reasonable
        if let (Ok(bid), Ok(ask)) = (
            exchange_rate.bid_price.parse::<f64>(),
            exchange_rate.ask_price.parse::<f64>()
        ) {
            let spread = ask - bid;
            let spread_pct = spread / bid * 100.0;

            // Penalize unreasonable spreads
            if spread_pct > 1.0 {  // More than 1% spread is suspicious for major pairs
                score *= 0.7;
            } else if spread_pct > 0.1 {  // More than 0.1% spread
                score *= 0.9;
            }

            // Check for zero or negative spreads
            if spread <= 0.0 {
                score *= 0.3;
            }
        } else {
            score *= 0.5; // Failed to parse prices
        }

        // Check timestamp freshness (Alpha Vantage data should be recent)
        if let Ok(timestamp) = self.parse_timestamp(&exchange_rate.last_refreshed) {
            let age_minutes = (Utc::now() - timestamp).num_minutes();
            if age_minutes > 60 {  // Data older than 1 hour
                score *= 0.6;
            } else if age_minutes > 15 {  // Data older than 15 minutes
                score *= 0.8;
            }
        } else {
            score *= 0.4; // Failed to parse timestamp
        }

        score.max(0.0).min(1.0)
    }

    /// Validate that the API key is configured
    pub fn validate_configuration(&self) -> Result<()> {
        if self.api_key.is_empty() {
            return Err(crate::utils::errors::PantherSwapError::market_data(
                "Alpha Vantage API key is not configured".to_string()
            ));
        }

        if self.api_key == "your_api_key_here" || self.api_key == "demo" {
            return Err(crate::utils::errors::PantherSwapError::market_data(
                "Alpha Vantage API key appears to be a placeholder".to_string()
            ));
        }

        Ok(())
    }

    /// Get the API key (for status checking)
    pub fn get_api_key(&self) -> &str {
        &self.api_key
    }

    /// Store market data in TimescaleDB
    async fn store_market_data(&self, quote: &MarketQuote, from_currency: &str, to_currency: &str) -> Result<()> {
        if let Some(ref database) = self.database {
            let instrument_id = self.get_or_create_instrument_id(from_currency, to_currency).await?;

            let market_tick = crate::database::types::MarketTick {
                timestamp: quote.timestamp,
                instrument_id,
                provider: quote.provider.clone(),
                bid_price: quote.bid_price,
                ask_price: quote.ask_price,
                bid_size: quote.bid_size.unwrap_or(0.0),
                ask_size: quote.ask_size.unwrap_or(0.0),
                last_price: Some(quote.mid_price),
                volume: quote.volume,
                spread: quote.ask_price - quote.bid_price,
                data_quality_score: quote.data_quality,
                raw_data: json!({
                    "symbol": quote.symbol,
                    "provider": quote.provider,
                    "spread": quote.spread,
                    "data_quality": quote.data_quality
                }),
            };

            database.insert_market_tick(&market_tick).await
                .map_err(|e| crate::utils::errors::PantherSwapError::market_data(
                    format!("Failed to store market tick: {}", e)
                ))?;

            info!("Stored market tick for {} in database", quote.symbol);
        }
        Ok(())
    }

    /// Get or create instrument ID for currency pair
    async fn get_or_create_instrument_id(&self, from_currency: &str, to_currency: &str) -> Result<Uuid> {
        let symbol = format!("{}{}", from_currency, to_currency);

        // Check cache first
        {
            let instruments = self.instruments.lock().unwrap();
            if let Some(&instrument_id) = instruments.get(&symbol) {
                return Ok(instrument_id);
            }
        }

        // Get or create in database
        if let Some(ref database) = self.database {
            let instrument_id = database.get_or_create_instrument(
                &symbol,
                &format!("{}/{} Currency Pair", from_currency, to_currency),
                "forex",
                from_currency,
                to_currency,
                0.0001, // Standard forex tick size
                1.0,    // Standard lot size
            ).await.map_err(|e| crate::utils::errors::PantherSwapError::market_data(
                format!("Failed to get/create instrument: {}", e)
            ))?;

            // Cache the result
            {
                let mut instruments = self.instruments.lock().unwrap();
                instruments.insert(symbol, instrument_id);
            }

            Ok(instrument_id)
        } else {
            Err(crate::utils::errors::PantherSwapError::market_data(
                "Database not available for instrument creation".to_string()
            ))
        }
    }

    /// Start real-time data collection for multiple currency pairs
    pub async fn start_real_time_collection(&self, currency_pairs: Vec<(String, String)>, interval_ms: u64) -> Result<()> {
        if !self.collection_enabled {
            return Err(crate::utils::errors::PantherSwapError::market_data(
                "Real-time collection is not enabled".to_string()
            ));
        }

        info!("Starting real-time data collection for {} currency pairs", currency_pairs.len());

        let provider = self.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(interval_ms));

            loop {
                interval.tick().await;

                for (from_currency, to_currency) in &currency_pairs {
                    match provider.get_fx_quote(from_currency, to_currency).await {
                        Ok(quote) => {
                            info!("Collected real-time data for {}/{}: bid={}, ask={}",
                                from_currency, to_currency, quote.bid_price, quote.ask_price);
                        }
                        Err(e) => {
                            warn!("Failed to collect data for {}/{}: {}", from_currency, to_currency, e);

                            // If rate limited, wait longer
                            if e.to_string().contains("rate limit") {
                                sleep(Duration::from_secs(60)).await;
                            }
                        }
                    }

                    // Small delay between requests to avoid overwhelming the API
                    sleep(Duration::from_millis(100)).await;
                }
            }
        });

        Ok(())
    }

    /// Get supported currency pairs
    pub fn get_supported_pairs() -> Vec<(&'static str, &'static str)> {
        vec![
            ("EUR", "USD"),
            ("GBP", "USD"),
            ("USD", "JPY"),
            ("AUD", "USD"),
            ("USD", "CAD"),
            ("USD", "CHF"),
            ("NZD", "USD"),
            ("EUR", "GBP"),
            ("EUR", "JPY"),
            ("GBP", "JPY"),
        ]
    }
}
