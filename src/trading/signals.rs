use crate::database::types::{SignalType, OrderType, TimeInForce, ExecutionStyle, RegimeType};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::time::Duration;
use std::collections::HashMap;

// Strategy Types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum StrategyType {
    PredictiveMarketMaking,
    MicrostructureMomentum,
    RegimeArbitrage,
    LiquidityHarvesting,
}

impl std::fmt::Display for StrategyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StrategyType::PredictiveMarketMaking => write!(f, "predictive_market_making"),
            StrategyType::MicrostructureMomentum => write!(f, "microstructure_momentum"),
            StrategyType::RegimeArbitrage => write!(f, "regime_arbitrage"),
            StrategyType::LiquidityHarvesting => write!(f, "liquidity_harvesting"),
        }
    }
}

// Enhanced Trading Signal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingSignal {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub instrument_id: Uuid,
    pub strategy_type: StrategyType,
    pub signal_type: SignalType,
    pub signal_strength: f64,        // -1.0 to 1.0
    pub confidence_score: f64,       // 0.0 to 1.0
    pub urgency_score: Option<f64>,  // 0.0 to 1.0
    pub entry_price: Option<f64>,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub time_horizon: Option<Duration>,
    pub expected_return: Option<f64>,
    pub max_risk: Option<f64>,
    pub supporting_evidence: SignalEvidence,
}

impl TradingSignal {
    /// Convert to database TradingSignal format
    pub fn to_database_signal(&self) -> crate::database::types::TradingSignal {
        use rust_decimal::Decimal;

        crate::database::types::TradingSignal {
            id: self.id,
            timestamp: self.timestamp,
            instrument_id: self.instrument_id,
            strategy_name: self.strategy_type.to_string(),
            signal_type: self.signal_type.clone(),
            signal_strength: Decimal::from_f64_retain(self.signal_strength).unwrap_or_default(),
            confidence_score: Decimal::from_f64_retain(self.confidence_score).unwrap_or_default(),
            recommended_size: Decimal::from_f64_retain(1.0).unwrap_or_default(), // Default size
            entry_price: self.entry_price.and_then(|p| Decimal::from_f64_retain(p)),
            stop_loss: self.stop_loss.and_then(|p| Decimal::from_f64_retain(p)),
            take_profit: self.take_profit.and_then(|p| Decimal::from_f64_retain(p)),
            time_horizon: self.time_horizon.and_then(|d| chrono::Duration::from_std(d).ok()),
            expected_return: self.expected_return.and_then(|r| Decimal::from_f64_retain(r)),
            risk_metrics: Some(serde_json::to_value(&self.supporting_evidence).unwrap_or_default()),
        }
    }
}

// Signal Evidence Structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalEvidence {
    pub microstructure_score: f64,
    pub ai_prediction_score: f64,
    pub regime_score: f64,
    pub liquidity_score: f64,
    pub risk_reward_ratio: f64,
}

// Trading Decision Structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingDecision {
    pub instrument_id: Uuid,
    pub strategy_type: StrategyType,
    pub signal: TradingSignal,
    pub risk_assessment: RiskAssessment,
    pub execution_plan: ExecutionPlan,
    pub confidence_score: f64,
    pub expected_pnl: f64,
}

// Risk Assessment Structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub is_acceptable: bool,
    pub adjusted_position_size: f64,
    pub var_95: f64,
    pub expected_shortfall: f64,
    pub max_drawdown_risk: f64,
    pub correlation_risk: f64,
    pub liquidity_risk: f64,
}

// Execution Plan Structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub order_type: OrderType,
    pub quantity: f64,
    pub price: Option<f64>,
    pub time_in_force: TimeInForce,
    pub execution_style: ExecutionStyle,
}

// Position Structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub instrument_id: Uuid,
    pub strategy_name: String,
    pub size: f64,
    pub entry_price: f64,
    pub entry_time: DateTime<Utc>,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub unrealized_pnl: f64,
    pub risk_metrics: RiskMetrics,
}

// Risk Metrics Structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    pub var_95: f64,
    pub expected_shortfall: f64,
    pub max_drawdown: f64,
    pub sharpe_estimate: f64,
}

// Order Request Structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrderRequest {
    pub instrument_id: Uuid,
    pub side: SignalType,
    pub quantity: f64,
    pub order_type: OrderType,
    pub price: Option<f64>,
    pub time_in_force: TimeInForce,
}

// Execution Result Structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub instrument_id: Uuid,
    pub strategy_name: String,
    pub filled_quantity: f64,
    pub average_price: f64,
    pub execution_time: DateTime<Utc>,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
}

// Signal Generator Configuration with Dynamic Optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalWeights {
    pub microstructure: f64,
    pub ai_prediction: f64,
    pub regime_detection: f64,
    pub liquidity_analysis: f64,
    pub last_updated: DateTime<Utc>,
    pub optimization_reason: String,
    pub performance_score: f64,
    pub regime_specific: HashMap<RegimeType, SignalWeights>,
}

impl Default for SignalWeights {
    fn default() -> Self {
        let mut regime_specific = HashMap::new();

        // Trending market weights - favor AI and momentum
        regime_specific.insert(RegimeType::Trending, SignalWeights {
            microstructure: 0.45,   // Higher microstructure for momentum
            ai_prediction: 0.40,    // Higher AI for trend prediction
            regime_detection: 0.10, // Lower regime (already trending)
            liquidity_analysis: 0.05, // Lower liquidity focus
            last_updated: Utc::now(),
            optimization_reason: "Trending regime default".to_string(),
            performance_score: 0.0,
            regime_specific: HashMap::new(),
        });

        // Volatile market weights - favor regime and liquidity
        regime_specific.insert(RegimeType::Volatile, SignalWeights {
            microstructure: 0.30,   // Lower microstructure in volatility
            ai_prediction: 0.25,    // Lower AI (harder to predict)
            regime_detection: 0.25, // Higher regime detection
            liquidity_analysis: 0.20, // Higher liquidity analysis
            last_updated: Utc::now(),
            optimization_reason: "Volatile regime default".to_string(),
            performance_score: 0.0,
            regime_specific: HashMap::new(),
        });

        // Normal market weights - balanced approach
        regime_specific.insert(RegimeType::Normal, SignalWeights {
            microstructure: 0.35,   // Balanced microstructure
            ai_prediction: 0.35,    // Balanced AI
            regime_detection: 0.20, // Moderate regime
            liquidity_analysis: 0.10, // Standard liquidity
            last_updated: Utc::now(),
            optimization_reason: "Normal regime default".to_string(),
            performance_score: 0.0,
            regime_specific: HashMap::new(),
        });

        // Crisis market weights - favor regime and liquidity
        regime_specific.insert(RegimeType::Crisis, SignalWeights {
            microstructure: 0.20,   // Lower microstructure (unreliable)
            ai_prediction: 0.20,    // Lower AI (crisis unpredictable)
            regime_detection: 0.40, // Much higher regime detection
            liquidity_analysis: 0.20, // Higher liquidity (risk management)
            last_updated: Utc::now(),
            optimization_reason: "Crisis regime default".to_string(),
            performance_score: 0.0,
            regime_specific: HashMap::new(),
        });

        Self {
            microstructure: 0.35,   // Optimized from 40% based on analysis
            ai_prediction: 0.35,    // Maintained at 35% (good performance)
            regime_detection: 0.20, // Increased from 15% (better regime awareness)
            liquidity_analysis: 0.10, // Maintained at 10%
            last_updated: Utc::now(),
            optimization_reason: "Initial optimized allocation".to_string(),
            performance_score: 0.0,
            regime_specific,
        }
    }
}

impl SignalWeights {
    /// Validate that weights sum to 1.0
    pub fn validate(&self) -> Result<()> {
        let total = self.microstructure + self.ai_prediction + self.regime_detection + self.liquidity_analysis;
        if (total - 1.0).abs() > 1e-6 {
            return Err(crate::utils::PantherSwapError::validation(
                format!("Signal weights sum to {}, not 1.0", total)
            ).into());
        }
        Ok(())
    }

    /// Normalize weights to sum to 1.0
    pub fn normalize(&mut self) {
        let total = self.microstructure + self.ai_prediction + self.regime_detection + self.liquidity_analysis;
        if total > 0.0 {
            self.microstructure /= total;
            self.ai_prediction /= total;
            self.regime_detection /= total;
            self.liquidity_analysis /= total;
        }
    }

    /// Get regime-specific weights if available, otherwise return default weights
    pub fn get_regime_weights(&self, regime: &RegimeType) -> &SignalWeights {
        self.regime_specific.get(regime).unwrap_or(self)
    }

    /// Update weights based on performance feedback
    pub fn update_from_performance(
        &mut self,
        microstructure_performance: f64,
        ai_performance: f64,
        regime_performance: f64,
        liquidity_performance: f64,
        learning_rate: f64,
    ) -> Result<()> {
        // Calculate performance-weighted adjustments
        let total_performance = microstructure_performance + ai_performance + regime_performance + liquidity_performance;

        if total_performance > 0.0 {
            // Adjust weights based on relative performance
            let microstructure_ratio = microstructure_performance / total_performance;
            let ai_ratio = ai_performance / total_performance;
            let regime_ratio = regime_performance / total_performance;
            let liquidity_ratio = liquidity_performance / total_performance;

            // Apply learning rate to gradual adjustment
            self.microstructure = self.microstructure * (1.0 - learning_rate) + microstructure_ratio * learning_rate;
            self.ai_prediction = self.ai_prediction * (1.0 - learning_rate) + ai_ratio * learning_rate;
            self.regime_detection = self.regime_detection * (1.0 - learning_rate) + regime_ratio * learning_rate;
            self.liquidity_analysis = self.liquidity_analysis * (1.0 - learning_rate) + liquidity_ratio * learning_rate;

            // Normalize to ensure sum = 1.0
            self.normalize();

            // Update metadata
            self.last_updated = Utc::now();
            self.optimization_reason = format!(
                "Performance-based update: μ={:.3}, AI={:.3}, R={:.3}, L={:.3}",
                microstructure_performance, ai_performance, regime_performance, liquidity_performance
            );
            self.performance_score = total_performance;
        }

        Ok(())
    }

    /// Apply Kelly Criterion optimization for signal weights
    pub fn optimize_kelly_weights(
        &mut self,
        returns: &[f64],
        signal_contributions: &[(f64, f64, f64, f64)], // (micro, ai, regime, liquidity) contributions
    ) -> Result<()> {
        if returns.len() != signal_contributions.len() || returns.is_empty() {
            return Err(crate::utils::PantherSwapError::validation(
                "Returns and signal contributions must have same non-zero length".to_string()
            ).into());
        }

        // Calculate Kelly fractions for each signal type
        let mut kelly_weights = [0.0; 4]; // [micro, ai, regime, liquidity]
        let mut overall_mean_return = 0.0;

        for i in 0..4 {
            let signal_returns: Vec<f64> = returns.iter().zip(signal_contributions.iter())
                .map(|(ret, contrib)| {
                    let signal_contrib = match i {
                        0 => contrib.0, // microstructure
                        1 => contrib.1, // ai
                        2 => contrib.2, // regime
                        _ => contrib.3, // liquidity
                    };
                    ret * signal_contrib
                })
                .collect();

            let mean_return = signal_returns.iter().sum::<f64>() / signal_returns.len() as f64;
            let variance = signal_returns.iter()
                .map(|r| (r - mean_return).powi(2))
                .sum::<f64>() / signal_returns.len() as f64;

            if variance > 1e-8 {
                kelly_weights[i] = (mean_return / variance).max(0.0).min(1.0);
            }

            overall_mean_return += mean_return;
        }

        // Normalize Kelly weights
        let total_kelly: f64 = kelly_weights.iter().sum();
        if total_kelly > 0.0 {
            self.microstructure = kelly_weights[0] / total_kelly;
            self.ai_prediction = kelly_weights[1] / total_kelly;
            self.regime_detection = kelly_weights[2] / total_kelly;
            self.liquidity_analysis = kelly_weights[3] / total_kelly;

            self.last_updated = Utc::now();
            self.optimization_reason = "Kelly Criterion optimization".to_string();
            self.performance_score = overall_mean_return / 4.0; // Average across signals
        }

        Ok(())
    }

    /// Apply constraints to ensure weights stay within reasonable bounds
    pub fn apply_constraints(&mut self) {
        // Minimum and maximum weight constraints
        let min_weight = 0.05; // 5% minimum
        let max_weight = 0.70; // 70% maximum

        self.microstructure = self.microstructure.max(min_weight).min(max_weight);
        self.ai_prediction = self.ai_prediction.max(min_weight).min(max_weight);
        self.regime_detection = self.regime_detection.max(min_weight).min(max_weight);
        self.liquidity_analysis = self.liquidity_analysis.max(min_weight).min(max_weight);

        // Normalize after applying constraints
        self.normalize();
    }
}

// Placeholder structures for AI and Microstructure components
// These will be replaced with actual implementations in later phases

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AISignal {
    pub instrument_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub price_predictions: Vec<PredictionResult>,
    pub regime_signal: Option<RegimeSignal>,
    pub rl_recommendation: Option<RLRecommendation>,
    pub confidence_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RLRecommendation {
    pub action: String,
    pub confidence: f64,
    pub expected_reward: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResult {
    pub horizon: Duration,
    pub predicted_price: f64,
    pub confidence_score: f64,
    pub prediction_interval: (f64, f64), // (lower, upper) bounds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeSignal {
    pub current_regime: RegimeType,
    pub regime: RegimeType, // Alias for current_regime for backward compatibility
    pub transition_probability: f64,
    pub confidence: f64,
    pub regime_strength: f64, // Alias for confidence for backward compatibility
    pub expected_duration_minutes: u32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct MicrostructureAnalysis {
    pub instrument_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub current_price: f64,
    pub liquidity_metrics: Option<LiquidityMetrics>,
    pub orderbook_imbalance: f64,
    pub bid_ask_spread: f64,
    pub market_depth: f64,
}

#[derive(Debug, Clone)]
pub struct LiquidityMetrics {
    pub imbalance_ratio: f64,
    pub depth_ratio: f64,
    pub spread_stability: f64,
}

use crate::utils::Result;

// Dynamic Signal Weight Optimizer
#[derive(Debug, Clone)]
pub struct DynamicSignalOptimizer {
    pub performance_history: Vec<SignalPerformanceRecord>,
    pub optimization_frequency: Duration,
    pub last_optimization: DateTime<Utc>,
    pub learning_rate: f64,
    pub min_samples_for_optimization: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalPerformanceRecord {
    pub timestamp: DateTime<Utc>,
    pub instrument_id: Uuid,
    pub signal_weights: SignalWeights,
    pub microstructure_contribution: f64,
    pub ai_contribution: f64,
    pub regime_contribution: f64,
    pub liquidity_contribution: f64,
    pub total_return: f64,
    pub regime_type: RegimeType,
}

impl DynamicSignalOptimizer {
    pub fn new() -> Self {
        Self {
            performance_history: Vec::new(),
            optimization_frequency: Duration::from_secs(3600), // 1 hour
            last_optimization: Utc::now(),
            learning_rate: 0.1,
            min_samples_for_optimization: 50,
        }
    }

    /// Record performance for signal weight optimization
    pub fn record_performance(
        &mut self,
        instrument_id: Uuid,
        signal_weights: &SignalWeights,
        signal_contributions: (f64, f64, f64, f64), // (micro, ai, regime, liquidity)
        total_return: f64,
        regime_type: RegimeType,
    ) {
        let record = SignalPerformanceRecord {
            timestamp: Utc::now(),
            instrument_id,
            signal_weights: signal_weights.clone(),
            microstructure_contribution: signal_contributions.0,
            ai_contribution: signal_contributions.1,
            regime_contribution: signal_contributions.2,
            liquidity_contribution: signal_contributions.3,
            total_return,
            regime_type,
        };

        self.performance_history.push(record);

        // Keep only recent history (last 1000 records)
        if self.performance_history.len() > 1000 {
            self.performance_history.drain(0..100);
        }
    }

    /// Optimize signal weights based on performance history
    pub fn optimize_weights(&mut self, current_weights: &mut SignalWeights) -> Result<bool> {
        // Check if optimization is due
        let now = Utc::now();
        if now.signed_duration_since(self.last_optimization) < chrono::Duration::from_std(self.optimization_frequency).unwrap() {
            return Ok(false);
        }

        // Check if we have enough samples
        if self.performance_history.len() < self.min_samples_for_optimization {
            return Ok(false);
        }

        // Get recent performance data
        let recent_cutoff = now - chrono::Duration::hours(24);
        let recent_records: Vec<&SignalPerformanceRecord> = self.performance_history
            .iter()
            .filter(|r| r.timestamp > recent_cutoff)
            .collect();

        if recent_records.len() < 20 {
            return Ok(false);
        }

        // Calculate performance metrics for each signal type
        let mut micro_performance = 0.0;
        let mut ai_performance = 0.0;
        let mut regime_performance = 0.0;
        let mut liquidity_performance = 0.0;

        for record in &recent_records {
            // Weight performance by signal contribution
            micro_performance += record.total_return * record.microstructure_contribution;
            ai_performance += record.total_return * record.ai_contribution;
            regime_performance += record.total_return * record.regime_contribution;
            liquidity_performance += record.total_return * record.liquidity_contribution;
        }

        // Normalize by number of records
        let num_records = recent_records.len() as f64;
        micro_performance /= num_records;
        ai_performance /= num_records;
        regime_performance /= num_records;
        liquidity_performance /= num_records;

        // Apply performance-based weight update
        current_weights.update_from_performance(
            micro_performance,
            ai_performance,
            regime_performance,
            liquidity_performance,
            self.learning_rate,
        )?;

        // Apply Kelly Criterion optimization if we have enough data
        if recent_records.len() >= 100 {
            let returns: Vec<f64> = recent_records.iter().map(|r| r.total_return).collect();
            let contributions: Vec<(f64, f64, f64, f64)> = recent_records.iter()
                .map(|r| (r.microstructure_contribution, r.ai_contribution, r.regime_contribution, r.liquidity_contribution))
                .collect();

            current_weights.optimize_kelly_weights(&returns, &contributions)?;
        }

        // Apply constraints
        current_weights.apply_constraints();

        self.last_optimization = now;
        Ok(true)
    }

    /// Get regime-specific optimization insights
    pub fn get_regime_insights(&self, regime: RegimeType) -> Option<SignalWeights> {
        let regime_records: Vec<&SignalPerformanceRecord> = self.performance_history
            .iter()
            .filter(|r| r.regime_type == regime)
            .collect();

        if regime_records.len() < 10 {
            return None;
        }

        // Calculate average performance for this regime
        let mut total_micro = 0.0;
        let mut total_ai = 0.0;
        let mut total_regime = 0.0;
        let mut total_liquidity = 0.0;
        let mut total_return = 0.0;

        for record in &regime_records {
            total_micro += record.microstructure_contribution * record.total_return;
            total_ai += record.ai_contribution * record.total_return;
            total_regime += record.regime_contribution * record.total_return;
            total_liquidity += record.liquidity_contribution * record.total_return;
            total_return += record.total_return;
        }

        let num_records = regime_records.len() as f64;
        if total_return > 0.0 {
            let mut regime_weights = SignalWeights::default();
            regime_weights.microstructure = (total_micro / total_return).max(0.05).min(0.70);
            regime_weights.ai_prediction = (total_ai / total_return).max(0.05).min(0.70);
            regime_weights.regime_detection = (total_regime / total_return).max(0.05).min(0.70);
            regime_weights.liquidity_analysis = (total_liquidity / total_return).max(0.05).min(0.70);

            regime_weights.normalize();
            regime_weights.last_updated = Utc::now();
            regime_weights.optimization_reason = format!("Regime-specific optimization for {:?}", regime);
            regime_weights.performance_score = total_return / num_records;

            Some(regime_weights)
        } else {
            None
        }
    }
}

// Signal Generator Implementation with Dynamic Optimization
#[derive(Clone)]
pub struct SignalGenerator {
    confidence_threshold: f64,
    signal_weights: SignalWeights,
    optimizer: DynamicSignalOptimizer,
}

impl SignalGenerator {
    pub fn new(confidence_threshold: f64) -> Self {
        Self {
            confidence_threshold,
            signal_weights: SignalWeights::default(),
            optimizer: DynamicSignalOptimizer::new(),
        }
    }

    /// Update signal weights based on performance feedback
    pub fn update_weights_from_performance(
        &mut self,
        instrument_id: Uuid,
        signal_contributions: (f64, f64, f64, f64),
        total_return: f64,
        regime_type: RegimeType,
    ) -> Result<()> {
        // Record performance for optimization
        self.optimizer.record_performance(
            instrument_id,
            &self.signal_weights,
            signal_contributions,
            total_return,
            regime_type,
        );

        // Attempt to optimize weights
        if self.optimizer.optimize_weights(&mut self.signal_weights)? {
            tracing::info!(
                "Signal weights optimized: micro={:.3}, ai={:.3}, regime={:.3}, liquidity={:.3}",
                self.signal_weights.microstructure,
                self.signal_weights.ai_prediction,
                self.signal_weights.regime_detection,
                self.signal_weights.liquidity_analysis
            );
        }

        Ok(())
    }

    /// Get current signal weights (regime-aware)
    pub fn get_current_weights(&self, regime: Option<&RegimeType>) -> &SignalWeights {
        if let Some(regime) = regime {
            self.signal_weights.get_regime_weights(regime)
        } else {
            &self.signal_weights
        }
    }

    /// Force weight optimization (for testing or manual triggers)
    pub fn force_optimize_weights(&mut self) -> Result<bool> {
        self.optimizer.last_optimization = Utc::now() - chrono::Duration::hours(2);
        self.optimizer.optimize_weights(&mut self.signal_weights)
    }

    pub async fn generate_signals(
        &self,
        microstructure_analysis: &HashMap<Uuid, MicrostructureAnalysis>,
        ai_signals: &[AISignal],
    ) -> Result<Vec<TradingSignal>> {
        let mut signals = Vec::new();

        // Group AI signals by instrument
        let mut ai_by_instrument: HashMap<Uuid, &AISignal> = HashMap::new();
        for ai_signal in ai_signals {
            ai_by_instrument.insert(ai_signal.instrument_id, ai_signal);
        }

        // Generate signals for each instrument with both microstructure and AI data
        for (&instrument_id, microstructure) in microstructure_analysis {
            if let Some(&ai_signal) = ai_by_instrument.get(&instrument_id) {
                let generated_signals = self.generate_instrument_signals(
                    instrument_id,
                    microstructure,
                    ai_signal,
                ).await?;

                signals.extend(generated_signals);
            }
        }

        Ok(signals)
    }

    async fn generate_instrument_signals(
        &self,
        instrument_id: Uuid,
        microstructure: &MicrostructureAnalysis,
        ai_signal: &AISignal,
    ) -> Result<Vec<TradingSignal>> {
        let mut signals = Vec::new();

        // 1. Predictive Market Making Signal
        if let Some(signal) = self.generate_market_making_signal(instrument_id, microstructure, ai_signal).await? {
            signals.push(signal);
        }

        // 2. Microstructure Momentum Signal
        if let Some(signal) = self.generate_momentum_signal(instrument_id, microstructure, ai_signal).await? {
            signals.push(signal);
        }

        // 3. Regime Arbitrage Signal
        if let Some(signal) = self.generate_regime_signal(instrument_id, microstructure, ai_signal).await? {
            signals.push(signal);
        }

        // 4. Liquidity Harvesting Signal
        if let Some(signal) = self.generate_liquidity_signal(instrument_id, microstructure, ai_signal).await? {
            signals.push(signal);
        }

        // Filter by confidence threshold
        signals.retain(|s| s.confidence_score >= self.confidence_threshold);

        Ok(signals)
    }

    async fn generate_market_making_signal(
        &self,
        instrument_id: Uuid,
        microstructure: &MicrostructureAnalysis,
        ai_signal: &AISignal,
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
                RegimeType::Sideways => 1.0,
                RegimeType::HighVolatility => 0.3,
            })
            .unwrap_or(0.5);

        let microstructure_score = liquidity_imbalance.abs().min(1.0);
        let ai_score = (volatility_prediction * 10.0).min(1.0); // Scale volatility to 0-1
        let regime_score = regime_stability;

        let overall_score =
            self.signal_weights.microstructure * microstructure_score +
            self.signal_weights.ai_prediction * ai_score +
            self.signal_weights.regime_detection * regime_score;

        if overall_score < self.confidence_threshold {
            return Ok(None);
        }

        let signal_type = if liquidity_imbalance > 0.1 {
            SignalType::Sell // Provide ask liquidity
        } else if liquidity_imbalance < -0.1 {
            SignalType::Buy  // Provide bid liquidity
        } else {
            SignalType::Hold
        };

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
            time_horizon: Some(Duration::from_secs(900)), // 15 minutes
            expected_return: Some(volatility_prediction * 0.5), // Half of predicted volatility
            max_risk: Some(0.02), // 2% max risk
            supporting_evidence: SignalEvidence {
                microstructure_score,
                ai_prediction_score: ai_score,
                regime_score,
                liquidity_score: liquidity_imbalance.abs(),
                risk_reward_ratio: 2.0, // Conservative market making
            },
        }))
    }

    async fn generate_momentum_signal(
        &self,
        instrument_id: Uuid,
        microstructure: &MicrostructureAnalysis,
        ai_signal: &AISignal,
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

        let regime_trending = ai_signal.regime_signal
            .as_ref()
            .map(|rs| match rs.current_regime {
                RegimeType::Trending => 1.0,
                RegimeType::Volatile => 0.7,
                RegimeType::Normal => 0.3,
                RegimeType::Crisis => 0.1,
                RegimeType::Bullish => 0.9,
                RegimeType::Bearish => 0.9,
                RegimeType::Sideways => 0.2,
                RegimeType::HighVolatility => 0.8,
            })
            .unwrap_or(0.5);

        let ai_confidence = (pred_1min.confidence_score + pred_5min.confidence_score) / 2.0;

        let overall_score =
            self.signal_weights.ai_prediction * ai_confidence +
            self.signal_weights.regime_detection * regime_trending +
            0.3 * direction_consistency +
            0.2 * momentum_strength.min(1.0);

        if overall_score < self.confidence_threshold || momentum_strength < 0.005 {
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
            stop_loss: Some(current_price * (1.0 - 0.01 * momentum_1min.signum())), // 1% stop
            take_profit: Some(pred_5min.predicted_price),
            time_horizon: Some(Duration::from_secs(300)), // 5 minutes
            expected_return: Some(momentum_5min),
            max_risk: Some(0.01), // 1% max risk
            supporting_evidence: SignalEvidence {
                microstructure_score: 0.5,
                ai_prediction_score: ai_confidence,
                regime_score: regime_trending,
                liquidity_score: 0.5,
                risk_reward_ratio: momentum_5min.abs() / 0.01, // Return/Risk ratio
            },
        }))
    }

    async fn generate_regime_signal(
        &self,
        instrument_id: Uuid,
        microstructure: &MicrostructureAnalysis,
        ai_signal: &AISignal,
    ) -> Result<Option<TradingSignal>> {
        // Regime arbitrage: exploit regime transitions

        let regime_signal = match &ai_signal.regime_signal {
            Some(rs) => rs,
            None => return Ok(None),
        };

        // Only trade on regime transitions with high confidence
        if regime_signal.transition_probability < 0.3 || regime_signal.confidence < 0.8 {
            return Ok(None);
        }

        // Strategy varies by regime transition
        let (signal_type, expected_return, time_horizon) = match regime_signal.current_regime {
            RegimeType::Crisis => {
                // Crisis regime: typically sell risk assets, buy safe havens
                (SignalType::Sell, -0.02, Duration::from_secs(3600)) // 1 hour
            },
            RegimeType::Volatile | RegimeType::HighVolatility => {
                // Volatile regime: reduce positions, increase cash
                (SignalType::Sell, -0.01, Duration::from_secs(1800)) // 30 minutes
            },
            RegimeType::Trending | RegimeType::Bullish | RegimeType::Bearish => {
                // Trending regime: follow the trend
                let price_prediction = ai_signal.price_predictions
                    .iter()
                    .find(|p| p.horizon == Duration::from_secs(3600)) // 1 hour
                    .map(|p| (p.predicted_price - microstructure.current_price) / microstructure.current_price)
                    .unwrap_or(0.0);

                let signal = if price_prediction > 0.0 || matches!(regime_signal.current_regime, RegimeType::Bullish) {
                    SignalType::Buy
                } else {
                    SignalType::Sell
                };
                (signal, price_prediction, Duration::from_secs(7200)) // 2 hours
            },
            RegimeType::Normal | RegimeType::Sideways => {
                // Normal/Sideways regime: mean reversion strategies
                (SignalType::Hold, 0.0, Duration::from_secs(3600))
            },
        };

        if matches!(signal_type, SignalType::Hold) {
            return Ok(None);
        }

        Ok(Some(TradingSignal {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            instrument_id,
            strategy_type: StrategyType::RegimeArbitrage,
            signal_type,
            signal_strength: regime_signal.transition_probability,
            confidence_score: regime_signal.confidence,
            urgency_score: Some(regime_signal.transition_probability), // Higher urgency for stronger transitions
            entry_price: Some(microstructure.current_price),
            stop_loss: Some(microstructure.current_price * 0.98), // 2% stop loss
            take_profit: None, // Let regime play out
            time_horizon: Some(time_horizon),
            expected_return: Some(expected_return),
            max_risk: Some(0.03), // 3% max risk for regime plays
            supporting_evidence: SignalEvidence {
                microstructure_score: 0.3,
                ai_prediction_score: 0.3,
                regime_score: regime_signal.confidence,
                liquidity_score: 0.3,
                risk_reward_ratio: expected_return.abs() / 0.03,
            },
        }))
    }

    async fn generate_liquidity_signal(
        &self,
        instrument_id: Uuid,
        microstructure: &MicrostructureAnalysis,
        ai_signal: &AISignal,
    ) -> Result<Option<TradingSignal>> {
        // Liquidity harvesting: exploit temporary liquidity imbalances

        let liquidity_metrics = match &microstructure.liquidity_metrics {
            Some(lm) => lm,
            None => return Ok(None),
        };

        // Look for significant liquidity imbalances
        if liquidity_metrics.imbalance_ratio.abs() < 0.2 {
            return Ok(None);
        }

        // Predict liquidity return (mean reversion)
        let liquidity_prediction = ai_signal.price_predictions
            .iter()
            .find(|p| p.horizon == Duration::from_secs(300)) // 5 minutes
            .map(|p| p.confidence_score)
            .unwrap_or(0.5);

        let signal_type = if liquidity_metrics.imbalance_ratio > 0.2 {
            SignalType::Sell // Excess bid liquidity, sell into it
        } else {
            SignalType::Buy  // Excess ask liquidity, buy from it
        };

        let confidence = liquidity_metrics.imbalance_ratio.abs() * liquidity_prediction;

        if confidence < self.confidence_threshold {
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
            urgency_score: Some(0.9), // Liquidity imbalances are temporary
            entry_price: Some(microstructure.current_price),
            stop_loss: Some(microstructure.current_price * 0.995), // Tight 0.5% stop
            take_profit: Some(microstructure.current_price * (1.0 + 0.002 * liquidity_metrics.imbalance_ratio.signum())), // 0.2% target
            time_horizon: Some(Duration::from_secs(300)), // 5 minutes
            expected_return: Some(0.002), // 0.2% expected return
            max_risk: Some(0.005), // 0.5% max risk
            supporting_evidence: SignalEvidence {
                microstructure_score: liquidity_metrics.imbalance_ratio.abs(),
                ai_prediction_score: liquidity_prediction,
                regime_score: 0.5,
                liquidity_score: 1.0,
                risk_reward_ratio: 0.002 / 0.005, // 0.4 risk-reward ratio
            },
        }))
    }
}
