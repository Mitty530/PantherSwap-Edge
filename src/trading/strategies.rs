use crate::trading::signals::{
    TradingSignal, StrategyType, SignalEvidence, AISignal, MicrostructureAnalysis,
    PredictionResult, RegimeSignal, LiquidityMetrics
};
use crate::database::types::{SignalType, RegimeType};
use crate::utils::Result;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::time::Duration;
use async_trait::async_trait;

// Trading Strategy Trait
#[async_trait]
pub trait TradingStrategy: Send + Sync {
    /// Get the strategy type
    fn strategy_type(&self) -> StrategyType;

    /// Generate trading signal based on market analysis
    async fn generate_signal(
        &self,
        instrument_id: Uuid,
        microstructure: &MicrostructureAnalysis,
        ai_signal: &AISignal,
        confidence_threshold: f64,
    ) -> Result<Option<TradingSignal>>;

    /// Update strategy parameters based on performance
    async fn update_parameters(&mut self, performance_metrics: &StrategyPerformance) -> Result<()>;

    /// Get strategy configuration
    fn get_config(&self) -> &dyn StrategyConfig;
}

// Strategy Configuration Trait
pub trait StrategyConfig: Send + Sync {
    fn min_confidence(&self) -> f64;
    fn max_position_size(&self) -> f64;
    fn time_horizon_range(&self) -> (Duration, Duration);
    fn risk_tolerance(&self) -> f64;
}

// Enhanced Strategy Performance Metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyPerformance {
    pub total_trades: u64,
    pub winning_trades: u64,
    pub total_pnl: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub avg_holding_period: Duration,
    pub success_rate: f64,
    pub avg_return_per_trade: f64,

    // Enhanced metrics for weight optimization
    pub sortino_ratio: f64,
    pub calmar_ratio: f64,
    pub information_ratio: f64,
    pub var_95: f64,
    pub expected_shortfall: f64,
    pub profit_factor: f64,
    pub recovery_factor: f64,
    pub tail_ratio: f64,
    pub skewness: f64,
    pub kurtosis: f64,
    pub daily_returns: Vec<f64>,
    pub rolling_sharpe_30d: f64,
    pub rolling_volatility_30d: f64,
    pub max_consecutive_losses: u32,
    pub avg_win_loss_ratio: f64,
    pub kelly_fraction: f64,
    pub correlation_to_market: f64,
    pub beta: f64,
    pub alpha: f64,
    pub tracking_error: f64,
    pub upside_capture: f64,
    pub downside_capture: f64,
}

impl Default for StrategyPerformance {
    fn default() -> Self {
        Self {
            total_trades: 0,
            winning_trades: 0,
            total_pnl: 0.0,
            sharpe_ratio: 0.0,
            max_drawdown: 0.0,
            avg_holding_period: Duration::from_secs(0),
            success_rate: 0.0,
            avg_return_per_trade: 0.0,

            // Enhanced metrics defaults
            sortino_ratio: 0.0,
            calmar_ratio: 0.0,
            information_ratio: 0.0,
            var_95: 0.0,
            expected_shortfall: 0.0,
            profit_factor: 0.0,
            recovery_factor: 0.0,
            tail_ratio: 0.0,
            skewness: 0.0,
            kurtosis: 0.0,
            daily_returns: Vec::new(),
            rolling_sharpe_30d: 0.0,
            rolling_volatility_30d: 0.0,
            max_consecutive_losses: 0,
            avg_win_loss_ratio: 0.0,
            kelly_fraction: 0.0,
            correlation_to_market: 0.0,
            beta: 0.0,
            alpha: 0.0,
            tracking_error: 0.0,
            upside_capture: 0.0,
            downside_capture: 0.0,
        }
    }
}

// Strategy Factory
pub fn create_strategy(strategy_type: StrategyType) -> Result<Box<dyn TradingStrategy>> {
    match strategy_type {
        StrategyType::PredictiveMarketMaking => {
            Ok(Box::new(PredictiveMarketMaking::new()))
        },
        StrategyType::MicrostructureMomentum => {
            Ok(Box::new(MicrostructureMomentum::new()))
        },
        StrategyType::RegimeArbitrage => {
            Ok(Box::new(RegimeArbitrage::new()))
        },
        StrategyType::LiquidityHarvesting => {
            Ok(Box::new(LiquidityHarvesting::new()))
        },
    }
}

// ============================================================================
// PREDICTIVE MARKET MAKING STRATEGY
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictiveMarketMakingConfig {
    pub min_confidence: f64,
    pub max_position_size: f64,
    pub min_spread_bps: f64,
    pub max_inventory_ratio: f64,
    pub volatility_threshold: f64,
    pub time_horizon: Duration,
    pub risk_tolerance: f64,
}

impl Default for PredictiveMarketMakingConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.6,
            max_position_size: 50000.0,
            min_spread_bps: 2.0,
            max_inventory_ratio: 0.3,
            volatility_threshold: 0.02,
            time_horizon: Duration::from_secs(900), // 15 minutes
            risk_tolerance: 0.02,
        }
    }
}

impl StrategyConfig for PredictiveMarketMakingConfig {
    fn min_confidence(&self) -> f64 { self.min_confidence }
    fn max_position_size(&self) -> f64 { self.max_position_size }
    fn time_horizon_range(&self) -> (Duration, Duration) {
        (Duration::from_secs(300), Duration::from_secs(1800)) // 5-30 minutes
    }
    fn risk_tolerance(&self) -> f64 { self.risk_tolerance }
}

pub struct PredictiveMarketMaking {
    config: PredictiveMarketMakingConfig,
    performance: StrategyPerformance,
    current_inventory: f64,
}

impl PredictiveMarketMaking {
    pub fn new() -> Self {
        Self {
            config: PredictiveMarketMakingConfig::default(),
            performance: StrategyPerformance::default(),
            current_inventory: 0.0,
        }
    }

    pub fn with_config(config: PredictiveMarketMakingConfig) -> Self {
        Self {
            config,
            performance: StrategyPerformance::default(),
            current_inventory: 0.0,
        }
    }
}

#[async_trait]
impl TradingStrategy for PredictiveMarketMaking {
    fn strategy_type(&self) -> StrategyType {
        StrategyType::PredictiveMarketMaking
    }

    async fn generate_signal(
        &self,
        instrument_id: Uuid,
        microstructure: &MicrostructureAnalysis,
        ai_signal: &AISignal,
        confidence_threshold: f64,
    ) -> Result<Option<TradingSignal>> {
        // Market making strategy: provide liquidity where it will be needed

        // Check if AI predicts increased volatility (good for market making)
        let volatility_prediction = ai_signal.price_predictions
            .iter()
            .find(|p| p.horizon == Duration::from_secs(900)) // 15 minutes
            .map(|p| {
                // Estimate volatility from prediction interval width
                let interval_width = p.prediction_interval.1 - p.prediction_interval.0;
                interval_width / p.predicted_price.max(1e-8)
            })
            .unwrap_or(0.0);

        // Check microstructure signals for liquidity gaps
        let liquidity_imbalance = microstructure.liquidity_metrics
            .as_ref()
            .map(|lm| lm.imbalance_ratio)
            .unwrap_or(0.0);

        // Market making is profitable when:
        // 1. Sufficient volatility for spread capture
        // 2. Liquidity imbalances to exploit
        // 3. Stable market regime (not crisis)

        let regime_stability = ai_signal.regime_signal
            .as_ref()
            .map(|rs| match rs.current_regime {
                RegimeType::Normal => 1.0,
                RegimeType::Trending => 0.7,
                RegimeType::Volatile => 0.5,
                RegimeType::Crisis => 0.1,
                RegimeType::Bullish => 0.8,
                RegimeType::Bearish => 0.6,
                RegimeType::Sideways => 0.9,
                RegimeType::HighVolatility => 0.4,
            })
            .unwrap_or(0.5);

        // Check volatility threshold
        if volatility_prediction < self.config.volatility_threshold {
            return Ok(None);
        }

        // Check inventory limits
        if self.current_inventory.abs() > self.config.max_inventory_ratio * self.config.max_position_size {
            return Ok(None);
        }

        let microstructure_score = liquidity_imbalance.abs().min(1.0);
        let ai_score = (volatility_prediction * 10.0).min(1.0); // Scale volatility to 0-1
        let regime_score = regime_stability;

        let overall_score = (microstructure_score + ai_score + regime_score) / 3.0;

        if overall_score < confidence_threshold.max(self.config.min_confidence) {
            return Ok(None);
        }

        let signal_type = if liquidity_imbalance > 0.1 {
            SignalType::Sell // Provide ask liquidity
        } else if liquidity_imbalance < -0.1 {
            SignalType::Buy  // Provide bid liquidity
        } else {
            SignalType::Hold
        };

        if matches!(signal_type, SignalType::Hold) {
            return Ok(None);
        }

        Ok(Some(TradingSignal {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            instrument_id,
            strategy_type: StrategyType::PredictiveMarketMaking,
            signal_type,
            signal_strength: liquidity_imbalance.abs(),
            confidence_score: overall_score,
            urgency_score: Some(0.3), // Market making is patient
            entry_price: None, // Will be set by execution engine
            stop_loss: None,   // Market making uses inventory management
            take_profit: None,
            time_horizon: Some(self.config.time_horizon),
            expected_return: Some(volatility_prediction * 0.5), // Half of predicted volatility
            max_risk: Some(self.config.risk_tolerance),
            supporting_evidence: crate::trading::signals::SignalEvidence {
                microstructure_score,
                ai_prediction_score: ai_score,
                regime_score,
                liquidity_score: liquidity_imbalance.abs(),
                risk_reward_ratio: 2.0, // Conservative market making
            },
        }))
    }

    async fn update_parameters(&mut self, performance_metrics: &StrategyPerformance) -> Result<()> {
        // Adaptive parameter adjustment based on performance
        if performance_metrics.success_rate < 0.4 {
            // Increase confidence threshold if success rate is low
            self.config.min_confidence = (self.config.min_confidence * 1.1).min(0.8);
        } else if performance_metrics.success_rate > 0.7 {
            // Decrease confidence threshold if success rate is high
            self.config.min_confidence = (self.config.min_confidence * 0.95).max(0.5);
        }

        // Adjust volatility threshold based on Sharpe ratio
        if performance_metrics.sharpe_ratio < 0.5 {
            self.config.volatility_threshold *= 1.1;
        } else if performance_metrics.sharpe_ratio > 1.5 {
            self.config.volatility_threshold *= 0.95;
        }

        self.performance = performance_metrics.clone();
        Ok(())
    }

    fn get_config(&self) -> &dyn StrategyConfig {
        &self.config
    }
}

// ============================================================================
// MICROSTRUCTURE MOMENTUM STRATEGY
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrostructureMomentumConfig {
    pub min_confidence: f64,
    pub max_position_size: f64,
    pub min_momentum_threshold: f64,
    pub direction_consistency_weight: f64,
    pub time_horizon: Duration,
    pub risk_tolerance: f64,
}

impl Default for MicrostructureMomentumConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.7,
            max_position_size: 75000.0,
            min_momentum_threshold: 0.005, // 0.5% minimum momentum
            direction_consistency_weight: 0.3,
            time_horizon: Duration::from_secs(300), // 5 minutes
            risk_tolerance: 0.01,
        }
    }
}

impl StrategyConfig for MicrostructureMomentumConfig {
    fn min_confidence(&self) -> f64 { self.min_confidence }
    fn max_position_size(&self) -> f64 { self.max_position_size }
    fn time_horizon_range(&self) -> (Duration, Duration) {
        (Duration::from_secs(60), Duration::from_secs(600)) // 1-10 minutes
    }
    fn risk_tolerance(&self) -> f64 { self.risk_tolerance }
}

pub struct MicrostructureMomentum {
    config: MicrostructureMomentumConfig,
    performance: StrategyPerformance,
}

impl MicrostructureMomentum {
    pub fn new() -> Self {
        Self {
            config: MicrostructureMomentumConfig::default(),
            performance: StrategyPerformance::default(),
        }
    }

    pub fn with_config(config: MicrostructureMomentumConfig) -> Self {
        Self {
            config,
            performance: StrategyPerformance::default(),
        }
    }
}

#[async_trait]
impl TradingStrategy for MicrostructureMomentum {
    fn strategy_type(&self) -> StrategyType {
        StrategyType::MicrostructureMomentum
    }

    async fn generate_signal(
        &self,
        instrument_id: Uuid,
        microstructure: &MicrostructureAnalysis,
        ai_signal: &AISignal,
        confidence_threshold: f64,
    ) -> Result<Option<TradingSignal>> {
        // Momentum strategy: follow short-term price movements predicted by AI

        let price_prediction_1min = ai_signal.price_predictions
            .iter()
            .find(|p| p.horizon == Duration::from_secs(60)) // 1 minute
            .cloned();

        let price_prediction_5min = ai_signal.price_predictions
            .iter()
            .find(|p| p.horizon == Duration::from_secs(300)) // 5 minutes
            .cloned();

        if price_prediction_1min.is_none() || price_prediction_5min.is_none() {
            return Ok(None);
        }

        let pred_1min = price_prediction_1min.unwrap();
        let pred_5min = price_prediction_5min.unwrap();

        // Check for consistent momentum direction
        let current_price = microstructure.current_price;
        let momentum_1min = (pred_1min.predicted_price - current_price) / current_price;
        let momentum_5min = (pred_5min.predicted_price - current_price) / current_price;

        // Momentum signal requires:
        // 1. Consistent direction across timeframes
        // 2. High confidence in predictions
        // 3. Trending regime (not ranging)

        let direction_consistency = if momentum_1min * momentum_5min > 0.0 { 1.0 } else { 0.0 };
        let momentum_strength = (momentum_1min.abs() + momentum_5min.abs()) / 2.0;

        // Check minimum momentum threshold
        if momentum_strength < self.config.min_momentum_threshold {
            return Ok(None);
        }

        let regime_trending = ai_signal.regime_signal
            .as_ref()
            .map(|rs| match rs.current_regime {
                RegimeType::Trending => 1.0,
                RegimeType::Volatile => 0.7,
                RegimeType::Normal => 0.3,
                RegimeType::Crisis => 0.1,
                RegimeType::Bullish => 0.9,
                RegimeType::Bearish => 0.8,
                RegimeType::Sideways => 0.2,
                RegimeType::HighVolatility => 0.6,
            })
            .unwrap_or(0.5);

        let ai_confidence = (pred_1min.confidence_score + pred_5min.confidence_score) / 2.0;

        let overall_score =
            0.4 * ai_confidence +
            0.3 * regime_trending +
            self.config.direction_consistency_weight * direction_consistency +
            0.2 * momentum_strength.min(1.0);

        if overall_score < confidence_threshold.max(self.config.min_confidence) {
            return Ok(None);
        }

        let signal_type = if momentum_1min > 0.0 {
            SignalType::Buy
        } else {
            SignalType::Sell
        };

        Ok(Some(TradingSignal {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            instrument_id,
            strategy_type: StrategyType::MicrostructureMomentum,
            signal_type,
            signal_strength: momentum_strength,
            confidence_score: overall_score,
            urgency_score: Some(0.8), // Momentum requires quick execution
            entry_price: Some(current_price),
            stop_loss: Some(current_price * (1.0 - self.config.risk_tolerance * momentum_1min.signum())),
            take_profit: Some(pred_5min.predicted_price),
            time_horizon: Some(self.config.time_horizon),
            expected_return: Some(momentum_5min),
            max_risk: Some(self.config.risk_tolerance),
            supporting_evidence: crate::trading::signals::SignalEvidence {
                microstructure_score: 0.5,
                ai_prediction_score: ai_confidence,
                regime_score: regime_trending,
                liquidity_score: 0.5,
                risk_reward_ratio: momentum_5min.abs() / self.config.risk_tolerance,
            },
        }))
    }

    async fn update_parameters(&mut self, performance_metrics: &StrategyPerformance) -> Result<()> {
        // Adaptive parameter adjustment
        if performance_metrics.success_rate < 0.5 {
            self.config.min_momentum_threshold *= 1.1;
            self.config.min_confidence = (self.config.min_confidence * 1.05).min(0.85);
        } else if performance_metrics.success_rate > 0.75 {
            self.config.min_momentum_threshold *= 0.95;
            self.config.min_confidence = (self.config.min_confidence * 0.98).max(0.6);
        }

        self.performance = performance_metrics.clone();
        Ok(())
    }

    fn get_config(&self) -> &dyn StrategyConfig {
        &self.config
    }
}

// ============================================================================
// REGIME ARBITRAGE STRATEGY
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeArbitrageConfig {
    pub min_confidence: f64,
    pub max_position_size: f64,
    pub min_transition_probability: f64,
    pub time_horizon: Duration,
    pub risk_tolerance: f64,
}

impl Default for RegimeArbitrageConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.8,
            max_position_size: 60000.0,
            min_transition_probability: 0.3,
            time_horizon: Duration::from_secs(3600), // 1 hour
            risk_tolerance: 0.03,
        }
    }
}

impl StrategyConfig for RegimeArbitrageConfig {
    fn min_confidence(&self) -> f64 { self.min_confidence }
    fn max_position_size(&self) -> f64 { self.max_position_size }
    fn time_horizon_range(&self) -> (Duration, Duration) {
        (Duration::from_secs(1800), Duration::from_secs(7200)) // 30 minutes - 2 hours
    }
    fn risk_tolerance(&self) -> f64 { self.risk_tolerance }
}

pub struct RegimeArbitrage {
    config: RegimeArbitrageConfig,
    performance: StrategyPerformance,
}

impl RegimeArbitrage {
    pub fn new() -> Self {
        Self {
            config: RegimeArbitrageConfig::default(),
            performance: StrategyPerformance::default(),
        }
    }
}

#[async_trait]
impl TradingStrategy for RegimeArbitrage {
    fn strategy_type(&self) -> StrategyType {
        StrategyType::RegimeArbitrage
    }

    async fn generate_signal(
        &self,
        instrument_id: Uuid,
        microstructure: &MicrostructureAnalysis,
        ai_signal: &AISignal,
        confidence_threshold: f64,
    ) -> Result<Option<TradingSignal>> {
        let regime_signal = match &ai_signal.regime_signal {
            Some(rs) => rs,
            None => return Ok(None),
        };

        if regime_signal.transition_probability < self.config.min_transition_probability
            || regime_signal.confidence < self.config.min_confidence {
            return Ok(None);
        }

        let (signal_type, expected_return) = match regime_signal.current_regime {
            RegimeType::Crisis => (SignalType::Sell, -0.02),
            RegimeType::Volatile => (SignalType::Sell, -0.01),
            RegimeType::HighVolatility => (SignalType::Sell, -0.015),
            RegimeType::Trending => {
                let price_pred = match ai_signal.price_predictions.first() {
                    Some(p) => p,
                    None => return Ok(None),
                };
                let return_est = (price_pred.predicted_price - microstructure.current_price) / microstructure.current_price;
                (if return_est > 0.0 { SignalType::Buy } else { SignalType::Sell }, return_est)
            },
            RegimeType::Bullish => {
                let price_pred = match ai_signal.price_predictions.first() {
                    Some(p) => p,
                    None => return Ok(None),
                };
                let return_est = (price_pred.predicted_price - microstructure.current_price) / microstructure.current_price;
                (SignalType::Buy, return_est.max(0.01))
            },
            RegimeType::Bearish => {
                let price_pred = match ai_signal.price_predictions.first() {
                    Some(p) => p,
                    None => return Ok(None),
                };
                let return_est = (price_pred.predicted_price - microstructure.current_price) / microstructure.current_price;
                (SignalType::Sell, return_est.min(-0.01))
            },
            RegimeType::Normal => return Ok(None),
            RegimeType::Sideways => return Ok(None),
        };

        Ok(Some(TradingSignal {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            instrument_id,
            strategy_type: StrategyType::RegimeArbitrage,
            signal_type,
            signal_strength: regime_signal.transition_probability,
            confidence_score: regime_signal.confidence,
            urgency_score: Some(regime_signal.transition_probability),
            entry_price: Some(microstructure.current_price),
            stop_loss: Some(microstructure.current_price * 0.98),
            take_profit: None,
            time_horizon: Some(self.config.time_horizon),
            expected_return: Some(expected_return),
            max_risk: Some(self.config.risk_tolerance),
            supporting_evidence: crate::trading::signals::SignalEvidence {
                microstructure_score: 0.3,
                ai_prediction_score: 0.3,
                regime_score: regime_signal.confidence,
                liquidity_score: 0.3,
                risk_reward_ratio: expected_return.abs() / self.config.risk_tolerance,
            },
        }))
    }

    async fn update_parameters(&mut self, performance_metrics: &StrategyPerformance) -> Result<()> {
        self.performance = performance_metrics.clone();
        Ok(())
    }

    fn get_config(&self) -> &dyn StrategyConfig {
        &self.config
    }
}

// ============================================================================
// LIQUIDITY HARVESTING STRATEGY
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityHarvestingConfig {
    pub min_confidence: f64,
    pub max_position_size: f64,
    pub min_imbalance_threshold: f64,
    pub time_horizon: Duration,
    pub risk_tolerance: f64,
}

impl Default for LiquidityHarvestingConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.7,
            max_position_size: 40000.0,
            min_imbalance_threshold: 0.2,
            time_horizon: Duration::from_secs(300), // 5 minutes
            risk_tolerance: 0.005,
        }
    }
}

impl StrategyConfig for LiquidityHarvestingConfig {
    fn min_confidence(&self) -> f64 { self.min_confidence }
    fn max_position_size(&self) -> f64 { self.max_position_size }
    fn time_horizon_range(&self) -> (Duration, Duration) {
        (Duration::from_secs(60), Duration::from_secs(600)) // 1-10 minutes
    }
    fn risk_tolerance(&self) -> f64 { self.risk_tolerance }
}

pub struct LiquidityHarvesting {
    config: LiquidityHarvestingConfig,
    performance: StrategyPerformance,
}

impl LiquidityHarvesting {
    pub fn new() -> Self {
        Self {
            config: LiquidityHarvestingConfig::default(),
            performance: StrategyPerformance::default(),
        }
    }
}

#[async_trait]
impl TradingStrategy for LiquidityHarvesting {
    fn strategy_type(&self) -> StrategyType {
        StrategyType::LiquidityHarvesting
    }

    async fn generate_signal(
        &self,
        instrument_id: Uuid,
        microstructure: &MicrostructureAnalysis,
        ai_signal: &AISignal,
        confidence_threshold: f64,
    ) -> Result<Option<TradingSignal>> {
        let liquidity_metrics = match &microstructure.liquidity_metrics {
            Some(lm) => lm,
            None => return Ok(None),
        };

        if liquidity_metrics.imbalance_ratio.abs() < self.config.min_imbalance_threshold {
            return Ok(None);
        }

        let liquidity_prediction = ai_signal.price_predictions
            .iter()
            .find(|p| p.horizon == Duration::from_secs(300))
            .map(|p| p.confidence_score)
            .unwrap_or(0.5);

        let signal_type = if liquidity_metrics.imbalance_ratio > self.config.min_imbalance_threshold {
            SignalType::Sell
        } else {
            SignalType::Buy
        };

        let confidence = liquidity_metrics.imbalance_ratio.abs() * liquidity_prediction;

        if confidence < confidence_threshold.max(self.config.min_confidence) {
            return Ok(None);
        }

        Ok(Some(TradingSignal {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            instrument_id,
            strategy_type: StrategyType::LiquidityHarvesting,
            signal_type,
            signal_strength: liquidity_metrics.imbalance_ratio.abs(),
            confidence_score: confidence,
            urgency_score: Some(0.9),
            entry_price: Some(microstructure.current_price),
            stop_loss: Some(microstructure.current_price * 0.995),
            take_profit: Some(microstructure.current_price * (1.0 + 0.002 * liquidity_metrics.imbalance_ratio.signum())),
            time_horizon: Some(self.config.time_horizon),
            expected_return: Some(0.002),
            max_risk: Some(self.config.risk_tolerance),
            supporting_evidence: crate::trading::signals::SignalEvidence {
                microstructure_score: liquidity_metrics.imbalance_ratio.abs(),
                ai_prediction_score: liquidity_prediction,
                regime_score: 0.5,
                liquidity_score: 1.0,
                risk_reward_ratio: 0.002 / self.config.risk_tolerance,
            },
        }))
    }

    async fn update_parameters(&mut self, performance_metrics: &StrategyPerformance) -> Result<()> {
        self.performance = performance_metrics.clone();
        Ok(())
    }

    fn get_config(&self) -> &dyn StrategyConfig {
        &self.config
    }
}
