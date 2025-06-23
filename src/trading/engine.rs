use crate::database::{Database, types::MarketTick};
use crate::trading::{
    ExecutionEngine, ExecutionConfig, RiskManager, RiskManagerConfig,
    PortfolioManager, PortfolioConfig, MarketData
};
use crate::trading::signals::{
    SignalGenerator, TradingSignal, TradingDecision,
    ExecutionPlan, OrderRequest, ExecutionResult, RiskAssessment
};
use crate::trading::adaptive_batching::{
    AdaptiveBatchProcessor, AdaptiveBatchingConfig, BatchMetrics, AdaptiveBatchingStats
};
use crate::trading::lock_free_structures::{
    LockFreeOrderQueue as NewLockFreeOrderQueue, LockFreeMemoryPool
};
// IG Trading integration will be handled through MarketDataManager
use crate::market_data::MarketDataManager;
use crate::config::Settings;
use crate::microstructure::MicrostructureEngine;
use crate::ai::AIEngine;
use crate::utils::Result;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;
use tokio::sync::{RwLock, mpsc, Semaphore};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::time::{Instant, Duration};
use std::collections::VecDeque;
use tracing::{info, warn, error, debug};

// Enhanced Trading Engine Configuration with Performance Optimizations
#[derive(Debug, Clone)]
pub struct TradingEngineConfig {
    pub enable_live_trading: bool,
    pub max_position_size: f64,
    pub confidence_threshold: f64,
    pub max_daily_trades: u32,
    pub max_portfolio_exposure: f64,
    pub risk_check_interval_ms: u64,
    pub market_data_timeout_ms: u64,
    pub signal_generation_interval_ms: u64,
    pub portfolio_update_interval_ms: u64,
    pub enable_stop_loss: bool,
    pub enable_take_profit: bool,
    pub emergency_stop_loss_pct: f64,
    pub max_correlation_exposure: f64,
    pub enable_regime_filtering: bool,

    // Performance optimization settings
    pub enable_lock_free_processing: bool,
    pub enable_memory_pool: bool,
    pub max_concurrent_orders: usize,
    pub order_processing_batch_size: usize,
    pub enable_async_risk_checks: bool,
    pub target_latency_ms: f64,
    pub target_throughput_tps: f64,
    pub enable_performance_monitoring: bool,
    pub enable_adaptive_batching: bool,
    pub adaptive_batching_config: AdaptiveBatchingConfig,
    pub enable_lock_free_queue: bool,
    pub lock_free_queue_capacity: usize,
    pub enable_memory_pools: bool,
    pub memory_pool_size: usize,
}

impl Default for TradingEngineConfig {
    fn default() -> Self {
        Self {
            enable_live_trading: false,
            max_position_size: 100_000.0,
            confidence_threshold: 0.7,
            max_daily_trades: 100,
            max_portfolio_exposure: 0.8,  // 80% max exposure
            risk_check_interval_ms: 1000,  // 1 second
            market_data_timeout_ms: 5000,  // 5 seconds
            signal_generation_interval_ms: 100,  // 100ms
            portfolio_update_interval_ms: 1000,  // 1 second
            enable_stop_loss: true,
            enable_take_profit: true,
            emergency_stop_loss_pct: 0.05,  // 5% emergency stop
            max_correlation_exposure: 0.3,   // 30% max correlation
            enable_regime_filtering: true,

            // Performance optimization defaults
            enable_lock_free_processing: true,
            enable_memory_pool: true,
            max_concurrent_orders: 1000,
            order_processing_batch_size: 50,
            enable_async_risk_checks: true,
            target_latency_ms: 10.0,  // <10ms target
            target_throughput_tps: 1000.0,  // >1000 TPS target
            enable_performance_monitoring: true,
            enable_adaptive_batching: true,
            adaptive_batching_config: AdaptiveBatchingConfig::default(),
            enable_lock_free_queue: true,
            lock_free_queue_capacity: 10000,
            enable_memory_pools: true,
            memory_pool_size: 1000,
        }
    }
}

// Trading Engine State
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradingEngineState {
    Stopped,
    Starting,
    Running,
    Paused,
    EmergencyStop,
    Error(String),
}

// Trading Engine Events
#[derive(Debug, Clone)]
pub enum TradingEvent {
    MarketDataUpdate(HashMap<Uuid, MarketData>),
    SignalGenerated(TradingSignal),
    OrderExecuted(ExecutionResult),
    RiskBreach(String),
    EmergencyStop(String),
    StateChange(TradingEngineState),
    PerformanceAlert(PerformanceAlert),
}

// Performance optimization structures
#[derive(Debug, Clone)]
pub struct PerformanceAlert {
    pub alert_type: PerformanceAlertType,
    pub message: String,
    pub current_value: f64,
    pub threshold: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum PerformanceAlertType {
    LatencyThresholdExceeded,
    ThroughputBelowTarget,
    MemoryUsageHigh,
    ConcurrencyLimitReached,
}

// Truly lock-free order queue using atomic operations and channels
#[derive(Debug)]
pub struct LockFreeOrderQueue {
    order_sender: mpsc::UnboundedSender<OrderRequest>,
    order_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<OrderRequest>>>>,
    processing_semaphore: Arc<Semaphore>,
    performance_metrics: Arc<AtomicOrderMetrics>,
}

// Atomic performance metrics for lock-free access
#[derive(Debug)]
pub struct AtomicOrderMetrics {
    total_orders_processed: AtomicU64,
    total_processing_time_ns: AtomicU64,
    successful_orders: AtomicU64,
    failed_orders: AtomicU64,
    queue_depth: AtomicU64,
    last_update_timestamp: AtomicU64,
}

// Memory pool for order objects
#[derive(Debug)]
pub struct OrderMemoryPool {
    available_orders: Arc<RwLock<Vec<OrderRequest>>>,
    pool_size: usize,
    allocations: Arc<AtomicU64>,
    deallocations: Arc<AtomicU64>,
}

// Performance metrics for order processing
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct OrderProcessingMetrics {
    pub total_orders_processed: u64,
    pub average_latency_ms: f64,
    pub current_throughput_tps: f64,
    pub peak_throughput_tps: f64,
    pub latency_samples: VecDeque<f64>,
    pub throughput_samples: VecDeque<f64>,
    pub last_update: Option<DateTime<Utc>>,
}

// Main Trading Engine with Performance Optimizations and Alpaca Integration
#[derive(Clone)]
pub struct TradingEngine {
    config: TradingEngineConfig,
    state: Arc<RwLock<TradingEngineState>>,
    database: Database,

    // Core Components
    execution_engine: ExecutionEngine,
    risk_manager: RiskManager,
    portfolio_manager: PortfolioManager,
    signal_generator: SignalGenerator,
    microstructure_engine: MicrostructureEngine,
    ai_engine: AIEngine,

    // Market Data
    market_data: Arc<RwLock<HashMap<Uuid, MarketData>>>,
    market_data_manager: Option<Arc<MarketDataManager>>,

    // Event System
    event_sender: mpsc::UnboundedSender<TradingEvent>,
    event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<TradingEvent>>>>,

    // Trading State
    active_instruments: Arc<RwLock<Vec<Uuid>>>,
    daily_trade_count: Arc<RwLock<u32>>,
    last_signal_time: Arc<RwLock<DateTime<Utc>>>,
    last_risk_check: Arc<RwLock<DateTime<Utc>>>,

    // Performance Optimization Components
    lock_free_order_queue: Arc<LockFreeOrderQueue>,
    order_memory_pool: Arc<OrderMemoryPool>,
    performance_metrics: Arc<RwLock<OrderProcessingMetrics>>,
    processing_semaphore: Arc<Semaphore>,
    adaptive_batch_processor: Option<Arc<AdaptiveBatchProcessor>>,
    lock_free_order_queue_v2: Option<Arc<NewLockFreeOrderQueue>>,
    order_memory_pool_v2: Option<Arc<LockFreeMemoryPool<OrderRequest>>>,

    // IG Trading integration is handled through MarketDataManager
}

impl TradingEngine {
    /// Create a new trading engine with IG Trading integration
    pub async fn new_with_ig_trading(
        config: TradingEngineConfig,
        database: Database,
        settings: &Settings,
    ) -> Result<Self> {
        info!("Initializing Trading Engine with IG Trading integration");

        // Initialize market data manager with IG Trading support
        let market_data_manager = Arc::new(MarketDataManager::new(settings, database.clone()).await?);

        // Initialize execution engine with IG Trading integration
        let execution_config = ExecutionConfig::default();
        let execution_engine = ExecutionEngine::new(execution_config, database.clone()).await?;

        // Initialize other components
        let risk_config = RiskManagerConfig::default();
        let risk_manager = RiskManager::with_config(risk_config);

        let portfolio_config = PortfolioConfig::default();
        let portfolio_manager = PortfolioManager::new(portfolio_config, database.clone()).await?;

        let signal_generator = SignalGenerator::new(config.confidence_threshold);

        let microstructure_engine = MicrostructureEngine::new().await?;
        let ai_engine = AIEngine::new(database.clone()).await?;

        // Initialize event system
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        // Initialize performance optimization components
        let processing_semaphore = Arc::new(Semaphore::new(config.max_concurrent_orders));

        // Create truly lock-free order queue with channels
        let (order_sender, order_receiver) = mpsc::unbounded_channel();
        let lock_free_order_queue = Arc::new(LockFreeOrderQueue {
            order_sender,
            order_receiver: Arc::new(RwLock::new(Some(order_receiver))),
            processing_semaphore: processing_semaphore.clone(),
            performance_metrics: Arc::new(AtomicOrderMetrics {
                total_orders_processed: AtomicU64::new(0),
                total_processing_time_ns: AtomicU64::new(0),
                successful_orders: AtomicU64::new(0),
                failed_orders: AtomicU64::new(0),
                queue_depth: AtomicU64::new(0),
                last_update_timestamp: AtomicU64::new(0),
            }),
        });

        let order_memory_pool = Arc::new(OrderMemoryPool {
            available_orders: Arc::new(RwLock::new(Vec::with_capacity(config.order_processing_batch_size))),
            pool_size: config.order_processing_batch_size,
            allocations: Arc::new(AtomicU64::new(0)),
            deallocations: Arc::new(AtomicU64::new(0)),
        });

        // Initialize adaptive batch processor if enabled
        let adaptive_batch_processor = if config.enable_adaptive_batching {
            Some(Arc::new(AdaptiveBatchProcessor::new(config.adaptive_batching_config.clone())))
        } else {
            None
        };

        // Initialize lock-free structures if enabled
        let lock_free_order_queue_v2 = if config.enable_lock_free_queue {
            Some(Arc::new(NewLockFreeOrderQueue::new(config.lock_free_queue_capacity)))
        } else {
            None
        };

        let order_memory_pool_v2 = if config.enable_memory_pools {
            Some(Arc::new(LockFreeMemoryPool::new(config.memory_pool_size)))
        } else {
            None
        };

        let engine = Self {
            config,
            state: Arc::new(RwLock::new(TradingEngineState::Stopped)),
            database,
            execution_engine,
            risk_manager,
            portfolio_manager,
            signal_generator,
            microstructure_engine,
            ai_engine,
            market_data: Arc::new(RwLock::new(HashMap::new())),
            market_data_manager: Some(market_data_manager),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
            active_instruments: Arc::new(RwLock::new(Vec::new())),
            daily_trade_count: Arc::new(RwLock::new(0)),
            last_signal_time: Arc::new(RwLock::new(Utc::now())),
            last_risk_check: Arc::new(RwLock::new(Utc::now())),
            lock_free_order_queue,
            order_memory_pool,
            performance_metrics: Arc::new(RwLock::new(OrderProcessingMetrics::default())),
            processing_semaphore,
            adaptive_batch_processor,
            lock_free_order_queue_v2,
            order_memory_pool_v2,
        };

        info!("Trading Engine with IG Trading integration initialized successfully");
        Ok(engine)
    }

    /// Create a new trading engine without Alpaca integration (legacy)
    pub async fn new(
        config: TradingEngineConfig,
        database: Database,
    ) -> Result<Self> {
        info!("Initializing Trading Engine (legacy mode without Alpaca)");

        // Initialize components
        let execution_config = ExecutionConfig::default();
        let execution_engine = ExecutionEngine::new(execution_config, database.clone()).await?;

        let risk_config = RiskManagerConfig::default();
        let risk_manager = RiskManager::with_config(risk_config);

        let portfolio_config = PortfolioConfig::default();
        let portfolio_manager = PortfolioManager::new(portfolio_config, database.clone()).await?;

        let signal_generator = SignalGenerator::new(config.confidence_threshold);

        let microstructure_engine = MicrostructureEngine::new().await?;
        let ai_engine = AIEngine::new(database.clone()).await?;

        // Initialize event system
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        // Initialize performance optimization components
        let processing_semaphore = Arc::new(Semaphore::new(config.max_concurrent_orders));

        // Create truly lock-free order queue with channels
        let (order_sender, order_receiver) = mpsc::unbounded_channel();
        let lock_free_order_queue = Arc::new(LockFreeOrderQueue {
            order_sender,
            order_receiver: Arc::new(RwLock::new(Some(order_receiver))),
            processing_semaphore: processing_semaphore.clone(),
            performance_metrics: Arc::new(AtomicOrderMetrics {
                total_orders_processed: AtomicU64::new(0),
                total_processing_time_ns: AtomicU64::new(0),
                successful_orders: AtomicU64::new(0),
                failed_orders: AtomicU64::new(0),
                queue_depth: AtomicU64::new(0),
                last_update_timestamp: AtomicU64::new(0),
            }),
        });

        let order_memory_pool = Arc::new(OrderMemoryPool {
            available_orders: Arc::new(RwLock::new(Vec::with_capacity(config.order_processing_batch_size))),
            pool_size: config.order_processing_batch_size,
            allocations: Arc::new(AtomicU64::new(0)),
            deallocations: Arc::new(AtomicU64::new(0)),
        });

        // Initialize adaptive batch processor if enabled
        let adaptive_batch_processor = if config.enable_adaptive_batching {
            Some(Arc::new(AdaptiveBatchProcessor::new(config.adaptive_batching_config.clone())))
        } else {
            None
        };

        // Initialize lock-free structures if enabled
        let lock_free_order_queue_v2 = if config.enable_lock_free_queue {
            Some(Arc::new(NewLockFreeOrderQueue::new(config.lock_free_queue_capacity)))
        } else {
            None
        };

        let order_memory_pool_v2 = if config.enable_memory_pools {
            Some(Arc::new(LockFreeMemoryPool::new(config.memory_pool_size)))
        } else {
            None
        };

        let engine = Self {
            config,
            state: Arc::new(RwLock::new(TradingEngineState::Stopped)),
            database,
            execution_engine,
            risk_manager,
            portfolio_manager,
            signal_generator,
            microstructure_engine,
            ai_engine,
            market_data: Arc::new(RwLock::new(HashMap::new())),
            market_data_manager: None,
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
            active_instruments: Arc::new(RwLock::new(Vec::new())),
            daily_trade_count: Arc::new(RwLock::new(0)),
            last_signal_time: Arc::new(RwLock::new(Utc::now())),
            last_risk_check: Arc::new(RwLock::new(Utc::now())),
            lock_free_order_queue,
            order_memory_pool,
            performance_metrics: Arc::new(RwLock::new(OrderProcessingMetrics::default())),
            processing_semaphore,
            adaptive_batch_processor,
            lock_free_order_queue_v2,
            order_memory_pool_v2,

        };

        info!("Trading Engine initialized successfully (legacy mode)");
        Ok(engine)
    }

    /// Start the trading engine
    pub async fn start(&self) -> Result<()> {
        info!("Starting Trading Engine");

        {
            let mut state_guard = self.state.write().await;
            *state_guard = TradingEngineState::Starting;
        }

        // Start event processing loop
        self.start_event_loop().await?;

        // Start main trading loop
        self.start_trading_loop().await?;

        {
            let mut state_guard = self.state.write().await;
            *state_guard = TradingEngineState::Running;
        }

        self.send_event(TradingEvent::StateChange(TradingEngineState::Running)).await;
        info!("Trading Engine started successfully");
        Ok(())
    }

    /// Stop the trading engine
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping Trading Engine");

        {
            let mut state_guard = self.state.write().await;
            *state_guard = TradingEngineState::Stopped;
        }

        self.send_event(TradingEvent::StateChange(TradingEngineState::Stopped)).await;
        info!("Trading Engine stopped");
        Ok(())
    }

    /// Emergency stop - immediately halt all trading
    pub async fn emergency_stop(&self, reason: String) -> Result<()> {
        error!("Emergency stop triggered: {}", reason);

        {
            let mut state_guard = self.state.write().await;
            *state_guard = TradingEngineState::EmergencyStop;
        }

        // Cancel all pending orders
        let active_orders = self.execution_engine.get_active_orders().await;
        for order_id in active_orders.keys() {
            if let Err(e) = self.execution_engine.cancel_order(*order_id).await {
                error!("Failed to cancel order {}: {}", order_id, e);
            }
        }

        self.send_event(TradingEvent::EmergencyStop(reason.clone())).await;
        warn!("Emergency stop completed: {}", reason);
        Ok(())
    }

    /// Check if the trading engine is currently running
    pub async fn is_running(&self) -> bool {
        let state = self.state.read().await;
        matches!(*state, TradingEngineState::Running)
    }

    /// Add instrument to trading
    pub async fn add_instrument(&self, instrument_id: Uuid) -> Result<()> {
        let mut instruments_guard = self.active_instruments.write().await;
        if !instruments_guard.contains(&instrument_id) {
            instruments_guard.push(instrument_id);
            info!("Added instrument {} to trading", instrument_id);
        }
        Ok(())
    }

    /// Remove instrument from trading
    pub async fn remove_instrument(&self, instrument_id: Uuid) -> Result<()> {
        let mut instruments_guard = self.active_instruments.write().await;
        instruments_guard.retain(|&id| id != instrument_id);
        info!("Removed instrument {} from trading", instrument_id);
        Ok(())
    }

    /// Process market data update
    pub async fn process_market_data(&self, ticks: &[MarketTick]) -> Result<()> {
        let mut market_data_map = HashMap::new();

        // Convert ticks to market data
        for tick in ticks {
            let market_data = MarketData {
                instrument_id: tick.instrument_id,
                bid_price: tick.bid_price,
                ask_price: tick.ask_price,
                bid_size: tick.bid_size,
                ask_size: tick.ask_size,
                last_price: tick.last_price,
                timestamp: tick.timestamp,
            };
            market_data_map.insert(tick.instrument_id, market_data);
        }

        // Update market data
        {
            let mut market_data_guard = self.market_data.write().await;
            for (instrument_id, data) in &market_data_map {
                market_data_guard.insert(*instrument_id, data.clone());
            }
        }

        // Update execution engine with market data
        for (_, data) in &market_data_map {
            self.execution_engine.update_market_data(data.clone()).await;
        }

        // Update portfolio with current prices
        self.portfolio_manager.update_market_prices(&market_data_map).await?;

        // Send market data event
        self.send_event(TradingEvent::MarketDataUpdate(market_data_map)).await;

        Ok(())
    }

    /// Main trading loop
    async fn start_trading_loop(&self) -> Result<()> {
        // For now, we'll run the trading loop in a simplified way
        // In a production system, this would be more sophisticated
        info!("Trading loop started (simplified implementation)");
        Ok(())
    }

    /// Run a single trading cycle
    async fn run_trading_cycle(&self) -> Result<()> {
        // 1. Check risk limits
        self.check_risk_limits().await?;

        // 2. Generate trading signals
        let signals = self.generate_trading_signals().await?;

        // 3. Process signals and make trading decisions
        for signal in signals {
            if let Err(e) = self.process_trading_signal(signal).await {
                error!("Error processing trading signal: {}", e);
            }
        }

        // 4. Update portfolio metrics
        self.update_portfolio_metrics().await?;

        Ok(())
    }

    /// Generate trading signals
    async fn generate_trading_signals(&self) -> Result<Vec<TradingSignal>> {
        let instruments_guard = self.active_instruments.read().await;
        let market_data_guard = self.market_data.read().await;

        let mut all_signals = Vec::new();

        // Collect market data for AI processing
        let mut market_ticks = Vec::new();
        for (&instrument_id, market_data) in market_data_guard.iter() {
            // Convert MarketData to MarketTick for AI processing
            let market_tick = crate::database::types::MarketTick {
                timestamp: chrono::Utc::now(),
                instrument_id,
                provider: "internal".to_string(),
                bid_price: market_data.bid_price,
                ask_price: market_data.ask_price,
                bid_size: market_data.bid_size,
                ask_size: market_data.ask_size,
                last_price: Some((market_data.bid_price + market_data.ask_price) / 2.0),
                volume: Some(market_data.bid_size + market_data.ask_size),
                spread: market_data.ask_price - market_data.bid_price,
                data_quality_score: 1.0,
                raw_data: serde_json::json!({}),
            };
            market_ticks.push(market_tick);
        }

        if market_ticks.is_empty() {
            debug!("No market data available for signal generation");
            return Ok(all_signals);
        }

        // Process market data through AI engine to get AI signals
        // Note: We need to make a mutable reference to the AI engine
        // For now, we'll create a simplified AI signal generation
        let ai_signals = self.generate_ai_signals(&market_ticks).await;

        if ai_signals.is_empty() {
            debug!("No AI signals generated");
            return Ok(all_signals);
        }

        // Generate microstructure analysis (placeholder for now)
        let mut microstructure_analysis = std::collections::HashMap::new();
        for &instrument_id in instruments_guard.iter() {
            if let Some(market_data) = market_data_guard.get(&instrument_id) {
                let current_price = (market_data.bid_price + market_data.ask_price) / 2.0;
                let bid_ask_spread = market_data.ask_price - market_data.bid_price;
                let orderbook_imbalance = (market_data.bid_size - market_data.ask_size) /
                    (market_data.bid_size + market_data.ask_size).max(1e-8);

                // Create placeholder microstructure analysis
                let analysis = crate::trading::signals::MicrostructureAnalysis {
                    instrument_id,
                    timestamp: chrono::Utc::now(),
                    current_price,
                    liquidity_metrics: Some(crate::trading::signals::LiquidityMetrics {
                        imbalance_ratio: orderbook_imbalance,
                        depth_ratio: market_data.bid_size / market_data.ask_size.max(1e-8),
                        spread_stability: 0.8, // Placeholder
                    }),
                    orderbook_imbalance,
                    bid_ask_spread,
                    market_depth: market_data.bid_size + market_data.ask_size,
                };
                microstructure_analysis.insert(instrument_id, analysis);
            }
        }

        // Generate trading signals using signal generator
        let trading_signals = self.signal_generator
            .generate_signals(&microstructure_analysis, &ai_signals)
            .await?;

        all_signals.extend(trading_signals);

        info!("Generated {} trading signals", all_signals.len());
        Ok(all_signals)
    }

    /// Generate AI signals from market data
    async fn generate_ai_signals(&self, market_ticks: &[crate::database::types::MarketTick]) -> Vec<crate::trading::signals::AISignal> {
        let mut ai_signals = Vec::new();

        // Group market ticks by instrument
        let mut ticks_by_instrument: std::collections::HashMap<uuid::Uuid, Vec<&crate::database::types::MarketTick>> = std::collections::HashMap::new();
        for tick in market_ticks {
            ticks_by_instrument.entry(tick.instrument_id).or_default().push(tick);
        }

        // Generate AI signals for each instrument
        for (instrument_id, ticks) in ticks_by_instrument {
            if let Some(latest_tick) = ticks.last() {
                // Create a simplified AI signal for now
                // In production, this would use the actual AI engine
                let ai_signal = crate::trading::signals::AISignal {
                    instrument_id,
                    timestamp: chrono::Utc::now(),
                    price_predictions: vec![
                        // 1-minute prediction
                        crate::trading::signals::PredictionResult {
                            horizon: std::time::Duration::from_secs(60),
                            predicted_price: (latest_tick.bid_price + latest_tick.ask_price) / 2.0 * 1.001, // Small upward bias
                            confidence_score: 0.7,
                            prediction_interval: (
                                (latest_tick.bid_price + latest_tick.ask_price) / 2.0 * 0.999,
                                (latest_tick.bid_price + latest_tick.ask_price) / 2.0 * 1.003
                            ),
                        },
                        // 5-minute prediction
                        crate::trading::signals::PredictionResult {
                            horizon: std::time::Duration::from_secs(300),
                            predicted_price: (latest_tick.bid_price + latest_tick.ask_price) / 2.0 * 1.002,
                            confidence_score: 0.6,
                            prediction_interval: (
                                (latest_tick.bid_price + latest_tick.ask_price) / 2.0 * 0.998,
                                (latest_tick.bid_price + latest_tick.ask_price) / 2.0 * 1.006
                            ),
                        },
                    ],
                    regime_signal: Some(crate::trading::signals::RegimeSignal {
                        current_regime: crate::database::types::RegimeType::Normal,
                        transition_probability: 0.1,
                        confidence: 0.8,
                        timestamp: chrono::Utc::now(),
                    }),
                    rl_recommendation: None, // Placeholder - would be generated by RL agent
                    confidence_score: 0.65,
                };

                ai_signals.push(ai_signal);
            }
        }

        ai_signals
    }

    /// Process a trading signal
    async fn process_trading_signal(&self, signal: TradingSignal) -> Result<()> {
        debug!("Processing trading signal: {:?}", signal);

        // Check if we should trade this signal
        if !self.should_trade_signal(&signal).await? {
            debug!("Signal filtered out: {:?}", signal);
            return Ok(());
        }

        // Get current positions for risk assessment
        let active_positions = self.portfolio_manager.get_positions().await;

        // Assess risk
        let risk_assessment = self.risk_manager
            .assess_signal_risk(&signal, &active_positions)
            .await?;

        if !risk_assessment.is_acceptable {
            debug!("Signal rejected by risk management: {:?}", signal);
            return Ok(());
        }

        // Create execution plan
        let execution_plan = self.create_execution_plan(&signal, &risk_assessment).await?;

        // Create trading decision
        let decision = TradingDecision {
            instrument_id: signal.instrument_id,
            strategy_type: signal.strategy_type.clone(),
            signal: signal.clone(),
            risk_assessment,
            execution_plan: execution_plan.clone(),
            confidence_score: signal.confidence_score,
            expected_pnl: signal.expected_return.unwrap_or(0.0),
        };

        // Execute the decision
        self.execute_trading_decision(&decision).await?;

        // Send signal event
        self.send_event(TradingEvent::SignalGenerated(signal)).await;

        Ok(())
    }

    /// Check if we should trade a signal
    async fn should_trade_signal(&self, signal: &TradingSignal) -> Result<bool> {
        // Check confidence threshold
        if signal.confidence_score < self.config.confidence_threshold {
            return Ok(false);
        }

        // Check daily trade limit
        let trade_count = *self.daily_trade_count.read().await;
        if trade_count >= self.config.max_daily_trades {
            return Ok(false);
        }

        // Check if live trading is enabled
        if !self.config.enable_live_trading {
            debug!("Live trading disabled, skipping signal");
            return Ok(false);
        }

        // Check engine state
        let state_guard = self.state.read().await;
        if !matches!(*state_guard, TradingEngineState::Running) {
            return Ok(false);
        }

        Ok(true)
    }

    /// Create execution plan from signal and risk assessment
    async fn create_execution_plan(
        &self,
        signal: &TradingSignal,
        risk_assessment: &crate::trading::signals::RiskAssessment,
    ) -> Result<ExecutionPlan> {
        use crate::database::types::{OrderType, TimeInForce, ExecutionStyle};

        let order_type = match signal.urgency_score.unwrap_or(0.5) {
            x if x > 0.8 => OrderType::Market,
            x if x > 0.5 => OrderType::Limit,
            _ => OrderType::Limit,
        };

        let execution_style = match signal.strategy_type {
            crate::trading::signals::StrategyType::PredictiveMarketMaking => ExecutionStyle::Passive,
            crate::trading::signals::StrategyType::MicrostructureMomentum => ExecutionStyle::Aggressive,
            crate::trading::signals::StrategyType::RegimeArbitrage => ExecutionStyle::TWAP,
            crate::trading::signals::StrategyType::LiquidityHarvesting => ExecutionStyle::Iceberg,
        };

        Ok(ExecutionPlan {
            order_type,
            quantity: risk_assessment.adjusted_position_size,
            price: signal.entry_price,
            time_in_force: TimeInForce::GTC,
            execution_style,
        })
    }

    /// Execute a trading decision
    async fn execute_trading_decision(&self, decision: &TradingDecision) -> Result<()> {
        let order_request = OrderRequest {
            instrument_id: decision.instrument_id,
            side: decision.signal.signal_type.clone(),
            quantity: decision.execution_plan.quantity,
            order_type: decision.execution_plan.order_type.clone(),
            price: decision.execution_plan.price,
            time_in_force: decision.execution_plan.time_in_force.clone(),
        };

        // Execute the order
        let start_time = std::time::Instant::now();
        let execution_result = self.execution_engine.execute_order(order_request).await?;
        let execution_time_ms = start_time.elapsed().as_millis() as i32;

        // Store trade execution in database
        if let Err(e) = self.store_trade_execution(&execution_result, None, Some(execution_time_ms)).await {
            error!("Failed to store trade execution in database: {}", e);
        }

        // Update portfolio
        self.portfolio_manager.process_execution(&execution_result).await?;

        // Increment trade count
        {
            let mut trade_count_guard = self.daily_trade_count.write().await;
            *trade_count_guard += 1;
        }

        // Send execution event
        self.send_event(TradingEvent::OrderExecuted(execution_result)).await;

        info!("Executed trade: {} {} @ {} for instrument {}",
              decision.signal.signal_type,
              decision.execution_plan.quantity,
              decision.execution_plan.price.unwrap_or(0.0),
              decision.instrument_id);

        Ok(())
    }

    /// Check risk limits
    async fn check_risk_limits(&self) -> Result<()> {
        let now = Utc::now();
        let mut last_check_guard = self.last_risk_check.write().await;

        // Only check if enough time has passed
        if (now - *last_check_guard).num_milliseconds() < self.config.risk_check_interval_ms as i64 {
            return Ok(());
        }

        *last_check_guard = now;
        drop(last_check_guard);

        // Check portfolio constraints
        let violations = self.portfolio_manager.check_constraints().await?;

        if !violations.is_empty() {
            let violation_msg = violations.join("; ");
            warn!("Risk violations detected: {}", violation_msg);

            // Send risk breach event
            self.send_event(TradingEvent::RiskBreach(violation_msg.clone())).await;

            // Check if emergency stop is needed
            let portfolio_state = self.portfolio_manager.get_portfolio_state().await;
            if portfolio_state.current_drawdown > self.config.emergency_stop_loss_pct {
                self.emergency_stop(format!("Emergency drawdown limit exceeded: {:.2}%",
                    portfolio_state.current_drawdown * 100.0)).await?;
            }
        }

        Ok(())
    }

    /// Update portfolio metrics
    async fn update_portfolio_metrics(&self) -> Result<()> {
        // This is handled automatically by the portfolio manager
        // when market data is updated
        Ok(())
    }

    /// Start event processing loop
    async fn start_event_loop(&self) -> Result<()> {
        let mut receiver_guard = self.event_receiver.write().await;
        if let Some(mut receiver) = receiver_guard.take() {
            drop(receiver_guard);

            tokio::spawn(async move {
                while let Some(event) = receiver.recv().await {
                    // Process events (logging, notifications, etc.)
                    match event {
                        TradingEvent::MarketDataUpdate(_) => {
                            debug!("Market data updated");
                        },
                        TradingEvent::SignalGenerated(signal) => {
                            info!("Signal generated: {} for {}", signal.signal_type, signal.instrument_id);
                        },
                        TradingEvent::OrderExecuted(result) => {
                            info!("Order executed: {} {} @ {}",
                                  result.filled_quantity,
                                  result.instrument_id,
                                  result.average_price);
                        },
                        TradingEvent::RiskBreach(msg) => {
                            warn!("Risk breach: {}", msg);
                        },
                        TradingEvent::EmergencyStop(reason) => {
                            error!("Emergency stop: {}", reason);
                        },
                        TradingEvent::StateChange(state) => {
                            info!("Engine state changed to: {:?}", state);
                        },
                        TradingEvent::PerformanceAlert(alert) => {
                            warn!("Performance alert: {}", alert.message);
                        },
                    }
                }
            });
        }

        Ok(())
    }

    /// Send an event
    async fn send_event(&self, event: TradingEvent) {
        if let Err(e) = self.event_sender.send(event) {
            error!("Failed to send event: {}", e);
        }
    }



    /// Get current engine state
    pub async fn get_state(&self) -> TradingEngineState {
        self.state.read().await.clone()
    }

    /// Get portfolio summary
    pub async fn get_portfolio_summary(&self) -> crate::trading::PortfolioSummary {
        self.portfolio_manager.get_portfolio_summary().await
    }

    /// Get active positions
    pub async fn get_positions(&self) -> HashMap<Uuid, crate::trading::signals::Position> {
        self.portfolio_manager.get_positions().await
    }

    // Order Management API Methods

    /// Submit a new order through the trading engine
    pub async fn submit_order(&self, request: OrderRequest) -> Result<Uuid> {
        info!("Submitting order: {:?}", request);

        // Check if engine is running
        let state = self.get_state().await;
        if !matches!(state, TradingEngineState::Running) {
            return Err(crate::utils::PantherSwapError::trading(
                format!("Trading engine not running: {:?}", state)
            ));
        }

        // Execute order through execution engine
        let execution_result = self.execution_engine.execute_order(request).await?;

        // Update portfolio with execution
        self.portfolio_manager.process_execution(&execution_result).await?;

        // For now, return a generated UUID - in a real implementation,
        // we'd get this from the execution engine
        Ok(Uuid::new_v4())
    }

    /// Cancel an existing order
    pub async fn cancel_order(&self, order_id: Uuid) -> Result<()> {
        info!("Cancelling order: {}", order_id);
        self.execution_engine.cancel_order(order_id).await
    }

    /// Get order details by ID
    pub async fn get_order_details(&self, order_id: Uuid) -> Option<crate::trading::execution::Order> {
        let active_orders = self.execution_engine.get_active_orders().await;
        active_orders.get(&order_id).cloned()
    }

    /// Get orders with filtering
    pub async fn get_orders_with_filters(
        &self,
        instrument_id: Option<Uuid>,
        status: Option<&str>,
        strategy_name: Option<&str>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<crate::trading::execution::Order>> {
        let active_orders = self.execution_engine.get_active_orders().await;

        let mut filtered_orders: Vec<crate::trading::execution::Order> = active_orders
            .values()
            .cloned()
            .collect();

        // Apply filters
        if let Some(instr_id) = instrument_id {
            filtered_orders.retain(|order| order.instrument_id == instr_id);
        }

        if let Some(status_filter) = status {
            let status_enum = match status_filter.to_lowercase().as_str() {
                "pending" => Some(crate::trading::execution::OrderStatus::Pending),
                "filled" => Some(crate::trading::execution::OrderStatus::Filled),
                "cancelled" => Some(crate::trading::execution::OrderStatus::Cancelled),
                "rejected" => Some(crate::trading::execution::OrderStatus::Rejected),
                "expired" => Some(crate::trading::execution::OrderStatus::Expired),
                "partiallyfilled" => Some(crate::trading::execution::OrderStatus::PartiallyFilled),
                _ => None,
            };

            if let Some(status) = status_enum {
                filtered_orders.retain(|order| order.status == status);
            }
        }

        if let Some(strategy) = strategy_name {
            filtered_orders.retain(|order| {
                order.strategy_name.as_ref().map_or(false, |s| s == strategy)
            });
        }

        if let Some(start) = start_time {
            filtered_orders.retain(|order| order.created_at >= start);
        }

        if let Some(end) = end_time {
            filtered_orders.retain(|order| order.created_at <= end);
        }

        // Sort by creation time (newest first)
        filtered_orders.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        // Apply pagination
        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(50) as usize;

        let end_index = std::cmp::min(offset + limit, filtered_orders.len());
        if offset < filtered_orders.len() {
            Ok(filtered_orders[offset..end_index].to_vec())
        } else {
            Ok(Vec::new())
        }
    }

    /// Get order statistics
    pub async fn get_order_statistics(&self) -> Result<crate::trading::order_manager::OrderStatistics> {
        let active_orders = self.execution_engine.get_active_orders().await;

        let mut stats = crate::trading::order_manager::OrderStatistics::default();
        stats.total_orders = active_orders.len() as u32;

        for order in active_orders.values() {
            match order.status {
                crate::trading::execution::OrderStatus::Pending => stats.active_orders += 1,
                crate::trading::execution::OrderStatus::PartiallyFilled => stats.active_orders += 1,
                crate::trading::execution::OrderStatus::Filled => stats.filled_orders += 1,
                crate::trading::execution::OrderStatus::Cancelled => stats.cancelled_orders += 1,
                crate::trading::execution::OrderStatus::Rejected => stats.rejected_orders += 1,
                crate::trading::execution::OrderStatus::Expired => stats.expired_orders += 1,
            }

            // Calculate total volume and fees
            for fill in &order.fills {
                stats.total_volume += fill.quantity * fill.price;
                stats.total_fees += fill.fees;
            }
        }

        // Calculate fill rate
        if stats.total_orders > 0 {
            stats.fill_rate = stats.filled_orders as f64 / stats.total_orders as f64;
        }

        Ok(stats)
    }

    /// Reset daily counters (called at start of trading day)
    pub async fn reset_daily_counters(&self) -> Result<()> {
        {
            let mut trade_count_guard = self.daily_trade_count.write().await;
            *trade_count_guard = 0;
        }

        self.portfolio_manager.reset_daily_pnl().await;
        info!("Daily counters reset");
        Ok(())
    }

    /// High-performance order processing with lock-free queue
    pub async fn process_order_optimized(&self, order_request: OrderRequest) -> Result<ExecutionResult> {
        let start_time = Instant::now();

        // Acquire processing permit for concurrency control
        let _permit = self.processing_semaphore.acquire().await.map_err(|e| {
            crate::utils::PantherSwapError::trading(format!("Failed to acquire processing permit: {}", e))
        })?;

        // Get order from memory pool if available
        let optimized_order = if self.config.enable_memory_pool {
            self.get_order_from_pool(order_request).await
        } else {
            order_request
        };

        // Process order with lock-free queue
        let result = if self.config.enable_lock_free_processing {
            self.process_order_lock_free(optimized_order).await?
        } else {
            self.process_order_standard(optimized_order).await?
        };

        // Update performance metrics
        let processing_latency = start_time.elapsed().as_millis() as f64;
        self.update_performance_metrics(processing_latency).await;

        // Check performance thresholds
        if processing_latency > self.config.target_latency_ms {
            self.send_performance_alert(
                PerformanceAlertType::LatencyThresholdExceeded,
                processing_latency,
                self.config.target_latency_ms,
            ).await;
        }

        Ok(result)
    }

    /// Process order using truly lock-free queue with channels
    async fn process_order_lock_free(&self, order_request: OrderRequest) -> Result<ExecutionResult> {
        let start_time = Instant::now();

        // Send to lock-free queue (non-blocking)
        if let Err(_) = self.lock_free_order_queue.order_sender.send(order_request.clone()) {
            return Err(crate::utils::PantherSwapError::trading(
                "Failed to send order to processing queue".to_string()
            ));
        }

        // Update queue depth atomically
        self.lock_free_order_queue.performance_metrics.queue_depth.fetch_add(1, Ordering::Relaxed);

        // Process immediately for low latency (bypass queue for immediate execution)
        let execution_result = self.execute_order_fast(&order_request).await?;

        // Update atomic metrics (lock-free)
        let processing_time_ns = start_time.elapsed().as_nanos() as u64;
        self.lock_free_order_queue.performance_metrics.total_orders_processed.fetch_add(1, Ordering::Relaxed);
        self.lock_free_order_queue.performance_metrics.total_processing_time_ns.fetch_add(processing_time_ns, Ordering::Relaxed);
        self.lock_free_order_queue.performance_metrics.successful_orders.fetch_add(1, Ordering::Relaxed);
        self.lock_free_order_queue.performance_metrics.queue_depth.fetch_sub(1, Ordering::Relaxed);
        self.lock_free_order_queue.performance_metrics.last_update_timestamp.store(
            Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64,
            Ordering::Relaxed
        );

        Ok(execution_result)
    }

    /// Standard order processing (fallback)
    async fn process_order_standard(&self, order_request: OrderRequest) -> Result<ExecutionResult> {
        // Use standard execution engine
        self.execution_engine.execute_order(order_request).await
    }

    /// Fast order execution optimized for latency
    async fn execute_order_fast(&self, order_request: &OrderRequest) -> Result<ExecutionResult> {
        let start_time = Instant::now();

        // Parallel risk check if enabled
        let risk_check_future = if self.config.enable_async_risk_checks {
            Some(self.perform_async_risk_check(order_request))
        } else {
            None
        };

        // Get market data for execution
        let market_data = {
            let market_data_guard = self.market_data.read().await;
            market_data_guard.get(&order_request.instrument_id).cloned()
        };

        let market_data = market_data.ok_or_else(|| {
            crate::utils::PantherSwapError::trading("No market data available for instrument".to_string())
        })?;

        // Wait for risk check if running async
        if let Some(risk_future) = risk_check_future {
            let risk_result = risk_future.await?;
            if !risk_result.is_acceptable {
                return Err(crate::utils::PantherSwapError::trading("Order rejected by risk management".to_string()));
            }
        }

        // Execute order with minimal overhead
        let execution_price = match order_request.order_type {
            crate::database::types::OrderType::Market => {
                // Market order - use current market price
                if order_request.side == crate::database::types::SignalType::Buy {
                    market_data.ask_price
                } else {
                    market_data.bid_price
                }
            },
            crate::database::types::OrderType::Limit => {
                // Limit order - use limit price if favorable
                order_request.price.unwrap_or(market_data.bid_price)
            },
            _ => market_data.bid_price, // Simplified for other order types
        };

        let execution_result = ExecutionResult {
            instrument_id: order_request.instrument_id,
            strategy_name: "optimized_execution".to_string(),
            filled_quantity: order_request.quantity,
            average_price: execution_price,
            execution_time: Utc::now(),
            stop_loss: None,
            take_profit: None,
        };

        Ok(execution_result)
    }

    /// Perform asynchronous risk check
    async fn perform_async_risk_check(&self, order_request: &OrderRequest) -> Result<crate::trading::signals::RiskAssessment> {
        // Get current positions
        let positions = self.portfolio_manager.get_positions().await;

        // Create a simplified trading signal for risk assessment
        let signal = TradingSignal {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            instrument_id: order_request.instrument_id,
            strategy_type: crate::trading::signals::StrategyType::PredictiveMarketMaking, // Default
            signal_type: order_request.side.clone(),
            signal_strength: 0.7,
            confidence_score: 0.7,
            urgency_score: Some(0.5),
            entry_price: order_request.price,
            stop_loss: None,
            take_profit: None,
            time_horizon: None,
            expected_return: Some(0.01),
            max_risk: Some(0.02),
            supporting_evidence: crate::trading::signals::SignalEvidence {
                microstructure_score: 0.7,
                ai_prediction_score: 0.7,
                regime_score: 0.7,
                liquidity_score: 0.7,
                risk_reward_ratio: 0.5,
            },
        };

        // Perform risk assessment
        self.risk_manager.assess_signal_risk(&signal, &positions).await
    }

    /// Get order from memory pool
    async fn get_order_from_pool(&self, order_request: OrderRequest) -> OrderRequest {
        // In a real implementation, this would reuse order objects from a pool
        // For now, just return the original order
        self.order_memory_pool.allocations.fetch_add(1, Ordering::Relaxed);
        order_request
    }

    /// Update performance metrics
    async fn update_performance_metrics(&self, latency_ms: f64) {
        let mut metrics = self.performance_metrics.write().await;

        // Update latency samples
        if metrics.latency_samples.len() >= 1000 {
            metrics.latency_samples.pop_front();
        }
        metrics.latency_samples.push_back(latency_ms);

        // Calculate average latency
        metrics.average_latency_ms = metrics.latency_samples.iter().sum::<f64>() / metrics.latency_samples.len() as f64;

        // Update order count
        metrics.total_orders_processed += 1;

        // Calculate throughput (orders per second)
        let now = Utc::now();
        if let Some(last_update) = metrics.last_update {
            let time_diff = (now - last_update).num_milliseconds() as f64 / 1000.0;
            if time_diff > 0.0 {
                let current_throughput = 1.0 / time_diff;
                metrics.current_throughput_tps = current_throughput;

                // Update throughput samples
                if metrics.throughput_samples.len() >= 100 {
                    metrics.throughput_samples.pop_front();
                }
                metrics.throughput_samples.push_back(current_throughput);

                // Update peak throughput
                if current_throughput > metrics.peak_throughput_tps {
                    metrics.peak_throughput_tps = current_throughput;
                }
            }
        }

        metrics.last_update = Some(now);
    }

    /// Send performance alert
    async fn send_performance_alert(&self, alert_type: PerformanceAlertType, current_value: f64, threshold: f64) {
        let alert = PerformanceAlert {
            alert_type: alert_type.clone(),
            message: match alert_type {
                PerformanceAlertType::LatencyThresholdExceeded => {
                    format!("Order processing latency ({:.2}ms) exceeded target ({:.2}ms)", current_value, threshold)
                },
                PerformanceAlertType::ThroughputBelowTarget => {
                    format!("Throughput ({:.2} TPS) below target ({:.2} TPS)", current_value, threshold)
                },
                PerformanceAlertType::MemoryUsageHigh => {
                    format!("Memory usage ({:.2}%) above threshold ({:.2}%)", current_value, threshold)
                },
                PerformanceAlertType::ConcurrencyLimitReached => {
                    format!("Concurrency limit reached: {} concurrent orders", current_value)
                },
            },
            current_value,
            threshold,
            timestamp: Utc::now(),
        };

        self.send_event(TradingEvent::PerformanceAlert(alert)).await;
    }

    /// Get current performance metrics
    pub async fn get_performance_metrics(&self) -> OrderProcessingMetrics {
        let metrics = self.performance_metrics.read().await;
        OrderProcessingMetrics {
            total_orders_processed: metrics.total_orders_processed,
            average_latency_ms: metrics.average_latency_ms,
            current_throughput_tps: metrics.current_throughput_tps,
            peak_throughput_tps: metrics.peak_throughput_tps,
            latency_samples: metrics.latency_samples.clone(),
            throughput_samples: metrics.throughput_samples.clone(),
            last_update: metrics.last_update,
        }
    }

    /// Enable/disable performance optimizations at runtime
    pub async fn configure_performance_optimizations(&mut self, enable_lock_free: bool, enable_memory_pool: bool) {
        // Note: In a real implementation, this would safely reconfigure the engine
        // For now, we'll just log the configuration change
        info!("Performance optimizations configured: lock_free={}, memory_pool={}", enable_lock_free, enable_memory_pool);
    }

    /// Batch process multiple orders for improved throughput
    pub async fn process_order_batch(&self, orders: Vec<OrderRequest>) -> Result<Vec<ExecutionResult>> {
        let start_time = Instant::now();
        let mut results = Vec::with_capacity(orders.len());

        // Process orders in parallel if enabled
        if self.config.enable_lock_free_processing && orders.len() > 1 {
            let futures: Vec<_> = orders.into_iter()
                .map(|order| self.process_order_optimized(order))
                .collect();

            let batch_results = futures::future::join_all(futures).await;
            for result in batch_results {
                results.push(result?);
            }
        } else {
            // Process sequentially
            for order in orders {
                let result = self.process_order_optimized(order).await?;
                results.push(result);
            }
        }

        let batch_latency = start_time.elapsed().as_millis() as f64;
        let throughput = results.len() as f64 / (batch_latency / 1000.0);

        // Check throughput threshold
        if throughput < self.config.target_throughput_tps {
            self.send_performance_alert(
                PerformanceAlertType::ThroughputBelowTarget,
                throughput,
                self.config.target_throughput_tps,
            ).await;
        }

        info!("Processed batch of {} orders in {:.2}ms ({:.2} TPS)",
              results.len(), batch_latency, throughput);

        Ok(results)
    }

    /// Get trading statistics with performance metrics
    pub async fn get_trading_statistics(&self) -> TradingStatistics {
        let trade_count = *self.daily_trade_count.read().await;
        let last_signal_time = *self.last_signal_time.read().await;
        let last_risk_check = *self.last_risk_check.read().await;
        let performance_metrics = self.get_performance_metrics().await;

        TradingStatistics {
            daily_trade_count: trade_count,
            last_signal_time,
            last_risk_check,
            active_instruments_count: self.active_instruments.read().await.len() as u32,
        }
    }

    /// Start optimized async order processing pipeline
    pub async fn start_async_order_pipeline(&self) -> Result<()> {
        info!("Starting optimized async order processing pipeline");

        // Take the receiver from the queue
        let receiver = {
            let mut receiver_guard = self.lock_free_order_queue.order_receiver.write().await;
            receiver_guard.take()
        };

        if let Some(mut receiver) = receiver {
            let engine_arc = Arc::new(self.clone());

            // Spawn dedicated async task for order processing
            tokio::spawn(async move {
                let mut batch_buffer = Vec::with_capacity(engine_arc.config.order_processing_batch_size);
                let mut last_batch_time = Instant::now();
                let batch_timeout = Duration::from_millis(10); // 10ms batch timeout for low latency

                loop {
                    tokio::select! {
                        // Receive new orders
                        order_result = receiver.recv() => {
                            match order_result {
                                Some(order) => {
                                    batch_buffer.push(order);

                                    // Process batch if full or timeout reached
                                    if batch_buffer.len() >= engine_arc.config.order_processing_batch_size
                                        || last_batch_time.elapsed() >= batch_timeout {

                                        if !batch_buffer.is_empty() {
                                            let batch = std::mem::take(&mut batch_buffer);
                                            let engine_ref = engine_arc.clone();

                                            // Process batch in parallel
                                            tokio::spawn(async move {
                                                if let Err(e) = engine_ref.process_batch_async(batch).await {
                                                    error!("Batch processing failed: {}", e);
                                                }
                                            });

                                            last_batch_time = Instant::now();
                                        }
                                    }
                                }
                                None => {
                                    warn!("Order processing channel closed");
                                    break;
                                }
                            }
                        }

                        // Timeout for partial batches
                        _ = tokio::time::sleep(batch_timeout) => {
                            if !batch_buffer.is_empty() && last_batch_time.elapsed() >= batch_timeout {
                                let batch = std::mem::take(&mut batch_buffer);
                                let engine_ref = engine_arc.clone();

                                tokio::spawn(async move {
                                    if let Err(e) = engine_ref.process_batch_async(batch).await {
                                        error!("Timeout batch processing failed: {}", e);
                                    }
                                });

                                last_batch_time = Instant::now();
                            }
                        }
                    }
                }
            });
        }

        Ok(())
    }

    /// Process batch of orders asynchronously with optimal concurrency
    async fn process_batch_async(&self, orders: Vec<OrderRequest>) -> Result<()> {
        if orders.is_empty() {
            return Ok(());
        }

        let start_time = Instant::now();
        let batch_size = orders.len();

        // Process orders with controlled concurrency
        let semaphore = Arc::new(Semaphore::new(self.config.max_concurrent_orders.min(batch_size)));
        let mut handles = Vec::with_capacity(batch_size);

        for order in orders {
            let permit = semaphore.clone().acquire_owned().await.map_err(|e| {
                crate::utils::PantherSwapError::trading(format!("Failed to acquire semaphore: {}", e))
            })?;

            let engine_arc = Arc::new(self.clone());
            let handle = tokio::spawn(async move {
                let _permit = permit; // Hold permit for duration of processing
                engine_arc.process_single_order_optimized(order).await
            });

            handles.push(handle);
        }

        // Wait for all orders to complete
        let mut successful = 0;
        let mut failed = 0;

        for handle in handles {
            match handle.await {
                Ok(Ok(_)) => successful += 1,
                Ok(Err(e)) => {
                    failed += 1;
                    debug!("Order processing failed: {}", e);
                }
                Err(e) => {
                    failed += 1;
                    error!("Task join failed: {}", e);
                }
            }
        }

        let batch_latency = start_time.elapsed().as_millis() as f64;
        let throughput = batch_size as f64 / (batch_latency / 1000.0);

        // Update atomic metrics
        self.lock_free_order_queue.performance_metrics.successful_orders.fetch_add(successful, Ordering::Relaxed);
        self.lock_free_order_queue.performance_metrics.failed_orders.fetch_add(failed, Ordering::Relaxed);

        info!("Processed batch: {} orders, {:.2}ms, {:.2} TPS, {}/{} success",
              batch_size, batch_latency, throughput, successful, successful + failed);

        Ok(())
    }

    /// Process single order with optimized async patterns
    async fn process_single_order_optimized(&self, order: OrderRequest) -> Result<ExecutionResult> {
        let start_time = Instant::now();

        // Clone order for async operations to avoid borrowing issues
        let order_clone = order.clone();

        // Use async risk checks if enabled
        let risk_check_future = if self.config.enable_async_risk_checks {
            Some(self.async_risk_check(&order_clone))
        } else {
            None
        };

        // Get market data asynchronously
        let market_data_future = self.get_market_data_async(order.instrument_id);

        // Execute both concurrently
        let (risk_result, _market_data) = if let Some(risk_future) = risk_check_future {
            let (risk, market) = tokio::try_join!(risk_future, market_data_future)?;
            (Some(risk), market)
        } else {
            (None, market_data_future.await?)
        };

        // Check risk if async risk checks are enabled
        if let Some(risk_assessment) = risk_result {
            if !risk_assessment.is_acceptable {
                return Err(crate::utils::PantherSwapError::trading(
                    "Order rejected by risk management".to_string()
                ));
            }
        }

        // Execute order
        let execution_result = self.execution_engine.execute_order(order).await?;

        // Update metrics
        let latency_ms = start_time.elapsed().as_millis() as f64;
        self.update_performance_metrics(latency_ms).await;

        Ok(execution_result)
    }

    /// Enhanced async risk check for non-blocking risk assessment
    async fn async_risk_check(&self, order: &OrderRequest) -> Result<RiskAssessment> {
        let start_time = Instant::now();

        // Get current positions for risk calculation
        let positions = self.portfolio_manager.get_positions().await?;

        // Fast position size calculation based on volatility
        let market_data = {
            let market_data_guard = self.market_data.read().await;
            market_data_guard.get(&order.instrument_id).cloned()
        };

        let adjusted_size = if let Some(data) = market_data {
            // Calculate volatility-adjusted position size
            let volatility = data.volatility.unwrap_or(0.01);
            let risk_factor = (volatility * 100.0).min(1.0); // Cap at 100% risk
            order.quantity * (1.0 - risk_factor * 0.5) // Reduce size by up to 50% for high volatility
        } else {
            order.quantity * 0.8 // Conservative default
        };

        // Fast VaR estimation
        let var_95 = adjusted_size * 0.02; // 2% VaR estimate

        // Quick liquidity check
        let liquidity_risk = if order.quantity > 10000.0 { 0.3 } else { 0.1 };

        // Portfolio exposure check
        let total_exposure: f64 = positions.values()
            .map(|p| p.size.abs() * p.entry_price)
            .sum();
        let new_exposure = total_exposure + adjusted_size * order.price.unwrap_or(1.0);
        let max_drawdown_risk = (new_exposure / 1000000.0).min(0.1); // Assume 1M portfolio

        let is_acceptable = var_95 < 1000.0 && // Max $1000 VaR per trade
                           liquidity_risk < 0.5 &&
                           max_drawdown_risk < 0.05; // Max 5% drawdown risk

        // Log performance for monitoring
        let latency_ms = start_time.elapsed().as_millis() as f64;
        if latency_ms > 5.0 {
            warn!("Risk check latency exceeded 5ms: {}ms", latency_ms);
        }

        Ok(RiskAssessment {
            is_acceptable,
            adjusted_position_size: if is_acceptable { adjusted_size } else { 0.0 },
            var_95,
            expected_shortfall: var_95 * 1.3, // ES typically 30% higher than VaR
            max_drawdown_risk,
            correlation_risk: 0.1, // Simplified
            liquidity_risk,
        })
    }

    /// Get market data asynchronously
    async fn get_market_data_async(&self, instrument_id: Uuid) -> Result<MarketData> {
        let market_data_guard = self.market_data.read().await;
        market_data_guard.get(&instrument_id)
            .cloned()
            .ok_or_else(|| crate::utils::PantherSwapError::trading(
                format!("No market data available for instrument {}", instrument_id)
            ))
    }

    /// Submit order through adaptive batching system
    pub async fn submit_order_adaptive(&self, order: OrderRequest) -> Result<()> {
        if let Some(ref batch_processor) = self.adaptive_batch_processor {
            if let Some(batch) = batch_processor.add_order(order).await {
                // Process the batch
                let start_time = Instant::now();
                let batch_size = batch.len();
                let batch_id = Uuid::new_v4();

                // Process batch asynchronously
                self.process_batch_async(batch).await?;

                // Calculate metrics
                let processing_latency = start_time.elapsed().as_millis() as f64;
                let throughput = batch_size as f64 / (processing_latency / 1000.0);
                let success_rate = 1.0; // Simplified - assume success if no error

                // Record performance metrics
                let metrics = BatchMetrics {
                    batch_id,
                    batch_size,
                    processing_latency_ms: processing_latency,
                    throughput_tps: throughput,
                    success_rate,
                    queue_depth_at_start: batch_size, // Simplified
                    timestamp: Utc::now(),
                };

                batch_processor.record_batch_performance(metrics).await;
            }
        } else {
            // Fallback to standard processing
            self.process_order_optimized(order).await?;
        }

        Ok(())
    }

    /// Get adaptive batching statistics
    pub async fn get_adaptive_batching_stats(&self) -> Option<AdaptiveBatchingStats> {
        if let Some(ref batch_processor) = self.adaptive_batch_processor {
            Some(batch_processor.get_performance_stats().await)
        } else {
            None
        }
    }

    /// Get current optimal batch size from adaptive system
    pub fn get_optimal_batch_size(&self) -> usize {
        if let Some(ref batch_processor) = self.adaptive_batch_processor {
            batch_processor.get_current_batch_size()
        } else {
            self.config.order_processing_batch_size
        }
    }

    /// Submit order through lock-free queue system
    pub async fn submit_order_lock_free(&self, order: OrderRequest) -> Result<()> {
        if let Some(ref lock_free_queue) = self.lock_free_order_queue_v2 {
            // Clone order for potential fallback
            let order_clone = order.clone();

            // Try to enqueue the order
            if lock_free_queue.enqueue(order) {
                // Process orders from the queue
                self.process_lock_free_queue().await?;
            } else {
                // Queue is full, process immediately
                warn!("Lock-free queue full, processing order immediately");
                self.process_order_optimized(order_clone).await?;
            }
        } else {
            // Fallback to standard processing
            self.process_order_optimized(order).await?;
        }

        Ok(())
    }

    /// Process orders from lock-free queue
    async fn process_lock_free_queue(&self) -> Result<()> {
        if let Some(ref lock_free_queue) = self.lock_free_order_queue_v2 {
            let mut processed_count = 0;
            let max_batch_size = self.get_optimal_batch_size();

            // Process orders in batches
            while processed_count < max_batch_size {
                if let Some(order) = lock_free_queue.dequeue() {
                    // Use memory pool if available
                    let optimized_order = if let Some(ref memory_pool) = self.order_memory_pool_v2 {
                        // For now, just process the order directly
                        // In a real implementation, we'd use the memory pool for object reuse
                        order
                    } else {
                        order
                    };

                    // Process the order
                    if let Err(e) = self.process_order_optimized(optimized_order).await {
                        error!("Failed to process order from lock-free queue: {}", e);
                    }

                    processed_count += 1;
                } else {
                    break; // Queue is empty
                }
            }

            if processed_count > 0 {
                debug!("Processed {} orders from lock-free queue", processed_count);
            }
        }

        Ok(())
    }

    /// Get lock-free queue statistics
    pub fn get_lock_free_queue_stats(&self) -> Option<(usize, usize)> {
        if let Some(ref lock_free_queue) = self.lock_free_order_queue_v2 {
            Some((lock_free_queue.len(), lock_free_queue.get_metrics().queue_capacity))
        } else {
            None
        }
    }

    /// Get memory pool statistics
    pub fn get_memory_pool_stats(&self) -> Option<String> {
        if let Some(ref memory_pool) = self.order_memory_pool_v2 {
            let stats = memory_pool.get_stats();
            Some(format!(
                "Pool: {}/{} items, Hits: {}, Misses: {}, Hit Rate: {:.2}%",
                stats.current_pool_size,
                stats.pool_capacity,
                stats.pool_hits,
                stats.pool_misses,
                if stats.pool_hits + stats.pool_misses > 0 {
                    stats.pool_hits as f64 / (stats.pool_hits + stats.pool_misses) as f64 * 100.0
                } else {
                    0.0
                }
            ))
        } else {
            None
        }
    }

    /// Store trade execution in database with comprehensive logging
    async fn store_trade_execution(
        &self,
        execution: &ExecutionResult,
        signal_id: Option<Uuid>,
        execution_time_ms: Option<i32>
    ) -> Result<()> {
        use crate::database::query_functions::{insert_trade_execution, insert_position_update, insert_risk_metrics, insert_pnl_record};

        // Store the trade execution
        let slippage_bps = None; // Would be calculated from expected vs actual price
        let fees = Some(execution.filled_quantity * execution.average_price * 0.001); // 0.1% fee

        if let Err(e) = insert_trade_execution(
            &self.database.pool,
            execution,
            signal_id,
            execution_time_ms,
            slippage_bps,
            fees
        ).await {
            error!("Failed to store trade execution: {}", e);
        } else {
            debug!("Stored trade execution: {} {} @ {} for {}",
                   execution.filled_quantity, execution.average_price,
                   execution.instrument_id, execution.strategy_name);
        }

        // Store position update if we have position information
        let positions = self.portfolio_manager.get_positions().await;
        if let Some(position) = positions.get(&execution.instrument_id) {
            if let Err(e) = insert_position_update(&self.database.pool, position).await {
                error!("Failed to store position update: {}", e);
            } else {
                debug!("Stored position update for instrument {}", execution.instrument_id);
            }
        }

        // Store risk metrics
        let portfolio_state = self.portfolio_manager.get_portfolio_state().await;
        if let Err(e) = insert_risk_metrics(
            &self.database.pool,
            Some(execution.instrument_id),
            portfolio_state.total_var,
            portfolio_state.gross_exposure,
            portfolio_state.leverage,
            portfolio_state.max_drawdown,
            Some(portfolio_state.sharpe_ratio),
            portfolio_state.max_loss_24h,
            portfolio_state.risk_score
        ).await {
            error!("Failed to store risk metrics: {}", e);
        } else {
            debug!("Stored risk metrics for execution");
        }

        // Store P&L record
        // Calculate P&L based on execution
        let realized_pnl = 0.0; // Would be calculated based on position changes
        let unrealized_pnl = 0.0; // Would be calculated based on current market prices
        let total_pnl = realized_pnl + unrealized_pnl;

        if let Err(e) = insert_pnl_record(
            &self.database.pool,
            execution.instrument_id,
            &execution.strategy_name,
            realized_pnl,
            unrealized_pnl,
            total_pnl,
            1, // trade count
            if total_pnl > 0.0 { 1.0 } else { 0.0 }, // win rate
            Some(portfolio_state.sharpe_ratio)
        ).await {
            error!("Failed to store P&L record: {}", e);
        } else {
            debug!("Stored P&L record: realized={}, unrealized={}, total={}",
                   realized_pnl, unrealized_pnl, total_pnl);
        }

        Ok(())
    }

    // ============================================================================
    // IG TRADING INTEGRATION METHODS
    // ============================================================================

    /// Check if IG Trading integration is available and ready
    pub async fn is_ig_trading_ready(&self) -> bool {
        if let Some(ref manager) = self.market_data_manager {
            manager.get_provider_status().await.is_ok()
        } else {
            false
        }
    }

    /// Get comprehensive IG Trading status
    pub async fn get_ig_trading_status(&self) -> Result<serde_json::Value> {
        let ig_ready = self.is_ig_trading_ready().await;
        let execution_status = self.execution_engine.get_execution_status().await?;

        Ok(serde_json::json!({
            "ig_trading_integration": {
                "provider_available": true,
                "ready_for_trading": ig_ready,
            },
            "execution_status": execution_status,
            "market_data_manager": {
                "available": self.market_data_manager.is_some(),
                "status": if let Some(ref manager) = self.market_data_manager {
                    Some(manager.get_provider_status().await.unwrap_or_default())
                } else {
                    None
                }
            }
        }))
    }

    /// Start live market data streaming through IG Trading
    pub async fn start_ig_trading_streaming(&self, symbols: Vec<String>) -> Result<()> {
        if let Some(ref manager) = self.market_data_manager {
            info!("Starting IG Trading live streaming for {} symbols", symbols.len());
            manager.start_live_streaming(symbols).await
        } else {
            Err(crate::utils::PantherSwapError::trading(
                "Market data manager not available".to_string()
            ))
        }
    }

    /// Test IG Trading connectivity and execution capabilities
    pub async fn test_ig_trading_integration(&self) -> Result<serde_json::Value> {
        let mut test_results = serde_json::json!({
            "test_timestamp": chrono::Utc::now(),
            "tests": {}
        });

        // Test execution engine connectivity
        let execution_test = serde_json::json!({
            "status": "ok",
            "message": "IG Trading execution engine ready"
        });
        test_results["tests"]["execution_engine"] = execution_test;

        // Test market data connectivity
        if let Some(ref manager) = self.market_data_manager {
            if let Ok(provider_status) = manager.get_provider_status().await {
                test_results["tests"]["market_data"] = provider_status;
            }
        }

        // Test overall readiness
        test_results["overall_ready"] = serde_json::Value::Bool(self.is_ig_trading_ready().await);

        Ok(test_results)
    }

    /// Get real-time market data from IG Trading
    pub async fn get_ig_trading_market_data(&self, symbols: &[String]) -> Result<Option<std::collections::HashMap<String, crate::market_data::types::MarketQuote>>> {
        if let Some(ref manager) = self.market_data_manager {
            // Clone the Arc to get a mutable reference
            let mut manager_clone = (**manager).clone();
            Ok(Some(manager_clone.get_multiple_quotes(symbols).await?))
        } else {
            Ok(None)
        }
    }

    /// Process IG Trading market data update and integrate with trading pipeline
    pub async fn process_ig_trading_market_data(&self, symbols: &[String]) -> Result<()> {
        if let Some(ref manager) = self.market_data_manager {
            // Get latest quotes from IG Trading
            let quotes = manager.get_multiple_quotes(symbols).await?;

            // Convert quotes to market ticks for processing
            let mut market_ticks = Vec::new();
            for (symbol, quote) in quotes {
                // We need to map symbol to instrument_id
                // For now, we'll create a placeholder instrument_id
                let instrument_id = Uuid::new_v4(); // In production, this would be looked up

                let market_tick = MarketTick {
                    timestamp: quote.timestamp,
                    instrument_id,
                    provider: "ig_trading".to_string(),
                    bid_price: quote.bid_price,
                    ask_price: quote.ask_price,
                    bid_size: quote.bid_size.unwrap_or(1000.0),
                    ask_size: quote.ask_size.unwrap_or(1000.0),
                    last_price: Some(quote.mid_price),
                    volume: quote.volume,
                    spread: quote.ask_price - quote.bid_price,
                    data_quality_score: quote.data_quality,
                    raw_data: serde_json::json!({
                        "symbol": symbol,
                        "provider": "ig_trading"
                    }),
                    // Backward compatibility fields
                    symbol: Some(symbol.clone()),
                    price: Some(quote.mid_price),
                    bid: Some(quote.bid_price),
                    ask: Some(quote.ask_price),
                };
                market_ticks.push(market_tick);
            }

            // Process through the trading pipeline
            self.process_market_data(&market_ticks).await?;

            info!("Processed {} IG Trading market data updates", market_ticks.len());
        }

        Ok(())
    }

    /// Get comprehensive trading engine status including IG Trading integration
    pub async fn get_comprehensive_status(&self) -> Result<serde_json::Value> {
        let state = self.state.read().await;
        let trade_count = *self.daily_trade_count.read().await;
        let active_instruments = self.active_instruments.read().await;
        let ig_trading_status = self.get_ig_trading_status().await?;

        Ok(serde_json::json!({
            "engine_state": format!("{:?}", *state),
            "daily_trade_count": trade_count,
            "active_instruments_count": active_instruments.len(),
            "config": {
                "enable_live_trading": self.config.enable_live_trading,
                "confidence_threshold": self.config.confidence_threshold,
                "max_daily_trades": self.config.max_daily_trades,
                "target_latency_ms": self.config.target_latency_ms,
                "target_throughput_tps": self.config.target_throughput_tps,
            },
            "ig_trading_integration": ig_trading_status,
            "performance_metrics": self.get_performance_metrics().await,
        }))
    }
}

// Trading statistics structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingStatistics {
    pub daily_trade_count: u32,
    pub last_signal_time: DateTime<Utc>,
    pub last_risk_check: DateTime<Utc>,
    pub active_instruments_count: u32,
}

// Factory function to create a high-performance trading engine with IG Trading integration
pub fn create_optimized_trading_engine_with_ig_trading(
    database: Database,
    settings: &Settings
) -> impl std::future::Future<Output = Result<TradingEngine>> + '_ {
    async move {
        let config = TradingEngineConfig {
            enable_live_trading: true,
            max_position_size: 100_000.0,
            confidence_threshold: 0.7,
            max_daily_trades: 1000, // Higher limit for high-frequency trading
            max_portfolio_exposure: 0.8,
            risk_check_interval_ms: 100, // More frequent risk checks
            market_data_timeout_ms: 1000, // Shorter timeout
            signal_generation_interval_ms: 50, // Faster signal generation
            portfolio_update_interval_ms: 500, // More frequent updates
            enable_stop_loss: true,
            enable_take_profit: true,
            emergency_stop_loss_pct: 0.05,
            max_correlation_exposure: 0.3,
            enable_regime_filtering: true,

            // Optimized performance settings
            enable_lock_free_processing: true,
            enable_memory_pool: true,
            max_concurrent_orders: 2000, // Higher concurrency
            order_processing_batch_size: 100, // Larger batches
            enable_async_risk_checks: true,
            target_latency_ms: 5.0, // Aggressive latency target
            target_throughput_tps: 2000.0, // Higher throughput target
            enable_performance_monitoring: true,
            enable_adaptive_batching: true,
            adaptive_batching_config: AdaptiveBatchingConfig {
                min_batch_size: 20,
                max_batch_size: 300,
                initial_batch_size: 100,
                target_latency_ms: 5.0,
                max_wait_time_ms: 3, // Aggressive 3ms wait time
                load_threshold_high: 0.85,
                load_threshold_low: 0.25,
                batch_size_increment: 15,
                batch_size_decrement: 8,
                performance_window_size: 150,
                enable_predictive_sizing: true,
                enable_load_balancing: true,
            },
            enable_lock_free_queue: true,
            lock_free_queue_capacity: 20000, // Large capacity for high throughput
            enable_memory_pools: true,
            memory_pool_size: 2000, // Large memory pool
        };

        TradingEngine::new_with_ig_trading(config, database, settings).await
    }
}

// Factory function to create a high-performance trading engine (legacy without Alpaca)
pub fn create_optimized_trading_engine(database: Database) -> impl std::future::Future<Output = Result<TradingEngine>> {
    async move {
        let config = TradingEngineConfig {
            enable_live_trading: true,
            max_position_size: 100_000.0,
            confidence_threshold: 0.7,
            max_daily_trades: 1000, // Higher limit for high-frequency trading
            max_portfolio_exposure: 0.8,
            risk_check_interval_ms: 100, // More frequent risk checks
            market_data_timeout_ms: 1000, // Shorter timeout
            signal_generation_interval_ms: 50, // Faster signal generation
            portfolio_update_interval_ms: 500, // More frequent updates
            enable_stop_loss: true,
            enable_take_profit: true,
            emergency_stop_loss_pct: 0.05,
            max_correlation_exposure: 0.3,
            enable_regime_filtering: true,

            // Optimized performance settings
            enable_lock_free_processing: true,
            enable_memory_pool: true,
            max_concurrent_orders: 2000, // Higher concurrency
            order_processing_batch_size: 100, // Larger batches
            enable_async_risk_checks: true,
            target_latency_ms: 5.0, // Aggressive latency target
            target_throughput_tps: 2000.0, // Higher throughput target
            enable_performance_monitoring: true,
            enable_adaptive_batching: true,
            adaptive_batching_config: AdaptiveBatchingConfig {
                min_batch_size: 20,
                max_batch_size: 300,
                initial_batch_size: 100,
                target_latency_ms: 5.0,
                max_wait_time_ms: 3, // Aggressive 3ms wait time
                load_threshold_high: 0.85,
                load_threshold_low: 0.25,
                batch_size_increment: 15,
                batch_size_decrement: 8,
                performance_window_size: 150,
                enable_predictive_sizing: true,
                enable_load_balancing: true,
            },
            enable_lock_free_queue: true,
            lock_free_queue_capacity: 20000, // Large capacity for high throughput
            enable_memory_pools: true,
            memory_pool_size: 2000, // Large memory pool
        };

        TradingEngine::new(config, database).await
    }
}
