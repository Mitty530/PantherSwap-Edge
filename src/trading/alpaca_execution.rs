use crate::config::AlpacaConfig;
use crate::database::Database;
use crate::market_data::alpaca::{AlpacaProvider, AlpacaExecutionStats, AlpacaRateLimiter};
use crate::trading::signals::{OrderRequest, ExecutionResult};
use crate::utils::Result;
use alpaca_api_client::{Client, Environment, Order, OrderSide, OrderType, TimeInForce};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Mutex, Semaphore};
use tracing::{error, info, warn, debug};
use uuid::Uuid;

/// Enhanced Alpaca order execution engine with performance optimizations
#[derive(Debug, Clone)]
pub struct AlpacaExecutionEngine {
    client: Arc<Client>,
    config: AlpacaConfig,
    database: Option<Database>,
    active_orders: Arc<RwLock<HashMap<String, AlpacaOrderInfo>>>,
    execution_stats: Arc<Mutex<AlpacaExecutionStats>>,
    rate_limiter: Arc<Mutex<AlpacaRateLimiter>>,

    // Performance optimization components
    execution_semaphore: Arc<Semaphore>,
    order_counter: Arc<AtomicU64>,
    latency_tracker: Arc<Mutex<LatencyTracker>>,

    // Integration with AlpacaProvider for market data
    alpaca_provider: Option<Arc<AlpacaProvider>>,
}

/// Extended order information for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlpacaOrderInfo {
    pub alpaca_order_id: String,
    pub internal_order_id: Uuid,
    pub symbol: String,
    pub side: String,
    pub quantity: f64,
    pub order_type: String,
    pub status: String,
    pub submitted_at: DateTime<Utc>,
    pub filled_at: Option<DateTime<Utc>>,
    pub filled_qty: f64,
    pub filled_avg_price: Option<f64>,
    pub time_in_force: String,
    pub limit_price: Option<f64>,
    pub stop_price: Option<f64>,
}

/// Execution statistics (using the enhanced version from AlpacaProvider)
pub type ExecutionStats = AlpacaExecutionStats;

/// Latency tracking for performance monitoring
#[derive(Debug, Default)]
pub struct LatencyTracker {
    pub total_executions: u64,
    pub total_latency_ms: f64,
    pub min_latency_ms: f64,
    pub max_latency_ms: f64,
    pub target_latency_ms: f64,
    pub violations: u64,
    pub last_updated: DateTime<Utc>,
}

impl LatencyTracker {
    pub fn new(target_latency_ms: f64) -> Self {
        Self {
            target_latency_ms,
            min_latency_ms: f64::MAX,
            last_updated: Utc::now(),
            ..Default::default()
        }
    }

    pub fn record_execution(&mut self, latency_ms: f64) {
        self.total_executions += 1;
        self.total_latency_ms += latency_ms;

        if latency_ms < self.min_latency_ms {
            self.min_latency_ms = latency_ms;
        }
        if latency_ms > self.max_latency_ms {
            self.max_latency_ms = latency_ms;
        }

        if latency_ms > self.target_latency_ms {
            self.violations += 1;
        }

        self.last_updated = Utc::now();
    }

    pub fn average_latency_ms(&self) -> f64 {
        if self.total_executions > 0 {
            self.total_latency_ms / self.total_executions as f64
        } else {
            0.0
        }
    }

    pub fn violation_rate(&self) -> f64 {
        if self.total_executions > 0 {
            self.violations as f64 / self.total_executions as f64
        } else {
            0.0
        }
    }
}

/// Order execution result from Alpaca
#[derive(Debug, Clone)]
pub struct AlpacaExecutionResult {
    pub order_id: String,
    pub internal_id: Uuid,
    pub symbol: String,
    pub side: String,
    pub quantity: f64,
    pub filled_quantity: f64,
    pub average_price: Option<f64>,
    pub status: String,
    pub execution_time_ms: i64,
    pub slippage_bps: f64,
    pub commission: f64,
}

impl AlpacaExecutionEngine {
    /// Create a new enhanced Alpaca execution engine
    pub fn new(config: AlpacaConfig) -> Result<Self> {
        let environment = if config.paper_trading {
            Environment::Paper
        } else {
            Environment::Live
        };

        let client = Client::new(
            &config.api_key,
            &config.secret_key,
            environment,
        ).map_err(|e| crate::utils::PantherSwapError::trading(
            format!("Failed to create Alpaca client: {}", e)
        ))?;

        // Create execution semaphore for concurrency control
        let max_concurrent_executions = 50; // Configurable based on performance needs
        let execution_semaphore = Arc::new(Semaphore::new(max_concurrent_executions));

        Ok(Self {
            client: Arc::new(client),
            config: config.clone(),
            database: None,
            active_orders: Arc::new(RwLock::new(HashMap::new())),
            execution_stats: Arc::new(Mutex::new(AlpacaExecutionStats::default())),
            rate_limiter: Arc::new(Mutex::new(AlpacaRateLimiter::new(config.rate_limit_per_minute))),
            execution_semaphore,
            order_counter: Arc::new(AtomicU64::new(0)),
            latency_tracker: Arc::new(Mutex::new(LatencyTracker::new(10.0))), // 10ms target
            alpaca_provider: None,
        })
    }

    /// Enable database integration
    pub fn with_database(mut self, database: Database) -> Self {
        self.database = Some(database);
        self
    }

    /// Enable AlpacaProvider integration for market data
    pub fn with_alpaca_provider(mut self, provider: Arc<AlpacaProvider>) -> Self {
        self.alpaca_provider = Some(provider);
        self
    }

    /// Enable database integration
    pub fn with_database(mut self, database: Database) -> Self {
        self.database = Some(database);
        self
    }

    /// Execute an order through Alpaca with performance optimizations
    pub async fn execute_order(&self, request: OrderRequest) -> Result<ExecutionResult> {
        let start_time = Instant::now();
        let order_number = self.order_counter.fetch_add(1, Ordering::Relaxed);

        debug!("Executing order #{} through Alpaca: {:?}", order_number, request);

        // Acquire execution permit for concurrency control
        let _permit = self.execution_semaphore.acquire().await
            .map_err(|e| crate::utils::PantherSwapError::trading(
                format!("Failed to acquire execution permit: {}", e)
            ))?;

        // Rate limiting check
        {
            let mut rate_limiter = self.rate_limiter.lock().await;
            rate_limiter.wait_if_needed().await?;
        }

        // Fast validation (optimized for latency)
        self.validate_order_request_fast(&request).await?;

        // Convert internal order to Alpaca order (optimized)
        let alpaca_order = self.convert_to_alpaca_order_fast(&request).await?;

        // Submit order to Alpaca with timeout
        let submit_future = self.client.submit_order(alpaca_order);
        let timeout_duration = Duration::from_millis(self.config.connection_timeout_ms);

        let submitted_order = tokio::time::timeout(timeout_duration, submit_future)
            .await
            .map_err(|_| crate::utils::PantherSwapError::trading(
                format!("Order submission timeout after {}ms", self.config.connection_timeout_ms)
            ))?
            .map_err(|e| crate::utils::PantherSwapError::trading(
                format!("Failed to submit order to Alpaca: {}", e)
            ))?;

        let execution_time_ms = start_time.elapsed().as_millis() as f64;

        // Create order info for tracking
        let order_info = AlpacaOrderInfo {
            alpaca_order_id: submitted_order.id.clone(),
            internal_order_id: request.id,
            symbol: request.instrument_id.to_string(), // Assuming instrument_id is symbol
            side: if request.quantity > 0.0 { "buy".to_string() } else { "sell".to_string() },
            quantity: request.quantity.abs(),
            order_type: format!("{:?}", request.order_type),
            status: format!("{:?}", submitted_order.status),
            submitted_at: Utc::now(),
            filled_at: None,
            filled_qty: submitted_order.filled_qty.unwrap_or(0.0),
            filled_avg_price: submitted_order.filled_avg_price,
            time_in_force: format!("{:?}", submitted_order.time_in_force),
            limit_price: submitted_order.limit_price,
            stop_price: submitted_order.stop_price,
        };

        // Store order info
        {
            let mut active_orders = self.active_orders.write().await;
            active_orders.insert(submitted_order.id.clone(), order_info.clone());
        }

        // Create execution result
        let execution_result = ExecutionResult {
            order_id: request.id,
            instrument_id: request.instrument_id,
            side: if request.quantity > 0.0 { 
                crate::trading::signals::OrderSide::Buy 
            } else { 
                crate::trading::signals::OrderSide::Sell 
            },
            quantity: request.quantity.abs(),
            filled_quantity: submitted_order.filled_qty.unwrap_or(0.0),
            average_price: submitted_order.filled_avg_price.unwrap_or(request.price.unwrap_or(0.0)),
            status: self.convert_alpaca_status_to_internal(submitted_order.status),
            execution_time: Utc::now(),
            commission: 0.0, // Alpaca has commission-free trading
            slippage_bps: self.calculate_slippage(&request, &submitted_order).await,
            market_impact_bps: 0.0, // Will be calculated later
            venue: "alpaca".to_string(),
            execution_id: submitted_order.id.clone(),
        };

        // Update execution statistics
        self.update_execution_stats(&execution_result, execution_time_ms).await;

        // Store execution in database
        if let Some(ref database) = self.database {
            if let Err(e) = self.store_execution_in_database(&execution_result, &order_info).await {
                error!("Failed to store execution in database: {}", e);
            }
        }

        info!("Order executed successfully: {} in {}ms", submitted_order.id, execution_time_ms);

        Ok(execution_result)
    }

    /// Validate order request before submission
    async fn validate_order_request(&self, request: &OrderRequest) -> Result<()> {
        // Check if trading is enabled
        if !self.is_trading_enabled().await? {
            return Err(crate::utils::PantherSwapError::trading(
                "Trading is currently disabled".to_string()
            ));
        }

        // Check order value limits
        let order_value = request.quantity.abs() * request.price.unwrap_or(100.0); // Estimate if no price
        if order_value > self.config.max_order_value {
            return Err(crate::utils::PantherSwapError::trading(
                format!("Order value ${:.2} exceeds maximum ${:.2}", 
                    order_value, self.config.max_order_value)
            ));
        }

        // Check position limits
        let current_positions = self.get_current_position_count().await?;
        if current_positions >= self.config.max_positions {
            return Err(crate::utils::PantherSwapError::trading(
                format!("Maximum positions ({}) reached", self.config.max_positions)
            ));
        }

        Ok(())
    }

    /// Convert internal order request to Alpaca order
    async fn convert_to_alpaca_order(&self, request: &OrderRequest) -> Result<Order> {
        let side = if request.quantity > 0.0 {
            OrderSide::Buy
        } else {
            OrderSide::Sell
        };

        let order_type = match request.order_type {
            crate::trading::signals::OrderType::Market => OrderType::Market,
            crate::trading::signals::OrderType::Limit => OrderType::Limit,
            crate::trading::signals::OrderType::Stop => OrderType::Stop,
            crate::trading::signals::OrderType::StopLimit => OrderType::StopLimit,
        };

        let time_in_force = TimeInForce::Day; // Default to day orders

        let mut alpaca_order = Order::new(
            request.instrument_id.to_string(), // Assuming instrument_id is symbol
            request.quantity.abs(),
            side,
            order_type,
            time_in_force,
        );

        // Set limit price if applicable
        if let Some(price) = request.price {
            alpaca_order = alpaca_order.limit_price(price);
        }

        // Set stop price if applicable
        if let Some(stop_price) = request.stop_price {
            alpaca_order = alpaca_order.stop_price(stop_price);
        }

        // Enable fractional shares if configured
        if self.config.enable_fractional_shares {
            alpaca_order = alpaca_order.extended_hours(true);
        }

        Ok(alpaca_order)
    }

    /// Check if trading is currently enabled
    async fn is_trading_enabled(&self) -> Result<bool> {
        let account = self.client.get_account().await
            .map_err(|e| crate::utils::PantherSwapError::trading(
                format!("Failed to get account status: {}", e)
            ))?;

        Ok(!account.trading_blocked && account.status == alpaca_finance::AccountStatus::Active)
    }

    /// Get current position count
    async fn get_current_position_count(&self) -> Result<u32> {
        let positions = self.client.get_positions().await
            .map_err(|e| crate::utils::PantherSwapError::trading(
                format!("Failed to get positions: {}", e)
            ))?;

        Ok(positions.len() as u32)
    }

    /// Calculate slippage for executed order
    async fn calculate_slippage(&self, request: &OrderRequest, order: &alpaca_finance::Order) -> f64 {
        if let (Some(expected_price), Some(actual_price)) = (request.price, order.filled_avg_price) {
            let slippage = ((actual_price - expected_price) / expected_price).abs() * 10000.0;
            slippage
        } else {
            0.0
        }
    }

    /// Convert Alpaca order status to internal status
    fn convert_alpaca_status_to_internal(&self, status: alpaca_finance::OrderStatus) -> crate::trading::signals::OrderStatus {
        match status {
            alpaca_finance::OrderStatus::New => crate::trading::signals::OrderStatus::Pending,
            alpaca_finance::OrderStatus::PartiallyFilled => crate::trading::signals::OrderStatus::PartiallyFilled,
            alpaca_finance::OrderStatus::Filled => crate::trading::signals::OrderStatus::Filled,
            alpaca_finance::OrderStatus::DoneForDay => crate::trading::signals::OrderStatus::Cancelled,
            alpaca_finance::OrderStatus::Canceled => crate::trading::signals::OrderStatus::Cancelled,
            alpaca_finance::OrderStatus::Expired => crate::trading::signals::OrderStatus::Cancelled,
            alpaca_finance::OrderStatus::Replaced => crate::trading::signals::OrderStatus::Pending,
            alpaca_finance::OrderStatus::PendingCancel => crate::trading::signals::OrderStatus::Pending,
            alpaca_finance::OrderStatus::PendingReplace => crate::trading::signals::OrderStatus::Pending,
            alpaca_finance::OrderStatus::Rejected => crate::trading::signals::OrderStatus::Rejected,
            alpaca_finance::OrderStatus::Suspended => crate::trading::signals::OrderStatus::Cancelled,
            alpaca_finance::OrderStatus::PendingNew => crate::trading::signals::OrderStatus::Pending,
            alpaca_finance::OrderStatus::Calculated => crate::trading::signals::OrderStatus::Pending,
            _ => crate::trading::signals::OrderStatus::Unknown,
        }
    }

    /// Update execution statistics
    async fn update_execution_stats(&self, result: &ExecutionResult, execution_time_ms: i64) {
        let mut stats = self.execution_stats.write().await;
        stats.total_orders += 1;
        
        match result.status {
            crate::trading::signals::OrderStatus::Filled => {
                stats.filled_orders += 1;
                stats.total_volume += result.filled_quantity;
                
                // Update average fill time
                let total_time = stats.average_fill_time_ms * (stats.filled_orders - 1) as f64 + execution_time_ms as f64;
                stats.average_fill_time_ms = total_time / stats.filled_orders as f64;
                
                // Update average slippage
                let total_slippage = stats.slippage_bps * (stats.filled_orders - 1) as f64 + result.slippage_bps;
                stats.slippage_bps = total_slippage / stats.filled_orders as f64;
            }
            crate::trading::signals::OrderStatus::Cancelled => stats.cancelled_orders += 1,
            crate::trading::signals::OrderStatus::Rejected => stats.rejected_orders += 1,
            _ => {}
        }
    }

    /// Store execution in database
    async fn store_execution_in_database(&self, result: &ExecutionResult, order_info: &AlpacaOrderInfo) -> Result<()> {
        if let Some(ref database) = self.database {
            // Store trade execution
            database.store_trade_execution(
                result.order_id,
                result.instrument_id,
                &result.side.to_string(),
                result.quantity,
                result.filled_quantity,
                result.average_price,
                &result.status.to_string(),
                result.execution_time,
                result.commission,
                result.slippage_bps,
                result.market_impact_bps,
                &result.venue,
                &serde_json::json!({
                    "alpaca_order_id": order_info.alpaca_order_id,
                    "order_type": order_info.order_type,
                    "time_in_force": order_info.time_in_force,
                    "limit_price": order_info.limit_price,
                    "stop_price": order_info.stop_price,
                    "paper_trading": self.config.paper_trading,
                }),
            ).await?;
        }

        Ok(())
    }

    /// Get execution statistics
    pub async fn get_execution_stats(&self) -> ExecutionStats {
        self.execution_stats.read().await.clone()
    }

    /// Cancel an order
    pub async fn cancel_order(&self, order_id: &str) -> Result<bool> {
        info!("Cancelling order: {}", order_id);

        match self.client.cancel_order(order_id).await {
            Ok(_) => {
                // Update order status in tracking
                if let Some(mut order_info) = {
                    let mut active_orders = self.active_orders.write().await;
                    active_orders.remove(order_id)
                } {
                    order_info.status = "cancelled".to_string();
                    
                    // Update stats
                    let mut stats = self.execution_stats.write().await;
                    stats.cancelled_orders += 1;
                }
                
                info!("Order {} cancelled successfully", order_id);
                Ok(true)
            }
            Err(e) => {
                error!("Failed to cancel order {}: {}", order_id, e);
                Ok(false)
            }
        }
    }

    /// Get order status
    pub async fn get_order_status(&self, order_id: &str) -> Result<Option<AlpacaOrderInfo>> {
        // Check local tracking first
        {
            let active_orders = self.active_orders.read().await;
            if let Some(order_info) = active_orders.get(order_id) {
                return Ok(Some(order_info.clone()));
            }
        }

        // Fetch from Alpaca if not in local tracking
        match self.client.get_order(order_id).await {
            Ok(order) => {
                let order_info = AlpacaOrderInfo {
                    alpaca_order_id: order.id.clone(),
                    internal_order_id: Uuid::new_v4(), // We don't have the original internal ID
                    symbol: order.symbol,
                    side: format!("{:?}", order.side),
                    quantity: order.qty,
                    order_type: format!("{:?}", order.order_type),
                    status: format!("{:?}", order.status),
                    submitted_at: order.submitted_at,
                    filled_at: order.filled_at,
                    filled_qty: order.filled_qty.unwrap_or(0.0),
                    filled_avg_price: order.filled_avg_price,
                    time_in_force: format!("{:?}", order.time_in_force),
                    limit_price: order.limit_price,
                    stop_price: order.stop_price,
                };

                Ok(Some(order_info))
            }
            Err(_) => Ok(None),
        }
    }

    /// Get all active orders
    pub async fn get_active_orders(&self) -> Result<Vec<AlpacaOrderInfo>> {
        let active_orders = self.active_orders.read().await;
        Ok(active_orders.values().cloned().collect())
    }

    /// Get all positions
    pub async fn get_positions(&self) -> Result<Vec<serde_json::Value>> {
        let positions = self.client.get_positions().await
            .map_err(|e| crate::utils::PantherSwapError::trading(
                format!("Failed to get positions: {}", e)
            ))?;

        let position_data: Vec<serde_json::Value> = positions.into_iter().map(|pos| {
            serde_json::json!({
                "symbol": pos.symbol,
                "qty": pos.qty,
                "side": pos.side,
                "market_value": pos.market_value,
                "cost_basis": pos.cost_basis,
                "unrealized_pl": pos.unrealized_pl,
                "unrealized_plpc": pos.unrealized_plpc,
                "current_price": pos.current_price,
            })
        }).collect();

        Ok(position_data)
    }

    /// Execute a market buy order
    pub async fn market_buy(&self, symbol: &str, quantity: f64) -> Result<ExecutionResult> {
        let order_request = OrderRequest {
            id: Uuid::new_v4(),
            instrument_id: Uuid::new_v4(), // Will be resolved from symbol
            order_type: crate::trading::signals::OrderType::Market,
            quantity,
            price: None,
            stop_price: None,
            time_in_force: crate::trading::signals::TimeInForce::Day,
            created_at: chrono::Utc::now(),
        };

        self.execute_order(order_request).await
    }

    /// Execute a market sell order
    pub async fn market_sell(&self, symbol: &str, quantity: f64) -> Result<ExecutionResult> {
        let order_request = OrderRequest {
            id: Uuid::new_v4(),
            instrument_id: Uuid::new_v4(), // Will be resolved from symbol
            order_type: crate::trading::signals::OrderType::Market,
            quantity: -quantity, // Negative for sell
            price: None,
            stop_price: None,
            time_in_force: crate::trading::signals::TimeInForce::Day,
            created_at: chrono::Utc::now(),
        };

        self.execute_order(order_request).await
    }

    /// Execute a limit buy order
    pub async fn limit_buy(&self, symbol: &str, quantity: f64, limit_price: f64) -> Result<ExecutionResult> {
        let order_request = OrderRequest {
            id: Uuid::new_v4(),
            instrument_id: Uuid::new_v4(), // Will be resolved from symbol
            order_type: crate::trading::signals::OrderType::Limit,
            quantity,
            price: Some(limit_price),
            stop_price: None,
            time_in_force: crate::trading::signals::TimeInForce::Day,
            created_at: chrono::Utc::now(),
        };

        self.execute_order(order_request).await
    }

    /// Execute a limit sell order
    pub async fn limit_sell(&self, symbol: &str, quantity: f64, limit_price: f64) -> Result<ExecutionResult> {
        let order_request = OrderRequest {
            id: Uuid::new_v4(),
            instrument_id: Uuid::new_v4(), // Will be resolved from symbol
            order_type: crate::trading::signals::OrderType::Limit,
            quantity: -quantity, // Negative for sell
            price: Some(limit_price),
            stop_price: None,
            time_in_force: crate::trading::signals::TimeInForce::Day,
            created_at: chrono::Utc::now(),
        };

        self.execute_order(order_request).await
    }

    /// Close all positions
    pub async fn close_all_positions(&self) -> Result<Vec<ExecutionResult>> {
        info!("Closing all positions");

        let positions = self.get_positions().await?;
        let mut results = Vec::new();

        for position in positions {
            if let (Some(symbol), Some(qty)) = (
                position.get("symbol").and_then(|s| s.as_str()),
                position.get("qty").and_then(|q| q.as_f64())
            ) {
                if qty != 0.0 {
                    // Close position by selling if long, buying if short
                    let close_qty = if qty > 0.0 { -qty } else { qty.abs() };

                    match self.market_sell(symbol, close_qty.abs()).await {
                        Ok(result) => {
                            info!("Closed position for {}: {} shares", symbol, close_qty);
                            results.push(result);
                        }
                        Err(e) => {
                            error!("Failed to close position for {}: {}", symbol, e);
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    /// Get portfolio summary
    pub async fn get_portfolio_summary(&self) -> Result<serde_json::Value> {
        let account = self.client.get_account().await
            .map_err(|e| crate::utils::PantherSwapError::trading(
                format!("Failed to get account: {}", e)
            ))?;

        let positions = self.get_positions().await?;
        let stats = self.get_execution_stats().await;

        Ok(serde_json::json!({
            "account": {
                "equity": account.equity,
                "cash": account.cash,
                "buying_power": account.buying_power,
                "portfolio_value": account.portfolio_value,
                "day_trade_buying_power": account.day_trade_buying_power,
                "regt_buying_power": account.regt_buying_power,
            },
            "positions": positions,
            "execution_stats": {
                "total_orders": stats.total_orders,
                "filled_orders": stats.filled_orders,
                "fill_rate": if stats.total_orders > 0 {
                    stats.filled_orders as f64 / stats.total_orders as f64 * 100.0
                } else { 0.0 },
                "average_fill_time_ms": stats.average_fill_time_ms,
                "average_slippage_bps": stats.slippage_bps,
                "total_volume": stats.total_volume,
            },
            "paper_trading": self.config.paper_trading,
        }))
    }

    /// Monitor order until completion or timeout
    pub async fn monitor_order(&self, order_id: &str, timeout_seconds: u64) -> Result<AlpacaOrderInfo> {
        let start_time = std::time::Instant::now();
        let timeout_duration = std::time::Duration::from_secs(timeout_seconds);

        loop {
            if start_time.elapsed() > timeout_duration {
                return Err(crate::utils::PantherSwapError::trading(
                    format!("Order monitoring timed out after {} seconds", timeout_seconds)
                ));
            }

            if let Some(order_info) = self.get_order_status(order_id).await? {
                match order_info.status.as_str() {
                    "filled" | "cancelled" | "rejected" | "expired" => {
                        return Ok(order_info);
                    }
                    _ => {
                        // Order still pending, wait and check again
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    }
                }
            } else {
                return Err(crate::utils::PantherSwapError::trading(
                    format!("Order {} not found", order_id)
                ));
            }
        }
    }

    /// Fast order validation optimized for latency
    async fn validate_order_request_fast(&self, request: &OrderRequest) -> Result<()> {
        // Basic validation only - skip expensive checks for speed
        if request.quantity == 0.0 {
            return Err(crate::utils::PantherSwapError::trading(
                "Order quantity cannot be zero".to_string()
            ));
        }

        // Quick order value check
        if let Some(price) = request.price {
            let order_value = request.quantity.abs() * price;
            if order_value > self.config.max_order_value {
                return Err(crate::utils::PantherSwapError::trading(
                    format!("Order value ${:.2} exceeds maximum ${:.2}",
                        order_value, self.config.max_order_value)
                ));
            }
        }

        Ok(())
    }

    /// Fast order conversion optimized for latency
    async fn convert_to_alpaca_order_fast(&self, request: &OrderRequest) -> Result<Order> {
        let side = if request.quantity > 0.0 {
            OrderSide::Buy
        } else {
            OrderSide::Sell
        };

        let order_type = match request.order_type {
            crate::trading::signals::OrderType::Market => OrderType::Market,
            crate::trading::signals::OrderType::Limit => OrderType::Limit,
            crate::trading::signals::OrderType::Stop => OrderType::Stop,
            crate::trading::signals::OrderType::StopLimit => OrderType::StopLimit,
        };

        // Create order with minimal allocations
        let alpaca_order = Order {
            symbol: request.instrument_id.to_string(), // Assuming instrument_id is symbol
            qty: request.quantity.abs(),
            side,
            order_type,
            time_in_force: TimeInForce::Day,
            limit_price: request.price,
            stop_price: request.stop_price,
            extended_hours: self.config.enable_fractional_shares,
            client_order_id: Some(format!("ps_{}", self.order_counter.load(Ordering::Relaxed))),
        };

        Ok(alpaca_order)
    }

    /// Update latency tracking
    async fn update_latency_tracking(&self, execution_time_ms: f64) {
        let mut tracker = self.latency_tracker.lock().await;
        tracker.record_execution(execution_time_ms);

        // Log performance alerts if needed
        if execution_time_ms > tracker.target_latency_ms {
            warn!("Execution latency {}ms exceeded target {}ms",
                execution_time_ms, tracker.target_latency_ms);
        }
    }

    /// Get performance metrics
    pub async fn get_performance_metrics(&self) -> Result<serde_json::Value> {
        let tracker = self.latency_tracker.lock().await;
        let stats = self.execution_stats.lock().await;

        Ok(serde_json::json!({
            "latency_metrics": {
                "total_executions": tracker.total_executions,
                "average_latency_ms": tracker.average_latency_ms(),
                "min_latency_ms": tracker.min_latency_ms,
                "max_latency_ms": tracker.max_latency_ms,
                "target_latency_ms": tracker.target_latency_ms,
                "violation_rate": tracker.violation_rate(),
                "violations": tracker.violations,
            },
            "execution_stats": {
                "total_orders": stats.total_orders,
                "filled_orders": stats.filled_orders,
                "cancelled_orders": stats.cancelled_orders,
                "rejected_orders": stats.rejected_orders,
                "total_volume": stats.total_volume,
                "average_fill_time_ms": stats.average_fill_time_ms,
                "slippage_bps": stats.slippage_bps,
            },
            "concurrency_metrics": {
                "available_permits": self.execution_semaphore.available_permits(),
                "total_orders_processed": self.order_counter.load(Ordering::Relaxed),
            }
        }))
    }

    /// Execute order with enhanced performance tracking
    pub async fn execute_order_with_tracking(&self, request: OrderRequest) -> Result<(ExecutionResult, f64)> {
        let start_time = Instant::now();
        let result = self.execute_order(request).await?;
        let execution_time_ms = start_time.elapsed().as_millis() as f64;

        // Update latency tracking
        self.update_latency_tracking(execution_time_ms).await;

        Ok((result, execution_time_ms))
    }

    /// Batch execute multiple orders for improved throughput
    pub async fn batch_execute_orders(&self, requests: Vec<OrderRequest>) -> Result<Vec<(ExecutionResult, f64)>> {
        let mut results = Vec::with_capacity(requests.len());
        let batch_start = Instant::now();

        info!("Executing batch of {} orders", requests.len());

        // Execute orders concurrently with semaphore limiting
        let futures: Vec<_> = requests.into_iter().map(|request| {
            self.execute_order_with_tracking(request)
        }).collect();

        // Wait for all orders to complete
        let batch_results = futures::future::join_all(futures).await;

        for result in batch_results {
            match result {
                Ok((execution_result, latency)) => {
                    results.push((execution_result, latency));
                }
                Err(e) => {
                    error!("Batch order execution failed: {}", e);
                    // Continue with other orders
                }
            }
        }

        let batch_time_ms = batch_start.elapsed().as_millis() as f64;
        info!("Batch execution completed: {} orders in {:.2}ms",
            results.len(), batch_time_ms);

        Ok(results)
    }
}
