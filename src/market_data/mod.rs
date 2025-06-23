pub mod providers;
pub mod processor;
pub mod types;
pub mod pipeline;
pub mod alpaca;
pub mod ig_trading;

use crate::config::Settings;
use crate::database::Database;
use crate::market_data::providers::AlphaVantageProvider;
use crate::market_data::alpaca::AlpacaProvider;
use crate::market_data::ig_trading::{IGTradingClient, IGTradingConfig};
use crate::market_data::processor::DataProcessor;
use crate::utils::Result;
use std::collections::HashMap;
use uuid::Uuid;
use tracing::{info, error};

#[derive(Clone)]
pub struct MarketDataManager {
    processor: Option<DataProcessor>,
    alpha_vantage_provider: Option<AlphaVantageProvider>,
    alpaca_provider: Option<AlpacaProvider>,
    ig_trading_client: Option<IGTradingClient>,
    database: Option<Database>,
    live_collection_enabled: bool,
    primary_provider: String,
}

impl MarketDataManager {
    /// Create a new market data manager with enhanced provider support
    pub async fn new(settings: &Settings, database: Database) -> Result<Self> {
        info!("Initializing Market Data Manager with primary provider: {}",
            settings.market_data.primary_provider);

        let mut manager = Self {
            processor: None,
            alpha_vantage_provider: None,
            alpaca_provider: None,
            ig_trading_client: None,
            database: Some(database.clone()),
            live_collection_enabled: true,
            primary_provider: settings.market_data.primary_provider.clone(),
        };

        // Initialize providers based on configuration
        if settings.market_data.providers.contains(&"alpaca".to_string()) {
            info!("Initializing Alpaca provider");
            let alpaca_provider = AlpacaProvider::new(settings.market_data.alpaca.clone())?
                .with_database(database.clone());

            // Validate Alpaca configuration
            if let Err(e) = alpaca_provider.validate_configuration().await {
                error!("Alpaca provider validation failed: {}", e);
                if manager.primary_provider == "alpaca" {
                    return Err(e);
                }
            } else {
                manager.alpaca_provider = Some(alpaca_provider);
                info!("✅ Alpaca provider initialized successfully");
            }
        }

        if settings.market_data.providers.contains(&"alpha_vantage".to_string()) {
            info!("Initializing Alpha Vantage provider");
            let alpha_vantage = AlphaVantageProvider::new_with_database(
                settings.market_data.alpha_vantage_api_key.clone(),
                database.clone()
            );

            if let Err(e) = alpha_vantage.validate_configuration() {
                error!("Alpha Vantage provider validation failed: {}", e);
                if manager.primary_provider == "alpha_vantage" {
                    return Err(e);
                }
            } else {
                manager.alpha_vantage_provider = Some(alpha_vantage);
                info!("✅ Alpha Vantage provider initialized successfully");
            }
        }

        // Ensure primary provider is available
        manager.validate_primary_provider()?;

        // Setup instruments based on primary provider
        let instruments = if manager.primary_provider == "alpaca" {
            Self::create_default_stock_instruments(&database).await?
        } else {
            Self::create_default_instruments(&database).await?
        };

        info!("Setup {} instruments for {} provider",
            instruments.len(), manager.primary_provider);

        Ok(manager)
    }

    /// Create a new market data manager with Alpaca integration
    pub async fn new_with_alpaca(settings: &Settings, database: Database) -> Result<Self> {
        info!("Initializing Market Data Manager with Alpaca integration");

        // Initialize Alpaca provider
        let alpaca_provider = AlpacaProvider::new(settings.market_data.alpaca.clone())?
            .with_database(database.clone());

        // Validate Alpaca configuration
        alpaca_provider.validate_configuration().await?;

        // Create default stock instruments for Alpaca
        let instruments = Self::create_default_stock_instruments(&database).await?;
        info!("Setup {} stock instruments for Alpaca data collection", instruments.len());

        Ok(Self {
            processor: None, // Will be created when needed
            alpha_vantage_provider: None,
            alpaca_provider: Some(alpaca_provider),
            ig_trading_client: None,
            database: Some(database),
            live_collection_enabled: true,
            primary_provider: "alpaca".to_string(),
        })
    }

    /// Create a new market data manager with live API integration
    pub async fn new_with_live_api(settings: &Settings, database: Database) -> Result<Self> {
        info!("Initializing Market Data Manager with live API integration");

        // Initialize Alpha Vantage provider with database integration
        let alpha_vantage = AlphaVantageProvider::new_with_database(
            settings.market_data.alpha_vantage_api_key.clone(),
            database.clone()
        );

        // Validate configuration
        alpha_vantage.validate_configuration()?;

        // Create default instruments if needed
        let instruments = Self::create_default_instruments(&database).await?;
        info!("Setup {} instruments for live data collection", instruments.len());

        // Create data processor
        let processor = DataProcessor::new(
            database.clone(),
            alpha_vantage.clone(),
            instruments,
            settings.market_data.update_interval_ms,
        );

        Ok(Self {
            processor: Some(processor),
            alpha_vantage_provider: Some(alpha_vantage),
            alpaca_provider: None,
            ig_trading_client: None,
            database: Some(database),
            live_collection_enabled: true,
            primary_provider: "alpha_vantage".to_string(),
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        info!("Starting market data collection...");

        if let Some(mut processor) = self.processor.take() {
            // Start data collection in a loop
            processor.start_data_collection().await?;
        } else {
            return Err(crate::utils::errors::PantherSwapError::internal(
                "Data processor not initialized".to_string()
            ));
        }

        Ok(())
    }

    /// Start live real-time data collection
    pub async fn start_live_collection(&self) -> Result<()> {
        if !self.live_collection_enabled {
            return Err(crate::utils::errors::PantherSwapError::market_data(
                "Live collection is not enabled for this manager".to_string()
            ));
        }

        if let Some(ref provider) = self.alpha_vantage_provider {
            info!("Starting live real-time data collection");

            // Get supported currency pairs
            let currency_pairs = AlphaVantageProvider::get_supported_pairs()
                .into_iter()
                .map(|(from, to)| (from.to_string(), to.to_string()))
                .collect();

            provider.start_real_time_collection(currency_pairs, 60000).await?; // 1 minute interval
            info!("Live data collection started successfully");
        } else {
            return Err(crate::utils::errors::PantherSwapError::market_data(
                "Alpha Vantage provider not available".to_string()
            ));
        }

        Ok(())
    }

    /// Get latest live quote for a currency pair
    pub async fn get_live_quote(&self, from_currency: &str, to_currency: &str) -> Result<crate::market_data::types::MarketQuote> {
        if let Some(ref provider) = self.alpha_vantage_provider {
            provider.get_fx_quote(from_currency, to_currency).await
        } else {
            Err(crate::utils::errors::PantherSwapError::market_data(
                "Alpha Vantage provider not available".to_string()
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
            // Convert symbol to the format expected by Alpha Vantage (e.g., "EUR/USD" -> "EURUSD")
            let symbol = instrument.symbol.replace("/", "").replace("-", "");
            instrument_map.insert(symbol, instrument.id);
        }

        // If no instruments in database, add default forex pairs
        if instrument_map.is_empty() {
            info!("No instruments found in database, will use default forex pairs");
            // Note: In a real implementation, you'd want to create these instruments in the database
            // For now, we'll just log this and return empty map
        }

        Ok(instrument_map)
    }

    /// Get supported currency pairs from Alpha Vantage
    pub fn get_supported_pairs() -> Vec<(&'static str, &'static str)> {
        AlphaVantageProvider::get_supported_pairs()
    }

    /// Create default forex instruments in database (or get existing ones)
    pub async fn create_default_instruments(database: &Database) -> Result<HashMap<String, Uuid>> {
        let query_manager = database.query_manager();
        let mut instrument_map = HashMap::new();

        let default_pairs = [
            ("EURUSD", "EUR/USD", "Euro / US Dollar"),
            ("GBPUSD", "GBP/USD", "British Pound / US Dollar"),
            ("USDJPY", "USD/JPY", "US Dollar / Japanese Yen"),
            ("AUDUSD", "AUD/USD", "Australian Dollar / US Dollar"),
            ("USDCAD", "USD/CAD", "US Dollar / Canadian Dollar"),
        ];

        for (symbol, display_symbol, name) in default_pairs {
            // First, try to get existing instrument
            match query_manager.get_instrument_by_symbol(display_symbol).await {
                Ok(Some(existing_instrument)) => {
                    info!("Found existing instrument: {} ({})", display_symbol, existing_instrument.id);
                    instrument_map.insert(symbol.to_string(), existing_instrument.id);
                    continue;
                }
                Ok(None) => {
                    // Instrument doesn't exist, create it
                    info!("Creating new instrument: {}", display_symbol);
                }
                Err(e) => {
                    error!("Error checking for existing instrument {}: {}", display_symbol, e);
                    continue;
                }
            }

            let (base_currency, quote_currency) = if symbol.len() == 6 {
                (symbol[0..3].to_string(), symbol[3..6].to_string())
            } else {
                continue;
            };

            let instrument = crate::database::types::Instrument {
                id: Uuid::new_v4(), // Will be overridden by database
                symbol: display_symbol.to_string(),
                name: name.to_string(),
                instrument_type: "forex".to_string(),
                base_currency,
                quote_currency,
                tick_size: 0.00001, // 1 pip for most forex pairs
                lot_size: 100000.0, // Standard lot size
                is_active: true,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };

            match query_manager.insert_instrument(&instrument).await {
                Ok(instrument_id) => {
                    info!("Created new instrument: {} ({})", display_symbol, instrument_id);
                    instrument_map.insert(symbol.to_string(), instrument_id);
                }
                Err(e) => {
                    // If it's a duplicate key error, try to get the existing instrument
                    if e.to_string().contains("duplicate key") {
                        info!("Instrument {} already exists, fetching existing record", display_symbol);
                        match query_manager.get_instrument_by_symbol(display_symbol).await {
                            Ok(Some(existing_instrument)) => {
                                info!("Retrieved existing instrument: {} ({})", display_symbol, existing_instrument.id);
                                instrument_map.insert(symbol.to_string(), existing_instrument.id);
                            }
                            Ok(None) => {
                                error!("Instrument {} should exist but not found", display_symbol);
                            }
                            Err(fetch_error) => {
                                error!("Failed to fetch existing instrument {}: {}", display_symbol, fetch_error);
                            }
                        }
                    } else {
                        error!("Failed to create instrument {}: {}", display_symbol, e);
                    }
                }
            }
        }

        info!("Instrument setup completed: {} instruments available", instrument_map.len());
        Ok(instrument_map)
    }

    /// Create default stock instruments for Alpaca trading
    pub async fn create_default_stock_instruments(database: &Database) -> Result<HashMap<String, Uuid>> {
        let query_manager = database.query_manager();
        let mut instrument_map = HashMap::new();

        let default_stocks = [
            ("AAPL", "Apple Inc.", "Technology"),
            ("MSFT", "Microsoft Corporation", "Technology"),
            ("GOOGL", "Alphabet Inc.", "Technology"),
            ("TSLA", "Tesla Inc.", "Automotive"),
            ("NVDA", "NVIDIA Corporation", "Technology"),
            ("SPY", "SPDR S&P 500 ETF", "ETF"),
            ("QQQ", "Invesco QQQ Trust", "ETF"),
            ("META", "Meta Platforms Inc.", "Technology"),
            ("AMZN", "Amazon.com Inc.", "Technology"),
            ("NFLX", "Netflix Inc.", "Technology"),
        ];

        for (symbol, name, sector) in default_stocks {
            // First, try to get existing instrument
            match query_manager.get_instrument_by_symbol(symbol).await {
                Ok(Some(existing_instrument)) => {
                    info!("Found existing stock instrument: {} ({})", symbol, existing_instrument.id);
                    instrument_map.insert(symbol.to_string(), existing_instrument.id);
                    continue;
                }
                Ok(None) => {
                    info!("Creating new stock instrument: {}", symbol);
                }
                Err(e) => {
                    error!("Error checking for existing stock instrument {}: {}", symbol, e);
                    continue;
                }
            }

            let instrument = crate::database::types::Instrument {
                id: Uuid::new_v4(),
                symbol: symbol.to_string(),
                name: name.to_string(),
                instrument_type: "stock".to_string(),
                base_currency: "USD".to_string(),
                quote_currency: "USD".to_string(),
                tick_size: 0.01, // 1 cent for stocks
                lot_size: 1.0,   // 1 share minimum
                is_active: true,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };

            match query_manager.insert_instrument(&instrument).await {
                Ok(instrument_id) => {
                    info!("Created new stock instrument: {} ({})", symbol, instrument_id);
                    instrument_map.insert(symbol.to_string(), instrument_id);
                }
                Err(e) => {
                    if e.to_string().contains("duplicate key") {
                        info!("Stock instrument {} already exists, fetching existing record", symbol);
                        match query_manager.get_instrument_by_symbol(symbol).await {
                            Ok(Some(existing_instrument)) => {
                                info!("Retrieved existing stock instrument: {} ({})", symbol, existing_instrument.id);
                                instrument_map.insert(symbol.to_string(), existing_instrument.id);
                            }
                            Ok(None) => {
                                error!("Stock instrument {} should exist but not found", symbol);
                            }
                            Err(fetch_error) => {
                                error!("Failed to fetch existing stock instrument {}: {}", symbol, fetch_error);
                            }
                        }
                    } else {
                        error!("Failed to create stock instrument {}: {}", symbol, e);
                    }
                }
            }
        }

        info!("Stock instrument setup completed: {} instruments available", instrument_map.len());
        Ok(instrument_map)
    }

    /// Validate that the primary provider is available
    fn validate_primary_provider(&self) -> Result<()> {
        match self.primary_provider.as_str() {
            "alpaca" => {
                if self.alpaca_provider.is_none() {
                    return Err(crate::utils::errors::PantherSwapError::market_data(
                        "Primary provider 'alpaca' is not available".to_string()
                    ));
                }
            }
            "alpha_vantage" => {
                if self.alpha_vantage_provider.is_none() {
                    return Err(crate::utils::errors::PantherSwapError::market_data(
                        "Primary provider 'alpha_vantage' is not available".to_string()
                    ));
                }
            }
            _ => {
                return Err(crate::utils::errors::PantherSwapError::market_data(
                    format!("Unknown primary provider: {}", self.primary_provider)
                ));
            }
        }
        Ok(())
    }

    /// Get latest quote from the primary provider with fallback
    pub async fn get_latest_quote_primary(&self, symbol: &str) -> Result<crate::market_data::types::MarketQuote> {
        // Try primary provider first
        let primary_result = self.try_get_quote_from_provider(&self.primary_provider, symbol).await;

        match primary_result {
            Ok(quote) => {
                info!("Successfully retrieved quote for {} from primary provider {}",
                    symbol, self.primary_provider);
                Ok(quote)
            }
            Err(e) => {
                warn!("Primary provider {} failed for symbol {}: {}",
                    self.primary_provider, symbol, e);

                // Try fallback providers
                self.try_fallback_providers(symbol).await
            }
        }
    }

    /// Try to get quote from a specific provider
    async fn try_get_quote_from_provider(&self, provider: &str, symbol: &str) -> Result<crate::market_data::types::MarketQuote> {
        match provider {
            "alpaca" => {
                if let Some(ref provider) = self.alpaca_provider {
                    provider.get_latest_quote(symbol).await
                } else {
                    Err(crate::utils::errors::PantherSwapError::market_data(
                        "Alpaca provider not available".to_string()
                    ))
                }
            }
            "alpha_vantage" => {
                if let Some(ref provider) = self.alpha_vantage_provider {
                    // For Alpha Vantage, we need to convert symbol format
                    let (from, to) = if symbol.len() >= 6 {
                        (symbol[0..3].to_string(), symbol[3..6].to_string())
                    } else {
                        (symbol.to_string(), "USD".to_string())
                    };
                    provider.get_fx_quote(&from, &to).await
                } else {
                    Err(crate::utils::errors::PantherSwapError::market_data(
                        "Alpha Vantage provider not available".to_string()
                    ))
                }
            }
            _ => Err(crate::utils::errors::PantherSwapError::market_data(
                format!("Unknown provider: {}", provider)
            ))
        }
    }

    /// Try fallback providers when primary fails
    async fn try_fallback_providers(&self, symbol: &str) -> Result<crate::market_data::types::MarketQuote> {
        // Define fallback order based on primary provider
        let fallback_providers = match self.primary_provider.as_str() {
            "alpaca" => vec!["alpha_vantage"],
            "alpha_vantage" => vec!["alpaca"],
            _ => vec!["alpaca", "alpha_vantage"],
        };

        for fallback_provider in fallback_providers {
            info!("Trying fallback provider: {}", fallback_provider);

            match self.try_get_quote_from_provider(fallback_provider, symbol).await {
                Ok(quote) => {
                    info!("Successfully retrieved quote for {} from fallback provider {}",
                        symbol, fallback_provider);
                    return Ok(quote);
                }
                Err(e) => {
                    warn!("Fallback provider {} failed for symbol {}: {}",
                        fallback_provider, symbol, e);
                    continue;
                }
            }
        }

        Err(crate::utils::errors::PantherSwapError::market_data(
            format!("All providers failed to retrieve quote for symbol: {}", symbol)
        ))
    }

    /// Start live data streaming for the primary provider
    pub async fn start_live_streaming(&self, symbols: Vec<String>) -> Result<()> {
        match self.primary_provider.as_str() {
            "alpaca" => {
                if let Some(ref provider) = self.alpaca_provider {
                    info!("Starting Alpaca live streaming for {} symbols", symbols.len());
                    let mut stream = provider.start_streaming(symbols).await?;

                    // Process streaming events in background
                    tokio::spawn(async move {
                        while let Some(event) = stream.recv().await {
                            match event {
                                crate::market_data::alpaca::AlpacaStreamEvent::Quote(quote) => {
                                    info!("Received quote: {} @ ${:.2}", quote.symbol, quote.bid_price);
                                }
                                crate::market_data::alpaca::AlpacaStreamEvent::Trade(trade) => {
                                    info!("Received trade: {} @ ${:.2}", trade.symbol, trade.price);
                                }
                                crate::market_data::alpaca::AlpacaStreamEvent::Error(error) => {
                                    error!("Streaming error: {}", error);
                                }
                                _ => {}
                            }
                        }
                    });

                    Ok(())
                } else {
                    Err(crate::utils::errors::PantherSwapError::market_data(
                        "Alpaca provider not available for streaming".to_string()
                    ))
                }
            }
            "alpha_vantage" => {
                // Alpha Vantage doesn't support real-time streaming
                info!("Alpha Vantage doesn't support real-time streaming, using polling");
                self.start_live_collection().await
            }
            _ => Err(crate::utils::errors::PantherSwapError::market_data(
                format!("Streaming not supported for provider: {}", self.primary_provider)
            ))
        }
    }

    /// Get multiple quotes efficiently
    pub async fn get_multiple_quotes(&self, symbols: &[String]) -> Result<std::collections::HashMap<String, crate::market_data::types::MarketQuote>> {
        match self.primary_provider.as_str() {
            "alpaca" => {
                if let Some(ref provider) = self.alpaca_provider {
                    provider.get_multiple_quotes(symbols).await
                } else {
                    Err(crate::utils::errors::PantherSwapError::market_data(
                        "Alpaca provider not available".to_string()
                    ))
                }
            }
            "alpha_vantage" => {
                // For Alpha Vantage, we need to make individual requests
                let mut quotes = std::collections::HashMap::new();
                for symbol in symbols {
                    match self.get_latest_quote_primary(symbol).await {
                        Ok(quote) => {
                            quotes.insert(symbol.clone(), quote);
                        }
                        Err(e) => {
                            warn!("Failed to get quote for {}: {}", symbol, e);
                        }
                    }
                }
                Ok(quotes)
            }
            _ => Err(crate::utils::errors::PantherSwapError::market_data(
                format!("Multiple quotes not supported for provider: {}", self.primary_provider)
            ))
        }
    }

    /// Get provider status and health information
    pub async fn get_provider_status(&self) -> Result<serde_json::Value> {
        let mut status = serde_json::json!({
            "primary_provider": self.primary_provider,
            "live_collection_enabled": self.live_collection_enabled,
            "providers": {}
        });

        // Check Alpaca provider status
        if let Some(ref provider) = self.alpaca_provider {
            let alpaca_status = match provider.get_account_info().await {
                Ok(account_info) => serde_json::json!({
                    "available": true,
                    "ready_for_trading": provider.is_ready_for_trading().await,
                    "account_info": account_info
                }),
                Err(e) => serde_json::json!({
                    "available": false,
                    "error": e.to_string()
                })
            };
            status["providers"]["alpaca"] = alpaca_status;
        }

        // Check Alpha Vantage provider status
        if let Some(ref provider) = self.alpha_vantage_provider {
            let av_status = match provider.validate_configuration() {
                Ok(_) => serde_json::json!({
                    "available": true,
                    "api_key_configured": !provider.get_api_key().is_empty()
                }),
                Err(e) => serde_json::json!({
                    "available": false,
                    "error": e.to_string()
                })
            };
            status["providers"]["alpha_vantage"] = av_status;
        }

        Ok(status)
    }
}
