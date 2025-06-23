use crate::database::Database;
use crate::trading::execution::{Order, OrderStatus, Fill};
use crate::trading::signals::OrderRequest;
use crate::database::types::{OrderType, TimeInForce};
use crate::utils::Result;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use std::collections::{HashMap, VecDeque};
use tokio::sync::{RwLock, mpsc};
use std::sync::Arc;
use tracing::{info, warn, error, debug};

// Order Management Configuration
#[derive(Debug, Clone)]
pub struct OrderManagerConfig {
    pub max_orders_per_instrument: u32,
    pub order_timeout_seconds: u64,
    pub max_order_age_hours: u64,
    pub enable_order_routing: bool,
    pub enable_smart_order_routing: bool,
    pub max_slippage_tolerance_bps: f64,
    pub order_size_precision: u32,
    pub price_precision: u32,
    pub enable_order_aggregation: bool,
    pub fill_reporting_interval_ms: u64,
}

impl Default for OrderManagerConfig {
    fn default() -> Self {
        Self {
            max_orders_per_instrument: 100,
            order_timeout_seconds: 300,  // 5 minutes
            max_order_age_hours: 24,     // 24 hours
            enable_order_routing: true,
            enable_smart_order_routing: true,
            max_slippage_tolerance_bps: 50.0,  // 0.5%
            order_size_precision: 8,
            price_precision: 8,
            enable_order_aggregation: true,
            fill_reporting_interval_ms: 100,  // 100ms
        }
    }
}

// Order Book Entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookEntry {
    pub price: f64,
    pub size: f64,
    pub order_count: u32,
    pub timestamp: DateTime<Utc>,
}

// Order Book State
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub instrument_id: Uuid,
    pub bids: Vec<OrderBookEntry>,
    pub asks: Vec<OrderBookEntry>,
    pub last_updated: DateTime<Utc>,
    pub sequence_number: u64,
}

// Order Events
#[derive(Debug, Clone)]
pub enum OrderEvent {
    OrderCreated(Order),
    OrderUpdated(Order),
    OrderFilled(Fill),
    OrderCancelled(Uuid),
    OrderExpired(Uuid),
    OrderRejected(Uuid, String),
    OrderBookUpdated(OrderBook),
}

// Order Statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderStatistics {
    pub total_orders: u32,
    pub active_orders: u32,
    pub filled_orders: u32,
    pub cancelled_orders: u32,
    pub rejected_orders: u32,
    pub expired_orders: u32,
    pub average_fill_time_ms: f64,
    pub fill_rate: f64,
    pub average_slippage_bps: f64,
    pub total_volume: f64,
    pub total_fees: f64,
}

impl Default for OrderStatistics {
    fn default() -> Self {
        Self {
            total_orders: 0,
            active_orders: 0,
            filled_orders: 0,
            cancelled_orders: 0,
            rejected_orders: 0,
            expired_orders: 0,
            average_fill_time_ms: 0.0,
            fill_rate: 0.0,
            average_slippage_bps: 0.0,
            total_volume: 0.0,
            total_fees: 0.0,
        }
    }
}

// Order Manager Implementation
pub struct OrderManager {
    config: OrderManagerConfig,
    database: Database,
    
    // Order Storage
    orders: Arc<RwLock<HashMap<Uuid, Order>>>,
    orders_by_instrument: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>,
    order_history: Arc<RwLock<VecDeque<Order>>>,
    
    // Order Books
    order_books: Arc<RwLock<HashMap<Uuid, OrderBook>>>,
    
    // Fill Processing
    pending_fills: Arc<RwLock<VecDeque<Fill>>>,
    fill_history: Arc<RwLock<VecDeque<Fill>>>,
    
    // Statistics
    statistics: Arc<RwLock<OrderStatistics>>,
    
    // Event System
    event_sender: mpsc::UnboundedSender<OrderEvent>,
    event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<OrderEvent>>>>,
}

impl OrderManager {
    pub async fn new(config: OrderManagerConfig, database: Database) -> Result<Self> {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        let manager = Self {
            config,
            database,
            orders: Arc::new(RwLock::new(HashMap::new())),
            orders_by_instrument: Arc::new(RwLock::new(HashMap::new())),
            order_history: Arc::new(RwLock::new(VecDeque::new())),
            order_books: Arc::new(RwLock::new(HashMap::new())),
            pending_fills: Arc::new(RwLock::new(VecDeque::new())),
            fill_history: Arc::new(RwLock::new(VecDeque::new())),
            statistics: Arc::new(RwLock::new(OrderStatistics::default())),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        };

        // Start background processing
        manager.start_background_processing().await?;

        info!("Order Manager initialized successfully");
        Ok(manager)
    }

    /// Submit a new order
    pub async fn submit_order(&self, request: OrderRequest) -> Result<Uuid> {
        // Validate order request
        self.validate_order_request(&request).await?;

        // Create order
        let instrument_id = request.instrument_id; // Store before moving
        let order = self.create_order_from_request(request).await?;
        let order_id = order.id;

        // Store order
        {
            let mut orders_guard = self.orders.write().await;
            orders_guard.insert(order_id, order.clone());
        }

        // Update instrument mapping
        {
            let mut by_instrument_guard = self.orders_by_instrument.write().await;
            by_instrument_guard
                .entry(order.instrument_id)
                .or_insert_with(Vec::new)
                .push(order_id);
        }

        // Update statistics
        {
            let mut stats_guard = self.statistics.write().await;
            stats_guard.total_orders += 1;
            stats_guard.active_orders += 1;
        }

        // Send event
        self.send_event(OrderEvent::OrderCreated(order)).await;

        info!("Order {} submitted for instrument {}", order_id, instrument_id);
        Ok(order_id)
    }

    /// Cancel an order
    pub async fn cancel_order(&self, order_id: Uuid) -> Result<bool> {
        let mut orders_guard = self.orders.write().await;
        
        if let Some(order) = orders_guard.get_mut(&order_id) {
            if matches!(order.status, OrderStatus::Pending | OrderStatus::PartiallyFilled) {
                order.status = OrderStatus::Cancelled;
                order.updated_at = Utc::now();

                // Update statistics
                {
                    let mut stats_guard = self.statistics.write().await;
                    stats_guard.active_orders = stats_guard.active_orders.saturating_sub(1);
                    stats_guard.cancelled_orders += 1;
                }

                // Send event
                self.send_event(OrderEvent::OrderCancelled(order_id)).await;

                info!("Order {} cancelled", order_id);
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Get order by ID
    pub async fn get_order(&self, order_id: Uuid) -> Option<Order> {
        let orders_guard = self.orders.read().await;
        orders_guard.get(&order_id).cloned()
    }

    /// Get orders for instrument
    pub async fn get_orders_for_instrument(&self, instrument_id: Uuid) -> Vec<Order> {
        let by_instrument_guard = self.orders_by_instrument.read().await;
        let orders_guard = self.orders.read().await;
        
        if let Some(order_ids) = by_instrument_guard.get(&instrument_id) {
            order_ids.iter()
                .filter_map(|&id| orders_guard.get(&id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get active orders
    pub async fn get_active_orders(&self) -> Vec<Order> {
        let orders_guard = self.orders.read().await;
        orders_guard.values()
            .filter(|order| matches!(order.status, OrderStatus::Pending | OrderStatus::PartiallyFilled))
            .cloned()
            .collect()
    }

    /// Process a fill
    pub async fn process_fill(&self, fill: Fill) -> Result<()> {
        // Add to pending fills for processing
        {
            let mut pending_guard = self.pending_fills.write().await;
            pending_guard.push_back(fill.clone());
        }

        // Update order with fill
        {
            let mut orders_guard = self.orders.write().await;
            if let Some(order) = orders_guard.get_mut(&fill.order_id) {
                order.fills.push(fill.clone());
                order.filled_quantity += fill.quantity;
                order.remaining_quantity -= fill.quantity;
                order.updated_at = Utc::now();

                // Update order status
                if order.remaining_quantity <= 0.0001 {
                    order.status = OrderStatus::Filled;
                    
                    // Update statistics
                    {
                        let mut stats_guard = self.statistics.write().await;
                        stats_guard.active_orders = stats_guard.active_orders.saturating_sub(1);
                        stats_guard.filled_orders += 1;
                        stats_guard.total_volume += fill.quantity;
                        stats_guard.total_fees += fill.fees;
                    }
                } else {
                    order.status = OrderStatus::PartiallyFilled;
                }

                // Send events
                self.send_event(OrderEvent::OrderFilled(fill.clone())).await;
                self.send_event(OrderEvent::OrderUpdated(order.clone())).await;
            }
        }

        // Add to fill history
        {
            let mut history_guard = self.fill_history.write().await;
            history_guard.push_back(fill);
            
            // Keep only recent fills
            while history_guard.len() > 10000 {
                history_guard.pop_front();
            }
        }

        Ok(())
    }

    /// Update order book
    pub async fn update_order_book(&self, order_book: OrderBook) -> Result<()> {
        {
            let mut books_guard = self.order_books.write().await;
            books_guard.insert(order_book.instrument_id, order_book.clone());
        }

        // Send event
        self.send_event(OrderEvent::OrderBookUpdated(order_book)).await;

        Ok(())
    }

    /// Get order book for instrument
    pub async fn get_order_book(&self, instrument_id: Uuid) -> Option<OrderBook> {
        let books_guard = self.order_books.read().await;
        books_guard.get(&instrument_id).cloned()
    }

    /// Get order statistics
    pub async fn get_statistics(&self) -> OrderStatistics {
        self.statistics.read().await.clone()
    }

    /// Validate order request
    async fn validate_order_request(&self, request: &OrderRequest) -> Result<()> {
        // Check order size
        if request.quantity <= 0.0 {
            return Err(crate::utils::PantherSwapError::validation(
                "Order quantity must be positive".to_string()
            ));
        }

        // Check price for limit orders
        if matches!(request.order_type, OrderType::Limit) && request.price.is_none() {
            return Err(crate::utils::PantherSwapError::validation(
                "Limit orders require a price".to_string()
            ));
        }

        if let Some(price) = request.price {
            if price <= 0.0 {
                return Err(crate::utils::PantherSwapError::validation(
                    "Order price must be positive".to_string()
                ));
            }
        }

        // Check instrument order limits
        let instrument_orders = self.get_orders_for_instrument(request.instrument_id).await;
        let active_count = instrument_orders.iter()
            .filter(|order| matches!(order.status, OrderStatus::Pending | OrderStatus::PartiallyFilled))
            .count() as u32;

        if active_count >= self.config.max_orders_per_instrument {
            return Err(crate::utils::PantherSwapError::validation(
                format!("Too many active orders for instrument: {}", active_count)
            ));
        }

        Ok(())
    }

    /// Create order from request
    async fn create_order_from_request(&self, request: OrderRequest) -> Result<Order> {
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
            execution_style: crate::database::types::ExecutionStyle::Aggressive, // Default
            status: OrderStatus::Pending,
            created_at: now,
            updated_at: now,
            fills: Vec::new(),
            strategy_name: None,
        })
    }

    /// Start background processing
    async fn start_background_processing(&self) -> Result<()> {
        // Start order expiry checker
        self.start_order_expiry_checker().await?;

        // Start fill processor
        self.start_fill_processor().await?;

        // Start event processor
        self.start_event_processor().await?;

        Ok(())
    }

    /// Start order expiry checker
    async fn start_order_expiry_checker(&self) -> Result<()> {
        let orders = self.orders.clone();
        let statistics = self.statistics.clone();
        let event_sender = self.event_sender.clone();
        let timeout_seconds = self.config.order_timeout_seconds;
        let max_age_hours = self.config.max_order_age_hours;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60)); // Check every minute

            loop {
                interval.tick().await;

                let now = Utc::now();
                let mut expired_orders = Vec::new();

                // Check for expired orders
                {
                    let mut orders_guard = orders.write().await;
                    for (order_id, order) in orders_guard.iter_mut() {
                        if matches!(order.status, OrderStatus::Pending | OrderStatus::PartiallyFilled) {
                            let age = now - order.created_at;

                            // Check timeout or max age
                            let should_expire = match order.time_in_force {
                                TimeInForce::IOC => age.num_seconds() > 1, // Immediate or cancel
                                TimeInForce::FOK => age.num_seconds() > 1, // Fill or kill
                                TimeInForce::DAY => age.num_hours() > 24,  // Day order
                                TimeInForce::GTC => age.num_hours() > max_age_hours as i64, // Good till cancel
                            } || age.num_seconds() > timeout_seconds as i64;

                            if should_expire {
                                order.status = OrderStatus::Expired;
                                order.updated_at = now;
                                expired_orders.push(*order_id);
                            }
                        }
                    }
                }

                // Update statistics and send events
                if !expired_orders.is_empty() {
                    {
                        let mut stats_guard = statistics.write().await;
                        stats_guard.active_orders = stats_guard.active_orders.saturating_sub(expired_orders.len() as u32);
                        stats_guard.expired_orders += expired_orders.len() as u32;
                    }

                    for order_id in expired_orders {
                        if let Err(e) = event_sender.send(OrderEvent::OrderExpired(order_id)) {
                            error!("Failed to send order expired event: {}", e);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Start fill processor
    async fn start_fill_processor(&self) -> Result<()> {
        let pending_fills = self.pending_fills.clone();
        let database = self.database.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_millis(100) // Process every 100ms
            );

            loop {
                interval.tick().await;

                // Process pending fills
                let fills_to_process = {
                    let mut pending_guard = pending_fills.write().await;
                    let mut fills = Vec::new();

                    // Take up to 100 fills at a time
                    for _ in 0..100 {
                        if let Some(fill) = pending_guard.pop_front() {
                            fills.push(fill);
                        } else {
                            break;
                        }
                    }

                    fills
                };

                // Store fills in database
                for fill in fills_to_process {
                    if let Err(e) = Self::store_fill_in_database(&database, &fill).await {
                        error!("Failed to store fill in database: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Start event processor
    async fn start_event_processor(&self) -> Result<()> {
        let mut receiver_guard = self.event_receiver.write().await;
        if let Some(mut receiver) = receiver_guard.take() {
            drop(receiver_guard);

            tokio::spawn(async move {
                while let Some(event) = receiver.recv().await {
                    // Process events (logging, notifications, etc.)
                    match event {
                        OrderEvent::OrderCreated(order) => {
                            debug!("Order created: {} for {}", order.id, order.instrument_id);
                        },
                        OrderEvent::OrderUpdated(order) => {
                            debug!("Order updated: {} status: {:?}", order.id, order.status);
                        },
                        OrderEvent::OrderFilled(fill) => {
                            info!("Order filled: {} quantity: {} price: {}",
                                  fill.order_id, fill.quantity, fill.price);
                        },
                        OrderEvent::OrderCancelled(order_id) => {
                            info!("Order cancelled: {}", order_id);
                        },
                        OrderEvent::OrderExpired(order_id) => {
                            warn!("Order expired: {}", order_id);
                        },
                        OrderEvent::OrderRejected(order_id, reason) => {
                            error!("Order rejected: {} reason: {}", order_id, reason);
                        },
                        OrderEvent::OrderBookUpdated(order_book) => {
                            debug!("Order book updated for instrument: {}", order_book.instrument_id);
                        },
                    }
                }
            });
        }

        Ok(())
    }

    /// Store fill in database
    async fn store_fill_in_database(database: &Database, fill: &Fill) -> Result<()> {
        use crate::database::query_functions::insert_fill;

        // Store the fill in database
        if let Err(e) = insert_fill(&database.pool, fill).await {
            error!("Failed to store fill in database: {}", e);
            return Err(e);
        }

        info!("Stored fill in database: {} {} @ {} (order: {})",
              fill.quantity, fill.price, fill.timestamp, fill.order_id);
        Ok(())
    }

    /// Send an event
    async fn send_event(&self, event: OrderEvent) {
        if let Err(e) = self.event_sender.send(event) {
            error!("Failed to send order event: {}", e);
        }
    }

    /// Clean up old orders and fills
    pub async fn cleanup_old_data(&self) -> Result<()> {
        let cutoff_time = Utc::now() - Duration::hours(self.config.max_order_age_hours as i64);

        // Clean up old orders
        {
            let mut orders_guard = self.orders.write().await;
            let mut history_guard = self.order_history.write().await;

            let old_orders: Vec<_> = orders_guard.iter()
                .filter(|(_, order)| order.updated_at < cutoff_time &&
                        matches!(order.status, OrderStatus::Filled | OrderStatus::Cancelled | OrderStatus::Expired))
                .map(|(&id, order)| (id, order.clone()))
                .collect();

            for (order_id, order) in old_orders {
                orders_guard.remove(&order_id);
                history_guard.push_back(order);

                // Keep history limited
                while history_guard.len() > 10000 {
                    history_guard.pop_front();
                }
            }
        }

        // Clean up old fills
        {
            let mut fill_history_guard = self.fill_history.write().await;
            while fill_history_guard.len() > 10000 {
                fill_history_guard.pop_front();
            }
        }

        info!("Cleaned up old order data");
        Ok(())
    }
}
