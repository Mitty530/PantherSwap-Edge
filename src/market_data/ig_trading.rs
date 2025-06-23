// IG Trading API Integration for PantherSwap Edge
use crate::utils::{Result, PantherSwapError};
use crate::database::types::MarketTick;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use reqwest::Client;
use std::collections::HashMap;
use tracing::{info, warn, error, debug};

/// IG Trading API configuration
#[derive(Debug, Clone)]
pub struct IGTradingConfig {
    pub api_key: String,
    pub username: String,
    pub password: String,
    pub security_token: String,
    pub cst: String,
    pub version: String,
    pub base_url: String,
    pub content_type: String,
    pub accept: String,
    pub demo_mode: bool,
    pub rate_limit_per_minute: u32,
    pub connection_timeout_ms: u64,
    pub retry_attempts: u32,
}

/// IG Trading API client
#[derive(Clone)]
pub struct IGTradingClient {
    config: IGTradingConfig,
    client: Client,
    session_token: Option<String>,
    last_request_time: Option<DateTime<Utc>>,
}

/// IG Trading market data response
#[derive(Debug, Serialize, Deserialize)]
pub struct IGMarketData {
    pub epic: String,
    pub instrument_name: String,
    pub bid: f64,
    pub offer: f64,
    pub high: f64,
    pub low: f64,
    pub market_delay: i32,
    pub net_change: f64,
    pub percentage_change: f64,
    pub update_time: String,
    pub market_status: String,
}

/// IG Trading price response
#[derive(Debug, Deserialize)]
pub struct IGPriceResponse {
    pub prices: Vec<IGMarketData>,
}

/// IG Trading session response
#[derive(Debug, Deserialize)]
pub struct IGSessionResponse {
    #[serde(rename = "oauthToken")]
    pub oauth_token: Option<String>,
    #[serde(rename = "accountId")]
    pub account_id: String,
    #[serde(rename = "clientId")]
    pub client_id: String,
    #[serde(rename = "timezoneOffset")]
    pub timezone_offset: i32,
}

impl IGTradingClient {
    /// Create new IG Trading client
    pub fn new(config: IGTradingConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_millis(config.connection_timeout_ms))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            client,
            session_token: None,
            last_request_time: None,
        }
    }

    /// Authenticate with IG Trading API
    pub async fn authenticate(&mut self) -> Result<()> {
        info!("Authenticating with IG Trading API...");

        // Check if credentials are available
        if self.config.username.is_empty() || self.config.password.is_empty() {
            warn!("IG Trading credentials not provided - using demo mode without authentication");
            // For demo mode, we can skip authentication and use the provided tokens
            if self.config.demo_mode {
                self.session_token = Some(self.config.cst.clone());
                info!("✅ IG Trading demo mode - using provided CST token");
                return Ok(());
            } else {
                return Err(PantherSwapError::market_data(
                    "IG Trading credentials (username/password) are required for authentication".to_string()
                ));
            }
        }

        let auth_url = format!("{}/session", self.config.base_url);

        let auth_body = serde_json::json!({
            "identifier": self.config.username,
            "password": self.config.password,
            "encryptedPassword": false
        });

        let response = self.client
            .post(&auth_url)
            .header("Content-Type", &self.config.content_type)
            .header("Accept", &self.config.accept)
            .header("X-IG-API-KEY", &self.config.api_key)
            .header("Version", &self.config.version)
            .json(&auth_body)
            .send()
            .await
            .map_err(|e| {
                error!("IG Trading authentication request failed: {}", e);
                PantherSwapError::market_data(format!("IG Trading auth request failed: {}", e))
            })?;

        if response.status().is_success() {
            // Extract session tokens from headers
            if let Some(cst) = response.headers().get("CST") {
                if let Ok(cst_str) = cst.to_str() {
                    self.session_token = Some(cst_str.to_string());
                    info!("✅ IG Trading authentication successful - CST token acquired");
                } else {
                    warn!("Failed to parse CST token from response headers");
                }
            } else {
                warn!("No CST token found in authentication response headers");
            }

            // Also try to extract X-SECURITY-TOKEN if available
            if let Some(security_token) = response.headers().get("X-SECURITY-TOKEN") {
                if let Ok(token_str) = security_token.to_str() {
                    debug!("Security token updated from authentication response");
                }
            }

            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            // Provide more specific error messages based on status code
            let error_message = match status.as_u16() {
                401 => "Authentication failed - Invalid credentials or API key".to_string(),
                403 => "Access forbidden - Check API permissions".to_string(),
                429 => "Rate limit exceeded - Too many authentication attempts".to_string(),
                500..=599 => "IG Trading server error - Try again later".to_string(),
                _ => format!("Authentication failed with status {}: {}", status, error_text),
            };

            error!("IG Trading authentication failed: {}", error_message);
            Err(PantherSwapError::market_data(error_message))
        }
    }

    /// Fetch market data for instruments with retry logic
    pub async fn fetch_market_data(&mut self, instruments: &[String]) -> Result<Vec<MarketTick>> {
        if self.session_token.is_none() {
            self.authenticate().await?;
        }

        let mut market_ticks = Vec::new();
        let mut failed_instruments = Vec::new();

        for instrument in instruments {
            let mut retry_count = 0;
            let max_retries = self.config.retry_attempts;

            loop {
                match self.fetch_instrument_data(instrument).await {
                    Ok(tick) => {
                        market_ticks.push(tick);
                        break;
                    }
                    Err(e) => {
                        retry_count += 1;

                        if retry_count <= max_retries {
                            warn!("Failed to fetch data for {} (attempt {}/{}): {}",
                                  instrument, retry_count, max_retries, e);

                            // Check if it's an authentication error
                            if e.to_string().contains("401") || e.to_string().contains("Unauthorized") {
                                info!("Authentication error detected, re-authenticating...");
                                if let Err(auth_err) = self.authenticate().await {
                                    error!("Re-authentication failed: {}", auth_err);
                                    failed_instruments.push(instrument.clone());
                                    break;
                                }
                            }

                            // Exponential backoff
                            let delay = std::time::Duration::from_millis(100 * (2_u64.pow(retry_count - 1)));
                            tokio::time::sleep(delay).await;
                        } else {
                            error!("Failed to fetch data for {} after {} attempts: {}",
                                   instrument, max_retries, e);
                            failed_instruments.push(instrument.clone());
                            break;
                        }
                    }
                }
            }

            // Rate limiting
            self.enforce_rate_limit().await;
        }

        if !failed_instruments.is_empty() {
            warn!("Failed to fetch data for {} instruments: {:?}",
                  failed_instruments.len(), failed_instruments);
        }

        info!("Fetched market data for {} instruments from IG Trading ({} failed)",
              market_ticks.len(), failed_instruments.len());
        Ok(market_ticks)
    }

    /// Fetch data for a specific instrument
    async fn fetch_instrument_data(&self, instrument: &str) -> Result<MarketTick> {
        let url = format!("{}/markets/{}", self.config.base_url, instrument);

        let mut request = self.client
            .get(&url)
            .header("Content-Type", &self.config.content_type)
            .header("Accept", &self.config.accept)
            .header("X-IG-API-KEY", &self.config.api_key)
            .header("Version", &self.config.version);

        // Add session headers if available
        if let Some(ref token) = self.session_token {
            request = request.header("CST", token);
            request = request.header("X-SECURITY-TOKEN", &self.config.security_token);
        }

        let response = request
            .send()
            .await
            .map_err(|e| PantherSwapError::market_data(format!("IG Trading request failed: {}", e)))?;

        if response.status().is_success() {
            let ig_data: IGMarketData = response
                .json()
                .await
                .map_err(|e| PantherSwapError::market_data(format!("Failed to parse IG Trading response: {}", e)))?;

            // Convert IG data to MarketTick
            let mid_price = (ig_data.bid + ig_data.offer) / 2.0;
            let market_tick = MarketTick {
                timestamp: Utc::now(),
                instrument_id: Uuid::new_v4(), // In production, map epic to UUID
                provider: "ig_trading".to_string(),
                bid_price: ig_data.bid,
                ask_price: ig_data.offer,
                bid_size: 1000.0, // Default size - IG doesn't provide this in basic API
                ask_size: 1000.0,
                last_price: Some(mid_price),
                volume: None, // Not available in basic market data
                spread: ig_data.offer - ig_data.bid,
                data_quality_score: 0.95, // IG is a reliable source
                raw_data: serde_json::to_value(&ig_data).unwrap_or_default(),
                // Backward compatibility fields
                symbol: Some(ig_data.epic.clone()),
                price: Some(mid_price),
                bid: Some(ig_data.bid),
                ask: Some(ig_data.offer),
            };

            debug!("Fetched IG Trading data for {}: bid={}, offer={}", instrument, ig_data.bid, ig_data.offer);
            Ok(market_tick)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            Err(PantherSwapError::market_data(format!("IG Trading API error {}: {}", status, error_text)))
        }
    }

    /// Enforce rate limiting
    async fn enforce_rate_limit(&mut self) {
        if let Some(last_time) = self.last_request_time {
            let min_interval = 60.0 / self.config.rate_limit_per_minute as f64;
            let elapsed = Utc::now().signed_duration_since(last_time).num_milliseconds() as f64 / 1000.0;
            
            if elapsed < min_interval {
                let sleep_duration = ((min_interval - elapsed) * 1000.0) as u64;
                tokio::time::sleep(std::time::Duration::from_millis(sleep_duration)).await;
            }
        }
        
        self.last_request_time = Some(Utc::now());
    }

    /// Validate IG Trading configuration
    pub async fn validate_configuration(&self) -> Result<()> {
        info!("Validating IG Trading configuration...");

        // Check required fields
        if self.config.api_key.is_empty() {
            return Err(PantherSwapError::market_data(
                "IG Trading API key is required".to_string()
            ));
        }

        if !self.config.demo_mode && (self.config.username.is_empty() || self.config.password.is_empty()) {
            return Err(PantherSwapError::market_data(
                "IG Trading username and password are required for production mode".to_string()
            ));
        }

        if self.config.demo_mode && self.config.cst.is_empty() {
            warn!("Demo mode enabled but no CST token provided - authentication may be required");
        }

        // Validate URL format
        if !self.config.base_url.starts_with("https://") {
            return Err(PantherSwapError::market_data(
                "IG Trading base URL must use HTTPS".to_string()
            ));
        }

        // Validate rate limiting
        if self.config.rate_limit_per_minute == 0 {
            return Err(PantherSwapError::market_data(
                "Rate limit must be greater than 0".to_string()
            ));
        }

        info!("✅ IG Trading configuration validation passed");
        Ok(())
    }

    /// Test connection to IG Trading API
    pub async fn test_connection(&mut self) -> Result<bool> {
        info!("Testing IG Trading API connection...");

        // First validate configuration
        if let Err(e) = self.validate_configuration().await {
            error!("Configuration validation failed: {}", e);
            return Ok(false);
        }

        match self.authenticate().await {
            Ok(_) => {
                info!("✅ IG Trading API connection test successful");
                Ok(true)
            }
            Err(e) => {
                error!("❌ IG Trading API connection test failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Get account information
    pub async fn get_account_info(&self) -> Result<serde_json::Value> {
        let url = format!("{}/accounts", self.config.base_url);

        let mut request = self.client
            .get(&url)
            .header("Content-Type", &self.config.content_type)
            .header("Accept", &self.config.accept)
            .header("X-IG-API-KEY", &self.config.api_key)
            .header("Version", &self.config.version);

        if let Some(ref token) = self.session_token {
            request = request.header("CST", token);
            request = request.header("X-SECURITY-TOKEN", &self.config.security_token);
        }

        let response = request
            .send()
            .await
            .map_err(|e| PantherSwapError::market_data(format!("IG Trading account request failed: {}", e)))?;

        if response.status().is_success() {
            let account_data: serde_json::Value = response
                .json()
                .await
                .map_err(|e| PantherSwapError::market_data(format!("Failed to parse account response: {}", e)))?;

            Ok(account_data)
        } else {
            let status = response.status();
            Err(PantherSwapError::market_data(format!("Failed to get account info: {}", status)))
        }
    }

    /// Get market quote for a specific symbol
    pub async fn get_market_quote(&mut self, symbol: &str) -> Result<crate::market_data::types::MarketQuote> {
        let market_ticks = self.fetch_market_data(&vec![symbol.to_string()]).await?;

        if let Some(tick) = market_ticks.first() {
            let mid_price = (tick.bid_price + tick.ask_price) / 2.0;
            let spread = tick.ask_price - tick.bid_price;

            Ok(crate::market_data::types::MarketQuote {
                symbol: symbol.to_string(),
                provider: "ig_trading".to_string(),
                timestamp: tick.timestamp,
                bid_price: tick.bid_price,
                ask_price: tick.ask_price,
                mid_price,
                bid_size: None, // IG Trading doesn't provide size in basic quotes
                ask_size: None,
                volume: tick.volume,
                spread: Some(spread),
                data_quality: 0.9, // Default quality score for IG Trading
            })
        } else {
            Err(PantherSwapError::market_data(
                format!("No market data available for symbol: {}", symbol)
            ))
        }
    }

    /// Get multiple quotes efficiently
    pub async fn get_multiple_quotes(&mut self, symbols: &[String]) -> Result<std::collections::HashMap<String, crate::market_data::types::MarketQuote>> {
        let market_ticks = self.fetch_market_data(symbols).await?;
        let mut quotes = std::collections::HashMap::new();

        for (i, tick) in market_ticks.iter().enumerate() {
            let symbol = if i < symbols.len() {
                symbols[i].clone()
            } else {
                format!("SYMBOL_{}", tick.instrument_id)
            };

            let mid_price = (tick.bid_price + tick.ask_price) / 2.0;
            let spread = tick.ask_price - tick.bid_price;

            let quote = crate::market_data::types::MarketQuote {
                symbol: symbol.clone(),
                provider: "ig_trading".to_string(),
                timestamp: tick.timestamp,
                bid_price: tick.bid_price,
                ask_price: tick.ask_price,
                mid_price,
                bid_size: None,
                ask_size: None,
                volume: tick.volume,
                spread: Some(spread),
                data_quality: 0.9,
            };
            quotes.insert(symbol, quote);
        }

        Ok(quotes)
    }

    /// Start streaming for specific symbols (placeholder implementation)
    pub async fn start_streaming(&self) -> Result<()> {
        info!("Starting IG Trading streaming...");
        // This would implement WebSocket streaming in a full implementation
        warn!("IG Trading streaming not yet implemented - using polling instead");
        Ok(())
    }

    /// Start streaming for specific symbols
    pub async fn start_streaming_for_symbols(&self, symbols: Vec<String>) -> Result<()> {
        info!("Starting IG Trading streaming for {} symbols", symbols.len());
        // This would implement WebSocket streaming for specific symbols
        warn!("IG Trading symbol-specific streaming not yet implemented");
        Ok(())
    }

    /// Check if client is in demo mode
    pub fn is_demo_mode(&self) -> bool {
        self.config.demo_mode
    }
}

impl Default for IGTradingConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            username: String::new(),
            password: String::new(),
            security_token: String::new(),
            cst: String::new(),
            version: "2".to_string(),
            base_url: "https://demo-api.ig.com/gateway/deal".to_string(),
            content_type: "application/json; charset=UTF-8".to_string(),
            accept: "application/json; charset=UTF-8".to_string(),
            demo_mode: true,
            rate_limit_per_minute: 100,
            connection_timeout_ms: 5000,
            retry_attempts: 3,
        }
    }
}
