pub mod types;
pub mod pipeline;
pub mod ig_trading;

use crate::config::Settings;
use crate::database::Database;
use crate::market_data::ig_trading::{IGTradingClient, IGTradingConfig};
use crate::utils::Result;
use std::collections::HashMap;
use uuid::Uuid;
use tracing::{info, error};

#[derive(Clone)]
pub struct MarketDataManager {
    ig_trading_client: Option<IGTradingClient>,
    database: Option<Database>,
    live_collection_enabled: bool,
}

impl MarketDataManager {
    /// Create a new market data manager with IG Trading integration
    pub async fn new(settings: &Settings, database: Database) -> Result<Self> {
        info!("Initializing Market Data Manager with IG Trading");

        // Initialize IG Trading client
        let ig_config = IGTradingConfig {
            api_key: settings.market_data.ig_trading.api_key.clone(),
            username: settings.market_data.ig_trading.username.clone(),
            password: settings.market_data.ig_trading.password.clone(),
            security_token: settings.market_data.ig_trading.security_token.clone(),
            cst: settings.market_data.ig_trading.cst.clone(),
            version: settings.market_data.ig_trading.version.clone(),
            base_url: settings.market_data.ig_trading.base_url.clone(),
            content_type: settings.market_data.ig_trading.content_type.clone(),
            accept: settings.market_data.ig_trading.accept.clone(),
            demo_mode: settings.market_data.ig_trading.demo_mode,
            rate_limit_per_minute: settings.market_data.ig_trading.rate_limit_per_minute,
            connection_timeout_ms: settings.market_data.ig_trading.connection_timeout_ms,
            retry_attempts: settings.market_data.ig_trading.retry_attempts,
        };

        let ig_client = IGTradingClient::new(ig_config)?;

        // Validate IG Trading configuration
        if let Err(e) = ig_client.validate_configuration().await {
            error!("IG Trading client validation failed: {}", e);
            return Err(e);
        }

        // Setup default instruments
        let instruments = Self::create_default_instruments(&database).await?;
        info!("Setup {} instruments for IG Trading", instruments.len());

        Ok(Self {
            ig_trading_client: Some(ig_client),
            database: Some(database),
            live_collection_enabled: true,
        })
    }

    /// Create a new market data manager with IG Trading for live trading
    pub async fn new_with_live_api(settings: &Settings, database: Database) -> Result<Self> {
        info!("Initializing Market Data Manager with IG Trading for live trading");
        Self::new(settings, database).await
    }

    pub async fn start(&mut self) -> Result<()> {
        info!("Starting IG Trading market data collection...");

        if let Some(ref client) = self.ig_trading_client {
            // Start data collection with IG Trading
            client.start_data_collection().await?;
            info!("IG Trading data collection started successfully");
        } else {
            return Err(crate::utils::errors::PantherSwapError::internal(
                "IG Trading client not initialized".to_string()
            ));
        }

        Ok(())
    }

    /// Start live real-time data collection with IG Trading
    pub async fn start_live_collection(&self) -> Result<()> {
        if !self.live_collection_enabled {
            return Err(crate::utils::errors::PantherSwapError::market_data(
                "Live collection is not enabled for this manager".to_string()
            ));
        }

        if let Some(ref client) = self.ig_trading_client {
            info!("Starting IG Trading live real-time data collection");
            client.start_streaming().await?;
            info!("IG Trading live data collection started successfully");
        } else {
            return Err(crate::utils::errors::PantherSwapError::market_data(
                "IG Trading client not available".to_string()
            ));
        }

        Ok(())
    }

    /// Get latest live quote for a symbol using IG Trading
    pub async fn get_live_quote(&mut self, symbol: &str) -> Result<crate::market_data::types::MarketQuote> {
        if let Some(ref mut client) = self.ig_trading_client {
            client.get_market_quote(symbol).await
        } else {
            Err(crate::utils::errors::PantherSwapError::market_data(
                "IG Trading client not available".to_string()
            ))
        }
    }

    /// Get historical market data from database
    pub async fn get_historical_data(&self, symbol: &str, hours: i64) -> Result<Vec<crate::database::types::MarketTick>> {
        if let Some(ref database) = self.database {
            database.get_market_ticks_by_symbol(symbol, hours).await
                .map_err(|e| crate::utils::errors::PantherSwapError::market_data(
                    format!("Failed to get historical data: {}", e)
                ))
        } else {
            Err(crate::utils::errors::PantherSwapError::market_data(
                "Database not available".to_string()
            ))
        }
    }

    /// Load active instruments from database
    async fn load_instruments(database: &Database) -> Result<HashMap<String, Uuid>> {
        let query_manager = database.query_manager();
        let instruments = query_manager.get_active_instruments().await?;

        let mut instrument_map = HashMap::new();
        for instrument in instruments {
            instrument_map.insert(instrument.symbol.clone(), instrument.id);
        }

        // If no instruments in database, add default instruments
        if instrument_map.is_empty() {
            info!("No instruments found in database, will use default instruments");
        }

        Ok(instrument_map)
    }

    /// Get supported trading instruments for IG Trading
    pub fn get_supported_instruments() -> Vec<&'static str> {
        vec![
            "EURUSD", "GBPUSD", "USDJPY", "AUDUSD", "USDCAD", "NZDUSD", "USDCHF",
            "AAPL", "MSFT", "GOOGL", "TSLA", "NVDA", "SPY", "QQQ", "META", "AMZN", "NFLX"
        ]
    }

    /// Create default instruments for IG Trading (forex and stocks)
    pub async fn create_default_instruments(database: &Database) -> Result<HashMap<String, Uuid>> {
        let query_manager = database.query_manager();
        let mut instrument_map = HashMap::new();

        // Forex pairs
        let forex_pairs = [
            ("EURUSD", "EUR/USD", "Euro / US Dollar", "EUR", "USD"),
            ("GBPUSD", "GBP/USD", "British Pound / US Dollar", "GBP", "USD"),
            ("USDJPY", "USD/JPY", "US Dollar / Japanese Yen", "USD", "JPY"),
            ("AUDUSD", "AUD/USD", "Australian Dollar / US Dollar", "AUD", "USD"),
            ("USDCAD", "USD/CAD", "US Dollar / Canadian Dollar", "USD", "CAD"),
            ("NZDUSD", "NZD/USD", "New Zealand Dollar / US Dollar", "NZD", "USD"),
            ("USDCHF", "USD/CHF", "US Dollar / Swiss Franc", "USD", "CHF"),
        ];

        // Stock instruments
        let stocks = [
            ("AAPL", "Apple Inc.", "Technology", "USD", "USD"),
            ("MSFT", "Microsoft Corporation", "Technology", "USD", "USD"),
            ("GOOGL", "Alphabet Inc.", "Technology", "USD", "USD"),
            ("TSLA", "Tesla Inc.", "Automotive", "USD", "USD"),
            ("NVDA", "NVIDIA Corporation", "Technology", "USD", "USD"),
            ("SPY", "SPDR S&P 500 ETF", "ETF", "USD", "USD"),
            ("QQQ", "Invesco QQQ Trust", "ETF", "USD", "USD"),
            ("META", "Meta Platforms Inc.", "Technology", "USD", "USD"),
            ("AMZN", "Amazon.com Inc.", "Technology", "USD", "USD"),
            ("NFLX", "Netflix Inc.", "Technology", "USD", "USD"),
        ];

        // Create forex instruments
        for (symbol, display_symbol, name, base, quote) in forex_pairs {
            if let Ok(instrument_id) = Self::create_or_get_instrument(
                &query_manager, symbol, display_symbol, name, "forex", base, quote, 0.00001, 100000.0
            ).await {
                instrument_map.insert(symbol.to_string(), instrument_id);
            }
        }

        // Create stock instruments
        for (symbol, name, _sector, base, quote) in stocks {
            if let Ok(instrument_id) = Self::create_or_get_instrument(
                &query_manager, symbol, symbol, name, "stock", base, quote, 0.01, 1.0
            ).await {
                instrument_map.insert(symbol.to_string(), instrument_id);
            }
        }

        info!("Instrument setup completed: {} instruments available", instrument_map.len());
        Ok(instrument_map)
    }

    /// Helper method to create or get existing instrument
    async fn create_or_get_instrument(
        query_manager: &crate::database::query_functions::SimpleQueryManager,
        symbol: &str,
        display_symbol: &str,
        name: &str,
        instrument_type: &str,
        base_currency: &str,
        quote_currency: &str,
        tick_size: f64,
        lot_size: f64,
    ) -> Result<Uuid> {
        // First, try to get existing instrument
        match query_manager.get_instrument_by_symbol(display_symbol).await {
            Ok(Some(existing_instrument)) => {
                info!("Found existing instrument: {} ({})", display_symbol, existing_instrument.id);
                return Ok(existing_instrument.id);
            }
            Ok(None) => {
                info!("Creating new instrument: {}", display_symbol);
            }
            Err(e) => {
                error!("Error checking for existing instrument {}: {}", display_symbol, e);
                return Err(crate::utils::errors::PantherSwapError::market_data(
                    format!("Failed to check existing instrument: {}", e)
                ));
            }
        }

        let instrument = crate::database::types::Instrument {
            id: Uuid::new_v4(),
            symbol: display_symbol.to_string(),
            name: name.to_string(),
            instrument_type: instrument_type.to_string(),
            base_currency: base_currency.to_string(),
            quote_currency: quote_currency.to_string(),
            tick_size,
            lot_size,
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        match query_manager.insert_instrument(&instrument).await {
            Ok(instrument_id) => {
                info!("Created new instrument: {} ({})", display_symbol, instrument_id);
                Ok(instrument_id)
            }
            Err(e) => {
                if e.to_string().contains("duplicate key") {
                    info!("Instrument {} already exists, fetching existing record", display_symbol);
                    match query_manager.get_instrument_by_symbol(display_symbol).await {
                        Ok(Some(existing_instrument)) => {
                            info!("Retrieved existing instrument: {} ({})", display_symbol, existing_instrument.id);
                            Ok(existing_instrument.id)
                        }
                        Ok(None) => {
                            Err(crate::utils::errors::PantherSwapError::market_data(
                                format!("Instrument {} should exist but not found", display_symbol)
                            ))
                        }
                        Err(fetch_error) => {
                            Err(crate::utils::errors::PantherSwapError::market_data(
                                format!("Failed to fetch existing instrument {}: {}", display_symbol, fetch_error)
                            ))
                        }
                    }
                } else {
                    Err(crate::utils::errors::PantherSwapError::market_data(
                        format!("Failed to create instrument {}: {}", display_symbol, e)
                    ))
                }
            }
        }
    }

    /// Get latest quote from IG Trading
    pub async fn get_latest_quote(&mut self, symbol: &str) -> Result<crate::market_data::types::MarketQuote> {
        if let Some(ref mut client) = self.ig_trading_client {
            client.get_market_quote(symbol).await
        } else {
            Err(crate::utils::errors::PantherSwapError::market_data(
                "IG Trading client not available".to_string()
            ))
        }
    }

    /// Start live data streaming for IG Trading
    pub async fn start_live_streaming(&self, symbols: Vec<String>) -> Result<()> {
        if let Some(ref client) = self.ig_trading_client {
            info!("Starting IG Trading live streaming for {} symbols", symbols.len());
            client.start_streaming_for_symbols(symbols).await
        } else {
            Err(crate::utils::errors::PantherSwapError::market_data(
                "IG Trading client not available for streaming".to_string()
            ))
        }
    }

    /// Get multiple quotes efficiently from IG Trading
    pub async fn get_multiple_quotes(&mut self, symbols: &[String]) -> Result<std::collections::HashMap<String, crate::market_data::types::MarketQuote>> {
        if let Some(ref mut client) = self.ig_trading_client {
            client.get_multiple_quotes(symbols).await
        } else {
            Err(crate::utils::errors::PantherSwapError::market_data(
                "IG Trading client not available".to_string()
            ))
        }
    }

    /// Stop the market data manager
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping Market Data Manager");
        // In a full implementation, this would stop any background tasks,
        // close connections, and clean up resources
        Ok(())
    }

    /// Get IG Trading provider status and health information
    pub async fn get_provider_status(&self) -> Result<serde_json::Value> {
        let mut status = serde_json::json!({
            "primary_provider": "ig_trading",
            "live_collection_enabled": self.live_collection_enabled,
            "providers": {}
        });

        // Check IG Trading client status
        if let Some(ref client) = self.ig_trading_client {
            let ig_status = match client.validate_configuration().await {
                Ok(_) => serde_json::json!({
                    "available": true,
                    "authenticated": true,
                    "demo_mode": client.is_demo_mode()
                }),
                Err(e) => serde_json::json!({
                    "available": false,
                    "error": e.to_string()
                })
            };
            status["providers"]["ig_trading"] = ig_status;
        } else {
            status["providers"]["ig_trading"] = serde_json::json!({
                "available": false,
                "error": "IG Trading client not initialized"
            });
        }

        Ok(status)
    }
}
