use crate::trading::strategies::StrategyPerformance;
use crate::trading::signals::StrategyType;
use crate::database::types::RegimeType;
use crate::database::Database;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use tracing::{info, warn};

/// Strategy weight allocation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyWeightConfig {
    pub min_weight: f64,
    pub max_weight: f64,
    pub target_weight: f64,
    pub weight_adjustment_speed: f64,
    pub performance_lookback_days: u32,
    pub rebalance_frequency_hours: u32,
    pub risk_budget_allocation: f64,
}

impl Default for StrategyWeightConfig {
    fn default() -> Self {
        Self {
            min_weight: 0.05,  // 5% minimum allocation
            max_weight: 0.60,  // 60% maximum allocation
            target_weight: 0.25, // 25% target allocation
            weight_adjustment_speed: 0.1, // 10% adjustment per rebalance
            performance_lookback_days: 30,
            rebalance_frequency_hours: 6,
            risk_budget_allocation: 0.25, // 25% of total risk budget
        }
    }
}

/// Strategy allocation weights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyWeights {
    pub predictive_market_making: f64,
    pub microstructure_momentum: f64,
    pub regime_arbitrage: f64,
    pub liquidity_harvesting: f64,
    pub last_updated: DateTime<Utc>,
    pub rebalance_reason: String,
}

impl Default for StrategyWeights {
    fn default() -> Self {
        Self {
            predictive_market_making: 0.30,  // 30% - stable income
            microstructure_momentum: 0.25,   // 25% - high frequency
            regime_arbitrage: 0.25,          // 25% - medium term
            liquidity_harvesting: 0.20,     // 20% - opportunistic
            last_updated: Utc::now(),
            rebalance_reason: "Initial allocation".to_string(),
        }
    }
}

impl StrategyWeights {
    /// Validate that weights sum to 1.0
    pub fn validate(&self) -> Result<()> {
        let total = self.predictive_market_making + 
                   self.microstructure_momentum + 
                   self.regime_arbitrage + 
                   self.liquidity_harvesting;
        
        if (total - 1.0).abs() > 1e-6 {
            return Err(anyhow::anyhow!("Strategy weights sum to {}, not 1.0", total));
        }
        Ok(())
    }

    /// Normalize weights to sum to 1.0
    pub fn normalize(&mut self) {
        let total = self.predictive_market_making + 
                   self.microstructure_momentum + 
                   self.regime_arbitrage + 
                   self.liquidity_harvesting;
        
        if total > 0.0 {
            self.predictive_market_making /= total;
            self.microstructure_momentum /= total;
            self.regime_arbitrage /= total;
            self.liquidity_harvesting /= total;
        }
    }

    /// Get weight for specific strategy
    pub fn get_weight(&self, strategy_type: StrategyType) -> f64 {
        match strategy_type {
            StrategyType::PredictiveMarketMaking => self.predictive_market_making,
            StrategyType::MicrostructureMomentum => self.microstructure_momentum,
            StrategyType::RegimeArbitrage => self.regime_arbitrage,
            StrategyType::LiquidityHarvesting => self.liquidity_harvesting,
        }
    }

    /// Set weight for specific strategy
    pub fn set_weight(&mut self, strategy_type: StrategyType, weight: f64) {
        match strategy_type {
            StrategyType::PredictiveMarketMaking => self.predictive_market_making = weight,
            StrategyType::MicrostructureMomentum => self.microstructure_momentum = weight,
            StrategyType::RegimeArbitrage => self.regime_arbitrage = weight,
            StrategyType::LiquidityHarvesting => self.liquidity_harvesting = weight,
        }
    }
}

/// Advanced performance analytics for strategy optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyAnalytics {
    pub strategy_type: StrategyType,
    pub performance: StrategyPerformance,
    pub risk_adjusted_return: f64,
    pub volatility_adjusted_sharpe: f64,
    pub regime_performance: HashMap<RegimeType, f64>,
    pub correlation_matrix: HashMap<StrategyType, f64>,
    pub contribution_to_portfolio_risk: f64,
    pub marginal_var: f64,
    pub component_var: f64,
    pub diversification_ratio: f64,

    // Enhanced performance metrics
    pub rolling_sharpe_7d: f64,
    pub rolling_sharpe_30d: f64,
    pub rolling_sortino_7d: f64,
    pub rolling_sortino_30d: f64,
    pub rolling_calmar_7d: f64,
    pub rolling_calmar_30d: f64,
    pub rolling_max_drawdown_7d: f64,
    pub rolling_max_drawdown_30d: f64,
    pub rolling_volatility_7d: f64,
    pub rolling_volatility_30d: f64,
    pub rolling_skewness_30d: f64,
    pub rolling_kurtosis_30d: f64,
    pub rolling_var_95_7d: f64,
    pub rolling_var_99_7d: f64,
    pub rolling_expected_shortfall_7d: f64,
    pub rolling_expected_shortfall_30d: f64,

    // Performance attribution metrics
    pub alpha_vs_market: f64,
    pub beta_vs_market: f64,
    pub tracking_error_vs_market: f64,
    pub information_ratio_vs_market: f64,
    pub upside_capture_ratio: f64,
    pub downside_capture_ratio: f64,
    pub hit_ratio: f64,
    pub profit_factor: f64,
    pub recovery_factor: f64,
    pub sterling_ratio: f64,
    pub burke_ratio: f64,
    pub pain_index: f64,
    pub ulcer_index: f64,

    // Regime-specific performance
    pub regime_sharpe_ratios: HashMap<RegimeType, f64>,
    pub regime_max_drawdowns: HashMap<RegimeType, f64>,
    pub regime_hit_ratios: HashMap<RegimeType, f64>,
    pub regime_profit_factors: HashMap<RegimeType, f64>,

    // Risk decomposition
    pub systematic_risk: f64,
    pub idiosyncratic_risk: f64,
    pub tail_risk_contribution: f64,
    pub stress_test_performance: HashMap<String, f64>,

    pub last_updated: DateTime<Utc>,
}

/// Strategy weight optimization engine
pub struct StrategyWeightOptimizer {
    database: Arc<Database>,
    current_weights: Arc<RwLock<StrategyWeights>>,
    strategy_configs: HashMap<StrategyType, StrategyWeightConfig>,
    performance_history: Arc<RwLock<HashMap<StrategyType, Vec<StrategyAnalytics>>>>,
    optimization_config: OptimizationConfig,
    ml_optimizer: Arc<RwLock<MLWeightOptimizer>>,
    market_feature_extractor: MarketFeatureExtractor,
}

/// Optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    pub target_sharpe_ratio: f64,
    pub max_drawdown_threshold: f64,
    pub min_diversification_ratio: f64,
    pub risk_free_rate: f64,
    pub confidence_level: f64,
    pub optimization_frequency_hours: u32,
    pub enable_regime_based_weights: bool,
    pub enable_correlation_adjustment: bool,
    pub enable_risk_parity: bool,
    pub kelly_fraction_limit: f64,

    // Enhanced ML-based optimization parameters
    pub enable_ml_optimization: bool,
    pub ml_learning_rate: f64,
    pub ml_momentum: f64,
    pub ml_regularization: f64,
    pub enable_adaptive_learning: bool,
    pub performance_decay_factor: f64,
    pub min_samples_for_ml: usize,
    pub enable_ensemble_optimization: bool,
    pub volatility_target: f64,
    pub max_weight_change_per_period: f64,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            target_sharpe_ratio: 2.0,
            max_drawdown_threshold: 0.15, // 15% max drawdown
            min_diversification_ratio: 1.2,
            risk_free_rate: 0.02, // 2% annual risk-free rate
            confidence_level: 0.95,
            optimization_frequency_hours: 6,
            enable_regime_based_weights: true,
            enable_correlation_adjustment: true,
            enable_risk_parity: false,
            kelly_fraction_limit: 0.25, // Max 25% Kelly allocation

            // Enhanced ML-based optimization defaults
            enable_ml_optimization: true,
            ml_learning_rate: 0.01,
            ml_momentum: 0.9,
            ml_regularization: 0.001,
            enable_adaptive_learning: true,
            performance_decay_factor: 0.95,
            min_samples_for_ml: 50,
            enable_ensemble_optimization: true,
            volatility_target: 0.15, // 15% target volatility
            max_weight_change_per_period: 0.1, // Max 10% weight change per optimization
        }
    }
}

/// Helper struct for rolling performance metrics
#[derive(Debug, Clone)]
struct RollingMetrics {
    pub sharpe_7d: f64,
    pub sharpe_30d: f64,
    pub sortino_7d: f64,
    pub sortino_30d: f64,
    pub calmar_7d: f64,
    pub calmar_30d: f64,
    pub max_drawdown_7d: f64,
    pub max_drawdown_30d: f64,
    pub volatility_7d: f64,
    pub volatility_30d: f64,
    pub skewness_30d: f64,
    pub kurtosis_30d: f64,
    pub var_95_7d: f64,
    pub var_99_7d: f64,
    pub expected_shortfall_7d: f64,
    pub expected_shortfall_30d: f64,
}

impl Default for RollingMetrics {
    fn default() -> Self {
        Self {
            sharpe_7d: 0.0,
            sharpe_30d: 0.0,
            sortino_7d: 0.0,
            sortino_30d: 0.0,
            calmar_7d: 0.0,
            calmar_30d: 0.0,
            max_drawdown_7d: 0.0,
            max_drawdown_30d: 0.0,
            volatility_7d: 0.0,
            volatility_30d: 0.0,
            skewness_30d: 0.0,
            kurtosis_30d: 0.0,
            var_95_7d: 0.0,
            var_99_7d: 0.0,
            expected_shortfall_7d: 0.0,
            expected_shortfall_30d: 0.0,
        }
    }
}

/// Helper struct for performance attribution metrics
#[derive(Debug, Clone)]
struct AttributionMetrics {
    pub alpha: f64,
    pub beta: f64,
    pub tracking_error: f64,
    pub information_ratio: f64,
    pub upside_capture: f64,
    pub downside_capture: f64,
    pub hit_ratio: f64,
    pub profit_factor: f64,
    pub recovery_factor: f64,
    pub sterling_ratio: f64,
    pub burke_ratio: f64,
    pub pain_index: f64,
    pub ulcer_index: f64,
}

/// Helper struct for regime-specific metrics
#[derive(Debug, Clone)]
struct RegimeMetrics {
    pub sharpe_ratios: HashMap<RegimeType, f64>,
    pub max_drawdowns: HashMap<RegimeType, f64>,
    pub hit_ratios: HashMap<RegimeType, f64>,
    pub profit_factors: HashMap<RegimeType, f64>,
}

/// Helper struct for risk decomposition
#[derive(Debug, Clone)]
struct RiskDecomposition {
    pub systematic: f64,
    pub idiosyncratic: f64,
    pub tail_risk: f64,
    pub stress_tests: HashMap<String, f64>,
}

/// Machine Learning optimizer for strategy weights
#[derive(Debug, Clone)]
pub struct MLWeightOptimizer {
    pub learning_rate: f64,
    pub momentum: f64,
    pub regularization: f64,
    pub weight_gradients: HashMap<StrategyType, f64>,
    pub momentum_terms: HashMap<StrategyType, f64>,
    pub performance_history: Vec<f64>,
    pub weight_history: Vec<StrategyWeights>,
    pub feature_matrix: Vec<Vec<f64>>,
    pub target_returns: Vec<f64>,
    pub adaptive_lr_enabled: bool,
    pub current_lr: f64,
}

impl MLWeightOptimizer {
    pub fn new(config: &OptimizationConfig) -> Self {
        let mut weight_gradients = HashMap::new();
        let mut momentum_terms = HashMap::new();

        // Initialize gradients and momentum for each strategy
        for strategy in [
            StrategyType::PredictiveMarketMaking,
            StrategyType::MicrostructureMomentum,
            StrategyType::RegimeArbitrage,
            StrategyType::LiquidityHarvesting,
        ] {
            weight_gradients.insert(strategy, 0.0);
            momentum_terms.insert(strategy, 0.0);
        }

        Self {
            learning_rate: config.ml_learning_rate,
            momentum: config.ml_momentum,
            regularization: config.ml_regularization,
            weight_gradients,
            momentum_terms,
            performance_history: Vec::new(),
            weight_history: Vec::new(),
            feature_matrix: Vec::new(),
            target_returns: Vec::new(),
            adaptive_lr_enabled: config.enable_adaptive_learning,
            current_lr: config.ml_learning_rate,
        }
    }

    /// Update weights using gradient descent with momentum
    pub fn optimize_weights(
        &mut self,
        current_weights: &StrategyWeights,
        performance_metrics: &HashMap<StrategyType, f64>,
        market_features: &[f64],
        target_return: f64,
    ) -> Result<StrategyWeights> {
        // Calculate gradients based on performance feedback
        self.calculate_gradients(current_weights, performance_metrics, target_return)?;

        // Apply momentum and update weights
        let mut new_weights = current_weights.clone();

        // Update each strategy weight using gradient descent with momentum
        for strategy in [
            StrategyType::PredictiveMarketMaking,
            StrategyType::MicrostructureMomentum,
            StrategyType::RegimeArbitrage,
            StrategyType::LiquidityHarvesting,
        ] {
            let gradient = self.weight_gradients.get(&strategy).unwrap_or(&0.0);
            let momentum_term = self.momentum_terms.get_mut(&strategy).unwrap();

            // Update momentum term
            *momentum_term = self.momentum * (*momentum_term) + self.current_lr * gradient;

            // Update weight
            let current_weight = new_weights.get_weight(strategy);
            let new_weight = current_weight + *momentum_term;

            // Apply regularization (L2)
            let regularized_weight = new_weight - self.regularization * current_weight;

            new_weights.set_weight(strategy, regularized_weight);
        }

        // Normalize weights and apply constraints
        new_weights.normalize();
        self.apply_weight_constraints(&mut new_weights)?;

        // Update history
        self.weight_history.push(new_weights.clone());
        self.feature_matrix.push(market_features.to_vec());
        self.target_returns.push(target_return);

        // Adaptive learning rate adjustment
        if self.adaptive_lr_enabled {
            self.adjust_learning_rate()?;
        }

        Ok(new_weights)
    }

    /// Calculate gradients based on performance feedback
    fn calculate_gradients(
        &mut self,
        current_weights: &StrategyWeights,
        performance_metrics: &HashMap<StrategyType, f64>,
        target_return: f64,
    ) -> Result<()> {
        // Calculate portfolio return with current weights
        let portfolio_return = self.calculate_weighted_return(current_weights, performance_metrics);

        // Calculate loss (negative of excess return over target)
        let loss = -(portfolio_return - target_return);

        // Calculate gradients for each strategy weight
        for strategy in [
            StrategyType::PredictiveMarketMaking,
            StrategyType::MicrostructureMomentum,
            StrategyType::RegimeArbitrage,
            StrategyType::LiquidityHarvesting,
        ] {
            let strategy_return = performance_metrics.get(&strategy).unwrap_or(&0.0);

            // Gradient of loss with respect to weight
            let gradient = -strategy_return + target_return;

            self.weight_gradients.insert(strategy, gradient);
        }

        Ok(())
    }

    /// Calculate weighted portfolio return
    fn calculate_weighted_return(
        &self,
        weights: &StrategyWeights,
        performance_metrics: &HashMap<StrategyType, f64>,
    ) -> f64 {
        let mut weighted_return = 0.0;

        for strategy in [
            StrategyType::PredictiveMarketMaking,
            StrategyType::MicrostructureMomentum,
            StrategyType::RegimeArbitrage,
            StrategyType::LiquidityHarvesting,
        ] {
            let weight = weights.get_weight(strategy);
            let strategy_return = performance_metrics.get(&strategy).unwrap_or(&0.0);
            weighted_return += weight * strategy_return;
        }

        weighted_return
    }

    /// Apply weight constraints
    fn apply_weight_constraints(&self, weights: &mut StrategyWeights) -> Result<()> {
        // Apply minimum and maximum weight constraints
        let min_weight = 0.05; // 5% minimum
        let max_weight = 0.70; // 70% maximum

        weights.predictive_market_making = weights.predictive_market_making.max(min_weight).min(max_weight);
        weights.microstructure_momentum = weights.microstructure_momentum.max(min_weight).min(max_weight);
        weights.regime_arbitrage = weights.regime_arbitrage.max(min_weight).min(max_weight);
        weights.liquidity_harvesting = weights.liquidity_harvesting.max(min_weight).min(max_weight);

        Ok(())
    }

    /// Adjust learning rate based on performance
    fn adjust_learning_rate(&mut self) -> Result<()> {
        if self.performance_history.len() < 2 {
            return Ok(());
        }

        let recent_performance = &self.performance_history[self.performance_history.len().saturating_sub(5)..];

        // Calculate performance trend
        let trend = if recent_performance.len() >= 2 {
            let recent_avg = recent_performance.iter().sum::<f64>() / recent_performance.len() as f64;
            let older_avg = if self.performance_history.len() >= 10 {
                let older_slice = &self.performance_history[self.performance_history.len().saturating_sub(10)..self.performance_history.len().saturating_sub(5)];
                older_slice.iter().sum::<f64>() / older_slice.len() as f64
            } else {
                recent_avg
            };
            recent_avg - older_avg
        } else {
            0.0
        };

        // Adjust learning rate based on trend
        if trend > 0.0 {
            // Performance improving - slightly increase learning rate
            self.current_lr = (self.current_lr * 1.05).min(self.learning_rate * 2.0);
        } else if trend < -0.01 {
            // Performance declining - decrease learning rate
            self.current_lr = (self.current_lr * 0.95).max(self.learning_rate * 0.1);
        }

        Ok(())
    }

    /// Update performance history
    pub fn update_performance(&mut self, performance: f64) {
        self.performance_history.push(performance);

        // Keep only recent history (last 100 samples)
        if self.performance_history.len() > 100 {
            self.performance_history.drain(0..self.performance_history.len() - 100);
        }
    }
}

/// Market feature extractor for ML optimization
#[derive(Debug, Clone)]
pub struct MarketFeatureExtractor {
    pub feature_window: usize,
    pub volatility_lookback: usize,
    pub momentum_lookback: usize,
}

impl Default for MarketFeatureExtractor {
    fn default() -> Self {
        Self {
            feature_window: 20,
            volatility_lookback: 10,
            momentum_lookback: 5,
        }
    }
}

impl MarketFeatureExtractor {
    pub fn new(feature_window: usize, volatility_lookback: usize, momentum_lookback: usize) -> Self {
        Self {
            feature_window,
            volatility_lookback,
            momentum_lookback,
        }
    }

    /// Extract market features for ML optimization
    pub fn extract_features(
        &self,
        performance_history: &HashMap<StrategyType, Vec<StrategyAnalytics>>,
        current_regime: Option<RegimeType>,
    ) -> Vec<f64> {
        let mut features = Vec::new();

        // Strategy performance features
        for strategy in [
            StrategyType::PredictiveMarketMaking,
            StrategyType::MicrostructureMomentum,
            StrategyType::RegimeArbitrage,
            StrategyType::LiquidityHarvesting,
        ] {
            if let Some(analytics_vec) = performance_history.get(&strategy) {
                if !analytics_vec.is_empty() {
                    let latest = analytics_vec.last().unwrap();

                    // Add strategy-specific features
                    features.push(latest.performance.sharpe_ratio);
                    features.push(latest.performance.max_drawdown);
                    features.push(latest.performance.success_rate);
                    features.push(latest.performance.avg_return_per_trade);
                    features.push(latest.rolling_volatility_30d);
                    features.push(latest.rolling_sharpe_30d);
                } else {
                    // Default values if no history
                    features.extend_from_slice(&[0.0, 0.0, 0.5, 0.0, 0.1, 0.0]);
                }
            } else {
                // Default values if strategy not found
                features.extend_from_slice(&[0.0, 0.0, 0.5, 0.0, 0.1, 0.0]);
            }
        }

        // Market regime features
        match current_regime {
            Some(RegimeType::Normal) => features.extend_from_slice(&[1.0, 0.0, 0.0, 0.0]),
            Some(RegimeType::Trending) => features.extend_from_slice(&[0.0, 1.0, 0.0, 0.0]),
            Some(RegimeType::Volatile) => features.extend_from_slice(&[0.0, 0.0, 1.0, 0.0]),
            Some(RegimeType::Crisis) => features.extend_from_slice(&[0.0, 0.0, 0.0, 1.0]),
            None => features.extend_from_slice(&[0.25, 0.25, 0.25, 0.25]), // Equal probability
        }

        // Cross-strategy correlation features
        let correlation_features = self.calculate_correlation_features(performance_history);
        features.extend(correlation_features);

        // Market volatility features
        let volatility_features = self.calculate_volatility_features(performance_history);
        features.extend(volatility_features);

        features
    }

    /// Calculate correlation features between strategies
    fn calculate_correlation_features(
        &self,
        performance_history: &HashMap<StrategyType, Vec<StrategyAnalytics>>,
    ) -> Vec<f64> {
        let mut correlations = Vec::new();

        let strategies = [
            StrategyType::PredictiveMarketMaking,
            StrategyType::MicrostructureMomentum,
            StrategyType::RegimeArbitrage,
            StrategyType::LiquidityHarvesting,
        ];

        // Calculate pairwise correlations
        for i in 0..strategies.len() {
            for j in (i + 1)..strategies.len() {
                let corr = self.calculate_strategy_correlation(
                    performance_history,
                    strategies[i],
                    strategies[j],
                );
                correlations.push(corr);
            }
        }

        correlations
    }

    /// Calculate correlation between two strategies
    fn calculate_strategy_correlation(
        &self,
        performance_history: &HashMap<StrategyType, Vec<StrategyAnalytics>>,
        strategy1: StrategyType,
        strategy2: StrategyType,
    ) -> f64 {
        let returns1 = self.get_strategy_returns(performance_history, strategy1);
        let returns2 = self.get_strategy_returns(performance_history, strategy2);

        if returns1.len() < 2 || returns2.len() < 2 || returns1.len() != returns2.len() {
            return 0.0; // Default correlation
        }

        let mean1 = returns1.iter().sum::<f64>() / returns1.len() as f64;
        let mean2 = returns2.iter().sum::<f64>() / returns2.len() as f64;

        let mut numerator = 0.0;
        let mut sum_sq1 = 0.0;
        let mut sum_sq2 = 0.0;

        for i in 0..returns1.len() {
            let diff1 = returns1[i] - mean1;
            let diff2 = returns2[i] - mean2;
            numerator += diff1 * diff2;
            sum_sq1 += diff1 * diff1;
            sum_sq2 += diff2 * diff2;
        }

        let denominator = (sum_sq1 * sum_sq2).sqrt();
        if denominator > 0.0 {
            numerator / denominator
        } else {
            0.0
        }
    }

    /// Get returns for a specific strategy
    fn get_strategy_returns(
        &self,
        performance_history: &HashMap<StrategyType, Vec<StrategyAnalytics>>,
        strategy: StrategyType,
    ) -> Vec<f64> {
        if let Some(analytics_vec) = performance_history.get(&strategy) {
            analytics_vec.iter()
                .map(|a| a.performance.avg_return_per_trade)
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Calculate volatility features
    fn calculate_volatility_features(
        &self,
        performance_history: &HashMap<StrategyType, Vec<StrategyAnalytics>>,
    ) -> Vec<f64> {
        let mut volatility_features = Vec::new();

        // Portfolio-level volatility
        let portfolio_volatility = self.calculate_portfolio_volatility(performance_history);
        volatility_features.push(portfolio_volatility);

        // Volatility trend (increasing/decreasing)
        let volatility_trend = self.calculate_volatility_trend(performance_history);
        volatility_features.push(volatility_trend);

        // Volatility regime (high/low)
        let volatility_regime = if portfolio_volatility > 0.2 { 1.0 } else { 0.0 };
        volatility_features.push(volatility_regime);

        volatility_features
    }

    /// Calculate portfolio volatility
    fn calculate_portfolio_volatility(
        &self,
        performance_history: &HashMap<StrategyType, Vec<StrategyAnalytics>>,
    ) -> f64 {
        let mut total_volatility = 0.0;
        let mut count = 0;

        for strategy in [
            StrategyType::PredictiveMarketMaking,
            StrategyType::MicrostructureMomentum,
            StrategyType::RegimeArbitrage,
            StrategyType::LiquidityHarvesting,
        ] {
            if let Some(analytics_vec) = performance_history.get(&strategy) {
                if let Some(latest) = analytics_vec.last() {
                    total_volatility += latest.rolling_volatility_30d;
                    count += 1;
                }
            }
        }

        if count > 0 {
            total_volatility / count as f64
        } else {
            0.15 // Default volatility
        }
    }

    /// Calculate volatility trend
    fn calculate_volatility_trend(
        &self,
        performance_history: &HashMap<StrategyType, Vec<StrategyAnalytics>>,
    ) -> f64 {
        // Simplified volatility trend calculation
        let current_vol = self.calculate_portfolio_volatility(performance_history);

        // Compare with historical average (simplified)
        let historical_avg = 0.15; // Assume 15% historical average

        (current_vol - historical_avg) / historical_avg
    }
}

impl StrategyWeightOptimizer {
    /// Create new strategy weight optimizer
    pub fn new(
        database: Arc<Database>,
        optimization_config: OptimizationConfig,
    ) -> Self {
        let mut strategy_configs = HashMap::new();

        // Initialize default configurations for each strategy
        strategy_configs.insert(StrategyType::PredictiveMarketMaking, StrategyWeightConfig {
            target_weight: 0.30,
            risk_budget_allocation: 0.25,
            ..Default::default()
        });

        strategy_configs.insert(StrategyType::MicrostructureMomentum, StrategyWeightConfig {
            target_weight: 0.25,
            risk_budget_allocation: 0.35, // Higher risk budget for momentum
            ..Default::default()
        });

        strategy_configs.insert(StrategyType::RegimeArbitrage, StrategyWeightConfig {
            target_weight: 0.25,
            risk_budget_allocation: 0.25,
            ..Default::default()
        });

        strategy_configs.insert(StrategyType::LiquidityHarvesting, StrategyWeightConfig {
            target_weight: 0.20,
            risk_budget_allocation: 0.15, // Lower risk budget for harvesting
            ..Default::default()
        });

        let ml_optimizer = MLWeightOptimizer::new(&optimization_config);
        let market_feature_extractor = MarketFeatureExtractor::default();

        Self {
            database,
            current_weights: Arc::new(RwLock::new(StrategyWeights::default())),
            strategy_configs,
            performance_history: Arc::new(RwLock::new(HashMap::new())),
            optimization_config,
            ml_optimizer: Arc::new(RwLock::new(ml_optimizer)),
            market_feature_extractor,
        }
    }

    /// Get current strategy weights
    pub async fn get_current_weights(&self) -> StrategyWeights {
        self.current_weights.read().await.clone()
    }

    /// Update strategy performance data
    pub async fn update_strategy_performance(
        &self,
        strategy_type: StrategyType,
        performance: StrategyPerformance,
    ) -> Result<()> {
        let analytics = self.calculate_strategy_analytics(strategy_type, performance).await?;
        
        let mut history = self.performance_history.write().await;
        let strategy_history = history.entry(strategy_type).or_insert_with(Vec::new);
        
        strategy_history.push(analytics);
        
        // Keep only recent history (configurable window)
        let max_history_size = (self.optimization_config.optimization_frequency_hours * 24) as usize;
        if strategy_history.len() > max_history_size {
            strategy_history.drain(0..strategy_history.len() - max_history_size);
        }

        info!("Updated performance for strategy {:?}", strategy_type);
        Ok(())
    }

    /// Calculate comprehensive strategy analytics with advanced performance metrics
    async fn calculate_strategy_analytics(
        &self,
        strategy_type: StrategyType,
        performance: StrategyPerformance,
    ) -> Result<StrategyAnalytics> {
        // Calculate risk-adjusted return
        let risk_adjusted_return = if performance.rolling_volatility_30d > 0.0 {
            performance.avg_return_per_trade / performance.rolling_volatility_30d
        } else {
            0.0
        };

        // Calculate volatility-adjusted Sharpe ratio
        let volatility_adjusted_sharpe = if performance.rolling_volatility_30d > 0.0 {
            (performance.avg_return_per_trade - self.optimization_config.risk_free_rate / 365.0)
                / performance.rolling_volatility_30d
        } else {
            0.0
        };

        // Calculate advanced rolling metrics
        let rolling_metrics = self.calculate_rolling_metrics(&performance).await?;
        let attribution_metrics = self.calculate_attribution_metrics(&performance).await?;
        let regime_metrics = self.calculate_regime_specific_metrics(strategy_type, &performance).await?;
        let risk_decomposition = self.calculate_risk_decomposition(&performance).await?;

        Ok(StrategyAnalytics {
            strategy_type,
            performance: performance.clone(),
            risk_adjusted_return,
            volatility_adjusted_sharpe,
            regime_performance: HashMap::new(), // Will be populated with historical analysis
            correlation_matrix: HashMap::new(), // Will be calculated from returns
            contribution_to_portfolio_risk: 0.0, // Will be calculated
            marginal_var: 0.0,
            component_var: 0.0,
            diversification_ratio: 1.0,

            // Enhanced rolling metrics
            rolling_sharpe_7d: rolling_metrics.sharpe_7d,
            rolling_sharpe_30d: rolling_metrics.sharpe_30d,
            rolling_sortino_7d: rolling_metrics.sortino_7d,
            rolling_sortino_30d: rolling_metrics.sortino_30d,
            rolling_calmar_7d: rolling_metrics.calmar_7d,
            rolling_calmar_30d: rolling_metrics.calmar_30d,
            rolling_max_drawdown_7d: rolling_metrics.max_drawdown_7d,
            rolling_max_drawdown_30d: rolling_metrics.max_drawdown_30d,
            rolling_volatility_7d: rolling_metrics.volatility_7d,
            rolling_volatility_30d: rolling_metrics.volatility_30d,
            rolling_skewness_30d: rolling_metrics.skewness_30d,
            rolling_kurtosis_30d: rolling_metrics.kurtosis_30d,
            rolling_var_95_7d: rolling_metrics.var_95_7d,
            rolling_var_99_7d: rolling_metrics.var_99_7d,
            rolling_expected_shortfall_7d: rolling_metrics.expected_shortfall_7d,
            rolling_expected_shortfall_30d: rolling_metrics.expected_shortfall_30d,

            // Performance attribution metrics
            alpha_vs_market: attribution_metrics.alpha,
            beta_vs_market: attribution_metrics.beta,
            tracking_error_vs_market: attribution_metrics.tracking_error,
            information_ratio_vs_market: attribution_metrics.information_ratio,
            upside_capture_ratio: attribution_metrics.upside_capture,
            downside_capture_ratio: attribution_metrics.downside_capture,
            hit_ratio: attribution_metrics.hit_ratio,
            profit_factor: attribution_metrics.profit_factor,
            recovery_factor: attribution_metrics.recovery_factor,
            sterling_ratio: attribution_metrics.sterling_ratio,
            burke_ratio: attribution_metrics.burke_ratio,
            pain_index: attribution_metrics.pain_index,
            ulcer_index: attribution_metrics.ulcer_index,

            // Regime-specific performance
            regime_sharpe_ratios: regime_metrics.sharpe_ratios,
            regime_max_drawdowns: regime_metrics.max_drawdowns,
            regime_hit_ratios: regime_metrics.hit_ratios,
            regime_profit_factors: regime_metrics.profit_factors,

            // Risk decomposition
            systematic_risk: risk_decomposition.systematic,
            idiosyncratic_risk: risk_decomposition.idiosyncratic,
            tail_risk_contribution: risk_decomposition.tail_risk,
            stress_test_performance: risk_decomposition.stress_tests,

            last_updated: Utc::now(),
        })
    }

    /// Optimize strategy weights based on performance and market conditions
    pub async fn optimize_weights(&self, current_regime: Option<RegimeType>) -> Result<StrategyWeights> {
        info!("Starting enhanced strategy weight optimization with ML");

        let history = self.performance_history.read().await;

        // Check if we have sufficient data for optimization
        if history.len() < 4 {
            warn!("Insufficient strategy performance data for optimization");
            return Ok(self.current_weights.read().await.clone());
        }

        let current_weights = self.current_weights.read().await.clone();
        drop(history);

        // Traditional optimization approaches
        let history = self.performance_history.read().await;
        let mean_variance_weights = self.calculate_mean_variance_weights(&history).await?;
        let risk_parity_weights = self.calculate_risk_parity_weights(&history).await?;
        let kelly_weights = self.calculate_kelly_weights(&history).await?;
        let regime_weights = self.calculate_regime_based_weights(&history, current_regime).await?;

        // Combine traditional optimization approaches
        let traditional_weights = self.combine_optimization_approaches(
            mean_variance_weights,
            risk_parity_weights,
            kelly_weights,
            regime_weights,
        ).await?;

        // ML-based optimization (if enabled and sufficient data)
        let final_weights = if self.optimization_config.enable_ml_optimization {
            self.apply_ml_optimization(&current_weights, &traditional_weights, current_regime, &history).await?
        } else {
            traditional_weights
        };

        drop(history);

        // Apply constraints and validation
        let mut optimized_weights = final_weights;
        self.apply_weight_constraints(&mut optimized_weights).await?;
        optimized_weights.normalize();
        optimized_weights.validate()?;

        // Update current weights
        optimized_weights.last_updated = Utc::now();
        optimized_weights.rebalance_reason = format!(
            "Enhanced optimization: regime={:?}, ML={}, target_sharpe={:.2}",
            current_regime,
            self.optimization_config.enable_ml_optimization,
            self.optimization_config.target_sharpe_ratio
        );

        *self.current_weights.write().await = optimized_weights.clone();

        info!("Enhanced strategy weights optimized: PMM={:.3}, MM={:.3}, RA={:.3}, LH={:.3}",
              optimized_weights.predictive_market_making,
              optimized_weights.microstructure_momentum,
              optimized_weights.regime_arbitrage,
              optimized_weights.liquidity_harvesting);

        Ok(optimized_weights)
    }

    /// Apply ML-based optimization
    async fn apply_ml_optimization(
        &self,
        current_weights: &StrategyWeights,
        traditional_weights: &StrategyWeights,
        current_regime: Option<RegimeType>,
        history: &HashMap<StrategyType, Vec<StrategyAnalytics>>,
    ) -> Result<StrategyWeights> {
        // Check if we have sufficient data for ML optimization
        let total_samples: usize = history.values().map(|v| v.len()).sum();
        if total_samples < self.optimization_config.min_samples_for_ml {
            info!("Insufficient data for ML optimization, using traditional approach");
            return Ok(traditional_weights.clone());
        }

        // Extract market features
        let market_features = self.market_feature_extractor.extract_features(history, current_regime);

        // Calculate performance metrics for each strategy
        let mut performance_metrics = HashMap::new();
        for strategy in [
            StrategyType::PredictiveMarketMaking,
            StrategyType::MicrostructureMomentum,
            StrategyType::RegimeArbitrage,
            StrategyType::LiquidityHarvesting,
        ] {
            let performance = if let Some(analytics_vec) = history.get(&strategy) {
                if let Some(latest) = analytics_vec.last() {
                    latest.performance.avg_return_per_trade
                } else {
                    0.0
                }
            } else {
                0.0
            };
            performance_metrics.insert(strategy, performance);
        }

        // Calculate target return based on current performance
        let target_return = self.calculate_target_return(&performance_metrics, current_regime);

        // Apply ML optimization
        let mut ml_optimizer = self.ml_optimizer.write().await;
        let ml_weights = ml_optimizer.optimize_weights(
            current_weights,
            &performance_metrics,
            &market_features,
            target_return,
        )?;

        // Update ML optimizer performance
        let portfolio_performance = self.calculate_portfolio_performance(&performance_metrics, &ml_weights);
        ml_optimizer.update_performance(portfolio_performance);

        // Ensemble approach: combine traditional and ML weights
        if self.optimization_config.enable_ensemble_optimization {
            let ensemble_weights = self.combine_traditional_and_ml_weights(traditional_weights, &ml_weights)?;
            Ok(ensemble_weights)
        } else {
            Ok(ml_weights)
        }
    }

    /// Calculate target return based on performance and regime
    fn calculate_target_return(
        &self,
        performance_metrics: &HashMap<StrategyType, f64>,
        current_regime: Option<RegimeType>,
    ) -> f64 {
        // Base target return
        let base_target = self.optimization_config.target_sharpe_ratio * 0.01; // 1% base target

        // Adjust based on regime
        let regime_adjustment = match current_regime {
            Some(RegimeType::Trending) => 1.2,  // Higher target in trending markets
            Some(RegimeType::Volatile) => 0.8,  // Lower target in volatile markets
            Some(RegimeType::Crisis) => 0.5,    // Much lower target in crisis
            Some(RegimeType::Normal) => 1.0,    // Normal target
            None => 1.0,
        };

        // Adjust based on recent performance
        let avg_performance: f64 = performance_metrics.values().sum::<f64>() / performance_metrics.len() as f64;
        let performance_adjustment = if avg_performance > 0.0 {
            1.0 + (avg_performance * 0.5) // Increase target if performing well
        } else {
            0.8 // Reduce target if underperforming
        };

        base_target * regime_adjustment * performance_adjustment
    }

    /// Calculate portfolio performance
    fn calculate_portfolio_performance(
        &self,
        performance_metrics: &HashMap<StrategyType, f64>,
        weights: &StrategyWeights,
    ) -> f64 {
        let mut portfolio_return = 0.0;

        for strategy in [
            StrategyType::PredictiveMarketMaking,
            StrategyType::MicrostructureMomentum,
            StrategyType::RegimeArbitrage,
            StrategyType::LiquidityHarvesting,
        ] {
            let weight = weights.get_weight(strategy);
            let strategy_return = performance_metrics.get(&strategy).unwrap_or(&0.0);
            portfolio_return += weight * strategy_return;
        }

        portfolio_return
    }

    /// Combine traditional and ML weights using ensemble approach
    fn combine_traditional_and_ml_weights(
        &self,
        traditional_weights: &StrategyWeights,
        ml_weights: &StrategyWeights,
    ) -> Result<StrategyWeights> {
        // Ensemble weights: 60% traditional, 40% ML
        let traditional_factor = 0.6;
        let ml_factor = 0.4;

        let mut ensemble_weights = StrategyWeights::default();

        ensemble_weights.predictive_market_making =
            traditional_factor * traditional_weights.predictive_market_making +
            ml_factor * ml_weights.predictive_market_making;

        ensemble_weights.microstructure_momentum =
            traditional_factor * traditional_weights.microstructure_momentum +
            ml_factor * ml_weights.microstructure_momentum;

        ensemble_weights.regime_arbitrage =
            traditional_factor * traditional_weights.regime_arbitrage +
            ml_factor * ml_weights.regime_arbitrage;

        ensemble_weights.liquidity_harvesting =
            traditional_factor * traditional_weights.liquidity_harvesting +
            ml_factor * ml_weights.liquidity_harvesting;

        ensemble_weights.normalize();
        Ok(ensemble_weights)
    }

    /// Calculate mean-variance optimal weights
    async fn calculate_mean_variance_weights(
        &self,
        history: &HashMap<StrategyType, Vec<StrategyAnalytics>>,
    ) -> Result<StrategyWeights> {
        // Extract returns and calculate covariance matrix
        let strategies = vec![
            StrategyType::PredictiveMarketMaking,
            StrategyType::MicrostructureMomentum,
            StrategyType::RegimeArbitrage,
            StrategyType::LiquidityHarvesting,
        ];

        let mut returns_matrix = Vec::new();
        let mut mean_returns = Vec::new();

        for strategy in &strategies {
            if let Some(analytics_vec) = history.get(strategy) {
                let returns: Vec<f64> = analytics_vec.iter()
                    .map(|a| a.performance.avg_return_per_trade)
                    .collect();

                if !returns.is_empty() {
                    mean_returns.push(returns.iter().sum::<f64>() / returns.len() as f64);
                    returns_matrix.push(returns);
                } else {
                    mean_returns.push(0.0);
                    returns_matrix.push(vec![0.0]);
                }
            } else {
                mean_returns.push(0.0);
                returns_matrix.push(vec![0.0]);
            }
        }

        // Simple mean-variance optimization (equal risk contribution as fallback)
        let mut weights = StrategyWeights::default();

        // Weight by inverse volatility (risk parity approximation)
        let mut inv_vol_weights = Vec::new();
        for strategy in &strategies {
            if let Some(analytics_vec) = history.get(strategy) {
                let latest_analytics = analytics_vec.last().unwrap();
                let vol = latest_analytics.performance.rolling_volatility_30d.max(1e-6);
                inv_vol_weights.push(1.0 / vol);
            } else {
                inv_vol_weights.push(1.0);
            }
        }

        let total_inv_vol: f64 = inv_vol_weights.iter().sum();
        if total_inv_vol > 0.0 {
            weights.predictive_market_making = inv_vol_weights[0] / total_inv_vol;
            weights.microstructure_momentum = inv_vol_weights[1] / total_inv_vol;
            weights.regime_arbitrage = inv_vol_weights[2] / total_inv_vol;
            weights.liquidity_harvesting = inv_vol_weights[3] / total_inv_vol;
        }

        Ok(weights)
    }

    /// Calculate risk parity weights
    async fn calculate_risk_parity_weights(
        &self,
        history: &HashMap<StrategyType, Vec<StrategyAnalytics>>,
    ) -> Result<StrategyWeights> {
        let strategies = vec![
            StrategyType::PredictiveMarketMaking,
            StrategyType::MicrostructureMomentum,
            StrategyType::RegimeArbitrage,
            StrategyType::LiquidityHarvesting,
        ];

        let mut risk_contributions = Vec::new();

        for strategy in &strategies {
            if let Some(analytics_vec) = history.get(strategy) {
                let latest_analytics = analytics_vec.last().unwrap();
                let risk_contrib = latest_analytics.performance.rolling_volatility_30d
                    * latest_analytics.performance.var_95.abs();
                risk_contributions.push(risk_contrib.max(1e-6));
            } else {
                risk_contributions.push(1e-6);
            }
        }

        // Inverse risk weighting
        let inv_risk_weights: Vec<f64> = risk_contributions.iter()
            .map(|&risk| 1.0 / risk)
            .collect();

        let total_inv_risk: f64 = inv_risk_weights.iter().sum();

        let mut weights = StrategyWeights::default();
        if total_inv_risk > 0.0 {
            weights.predictive_market_making = inv_risk_weights[0] / total_inv_risk;
            weights.microstructure_momentum = inv_risk_weights[1] / total_inv_risk;
            weights.regime_arbitrage = inv_risk_weights[2] / total_inv_risk;
            weights.liquidity_harvesting = inv_risk_weights[3] / total_inv_risk;
        }

        Ok(weights)
    }

    /// Calculate Kelly criterion optimal weights
    async fn calculate_kelly_weights(
        &self,
        history: &HashMap<StrategyType, Vec<StrategyAnalytics>>,
    ) -> Result<StrategyWeights> {
        let strategies = vec![
            StrategyType::PredictiveMarketMaking,
            StrategyType::MicrostructureMomentum,
            StrategyType::RegimeArbitrage,
            StrategyType::LiquidityHarvesting,
        ];

        let mut kelly_fractions = Vec::new();

        for strategy in &strategies {
            if let Some(analytics_vec) = history.get(strategy) {
                let latest_analytics = analytics_vec.last().unwrap();
                let perf = &latest_analytics.performance;

                // Kelly fraction = (win_rate * avg_win - loss_rate * avg_loss) / avg_win
                let win_rate = perf.success_rate;
                let loss_rate = 1.0 - win_rate;
                let avg_win = if perf.winning_trades > 0 {
                    perf.total_pnl.max(0.0) / perf.winning_trades as f64
                } else {
                    0.0
                };
                let avg_loss = if perf.total_trades > perf.winning_trades {
                    perf.total_pnl.min(0.0).abs() / (perf.total_trades - perf.winning_trades) as f64
                } else {
                    1.0
                };

                let kelly_fraction = if avg_win > 0.0 {
                    ((win_rate * avg_win - loss_rate * avg_loss) / avg_win)
                        .max(0.0)
                        .min(self.optimization_config.kelly_fraction_limit)
                } else {
                    0.0
                };

                kelly_fractions.push(kelly_fraction);
            } else {
                kelly_fractions.push(0.0);
            }
        }

        // Normalize Kelly fractions to sum to 1.0
        let total_kelly: f64 = kelly_fractions.iter().sum();

        let mut weights = StrategyWeights::default();
        if total_kelly > 0.0 {
            weights.predictive_market_making = kelly_fractions[0] / total_kelly;
            weights.microstructure_momentum = kelly_fractions[1] / total_kelly;
            weights.regime_arbitrage = kelly_fractions[2] / total_kelly;
            weights.liquidity_harvesting = kelly_fractions[3] / total_kelly;
        }

        Ok(weights)
    }

    /// Calculate regime-based optimal weights
    async fn calculate_regime_based_weights(
        &self,
        _history: &HashMap<StrategyType, Vec<StrategyAnalytics>>,
        current_regime: Option<RegimeType>,
    ) -> Result<StrategyWeights> {
        let mut weights = StrategyWeights::default();

        match current_regime {
            Some(RegimeType::Trending) => {
                // Favor momentum and regime arbitrage in trending markets
                weights.predictive_market_making = 0.20;
                weights.microstructure_momentum = 0.40;
                weights.regime_arbitrage = 0.30;
                weights.liquidity_harvesting = 0.10;
            },
            Some(RegimeType::Volatile) => {
                // Favor market making and liquidity harvesting in volatile markets
                weights.predictive_market_making = 0.40;
                weights.microstructure_momentum = 0.15;
                weights.regime_arbitrage = 0.20;
                weights.liquidity_harvesting = 0.25;
            },
            Some(RegimeType::Normal) => {
                // Balanced allocation in normal markets
                weights.predictive_market_making = 0.30;
                weights.microstructure_momentum = 0.25;
                weights.regime_arbitrage = 0.25;
                weights.liquidity_harvesting = 0.20;
            },
            Some(RegimeType::Crisis) => {
                // Conservative allocation in crisis - favor liquidity and reduce risk
                weights.predictive_market_making = 0.50;
                weights.microstructure_momentum = 0.10;
                weights.regime_arbitrage = 0.15;
                weights.liquidity_harvesting = 0.25;
            },
            Some(RegimeType::Bullish) => {
                // Favor momentum strategies in bullish markets
                weights.microstructure_momentum = 0.40;
                weights.predictive_market_making = 0.30;
                weights.regime_arbitrage = 0.20;
                weights.liquidity_harvesting = 0.10;
            }
            Some(RegimeType::Bearish) => {
                // Favor defensive strategies in bearish markets
                weights.predictive_market_making = 0.40;
                weights.liquidity_harvesting = 0.30;
                weights.regime_arbitrage = 0.20;
                weights.microstructure_momentum = 0.10;
            }
            Some(RegimeType::Sideways) => {
                // Favor market making in sideways markets
                weights.predictive_market_making = 0.50;
                weights.liquidity_harvesting = 0.30;
                weights.regime_arbitrage = 0.15;
                weights.microstructure_momentum = 0.05;
            }
            None => {
                // Default balanced allocation
                weights = StrategyWeights::default();
            }
            _ => {
                // Handle any other regime types with default allocation
                weights = StrategyWeights::default();
            }
        }

        Ok(weights)
    }

    /// Combine multiple optimization approaches
    async fn combine_optimization_approaches(
        &self,
        mean_variance: StrategyWeights,
        risk_parity: StrategyWeights,
        kelly: StrategyWeights,
        regime: StrategyWeights,
    ) -> Result<StrategyWeights> {
        // Weighted combination of different approaches
        let mv_weight = 0.30;  // Mean-variance optimization
        let rp_weight = 0.25;  // Risk parity
        let kelly_weight = 0.20; // Kelly criterion
        let regime_weight = 0.25; // Regime-based

        let mut combined = StrategyWeights::default();

        combined.predictive_market_making =
            mv_weight * mean_variance.predictive_market_making +
            rp_weight * risk_parity.predictive_market_making +
            kelly_weight * kelly.predictive_market_making +
            regime_weight * regime.predictive_market_making;

        combined.microstructure_momentum =
            mv_weight * mean_variance.microstructure_momentum +
            rp_weight * risk_parity.microstructure_momentum +
            kelly_weight * kelly.microstructure_momentum +
            regime_weight * regime.microstructure_momentum;

        combined.regime_arbitrage =
            mv_weight * mean_variance.regime_arbitrage +
            rp_weight * risk_parity.regime_arbitrage +
            kelly_weight * kelly.regime_arbitrage +
            regime_weight * regime.regime_arbitrage;

        combined.liquidity_harvesting =
            mv_weight * mean_variance.liquidity_harvesting +
            rp_weight * risk_parity.liquidity_harvesting +
            kelly_weight * kelly.liquidity_harvesting +
            regime_weight * regime.liquidity_harvesting;

        Ok(combined)
    }

    /// Apply weight constraints and limits
    async fn apply_weight_constraints(&self, weights: &mut StrategyWeights) -> Result<()> {
        let strategies = vec![
            StrategyType::PredictiveMarketMaking,
            StrategyType::MicrostructureMomentum,
            StrategyType::RegimeArbitrage,
            StrategyType::LiquidityHarvesting,
        ];

        for strategy in strategies {
            let config = self.strategy_configs.get(&strategy).unwrap();
            let current_weight = weights.get_weight(strategy);

            // Apply min/max constraints
            let constrained_weight = current_weight
                .max(config.min_weight)
                .min(config.max_weight);

            weights.set_weight(strategy, constrained_weight);
        }

        Ok(())
    }

    /// Calculate rolling performance metrics
    async fn calculate_rolling_metrics(&self, performance: &StrategyPerformance) -> Result<RollingMetrics> {
        // Extract daily returns from performance data
        let daily_returns = &performance.daily_returns;

        if daily_returns.is_empty() {
            return Ok(RollingMetrics::default());
        }

        // Calculate 7-day rolling metrics
        let returns_7d = if daily_returns.len() >= 7 {
            &daily_returns[daily_returns.len() - 7..]
        } else {
            daily_returns
        };

        let returns_30d = if daily_returns.len() >= 30 {
            &daily_returns[daily_returns.len() - 30..]
        } else {
            daily_returns
        };

        Ok(RollingMetrics {
            sharpe_7d: self.calculate_sharpe_ratio(returns_7d),
            sharpe_30d: self.calculate_sharpe_ratio(returns_30d),
            sortino_7d: self.calculate_sortino_ratio(returns_7d),
            sortino_30d: self.calculate_sortino_ratio(returns_30d),
            calmar_7d: self.calculate_calmar_ratio(returns_7d),
            calmar_30d: self.calculate_calmar_ratio(returns_30d),
            max_drawdown_7d: self.calculate_max_drawdown(returns_7d),
            max_drawdown_30d: self.calculate_max_drawdown(returns_30d),
            volatility_7d: self.calculate_volatility(returns_7d),
            volatility_30d: self.calculate_volatility(returns_30d),
            skewness_30d: self.calculate_skewness(returns_30d),
            kurtosis_30d: self.calculate_kurtosis(returns_30d),
            var_95_7d: self.calculate_var(returns_7d, 0.95),
            var_99_7d: self.calculate_var(returns_7d, 0.99),
            expected_shortfall_7d: self.calculate_expected_shortfall(returns_7d, 0.95),
            expected_shortfall_30d: self.calculate_expected_shortfall(returns_30d, 0.95),
        })
    }

    /// Calculate performance attribution metrics
    async fn calculate_attribution_metrics(&self, performance: &StrategyPerformance) -> Result<AttributionMetrics> {
        // For now, use existing performance metrics as basis
        // In production, this would compare against market benchmarks

        Ok(AttributionMetrics {
            alpha: performance.alpha,
            beta: performance.beta,
            tracking_error: performance.tracking_error,
            information_ratio: performance.information_ratio,
            upside_capture: performance.upside_capture,
            downside_capture: performance.downside_capture,
            hit_ratio: performance.success_rate,
            profit_factor: performance.profit_factor,
            recovery_factor: performance.recovery_factor,
            sterling_ratio: if performance.max_drawdown > 0.0 {
                performance.avg_return_per_trade / performance.max_drawdown
            } else {
                0.0
            },
            burke_ratio: self.calculate_burke_ratio(&performance.daily_returns),
            pain_index: self.calculate_pain_index(&performance.daily_returns),
            ulcer_index: self.calculate_ulcer_index(&performance.daily_returns),
        })
    }

    /// Calculate regime-specific performance metrics
    async fn calculate_regime_specific_metrics(
        &self,
        _strategy_type: StrategyType,
        performance: &StrategyPerformance,
    ) -> Result<RegimeMetrics> {
        // This would require historical regime data analysis
        // For now, return default values
        let mut sharpe_ratios = HashMap::new();
        let mut max_drawdowns = HashMap::new();
        let mut hit_ratios = HashMap::new();
        let mut profit_factors = HashMap::new();

        // Placeholder values - in production, calculate from historical regime data
        for regime in [RegimeType::Normal, RegimeType::Trending, RegimeType::Volatile, RegimeType::Crisis] {
            sharpe_ratios.insert(regime, performance.sharpe_ratio * 0.8); // Slightly lower than overall
            max_drawdowns.insert(regime, performance.max_drawdown * 1.2); // Slightly higher than overall
            hit_ratios.insert(regime, performance.success_rate * 0.9); // Slightly lower than overall
            profit_factors.insert(regime, performance.profit_factor * 0.85); // Slightly lower than overall
        }

        Ok(RegimeMetrics {
            sharpe_ratios,
            max_drawdowns,
            hit_ratios,
            profit_factors,
        })
    }

    /// Calculate risk decomposition metrics
    async fn calculate_risk_decomposition(&self, performance: &StrategyPerformance) -> Result<RiskDecomposition> {
        let total_risk = performance.rolling_volatility_30d;

        // Estimate systematic vs idiosyncratic risk
        let systematic_risk = total_risk * performance.beta.abs();
        let idiosyncratic_risk = (total_risk.powi(2) - systematic_risk.powi(2)).sqrt().max(0.0);

        // Estimate tail risk contribution
        let tail_risk = performance.var_95.abs() * 0.3; // Approximate tail contribution

        // Stress test scenarios
        let mut stress_tests = HashMap::new();
        stress_tests.insert("market_crash".to_string(), performance.expected_shortfall * 1.5);
        stress_tests.insert("volatility_spike".to_string(), performance.var_95 * 2.0);
        stress_tests.insert("liquidity_crisis".to_string(), performance.max_drawdown * 1.3);
        stress_tests.insert("regime_change".to_string(), performance.rolling_volatility_30d * 1.8);

        Ok(RiskDecomposition {
            systematic: systematic_risk,
            idiosyncratic: idiosyncratic_risk,
            tail_risk,
            stress_tests,
        })
    }

    /// Calculate portfolio-level metrics
    pub async fn calculate_portfolio_metrics(&self) -> Result<PortfolioOptimizationMetrics> {
        let weights = self.current_weights.read().await;
        let history = self.performance_history.read().await;

        // Calculate weighted portfolio metrics
        let mut portfolio_sharpe = 0.0;
        let mut portfolio_volatility = 0.0;
        let mut portfolio_return = 0.0;
        let mut _total_weight = 0.0;

        let strategies = vec![
            StrategyType::PredictiveMarketMaking,
            StrategyType::MicrostructureMomentum,
            StrategyType::RegimeArbitrage,
            StrategyType::LiquidityHarvesting,
        ];

        for strategy in &strategies {
            let weight = weights.get_weight(*strategy);
            if let Some(analytics_vec) = history.get(strategy) {
                if let Some(latest) = analytics_vec.last() {
                    portfolio_sharpe += weight * latest.performance.sharpe_ratio;
                    portfolio_volatility += weight * latest.performance.rolling_volatility_30d;
                    portfolio_return += weight * latest.performance.avg_return_per_trade;
                    _total_weight += weight;
                }
            }
        }

        Ok(PortfolioOptimizationMetrics {
            portfolio_sharpe_ratio: portfolio_sharpe,
            portfolio_volatility,
            portfolio_return,
            diversification_ratio: self.calculate_diversification_ratio().await?,
            risk_concentration: self.calculate_risk_concentration().await?,
            weight_stability: self.calculate_weight_stability().await?,
            last_optimization: weights.last_updated,
        })
    }

    /// Calculate diversification ratio
    async fn calculate_diversification_ratio(&self) -> Result<f64> {
        // Simplified diversification ratio calculation
        // In practice, this would use the full covariance matrix
        let weights = self.current_weights.read().await;

        // Herfindahl index as proxy for concentration
        let herfindahl = weights.predictive_market_making.powi(2) +
                        weights.microstructure_momentum.powi(2) +
                        weights.regime_arbitrage.powi(2) +
                        weights.liquidity_harvesting.powi(2);

        // Diversification ratio = 1 / sqrt(Herfindahl)
        Ok(1.0 / herfindahl.sqrt())
    }

    /// Calculate risk concentration
    async fn calculate_risk_concentration(&self) -> Result<f64> {
        // Risk concentration based on weight distribution
        let weights = self.current_weights.read().await;
        let max_weight = weights.predictive_market_making
            .max(weights.microstructure_momentum)
            .max(weights.regime_arbitrage)
            .max(weights.liquidity_harvesting);

        Ok(max_weight)
    }

    /// Calculate weight stability (how much weights have changed)
    async fn calculate_weight_stability(&self) -> Result<f64> {
        // This would compare current weights to previous weights
        // For now, return a placeholder
        Ok(0.95) // 95% stability
    }

    // ============================================================================
    // ADVANCED MATHEMATICAL CALCULATIONS FOR PERFORMANCE METRICS
    // ============================================================================

    /// Calculate Sharpe ratio for given returns
    fn calculate_sharpe_ratio(&self, returns: &[f64]) -> f64 {
        if returns.is_empty() {
            return 0.0;
        }

        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>() / returns.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev > 0.0 {
            (mean_return - self.optimization_config.risk_free_rate / 365.0) / std_dev
        } else {
            0.0
        }
    }

    /// Calculate Sortino ratio (downside deviation)
    fn calculate_sortino_ratio(&self, returns: &[f64]) -> f64 {
        if returns.is_empty() {
            return 0.0;
        }

        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let downside_returns: Vec<f64> = returns.iter()
            .filter(|&&r| r < 0.0)
            .cloned()
            .collect();

        if downside_returns.is_empty() {
            return if mean_return > 0.0 { f64::INFINITY } else { 0.0 };
        }

        let downside_variance = downside_returns.iter()
            .map(|r| r.powi(2))
            .sum::<f64>() / downside_returns.len() as f64;
        let downside_deviation = downside_variance.sqrt();

        if downside_deviation > 0.0 {
            (mean_return - self.optimization_config.risk_free_rate / 365.0) / downside_deviation
        } else {
            0.0
        }
    }

    /// Calculate Calmar ratio (return / max drawdown)
    fn calculate_calmar_ratio(&self, returns: &[f64]) -> f64 {
        if returns.is_empty() {
            return 0.0;
        }

        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let max_drawdown = self.calculate_max_drawdown(returns);

        if max_drawdown > 0.0 {
            mean_return / max_drawdown
        } else {
            0.0
        }
    }

    /// Calculate maximum drawdown
    fn calculate_max_drawdown(&self, returns: &[f64]) -> f64 {
        if returns.is_empty() {
            return 0.0;
        }

        let mut cumulative = 1.0;
        let mut peak = 1.0;
        let mut max_dd = 0.0;

        for &ret in returns {
            cumulative *= 1.0 + ret;
            if cumulative > peak {
                peak = cumulative;
            }
            let drawdown = (peak - cumulative) / peak;
            if drawdown > max_dd {
                max_dd = drawdown;
            }
        }

        max_dd
    }

    /// Calculate volatility (standard deviation)
    fn calculate_volatility(&self, returns: &[f64]) -> f64 {
        if returns.len() < 2 {
            return 0.0;
        }

        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / (returns.len() - 1) as f64;

        variance.sqrt()
    }

    /// Calculate skewness
    fn calculate_skewness(&self, returns: &[f64]) -> f64 {
        if returns.len() < 3 {
            return 0.0;
        }

        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let std_dev = self.calculate_volatility(returns);

        if std_dev == 0.0 {
            return 0.0;
        }

        let n = returns.len() as f64;
        let skew_sum = returns.iter()
            .map(|r| ((r - mean) / std_dev).powi(3))
            .sum::<f64>();

        (n / ((n - 1.0) * (n - 2.0))) * skew_sum
    }

    /// Calculate kurtosis
    fn calculate_kurtosis(&self, returns: &[f64]) -> f64 {
        if returns.len() < 4 {
            return 0.0;
        }

        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let std_dev = self.calculate_volatility(returns);

        if std_dev == 0.0 {
            return 0.0;
        }

        let n = returns.len() as f64;
        let kurt_sum = returns.iter()
            .map(|r| ((r - mean) / std_dev).powi(4))
            .sum::<f64>();

        let excess_kurtosis = (n * (n + 1.0) / ((n - 1.0) * (n - 2.0) * (n - 3.0))) * kurt_sum
            - (3.0 * (n - 1.0).powi(2) / ((n - 2.0) * (n - 3.0)));

        excess_kurtosis
    }

    /// Calculate Value at Risk (VaR)
    fn calculate_var(&self, returns: &[f64], confidence_level: f64) -> f64 {
        if returns.is_empty() {
            return 0.0;
        }

        let mut sorted_returns = returns.to_vec();
        sorted_returns.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let index = ((1.0 - confidence_level) * sorted_returns.len() as f64) as usize;
        let index = index.min(sorted_returns.len() - 1);

        -sorted_returns[index] // VaR is typically reported as positive
    }

    /// Calculate Expected Shortfall (Conditional VaR)
    fn calculate_expected_shortfall(&self, returns: &[f64], confidence_level: f64) -> f64 {
        if returns.is_empty() {
            return 0.0;
        }

        let var = self.calculate_var(returns, confidence_level);
        let mut sorted_returns = returns.to_vec();
        sorted_returns.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let threshold = -var;
        let tail_returns: Vec<f64> = sorted_returns.iter()
            .filter(|&&r| r <= threshold)
            .cloned()
            .collect();

        if tail_returns.is_empty() {
            return var;
        }

        -tail_returns.iter().sum::<f64>() / tail_returns.len() as f64
    }

    /// Calculate Burke ratio
    fn calculate_burke_ratio(&self, returns: &[f64]) -> f64 {
        if returns.is_empty() {
            return 0.0;
        }

        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;

        // Calculate sum of squared drawdowns
        let mut cumulative = 1.0;
        let mut peak = 1.0;
        let mut squared_drawdowns = 0.0;

        for &ret in returns {
            cumulative *= 1.0 + ret;
            if cumulative > peak {
                peak = cumulative;
            }
            let drawdown = (peak - cumulative) / peak;
            squared_drawdowns += drawdown.powi(2);
        }

        let burke_denominator = (squared_drawdowns / returns.len() as f64).sqrt();

        if burke_denominator > 0.0 {
            mean_return / burke_denominator
        } else {
            0.0
        }
    }

    /// Calculate Pain Index (average drawdown)
    fn calculate_pain_index(&self, returns: &[f64]) -> f64 {
        if returns.is_empty() {
            return 0.0;
        }

        let mut cumulative = 1.0;
        let mut peak = 1.0;
        let mut total_pain = 0.0;

        for &ret in returns {
            cumulative *= 1.0 + ret;
            if cumulative > peak {
                peak = cumulative;
            }
            let drawdown = (peak - cumulative) / peak;
            total_pain += drawdown;
        }

        total_pain / returns.len() as f64
    }

    /// Calculate Ulcer Index (RMS of drawdowns)
    fn calculate_ulcer_index(&self, returns: &[f64]) -> f64 {
        if returns.is_empty() {
            return 0.0;
        }

        let mut cumulative = 1.0;
        let mut peak = 1.0;
        let mut squared_drawdowns = 0.0;

        for &ret in returns {
            cumulative *= 1.0 + ret;
            if cumulative > peak {
                peak = cumulative;
            }
            let drawdown = (peak - cumulative) / peak;
            squared_drawdowns += drawdown.powi(2);
        }

        (squared_drawdowns / returns.len() as f64).sqrt()
    }
}

/// Portfolio optimization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioOptimizationMetrics {
    pub portfolio_sharpe_ratio: f64,
    pub portfolio_volatility: f64,
    pub portfolio_return: f64,
    pub diversification_ratio: f64,
    pub risk_concentration: f64,
    pub weight_stability: f64,
    pub last_optimization: DateTime<Utc>,
}
