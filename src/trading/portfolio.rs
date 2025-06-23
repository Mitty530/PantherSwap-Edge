use crate::database::Database;
use crate::trading::signals::{Position, RiskMetrics, ExecutionResult};
use crate::trading::execution::MarketData;
use crate::database::types::SignalType;
use crate::utils::Result;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;

// Portfolio Configuration
#[derive(Debug, Clone)]
pub struct PortfolioConfig {
    pub initial_capital: f64,
    pub max_leverage: f64,
    pub max_position_concentration: f64,  // Max % of portfolio in single position
    pub max_sector_concentration: f64,    // Max % of portfolio in single sector
    pub rebalance_threshold: f64,         // Trigger rebalancing when drift exceeds this
    pub margin_requirement: f64,          // Margin requirement for leveraged positions
    pub max_daily_loss: f64,             // Max daily loss before trading halt
    pub performance_fee_rate: f64,        // Performance fee rate
    pub management_fee_rate: f64,         // Annual management fee rate
}

impl Default for PortfolioConfig {
    fn default() -> Self {
        Self {
            initial_capital: 1_000_000.0,    // $1M initial capital
            max_leverage: 3.0,               // 3x max leverage
            max_position_concentration: 0.1,  // 10% max per position
            max_sector_concentration: 0.3,    // 30% max per sector
            rebalance_threshold: 0.05,        // 5% drift threshold
            margin_requirement: 0.2,          // 20% margin requirement
            max_daily_loss: 0.02,            // 2% max daily loss
            performance_fee_rate: 0.20,       // 20% performance fee
            management_fee_rate: 0.02,        // 2% annual management fee
        }
    }
}

// Portfolio State
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioState {
    pub total_value: f64,
    pub cash: f64,
    pub invested_capital: f64,
    pub total_exposure: f64,
    pub net_exposure: f64,
    pub gross_exposure: f64,
    pub leverage: f64,
    pub margin_used: f64,
    pub margin_available: f64,
    pub daily_pnl: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
    pub total_pnl: f64,
    pub current_drawdown: f64,
    pub max_drawdown: f64,
    pub high_water_mark: f64,
    pub last_updated: DateTime<Utc>,
}

impl Default for PortfolioState {
    fn default() -> Self {
        let initial_capital = 1_000_000.0;
        Self {
            total_value: initial_capital,
            cash: initial_capital,
            invested_capital: 0.0,
            total_exposure: 0.0,
            net_exposure: 0.0,
            gross_exposure: 0.0,
            leverage: 0.0,
            margin_used: 0.0,
            margin_available: initial_capital,
            daily_pnl: 0.0,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
            total_pnl: 0.0,
            current_drawdown: 0.0,
            max_drawdown: 0.0,
            high_water_mark: initial_capital,
            last_updated: Utc::now(),
        }
    }
}

// Performance Metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_return: f64,
    pub annualized_return: f64,
    pub volatility: f64,
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub max_drawdown: f64,
    pub calmar_ratio: f64,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub avg_win: f64,
    pub avg_loss: f64,
    pub total_trades: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub largest_win: f64,
    pub largest_loss: f64,
    pub consecutive_wins: u32,
    pub consecutive_losses: u32,
    pub recovery_factor: f64,
    pub var_95: f64,
    pub expected_shortfall: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            total_return: 0.0,
            annualized_return: 0.0,
            volatility: 0.0,
            sharpe_ratio: 0.0,
            sortino_ratio: 0.0,
            max_drawdown: 0.0,
            calmar_ratio: 0.0,
            win_rate: 0.0,
            profit_factor: 0.0,
            avg_win: 0.0,
            avg_loss: 0.0,
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            largest_win: 0.0,
            largest_loss: 0.0,
            consecutive_wins: 0,
            consecutive_losses: 0,
            recovery_factor: 0.0,
            var_95: 0.0,
            expected_shortfall: 0.0,
        }
    }
}

// Trade Record for Performance Tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    pub id: Uuid,
    pub instrument_id: Uuid,
    pub strategy_name: String,
    pub entry_time: DateTime<Utc>,
    pub exit_time: Option<DateTime<Utc>>,
    pub entry_price: f64,
    pub exit_price: Option<f64>,
    pub quantity: f64,
    pub side: SignalType,
    pub realized_pnl: f64,
    pub fees: f64,
    pub holding_period: Option<Duration>,
    pub max_favorable_excursion: f64,
    pub max_adverse_excursion: f64,
    pub is_closed: bool,
}

// Position Update Event
#[derive(Debug, Clone)]
pub struct PositionUpdate {
    pub instrument_id: Uuid,
    pub current_price: f64,
    pub timestamp: DateTime<Utc>,
}

// Portfolio Manager Implementation
#[derive(Clone)]
pub struct PortfolioManager {
    config: PortfolioConfig,
    state: Arc<RwLock<PortfolioState>>,
    positions: Arc<RwLock<HashMap<Uuid, Position>>>,
    trade_history: Arc<RwLock<Vec<TradeRecord>>>,
    performance_metrics: Arc<RwLock<PerformanceMetrics>>,
    daily_returns: Arc<RwLock<Vec<f64>>>,
    database: Database,
}

impl PortfolioManager {
    pub async fn new(config: PortfolioConfig, database: Database) -> Result<Self> {
        let initial_state = PortfolioState {
            total_value: config.initial_capital,
            cash: config.initial_capital,
            margin_available: config.initial_capital,
            high_water_mark: config.initial_capital,
            ..Default::default()
        };

        Ok(Self {
            config,
            state: Arc::new(RwLock::new(initial_state)),
            positions: Arc::new(RwLock::new(HashMap::new())),
            trade_history: Arc::new(RwLock::new(Vec::new())),
            performance_metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            daily_returns: Arc::new(RwLock::new(Vec::new())),
            database,
        })
    }

    /// Process a new execution result and update portfolio
    pub async fn process_execution(&self, execution: &ExecutionResult) -> Result<()> {
        let mut positions_guard = self.positions.write().await;
        let mut state_guard = self.state.write().await;

        // Check if we have an existing position
        if let Some(existing_position) = positions_guard.get_mut(&execution.instrument_id) {
            // Update existing position
            self.update_existing_position(existing_position, execution, &mut state_guard).await?;
        } else {
            // Create new position
            let new_position = self.create_new_position(execution).await?;
            positions_guard.insert(execution.instrument_id, new_position);
        }

        // Update portfolio state
        self.recalculate_portfolio_state(&positions_guard, &mut state_guard).await?;

        Ok(())
    }

    /// Update portfolio with current market prices
    pub async fn update_market_prices(&self, market_data: &HashMap<Uuid, MarketData>) -> Result<()> {
        let mut positions_guard = self.positions.write().await;
        let mut state_guard = self.state.write().await;

        // Update unrealized P&L for all positions
        for (instrument_id, position) in positions_guard.iter_mut() {
            if let Some(market_price) = market_data.get(instrument_id) {
                self.update_position_pnl(position, market_price.last_price.unwrap_or(market_price.bid_price)).await?;
            }
        }

        // Recalculate portfolio metrics
        self.recalculate_portfolio_state(&positions_guard, &mut state_guard).await?;

        Ok(())
    }

    /// Get current portfolio state
    pub async fn get_portfolio_state(&self) -> PortfolioState {
        self.state.read().await.clone()
    }

    /// Get current positions
    pub async fn get_positions(&self) -> HashMap<Uuid, Position> {
        self.positions.read().await.clone()
    }

    /// Get performance metrics
    pub async fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.performance_metrics.read().await.clone()
    }

    /// Update existing position with new execution
    async fn update_existing_position(
        &self,
        position: &mut Position,
        execution: &ExecutionResult,
        state: &mut PortfolioState,
    ) -> Result<()> {
        let is_same_side = match (&position.size > &0.0, execution.filled_quantity > 0.0) {
            (true, true) | (false, false) => true,
            _ => false,
        };

        if is_same_side {
            // Adding to position - calculate new average price
            let total_value = position.size.abs() * position.entry_price +
                             execution.filled_quantity.abs() * execution.average_price;
            let total_quantity = position.size.abs() + execution.filled_quantity.abs();

            position.entry_price = total_value / total_quantity;
            position.size += execution.filled_quantity;
        } else {
            // Reducing or closing position
            let reduction = execution.filled_quantity.abs().min(position.size.abs());
            let remaining = position.size.abs() - reduction;

            // Calculate realized P&L for the closed portion
            let price_diff = execution.average_price - position.entry_price;
            let realized_pnl = if position.size > 0.0 {
                price_diff * reduction
            } else {
                -price_diff * reduction
            };

            // Update realized P&L
            state.realized_pnl += realized_pnl;
            state.total_pnl += realized_pnl;

            // Record trade if position is closed
            if remaining <= 0.0001 {
                self.record_closed_trade(position, execution, realized_pnl).await?;
                position.size = 0.0;
            } else {
                // Partial close - update position size
                position.size = if position.size > 0.0 { remaining } else { -remaining };
            }
        }

        Ok(())
    }

    /// Create new position from execution
    async fn create_new_position(&self, execution: &ExecutionResult) -> Result<Position> {
        Ok(Position {
            instrument_id: execution.instrument_id,
            strategy_name: execution.strategy_name.clone(),
            size: execution.filled_quantity,
            entry_price: execution.average_price,
            entry_time: execution.execution_time,
            stop_loss: execution.stop_loss,
            take_profit: execution.take_profit,
            unrealized_pnl: 0.0,
            risk_metrics: RiskMetrics {
                var_95: 0.0,
                expected_shortfall: 0.0,
                max_drawdown: 0.0,
                sharpe_estimate: 0.0,
            },
        })
    }

    /// Update position P&L with current market price
    async fn update_position_pnl(&self, position: &mut Position, current_price: f64) -> Result<()> {
        if position.size.abs() > 0.0001 {
            let price_diff = current_price - position.entry_price;
            position.unrealized_pnl = if position.size > 0.0 {
                price_diff * position.size
            } else {
                -price_diff * position.size.abs()
            };
        }
        Ok(())
    }

    /// Recalculate portfolio state from positions
    async fn recalculate_portfolio_state(
        &self,
        positions: &HashMap<Uuid, Position>,
        state: &mut PortfolioState,
    ) -> Result<()> {
        // Calculate exposure metrics
        let mut gross_long = 0.0;
        let mut gross_short = 0.0;
        let mut total_unrealized_pnl = 0.0;

        for position in positions.values() {
            let position_value = position.size.abs() * position.entry_price;

            if position.size > 0.0 {
                gross_long += position_value;
            } else if position.size < 0.0 {
                gross_short += position_value;
            }

            total_unrealized_pnl += position.unrealized_pnl;
        }

        // Update state
        state.gross_exposure = gross_long + gross_short;
        state.net_exposure = gross_long - gross_short;
        state.total_exposure = state.gross_exposure;
        state.unrealized_pnl = total_unrealized_pnl;
        state.total_pnl = state.realized_pnl + state.unrealized_pnl;

        // Calculate total portfolio value
        state.total_value = state.cash + state.total_pnl;
        state.invested_capital = state.gross_exposure;

        // Calculate leverage
        state.leverage = if state.total_value > 0.0 {
            state.gross_exposure / state.total_value
        } else {
            0.0
        };

        // Calculate margin metrics
        state.margin_used = state.gross_exposure * self.config.margin_requirement;
        state.margin_available = state.total_value - state.margin_used;

        // Update drawdown metrics
        if state.total_value > state.high_water_mark {
            state.high_water_mark = state.total_value;
            state.current_drawdown = 0.0;
        } else {
            state.current_drawdown = (state.high_water_mark - state.total_value) / state.high_water_mark;
            if state.current_drawdown > state.max_drawdown {
                state.max_drawdown = state.current_drawdown;
            }
        }

        state.last_updated = Utc::now();
        Ok(())
    }

    /// Record a closed trade for performance tracking
    async fn record_closed_trade(
        &self,
        position: &Position,
        execution: &ExecutionResult,
        realized_pnl: f64,
    ) -> Result<()> {
        let trade_record = TradeRecord {
            id: Uuid::new_v4(),
            instrument_id: position.instrument_id,
            strategy_name: position.strategy_name.clone(),
            entry_time: position.entry_time,
            exit_time: Some(execution.execution_time),
            entry_price: position.entry_price,
            exit_price: Some(execution.average_price),
            quantity: position.size.abs(),
            side: if position.size > 0.0 { SignalType::Buy } else { SignalType::Sell },
            realized_pnl,
            fees: 0.0, // TODO: Calculate fees
            holding_period: Some(execution.execution_time - position.entry_time),
            max_favorable_excursion: 0.0, // TODO: Track during position lifetime
            max_adverse_excursion: 0.0,   // TODO: Track during position lifetime
            is_closed: true,
        };

        let mut trade_history_guard = self.trade_history.write().await;
        trade_history_guard.push(trade_record);

        // Update performance metrics
        self.update_performance_metrics().await?;

        Ok(())
    }

    /// Update performance metrics based on trade history
    async fn update_performance_metrics(&self) -> Result<()> {
        let trade_history_guard = self.trade_history.read().await;
        let state_guard = self.state.read().await;
        let mut metrics_guard = self.performance_metrics.write().await;

        let closed_trades: Vec<&TradeRecord> = trade_history_guard
            .iter()
            .filter(|t| t.is_closed)
            .collect();

        if closed_trades.is_empty() {
            return Ok(());
        }

        // Basic trade statistics
        metrics_guard.total_trades = closed_trades.len() as u32;
        metrics_guard.winning_trades = closed_trades.iter()
            .filter(|t| t.realized_pnl > 0.0)
            .count() as u32;
        metrics_guard.losing_trades = metrics_guard.total_trades - metrics_guard.winning_trades;

        // Win rate
        metrics_guard.win_rate = if metrics_guard.total_trades > 0 {
            metrics_guard.winning_trades as f64 / metrics_guard.total_trades as f64
        } else {
            0.0
        };

        // P&L statistics
        let total_pnl: f64 = closed_trades.iter().map(|t| t.realized_pnl).sum();
        let winning_pnl: f64 = closed_trades.iter()
            .filter(|t| t.realized_pnl > 0.0)
            .map(|t| t.realized_pnl)
            .sum();
        let losing_pnl: f64 = closed_trades.iter()
            .filter(|t| t.realized_pnl < 0.0)
            .map(|t| t.realized_pnl)
            .sum();

        metrics_guard.avg_win = if metrics_guard.winning_trades > 0 {
            winning_pnl / metrics_guard.winning_trades as f64
        } else {
            0.0
        };

        metrics_guard.avg_loss = if metrics_guard.losing_trades > 0 {
            losing_pnl / metrics_guard.losing_trades as f64
        } else {
            0.0
        };

        metrics_guard.profit_factor = if losing_pnl.abs() > 0.0 {
            winning_pnl / losing_pnl.abs()
        } else {
            0.0
        };

        // Largest win/loss
        metrics_guard.largest_win = closed_trades.iter()
            .map(|t| t.realized_pnl)
            .fold(0.0, f64::max);
        metrics_guard.largest_loss = closed_trades.iter()
            .map(|t| t.realized_pnl)
            .fold(0.0, f64::min);

        // Total return
        metrics_guard.total_return = (state_guard.total_value - self.config.initial_capital) / self.config.initial_capital;

        Ok(())
    }

    /// Check if portfolio constraints are violated
    pub async fn check_constraints(&self) -> Result<Vec<String>> {
        let state_guard = self.state.read().await;
        let positions_guard = self.positions.read().await;
        let mut violations = Vec::new();

        // Check leverage constraint
        if state_guard.leverage > self.config.max_leverage {
            violations.push(format!(
                "Leverage {} exceeds maximum {}",
                state_guard.leverage, self.config.max_leverage
            ));
        }

        // Check daily loss constraint
        if state_guard.daily_pnl < -self.config.max_daily_loss * self.config.initial_capital {
            violations.push(format!(
                "Daily loss {} exceeds maximum {}",
                state_guard.daily_pnl,
                self.config.max_daily_loss * self.config.initial_capital
            ));
        }

        // Check position concentration
        for position in positions_guard.values() {
            let position_value = position.size.abs() * position.entry_price;
            let concentration = position_value / state_guard.total_value;

            if concentration > self.config.max_position_concentration {
                violations.push(format!(
                    "Position {} concentration {} exceeds maximum {}",
                    position.instrument_id,
                    concentration,
                    self.config.max_position_concentration
                ));
            }
        }

        // Check margin availability
        if state_guard.margin_available < 0.0 {
            violations.push(format!(
                "Insufficient margin: {} available",
                state_guard.margin_available
            ));
        }

        Ok(violations)
    }

    /// Calculate portfolio Value at Risk (VaR)
    pub async fn calculate_portfolio_var(&self, confidence_level: f64, days: u32) -> Result<f64> {
        let daily_returns_guard = self.daily_returns.read().await;

        if daily_returns_guard.len() < 30 {
            return Ok(0.0); // Not enough data
        }

        // Calculate historical VaR
        let mut sorted_returns = daily_returns_guard.clone();
        sorted_returns.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let percentile_index = ((1.0 - confidence_level) * sorted_returns.len() as f64) as usize;
        let daily_var = sorted_returns[percentile_index];

        // Scale to multi-day VaR
        let portfolio_value = self.state.read().await.total_value;
        let var = -daily_var * portfolio_value * (days as f64).sqrt();

        Ok(var)
    }

    /// Calculate Expected Shortfall (Conditional VaR)
    pub async fn calculate_expected_shortfall(&self, confidence_level: f64) -> Result<f64> {
        let daily_returns_guard = self.daily_returns.read().await;

        if daily_returns_guard.len() < 30 {
            return Ok(0.0);
        }

        let mut sorted_returns = daily_returns_guard.clone();
        sorted_returns.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let percentile_index = ((1.0 - confidence_level) * sorted_returns.len() as f64) as usize;
        let tail_returns: Vec<f64> = sorted_returns.iter().take(percentile_index).cloned().collect();

        if tail_returns.is_empty() {
            return Ok(0.0);
        }

        let expected_shortfall = tail_returns.iter().sum::<f64>() / tail_returns.len() as f64;
        let portfolio_value = self.state.read().await.total_value;

        Ok(-expected_shortfall * portfolio_value)
    }

    /// Add daily return for risk calculations
    pub async fn add_daily_return(&self, return_rate: f64) {
        let mut daily_returns_guard = self.daily_returns.write().await;
        daily_returns_guard.push(return_rate);

        // Keep only last 252 days (1 year)
        if daily_returns_guard.len() > 252 {
            daily_returns_guard.remove(0);
        }
    }

    /// Get position by instrument ID
    pub async fn get_position(&self, instrument_id: Uuid) -> Option<Position> {
        self.positions.read().await.get(&instrument_id).cloned()
    }

    /// Close position
    pub async fn close_position(&self, instrument_id: Uuid, exit_price: f64) -> Result<Option<f64>> {
        let mut positions_guard = self.positions.write().await;
        let mut state_guard = self.state.write().await;

        if let Some(position) = positions_guard.remove(&instrument_id) {
            // Calculate realized P&L
            let price_diff = exit_price - position.entry_price;
            let realized_pnl = if position.size > 0.0 {
                price_diff * position.size
            } else {
                -price_diff * position.size.abs()
            };

            // Update portfolio state
            state_guard.realized_pnl += realized_pnl;
            state_guard.total_pnl += realized_pnl;

            // Record the trade
            let trade_record = TradeRecord {
                id: Uuid::new_v4(),
                instrument_id: position.instrument_id,
                strategy_name: position.strategy_name.clone(),
                entry_time: position.entry_time,
                exit_time: Some(Utc::now()),
                entry_price: position.entry_price,
                exit_price: Some(exit_price),
                quantity: position.size.abs(),
                side: if position.size > 0.0 { SignalType::Buy } else { SignalType::Sell },
                realized_pnl,
                fees: 0.0,
                holding_period: Some(Utc::now() - position.entry_time),
                max_favorable_excursion: 0.0,
                max_adverse_excursion: 0.0,
                is_closed: true,
            };

            let mut trade_history_guard = self.trade_history.write().await;
            trade_history_guard.push(trade_record);
            drop(trade_history_guard);

            // Recalculate portfolio state
            self.recalculate_portfolio_state(&positions_guard, &mut state_guard).await?;

            Ok(Some(realized_pnl))
        } else {
            Ok(None)
        }
    }

    /// Get trade history
    pub async fn get_trade_history(&self) -> Vec<TradeRecord> {
        self.trade_history.read().await.clone()
    }

    /// Reset daily P&L (called at start of each trading day)
    pub async fn reset_daily_pnl(&self) {
        let mut state_guard = self.state.write().await;
        state_guard.daily_pnl = 0.0;
    }

    /// Get portfolio summary
    pub async fn get_portfolio_summary(&self) -> PortfolioSummary {
        let state = self.state.read().await;
        let positions = self.positions.read().await;
        let metrics = self.performance_metrics.read().await;

        PortfolioSummary {
            total_value: state.total_value,
            cash: state.cash,
            total_pnl: state.total_pnl,
            unrealized_pnl: state.unrealized_pnl,
            realized_pnl: state.realized_pnl,
            daily_pnl: state.daily_pnl,
            leverage: state.leverage,
            num_positions: positions.len(),
            max_drawdown: state.max_drawdown,
            current_drawdown: state.current_drawdown,
            total_return: metrics.total_return,
            sharpe_ratio: metrics.sharpe_ratio,
            win_rate: metrics.win_rate,
            total_trades: metrics.total_trades,
        }
    }
}

// Portfolio Summary for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioSummary {
    pub total_value: f64,
    pub cash: f64,
    pub total_pnl: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
    pub daily_pnl: f64,
    pub leverage: f64,
    pub num_positions: usize,
    pub max_drawdown: f64,
    pub current_drawdown: f64,
    pub total_return: f64,
    pub sharpe_ratio: f64,
    pub win_rate: f64,
    pub total_trades: u32,
}
