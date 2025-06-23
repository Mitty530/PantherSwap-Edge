// Advanced Risk-Reward Enhancement System
use crate::utils::{Result, PantherSwapError};
use crate::trading::signals::{StrategyType, AISignal};
use crate::trading::strategies::StrategyPerformance;
use crate::database::types::{RegimeType, MarketTick};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use tracing::{info, warn, debug};

/// Enhanced risk management configuration with advanced optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskRewardConfig {
    // Advanced Position sizing parameters
    pub base_position_size: f64,
    pub max_position_size: f64,
    pub min_position_size: f64,
    pub volatility_scaling_factor: f64,
    pub confidence_scaling_factor: f64,

    // Enhanced Kelly Criterion parameters
    pub kelly_fraction_limit: f64,
    pub kelly_lookback_periods: u32,
    pub kelly_confidence_threshold: f64,
    pub kelly_safety_margin: f64,
    pub enable_fractional_kelly: bool,

    // Advanced volatility targeting
    pub volatility_target: f64,
    pub volatility_lookback_periods: u32,
    pub volatility_decay_factor: f64,
    pub multi_timeframe_volatility: bool,
    pub volatility_regime_adjustment: bool,

    // Dynamic stop loss parameters
    pub base_stop_loss_pct: f64,
    pub volatility_adjusted_stop_loss: bool,
    pub trailing_stop_enabled: bool,
    pub trailing_stop_distance: f64,
    pub adaptive_stop_loss: bool,
    pub stop_loss_tightening_factor: f64,
    pub regime_based_stops: bool,

    // Dynamic profit target parameters
    pub base_profit_target_pct: f64,
    pub dynamic_profit_targets: bool,
    pub risk_reward_ratio: f64,
    pub profit_scaling_factor: f64,
    pub adaptive_profit_targets: bool,
    pub momentum_profit_extension: bool,
    pub profit_target_optimization: bool,

    // Advanced portfolio risk parameters
    pub max_portfolio_risk: f64,
    pub max_correlation_exposure: f64,
    pub max_sector_concentration: f64,
    pub var_limit_95: f64,
    pub var_limit_99: f64,
    pub expected_shortfall_limit: f64,
    pub stress_test_threshold: f64,

    // Dynamic adjustment parameters
    pub enable_regime_adjustment: bool,
    pub enable_performance_feedback: bool,
    pub risk_adjustment_speed: f64,
    pub performance_lookback_days: u32,
    pub execution_speed_target_ms: f64,
    pub enable_fast_path_optimization: bool,
    pub cache_calculation_results: bool,
}

impl Default for RiskRewardConfig {
    fn default() -> Self {
        Self {
            // Optimized position sizing
            base_position_size: 10000.0,
            max_position_size: 100000.0,
            min_position_size: 1000.0,
            volatility_scaling_factor: 0.6, // Increased for better volatility response
            confidence_scaling_factor: 1.2, // Enhanced confidence weighting

            // Enhanced Kelly Criterion
            kelly_fraction_limit: 0.25, // Conservative 25% of full Kelly
            kelly_lookback_periods: 100, // 100 periods for Kelly calculation
            kelly_confidence_threshold: 0.6, // Minimum confidence for Kelly
            kelly_safety_margin: 0.5, // 50% safety margin on Kelly
            enable_fractional_kelly: true,

            // Advanced volatility targeting
            volatility_target: 0.15, // 15% target volatility
            volatility_lookback_periods: 50, // 50 periods for volatility calculation
            volatility_decay_factor: 0.94, // EWMA decay factor
            multi_timeframe_volatility: true,
            volatility_regime_adjustment: true,

            // Dynamic stop losses
            base_stop_loss_pct: 0.015, // Tighter 1.5% base stop
            volatility_adjusted_stop_loss: true,
            trailing_stop_enabled: true,
            trailing_stop_distance: 0.008, // Tighter 0.8% trailing distance
            adaptive_stop_loss: true,
            stop_loss_tightening_factor: 0.8, // Tighten stops as profit increases
            regime_based_stops: true,

            // Dynamic profit targets
            base_profit_target_pct: 0.035, // Optimized 3.5% base target
            dynamic_profit_targets: true,
            risk_reward_ratio: 2.5, // Enhanced risk-reward ratio
            profit_scaling_factor: 1.8, // Increased profit scaling
            adaptive_profit_targets: true,
            momentum_profit_extension: true,
            profit_target_optimization: true,

            // Enhanced portfolio risk
            max_portfolio_risk: 0.04, // Tighter 4% portfolio risk
            max_correlation_exposure: 0.25, // Reduced correlation exposure
            max_sector_concentration: 0.35, // Reduced sector concentration
            var_limit_95: 0.025, // Tighter VaR limits
            var_limit_99: 0.04, // Tighter extreme VaR
            expected_shortfall_limit: 0.06, // Expected shortfall limit
            stress_test_threshold: 0.08, // Stress test threshold

            // Performance optimization
            enable_regime_adjustment: true,
            enable_performance_feedback: true,
            risk_adjustment_speed: 0.15, // Faster adjustment
            performance_lookback_days: 20, // Shorter lookback for faster adaptation
            execution_speed_target_ms: 8.0, // Target <8ms execution
            enable_fast_path_optimization: true,
            cache_calculation_results: true,
        }
    }
}

/// Enhanced dynamic position sizing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionSizingResult {
    pub recommended_size: f64,
    pub max_allowed_size: f64,
    pub risk_adjusted_size: f64,
    pub kelly_optimal_size: f64,
    pub volatility_targeted_size: f64,

    // Adjustment factors
    pub confidence_factor: f64,
    pub volatility_factor: f64,
    pub regime_factor: f64,
    pub correlation_factor: f64,
    pub kelly_factor: f64,
    pub momentum_factor: f64,

    // Risk metrics
    pub estimated_var_impact: f64,
    pub portfolio_risk_contribution: f64,
    pub expected_return: f64,
    pub max_loss_estimate: f64,

    // Metadata
    pub sizing_method: String,
    pub sizing_reason: String,
    pub calculation_time_ms: f64,
    pub confidence_level: f64,
}

/// Advanced volatility metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilityMetrics {
    pub current_volatility: f64,
    pub ewma_volatility: f64,
    pub garch_volatility: f64,
    pub realized_volatility: f64,
    pub volatility_percentile: f64,
    pub volatility_regime: VolatilityRegime,
    pub multi_timeframe_vol: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VolatilityRegime {
    Low,
    Normal,
    High,
    Extreme,
}

/// Enhanced Kelly Criterion metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KellyMetrics {
    pub kelly_fraction: f64,
    pub safe_kelly_fraction: f64,
    pub win_probability: f64,
    pub average_win: f64,
    pub average_loss: f64,
    pub expected_value: f64,
    pub kelly_confidence: f64,
    pub lookback_periods: u32,
    pub sample_size: u32,
}

/// Risk management decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskManagementDecision {
    pub position_sizing: PositionSizingResult,
    pub stop_loss_price: f64,
    pub profit_target_price: f64,
    pub trailing_stop_price: Option<f64>,
    pub max_risk_amount: f64,
    pub expected_reward: f64,
    pub risk_reward_ratio: f64,
    pub portfolio_risk_impact: f64,
    pub decision_timestamp: DateTime<Utc>,
}

/// Portfolio risk metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioRiskMetrics {
    pub total_exposure: f64,
    pub var_95: f64,
    pub var_99: f64,
    pub expected_shortfall: f64,
    pub correlation_risk: f64,
    pub concentration_risk: f64,
    pub regime_risk: f64,
    pub stress_test_results: HashMap<String, f64>,
    pub last_updated: DateTime<Utc>,
}

/// Performance feedback for risk adjustment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceFeedback {
    pub strategy_type: StrategyType,
    pub realized_pnl: f64,
    pub max_adverse_excursion: f64,
    pub max_favorable_excursion: f64,
    pub hit_ratio: f64,
    pub avg_win: f64,
    pub avg_loss: f64,
    pub profit_factor: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub feedback_period: chrono::Duration,
}



/// Advanced Risk-Reward Enhancement Engine
pub struct RiskRewardEngine {
    config: RiskRewardConfig,
    portfolio_positions: Arc<RwLock<HashMap<Uuid, f64>>>, // instrument_id -> position_size
    portfolio_risk_metrics: Arc<RwLock<PortfolioRiskMetrics>>,
    performance_history: Arc<RwLock<HashMap<StrategyType, Vec<PerformanceFeedback>>>>,
    volatility_tracker: Arc<RwLock<HashMap<Uuid, VecDeque<f64>>>>, // instrument_id -> volatility history
    correlation_matrix: Arc<RwLock<HashMap<(Uuid, Uuid), f64>>>,
    current_regime: Arc<RwLock<Option<RegimeType>>>,
}

impl RiskRewardEngine {
    /// Create new risk-reward enhancement engine
    pub fn new(config: RiskRewardConfig) -> Self {
        Self {
            config,
            portfolio_positions: Arc::new(RwLock::new(HashMap::new())),
            portfolio_risk_metrics: Arc::new(RwLock::new(PortfolioRiskMetrics {
                total_exposure: 0.0,
                var_95: 0.0,
                var_99: 0.0,
                expected_shortfall: 0.0,
                correlation_risk: 0.0,
                concentration_risk: 0.0,
                regime_risk: 0.0,
                stress_test_results: HashMap::new(),
                last_updated: Utc::now(),
            })),
            performance_history: Arc::new(RwLock::new(HashMap::new())),
            volatility_tracker: Arc::new(RwLock::new(HashMap::new())),
            correlation_matrix: Arc::new(RwLock::new(HashMap::new())),
            current_regime: Arc::new(RwLock::new(None)),
        }
    }

    /// Calculate enhanced dynamic position sizing with multiple methods
    pub async fn calculate_position_size(
        &self,
        instrument_id: Uuid,
        strategy_type: StrategyType,
        ai_signal: &AISignal,
        current_price: f64,
        market_data: &[MarketTick],
    ) -> Result<PositionSizingResult> {
        let start_time = std::time::Instant::now();
        debug!("Calculating enhanced position size for instrument: {}", instrument_id);

        // Calculate multiple sizing methods
        let kelly_size = self.calculate_enhanced_kelly_size(ai_signal, market_data).await?;
        let volatility_targeted_size = self.calculate_volatility_targeted_size(instrument_id, market_data).await?;
        let base_size = self.config.base_position_size;

        // Calculate adjustment factors
        let confidence_factor = self.calculate_confidence_factor(ai_signal).await;
        let volatility_factor = self.calculate_volatility_factor(instrument_id, market_data).await?;
        let regime_factor = self.calculate_regime_factor().await;
        let correlation_factor = self.calculate_correlation_factor(instrument_id, strategy_type).await?;
        let kelly_factor = if kelly_size > 0.0 { kelly_size / base_size } else { 1.0 };
        let momentum_factor = self.calculate_momentum_factor(ai_signal).await;

        // Ensemble position sizing (weighted combination)
        let ensemble_size = if self.config.enable_fractional_kelly {
            // Use Kelly as primary with adjustments
            kelly_size * confidence_factor * regime_factor * correlation_factor
        } else {
            // Use base size with all adjustments
            base_size * confidence_factor * volatility_factor * regime_factor * correlation_factor * momentum_factor
        };

        // Apply portfolio risk constraints
        let risk_adjusted_size = self.apply_portfolio_risk_constraints(
            instrument_id,
            ensemble_size,
            current_price,
        ).await?;

        // Apply min/max constraints
        let final_size = risk_adjusted_size
            .max(self.config.min_position_size)
            .min(self.config.max_position_size);

        // Calculate risk metrics
        let estimated_var_impact = self.estimate_var_impact(instrument_id, final_size, current_price).await?;
        let portfolio_risk_contribution = self.calculate_portfolio_risk_impact(instrument_id, final_size, current_price).await?;
        let expected_return = ai_signal.price_predictions.first()
            .map(|p| (p.predicted_price - current_price) / current_price)
            .unwrap_or(0.0);
        let max_loss_estimate = final_size * current_price * self.config.base_stop_loss_pct;

        let calculation_time = start_time.elapsed().as_millis() as f64;
        let sizing_method = if self.config.enable_fractional_kelly { "Enhanced Kelly" } else { "Multi-Factor" };
        let sizing_reason = format!(
            "Method: {}, Kelly: {:.0}, Vol-Target: {:.0}, Factors: C{:.2}/V{:.2}/R{:.2}/Cr{:.2}/M{:.2}",
            sizing_method, kelly_size, volatility_targeted_size,
            confidence_factor, volatility_factor, regime_factor, correlation_factor, momentum_factor
        );

        Ok(PositionSizingResult {
            recommended_size: final_size,
            max_allowed_size: self.config.max_position_size,
            risk_adjusted_size,
            kelly_optimal_size: kelly_size,
            volatility_targeted_size,
            confidence_factor,
            volatility_factor,
            regime_factor,
            correlation_factor,
            kelly_factor,
            momentum_factor,
            estimated_var_impact,
            portfolio_risk_contribution,
            expected_return,
            max_loss_estimate,
            sizing_method: sizing_method.to_string(),
            sizing_reason,
            calculation_time_ms: calculation_time,
            confidence_level: ai_signal.confidence_score,
        })
    }

    /// Generate comprehensive risk management decision
    pub async fn generate_risk_decision(
        &self,
        instrument_id: Uuid,
        strategy_type: StrategyType,
        ai_signal: &AISignal,
        current_price: f64,
        market_data: &[MarketTick],
        trade_direction: i8, // 1 for long, -1 for short
    ) -> Result<RiskManagementDecision> {
        info!("Generating risk management decision for instrument: {}", instrument_id);

        // Calculate position sizing
        let position_sizing = self.calculate_position_size(
            instrument_id,
            strategy_type,
            ai_signal,
            current_price,
            market_data,
        ).await?;

        // Calculate stop loss
        let stop_loss_price = self.calculate_dynamic_stop_loss(
            instrument_id,
            current_price,
            trade_direction,
            market_data,
        ).await?;

        // Calculate profit target
        let profit_target_price = self.calculate_dynamic_profit_target(
            instrument_id,
            current_price,
            trade_direction,
            &stop_loss_price,
            market_data,
        ).await?;

        // Calculate trailing stop if enabled
        let trailing_stop_price = if self.config.trailing_stop_enabled {
            Some(self.calculate_trailing_stop(current_price, trade_direction).await?)
        } else {
            None
        };

        // Calculate risk and reward amounts
        let max_risk_amount = (current_price - stop_loss_price).abs() * position_sizing.recommended_size;
        let expected_reward = (profit_target_price - current_price).abs() * position_sizing.recommended_size;
        let risk_reward_ratio = if max_risk_amount > 0.0 {
            expected_reward / max_risk_amount
        } else {
            0.0
        };

        // Calculate portfolio risk impact
        let portfolio_risk_impact = self.calculate_portfolio_risk_impact(
            instrument_id,
            position_sizing.recommended_size,
            current_price,
        ).await?;

        Ok(RiskManagementDecision {
            position_sizing,
            stop_loss_price,
            profit_target_price,
            trailing_stop_price,
            max_risk_amount,
            expected_reward,
            risk_reward_ratio,
            portfolio_risk_impact,
            decision_timestamp: Utc::now(),
        })
    }
}

use std::collections::VecDeque;

impl RiskRewardEngine {
    /// Calculate enhanced Kelly Criterion position size
    async fn calculate_enhanced_kelly_size(
        &self,
        ai_signal: &AISignal,
        market_data: &[MarketTick],
    ) -> Result<f64> {
        if market_data.len() < self.config.kelly_lookback_periods as usize {
            return Ok(self.config.base_position_size);
        }

        // Calculate returns for Kelly analysis
        let returns = self.calculate_returns(market_data);
        let recent_returns = &returns[returns.len().saturating_sub(self.config.kelly_lookback_periods as usize)..];

        if recent_returns.is_empty() {
            return Ok(self.config.base_position_size);
        }

        // Calculate win probability and average win/loss
        let positive_returns: Vec<f64> = recent_returns.iter().filter(|&&r| r > 0.0).cloned().collect();
        let negative_returns: Vec<f64> = recent_returns.iter().filter(|&&r| r < 0.0).cloned().collect();

        let win_probability = positive_returns.len() as f64 / recent_returns.len() as f64;
        let average_win = if !positive_returns.is_empty() {
            positive_returns.iter().sum::<f64>() / positive_returns.len() as f64
        } else {
            0.0
        };
        let average_loss = if !negative_returns.is_empty() {
            negative_returns.iter().sum::<f64>().abs() / negative_returns.len() as f64
        } else {
            0.0
        };

        // Calculate Kelly fraction: f = (bp - q) / b
        // where b = average_win/average_loss, p = win_probability, q = 1-p
        let kelly_fraction = if average_loss > 0.0 {
            let b = average_win / average_loss;
            let p = win_probability;
            let q = 1.0 - p;
            ((b * p) - q) / b
        } else {
            0.0
        };

        // Apply safety constraints
        let safe_kelly = kelly_fraction
            .max(0.0) // No negative Kelly
            .min(self.config.kelly_fraction_limit); // Limit maximum Kelly

        // Apply confidence threshold
        let kelly_confidence = if recent_returns.len() >= 50 { 1.0 } else { recent_returns.len() as f64 / 50.0 };

        if kelly_confidence < self.config.kelly_confidence_threshold {
            return Ok(self.config.base_position_size);
        }

        // Apply fractional Kelly with safety margin
        let fractional_kelly = safe_kelly * self.config.kelly_safety_margin;

        // Convert Kelly fraction to position size
        let kelly_size = self.config.base_position_size * (1.0 + fractional_kelly * 4.0); // Scale factor

        Ok(kelly_size.max(self.config.min_position_size).min(self.config.max_position_size))
    }

    /// Calculate volatility-targeted position size
    async fn calculate_volatility_targeted_size(
        &self,
        instrument_id: Uuid,
        market_data: &[MarketTick],
    ) -> Result<f64> {
        if !self.config.multi_timeframe_volatility || market_data.is_empty() {
            return Ok(self.config.base_position_size);
        }

        // Calculate current volatility
        let returns = self.calculate_returns(market_data);
        let current_volatility = self.calculate_enhanced_volatility(&returns).await?;

        // Calculate volatility scaling factor
        let volatility_ratio = if current_volatility > 0.0 {
            self.config.volatility_target / current_volatility
        } else {
            1.0
        };

        // Apply volatility targeting
        let vol_targeted_size = self.config.base_position_size * volatility_ratio.sqrt();

        Ok(vol_targeted_size.max(self.config.min_position_size).min(self.config.max_position_size))
    }

    /// Calculate enhanced volatility with multiple methods
    async fn calculate_enhanced_volatility(&self, returns: &[f64]) -> Result<f64> {
        if returns.is_empty() {
            return Ok(0.15); // Default 15% volatility
        }

        // Simple volatility
        let simple_vol = self.calculate_volatility(returns);

        // EWMA volatility
        let ewma_vol = self.calculate_ewma_volatility(returns);

        // Combine volatilities (weighted average)
        let combined_vol = (simple_vol * 0.4) + (ewma_vol * 0.6);

        Ok(combined_vol)
    }

    /// Calculate EWMA (Exponentially Weighted Moving Average) volatility
    fn calculate_ewma_volatility(&self, returns: &[f64]) -> f64 {
        if returns.is_empty() {
            return 0.15;
        }

        let lambda = self.config.volatility_decay_factor;
        let mut ewma_variance = 0.0;
        let mut weight_sum = 0.0;

        for (i, &ret) in returns.iter().enumerate() {
            let weight = lambda.powi(i as i32);
            ewma_variance += weight * ret.powi(2);
            weight_sum += weight;
        }

        if weight_sum > 0.0 {
            (ewma_variance / weight_sum).sqrt()
        } else {
            0.15
        }
    }

    /// Calculate momentum factor for position sizing
    async fn calculate_momentum_factor(&self, ai_signal: &AISignal) -> f64 {
        // Calculate momentum based on AI signal strength and direction consistency
        let signal_strength = ai_signal.confidence_score;

        // Check prediction consistency
        let prediction_consistency = if ai_signal.price_predictions.len() > 1 {
            let predictions: Vec<f64> = ai_signal.price_predictions.iter()
                .map(|p| p.predicted_price)
                .collect();

            let mean_prediction = predictions.iter().sum::<f64>() / predictions.len() as f64;
            let variance = predictions.iter()
                .map(|p| (p - mean_prediction).powi(2))
                .sum::<f64>() / predictions.len() as f64;

            // Lower variance = higher consistency
            1.0 / (1.0 + variance.sqrt())
        } else {
            0.5
        };

        // Combine signal strength and consistency
        let momentum_factor = (signal_strength + prediction_consistency) / 2.0;

        // Scale to reasonable range (0.8 to 1.3)
        0.8 + (momentum_factor * 0.5)
    }
    /// Calculate confidence factor based on AI signal
    async fn calculate_confidence_factor(&self, ai_signal: &AISignal) -> f64 {
        let base_confidence = ai_signal.confidence_score;

        // Adjust based on signal strength
        let signal_strength = if !ai_signal.price_predictions.is_empty() {
            ai_signal.price_predictions.iter()
                .map(|p| p.confidence_score)
                .sum::<f64>() / ai_signal.price_predictions.len() as f64
        } else {
            0.5
        };

        // Combine confidence sources
        let combined_confidence = (base_confidence + signal_strength) / 2.0;

        // Scale confidence factor (0.5 to 1.5 range)
        0.5 + (combined_confidence * self.config.confidence_scaling_factor)
    }

    /// Calculate volatility factor for position sizing
    async fn calculate_volatility_factor(
        &self,
        instrument_id: Uuid,
        market_data: &[MarketTick],
    ) -> Result<f64> {
        if market_data.is_empty() {
            return Ok(1.0);
        }

        // Calculate recent volatility
        let returns = self.calculate_returns(market_data);
        let volatility = self.calculate_volatility(&returns);

        // Update volatility tracker
        {
            let mut tracker = self.volatility_tracker.write().await;
            let vol_history = tracker.entry(instrument_id).or_insert_with(VecDeque::new);
            vol_history.push_back(volatility);

            // Keep only recent history (50 periods)
            if vol_history.len() > 50 {
                vol_history.pop_front();
            }
        }

        // Classify volatility regime
        let vol_regime = self.classify_volatility_regime(volatility);

        // Calculate volatility factor (inverse relationship)
        let vol_factor = match vol_regime {
            VolatilityRegime::Low => 1.2,      // Increase size in low vol
            VolatilityRegime::Normal => 1.0,   // Normal size
            VolatilityRegime::High => 0.8,     // Reduce size in high vol
            VolatilityRegime::Extreme => 0.5,  // Significantly reduce in extreme vol
        };

        Ok(vol_factor * self.config.volatility_scaling_factor + (1.0 - self.config.volatility_scaling_factor))
    }

    /// Calculate regime factor for position sizing
    async fn calculate_regime_factor(&self) -> f64 {
        if !self.config.enable_regime_adjustment {
            return 1.0;
        }

        let current_regime = self.current_regime.read().await;

        match current_regime.as_ref() {
            Some(RegimeType::Normal) => 1.0,     // Normal sizing
            Some(RegimeType::Trending) => 1.1,   // Slightly increase in trending
            Some(RegimeType::Volatile) => 0.8,   // Reduce in volatile markets
            Some(RegimeType::Crisis) => 0.5,     // Significantly reduce in crisis
            None => 0.9,                         // Conservative when regime unknown
        }
    }

    /// Calculate correlation factor to avoid over-concentration
    async fn calculate_correlation_factor(
        &self,
        instrument_id: Uuid,
        strategy_type: StrategyType,
    ) -> Result<f64> {
        let positions = self.portfolio_positions.read().await;
        let correlations = self.correlation_matrix.read().await;

        let mut correlation_exposure = 0.0;
        let mut total_exposure = 0.0;

        for (&other_instrument, &position_size) in positions.iter() {
            if other_instrument != instrument_id {
                let correlation = correlations.get(&(instrument_id, other_instrument))
                    .or_else(|| correlations.get(&(other_instrument, instrument_id)))
                    .unwrap_or(&0.0);

                correlation_exposure += correlation.abs() * position_size;
                total_exposure += position_size;
            }
        }

        let correlation_ratio = if total_exposure > 0.0 {
            correlation_exposure / total_exposure
        } else {
            0.0
        };

        // Reduce position size if high correlation exposure
        if correlation_ratio > self.config.max_correlation_exposure {
            let reduction_factor = self.config.max_correlation_exposure / correlation_ratio;
            Ok(reduction_factor.max(0.3)) // Minimum 30% of normal size
        } else {
            Ok(1.0)
        }
    }

    /// Apply portfolio risk constraints
    async fn apply_portfolio_risk_constraints(
        &self,
        instrument_id: Uuid,
        proposed_size: f64,
        current_price: f64,
    ) -> Result<f64> {
        let positions = self.portfolio_positions.read().await;
        let risk_metrics = self.portfolio_risk_metrics.read().await;

        // Calculate current portfolio exposure
        let current_exposure: f64 = positions.values().sum();
        let proposed_exposure = current_exposure + (proposed_size * current_price);

        // Check portfolio risk limit
        if proposed_exposure > self.config.max_portfolio_risk * 1000000.0 { // Assuming $1M portfolio
            let max_additional_exposure = (self.config.max_portfolio_risk * 1000000.0) - current_exposure;
            let max_size = (max_additional_exposure / current_price).max(0.0);
            return Ok(max_size.min(proposed_size));
        }

        // Check VaR limits
        let estimated_var_impact = self.estimate_var_impact(instrument_id, proposed_size, current_price).await?;
        if risk_metrics.var_95 + estimated_var_impact > self.config.var_limit_95 {
            let var_reduction_factor = (self.config.var_limit_95 - risk_metrics.var_95) / estimated_var_impact;
            return Ok(proposed_size * var_reduction_factor.max(0.0));
        }

        Ok(proposed_size)
    }

    /// Calculate enhanced adaptive stop loss
    async fn calculate_dynamic_stop_loss(
        &self,
        instrument_id: Uuid,
        current_price: f64,
        trade_direction: i8,
        market_data: &[MarketTick],
    ) -> Result<f64> {
        let mut stop_loss_pct = self.config.base_stop_loss_pct;

        // Enhanced volatility adjustment
        if self.config.volatility_adjusted_stop_loss {
            let returns = self.calculate_returns(market_data);
            let volatility = self.calculate_enhanced_volatility(&returns).await?;

            // Adaptive volatility scaling based on regime
            let vol_regime = self.classify_volatility_regime(volatility);
            let vol_multiplier = match vol_regime {
                VolatilityRegime::Low => 0.8,      // Tighter stops in low vol
                VolatilityRegime::Normal => 1.0,   // Normal stops
                VolatilityRegime::High => 1.4,     // Wider stops in high vol
                VolatilityRegime::Extreme => 2.0,  // Much wider stops in extreme vol
            };

            stop_loss_pct *= vol_multiplier;
        }

        // Regime-based stop loss adjustment
        if self.config.regime_based_stops {
            let regime_factor = self.get_enhanced_regime_stop_factor().await;
            stop_loss_pct *= regime_factor;
        }

        // Adaptive stop loss based on recent performance
        if self.config.adaptive_stop_loss {
            let performance_factor = self.get_performance_based_stop_factor(instrument_id).await;
            stop_loss_pct *= performance_factor;
        }

        // Apply minimum and maximum stop loss constraints
        stop_loss_pct = stop_loss_pct.max(0.005).min(0.05); // 0.5% to 5% range

        // Calculate stop loss price
        let stop_loss_price = if trade_direction > 0 {
            current_price * (1.0 - stop_loss_pct)
        } else {
            current_price * (1.0 + stop_loss_pct)
        };

        Ok(stop_loss_price)
    }

    /// Calculate enhanced adaptive profit target
    async fn calculate_dynamic_profit_target(
        &self,
        instrument_id: Uuid,
        current_price: f64,
        trade_direction: i8,
        stop_loss_price: &f64,
        market_data: &[MarketTick],
    ) -> Result<f64> {
        let risk_amount = (current_price - stop_loss_price).abs();

        // Base profit target calculation
        let mut profit_target_pct = if self.config.dynamic_profit_targets {
            // Enhanced risk-reward ratio calculation
            let target_reward = risk_amount * self.config.risk_reward_ratio;
            target_reward / current_price
        } else {
            self.config.base_profit_target_pct
        };

        // Enhanced volatility-based adjustment
        if self.config.adaptive_profit_targets {
            let returns = self.calculate_returns(market_data);
            let volatility = self.calculate_enhanced_volatility(&returns).await?;

            // Volatility regime-based scaling
            let vol_regime = self.classify_volatility_regime(volatility);
            let vol_scaling = match vol_regime {
                VolatilityRegime::Low => 0.8,      // Lower targets in low vol (take profits quickly)
                VolatilityRegime::Normal => 1.0,   // Normal targets
                VolatilityRegime::High => 1.3,     // Higher targets in high vol (let winners run)
                VolatilityRegime::Extreme => 1.6,  // Much higher targets in extreme vol
            };

            profit_target_pct *= vol_scaling;
        }

        // Momentum-based profit extension
        if self.config.momentum_profit_extension {
            let momentum_factor = self.calculate_momentum_profit_factor(market_data).await;
            profit_target_pct *= momentum_factor;
        }

        // Regime-based profit target adjustment
        if self.config.profit_target_optimization {
            let regime_factor = self.get_enhanced_regime_profit_factor().await;
            profit_target_pct *= regime_factor;
        }

        // Performance-based adjustment
        let performance_factor = self.get_performance_based_profit_factor(instrument_id).await;
        profit_target_pct *= performance_factor;

        // Apply constraints (minimum 1%, maximum 10%)
        profit_target_pct = profit_target_pct.max(0.01).min(0.10);

        // Calculate profit target price
        let profit_target_price = if trade_direction > 0 {
            current_price * (1.0 + profit_target_pct)
        } else {
            current_price * (1.0 - profit_target_pct)
        };

        Ok(profit_target_price)
    }

    /// Calculate trailing stop price
    async fn calculate_trailing_stop(&self, current_price: f64, trade_direction: i8) -> Result<f64> {
        let trailing_distance = self.config.trailing_stop_distance;

        let trailing_stop_price = if trade_direction > 0 {
            // Long position - trailing stop below current price
            current_price * (1.0 - trailing_distance)
        } else {
            // Short position - trailing stop above current price
            current_price * (1.0 + trailing_distance)
        };

        Ok(trailing_stop_price)
    }

    /// Calculate portfolio risk impact
    async fn calculate_portfolio_risk_impact(
        &self,
        instrument_id: Uuid,
        position_size: f64,
        current_price: f64,
    ) -> Result<f64> {
        let positions = self.portfolio_positions.read().await;
        let current_exposure: f64 = positions.values().sum();
        let new_exposure = position_size * current_price;

        // Calculate risk impact as percentage of portfolio
        let portfolio_value = 1000000.0; // Assume $1M portfolio
        let risk_impact = new_exposure / portfolio_value;

        Ok(risk_impact)
    }

    /// Estimate VaR impact of new position
    async fn estimate_var_impact(
        &self,
        instrument_id: Uuid,
        position_size: f64,
        current_price: f64,
    ) -> Result<f64> {
        let volatility_tracker = self.volatility_tracker.read().await;

        // Get historical volatility for the instrument
        let volatility = if let Some(vol_history) = volatility_tracker.get(&instrument_id) {
            if !vol_history.is_empty() {
                vol_history.iter().sum::<f64>() / vol_history.len() as f64
            } else {
                0.15 // Default 15% volatility
            }
        } else {
            0.15
        };

        // Estimate VaR impact (simplified calculation)
        let position_value = position_size * current_price;
        let var_impact = position_value * volatility * 1.65; // 95% confidence level

        Ok(var_impact / 1000000.0) // As percentage of $1M portfolio
    }

    /// Get regime-specific stop loss factor
    async fn get_regime_stop_loss_factor(&self) -> f64 {
        let current_regime = self.current_regime.read().await;

        match current_regime.as_ref() {
            Some(RegimeType::Normal) => 1.0,     // Normal stops
            Some(RegimeType::Trending) => 0.8,   // Tighter stops in trending
            Some(RegimeType::Volatile) => 1.5,   // Wider stops in volatile
            Some(RegimeType::Crisis) => 2.0,     // Much wider stops in crisis
            None => 1.2,                         // Conservative when unknown
        }
    }

    /// Get enhanced regime-specific stop loss factor
    async fn get_enhanced_regime_stop_factor(&self) -> f64 {
        let current_regime = self.current_regime.read().await;

        match current_regime.as_ref() {
            Some(RegimeType::Normal) => 1.0,     // Normal stops
            Some(RegimeType::Trending) => 0.7,   // Tighter stops in trending (let trends run)
            Some(RegimeType::Volatile) => 1.6,   // Wider stops in volatile (avoid whipsaws)
            Some(RegimeType::Crisis) => 2.2,     // Much wider stops in crisis (extreme volatility)
            None => 1.3,                         // Conservative when unknown
        }
    }

    /// Get enhanced regime-specific profit target factor
    async fn get_enhanced_regime_profit_factor(&self) -> f64 {
        let current_regime = self.current_regime.read().await;

        match current_regime.as_ref() {
            Some(RegimeType::Normal) => 1.0,     // Normal targets
            Some(RegimeType::Trending) => 1.5,   // Higher targets in trending (ride the trend)
            Some(RegimeType::Volatile) => 0.7,   // Lower targets in volatile (take profits quickly)
            Some(RegimeType::Crisis) => 0.5,     // Much lower targets in crisis (capital preservation)
            None => 0.8,                         // Conservative when unknown
        }
    }

    /// Calculate momentum-based profit factor
    async fn calculate_momentum_profit_factor(&self, market_data: &[MarketTick]) -> f64 {
        if market_data.len() < 10 {
            return 1.0;
        }

        // Calculate short-term momentum (last 10 periods)
        let recent_data = &market_data[market_data.len() - 10..];
        let first_price = recent_data[0].price;
        let last_price = recent_data[recent_data.len() - 1].price;
        let momentum = (last_price - first_price) / first_price;

        // Scale momentum factor (0.8 to 1.4 range)
        let momentum_factor = 1.0 + (momentum.abs() * 2.0).min(0.4);

        // Strong momentum = higher profit targets
        if momentum.abs() > 0.02 { // > 2% momentum
            momentum_factor
        } else {
            1.0
        }
    }

    /// Get performance-based stop loss factor
    async fn get_performance_based_stop_factor(&self, instrument_id: Uuid) -> f64 {
        // This would typically look at recent trade performance for this instrument
        // For now, return a default factor
        // TODO: Implement based on actual performance history
        1.0
    }

    /// Get performance-based profit target factor
    async fn get_performance_based_profit_factor(&self, instrument_id: Uuid) -> f64 {
        // This would typically look at recent trade performance for this instrument
        // For now, return a default factor
        // TODO: Implement based on actual performance history
        1.0
    }

    /// Classify volatility regime
    fn classify_volatility_regime(&self, volatility: f64) -> VolatilityRegime {
        // Annualized volatility thresholds
        let annualized_vol = volatility * (252.0_f64).sqrt(); // Assuming daily data

        if annualized_vol < 0.10 {
            VolatilityRegime::Low
        } else if annualized_vol < 0.20 {
            VolatilityRegime::Normal
        } else if annualized_vol < 0.30 {
            VolatilityRegime::High
        } else {
            VolatilityRegime::Extreme
        }
    }

    /// Update regime information
    pub async fn update_regime(&self, regime: RegimeType) {
        let mut current_regime = self.current_regime.write().await;
        *current_regime = Some(regime);
    }

    /// Get current volatility metrics for an instrument
    pub async fn get_volatility_metrics(&self, instrument_id: Uuid, market_data: &[MarketTick]) -> Result<VolatilityMetrics> {
        let returns = self.calculate_returns(market_data);
        let current_volatility = self.calculate_enhanced_volatility(&returns).await?;
        let ewma_volatility = self.calculate_ewma_volatility(&returns);
        let realized_volatility = self.calculate_volatility(&returns);

        // Calculate volatility percentile (simplified)
        let volatility_tracker = self.volatility_tracker.read().await;
        let volatility_percentile = if let Some(vol_history) = volatility_tracker.get(&instrument_id) {
            if vol_history.len() > 10 {
                let mut sorted_vols: Vec<f64> = vol_history.iter().cloned().collect();
                sorted_vols.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let position = sorted_vols.iter().position(|&v| v >= current_volatility).unwrap_or(sorted_vols.len());
                position as f64 / sorted_vols.len() as f64
            } else {
                0.5
            }
        } else {
            0.5
        };

        let volatility_regime = self.classify_volatility_regime(current_volatility);

        // Multi-timeframe volatility (simplified - would need different timeframe data)
        let mut multi_timeframe_vol = HashMap::new();
        multi_timeframe_vol.insert("1m".to_string(), current_volatility);
        multi_timeframe_vol.insert("5m".to_string(), current_volatility * 0.9);
        multi_timeframe_vol.insert("15m".to_string(), current_volatility * 0.8);
        multi_timeframe_vol.insert("1h".to_string(), current_volatility * 0.7);

        Ok(VolatilityMetrics {
            current_volatility,
            ewma_volatility,
            garch_volatility: current_volatility, // Simplified - would need GARCH implementation
            realized_volatility,
            volatility_percentile,
            volatility_regime,
            multi_timeframe_vol,
        })
    }

    /// Get Kelly Criterion metrics for an instrument
    pub async fn get_kelly_metrics(&self, ai_signal: &AISignal, market_data: &[MarketTick]) -> Result<KellyMetrics> {
        let returns = self.calculate_returns(market_data);
        let lookback_periods = self.config.kelly_lookback_periods.min(returns.len() as u32);
        let recent_returns = &returns[returns.len().saturating_sub(lookback_periods as usize)..];

        if recent_returns.is_empty() {
            return Ok(KellyMetrics {
                kelly_fraction: 0.0,
                safe_kelly_fraction: 0.0,
                win_probability: 0.5,
                average_win: 0.0,
                average_loss: 0.0,
                expected_value: 0.0,
                kelly_confidence: 0.0,
                lookback_periods,
                sample_size: 0,
            });
        }

        let positive_returns: Vec<f64> = recent_returns.iter().filter(|&&r| r > 0.0).cloned().collect();
        let negative_returns: Vec<f64> = recent_returns.iter().filter(|&&r| r < 0.0).cloned().collect();

        let win_probability = positive_returns.len() as f64 / recent_returns.len() as f64;
        let average_win = if !positive_returns.is_empty() {
            positive_returns.iter().sum::<f64>() / positive_returns.len() as f64
        } else {
            0.0
        };
        let average_loss = if !negative_returns.is_empty() {
            negative_returns.iter().sum::<f64>().abs() / negative_returns.len() as f64
        } else {
            0.0
        };

        let kelly_fraction = if average_loss > 0.0 {
            let b = average_win / average_loss;
            let p = win_probability;
            let q = 1.0 - p;
            ((b * p) - q) / b
        } else {
            0.0
        };

        let safe_kelly_fraction = kelly_fraction.max(0.0).min(self.config.kelly_fraction_limit) * self.config.kelly_safety_margin;
        let expected_value = (win_probability * average_win) - ((1.0 - win_probability) * average_loss);
        let kelly_confidence = if recent_returns.len() >= 50 { 1.0 } else { recent_returns.len() as f64 / 50.0 };

        Ok(KellyMetrics {
            kelly_fraction,
            safe_kelly_fraction,
            win_probability,
            average_win,
            average_loss,
            expected_value,
            kelly_confidence,
            lookback_periods,
            sample_size: recent_returns.len() as u32,
        })
    }

    /// Calculate returns from market data
    fn calculate_returns(&self, market_data: &[MarketTick]) -> Vec<f64> {
        if market_data.len() < 2 {
            return Vec::new();
        }

        let mut returns = Vec::new();
        for i in 1..market_data.len() {
            let prev_price = (market_data[i-1].bid_price + market_data[i-1].ask_price) / 2.0;
            let curr_price = (market_data[i].bid_price + market_data[i].ask_price) / 2.0;

            if prev_price > 0.0 {
                let return_val = (curr_price - prev_price) / prev_price;
                returns.push(return_val);
            }
        }

        returns
    }

    /// Calculate volatility from returns
    fn calculate_volatility(&self, returns: &[f64]) -> f64 {
        if returns.len() < 2 {
            return 0.15; // Default volatility
        }

        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / (returns.len() - 1) as f64;

        variance.sqrt() * (252.0_f64).sqrt() // Annualized volatility
    }

    /// Classify volatility regime
    fn classify_volatility_regime(&self, volatility: f64) -> VolatilityRegime {
        if volatility < 0.10 {
            VolatilityRegime::Low
        } else if volatility < 0.20 {
            VolatilityRegime::Normal
        } else if volatility < 0.30 {
            VolatilityRegime::High
        } else {
            VolatilityRegime::Extreme
        }
    }

    /// Update portfolio positions
    pub async fn update_position(&self, instrument_id: Uuid, position_size: f64) -> Result<()> {
        let mut positions = self.portfolio_positions.write().await;

        if position_size == 0.0 {
            positions.remove(&instrument_id);
        } else {
            positions.insert(instrument_id, position_size);
        }

        // Update portfolio risk metrics
        self.update_portfolio_risk_metrics().await?;

        Ok(())
    }

    /// Update portfolio risk metrics
    async fn update_portfolio_risk_metrics(&self) -> Result<()> {
        let positions = self.portfolio_positions.read().await;
        let volatility_tracker = self.volatility_tracker.read().await;
        let correlations = self.correlation_matrix.read().await;

        // Calculate total exposure
        let total_exposure: f64 = positions.values().sum();

        // Calculate portfolio volatility and VaR
        let (portfolio_vol, var_95, var_99) = self.calculate_portfolio_var(&positions, &volatility_tracker, &correlations);

        // Calculate concentration risk
        let concentration_risk = self.calculate_concentration_risk(&positions);

        // Calculate correlation risk
        let correlation_risk = self.calculate_correlation_risk(&positions, &correlations);

        // Update metrics
        {
            let mut metrics = self.portfolio_risk_metrics.write().await;
            metrics.total_exposure = total_exposure;
            metrics.var_95 = var_95;
            metrics.var_99 = var_99;
            metrics.expected_shortfall = var_99 * 1.3; // Approximation
            metrics.correlation_risk = correlation_risk;
            metrics.concentration_risk = concentration_risk;
            metrics.last_updated = Utc::now();
        }

        Ok(())
    }

    /// Calculate portfolio VaR
    fn calculate_portfolio_var(
        &self,
        positions: &HashMap<Uuid, f64>,
        volatility_tracker: &HashMap<Uuid, VecDeque<f64>>,
        correlations: &HashMap<(Uuid, Uuid), f64>,
    ) -> (f64, f64, f64) {
        if positions.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        // Simplified portfolio VaR calculation
        let mut portfolio_variance = 0.0;

        for (&instrument1, &position1) in positions.iter() {
            let vol1 = volatility_tracker.get(&instrument1)
                .and_then(|v| v.back())
                .unwrap_or(&0.15);

            for (&instrument2, &position2) in positions.iter() {
                let vol2 = volatility_tracker.get(&instrument2)
                    .and_then(|v| v.back())
                    .unwrap_or(&0.15);

                let correlation = if instrument1 == instrument2 {
                    1.0
                } else {
                    *correlations.get(&(instrument1, instrument2))
                        .or_else(|| correlations.get(&(instrument2, instrument1)))
                        .unwrap_or(&0.3) // Default correlation
                };

                portfolio_variance += position1 * position2 * vol1 * vol2 * correlation;
            }
        }

        let portfolio_vol = portfolio_variance.sqrt();
        let var_95 = portfolio_vol * 1.65; // 95% confidence
        let var_99 = portfolio_vol * 2.33; // 99% confidence

        (portfolio_vol, var_95, var_99)
    }

    /// Calculate concentration risk
    fn calculate_concentration_risk(&self, positions: &HashMap<Uuid, f64>) -> f64 {
        if positions.is_empty() {
            return 0.0;
        }

        let total_exposure: f64 = positions.values().sum();
        let max_position = positions.values().fold(0.0f64, |a, &b| a.max(b));

        max_position / total_exposure
    }

    /// Calculate correlation risk
    fn calculate_correlation_risk(
        &self,
        positions: &HashMap<Uuid, f64>,
        correlations: &HashMap<(Uuid, Uuid), f64>,
    ) -> f64 {
        if positions.len() < 2 {
            return 0.0;
        }

        let mut total_correlation = 0.0;
        let mut pair_count = 0;

        for (&instrument1, _) in positions.iter() {
            for (&instrument2, _) in positions.iter() {
                if instrument1 != instrument2 {
                    let correlation = correlations.get(&(instrument1, instrument2))
                        .or_else(|| correlations.get(&(instrument2, instrument1)))
                        .unwrap_or(&0.0);

                    total_correlation += correlation.abs();
                    pair_count += 1;
                }
            }
        }

        if pair_count > 0 {
            total_correlation / pair_count as f64
        } else {
            0.0
        }
    }

    /// Update market regime
    pub async fn update_regime(&self, regime: RegimeType) -> Result<()> {
        *self.current_regime.write().await = Some(regime);
        info!("Updated market regime to: {:?}", regime);
        Ok(())
    }

    /// Get current portfolio risk metrics
    pub async fn get_portfolio_risk_metrics(&self) -> PortfolioRiskMetrics {
        self.portfolio_risk_metrics.read().await.clone()
    }

    /// Add performance feedback for risk adjustment
    pub async fn add_performance_feedback(
        &self,
        strategy_type: StrategyType,
        feedback: PerformanceFeedback,
    ) -> Result<()> {
        let mut history = self.performance_history.write().await;
        let strategy_history = history.entry(strategy_type).or_insert_with(Vec::new);

        strategy_history.push(feedback);

        // Keep only recent feedback (last 100 entries)
        if strategy_history.len() > 100 {
            strategy_history.drain(0..strategy_history.len() - 100);
        }

        debug!("Added performance feedback for strategy: {:?}", strategy_type);
        Ok(())
    }
}
