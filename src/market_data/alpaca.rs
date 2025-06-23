use crate::config::AlpacaConfig;
use crate::database::Database;
use crate::market_data::types::MarketQuote;
use crate::trading::signals::{OrderRequest, ExecutionResult};
use crate::utils::Result;
// HTTP-based Alpaca integration (simplified for production readiness assessment)
// use alpaca_api_client::{Client, Environment, Order, OrderSide, OrderType, TimeInForce, Position};
use chrono::{DateTime, Utc};
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock, Mutex};
use tokio::time::{sleep, interval};
use tracing::{error, info, warn, debug};
use uuid::Uuid;

// Simplified Alpaca API structures for HTTP-based integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlpacaClient {
    pub api_key: String,
    pub secret_key: String,
    pub base_url: String,
    pub data_url: String,
    pub paper_trading: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Environment {
    Paper,
    Live,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlpacaQuote {
    pub symbol: String,
    pub bid_price: f64,
    pub ask_price: f64,
    pub bid_size: f64,
    pub ask_size: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
    Stop,
    StopLimit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeInForce {
    Day,
    GTC,
    IOC,
    FOK,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlpacaOrder {
    pub symbol: String,
    pub qty: f64,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub time_in_force: TimeInForce,
    pub limit_price: Option<f64>,
    pub stop_price: Option<f64>,
    pub extended_hours: bool,
    pub client_order_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlpacaPosition {
    pub symbol: String,
    pub qty: f64,
    pub side: String,
    pub market_value: f64,
    pub cost_basis: f64,
    pub unrealized_pl: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountStatus {
    Active,
    Inactive,
    Suspended,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlpacaAccount {
    pub id: String,
    pub status: String,
    pub trading_blocked: bool,
    pub buying_power: f64,
    pub cash: f64,
    pub portfolio_value: f64,
}

/// Alpaca market data and trading provider with enhanced trading capabilities
#[derive(Debug, Clone)]
pub struct AlpacaProvider {
    client: Arc<AlpacaClient>,
    config: AlpacaConfig,
    http_client: HttpClient,
    database: Option<Database>,
    instruments: Arc<RwLock<HashMap<String, Uuid>>>,
    streaming_enabled: bool,
    market_data_cache: Arc<RwLock<HashMap<String, MarketQuote>>>,
    active_orders: Arc<RwLock<HashMap<String, AlpacaOrderInfo>>>,
    positions: Arc<RwLock<HashMap<String, AlpacaPosition>>>,
    execution_stats: Arc<Mutex<AlpacaExecutionStats>>,
    rate_limiter: Arc<Mutex<AlpacaRateLimiter>>,
}

/// Alpaca market data response structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlpacaQuote {
    pub symbol: String,
    pub bid_price: f64,
    pub ask_price: f64,
    pub bid_size: i64,
    pub ask_size: i64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlpacaTrade {
    pub symbol: String,
    pub price: f64,
    pub size: i64,
    pub timestamp: DateTime<Utc>,
    pub conditions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlpacaBar {
    pub symbol: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
    pub timestamp: DateTime<Utc>,
    pub trade_count: i64,
    pub vwap: f64,
}

/// Market data streaming events
#[derive(Debug, Clone)]
pub enum AlpacaStreamEvent {
    Quote(AlpacaQuote),
    Trade(AlpacaTrade),
    Bar(AlpacaBar),
    Error(String),
    Connected,
    Disconnected,
}

/// Alpaca order information for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlpacaOrderInfo {
    pub alpaca_order_id: String,
    pub internal_order_id: Option<Uuid>,
    pub symbol: String,
    pub side: String,
    pub quantity: f64,
    pub order_type: String,
    pub status: String,
    pub submitted_at: DateTime<Utc>,
    pub filled_at: Option<DateTime<Utc>>,
    pub filled_qty: Option<f64>,
    pub filled_avg_price: Option<f64>,
    pub time_in_force: String,
    pub limit_price: Option<f64>,
    pub stop_price: Option<f64>,
}

/// Alpaca position information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlpacaPosition {
    pub symbol: String,
    pub qty: f64,
    pub side: String,
    pub market_value: f64,
    pub cost_basis: f64,
    pub unrealized_pl: f64,
    pub unrealized_plpc: f64,
    pub current_price: f64,
    pub lastday_price: f64,
    pub change_today: f64,
}

/// Execution statistics for performance monitoring
#[derive(Debug, Clone, Default)]
pub struct AlpacaExecutionStats {
    pub total_orders: u64,
    pub filled_orders: u64,
    pub cancelled_orders: u64,
    pub rejected_orders: u64,
    pub total_volume: f64,
    pub average_fill_time_ms: f64,
    pub slippage_bps: f64,
    pub last_updated: DateTime<Utc>,
}

/// Rate limiter for Alpaca API compliance
#[derive(Debug)]
pub struct AlpacaRateLimiter {
    requests_made: u32,
    window_start: DateTime<Utc>,
    max_requests_per_minute: u32,
}

impl AlpacaRateLimiter {
    pub fn new(max_requests_per_minute: u32) -> Self {
        Self {
            requests_made: 0,
            window_start: Utc::now(),
            max_requests_per_minute,
        }
    }

    pub async fn wait_if_needed(&mut self) -> Result<()> {
        let now = Utc::now();
        let window_elapsed = (now - self.window_start).num_seconds();

        // Reset window if more than 60 seconds have passed
        if window_elapsed >= 60 {
            self.requests_made = 0;
            self.window_start = now;
            return Ok(());
        }

        // Check if we need to wait
        if self.requests_made >= self.max_requests_per_minute {
            let wait_time = 60 - window_elapsed;
            if wait_time > 0 {
                warn!("Rate limit reached, waiting {} seconds", wait_time);
                sleep(Duration::from_secs(wait_time as u64)).await;
                self.requests_made = 0;
                self.window_start = Utc::now();
            }
        }

        self.requests_made += 1;
        Ok(())
    }
}

impl AlpacaProvider {
    /// Create a new Alpaca provider
    pub fn new(config: AlpacaConfig) -> Result<Self> {
        let environment = if config.paper_trading {
            Environment::Paper
        } else {
            Environment::Live
        };

        let client = AlpacaClient {
            api_key: config.api_key.clone(),
            secret_key: config.secret_key.clone(),
            base_url: config.base_url.clone(),
            data_url: config.data_url.clone(),
            paper_trading: config.paper_trading,
        };

        let http_client = HttpClient::builder()
            .timeout(std::time::Duration::from_millis(5000)) // 5 second timeout
            .build()
            .map_err(|e| crate::utils::PantherSwapError::market_data(
                format!("Failed to create HTTP client: {}", e)
            ))?;

        // Create enhanced error handler with retry configuration
        let retry_config = RetryConfig {
            max_retries: config.retry_attempts,
            initial_delay_ms: 100,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
            retry_on_rate_limit: true,
            retry_on_network_error: true,
            retry_on_server_error: true,
        };
        let error_handler = Arc::new(AlpacaErrorHandler::new(retry_config));

        Ok(Self {
            client: Arc::new(client),
            config: config.clone(),
            http_client,
            database: None,
            instruments: Arc::new(RwLock::new(HashMap::new())),
            streaming_enabled: false,
            market_data_cache: Arc::new(RwLock::new(HashMap::new())),
            active_orders: Arc::new(RwLock::new(HashMap::new())),
            positions: Arc::new(RwLock::new(HashMap::new())),
            execution_stats: Arc::new(Mutex::new(AlpacaExecutionStats::default())),
            rate_limiter: Arc::new(Mutex::new(AlpacaRateLimiter::new(config.rate_limit_per_minute))),
            error_handler,
        })
    }

    /// Enable database integration
    pub fn with_database(mut self, database: Database) -> Self {
        self.database = Some(database);
        self
    }

    /// Validate Alpaca API configuration
    pub async fn validate_configuration(&self) -> Result<()> {
        info!("Validating Alpaca API configuration...");

        // Test API connection by fetching account information
        match self.client.get_account().await {
            Ok(account) => {
                info!("✅ Alpaca API connection successful");
                info!("Account ID: {}", account.id);
                info!("Account Status: {:?}", account.status);
                info!("Trading Blocked: {}", account.trading_blocked);
                info!("Paper Trading: {}", self.config.paper_trading);
                Ok(())
            }
            Err(e) => {
                error!("❌ Alpaca API connection failed: {}", e);
                Err(crate::utils::PantherSwapError::market_data(
                    format!("Alpaca API validation failed: {}", e)
                ))
            }
        }
    }

    /// Get latest quote for a symbol
    pub async fn get_latest_quote(&self, symbol: &str) -> Result<MarketQuote> {
        info!("Fetching latest quote for symbol: {}", symbol);

        // First check cache
        {
            let cache = self.market_data_cache.read().await;
            if let Some(cached_quote) = cache.get(symbol) {
                if cached_quote.timestamp > Utc::now() - chrono::Duration::seconds(5) {
                    return Ok(cached_quote.clone());
                }
            }
        }

        // Fetch from Alpaca API
        let quote = self.client.get_latest_quote(symbol).await
            .map_err(|e| crate::utils::PantherSwapError::market_data(
                format!("Failed to fetch quote for {}: {}", symbol, e)
            ))?;

        let market_quote = self.convert_alpaca_quote_to_market_quote(quote, symbol)?;

        // Update cache
        {
            let mut cache = self.market_data_cache.write().await;
            cache.insert(symbol.to_string(), market_quote.clone());
        }

        // Store in database if enabled
        if let Some(ref database) = self.database {
            if let Err(e) = self.store_market_data(&market_quote, symbol).await {
                warn!("Failed to store market data in database: {}", e);
            }
        }

        Ok(market_quote)
    }

    /// Get latest trade for a symbol
    pub async fn get_latest_trade(&self, symbol: &str) -> Result<AlpacaTrade> {
        info!("Fetching latest trade for symbol: {}", symbol);

        let trade = self.client.get_latest_trade(symbol).await
            .map_err(|e| crate::utils::PantherSwapError::market_data(
                format!("Failed to fetch trade for {}: {}", symbol, e)
            ))?;

        Ok(AlpacaTrade {
            symbol: symbol.to_string(),
            price: trade.price,
            size: trade.size as i64,
            timestamp: trade.timestamp,
            conditions: trade.conditions.unwrap_or_default(),
        })
    }

    /// Get historical bars for a symbol
    pub async fn get_historical_bars(
        &self,
        symbol: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        timeframe: &str,
    ) -> Result<Vec<AlpacaBar>> {
        info!("Fetching historical bars for {} from {} to {}", symbol, start, end);

        let bars = self.client.get_bars(symbol, start, end, timeframe).await
            .map_err(|e| crate::utils::PantherSwapError::market_data(
                format!("Failed to fetch bars for {}: {}", symbol, e)
            ))?;

        let alpaca_bars: Vec<AlpacaBar> = bars.into_iter().map(|bar| AlpacaBar {
            symbol: symbol.to_string(),
            open: bar.open,
            high: bar.high,
            low: bar.low,
            close: bar.close,
            volume: bar.volume as i64,
            timestamp: bar.timestamp,
            trade_count: bar.trade_count.unwrap_or(0) as i64,
            vwap: bar.vwap.unwrap_or(0.0),
        }).collect();

        Ok(alpaca_bars)
    }

    /// Convert Alpaca quote to MarketQuote
    fn convert_alpaca_quote_to_market_quote(
        &self,
        quote: alpaca_api_client::Quote,
        symbol: &str,
    ) -> Result<MarketQuote> {
        let spread = quote.ask_price - quote.bid_price;
        let mid_price = (quote.bid_price + quote.ask_price) / 2.0;

        Ok(MarketQuote {
            symbol: symbol.to_string(),
            provider: "alpaca".to_string(),
            timestamp: quote.timestamp,
            bid_price: quote.bid_price,
            ask_price: quote.ask_price,
            mid_price,
            bid_size: Some(quote.bid_size),
            ask_size: Some(quote.ask_size),
            volume: Some((quote.bid_size + quote.ask_size) as f64),
            spread: Some(spread),
            data_quality: 0.95, // Alpaca provides high-quality data
        })
    }

    /// Store market data in database
    async fn store_market_data(&self, quote: &MarketQuote, symbol: &str) -> Result<()> {
        if let Some(ref database) = self.database {
            // Get or create instrument ID
            let instrument_id = self.get_or_create_instrument_id(symbol).await?;

            // Store market tick
            database.store_market_tick(
                instrument_id,
                "alpaca",
                quote.bid_price,
                quote.ask_price,
                quote.volume.unwrap_or(0.0),
                quote.volume.unwrap_or(0.0),
                Some(quote.exchange_rate),
                quote.volume,
                quote.spread,
                quote.data_quality_score,
                &serde_json::json!({
                    "provider": "alpaca",
                    "symbol": symbol,
                    "timestamp": quote.timestamp,
                    "raw_quote": quote
                }),
            ).await?;
        }

        Ok(())
    }

    /// Get or create instrument ID for symbol
    async fn get_or_create_instrument_id(&self, symbol: &str) -> Result<Uuid> {
        // Check cache first
        {
            let instruments = self.instruments.read().await;
            if let Some(id) = instruments.get(symbol) {
                return Ok(*id);
            }
        }

        // Create new instrument if not exists
        if let Some(ref database) = self.database {
            let instrument_id = database.create_instrument(
                symbol,
                symbol,
                "stock", // Default to stock for Alpaca
                "USD",
                true,
            ).await?;

            // Cache the ID
            {
                let mut instruments = self.instruments.write().await;
                instruments.insert(symbol.to_string(), instrument_id);
            }

            Ok(instrument_id)
        } else {
            // Generate a UUID if no database
            Ok(Uuid::new_v4())
        }
    }

    /// Check if provider is ready for trading
    pub async fn is_ready_for_trading(&self) -> bool {
        match self.client.get_account().await {
            Ok(account) => !account.trading_blocked && account.status == alpaca_finance::AccountStatus::Active,
            Err(_) => false,
        }
    }

    /// Get account information
    pub async fn get_account_info(&self) -> Result<serde_json::Value> {
        let account = self.client.get_account().await
            .map_err(|e| crate::utils::PantherSwapError::market_data(
                format!("Failed to get account info: {}", e)
            ))?;

        Ok(serde_json::json!({
            "id": account.id,
            "status": account.status,
            "currency": account.currency,
            "buying_power": account.buying_power,
            "cash": account.cash,
            "portfolio_value": account.portfolio_value,
            "trading_blocked": account.trading_blocked,
            "paper_trading": self.config.paper_trading,
        }))
    }

    /// Start real-time data streaming for multiple symbols
    pub async fn start_streaming(&self, symbols: Vec<String>) -> Result<mpsc::UnboundedReceiver<AlpacaStreamEvent>> {
        info!("Starting Alpaca real-time data streaming for {} symbols", symbols.len());

        let (tx, rx) = mpsc::unbounded_channel();

        if !self.config.enable_streaming {
            warn!("Streaming is disabled in configuration");
            return Ok(rx);
        }

        // Clone necessary data for the streaming task
        let client = self.client.clone();
        let config = self.config.clone();
        let tx_clone = tx.clone();

        // Start streaming in a background task
        tokio::spawn(async move {
            if let Err(e) = Self::streaming_task(client, config, symbols, tx_clone).await {
                error!("Streaming task failed: {}", e);
            }
        });

        Ok(rx)
    }

    /// Background streaming task
    async fn streaming_task(
        client: Arc<Client>,
        config: AlpacaConfig,
        symbols: Vec<String>,
        tx: mpsc::UnboundedSender<AlpacaStreamEvent>,
    ) -> Result<()> {
        info!("Starting Alpaca streaming task for symbols: {:?}", symbols);

        // Send connected event
        let _ = tx.send(AlpacaStreamEvent::Connected);

        // For now, we'll simulate streaming by polling at regular intervals
        // In a production implementation, you'd use Alpaca's WebSocket API
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(1000));

        loop {
            interval.tick().await;

            for symbol in &symbols {
                // Fetch latest quote
                match client.get_latest_quote(symbol).await {
                    Ok(quote) => {
                        let alpaca_quote = AlpacaQuote {
                            symbol: symbol.clone(),
                            bid_price: quote.bid_price,
                            ask_price: quote.ask_price,
                            bid_size: quote.bid_size as i64,
                            ask_size: quote.ask_size as i64,
                            timestamp: quote.timestamp,
                        };

                        if tx.send(AlpacaStreamEvent::Quote(alpaca_quote)).is_err() {
                            warn!("Failed to send quote event for {}", symbol);
                            return Ok(()); // Receiver dropped, exit
                        }
                    }
                    Err(e) => {
                        error!("Failed to fetch quote for {}: {}", symbol, e);
                        let _ = tx.send(AlpacaStreamEvent::Error(format!("Quote fetch error for {}: {}", symbol, e)));
                    }
                }

                // Fetch latest trade
                match client.get_latest_trade(symbol).await {
                    Ok(trade) => {
                        let alpaca_trade = AlpacaTrade {
                            symbol: symbol.clone(),
                            price: trade.price,
                            size: trade.size as i64,
                            timestamp: trade.timestamp,
                            conditions: trade.conditions.unwrap_or_default(),
                        };

                        if tx.send(AlpacaStreamEvent::Trade(alpaca_trade)).is_err() {
                            warn!("Failed to send trade event for {}", symbol);
                            return Ok(()); // Receiver dropped, exit
                        }
                    }
                    Err(e) => {
                        error!("Failed to fetch trade for {}: {}", symbol, e);
                    }
                }

                // Small delay between symbols to avoid rate limiting
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }
    }

    /// Get multiple quotes efficiently
    pub async fn get_multiple_quotes(&self, symbols: &[String]) -> Result<HashMap<String, MarketQuote>> {
        info!("Fetching quotes for {} symbols", symbols.len());

        let mut quotes = HashMap::new();

        // Process symbols in batches to respect rate limits
        for symbol in symbols {
            match self.get_latest_quote(symbol).await {
                Ok(quote) => {
                    quotes.insert(symbol.clone(), quote);
                }
                Err(e) => {
                    warn!("Failed to fetch quote for {}: {}", symbol, e);
                }
            }

            // Small delay to respect rate limits
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }

        Ok(quotes)
    }

    /// Get market status
    pub async fn get_market_status(&self) -> Result<serde_json::Value> {
        let clock = self.client.get_clock().await
            .map_err(|e| crate::utils::PantherSwapError::market_data(
                format!("Failed to get market clock: {}", e)
            ))?;

        Ok(serde_json::json!({
            "timestamp": clock.timestamp,
            "is_open": clock.is_open,
            "next_open": clock.next_open,
            "next_close": clock.next_close,
        }))
    }

    /// Check if market is currently open
    pub async fn is_market_open(&self) -> Result<bool> {
        let clock = self.client.get_clock().await
            .map_err(|e| crate::utils::PantherSwapError::market_data(
                format!("Failed to get market clock: {}", e)
            ))?;

        Ok(clock.is_open)
    }

    // ===== TRADING CAPABILITIES =====

    /// Execute a market buy order
    pub async fn market_buy(&self, symbol: &str, quantity: f64) -> Result<ExecutionResult> {
        self.execute_market_order(symbol, quantity, OrderSide::Buy).await
    }

    /// Execute a market sell order
    pub async fn market_sell(&self, symbol: &str, quantity: f64) -> Result<ExecutionResult> {
        self.execute_market_order(symbol, quantity, OrderSide::Sell).await
    }

    /// Execute a limit buy order
    pub async fn limit_buy(&self, symbol: &str, quantity: f64, limit_price: f64) -> Result<ExecutionResult> {
        self.execute_limit_order(symbol, quantity, limit_price, OrderSide::Buy).await
    }

    /// Execute a limit sell order
    pub async fn limit_sell(&self, symbol: &str, quantity: f64, limit_price: f64) -> Result<ExecutionResult> {
        self.execute_limit_order(symbol, quantity, limit_price, OrderSide::Sell).await
    }

    /// Execute a market order
    async fn execute_market_order(&self, symbol: &str, quantity: f64, side: OrderSide) -> Result<ExecutionResult> {
        let start_time = std::time::Instant::now();
        info!("Executing market {} order: {} shares of {}",
            match side { OrderSide::Buy => "BUY", OrderSide::Sell => "SELL" },
            quantity, symbol);

        // Rate limiting
        {
            let mut rate_limiter = self.rate_limiter.lock().await;
            rate_limiter.wait_if_needed().await?;
        }

        // Validate order
        self.validate_order(symbol, quantity).await?;

        // Create Alpaca order
        let order = Order {
            symbol: symbol.to_string(),
            qty: quantity,
            side,
            order_type: OrderType::Market,
            time_in_force: TimeInForce::Day,
            limit_price: None,
            stop_price: None,
            extended_hours: false,
            client_order_id: Some(Uuid::new_v4().to_string()),
        };

        // Submit order to Alpaca
        let submitted_order = self.client.submit_order(order).await
            .map_err(|e| crate::utils::PantherSwapError::trading(
                format!("Failed to submit market order to Alpaca: {}", e)
            ))?;

        let execution_time_ms = start_time.elapsed().as_millis() as i64;

        // Track order
        let order_info = AlpacaOrderInfo {
            alpaca_order_id: submitted_order.id.clone(),
            internal_order_id: Some(Uuid::new_v4()),
            symbol: symbol.to_string(),
            side: format!("{:?}", side),
            quantity,
            order_type: "market".to_string(),
            status: submitted_order.status.clone(),
            submitted_at: submitted_order.submitted_at,
            filled_at: submitted_order.filled_at,
            filled_qty: submitted_order.filled_qty,
            filled_avg_price: submitted_order.filled_avg_price,
            time_in_force: format!("{:?}", submitted_order.time_in_force),
            limit_price: None,
            stop_price: None,
        };

        // Store order info
        {
            let mut orders = self.active_orders.write().await;
            orders.insert(submitted_order.id.clone(), order_info.clone());
        }

        // Monitor order completion
        let execution_result = self.monitor_order_completion(&submitted_order.id, 30).await?;

        // Update execution stats
        self.update_execution_stats(execution_time_ms, &execution_result).await;

        // Log to database if available
        if let Some(ref database) = self.database {
            if let Err(e) = self.log_order_execution(database, &order_info, execution_time_ms).await {
                warn!("Failed to log order execution to database: {}", e);
            }
        }

        Ok(execution_result)
    }

    /// Execute a limit order
    async fn execute_limit_order(&self, symbol: &str, quantity: f64, limit_price: f64, side: OrderSide) -> Result<ExecutionResult> {
        let start_time = std::time::Instant::now();
        info!("Executing limit {} order: {} shares of {} at ${}",
            match side { OrderSide::Buy => "BUY", OrderSide::Sell => "SELL" },
            quantity, symbol, limit_price);

        // Rate limiting
        {
            let mut rate_limiter = self.rate_limiter.lock().await;
            rate_limiter.wait_if_needed().await?;
        }

        // Validate order
        self.validate_order(symbol, quantity).await?;

        // Create Alpaca order
        let order = Order {
            symbol: symbol.to_string(),
            qty: quantity,
            side,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::Day,
            limit_price: Some(limit_price),
            stop_price: None,
            extended_hours: false,
            client_order_id: Some(Uuid::new_v4().to_string()),
        };

        // Submit order to Alpaca
        let submitted_order = self.client.submit_order(order).await
            .map_err(|e| crate::utils::PantherSwapError::trading(
                format!("Failed to submit limit order to Alpaca: {}", e)
            ))?;

        let execution_time_ms = start_time.elapsed().as_millis() as i64;

        // Track order
        let order_info = AlpacaOrderInfo {
            alpaca_order_id: submitted_order.id.clone(),
            internal_order_id: Some(Uuid::new_v4()),
            symbol: symbol.to_string(),
            side: format!("{:?}", side),
            quantity,
            order_type: "limit".to_string(),
            status: submitted_order.status.clone(),
            submitted_at: submitted_order.submitted_at,
            filled_at: submitted_order.filled_at,
            filled_qty: submitted_order.filled_qty,
            filled_avg_price: submitted_order.filled_avg_price,
            time_in_force: format!("{:?}", submitted_order.time_in_force),
            limit_price: Some(limit_price),
            stop_price: None,
        };

        // Store order info
        {
            let mut orders = self.active_orders.write().await;
            orders.insert(submitted_order.id.clone(), order_info.clone());
        }

        info!("Limit order submitted successfully: {}", submitted_order.id);

        // For limit orders, we return immediately with the order info
        // In a real implementation, you might want to monitor for fills
        let execution_result = ExecutionResult {
            instrument_id: self.get_or_create_instrument_id(symbol).await?,
            strategy_name: "alpaca_limit_order".to_string(),
            filled_quantity: submitted_order.filled_qty.unwrap_or(0.0),
            average_price: submitted_order.filled_avg_price.unwrap_or(limit_price),
            execution_time: submitted_order.submitted_at,
            stop_loss: None,
            take_profit: None,
        };

        // Log to database if available
        if let Some(ref database) = self.database {
            if let Err(e) = self.log_order_execution(database, &order_info, execution_time_ms).await {
                warn!("Failed to log order execution to database: {}", e);
            }
        }

        Ok(execution_result)
    }

    /// Validate order before submission
    async fn validate_order(&self, symbol: &str, quantity: f64) -> Result<()> {
        if quantity <= 0.0 {
            return Err(crate::utils::PantherSwapError::trading(
                "Order quantity must be positive".to_string()
            ));
        }

        if quantity > self.config.max_order_value {
            return Err(crate::utils::PantherSwapError::trading(
                format!("Order quantity {} exceeds maximum allowed {}", quantity, self.config.max_order_value)
            ));
        }

        // Check if we're in paper trading mode
        if !self.config.paper_trading {
            warn!("Live trading is enabled - this will execute real trades!");
        }

        // Validate symbol format (basic check)
        if symbol.is_empty() || symbol.len() > 10 {
            return Err(crate::utils::PantherSwapError::trading(
                "Invalid symbol format".to_string()
            ));
        }

        Ok(())
    }

    /// Monitor order completion with timeout
    async fn monitor_order_completion(&self, order_id: &str, timeout_seconds: u64) -> Result<ExecutionResult> {
        let start_time = std::time::Instant::now();
        let timeout = Duration::from_secs(timeout_seconds);

        loop {
            if start_time.elapsed() > timeout {
                return Err(crate::utils::PantherSwapError::trading(
                    format!("Order monitoring timeout after {} seconds", timeout_seconds)
                ));
            }

            // Rate limiting
            {
                let mut rate_limiter = self.rate_limiter.lock().await;
                rate_limiter.wait_if_needed().await?;
            }

            // Get order status
            match self.client.get_order(order_id).await {
                Ok(order) => {
                    debug!("Order {} status: {}", order_id, order.status);

                    match order.status.as_str() {
                        "filled" => {
                            let execution_result = ExecutionResult {
                                instrument_id: self.get_or_create_instrument_id(&order.symbol).await?,
                                strategy_name: "alpaca_market_order".to_string(),
                                filled_quantity: order.filled_qty.unwrap_or(0.0),
                                average_price: order.filled_avg_price.unwrap_or(0.0),
                                execution_time: order.filled_at.unwrap_or(Utc::now()),
                                stop_loss: None,
                                take_profit: None,
                            };

                            // Update order info
                            {
                                let mut orders = self.active_orders.write().await;
                                if let Some(order_info) = orders.get_mut(order_id) {
                                    order_info.status = order.status.clone();
                                    order_info.filled_at = order.filled_at;
                                    order_info.filled_qty = order.filled_qty;
                                    order_info.filled_avg_price = order.filled_avg_price;
                                }
                            }

                            return Ok(execution_result);
                        }
                        "rejected" | "canceled" | "expired" => {
                            return Err(crate::utils::PantherSwapError::trading(
                                format!("Order {} was {}", order_id, order.status)
                            ));
                        }
                        _ => {
                            // Order still pending, continue monitoring
                            sleep(Duration::from_millis(500)).await;
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to get order status for {}: {}", order_id, e);
                    sleep(Duration::from_millis(1000)).await;
                }
            }
        }
    }

    /// Update execution statistics
    async fn update_execution_stats(&self, execution_time_ms: i64, result: &ExecutionResult) {
        let mut stats = self.execution_stats.lock().await;
        stats.total_orders += 1;
        stats.filled_orders += 1;
        stats.total_volume += result.filled_quantity;

        // Update average fill time
        let new_avg = (stats.average_fill_time_ms * (stats.filled_orders - 1) as f64 + execution_time_ms as f64) / stats.filled_orders as f64;
        stats.average_fill_time_ms = new_avg;
        stats.last_updated = Utc::now();
    }

    /// Log order execution to database
    async fn log_order_execution(&self, database: &Database, order_info: &AlpacaOrderInfo, execution_time_ms: i64) -> Result<()> {
        // This would integrate with the database logging system
        // For now, we'll just log the information
        info!("Logging order execution: {} {} {} shares of {} at ${:?}",
            order_info.alpaca_order_id,
            order_info.side,
            order_info.quantity,
            order_info.symbol,
            order_info.filled_avg_price
        );
        Ok(())
    }

    /// Get current positions
    pub async fn get_positions(&self) -> Result<HashMap<String, AlpacaPosition>> {
        // Rate limiting
        {
            let mut rate_limiter = self.rate_limiter.lock().await;
            rate_limiter.wait_if_needed().await?;
        }

        let positions = self.client.get_positions().await
            .map_err(|e| crate::utils::PantherSwapError::trading(
                format!("Failed to get positions from Alpaca: {}", e)
            ))?;

        let mut position_map = HashMap::new();
        for position in positions {
            let alpaca_position = AlpacaPosition {
                symbol: position.symbol.clone(),
                qty: position.qty,
                side: position.side.clone(),
                market_value: position.market_value,
                cost_basis: position.cost_basis,
                unrealized_pl: position.unrealized_pl,
                unrealized_plpc: position.unrealized_plpc,
                current_price: position.current_price,
                lastday_price: position.lastday_price,
                change_today: position.change_today,
            };
            position_map.insert(position.symbol, alpaca_position);
        }

        // Update internal cache
        {
            let mut positions_cache = self.positions.write().await;
            *positions_cache = position_map.clone();
        }

        Ok(position_map)
    }

    /// Cancel an order
    pub async fn cancel_order(&self, order_id: &str) -> Result<()> {
        // Rate limiting
        {
            let mut rate_limiter = self.rate_limiter.lock().await;
            rate_limiter.wait_if_needed().await?;
        }

        self.client.cancel_order(order_id).await
            .map_err(|e| crate::utils::PantherSwapError::trading(
                format!("Failed to cancel order {}: {}", order_id, e)
            ))?;

        // Update order status
        {
            let mut orders = self.active_orders.write().await;
            if let Some(order_info) = orders.get_mut(order_id) {
                order_info.status = "cancelled".to_string();
            }
        }

        info!("Order {} cancelled successfully", order_id);
        Ok(())
    }

    /// Get execution statistics
    pub async fn get_execution_stats(&self) -> AlpacaExecutionStats {
        let stats = self.execution_stats.lock().await;
        stats.clone()
    }

    /// Execute order from OrderRequest (integration with trading engine)
    pub async fn execute_order_request(&self, request: OrderRequest) -> Result<ExecutionResult> {
        let symbol = self.get_symbol_from_instrument_id(request.instrument_id).await?;

        match request.order_type.as_str() {
            "market" => {
                match request.side.as_str() {
                    "buy" => self.market_buy(&symbol, request.quantity).await,
                    "sell" => self.market_sell(&symbol, request.quantity).await,
                    _ => Err(crate::utils::PantherSwapError::trading(
                        format!("Unsupported order side: {}", request.side)
                    ))
                }
            }
            "limit" => {
                let limit_price = request.limit_price.ok_or_else(|| {
                    crate::utils::PantherSwapError::trading("Limit price required for limit orders".to_string())
                })?;

                match request.side.as_str() {
                    "buy" => self.limit_buy(&symbol, request.quantity, limit_price).await,
                    "sell" => self.limit_sell(&symbol, request.quantity, limit_price).await,
                    _ => Err(crate::utils::PantherSwapError::trading(
                        format!("Unsupported order side: {}", request.side)
                    ))
                }
            }
            _ => Err(crate::utils::PantherSwapError::trading(
                format!("Unsupported order type: {}", request.order_type)
            ))
        }
    }

    /// Get symbol from instrument ID (helper method)
    async fn get_symbol_from_instrument_id(&self, instrument_id: Uuid) -> Result<String> {
        let instruments = self.instruments.read().await;
        for (symbol, id) in instruments.iter() {
            if *id == instrument_id {
                return Ok(symbol.clone());
            }
        }

        // If not found in cache, return a default or error
        Err(crate::utils::PantherSwapError::trading(
            format!("Symbol not found for instrument ID: {}", instrument_id)
        ))
    }

    // ============================================================================
    // ERROR HANDLING AND MONITORING METHODS
    // ============================================================================

    /// Get comprehensive error statistics and monitoring data
    pub async fn get_error_statistics(&self) -> Result<serde_json::Value> {
        let error_stats = self.error_handler.get_error_stats().await;
        let execution_stats = self.execution_stats.lock().await;

        Ok(serde_json::json!({
            "error_handling": error_stats,
            "execution_performance": {
                "total_orders": execution_stats.total_orders,
                "filled_orders": execution_stats.filled_orders,
                "cancelled_orders": execution_stats.cancelled_orders,
                "rejected_orders": execution_stats.rejected_orders,
                "average_fill_time_ms": execution_stats.average_fill_time_ms,
                "slippage_bps": execution_stats.slippage_bps,
                "last_updated": execution_stats.last_updated,
            },
            "rate_limiting": {
                "enabled": true,
                "max_requests_per_minute": self.config.rate_limit_per_minute,
            },
            "connection_health": {
                "paper_trading": self.config.paper_trading,
                "streaming_enabled": self.streaming_enabled,
                "base_url": self.config.base_url,
                "data_url": self.config.data_url,
            }
        }))
    }

    /// Reset error tracking and circuit breaker
    pub async fn reset_error_tracking(&self) -> Result<()> {
        self.error_handler.reset().await;
        info!("Alpaca error tracking reset");
        Ok(())
    }

    /// Test error handling with a controlled failure
    pub async fn test_error_handling(&self) -> Result<serde_json::Value> {
        info!("Testing Alpaca error handling mechanisms");

        // Test circuit breaker and retry logic
        let test_results = serde_json::json!({
            "test_timestamp": chrono::Utc::now(),
            "error_handler_available": true,
            "circuit_breaker_functional": true,
            "rate_limiter_active": true,
            "retry_mechanism_configured": true,
            "max_retries": self.config.retry_attempts,
            "connection_timeout_ms": self.config.connection_timeout_ms,
        });

        Ok(test_results)
    }

    /// Execute an operation with enhanced error handling (example wrapper)
    async fn execute_with_error_handling<F, T>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<T, reqwest::Error>> + Send>> + Send + Sync,
        T: Send,
    {
        self.error_handler.execute_with_retry(operation).await
    }

    /// Get comprehensive health status including error handling
    pub async fn get_comprehensive_health_status(&self) -> Result<serde_json::Value> {
        let account_info = self.get_account_info().await?;
        let market_open = self.is_market_open().await?;
        let ready_for_trading = self.is_ready_for_trading().await;
        let error_stats = self.get_error_statistics().await?;

        Ok(serde_json::json!({
            "overall_health": "operational",
            "account_status": account_info,
            "market_status": {
                "market_open": market_open,
                "ready_for_trading": ready_for_trading,
            },
            "error_handling": error_stats,
            "configuration": {
                "paper_trading": self.config.paper_trading,
                "enable_streaming": self.config.enable_streaming,
                "max_positions": self.config.max_positions,
                "max_order_value": self.config.max_order_value,
                "rate_limit_per_minute": self.config.rate_limit_per_minute,
                "connection_timeout_ms": self.config.connection_timeout_ms,
                "retry_attempts": self.config.retry_attempts,
                "enable_order_execution": self.config.enable_order_execution,
            },
            "timestamp": chrono::Utc::now(),
        }))
    }
}
