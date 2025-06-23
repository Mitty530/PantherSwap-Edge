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
#[derive(Debug, Deserialize)]
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

        let auth_url = format!("{}/session", self.config.base_url);
        
        let auth_body = serde_json::json!({
            "identifier": "", // Username would go here in production
            "password": "",   // Password would go here in production
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
            .map_err(|e| PantherSwapError::market_data(format!("IG Trading auth request failed: {}", e)))?;

        if response.status().is_success() {
            // Extract session tokens from headers
            if let Some(cst) = response.headers().get("CST") {
                if let Ok(cst_str) = cst.to_str() {
                    self.session_token = Some(cst_str.to_string());
                }
            }

            info!("✅ IG Trading authentication successful");
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("IG Trading authentication failed: {} - {}", status, error_text);
            Err(PantherSwapError::market_data(format!("IG Trading auth failed: {}", status)))
        }
    }

    /// Fetch market data for instruments
    pub async fn fetch_market_data(&mut self, instruments: &[String]) -> Result<Vec<MarketTick>> {
        if self.session_token.is_none() {
            self.authenticate().await?;
        }

        let mut market_ticks = Vec::new();

        for instrument in instruments {
            match self.fetch_instrument_data(instrument).await {
                Ok(tick) => market_ticks.push(tick),
                Err(e) => {
                    warn!("Failed to fetch data for {}: {}", instrument, e);
                    continue;
                }
            }

            // Rate limiting
            self.enforce_rate_limit().await;
        }

        info!("Fetched market data for {} instruments from IG Trading", market_ticks.len());
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
            let market_tick = MarketTick {
                timestamp: Utc::now(),
                instrument_id: Uuid::new_v4(), // In production, map epic to UUID
                provider: "ig_trading".to_string(),
                bid_price: ig_data.bid,
                ask_price: ig_data.offer,
                bid_size: 1000.0, // Default size - IG doesn't provide this in basic API
                ask_size: 1000.0,
                last_price: Some((ig_data.bid + ig_data.offer) / 2.0),
                volume: None, // Not available in basic market data
                spread: ig_data.offer - ig_data.bid,
                data_quality_score: 0.95, // IG is a reliable source
                raw_data: serde_json::to_value(&ig_data).unwrap_or_default(),
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

    /// Test connection to IG Trading API
    pub async fn test_connection(&mut self) -> Result<bool> {
        info!("Testing IG Trading API connection...");
        
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
}

impl Default for IGTradingConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
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
