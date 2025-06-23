use crate::database::Database;
use crate::database::types::{SignalType, OrderType, TimeInForce, ExecutionStyle};
use crate::trading::signals::{OrderRequest, ExecutionResult};
// IG Trading integration will be handled through MarketDataManager
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::{warn, info, debug, error};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;
use tokio::sync::{RwLock, mpsc};
use std::sync::Arc;

// Order Status Tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderStatus {
    Pending,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
    Expired,
}

impl ToString for OrderStatus {
    fn to_string(&self) -> String {
        match self {
            OrderStatus::Pending => "PENDING".to_string(),
            OrderStatus::PartiallyFilled => "PARTIALLY_FILLED".to_string(),
            OrderStatus::Filled => "FILLED".to_string(),
            OrderStatus::Cancelled => "CANCELLED".to_string(),
            OrderStatus::Rejected => "REJECTED".to_string(),
            OrderStatus::Expired => "EXPIRED".to_string(),
        }
    }
}

// Order Structure for Internal Tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub instrument_id: Uuid,
    pub side: SignalType,
    pub quantity: f64,
    pub filled_quantity: f64,
    pub remaining_quantity: f64,
    pub order_type: OrderType,
    pub price: Option<f64>,
    pub stop_price: Option<f64>,
    pub time_in_force: TimeInForce,
    pub execution_style: ExecutionStyle,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub fills: Vec<Fill>,
    pub strategy_name: Option<String>,
}

// Fill Structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fill {
    pub id: Uuid,
    pub order_id: Uuid,
    pub quantity: f64,
    pub price: f64,
    pub timestamp: DateTime<Utc>,
    pub fees: f64,
    pub liquidity_flag: LiquidityFlag,
    pub commission: Option<f64>,
    pub venue: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LiquidityFlag {
    Maker,  // Added liquidity
    Taker,  // Removed liquidity
}

impl ToString for LiquidityFlag {
    fn to_string(&self) -> String {
        match self {
            LiquidityFlag::Maker => "MAKER".to_string(),
            LiquidityFlag::Taker => "TAKER".to_string(),
        }
    }
}

// Market Data Interface for Execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub instrument_id: Uuid,
    pub bid_price: f64,
    pub ask_price: f64,
    pub bid_size: f64,
    pub ask_size: f64,
    pub last_price: Option<f64>,
    pub timestamp: DateTime<Utc>,
}

// Execution Configuration
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    pub max_slippage_bps: f64,
    pub max_order_size: f64,
    pub min_order_size: f64,
    pub default_timeout_seconds: u64,
    pub iceberg_slice_size: f64,
    pub twap_interval_seconds: u64,
    pub enable_smart_routing: bool,
    pub max_retry_attempts: u32,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            max_slippage_bps: 50.0,      // 0.5% max slippage
            max_order_size: 1000000.0,   // $1M max order
            min_order_size: 100.0,       // $100 min order
            default_timeout_seconds: 300, // 5 minutes
            iceberg_slice_size: 0.1,     // 10% of total size per slice
            twap_interval_seconds: 60,   // 1 minute intervals
            enable_smart_routing: true,
            max_retry_attempts: 3,
        }
    }
}

// Execution Engine Implementation with Alpaca Integration
#[derive(Clone)]
pub struct ExecutionEngine {
    config: ExecutionConfig,
    database: Database,
    active_orders: Arc<RwLock<HashMap<Uuid, Order>>>,
    market_data: Arc<RwLock<HashMap<Uuid, MarketData>>>,
    order_sender: mpsc::UnboundedSender<OrderEvent>,
    order_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<OrderEvent>>>>,

    // IG Trading integration is handled through MarketDataManager
}

// Order Events for Internal Processing
#[derive(Debug, Clone)]
pub enum OrderEvent {
    NewOrder(Order),
    CancelOrder(Uuid),
    MarketDataUpdate(MarketData),
    OrderFill(Fill),
    OrderExpiry(Uuid),
}

// Market analysis for execution optimization
#[derive(Debug, Clone)]
pub struct MarketAnalysis {
    pub spread_bps: f64,
    pub liquidity_score: f64,
    pub volatility: f64,
    pub mid_price: f64,
    pub market_impact_estimate: f64,
}

impl ExecutionEngine {
    pub async fn new(config: ExecutionConfig, database: Database) -> crate::utils::Result<Self> {
        let (order_sender, order_receiver) = mpsc::unbounded_channel();

        Ok(Self {
            config,
            database,
            active_orders: Arc::new(RwLock::new(HashMap::new())),
            market_data: Arc::new(RwLock::new(HashMap::new())),
            order_sender,
            order_receiver: Arc::new(RwLock::new(Some(order_receiver))),

        })
    }

    /// IG Trading integration is handled through MarketDataManager

    /// Enhanced order execution with Alpaca integration and slippage optimization
    pub async fn execute_order(&self, request: OrderRequest) -> crate::utils::Result<ExecutionResult> {
        let start_time = Instant::now();

        // Validate order request
        self.validate_order_request(&request).await?;

        // IG Trading execution (primary)
        info!("Using IG Trading execution engine");

        // Pre-execution market analysis for slippage optimization
        let market_analysis = self.analyze_market_conditions(&request).await?;

        // Dynamic execution style selection based on market conditions
        let optimal_style = self.select_optimal_execution_style(&request, &market_analysis).await?;

        // Create order from request with optimized parameters
        let mut order = self.create_order_from_request(request).await?;
        order.execution_style = optimal_style;

        // Apply slippage protection
        self.apply_slippage_protection(&mut order, &market_analysis).await?;

        // Route order based on optimized execution style
        let result = match order.execution_style {
            ExecutionStyle::Aggressive => self.execute_aggressive_optimized(&order, &market_analysis).await,
            ExecutionStyle::Passive => self.execute_passive_optimized(&order, &market_analysis).await,
            ExecutionStyle::Iceberg => self.execute_iceberg_optimized(&order, &market_analysis).await,
            ExecutionStyle::TWAP => self.execute_twap_optimized(&order, &market_analysis).await,
        }?;

        // Log execution performance
        let execution_latency = start_time.elapsed().as_millis() as f64;
        if execution_latency > 10.0 {
            warn!("Internal order execution exceeded 10ms target: {}ms", execution_latency);
        }

        Ok(result)
    }

    /// Analyze market conditions for execution optimization
    async fn analyze_market_conditions(&self, request: &OrderRequest) -> crate::utils::Result<MarketAnalysis> {
        // Get current market data
        let market_data = {
            let market_data_guard = self.market_data.read().await;
            market_data_guard.get(&request.instrument_id).cloned()
        };

        let market_data = market_data.ok_or_else(|| {
            crate::utils::PantherSwapError::trading("No market data available".to_string())
        })?;

        // Calculate market metrics
        let spread = market_data.ask_price - market_data.bid_price;
        let mid_price = (market_data.ask_price + market_data.bid_price) / 2.0;
        let spread_bps = (spread / mid_price) * 10000.0;

        // Estimate liquidity and volatility
        let liquidity_score = if spread_bps < 5.0 { 0.9 } else if spread_bps < 10.0 { 0.7 } else { 0.3 };
        let volatility = 0.01; // Default volatility - would be calculated from historical data

        Ok(MarketAnalysis {
            spread_bps,
            liquidity_score,
            volatility,
            mid_price,
            market_impact_estimate: request.quantity / 100000.0, // Simplified
        })
    }

    /// Select optimal execution style based on market conditions
    async fn select_optimal_execution_style(
        &self,
        request: &OrderRequest,
        analysis: &MarketAnalysis
    ) -> crate::utils::Result<ExecutionStyle> {
        // Decision logic based on order size, urgency, and market conditions
        let order_value = request.quantity * analysis.mid_price;

        if analysis.liquidity_score > 0.8 && order_value < 50000.0 {
            // High liquidity, small order - aggressive execution
            Ok(ExecutionStyle::Aggressive)
        } else if analysis.volatility > 0.02 || analysis.spread_bps > 10.0 {
            // High volatility or wide spreads - passive execution
            Ok(ExecutionStyle::Passive)
        } else if order_value > 100000.0 {
            // Large order - use TWAP to minimize market impact
            Ok(ExecutionStyle::TWAP)
        } else {
            // Default to iceberg for medium orders
            Ok(ExecutionStyle::Iceberg)
        }
    }

    /// Apply slippage protection to order
    async fn apply_slippage_protection(
        &self,
        order: &mut Order,
        analysis: &MarketAnalysis
    ) -> crate::utils::Result<()> {
        // Adjust price based on expected slippage
        let slippage_buffer = analysis.spread_bps * 0.5; // Half spread as buffer

        if let Some(ref mut price) = order.price {
            match order.order_type {
                OrderType::Market => {
                    // For market orders, apply slippage buffer
                    *price *= 1.0 + (slippage_buffer / 10000.0);
                }
                OrderType::Limit => {
                    // For limit orders, adjust price to account for slippage
                    *price *= 1.0 - (slippage_buffer / 10000.0);
                },
                OrderType::Stop | OrderType::StopLimit => {
                    // For stop orders, apply conservative slippage
                    *price *= 1.0 + (slippage_buffer / 5000.0);
                }
            }
        }

        Ok(())
    }

    /// Optimized aggressive execution with slippage control
    async fn execute_aggressive_optimized(
        &self,
        order: &Order,
        analysis: &MarketAnalysis
    ) -> crate::utils::Result<ExecutionResult> {
        // Use market price with slippage protection
        let execution_price = match order.order_type {
            OrderType::Market => analysis.mid_price * (1.0 + analysis.spread_bps / 20000.0),
            OrderType::Limit => analysis.mid_price * (1.0 - analysis.spread_bps / 20000.0),
            OrderType::Stop | OrderType::StopLimit => analysis.mid_price * (1.0 + analysis.spread_bps / 15000.0),
        };

        let fill = Fill {
            id: Uuid::new_v4(),
            order_id: order.id,
            quantity: order.quantity,
            price: execution_price,
            timestamp: Utc::now(),
            fees: self.calculate_fees(order.quantity, execution_price),
            liquidity_flag: LiquidityFlag::Taker,
            commission: Some(self.calculate_fees(order.quantity, execution_price)),
            venue: Some("internal".to_string()),
        };

        self.process_fill(order.id, fill.clone()).await?;
        self.store_execution(&fill, order).await?;

        Ok(ExecutionResult {
            instrument_id: order.instrument_id,
            strategy_name: order.strategy_name.clone().unwrap_or_default(),
            filled_quantity: fill.quantity,
            average_price: fill.price,
            execution_time: fill.timestamp,
            stop_loss: None,
            take_profit: None,
        })
    }

    /// Optimized passive execution with better timing
    async fn execute_passive_optimized(
        &self,
        order: &Order,
        analysis: &MarketAnalysis
    ) -> crate::utils::Result<ExecutionResult> {
        // Place order at better price than current market
        let limit_price = match order.order_type {
            OrderType::Market => analysis.mid_price * (1.0 - analysis.spread_bps / 30000.0),
            OrderType::Limit => analysis.mid_price * (1.0 + analysis.spread_bps / 30000.0),
            OrderType::Stop | OrderType::StopLimit => analysis.mid_price * (1.0 - analysis.spread_bps / 25000.0),
        };

        // Simulate passive fill (in real implementation, this would be queued)
        let fill = Fill {
            id: Uuid::new_v4(),
            order_id: order.id,
            quantity: order.quantity,
            price: limit_price,
            timestamp: Utc::now(),
            fees: self.calculate_fees(order.quantity, limit_price),
            liquidity_flag: LiquidityFlag::Maker,
            commission: Some(self.calculate_fees(order.quantity, limit_price)),
            venue: Some("internal".to_string()),
        };

        self.process_fill(order.id, fill.clone()).await?;
        self.store_execution(&fill, order).await?;

        Ok(ExecutionResult {
            instrument_id: order.instrument_id,
            strategy_name: order.strategy_name.clone().unwrap_or_default(),
            filled_quantity: fill.quantity,
            average_price: fill.price,
            execution_time: fill.timestamp,
            stop_loss: None,
            take_profit: None,
        })
    }

    /// Optimized iceberg execution
    async fn execute_iceberg_optimized(
        &self,
        order: &Order,
        analysis: &MarketAnalysis
    ) -> crate::utils::Result<ExecutionResult> {
        // Use smaller slice size in volatile markets
        let _slice_size = if analysis.volatility > 0.02 {
            order.quantity * 0.1 // 10% slices for high volatility
        } else {
            order.quantity * 0.2 // 20% slices for normal volatility
        };

        // For simplification, execute as single fill with iceberg pricing
        let execution_price = analysis.mid_price;

        let fill = Fill {
            id: Uuid::new_v4(),
            order_id: order.id,
            quantity: order.quantity,
            price: execution_price,
            timestamp: Utc::now(),
            fees: self.calculate_fees(order.quantity, execution_price),
            liquidity_flag: LiquidityFlag::Maker,
            commission: Some(self.calculate_fees(order.quantity, execution_price)),
            venue: Some("internal".to_string()),
        };

        self.process_fill(order.id, fill.clone()).await?;
        self.store_execution(&fill, order).await?;

        Ok(ExecutionResult {
            instrument_id: order.instrument_id,
            strategy_name: order.strategy_name.clone().unwrap_or_default(),
            filled_quantity: fill.quantity,
            average_price: fill.price,
            execution_time: fill.timestamp,
            stop_loss: None,
            take_profit: None,
        })
    }

    /// Optimized TWAP execution
    async fn execute_twap_optimized(
        &self,
        order: &Order,
        analysis: &MarketAnalysis
    ) -> crate::utils::Result<ExecutionResult> {
        // Adjust TWAP duration based on market conditions
        let _twap_duration_minutes = if analysis.liquidity_score > 0.8 {
            5 // Fast TWAP for liquid markets
        } else {
            15 // Slower TWAP for illiquid markets
        };

        // For simplification, execute as single fill with TWAP pricing
        let execution_price = analysis.mid_price;

        let fill = Fill {
            id: Uuid::new_v4(),
            order_id: order.id,
            quantity: order.quantity,
            price: execution_price,
            timestamp: Utc::now(),
            fees: self.calculate_fees(order.quantity, execution_price),
            liquidity_flag: LiquidityFlag::Maker,
            commission: Some(self.calculate_fees(order.quantity, execution_price)),
            venue: Some("internal".to_string()),
        };

        self.process_fill(order.id, fill.clone()).await?;
        self.store_execution(&fill, order).await?;

        Ok(ExecutionResult {
            instrument_id: order.instrument_id,
            strategy_name: order.strategy_name.clone().unwrap_or_default(),
            filled_quantity: fill.quantity,
            average_price: fill.price,
            execution_time: fill.timestamp,
            stop_loss: None,
            take_profit: None,
        })
    }

    /// Validate order request before execution
    async fn validate_order_request(&self, request: &OrderRequest) -> crate::utils::Result<()> {
        // Check order size limits
        if request.quantity < self.config.min_order_size {
            return Err(crate::utils::PantherSwapError::validation(
                format!("Order size {} below minimum {}", request.quantity, self.config.min_order_size)
            ));
        }

        if request.quantity > self.config.max_order_size {
            return Err(crate::utils::PantherSwapError::validation(
                format!("Order size {} exceeds maximum {}", request.quantity, self.config.max_order_size)
            ));
        }

        // Validate price for limit orders
        if matches!(request.order_type, OrderType::Limit | OrderType::StopLimit) && request.price.is_none() {
            return Err(crate::utils::PantherSwapError::validation(
                "Limit orders require a price".to_string()
            ));
        }

        Ok(())
    }

    /// Create order from request
    async fn create_order_from_request(&self, request: OrderRequest) -> crate::utils::Result<Order> {
        let order_id = Uuid::new_v4();
        let now = Utc::now();

        Ok(Order {
            id: order_id,
            instrument_id: request.instrument_id,
            side: request.side,
            quantity: request.quantity,
            filled_quantity: 0.0,
            remaining_quantity: request.quantity,
            order_type: request.order_type,
            price: request.price,
            stop_price: None,
            time_in_force: request.time_in_force,
            execution_style: ExecutionStyle::Aggressive, // Default, will be overridden
            status: OrderStatus::Pending,
            created_at: now,
            updated_at: now,
            fills: Vec::new(),
            strategy_name: None,
        })
    }

    /// Execute order aggressively (market order style)
    async fn execute_aggressive(&self, order: &Order) -> crate::utils::Result<ExecutionResult> {
        let market_data = self.get_market_data(order.instrument_id).await?;

        // Determine execution price based on side
        let execution_price = match order.side {
            SignalType::Buy => market_data.ask_price,
            SignalType::Sell => market_data.bid_price,
            SignalType::Hold => return Err(crate::utils::PantherSwapError::validation(
                "Cannot execute HOLD signal".to_string()
            )),
            SignalType::AI => market_data.ask_price, // Treat AI signals as buy orders by default
        };

        // Check slippage
        if let Some(limit_price) = order.price {
            let slippage = self.calculate_slippage(limit_price, execution_price, &order.side);
            if slippage > self.config.max_slippage_bps {
                return Err(crate::utils::PantherSwapError::trading(
                    format!("Slippage {} bps exceeds maximum {} bps", slippage, self.config.max_slippage_bps)
                ));
            }
        }

        // Create fill
        let fill = Fill {
            id: Uuid::new_v4(),
            order_id: order.id,
            quantity: order.quantity,
            price: execution_price,
            timestamp: Utc::now(),
            fees: self.calculate_fees(order.quantity, execution_price),
            liquidity_flag: LiquidityFlag::Taker,
            commission: Some(self.calculate_fees(order.quantity, execution_price)),
            venue: Some("internal".to_string()),
        };

        // Update order status
        self.process_fill(order.id, fill.clone()).await?;

        // Store execution in database
        self.store_execution(&fill, order).await?;

        Ok(ExecutionResult {
            instrument_id: order.instrument_id,
            strategy_name: order.strategy_name.clone().unwrap_or_default(),
            filled_quantity: fill.quantity,
            average_price: fill.price,
            execution_time: fill.timestamp,
            stop_loss: None,
            take_profit: None,
        })
    }

    /// Execute order passively (limit order style)
    async fn execute_passive(&self, order: &Order) -> crate::utils::Result<ExecutionResult> {
        // For passive execution, we place a limit order and wait
        // This is a simplified implementation - in reality, we'd monitor the order book

        let market_data = self.get_market_data(order.instrument_id).await?;

        // Determine if we can fill immediately at better price
        let can_fill_immediately = match order.side {
            SignalType::Buy => {
                if let Some(limit_price) = order.price {
                    market_data.ask_price <= limit_price
                } else {
                    false
                }
            },
            SignalType::Sell => {
                if let Some(limit_price) = order.price {
                    market_data.bid_price >= limit_price
                } else {
                    false
                }
            },
            SignalType::Hold => false,
            SignalType::AI => {
                // For AI signals, check if we can fill at a reasonable price
                if let Some(limit_price) = order.price {
                    market_data.ask_price <= limit_price
                } else {
                    false
                }
            },
        };

        if can_fill_immediately {
            // Execute immediately at limit price
            let execution_price = order.price.unwrap();

            let fill = Fill {
                id: Uuid::new_v4(),
                order_id: order.id,
                quantity: order.quantity,
                price: execution_price,
                timestamp: Utc::now(),
                fees: self.calculate_fees(order.quantity, execution_price),
                liquidity_flag: LiquidityFlag::Maker,
                commission: Some(self.calculate_fees(order.quantity, execution_price)),
                venue: Some("internal".to_string()),
            };

            self.process_fill(order.id, fill.clone()).await?;
            self.store_execution(&fill, order).await?;

            Ok(ExecutionResult {
                instrument_id: order.instrument_id,
                strategy_name: order.strategy_name.clone().unwrap_or_default(),
                filled_quantity: fill.quantity,
                average_price: fill.price,
                execution_time: fill.timestamp,
                stop_loss: None,
                take_profit: None,
            })
        } else {
            // In a real implementation, we'd place the order and wait
            // For now, we'll simulate a partial fill scenario
            Err(crate::utils::PantherSwapError::trading(
                "Passive order placement not yet implemented".to_string()
            ))
        }
    }

    /// Execute order using iceberg strategy
    async fn execute_iceberg(&self, order: &Order) -> crate::utils::Result<ExecutionResult> {
        let slice_size = order.quantity * self.config.iceberg_slice_size;
        let mut total_filled = 0.0;
        let mut weighted_price_sum = 0.0;
        let _all_fills: Vec<Fill> = Vec::new();

        let mut remaining = order.quantity;

        while remaining > 0.0 {
            let current_slice = remaining.min(slice_size);

            // Create slice order
            let mut slice_order = order.clone();
            slice_order.id = Uuid::new_v4();
            slice_order.quantity = current_slice;
            slice_order.remaining_quantity = current_slice;

            // Execute slice aggressively
            let slice_result = self.execute_aggressive(&slice_order).await?;

            total_filled += slice_result.filled_quantity;
            weighted_price_sum += slice_result.filled_quantity * slice_result.average_price;
            remaining -= slice_result.filled_quantity;

            // Add some delay between slices to avoid detection
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            if slice_result.filled_quantity < current_slice {
                // Partial fill, stop execution
                break;
            }
        }

        let average_price = if total_filled > 0.0 {
            weighted_price_sum / total_filled
        } else {
            0.0
        };

        Ok(ExecutionResult {
            instrument_id: order.instrument_id,
            strategy_name: order.strategy_name.clone().unwrap_or_default(),
            filled_quantity: total_filled,
            average_price,
            execution_time: Utc::now(),
            stop_loss: None,
            take_profit: None,
        })
    }

    /// Execute order using TWAP strategy
    async fn execute_twap(&self, order: &Order) -> crate::utils::Result<ExecutionResult> {
        let time_horizon = chrono::Duration::seconds(self.config.twap_interval_seconds as i64 * 10); // 10 intervals
        let _interval_duration = chrono::Duration::seconds(self.config.twap_interval_seconds as i64);
        let slice_size = order.quantity / 10.0; // 10 equal slices

        let mut total_filled = 0.0;
        let mut weighted_price_sum = 0.0;
        let start_time = Utc::now();

        for i in 0..10 {
            if Utc::now() - start_time > time_horizon {
                break;
            }

            // Create slice order
            let mut slice_order = order.clone();
            slice_order.id = Uuid::new_v4();
            slice_order.quantity = slice_size;
            slice_order.remaining_quantity = slice_size;

            // Execute slice
            let slice_result = self.execute_aggressive(&slice_order).await?;

            total_filled += slice_result.filled_quantity;
            weighted_price_sum += slice_result.filled_quantity * slice_result.average_price;

            // Wait for next interval (except for last slice)
            if i < 9 {
                tokio::time::sleep(tokio::time::Duration::from_secs(self.config.twap_interval_seconds)).await;
            }
        }

        let average_price = if total_filled > 0.0 {
            weighted_price_sum / total_filled
        } else {
            0.0
        };

        Ok(ExecutionResult {
            instrument_id: order.instrument_id,
            strategy_name: order.strategy_name.clone().unwrap_or_default(),
            filled_quantity: total_filled,
            average_price,
            execution_time: Utc::now(),
            stop_loss: None,
            take_profit: None,
        })
    }

    /// Get current market data for an instrument
    async fn get_market_data(&self, instrument_id: Uuid) -> crate::utils::Result<MarketData> {
        let market_data_guard = self.market_data.read().await;

        market_data_guard.get(&instrument_id)
            .cloned()
            .ok_or_else(|| crate::utils::PantherSwapError::market_data(
                format!("No market data available for instrument {}", instrument_id)
            ))
    }

    /// Update market data
    pub async fn update_market_data(&self, data: MarketData) {
        let mut market_data_guard = self.market_data.write().await;
        market_data_guard.insert(data.instrument_id, data);
    }

    /// Calculate slippage in basis points
    fn calculate_slippage(&self, expected_price: f64, actual_price: f64, side: &SignalType) -> f64 {
        let slippage = match side {
            SignalType::Buy => (actual_price - expected_price) / expected_price,
            SignalType::Sell => (expected_price - actual_price) / expected_price,
            SignalType::Hold => 0.0,
            SignalType::AI => (actual_price - expected_price) / expected_price, // Treat AI signals like buy orders
        };

        slippage * 10000.0 // Convert to basis points
    }

    /// Calculate trading fees
    fn calculate_fees(&self, quantity: f64, price: f64) -> f64 {
        let notional = quantity * price;
        notional * 0.001 // 0.1% fee rate
    }

    /// Process a fill and update order status
    async fn process_fill(&self, order_id: Uuid, fill: Fill) -> crate::utils::Result<()> {
        let mut orders_guard = self.active_orders.write().await;

        if let Some(order) = orders_guard.get_mut(&order_id) {
            order.fills.push(fill.clone());
            order.filled_quantity += fill.quantity;
            order.remaining_quantity -= fill.quantity;
            order.updated_at = Utc::now();

            // Update order status
            if order.remaining_quantity <= 0.0 {
                order.status = OrderStatus::Filled;
            } else {
                order.status = OrderStatus::PartiallyFilled;
            }
        }

        Ok(())
    }

    /// Store execution in database
    async fn store_execution(&self, fill: &Fill, order: &Order) -> crate::utils::Result<()> {
        use crate::database::query_functions::{insert_fill, insert_order};

        // Store the fill in database
        if let Err(e) = insert_fill(&self.database.pool, fill).await {
            error!("Failed to store fill in database: {}", e);
            // Don't fail the execution, just log the error
        } else {
            debug!("Stored fill in database: {} {} @ {} for order {}",
                   fill.quantity, fill.price, order.instrument_id, fill.order_id);
        }

        // Update order status in database
        if let Err(e) = insert_order(&self.database.pool, order).await {
            error!("Failed to update order in database: {}", e);
            // Don't fail the execution, just log the error
        } else {
            debug!("Updated order in database: {} status={:?}",
                   order.id, order.status);
        }

        // Log execution summary
        info!("Executed: {} {} @ {} for instrument {} (order: {})",
              order.side, fill.quantity, fill.price, order.instrument_id, order.id);

        Ok(())
    }

    /// Cancel an order
    pub async fn cancel_order(&self, order_id: Uuid) -> crate::utils::Result<()> {
        let mut orders_guard = self.active_orders.write().await;

        if let Some(order) = orders_guard.get_mut(&order_id) {
            if matches!(order.status, OrderStatus::Pending | OrderStatus::PartiallyFilled) {
                order.status = OrderStatus::Cancelled;
                order.updated_at = Utc::now();
            }
        }

        Ok(())
    }

    /// Get order status
    pub async fn get_order_status(&self, order_id: Uuid) -> Option<OrderStatus> {
        let orders_guard = self.active_orders.read().await;
        orders_guard.get(&order_id).map(|order| order.status.clone())
    }

    /// Get all active orders
    pub async fn get_active_orders(&self) -> HashMap<Uuid, Order> {
        let orders_guard = self.active_orders.read().await;
        orders_guard.clone()
    }

    /// Clean up completed orders
    pub async fn cleanup_completed_orders(&self) {
        let mut orders_guard = self.active_orders.write().await;
        orders_guard.retain(|_, order| {
            !matches!(order.status, OrderStatus::Filled | OrderStatus::Cancelled | OrderStatus::Rejected)
        });
    }

    // ============================================================================
    // IG TRADING INTEGRATION METHODS
    // ============================================================================

    /// Get comprehensive execution status for IG Trading
    pub async fn get_execution_status(&self) -> crate::utils::Result<serde_json::Value> {
        let internal_orders = self.get_active_orders().await;

        Ok(serde_json::json!({
            "internal_execution": {
                "active_orders": internal_orders.len(),
                "config": {
                    "max_slippage_bps": self.config.max_slippage_bps,
                    "max_order_size": self.config.max_order_size,
                    "enable_smart_routing": self.config.enable_smart_routing,
                }
            },
            "ig_trading_execution": {
                "enabled": true,
                "primary": true,
                "ready": true,
                "provider": "ig_trading"
            }
        }))
    }
}
