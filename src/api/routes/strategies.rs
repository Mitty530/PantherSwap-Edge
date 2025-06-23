use axum::{
    extract::{Path, Query, Extension},
    Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::{info, error, warn};

use crate::api::{AppState, responses::*};
use crate::api::middleware::auth::{AuthenticatedUser, Permission, has_permission};

/// Query parameters for strategies list
#[derive(Debug, Deserialize)]
pub struct StrategiesQuery {
    pub strategy_type: Option<String>,
    pub is_active: Option<bool>,
    pub min_performance: Option<f64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Query parameters for strategy performance
#[derive(Debug, Deserialize)]
pub struct StrategyPerformanceQuery {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub instrument_id: Option<Uuid>,
    pub include_benchmark: Option<bool>,
}

/// Request to update strategy configuration
#[derive(Debug, Deserialize)]
pub struct UpdateStrategyRequest {
    pub is_active: Option<bool>,
    pub confidence_threshold: Option<f64>,
    pub max_position_size: Option<f64>,
    pub risk_multiplier: Option<f64>,
    pub parameters: Option<serde_json::Value>,
}

/// Response for strategy information
#[derive(Debug, Serialize)]
pub struct StrategyResponse {
    pub id: Uuid,
    pub name: String,
    pub strategy_type: String,
    pub description: String,
    pub is_active: bool,
    pub confidence_threshold: f64,
    pub max_position_size: f64,
    pub risk_multiplier: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub parameters: serde_json::Value,
    pub performance_summary: StrategyPerformanceSummary,
}

/// Summary of strategy performance
#[derive(Debug, Serialize)]
pub struct StrategyPerformanceSummary {
    pub total_return: f64,
    pub total_return_percentage: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub win_rate: f64,
    pub total_trades: u32,
    pub active_positions: u32,
    pub last_signal_time: Option<DateTime<Utc>>,
}

/// Response for strategies list
#[derive(Debug, Serialize)]
pub struct StrategiesResponse {
    pub strategies: Vec<StrategyResponse>,
    pub total_count: u64,
    pub page_info: PageInfo,
}

/// Detailed strategy performance response
#[derive(Debug, Serialize)]
pub struct StrategyPerformanceResponse {
    pub strategy_name: String,
    pub time_period: TimePeriod,
    pub performance_metrics: PerformanceMetrics,
    pub risk_metrics: RiskMetrics,
    pub trade_statistics: TradeStatistics,
    pub monthly_returns: Vec<MonthlyReturn>,
    pub drawdown_periods: Vec<DrawdownPeriod>,
    pub benchmark_comparison: Option<BenchmarkComparison>,
}

/// Time period for performance analysis
#[derive(Debug, Serialize)]
pub struct TimePeriod {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub total_days: u32,
    pub trading_days: u32,
}

/// Performance metrics
#[derive(Debug, Serialize)]
pub struct PerformanceMetrics {
    pub total_return: f64,
    pub annualized_return: f64,
    pub volatility: f64,
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub calmar_ratio: f64,
    pub max_drawdown: f64,
    pub var_95: f64,
    pub expected_shortfall: f64,
}

/// Risk metrics
#[derive(Debug, Serialize)]
pub struct RiskMetrics {
    pub beta: f64,
    pub alpha: f64,
    pub correlation: f64,
    pub tracking_error: f64,
    pub information_ratio: f64,
    pub downside_deviation: f64,
    pub upside_capture: f64,
    pub downside_capture: f64,
}

/// Trade statistics
#[derive(Debug, Serialize)]
pub struct TradeStatistics {
    pub total_trades: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub win_rate: f64,
    pub average_win: f64,
    pub average_loss: f64,
    pub largest_win: f64,
    pub largest_loss: f64,
    pub profit_factor: f64,
    pub average_trade_duration_hours: f64,
    pub trades_per_day: f64,
}

/// Monthly return data
#[derive(Debug, Serialize)]
pub struct MonthlyReturn {
    pub year: i32,
    pub month: u32,
    pub return_percentage: f64,
    pub trades_count: u32,
    pub max_drawdown: f64,
}

/// Drawdown period information
#[derive(Debug, Serialize)]
pub struct DrawdownPeriod {
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub peak_value: f64,
    pub trough_value: f64,
    pub drawdown_percentage: f64,
    pub recovery_days: Option<u32>,
}

/// Benchmark comparison
#[derive(Debug, Serialize)]
pub struct BenchmarkComparison {
    pub benchmark_name: String,
    pub strategy_return: f64,
    pub benchmark_return: f64,
    pub excess_return: f64,
    pub tracking_error: f64,
    pub information_ratio: f64,
    pub beta: f64,
    pub alpha: f64,
}

/// List all trading strategies
pub async fn list_strategies(
    Query(params): Query<StrategiesQuery>,
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<ApiResponse<StrategiesResponse>>, StatusCode> {
    // Check permissions
    if !has_permission(&user, &Permission::ReadStrategies) {
        warn!(user_id = %user.id, "Permission denied for strategies access");
        return Err(StatusCode::FORBIDDEN);
    }

    info!(
        user_id = %user.id,
        strategy_type = ?params.strategy_type,
        is_active = ?params.is_active,
        "Strategies list requested"
    );

    let limit = params.limit.unwrap_or(50).min(1000);
    let offset = params.offset.unwrap_or(0);

    // Generate mock strategies data
    let strategies = generate_mock_strategies(&params);
    let total_count = strategies.len() as u64;

    let response = StrategiesResponse {
        strategies,
        total_count,
        page_info: PageInfo {
            has_next_page: total_count > (offset + limit) as u64,
            has_previous_page: offset > 0,
            total_count: Some(total_count as i64),
            page: Some((offset / limit) as i64 + 1),
            per_page: Some(limit),
        },
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Get specific strategy details
pub async fn get_strategy(
    Path(strategy_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<ApiResponse<StrategyResponse>>, StatusCode> {
    // Check permissions
    if !has_permission(&user, &Permission::ReadStrategies) {
        return Err(StatusCode::FORBIDDEN);
    }

    info!(
        user_id = %user.id,
        strategy_id = %strategy_id,
        "Strategy details requested"
    );

    // Generate mock strategy data
    let strategy = generate_mock_strategy(strategy_id);

    Ok(Json(ApiResponse::success(strategy)))
}

/// Get strategy performance metrics
pub async fn get_strategy_performance(
    Path(strategy_id): Path<Uuid>,
    Query(params): Query<StrategyPerformanceQuery>,
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<ApiResponse<StrategyPerformanceResponse>>, StatusCode> {
    // Check permissions
    if !has_permission(&user, &Permission::ReadStrategies) {
        return Err(StatusCode::FORBIDDEN);
    }

    info!(
        user_id = %user.id,
        strategy_id = %strategy_id,
        "Strategy performance requested"
    );

    // Generate mock performance data
    let performance = generate_mock_performance(strategy_id, &params);

    Ok(Json(ApiResponse::success(performance)))
}

/// Update strategy configuration
pub async fn update_strategy(
    Path(strategy_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(request): Json<UpdateStrategyRequest>,
) -> Result<Json<ApiResponse<StrategyResponse>>, StatusCode> {
    // Check permissions
    if !has_permission(&user, &Permission::WriteStrategies) {
        warn!(user_id = %user.id, "Permission denied for strategy update");
        return Err(StatusCode::FORBIDDEN);
    }

    info!(
        user_id = %user.id,
        strategy_id = %strategy_id,
        "Strategy update requested"
    );

    // In a real implementation, this would update the strategy in the database
    // and potentially restart the strategy if needed

    let updated_strategy = generate_mock_strategy(strategy_id);

    Ok(Json(ApiResponse::success(updated_strategy)))
}

// Helper functions for mock data generation

fn generate_mock_strategies(params: &StrategiesQuery) -> Vec<StrategyResponse> {
    vec![
        StrategyResponse {
            id: Uuid::new_v4(),
            name: "MeanReversion".to_string(),
            strategy_type: "Statistical Arbitrage".to_string(),
            description: "Mean reversion strategy using statistical analysis".to_string(),
            is_active: true,
            confidence_threshold: 0.75,
            max_position_size: 10000.0,
            risk_multiplier: 1.0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            parameters: serde_json::json!({
                "lookback_period": 20,
                "z_score_threshold": 2.0,
                "stop_loss_multiplier": 1.5
            }),
            performance_summary: StrategyPerformanceSummary {
                total_return: 0.125,
                total_return_percentage: 12.5,
                sharpe_ratio: 1.45,
                max_drawdown: -0.08,
                win_rate: 0.624,
                total_trades: 125,
                active_positions: 3,
                last_signal_time: Some(Utc::now()),
            },
        },
        StrategyResponse {
            id: Uuid::new_v4(),
            name: "TrendFollowing".to_string(),
            strategy_type: "Momentum".to_string(),
            description: "Trend following strategy using momentum indicators".to_string(),
            is_active: true,
            confidence_threshold: 0.70,
            max_position_size: 15000.0,
            risk_multiplier: 1.2,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            parameters: serde_json::json!({
                "fast_ma_period": 10,
                "slow_ma_period": 30,
                "momentum_threshold": 0.02
            }),
            performance_summary: StrategyPerformanceSummary {
                total_return: 0.089,
                total_return_percentage: 8.9,
                sharpe_ratio: 1.28,
                max_drawdown: -0.12,
                win_rate: 0.652,
                total_trades: 89,
                active_positions: 2,
                last_signal_time: Some(Utc::now()),
            },
        },
    ]
}

fn generate_mock_strategy(strategy_id: Uuid) -> StrategyResponse {
    StrategyResponse {
        id: strategy_id,
        name: "MeanReversion".to_string(),
        strategy_type: "Statistical Arbitrage".to_string(),
        description: "Mean reversion strategy using statistical analysis".to_string(),
        is_active: true,
        confidence_threshold: 0.75,
        max_position_size: 10000.0,
        risk_multiplier: 1.0,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        parameters: serde_json::json!({
            "lookback_period": 20,
            "z_score_threshold": 2.0,
            "stop_loss_multiplier": 1.5
        }),
        performance_summary: StrategyPerformanceSummary {
            total_return: 0.125,
            total_return_percentage: 12.5,
            sharpe_ratio: 1.45,
            max_drawdown: -0.08,
            win_rate: 0.624,
            total_trades: 125,
            active_positions: 3,
            last_signal_time: Some(Utc::now()),
        },
    }
}

fn generate_mock_performance(strategy_id: Uuid, params: &StrategyPerformanceQuery) -> StrategyPerformanceResponse {
    StrategyPerformanceResponse {
        strategy_name: "MeanReversion".to_string(),
        time_period: TimePeriod {
            start_date: Utc::now() - chrono::Duration::days(90),
            end_date: Utc::now(),
            total_days: 90,
            trading_days: 65,
        },
        performance_metrics: PerformanceMetrics {
            total_return: 0.125,
            annualized_return: 0.52,
            volatility: 0.18,
            sharpe_ratio: 1.45,
            sortino_ratio: 1.82,
            calmar_ratio: 2.15,
            max_drawdown: -0.08,
            var_95: -0.025,
            expected_shortfall: -0.035,
        },
        risk_metrics: RiskMetrics {
            beta: 0.85,
            alpha: 0.08,
            correlation: 0.72,
            tracking_error: 0.12,
            information_ratio: 0.67,
            downside_deviation: 0.14,
            upside_capture: 1.15,
            downside_capture: 0.82,
        },
        trade_statistics: TradeStatistics {
            total_trades: 125,
            winning_trades: 78,
            losing_trades: 47,
            win_rate: 0.624,
            average_win: 0.045,
            average_loss: -0.028,
            largest_win: 0.125,
            largest_loss: -0.085,
            profit_factor: 1.61,
            average_trade_duration_hours: 4.5,
            trades_per_day: 1.92,
        },
        monthly_returns: vec![
            MonthlyReturn {
                year: 2024,
                month: 12,
                return_percentage: 0.042,
                trades_count: 18,
                max_drawdown: -0.025,
            },
        ],
        drawdown_periods: vec![
            DrawdownPeriod {
                start_date: Utc::now() - chrono::Duration::days(15),
                end_date: Some(Utc::now() - chrono::Duration::days(8)),
                peak_value: 10500.0,
                trough_value: 9800.0,
                drawdown_percentage: -0.067,
                recovery_days: Some(7),
            },
        ],
        benchmark_comparison: Some(BenchmarkComparison {
            benchmark_name: "Market Index".to_string(),
            strategy_return: 0.125,
            benchmark_return: 0.085,
            excess_return: 0.040,
            tracking_error: 0.12,
            information_ratio: 0.33,
            beta: 0.85,
            alpha: 0.08,
        }),
    }
}
