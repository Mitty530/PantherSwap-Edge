use crate::database::Database;
use crate::trading::strategies::StrategyPerformance;
use crate::trading::strategy_optimization::{
    StrategyWeights, PortfolioOptimizationMetrics
};
use crate::trading::signals::StrategyType;
use crate::database::types::RegimeType;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, NaiveDate};
use std::collections::HashMap;
use anyhow::Result;
use uuid::Uuid;
use sqlx::Row;
use tracing::info;

/// Database interface for strategy analytics and optimization
pub struct StrategyAnalyticsDB {
    database: std::sync::Arc<Database>,
}

impl StrategyAnalyticsDB {
    pub fn new(database: std::sync::Arc<Database>) -> Self {
        Self { database }
    }

    /// Store strategy performance metrics
    pub async fn store_strategy_performance(
        &self,
        strategy_type: StrategyType,
        performance: &StrategyPerformance,
    ) -> Result<Uuid> {
        let strategy_type_str = format!("{:?}", strategy_type);
        
        let query = r#"
            INSERT INTO strategy_performance_metrics (
                strategy_type, total_trades, winning_trades, total_pnl, sharpe_ratio,
                max_drawdown, avg_holding_period_seconds, success_rate, avg_return_per_trade,
                sortino_ratio, calmar_ratio, information_ratio, var_95, expected_shortfall,
                profit_factor, recovery_factor, tail_ratio, skewness, kurtosis,
                rolling_sharpe_30d, rolling_volatility_30d, max_consecutive_losses,
                avg_win_loss_ratio, kelly_fraction, correlation_to_market, beta, alpha,
                tracking_error, upside_capture, downside_capture
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19,
                $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30
            ) RETURNING id
        "#;

        let row = sqlx::query(query)
            .bind(&strategy_type_str)
            .bind(performance.total_trades as i64)
            .bind(performance.winning_trades as i64)
            .bind(performance.total_pnl)
            .bind(performance.sharpe_ratio)
            .bind(performance.max_drawdown)
            .bind(performance.avg_holding_period.as_secs() as i64)
            .bind(performance.success_rate)
            .bind(performance.avg_return_per_trade)
            .bind(performance.sortino_ratio)
            .bind(performance.calmar_ratio)
            .bind(performance.information_ratio)
            .bind(performance.var_95)
            .bind(performance.expected_shortfall)
            .bind(performance.profit_factor)
            .bind(performance.recovery_factor)
            .bind(performance.tail_ratio)
            .bind(performance.skewness)
            .bind(performance.kurtosis)
            .bind(performance.rolling_sharpe_30d)
            .bind(performance.rolling_volatility_30d)
            .bind(performance.max_consecutive_losses as i32)
            .bind(performance.avg_win_loss_ratio)
            .bind(performance.kelly_fraction)
            .bind(performance.correlation_to_market)
            .bind(performance.beta)
            .bind(performance.alpha)
            .bind(performance.tracking_error)
            .bind(performance.upside_capture)
            .bind(performance.downside_capture)
            .fetch_one(&self.database.pool)
            .await?;

        let id: Uuid = row.get("id");
        info!("Stored strategy performance for {:?} with ID: {}", strategy_type, id);
        Ok(id)
    }

    /// Store strategy weight allocation
    pub async fn store_strategy_weights(
        &self,
        weights: &StrategyWeights,
        optimization_method: &str,
        target_sharpe: Option<f64>,
        achieved_sharpe: Option<f64>,
        portfolio_volatility: Option<f64>,
        diversification_ratio: Option<f64>,
    ) -> Result<Uuid> {
        let query = r#"
            INSERT INTO strategy_weight_allocations (
                predictive_market_making, microstructure_momentum, regime_arbitrage,
                liquidity_harvesting, rebalance_reason, optimization_method,
                target_sharpe_ratio, achieved_sharpe_ratio, portfolio_volatility,
                diversification_ratio
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id
        "#;

        let row = sqlx::query(query)
            .bind(weights.predictive_market_making)
            .bind(weights.microstructure_momentum)
            .bind(weights.regime_arbitrage)
            .bind(weights.liquidity_harvesting)
            .bind(&weights.rebalance_reason)
            .bind(optimization_method)
            .bind(target_sharpe)
            .bind(achieved_sharpe)
            .bind(portfolio_volatility)
            .bind(diversification_ratio)
            .fetch_one(&self.database.pool)
            .await?;

        let id: Uuid = row.get("id");
        info!("Stored strategy weights with ID: {}", id);
        Ok(id)
    }

    /// Get latest strategy performance for all strategies
    pub async fn get_latest_strategy_performance(&self) -> Result<HashMap<StrategyType, StrategyPerformance>> {
        let query = r#"
            SELECT DISTINCT ON (strategy_type) 
                strategy_type, total_trades, winning_trades, total_pnl, sharpe_ratio,
                max_drawdown, avg_holding_period_seconds, success_rate, avg_return_per_trade,
                sortino_ratio, calmar_ratio, information_ratio, var_95, expected_shortfall,
                profit_factor, recovery_factor, tail_ratio, skewness, kurtosis,
                rolling_sharpe_30d, rolling_volatility_30d, max_consecutive_losses,
                avg_win_loss_ratio, kelly_fraction, correlation_to_market, beta, alpha,
                tracking_error, upside_capture, downside_capture, timestamp
            FROM strategy_performance_metrics
            ORDER BY strategy_type, timestamp DESC
        "#;

        let rows = sqlx::query(query)
            .fetch_all(&self.database.pool)
            .await?;

        let mut performance_map = HashMap::new();

        for row in rows {
            let strategy_type_str: String = row.get("strategy_type");
            let strategy_type = match strategy_type_str.as_str() {
                "PredictiveMarketMaking" => StrategyType::PredictiveMarketMaking,
                "MicrostructureMomentum" => StrategyType::MicrostructureMomentum,
                "RegimeArbitrage" => StrategyType::RegimeArbitrage,
                "LiquidityHarvesting" => StrategyType::LiquidityHarvesting,
                _ => continue,
            };

            let performance = StrategyPerformance {
                total_trades: row.get::<i64, _>("total_trades") as u64,
                winning_trades: row.get::<i64, _>("winning_trades") as u64,
                total_pnl: row.get("total_pnl"),
                sharpe_ratio: row.get("sharpe_ratio"),
                max_drawdown: row.get("max_drawdown"),
                avg_holding_period: std::time::Duration::from_secs(row.get::<i64, _>("avg_holding_period_seconds") as u64),
                success_rate: row.get("success_rate"),
                avg_return_per_trade: row.get("avg_return_per_trade"),
                sortino_ratio: row.get("sortino_ratio"),
                calmar_ratio: row.get("calmar_ratio"),
                information_ratio: row.get("information_ratio"),
                var_95: row.get("var_95"),
                expected_shortfall: row.get("expected_shortfall"),
                profit_factor: row.get("profit_factor"),
                recovery_factor: row.get("recovery_factor"),
                tail_ratio: row.get("tail_ratio"),
                skewness: row.get("skewness"),
                kurtosis: row.get("kurtosis"),
                daily_returns: Vec::new(), // Will be populated separately if needed
                rolling_sharpe_30d: row.get("rolling_sharpe_30d"),
                rolling_volatility_30d: row.get("rolling_volatility_30d"),
                max_consecutive_losses: row.get::<i32, _>("max_consecutive_losses") as u32,
                avg_win_loss_ratio: row.get("avg_win_loss_ratio"),
                kelly_fraction: row.get("kelly_fraction"),
                correlation_to_market: row.get("correlation_to_market"),
                beta: row.get("beta"),
                alpha: row.get("alpha"),
                tracking_error: row.get("tracking_error"),
                upside_capture: row.get("upside_capture"),
                downside_capture: row.get("downside_capture"),
            };

            performance_map.insert(strategy_type, performance);
        }

        Ok(performance_map)
    }

    /// Get latest strategy weights
    pub async fn get_latest_strategy_weights(&self) -> Result<Option<StrategyWeights>> {
        let query = r#"
            SELECT predictive_market_making, microstructure_momentum, regime_arbitrage,
                   liquidity_harvesting, rebalance_reason, timestamp
            FROM strategy_weight_allocations
            ORDER BY timestamp DESC
            LIMIT 1
        "#;

        let row = match sqlx::query(query)
            .fetch_optional(&self.database.pool)
            .await? {
            Some(row) => row,
            None => return Ok(None),
        };

        let weights = StrategyWeights {
            predictive_market_making: row.get("predictive_market_making"),
            microstructure_momentum: row.get("microstructure_momentum"),
            regime_arbitrage: row.get("regime_arbitrage"),
            liquidity_harvesting: row.get("liquidity_harvesting"),
            last_updated: row.get("timestamp"),
            rebalance_reason: row.get("rebalance_reason"),
        };

        Ok(Some(weights))
    }

    /// Store strategy daily returns
    pub async fn store_daily_returns(
        &self,
        strategy_type: StrategyType,
        date: NaiveDate,
        daily_return: f64,
        daily_volatility: f64,
        daily_trades: i32,
        daily_pnl: f64,
        max_intraday_drawdown: f64,
        max_intraday_gain: f64,
        market_regime: Option<RegimeType>,
        market_volatility: Option<f64>,
    ) -> Result<Uuid> {
        let strategy_type_str = format!("{:?}", strategy_type);
        let market_regime_str = market_regime.map(|r| format!("{:?}", r));

        let query = r#"
            INSERT INTO strategy_daily_returns (
                strategy_type, date, daily_return, daily_volatility, daily_trades,
                daily_pnl, max_intraday_drawdown, max_intraday_gain,
                market_regime, market_volatility
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (strategy_type, date) 
            DO UPDATE SET
                daily_return = EXCLUDED.daily_return,
                daily_volatility = EXCLUDED.daily_volatility,
                daily_trades = EXCLUDED.daily_trades,
                daily_pnl = EXCLUDED.daily_pnl,
                max_intraday_drawdown = EXCLUDED.max_intraday_drawdown,
                max_intraday_gain = EXCLUDED.max_intraday_gain,
                market_regime = EXCLUDED.market_regime,
                market_volatility = EXCLUDED.market_volatility
            RETURNING id
        "#;

        let row = sqlx::query(query)
            .bind(&strategy_type_str)
            .bind(date)
            .bind(daily_return)
            .bind(daily_volatility)
            .bind(daily_trades)
            .bind(daily_pnl)
            .bind(max_intraday_drawdown)
            .bind(max_intraday_gain)
            .bind(market_regime_str)
            .bind(market_volatility)
            .fetch_one(&self.database.pool)
            .await?;

        let id: Uuid = row.get("id");
        Ok(id)
    }

    /// Get strategy correlation matrix
    pub async fn get_strategy_correlation_matrix(&self, days_back: i32) -> Result<HashMap<(StrategyType, StrategyType), f64>> {
        let query = r#"
            SELECT strategy_1, strategy_2, correlation_coefficient
            FROM strategy_correlation_matrix
            WHERE timestamp >= NOW() - INTERVAL '%d days'
            ORDER BY timestamp DESC
        "#;

        let formatted_query = query.replace("%d", &days_back.to_string());
        
        let rows = sqlx::query(&formatted_query)
            .fetch_all(&self.database.pool)
            .await?;

        let mut correlation_matrix = HashMap::new();

        for row in rows {
            let strategy_1_str: String = row.get("strategy_1");
            let strategy_2_str: String = row.get("strategy_2");
            let correlation: f64 = row.get("correlation_coefficient");

            if let (Ok(s1), Ok(s2)) = (
                strategy_1_str.parse::<StrategyType>(),
                strategy_2_str.parse::<StrategyType>()
            ) {
                correlation_matrix.insert((s1, s2), correlation);
            }
        }

        Ok(correlation_matrix)
    }

    /// Store optimization history
    pub async fn store_optimization_history(
        &self,
        optimization_method: &str,
        target_function: &str,
        weights_before: &StrategyWeights,
        weights_after: &StrategyWeights,
        portfolio_metrics_before: &PortfolioOptimizationMetrics,
        portfolio_metrics_after: &PortfolioOptimizationMetrics,
        improvement_score: f64,
        convergence_iterations: i32,
        optimization_duration_ms: i32,
    ) -> Result<Uuid> {
        let weights_before_json = serde_json::to_value(weights_before)?;
        let weights_after_json = serde_json::to_value(weights_after)?;
        let metrics_before_json = serde_json::to_value(portfolio_metrics_before)?;
        let metrics_after_json = serde_json::to_value(portfolio_metrics_after)?;

        let query = r#"
            INSERT INTO strategy_optimization_history (
                optimization_method, target_function, weights_before, weights_after,
                portfolio_metrics_before, portfolio_metrics_after, improvement_score,
                convergence_iterations, optimization_duration_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id
        "#;

        let row = sqlx::query(query)
            .bind(optimization_method)
            .bind(target_function)
            .bind(weights_before_json)
            .bind(weights_after_json)
            .bind(metrics_before_json)
            .bind(metrics_after_json)
            .bind(improvement_score)
            .bind(convergence_iterations)
            .bind(optimization_duration_ms)
            .fetch_one(&self.database.pool)
            .await?;

        let id: Uuid = row.get("id");
        info!("Stored optimization history with ID: {}", id);
        Ok(id)
    }
}

// Helper trait for parsing strategy types from strings
impl std::str::FromStr for StrategyType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PredictiveMarketMaking" => Ok(StrategyType::PredictiveMarketMaking),
            "MicrostructureMomentum" => Ok(StrategyType::MicrostructureMomentum),
            "RegimeArbitrage" => Ok(StrategyType::RegimeArbitrage),
            "LiquidityHarvesting" => Ok(StrategyType::LiquidityHarvesting),
            _ => Err(anyhow::anyhow!("Unknown strategy type: {}", s)),
        }
    }
}
