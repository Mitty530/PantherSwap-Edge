use crate::config::{Settings, AlpacaConfig};
use crate::database::Database;
use crate::market_data::AlpacaProvider;
use crate::trading::{AlpacaExecutionEngine, signals::{OrderRequest, ExecutionResult}};
use crate::utils::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tracing::{info, warn, error};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Integrated Alpaca trading engine combining market data and execution
#[derive(Clone)]
pub struct AlpacaTradingEngine {
    market_provider: AlpacaProvider,
    execution_engine: AlpacaExecutionEngine,
    database: Database,
    config: AlpacaTradingConfig,
    active_strategies: Arc<RwLock<HashMap<String, TradingStrategy>>>,
    performance_metrics: Arc<RwLock<TradingPerformanceMetrics>>,
    is_running: Arc<RwLock<bool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlpacaTradingConfig {
    pub enable_live_trading: bool,
    pub max_position_size: f64,
    pub max_daily_loss: f64,
    pub risk_check_interval_ms: u64,
    pub signal_generation_interval_ms: u64,
    pub enable_paper_trading: bool,
    pub symbols: Vec<String>,
    pub strategy_allocation: HashMap<String, f64>,
}

#[derive(Debug, Clone)]
pub struct TradingStrategy {
    pub name: String,
    pub allocation: f64,
    pub enabled: bool,
    pub last_signal_time: Option<DateTime<Utc>>,
    pub total_trades: u64,
    pub profitable_trades: u64,
    pub total_pnl: f64,
}

#[derive(Debug, Clone, Default)]
pub struct TradingPerformanceMetrics {
    pub total_trades: u64,
    pub profitable_trades: u64,
    pub total_pnl: f64,
    pub total_volume: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub win_rate: f64,
    pub average_trade_duration_minutes: f64,
    pub daily_pnl: HashMap<String, f64>, // Date -> PnL
}

/// Trading signal with Alpaca-specific information
#[derive(Debug, Clone)]
pub struct AlpacaTradingSignal {
    pub symbol: String,
    pub signal_type: SignalType,
    pub strength: f64,
    pub confidence: f64,
    pub target_price: Option<f64>,
    pub stop_loss: Option<f64>,
    pub position_size: f64,
    pub strategy_name: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum SignalType {
    Buy,
    Sell,
    Hold,
    Close,
}

impl AlpacaTradingEngine {
    /// Create a new Alpaca trading engine
    pub async fn new(settings: &Settings, database: Database) -> Result<Self> {
        info!("Initializing Alpaca Trading Engine");

        // Initialize market data provider
        let market_provider = AlpacaProvider::new(settings.market_data.alpaca.clone())?
            .with_database(database.clone());

        // Validate market data connection
        market_provider.validate_configuration().await?;

        // Initialize execution engine
        let execution_engine = AlpacaExecutionEngine::new(settings.market_data.alpaca.clone())?
            .with_database(database.clone());

        // Create trading configuration
        let config = AlpacaTradingConfig {
            enable_live_trading: settings.trading.enable_live_trading,
            max_position_size: settings.trading.max_position_size,
            max_daily_loss: settings.risk.max_daily_loss,
            risk_check_interval_ms: settings.trading.risk_check_interval_ms,
            signal_generation_interval_ms: settings.trading.signal_generation_interval_ms,
            enable_paper_trading: settings.market_data.alpaca.paper_trading,
            symbols: settings.market_data.instruments.clone(),
            strategy_allocation: HashMap::new(), // Will be configured later
        };

        Ok(Self {
            market_provider,
            execution_engine,
            database,
            config,
            active_strategies: Arc::new(RwLock::new(HashMap::new())),
            performance_metrics: Arc::new(RwLock::new(TradingPerformanceMetrics::default())),
            is_running: Arc::new(RwLock::new(false)),
        })
    }

    /// Start the trading engine
    pub async fn start(&self) -> Result<()> {
        info!("Starting Alpaca Trading Engine");

        {
            let mut running = self.is_running.write().await;
            if *running {
                return Err(crate::utils::PantherSwapError::trading(
                    "Trading engine is already running".to_string()
                ));
            }
            *running = true;
        }

        // Check market status
        let market_status = self.market_provider.get_market_status().await?;
        info!("Market status: {}", serde_json::to_string_pretty(&market_status)?);

        // Initialize default strategies
        self.initialize_default_strategies().await?;

        // Start market data streaming
        let symbols = self.config.symbols.clone();
        let mut stream_rx = self.market_provider.start_streaming(symbols.clone()).await?;

        // Start trading loop
        let engine_clone = self.clone();
        tokio::spawn(async move {
            engine_clone.trading_loop(stream_rx).await;
        });

        info!("✅ Alpaca Trading Engine started successfully");
        Ok(())
    }

    /// Stop the trading engine
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping Alpaca Trading Engine");

        {
            let mut running = self.is_running.write().await;
            *running = false;
        }

        // Close all positions if configured
        if self.config.enable_live_trading {
            info!("Closing all positions before shutdown");
            match self.execution_engine.close_all_positions().await {
                Ok(results) => {
                    info!("Closed {} positions", results.len());
                }
                Err(e) => {
                    error!("Failed to close positions: {}", e);
                }
            }
        }

        info!("✅ Alpaca Trading Engine stopped");
        Ok(())
    }

    /// Main trading loop
    async fn trading_loop(&self, mut stream_rx: mpsc::UnboundedReceiver<crate::market_data::alpaca::AlpacaStreamEvent>) {
        info!("Starting trading loop");

        let mut signal_interval = tokio::time::interval(
            std::time::Duration::from_millis(self.config.signal_generation_interval_ms)
        );

        loop {
            // Check if engine should continue running
            {
                let running = self.is_running.read().await;
                if !*running {
                    break;
                }
            }

            tokio::select! {
                // Handle market data events
                event = stream_rx.recv() => {
                    if let Some(event) = event {
                        if let Err(e) = self.handle_market_event(event).await {
                            error!("Error handling market event: {}", e);
                        }
                    }
                }

                // Generate trading signals periodically
                _ = signal_interval.tick() => {
                    if let Err(e) = self.generate_and_execute_signals().await {
                        error!("Error in signal generation: {}", e);
                    }
                }
            }
        }

        info!("Trading loop ended");
    }

    /// Handle incoming market data events
    async fn handle_market_event(&self, event: crate::market_data::alpaca::AlpacaStreamEvent) -> Result<()> {
        match event {
            crate::market_data::alpaca::AlpacaStreamEvent::Quote(quote) => {
                // Update internal market data and potentially trigger signals
                self.process_quote_update(&quote).await?;
            }
            crate::market_data::alpaca::AlpacaStreamEvent::Trade(trade) => {
                // Process trade data for market analysis
                self.process_trade_update(&trade).await?;
            }
            crate::market_data::alpaca::AlpacaStreamEvent::Error(error) => {
                error!("Market data error: {}", error);
            }
            _ => {}
        }

        Ok(())
    }

    /// Process quote updates
    async fn process_quote_update(&self, quote: &crate::market_data::alpaca::AlpacaQuote) -> Result<()> {
        // Store quote in database for analysis
        // This would integrate with your existing market data pipeline
        info!("Processing quote for {}: ${:.2}/${:.2}", 
            quote.symbol, quote.bid_price, quote.ask_price);

        Ok(())
    }

    /// Process trade updates
    async fn process_trade_update(&self, trade: &crate::market_data::alpaca::AlpacaTrade) -> Result<()> {
        // Analyze trade data for momentum signals
        info!("Processing trade for {}: ${:.2} x{}", 
            trade.symbol, trade.price, trade.size);

        Ok(())
    }

    /// Generate and execute trading signals
    async fn generate_and_execute_signals(&self) -> Result<()> {
        // Check if market is open
        if !self.market_provider.is_market_open().await? {
            return Ok(());
        }

        // Generate signals for each symbol
        for symbol in &self.config.symbols {
            if let Ok(signal) = self.generate_signal_for_symbol(symbol).await {
                if signal.confidence > 0.7 { // Only execute high-confidence signals
                    if let Err(e) = self.execute_signal(&signal).await {
                        error!("Failed to execute signal for {}: {}", symbol, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Generate a trading signal for a specific symbol
    async fn generate_signal_for_symbol(&self, symbol: &str) -> Result<AlpacaTradingSignal> {
        // Get latest market data
        let quote = self.market_provider.get_latest_quote(symbol).await?;
        
        // Simple momentum strategy (placeholder - replace with your AI models)
        let signal_strength = self.calculate_momentum_signal(&quote).await?;
        
        let signal_type = if signal_strength > 0.1 {
            SignalType::Buy
        } else if signal_strength < -0.1 {
            SignalType::Sell
        } else {
            SignalType::Hold
        };

        Ok(AlpacaTradingSignal {
            symbol: symbol.to_string(),
            signal_type,
            strength: signal_strength.abs(),
            confidence: 0.8, // Placeholder confidence
            target_price: Some(quote.ask_price * 1.02), // 2% target
            stop_loss: Some(quote.bid_price * 0.98), // 2% stop loss
            position_size: self.calculate_position_size(symbol, signal_strength).await?,
            strategy_name: "momentum".to_string(),
            timestamp: Utc::now(),
        })
    }

    /// Calculate momentum signal (placeholder implementation)
    async fn calculate_momentum_signal(&self, quote: &crate::market_data::types::MarketQuote) -> Result<f64> {
        // Placeholder momentum calculation
        // In production, this would use your AI models and technical indicators
        let spread_ratio = quote.spread / quote.exchange_rate;
        let momentum = if spread_ratio < 0.001 { 0.5 } else { -0.2 }; // Simple logic
        
        Ok(momentum)
    }

    /// Calculate position size based on risk management
    async fn calculate_position_size(&self, symbol: &str, signal_strength: f64) -> Result<f64> {
        let base_size = self.config.max_position_size * 0.1; // 10% of max position
        let adjusted_size = base_size * signal_strength.abs();
        
        Ok(adjusted_size.min(self.config.max_position_size))
    }

    /// Execute a trading signal
    async fn execute_signal(&self, signal: &AlpacaTradingSignal) -> Result<()> {
        if !self.config.enable_live_trading {
            info!("Paper trading mode - would execute: {:?} {} shares of {}", 
                signal.signal_type, signal.position_size, signal.symbol);
            return Ok(());
        }

        match signal.signal_type {
            SignalType::Buy => {
                let result = self.execution_engine.market_buy(&signal.symbol, signal.position_size).await?;
                self.update_performance_metrics(&result).await;
                info!("Executed BUY order for {}: {} shares", signal.symbol, signal.position_size);
            }
            SignalType::Sell => {
                let result = self.execution_engine.market_sell(&signal.symbol, signal.position_size).await?;
                self.update_performance_metrics(&result).await;
                info!("Executed SELL order for {}: {} shares", signal.symbol, signal.position_size);
            }
            SignalType::Close => {
                // Close existing position
                info!("Closing position for {}", signal.symbol);
            }
            SignalType::Hold => {
                // No action needed
            }
        }

        Ok(())
    }

    /// Initialize default trading strategies
    async fn initialize_default_strategies(&self) -> Result<()> {
        let mut strategies = self.active_strategies.write().await;
        
        strategies.insert("momentum".to_string(), TradingStrategy {
            name: "momentum".to_string(),
            allocation: 0.5,
            enabled: true,
            last_signal_time: None,
            total_trades: 0,
            profitable_trades: 0,
            total_pnl: 0.0,
        });

        strategies.insert("mean_reversion".to_string(), TradingStrategy {
            name: "mean_reversion".to_string(),
            allocation: 0.3,
            enabled: true,
            last_signal_time: None,
            total_trades: 0,
            profitable_trades: 0,
            total_pnl: 0.0,
        });

        info!("Initialized {} trading strategies", strategies.len());
        Ok(())
    }

    /// Update performance metrics
    async fn update_performance_metrics(&self, result: &ExecutionResult) {
        let mut metrics = self.performance_metrics.write().await;
        metrics.total_trades += 1;
        metrics.total_volume += result.filled_quantity;
        
        // Calculate PnL (simplified)
        let trade_pnl = result.filled_quantity * result.average_price * 0.001; // Placeholder
        metrics.total_pnl += trade_pnl;
        
        if trade_pnl > 0.0 {
            metrics.profitable_trades += 1;
        }
        
        metrics.win_rate = metrics.profitable_trades as f64 / metrics.total_trades as f64;
    }

    /// Get current performance metrics
    pub async fn get_performance_metrics(&self) -> TradingPerformanceMetrics {
        self.performance_metrics.read().await.clone()
    }

    /// Get portfolio summary
    pub async fn get_portfolio_summary(&self) -> Result<serde_json::Value> {
        self.execution_engine.get_portfolio_summary().await
    }

    /// Check if engine is running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }
}
