// Advanced Performance Monitoring and Profit Optimization System
use crate::utils::Result;
use crate::trading::signals::StrategyType;
use crate::database::types::{MarketTick, RegimeType};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, Duration};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Performance monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMonitoringConfig {
    // Real-time monitoring
    pub enable_realtime_monitoring: bool,
    pub monitoring_interval_ms: u64,
    pub pnl_calculation_frequency_ms: u64,
    
    // Performance targets
    pub daily_pnl_target: f64,
    pub weekly_pnl_target: f64,
    pub monthly_pnl_target: f64,
    pub max_daily_drawdown: f64,
    pub target_sharpe_ratio: f64,
    pub target_win_rate: f64,
    
    // Risk limits
    pub max_portfolio_risk: f64,
    pub position_size_limit: f64,
    pub correlation_limit: f64,
    pub var_limit_95: f64,
    pub var_limit_99: f64,
    
    // Optimization triggers
    pub enable_auto_optimization: bool,
    pub optimization_trigger_threshold: f64,
    pub performance_lookback_hours: u32,
    pub min_trades_for_optimization: u32,
    
    // Alerting
    pub enable_performance_alerts: bool,
    pub alert_threshold_pnl: f64,
    pub alert_threshold_drawdown: f64,
    pub alert_threshold_risk: f64,
}

impl Default for PerformanceMonitoringConfig {
    fn default() -> Self {
        Self {
            enable_realtime_monitoring: true,
            monitoring_interval_ms: 1000, // 1 second
            pnl_calculation_frequency_ms: 5000, // 5 seconds
            
            daily_pnl_target: 25000.0, // $25,000 daily target
            weekly_pnl_target: 125000.0, // $125,000 weekly target
            monthly_pnl_target: 500000.0, // $500,000 monthly target
            max_daily_drawdown: 0.05, // 5% max daily drawdown
            target_sharpe_ratio: 2.5,
            target_win_rate: 0.65, // 65% win rate target
            
            max_portfolio_risk: 0.1, // 10% portfolio risk
            position_size_limit: 100000.0, // $100k max position
            correlation_limit: 0.7, // 70% max correlation
            var_limit_95: 0.03, // 3% VaR limit
            var_limit_99: 0.05, // 5% VaR limit
            
            enable_auto_optimization: true,
            optimization_trigger_threshold: 0.8, // Trigger when performance < 80% of target
            performance_lookback_hours: 24,
            min_trades_for_optimization: 10,
            
            enable_performance_alerts: true,
            alert_threshold_pnl: -10000.0, // Alert on $10k loss
            alert_threshold_drawdown: 0.03, // Alert on 3% drawdown
            alert_threshold_risk: 0.08, // Alert on 8% risk
        }
    }
}

/// Real-time P&L tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimePnL {
    pub timestamp: DateTime<Utc>,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
    pub total_pnl: f64,
    pub daily_pnl: f64,
    pub weekly_pnl: f64,
    pub monthly_pnl: f64,
    pub ytd_pnl: f64,
    pub position_count: u32,
    pub active_strategies: u32,
    pub portfolio_value: f64,
    pub cash_balance: f64,
    pub margin_used: f64,
    pub margin_available: f64,
}

/// Strategy attribution analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyAttribution {
    pub strategy_type: StrategyType,
    pub pnl_contribution: f64,
    pub pnl_percentage: f64,
    pub trade_count: u32,
    pub win_rate: f64,
    pub avg_win: f64,
    pub avg_loss: f64,
    pub profit_factor: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub current_drawdown: f64,
    pub risk_contribution: f64,
    pub last_updated: DateTime<Utc>,
}

/// Performance benchmarking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBenchmark {
    pub benchmark_name: String,
    pub our_return: f64,
    pub benchmark_return: f64,
    pub alpha: f64,
    pub beta: f64,
    pub tracking_error: f64,
    pub information_ratio: f64,
    pub correlation: f64,
    pub outperformance: f64,
    pub period: String, // "daily", "weekly", "monthly", "ytd"
}

/// Performance alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    pub alert_id: Uuid,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub current_value: f64,
    pub threshold_value: f64,
    pub strategy_type: Option<StrategyType>,
    pub timestamp: DateTime<Utc>,
    pub acknowledged: bool,
}

/// Alert types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertType {
    PnLThreshold,
    DrawdownLimit,
    RiskLimit,
    PerformanceTarget,
    StrategyUnderperformance,
    PositionLimit,
    CorrelationLimit,
    VarLimit,
    TechnicalIssue,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// Optimization recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    pub recommendation_id: Uuid,
    pub recommendation_type: OptimizationType,
    pub target_strategy: Option<StrategyType>,
    pub current_value: f64,
    pub recommended_value: f64,
    pub expected_impact: f64,
    pub confidence: f64,
    pub reasoning: String,
    pub priority: u8, // 1-10, 10 being highest
    pub timestamp: DateTime<Utc>,
    pub implemented: bool,
}

/// Optimization types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OptimizationType {
    StrategyWeightAdjustment,
    PositionSizeOptimization,
    RiskParameterTuning,
    ExecutionAlgorithmChange,
    PortfolioRebalancing,
    StrategyActivation,
    StrategyDeactivation,
    ParameterOptimization,
}

/// Comprehensive performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensiveMetrics {
    pub timestamp: DateTime<Utc>,
    
    // P&L Metrics
    pub realtime_pnl: RealTimePnL,
    pub strategy_attribution: Vec<StrategyAttribution>,
    pub benchmarks: Vec<PerformanceBenchmark>,
    
    // Risk Metrics
    pub portfolio_var_95: f64,
    pub portfolio_var_99: f64,
    pub expected_shortfall: f64,
    pub maximum_drawdown: f64,
    pub current_drawdown: f64,
    pub volatility_annualized: f64,
    pub beta_to_market: f64,
    
    // Trading Metrics
    pub total_trades: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub calmar_ratio: f64,
    pub information_ratio: f64,
    
    // Execution Metrics
    pub average_slippage_bps: f64,
    pub average_execution_time_ms: f64,
    pub fill_rate: f64,
    pub market_impact_bps: f64,
    
    // Target Achievement
    pub daily_target_achievement: f64, // Percentage of daily target achieved
    pub weekly_target_achievement: f64,
    pub monthly_target_achievement: f64,
    pub risk_utilization: f64, // Percentage of risk budget used
}

/// Advanced Performance Monitor
pub struct PerformanceMonitor {
    config: PerformanceMonitoringConfig,
    realtime_pnl: Arc<RwLock<RealTimePnL>>,
    strategy_attributions: Arc<RwLock<HashMap<StrategyType, StrategyAttribution>>>,
    performance_history: Arc<RwLock<VecDeque<ComprehensiveMetrics>>>,
    active_alerts: Arc<RwLock<Vec<PerformanceAlert>>>,
    optimization_recommendations: Arc<RwLock<Vec<OptimizationRecommendation>>>,
    benchmarks: Arc<RwLock<Vec<PerformanceBenchmark>>>,
    trade_history: Arc<RwLock<VecDeque<TradeRecord>>>,
    position_tracker: Arc<RwLock<HashMap<Uuid, PositionInfo>>>,
    market_data_cache: Arc<RwLock<VecDeque<MarketTick>>>,
}

/// Trade record for performance analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    pub trade_id: Uuid,
    pub strategy_type: StrategyType,
    pub instrument_id: Uuid,
    pub side: String, // "buy" or "sell"
    pub quantity: f64,
    pub entry_price: f64,
    pub exit_price: Option<f64>,
    pub entry_time: DateTime<Utc>,
    pub exit_time: Option<DateTime<Utc>>,
    pub pnl: Option<f64>,
    pub commission: f64,
    pub slippage_bps: f64,
    pub market_regime: Option<RegimeType>,
    pub signal_confidence: f64,
}

/// Position information for P&L calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionInfo {
    pub instrument_id: Uuid,
    pub strategy_type: StrategyType,
    pub quantity: f64,
    pub average_price: f64,
    pub current_price: f64,
    pub unrealized_pnl: f64,
    pub market_value: f64,
    pub last_updated: DateTime<Utc>,
}

impl PerformanceMonitor {
    /// Create new performance monitor
    pub fn new(config: PerformanceMonitoringConfig) -> Self {
        Self {
            config,
            realtime_pnl: Arc::new(RwLock::new(RealTimePnL {
                timestamp: Utc::now(),
                unrealized_pnl: 0.0,
                realized_pnl: 0.0,
                total_pnl: 0.0,
                daily_pnl: 0.0,
                weekly_pnl: 0.0,
                monthly_pnl: 0.0,
                ytd_pnl: 0.0,
                position_count: 0,
                active_strategies: 0,
                portfolio_value: 1000000.0, // Start with $1M
                cash_balance: 1000000.0,
                margin_used: 0.0,
                margin_available: 1000000.0,
            })),
            strategy_attributions: Arc::new(RwLock::new(HashMap::new())),
            performance_history: Arc::new(RwLock::new(VecDeque::with_capacity(10000))),
            active_alerts: Arc::new(RwLock::new(Vec::new())),
            optimization_recommendations: Arc::new(RwLock::new(Vec::new())),
            benchmarks: Arc::new(RwLock::new(Vec::new())),
            trade_history: Arc::new(RwLock::new(VecDeque::with_capacity(10000))),
            position_tracker: Arc::new(RwLock::new(HashMap::new())),
            market_data_cache: Arc::new(RwLock::new(VecDeque::with_capacity(1000))),
        }
    }

    /// Start real-time monitoring
    pub async fn start_monitoring(&self) -> Result<()> {
        if !self.config.enable_realtime_monitoring {
            return Ok(());
        }

        // Start monitoring loop
        let monitor = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_millis(monitor.config.monitoring_interval_ms)
            );

            loop {
                interval.tick().await;

                if let Err(e) = monitor.update_realtime_metrics().await {
                    eprintln!("Error updating real-time metrics: {}", e);
                }

                if let Err(e) = monitor.check_performance_alerts().await {
                    eprintln!("Error checking performance alerts: {}", e);
                }

                if monitor.config.enable_auto_optimization {
                    if let Err(e) = monitor.generate_optimization_recommendations().await {
                        eprintln!("Error generating optimization recommendations: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Update real-time P&L and metrics
    async fn update_realtime_metrics(&self) -> Result<()> {
        // Calculate current P&L from positions
        let positions = self.position_tracker.read().await;
        let market_data = self.market_data_cache.read().await;

        let mut total_unrealized_pnl = 0.0;
        let mut total_market_value = 0.0;
        let position_count = positions.len() as u32;

        for (_, position) in positions.iter() {
            // Update position with latest market price if available
            if let Some(latest_tick) = market_data.back() {
                let current_price = (latest_tick.bid_price + latest_tick.ask_price) / 2.0;
                let unrealized_pnl = (current_price - position.average_price) * position.quantity;
                total_unrealized_pnl += unrealized_pnl;
                total_market_value += current_price * position.quantity.abs();
            }
        }

        // Update real-time P&L
        {
            let mut pnl = self.realtime_pnl.write().await;
            pnl.timestamp = Utc::now();
            pnl.unrealized_pnl = total_unrealized_pnl;
            pnl.total_pnl = pnl.realized_pnl + total_unrealized_pnl;
            pnl.position_count = position_count;
            pnl.portfolio_value = pnl.cash_balance + total_market_value;

            // Update daily/weekly/monthly P&L (simplified - would need proper date handling)
            pnl.daily_pnl = pnl.total_pnl; // Simplified
        }

        // Update strategy attributions
        self.update_strategy_attributions().await?;

        Ok(())
    }

    /// Update strategy attribution analysis
    async fn update_strategy_attributions(&self) -> Result<()> {
        let trade_history = self.trade_history.read().await;
        let mut strategy_attributions = self.strategy_attributions.write().await;

        // Group trades by strategy
        let mut strategy_trades: HashMap<StrategyType, Vec<&TradeRecord>> = HashMap::new();

        for trade in trade_history.iter() {
            strategy_trades.entry(trade.strategy_type).or_insert_with(Vec::new).push(trade);
        }

        // Calculate attribution for each strategy
        for (strategy_type, trades) in strategy_trades.iter() {
            let mut total_pnl = 0.0;
            let mut wins = 0;
            let mut losses = 0;
            let mut total_win_amount = 0.0;
            let mut total_loss_amount = 0.0;

            for trade in trades.iter() {
                if let Some(pnl) = trade.pnl {
                    total_pnl += pnl;
                    if pnl > 0.0 {
                        wins += 1;
                        total_win_amount += pnl;
                    } else {
                        losses += 1;
                        total_loss_amount += pnl.abs();
                    }
                }
            }

            let trade_count = trades.len() as u32;
            let win_rate = if trade_count > 0 { wins as f64 / trade_count as f64 } else { 0.0 };
            let avg_win = if wins > 0 { total_win_amount / wins as f64 } else { 0.0 };
            let avg_loss = if losses > 0 { total_loss_amount / losses as f64 } else { 0.0 };
            let profit_factor = if total_loss_amount > 0.0 { total_win_amount / total_loss_amount } else { 0.0 };

            strategy_attributions.insert(*strategy_type, StrategyAttribution {
                strategy_type: *strategy_type,
                pnl_contribution: total_pnl,
                pnl_percentage: 0.0, // Will calculate after all strategies
                trade_count,
                win_rate,
                avg_win,
                avg_loss,
                profit_factor,
                sharpe_ratio: 0.0, // Would need returns series to calculate
                max_drawdown: 0.0, // Would need to calculate from trade sequence
                current_drawdown: 0.0,
                risk_contribution: 0.0, // Would need position data
                last_updated: Utc::now(),
            });
        }

        // Calculate percentage contributions
        let total_pnl: f64 = strategy_attributions.values().map(|attr| attr.pnl_contribution).sum();
        if total_pnl != 0.0 {
            for attribution in strategy_attributions.values_mut() {
                attribution.pnl_percentage = (attribution.pnl_contribution / total_pnl) * 100.0;
            }
        }

        Ok(())
    }

    /// Check for performance alerts
    async fn check_performance_alerts(&self) -> Result<()> {
        if !self.config.enable_performance_alerts {
            return Ok(());
        }

        let pnl = self.realtime_pnl.read().await;
        let mut alerts = self.active_alerts.write().await;

        // Check P&L threshold
        if pnl.daily_pnl < self.config.alert_threshold_pnl {
            alerts.push(PerformanceAlert {
                alert_id: Uuid::new_v4(),
                alert_type: AlertType::PnLThreshold,
                severity: AlertSeverity::Warning,
                message: format!("Daily P&L below threshold: ${:.2}", pnl.daily_pnl),
                current_value: pnl.daily_pnl,
                threshold_value: self.config.alert_threshold_pnl,
                strategy_type: None,
                timestamp: Utc::now(),
                acknowledged: false,
            });
        }

        // Check drawdown
        let current_drawdown = self.calculate_current_drawdown().await?;
        if current_drawdown > self.config.alert_threshold_drawdown {
            alerts.push(PerformanceAlert {
                alert_id: Uuid::new_v4(),
                alert_type: AlertType::DrawdownLimit,
                severity: AlertSeverity::Critical,
                message: format!("Drawdown exceeds threshold: {:.2}%", current_drawdown * 100.0),
                current_value: current_drawdown,
                threshold_value: self.config.alert_threshold_drawdown,
                strategy_type: None,
                timestamp: Utc::now(),
                acknowledged: false,
            });
        }

        // Check risk utilization
        let risk_utilization = self.calculate_risk_utilization().await?;
        if risk_utilization > self.config.alert_threshold_risk {
            alerts.push(PerformanceAlert {
                alert_id: Uuid::new_v4(),
                alert_type: AlertType::RiskLimit,
                severity: AlertSeverity::Warning,
                message: format!("Risk utilization high: {:.2}%", risk_utilization * 100.0),
                current_value: risk_utilization,
                threshold_value: self.config.alert_threshold_risk,
                strategy_type: None,
                timestamp: Utc::now(),
                acknowledged: false,
            });
        }

        // Keep only recent alerts (last 100)
        let alerts_len = alerts.len();
        if alerts_len > 100 {
            alerts.drain(0..alerts_len - 100);
        }

        Ok(())
    }

    /// Generate optimization recommendations
    async fn generate_optimization_recommendations(&self) -> Result<()> {
        let strategy_attributions = self.strategy_attributions.read().await;
        let pnl = self.realtime_pnl.read().await;
        let mut recommendations = self.optimization_recommendations.write().await;

        // Check if daily target achievement is below threshold
        let daily_achievement = pnl.daily_pnl / self.config.daily_pnl_target;
        if daily_achievement < self.config.optimization_trigger_threshold {

            // Recommend strategy weight adjustments for underperforming strategies
            for (strategy_type, attribution) in strategy_attributions.iter() {
                if attribution.pnl_contribution < 0.0 && attribution.trade_count >= self.config.min_trades_for_optimization {
                    recommendations.push(OptimizationRecommendation {
                        recommendation_id: Uuid::new_v4(),
                        recommendation_type: OptimizationType::StrategyWeightAdjustment,
                        target_strategy: Some(*strategy_type),
                        current_value: attribution.pnl_percentage,
                        recommended_value: attribution.pnl_percentage * 0.5, // Reduce weight by 50%
                        expected_impact: attribution.pnl_contribution.abs() * 0.3, // Expect 30% improvement
                        confidence: 0.7,
                        reasoning: format!("Strategy {} underperforming with negative P&L", strategy_type),
                        priority: 8,
                        timestamp: Utc::now(),
                        implemented: false,
                    });
                }

                // Recommend increasing allocation to high-performing strategies
                if attribution.profit_factor > 2.0 && attribution.win_rate > 0.7 {
                    recommendations.push(OptimizationRecommendation {
                        recommendation_id: Uuid::new_v4(),
                        recommendation_type: OptimizationType::StrategyWeightAdjustment,
                        target_strategy: Some(*strategy_type),
                        current_value: attribution.pnl_percentage,
                        recommended_value: attribution.pnl_percentage * 1.2, // Increase weight by 20%
                        expected_impact: attribution.pnl_contribution * 0.2,
                        confidence: 0.8,
                        reasoning: format!("Strategy {} performing well with profit factor {:.2}", strategy_type, attribution.profit_factor),
                        priority: 7,
                        timestamp: Utc::now(),
                        implemented: false,
                    });
                }
            }

            // Recommend portfolio rebalancing if risk utilization is low
            let risk_utilization = self.calculate_risk_utilization().await?;
            if risk_utilization < 0.5 {
                recommendations.push(OptimizationRecommendation {
                    recommendation_id: Uuid::new_v4(),
                    recommendation_type: OptimizationType::PortfolioRebalancing,
                    target_strategy: None,
                    current_value: risk_utilization,
                    recommended_value: 0.7, // Target 70% risk utilization
                    expected_impact: (0.7 - risk_utilization) * self.config.daily_pnl_target,
                    confidence: 0.6,
                    reasoning: "Low risk utilization suggests opportunity for increased allocation".to_string(),
                    priority: 6,
                    timestamp: Utc::now(),
                    implemented: false,
                });
            }
        }

        // Keep only recent recommendations (last 50)
        let recommendations_len = recommendations.len();
        if recommendations_len > 50 {
            recommendations.drain(0..recommendations_len - 50);
        }

        Ok(())
    }

    /// Calculate current drawdown
    async fn calculate_current_drawdown(&self) -> Result<f64> {
        let performance_history = self.performance_history.read().await;

        if performance_history.is_empty() {
            return Ok(0.0);
        }

        // Find peak portfolio value in recent history
        let mut peak_value: f64 = 0.0;
        for metrics in performance_history.iter().rev().take(100) { // Last 100 data points
            peak_value = peak_value.max(metrics.realtime_pnl.portfolio_value);
        }

        let current_pnl = self.realtime_pnl.read().await;
        let current_value = current_pnl.portfolio_value;

        if peak_value > 0.0 {
            Ok((peak_value - current_value) / peak_value)
        } else {
            Ok(0.0)
        }
    }

    /// Calculate risk utilization
    async fn calculate_risk_utilization(&self) -> Result<f64> {
        let positions = self.position_tracker.read().await;
        let pnl = self.realtime_pnl.read().await;

        let total_exposure: f64 = positions.values()
            .map(|pos| pos.market_value)
            .sum();

        let max_allowed_exposure = pnl.portfolio_value * self.config.max_portfolio_risk;

        if max_allowed_exposure > 0.0 {
            Ok(total_exposure / max_allowed_exposure)
        } else {
            Ok(0.0)
        }
    }

    /// Generate comprehensive performance metrics
    pub async fn generate_comprehensive_metrics(&self) -> Result<ComprehensiveMetrics> {
        let pnl = self.realtime_pnl.read().await.clone();
        let strategy_attributions = self.strategy_attributions.read().await;
        let benchmarks = self.benchmarks.read().await.clone();
        let trade_history = self.trade_history.read().await;

        // Calculate trading metrics
        let total_trades = trade_history.len() as u32;
        let winning_trades = trade_history.iter()
            .filter(|trade| trade.pnl.unwrap_or(0.0) > 0.0)
            .count() as u32;
        let losing_trades = total_trades - winning_trades;
        let win_rate = if total_trades > 0 { winning_trades as f64 / total_trades as f64 } else { 0.0 };

        // Calculate profit factor
        let total_wins: f64 = trade_history.iter()
            .filter_map(|trade| trade.pnl)
            .filter(|&pnl| pnl > 0.0)
            .sum();
        let total_losses: f64 = trade_history.iter()
            .filter_map(|trade| trade.pnl)
            .filter(|&pnl| pnl < 0.0)
            .map(|pnl| pnl.abs())
            .sum();
        let profit_factor = if total_losses > 0.0 { total_wins / total_losses } else { 0.0 };

        // Calculate execution metrics
        let avg_slippage = if !trade_history.is_empty() {
            trade_history.iter().map(|trade| trade.slippage_bps).sum::<f64>() / trade_history.len() as f64
        } else {
            0.0
        };

        // Calculate target achievements
        let daily_achievement = if self.config.daily_pnl_target != 0.0 {
            (pnl.daily_pnl / self.config.daily_pnl_target).max(0.0)
        } else {
            0.0
        };

        let weekly_achievement = if self.config.weekly_pnl_target != 0.0 {
            (pnl.weekly_pnl / self.config.weekly_pnl_target).max(0.0)
        } else {
            0.0
        };

        let monthly_achievement = if self.config.monthly_pnl_target != 0.0 {
            (pnl.monthly_pnl / self.config.monthly_pnl_target).max(0.0)
        } else {
            0.0
        };

        let risk_utilization = self.calculate_risk_utilization().await?;
        let current_drawdown = self.calculate_current_drawdown().await?;

        Ok(ComprehensiveMetrics {
            timestamp: Utc::now(),
            realtime_pnl: pnl,
            strategy_attribution: strategy_attributions.values().cloned().collect(),
            benchmarks,

            // Risk metrics (simplified calculations)
            portfolio_var_95: risk_utilization * 0.03, // Simplified VaR
            portfolio_var_99: risk_utilization * 0.05,
            expected_shortfall: risk_utilization * 0.06,
            maximum_drawdown: current_drawdown,
            current_drawdown,
            volatility_annualized: 0.15, // Would need returns series
            beta_to_market: 1.0, // Would need market correlation

            // Trading metrics
            total_trades,
            winning_trades,
            losing_trades,
            win_rate,
            profit_factor,
            sharpe_ratio: 0.0, // Would need returns series
            sortino_ratio: 0.0,
            calmar_ratio: 0.0,
            information_ratio: 0.0,

            // Execution metrics
            average_slippage_bps: avg_slippage,
            average_execution_time_ms: 50.0, // Would track from execution system
            fill_rate: 0.95, // Would track from execution system
            market_impact_bps: avg_slippage * 0.5,

            // Target achievement
            daily_target_achievement: daily_achievement,
            weekly_target_achievement: weekly_achievement,
            monthly_target_achievement: monthly_achievement,
            risk_utilization,
        })
    }

    /// Record a completed trade
    pub async fn record_trade(&self, trade: TradeRecord) -> Result<()> {
        let mut trade_history = self.trade_history.write().await;

        if trade_history.len() >= 10000 {
            trade_history.pop_front();
        }
        trade_history.push_back(trade);

        // Update real-time P&L if trade is closed
        if let Some(pnl) = trade_history.back().unwrap().pnl {
            let mut realtime_pnl = self.realtime_pnl.write().await;
            realtime_pnl.realized_pnl += pnl;
            realtime_pnl.total_pnl = realtime_pnl.realized_pnl + realtime_pnl.unrealized_pnl;
        }

        Ok(())
    }

    /// Update position information
    pub async fn update_position(&self, position: PositionInfo) -> Result<()> {
        let mut positions = self.position_tracker.write().await;

        if position.quantity == 0.0 {
            positions.remove(&position.instrument_id);
        } else {
            positions.insert(position.instrument_id, position);
        }

        Ok(())
    }

    /// Get current performance alerts
    pub async fn get_active_alerts(&self) -> Vec<PerformanceAlert> {
        self.active_alerts.read().await.clone()
    }

    /// Get optimization recommendations
    pub async fn get_optimization_recommendations(&self) -> Vec<OptimizationRecommendation> {
        self.optimization_recommendations.read().await.clone()
    }

    /// Acknowledge an alert
    pub async fn acknowledge_alert(&self, alert_id: Uuid) -> Result<()> {
        let mut alerts = self.active_alerts.write().await;

        if let Some(alert) = alerts.iter_mut().find(|a| a.alert_id == alert_id) {
            alert.acknowledged = true;
        }

        Ok(())
    }

    /// Mark optimization recommendation as implemented
    pub async fn implement_recommendation(&self, recommendation_id: Uuid) -> Result<()> {
        let mut recommendations = self.optimization_recommendations.write().await;

        if let Some(rec) = recommendations.iter_mut().find(|r| r.recommendation_id == recommendation_id) {
            rec.implemented = true;
        }

        Ok(())
    }

    /// Update market data for P&L calculations
    pub async fn update_market_data(&self, tick: MarketTick) -> Result<()> {
        let mut market_data = self.market_data_cache.write().await;

        if market_data.len() >= 1000 {
            market_data.pop_front();
        }
        market_data.push_back(tick);

        Ok(())
    }
}

// Implement Clone for PerformanceMonitor to enable spawning async tasks
impl Clone for PerformanceMonitor {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            realtime_pnl: Arc::clone(&self.realtime_pnl),
            strategy_attributions: Arc::clone(&self.strategy_attributions),
            performance_history: Arc::clone(&self.performance_history),
            active_alerts: Arc::clone(&self.active_alerts),
            optimization_recommendations: Arc::clone(&self.optimization_recommendations),
            benchmarks: Arc::clone(&self.benchmarks),
            trade_history: Arc::clone(&self.trade_history),
            position_tracker: Arc::clone(&self.position_tracker),
            market_data_cache: Arc::clone(&self.market_data_cache),
        }
    }
}
