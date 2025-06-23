use axum::{
    extract::{Query, Extension},
    Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::{info, warn};

use crate::api::{AppState, responses::*};
use crate::api::middleware::auth::{AuthenticatedUser, Permission, has_permission};
use crate::trading::{PortfolioSummary, PerformanceMetrics};
use crate::trading::signals::Position;

/// Query parameters for portfolio positions
#[derive(Debug, Deserialize)]
pub struct PositionsQuery {
    pub instrument_id: Option<Uuid>,
    pub strategy_name: Option<String>,
    pub min_size: Option<f64>,
    pub include_closed: Option<bool>,
}

/// Query parameters for performance metrics
#[derive(Debug, Deserialize)]
pub struct PerformanceQuery {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub strategy_name: Option<String>,
    pub instrument_id: Option<Uuid>,
}

/// Response for portfolio position
#[derive(Debug, Serialize)]
pub struct PositionResponse {
    pub instrument_id: Uuid,
    pub strategy_name: String,
    pub size: f64,
    pub entry_price: f64,
    pub current_price: f64,
    pub entry_time: DateTime<Utc>,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub duration_hours: f64,
}

/// Response for portfolio positions list
#[derive(Debug, Serialize)]
pub struct PositionsResponse {
    pub positions: Vec<PositionResponse>,
    pub total_count: u64,
    pub summary: PositionsSummary,
}

/// Summary of positions
#[derive(Debug, Serialize)]
pub struct PositionsSummary {
    pub total_positions: u32,
    pub long_positions: u32,
    pub short_positions: u32,
    pub total_unrealized_pnl: f64,
    pub total_realized_pnl: f64,
    pub total_exposure: f64,
}

/// Response for portfolio performance metrics
#[derive(Debug, Serialize)]
pub struct PerformanceResponse {
    pub total_return: f64,
    pub total_return_percentage: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub average_win: f64,
    pub average_loss: f64,
    pub total_trades: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub largest_win: f64,
    pub largest_loss: f64,
    pub average_trade_duration_hours: f64,
    pub volatility: f64,
    pub calmar_ratio: f64,
    pub sortino_ratio: f64,
}

/// Response for portfolio risk metrics
#[derive(Debug, Serialize)]
pub struct RiskMetricsResponse {
    pub var_95: f64,
    pub var_99: f64,
    pub expected_shortfall_95: f64,
    pub expected_shortfall_99: f64,
    pub max_drawdown: f64,
    pub current_drawdown: f64,
    pub beta: f64,
    pub correlation_to_market: f64,
    pub portfolio_volatility: f64,
    pub risk_adjusted_return: f64,
    pub leverage_ratio: f64,
    pub concentration_risk: f64,
}

/// Response for portfolio summary
#[derive(Debug, Serialize)]
pub struct PortfolioSummaryResponse {
    pub total_value: f64,
    pub cash_balance: f64,
    pub invested_amount: f64,
    pub total_pnl: f64,
    pub total_pnl_percentage: f64,
    pub day_pnl: f64,
    pub day_pnl_percentage: f64,
    pub active_positions: u32,
    pub total_trades_today: u32,
    pub last_updated: DateTime<Utc>,
}

/// Get portfolio positions
pub async fn get_positions(
    Query(params): Query<PositionsQuery>,
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<ApiResponse<PositionsResponse>>, StatusCode> {
    // Check permissions
    if !has_permission(&user, &Permission::ReadPortfolio) {
        warn!(user_id = %user.id, "Permission denied for portfolio positions access");
        return Err(StatusCode::FORBIDDEN);
    }

    info!(
        user_id = %user.id,
        instrument_id = ?params.instrument_id,
        strategy_name = ?params.strategy_name,
        "Portfolio positions requested"
    );

    let trading_engine = state.trading_engine.read().await;
    let positions = trading_engine.get_positions().await;

    // Filter positions based on query parameters
    let mut filtered_positions: Vec<Position> = positions.values().cloned().collect();

    if let Some(instrument_id) = params.instrument_id {
        filtered_positions.retain(|pos| pos.instrument_id == instrument_id);
    }

    if let Some(strategy_name) = &params.strategy_name {
        filtered_positions.retain(|pos| pos.strategy_name == *strategy_name);
    }

    if let Some(min_size) = params.min_size {
        filtered_positions.retain(|pos| pos.size.abs() >= min_size);
    }

    // Convert to response format
    let position_responses: Vec<PositionResponse> = filtered_positions
        .into_iter()
        .map(|pos| PositionResponse {
            instrument_id: pos.instrument_id,
            strategy_name: pos.strategy_name.clone(),
            size: pos.size,
            entry_price: pos.entry_price,
            current_price: pos.entry_price, // Would be updated with current market price
            entry_time: pos.entry_time,
            unrealized_pnl: pos.unrealized_pnl,
            realized_pnl: 0.0, // Would come from portfolio manager
            stop_loss: pos.stop_loss,
            take_profit: pos.take_profit,
            duration_hours: (Utc::now() - pos.entry_time).num_hours() as f64,
        })
        .collect();

    // Calculate summary
    let summary = calculate_positions_summary(&position_responses);
    let total_count = position_responses.len() as u64;

    let response = PositionsResponse {
        positions: position_responses,
        total_count,
        summary,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Get portfolio performance metrics
pub async fn get_performance(
    Query(params): Query<PerformanceQuery>,
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<ApiResponse<PerformanceResponse>>, StatusCode> {
    // Check permissions
    if !has_permission(&user, &Permission::ReadPortfolio) {
        return Err(StatusCode::FORBIDDEN);
    }

    info!(
        user_id = %user.id,
        start_date = ?params.start_date,
        end_date = ?params.end_date,
        "Portfolio performance requested"
    );

    let trading_engine = state.trading_engine.read().await;
    let portfolio_summary = trading_engine.get_portfolio_summary().await;

    // In a real implementation, this would calculate performance metrics
    // based on historical data and the query parameters
    let performance = PerformanceResponse {
        total_return: portfolio_summary.total_return,
        total_return_percentage: portfolio_summary.total_return * 100.0,
        sharpe_ratio: portfolio_summary.sharpe_ratio,
        max_drawdown: -0.05, // 5% max drawdown
        win_rate: 0.65, // 65% win rate
        profit_factor: 1.8,
        average_win: 150.0,
        average_loss: -85.0,
        total_trades: 45,
        winning_trades: 29,
        losing_trades: 16,
        largest_win: 500.0,
        largest_loss: -200.0,
        average_trade_duration_hours: 4.5,
        volatility: 0.15, // 15% annualized volatility
        calmar_ratio: 2.4,
        sortino_ratio: 1.8,
    };

    Ok(Json(ApiResponse::success(performance)))
}

/// Get portfolio risk metrics
pub async fn get_risk_metrics(
    Extension(_state): Extension<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<ApiResponse<RiskMetricsResponse>>, StatusCode> {
    // Check permissions
    if !has_permission(&user, &Permission::ReadPortfolio) {
        return Err(StatusCode::FORBIDDEN);
    }

    info!(user_id = %user.id, "Portfolio risk metrics requested");

    // In a real implementation, this would calculate risk metrics
    // from current positions and historical data
    let risk_metrics = RiskMetricsResponse {
        var_95: -1250.0, // 95% VaR
        var_99: -2100.0, // 99% VaR
        expected_shortfall_95: -1800.0,
        expected_shortfall_99: -2800.0,
        max_drawdown: -0.08, // 8% max drawdown
        current_drawdown: -0.02, // 2% current drawdown
        beta: 0.85, // Beta to market
        correlation_to_market: 0.72,
        portfolio_volatility: 0.18, // 18% annualized volatility
        risk_adjusted_return: 0.95, // Risk-adjusted return
        leverage_ratio: 1.2, // 1.2x leverage
        concentration_risk: 0.25, // 25% concentration in top position
    };

    Ok(Json(ApiResponse::success(risk_metrics)))
}

/// Get portfolio summary
pub async fn get_summary(
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<ApiResponse<PortfolioSummaryResponse>>, StatusCode> {
    // Check permissions
    if !has_permission(&user, &Permission::ReadPortfolio) {
        return Err(StatusCode::FORBIDDEN);
    }

    info!(user_id = %user.id, "Portfolio summary requested");

    let trading_engine = state.trading_engine.read().await;
    let portfolio_summary = trading_engine.get_portfolio_summary().await;

    let response = PortfolioSummaryResponse {
        total_value: portfolio_summary.total_value,
        cash_balance: portfolio_summary.cash,
        invested_amount: portfolio_summary.total_value - portfolio_summary.cash,
        total_pnl: portfolio_summary.total_pnl,
        total_pnl_percentage: portfolio_summary.total_return * 100.0,
        day_pnl: portfolio_summary.daily_pnl,
        day_pnl_percentage: (portfolio_summary.daily_pnl / portfolio_summary.total_value) * 100.0,
        active_positions: portfolio_summary.num_positions as u32,
        total_trades_today: portfolio_summary.total_trades, // Using total trades as approximation
        last_updated: Utc::now(),
    };

    Ok(Json(ApiResponse::success(response)))
}

// Helper functions

fn calculate_positions_summary(positions: &[PositionResponse]) -> PositionsSummary {
    let total_positions = positions.len() as u32;
    let long_positions = positions.iter().filter(|p| p.size > 0.0).count() as u32;
    let short_positions = positions.iter().filter(|p| p.size < 0.0).count() as u32;
    let total_unrealized_pnl = positions.iter().map(|p| p.unrealized_pnl).sum();
    let total_realized_pnl = positions.iter().map(|p| p.realized_pnl).sum();
    let total_exposure = positions.iter().map(|p| p.size.abs() * p.current_price).sum();

    PositionsSummary {
        total_positions,
        long_positions,
        short_positions,
        total_unrealized_pnl,
        total_realized_pnl,
        total_exposure,
    }
}
