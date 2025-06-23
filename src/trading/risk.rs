use crate::trading::signals::{TradingSignal, RiskAssessment, Position};
use crate::utils::Result;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;

// Risk Management Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskManagerConfig {
    pub max_position_size: f64,
    pub max_portfolio_var: f64,
    pub max_correlation_exposure: f64,
    pub max_drawdown_threshold: f64,
    pub confidence_level: f64,
    pub var_lookback_days: u32,
    pub stress_test_scenarios: Vec<StressScenario>,
    pub position_size_method: PositionSizingMethod,
}

impl Default for RiskManagerConfig {
    fn default() -> Self {
        Self {
            max_position_size: 100000.0,     // $100k max position
            max_portfolio_var: 50000.0,      // $50k max portfolio VaR
            max_correlation_exposure: 0.3,   // 30% max correlation exposure
            max_drawdown_threshold: 0.05,    // 5% max drawdown
            confidence_level: 0.95,          // 95% confidence level for VaR
            var_lookback_days: 252,          // 1 year of trading days
            stress_test_scenarios: vec![
                StressScenario::MarketCrash,
                StressScenario::VolatilitySpike,
                StressScenario::LiquidityCrisis,
            ],
            position_size_method: PositionSizingMethod::KellyOptimal,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PositionSizingMethod {
    FixedFractional(f64),
    KellyOptimal,
    VolatilityTargeting(f64),
    RiskParity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StressScenario {
    MarketCrash,
    VolatilitySpike,
    LiquidityCrisis,
    InterestRateShock,
    CurrencyCrisis,
}

// Risk Manager Implementation
#[derive(Clone)]
pub struct RiskManager {
    config: RiskManagerConfig,
    position_tracker: HashMap<Uuid, Position>,
    historical_returns: HashMap<Uuid, Vec<f64>>,
    correlation_matrix: HashMap<(Uuid, Uuid), f64>,
    portfolio_metrics: PortfolioMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioMetrics {
    pub total_exposure: f64,
    pub net_exposure: f64,
    pub gross_exposure: f64,
    pub portfolio_var: f64,
    pub portfolio_expected_shortfall: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub beta: f64,
    pub last_updated: DateTime<Utc>,
}

impl Default for PortfolioMetrics {
    fn default() -> Self {
        Self {
            total_exposure: 0.0,
            net_exposure: 0.0,
            gross_exposure: 0.0,
            portfolio_var: 0.0,
            portfolio_expected_shortfall: 0.0,
            max_drawdown: 0.0,
            sharpe_ratio: 0.0,
            beta: 1.0,
            last_updated: Utc::now(),
        }
    }
}

impl RiskManager {
    pub fn new(max_position_size: f64) -> Self {
        let mut config = RiskManagerConfig::default();
        config.max_position_size = max_position_size;

        Self {
            config,
            position_tracker: HashMap::new(),
            historical_returns: HashMap::new(),
            correlation_matrix: HashMap::new(),
            portfolio_metrics: PortfolioMetrics::default(),
        }
    }

    pub fn with_config(config: RiskManagerConfig) -> Self {
        Self {
            config,
            position_tracker: HashMap::new(),
            historical_returns: HashMap::new(),
            correlation_matrix: HashMap::new(),
            portfolio_metrics: PortfolioMetrics::default(),
        }
    }

    /// Assess risk for a trading signal
    pub async fn assess_signal_risk(
        &self,
        signal: &TradingSignal,
        active_positions: &HashMap<Uuid, Position>,
    ) -> Result<RiskAssessment> {
        // 1. Calculate position size based on risk method
        let suggested_size = self.calculate_position_size(signal).await?;

        // 2. Check position limits
        let size_after_limits = self.apply_position_limits(signal.instrument_id, suggested_size);

        // 3. Calculate VaR impact
        let var_impact = self.calculate_var_impact(signal, size_after_limits, active_positions).await?;

        // 4. Check portfolio constraints
        let portfolio_impact = self.assess_portfolio_impact(signal, size_after_limits, active_positions).await?;

        // 5. Calculate correlation risk
        let correlation_risk = self.calculate_correlation_risk(signal.instrument_id, active_positions).await?;

        // 6. Liquidity risk assessment
        let liquidity_risk = self.assess_liquidity_risk(signal).await?;

        // 7. Overall risk decision
        let is_acceptable = self.is_risk_acceptable(&var_impact, &portfolio_impact, correlation_risk, liquidity_risk);

        Ok(RiskAssessment {
            is_acceptable,
            adjusted_position_size: if is_acceptable { size_after_limits } else { 0.0 },
            var_95: var_impact.var_95,
            expected_shortfall: var_impact.expected_shortfall,
            max_drawdown_risk: portfolio_impact.max_drawdown_risk,
            correlation_risk,
            liquidity_risk,
        })
    }

    /// Calculate optimal position size based on configured method
    async fn calculate_position_size(&self, signal: &TradingSignal) -> Result<f64> {
        match &self.config.position_size_method {
            PositionSizingMethod::FixedFractional(fraction) => {
                Ok(self.portfolio_metrics.total_exposure * fraction)
            },
            PositionSizingMethod::KellyOptimal => {
                self.calculate_kelly_position_size(signal).await
            },
            PositionSizingMethod::VolatilityTargeting(target_vol) => {
                self.calculate_volatility_targeted_size(signal, *target_vol).await
            },
            PositionSizingMethod::RiskParity => {
                self.calculate_risk_parity_size(signal).await
            },
        }
    }

    /// Kelly Criterion position sizing
    async fn calculate_kelly_position_size(&self, signal: &TradingSignal) -> Result<f64> {
        let win_probability = signal.confidence_score;
        let expected_return = signal.expected_return.unwrap_or(0.0);
        let max_loss = signal.max_risk.unwrap_or(0.02); // Default 2% max loss

        if win_probability <= 0.0 || expected_return <= 0.0 || max_loss <= 0.0 {
            return Ok(0.0);
        }

        // Kelly formula: f = (bp - q) / b
        // where b = odds received on the wager, p = probability of winning, q = probability of losing
        let odds = expected_return / max_loss;
        let kelly_fraction = (odds * win_probability - (1.0 - win_probability)) / odds;

        // Apply Kelly fraction with safety margin (typically 25% of full Kelly)
        let safe_kelly_fraction = (kelly_fraction * 0.25).max(0.0).min(0.1); // Cap at 10%

        Ok(self.portfolio_metrics.total_exposure * safe_kelly_fraction)
    }

    /// Volatility targeting position sizing
    async fn calculate_volatility_targeted_size(&self, signal: &TradingSignal, target_vol: f64) -> Result<f64> {
        // Get historical volatility for the instrument
        let historical_vol = self.get_historical_volatility(signal.instrument_id).await.unwrap_or(0.2); // Default 20%

        if historical_vol <= 0.0 {
            return Ok(0.0);
        }

        // Scale position size inversely to volatility
        let vol_scalar = target_vol / historical_vol;
        let base_size = self.portfolio_metrics.total_exposure * 0.05; // 5% base allocation

        Ok(base_size * vol_scalar)
    }

    /// Risk parity position sizing
    async fn calculate_risk_parity_size(&self, signal: &TradingSignal) -> Result<f64> {
        // Simplified risk parity: equal risk contribution
        let instrument_vol = self.get_historical_volatility(signal.instrument_id).await.unwrap_or(0.2);
        let target_risk_contribution = 0.05; // 5% risk contribution

        if instrument_vol <= 0.0 {
            return Ok(0.0);
        }

        // Position size = target risk / volatility
        Ok(self.portfolio_metrics.total_exposure * target_risk_contribution / instrument_vol)
    }

    /// Apply position limits and constraints
    fn apply_position_limits(&self, instrument_id: Uuid, suggested_size: f64) -> f64 {
        let max_size = self.config.max_position_size;

        // Check existing position
        let existing_position = self.position_tracker.get(&instrument_id)
            .map(|p| p.size.abs())
            .unwrap_or(0.0);

        let available_capacity = max_size - existing_position;
        suggested_size.min(available_capacity).max(0.0)
    }

    /// Calculate VaR impact of new position
    async fn calculate_var_impact(
        &self,
        signal: &TradingSignal,
        position_size: f64,
        _active_positions: &HashMap<Uuid, Position>,
    ) -> Result<VarImpact> {
        let instrument_vol = self.get_historical_volatility(signal.instrument_id).await.unwrap_or(0.2);
        let confidence_level = self.config.confidence_level;

        // Simplified VaR calculation using normal distribution
        let z_score = match confidence_level {
            x if x >= 0.99 => 2.33,
            x if x >= 0.95 => 1.65,
            x if x >= 0.90 => 1.28,
            _ => 1.65,
        };

        let position_value = position_size * signal.entry_price.unwrap_or(1.0);
        let var_95 = position_value * instrument_vol * z_score;
        let expected_shortfall = var_95 * 1.3; // Approximation: ES ≈ 1.3 * VaR for normal distribution

        Ok(VarImpact {
            var_95,
            expected_shortfall,
            confidence_level,
        })
    }

    /// Assess portfolio-level impact
    async fn assess_portfolio_impact(
        &self,
        _signal: &TradingSignal,
        position_size: f64,
        active_positions: &HashMap<Uuid, Position>,
    ) -> Result<PortfolioImpact> {
        let current_exposure = active_positions.values()
            .map(|p| p.size.abs() * p.entry_price)
            .sum::<f64>();

        let new_exposure = current_exposure + position_size;
        let exposure_ratio = new_exposure / self.portfolio_metrics.total_exposure.max(1.0);

        // Estimate drawdown risk based on exposure
        let max_drawdown_risk = (exposure_ratio * 0.1).min(self.config.max_drawdown_threshold);

        Ok(PortfolioImpact {
            exposure_increase: position_size,
            new_total_exposure: new_exposure,
            exposure_ratio,
            max_drawdown_risk,
        })
    }

    /// Calculate correlation risk with existing positions
    async fn calculate_correlation_risk(
        &self,
        instrument_id: Uuid,
        active_positions: &HashMap<Uuid, Position>,
    ) -> Result<f64> {
        let mut total_correlation_exposure = 0.0;

        for (existing_id, position) in active_positions {
            if *existing_id == instrument_id {
                continue;
            }

            let correlation = self.correlation_matrix
                .get(&(instrument_id, *existing_id))
                .or_else(|| self.correlation_matrix.get(&(*existing_id, instrument_id)))
                .unwrap_or(&0.0);

            let correlation_exposure = correlation.abs() * position.size.abs();
            total_correlation_exposure += correlation_exposure;
        }

        Ok(total_correlation_exposure / self.portfolio_metrics.total_exposure.max(1.0))
    }

    /// Assess liquidity risk
    async fn assess_liquidity_risk(&self, signal: &TradingSignal) -> Result<f64> {
        // Simplified liquidity risk based on urgency and market conditions
        let urgency_risk = signal.urgency_score.unwrap_or(0.5);
        let time_horizon_risk = match signal.time_horizon {
            Some(duration) if duration.as_secs() < 300 => 0.8, // High risk for < 5 min
            Some(duration) if duration.as_secs() < 3600 => 0.5, // Medium risk for < 1 hour
            _ => 0.2, // Low risk for longer horizons
        };

        Ok((urgency_risk + time_horizon_risk) / 2.0)
    }

    /// Determine if risk is acceptable based on all factors
    fn is_risk_acceptable(
        &self,
        var_impact: &VarImpact,
        portfolio_impact: &PortfolioImpact,
        correlation_risk: f64,
        liquidity_risk: f64,
    ) -> bool {
        // Check VaR limits
        if var_impact.var_95 > self.config.max_portfolio_var * 0.1 { // Single position shouldn't exceed 10% of portfolio VaR
            return false;
        }

        // Check drawdown limits
        if portfolio_impact.max_drawdown_risk > self.config.max_drawdown_threshold {
            return false;
        }

        // Check correlation limits
        if correlation_risk > self.config.max_correlation_exposure {
            return false;
        }

        // Check liquidity risk
        if liquidity_risk > 0.8 { // High liquidity risk threshold
            return false;
        }

        true
    }

    /// Get historical volatility for an instrument
    async fn get_historical_volatility(&self, instrument_id: Uuid) -> Option<f64> {
        self.historical_returns.get(&instrument_id)
            .and_then(|returns| {
                if returns.len() < 10 {
                    return None;
                }

                let mean = returns.iter().sum::<f64>() / returns.len() as f64;
                let variance = returns.iter()
                    .map(|r| (r - mean).powi(2))
                    .sum::<f64>() / (returns.len() - 1) as f64;

                Some(variance.sqrt())
            })
    }

    /// Update position tracking
    pub fn update_position(&mut self, position: Position) {
        self.position_tracker.insert(position.instrument_id, position);
        self.update_portfolio_metrics();
    }

    /// Remove position from tracking
    pub fn remove_position(&mut self, instrument_id: Uuid) {
        self.position_tracker.remove(&instrument_id);
        self.update_portfolio_metrics();
    }

    /// Update portfolio-level metrics
    fn update_portfolio_metrics(&mut self) {
        let total_long = self.position_tracker.values()
            .filter(|p| p.size > 0.0)
            .map(|p| p.size * p.entry_price)
            .sum::<f64>();

        let total_short = self.position_tracker.values()
            .filter(|p| p.size < 0.0)
            .map(|p| p.size.abs() * p.entry_price)
            .sum::<f64>();

        self.portfolio_metrics.gross_exposure = total_long + total_short;
        self.portfolio_metrics.net_exposure = total_long - total_short;
        self.portfolio_metrics.total_exposure = self.portfolio_metrics.gross_exposure;
        self.portfolio_metrics.last_updated = Utc::now();
    }

    /// Add historical return data for volatility calculations
    pub fn add_return_data(&mut self, instrument_id: Uuid, returns: Vec<f64>) {
        self.historical_returns.insert(instrument_id, returns);
    }

    /// Update correlation matrix
    pub fn update_correlation(&mut self, instrument1: Uuid, instrument2: Uuid, correlation: f64) {
        self.correlation_matrix.insert((instrument1, instrument2), correlation);
    }

    /// Get current portfolio metrics
    pub fn get_portfolio_metrics(&self) -> &PortfolioMetrics {
        &self.portfolio_metrics
    }

    /// Get active positions
    pub fn get_active_positions(&self) -> &HashMap<Uuid, Position> {
        &self.position_tracker
    }
}

// Supporting structures
#[derive(Debug, Clone)]
struct VarImpact {
    var_95: f64,
    expected_shortfall: f64,
    confidence_level: f64,
}

#[derive(Debug, Clone)]
struct PortfolioImpact {
    exposure_increase: f64,
    new_total_exposure: f64,
    exposure_ratio: f64,
    max_drawdown_risk: f64,
}
