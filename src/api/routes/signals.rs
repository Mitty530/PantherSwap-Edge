use axum::{
    extract::{Query, Extension},
    Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::{info, error, warn};

use crate::api::{AppState, responses::*};
use crate::api::middleware::auth::{AuthenticatedUser, Permission, has_permission};
use crate::database::types::SignalType;
use crate::trading::signals::TradingSignal;

/// Query parameters for trading signals
#[derive(Debug, Deserialize)]
pub struct SignalsQuery {
    pub instrument_id: Option<Uuid>,
    pub strategy_name: Option<String>,
    pub signal_type: Option<String>,
    pub min_confidence: Option<f64>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Query parameters for signal performance
#[derive(Debug, Deserialize)]
pub struct SignalPerformanceQuery {
    pub strategy_name: Option<String>,
    pub instrument_id: Option<Uuid>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub signal_type: Option<String>,
}

/// Response for trading signal
#[derive(Debug, Serialize)]
pub struct SignalResponse {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub instrument_id: Uuid,
    pub strategy_name: String,
    pub signal_type: SignalType,
    pub signal_strength: f64,
    pub confidence_score: f64,
    pub recommended_size: f64,
    pub entry_price: Option<f64>,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub time_horizon: Option<i32>,
    pub expected_return: Option<f64>,
    pub risk_score: f64,
    pub microstructure_score: f64,
    pub ai_prediction_score: f64,
    pub regime_score: f64,
}

/// Response for signals list
#[derive(Debug, Serialize)]
pub struct SignalsResponse {
    pub signals: Vec<SignalResponse>,
    pub total_count: u64,
    pub page_info: PageInfo,
}

/// Response for latest signals by instrument
#[derive(Debug, Serialize)]
pub struct LatestSignalsResponse {
    pub signals: Vec<SignalResponse>,
    pub timestamp: DateTime<Utc>,
    pub total_instruments: u32,
}

/// Response for signal performance metrics
#[derive(Debug, Serialize)]
pub struct SignalPerformanceResponse {
    pub strategy_name: String,
    pub total_signals: u32,
    pub successful_signals: u32,
    pub failed_signals: u32,
    pub success_rate: f64,
    pub average_return: f64,
    pub total_return: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub win_rate: f64,
    pub average_win: f64,
    pub average_loss: f64,
    pub profit_factor: f64,
    pub average_confidence: f64,
    pub confidence_accuracy: f64,
}

/// Response for signal analytics
#[derive(Debug, Serialize)]
pub struct SignalAnalyticsResponse {
    pub total_signals_today: u32,
    pub signals_by_type: SignalTypeBreakdown,
    pub signals_by_strategy: Vec<StrategySignalCount>,
    pub average_confidence: f64,
    pub high_confidence_signals: u32,
    pub signals_executed: u32,
    pub execution_rate: f64,
}

#[derive(Debug, Serialize)]
pub struct SignalTypeBreakdown {
    pub buy_signals: u32,
    pub sell_signals: u32,
    pub hold_signals: u32,
}

#[derive(Debug, Serialize)]
pub struct StrategySignalCount {
    pub strategy_name: String,
    pub signal_count: u32,
    pub average_confidence: f64,
}

/// Get trading signals with filtering and pagination
pub async fn get_signals(
    Query(params): Query<SignalsQuery>,
    Extension(_state): Extension<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<ApiResponse<SignalsResponse>>, StatusCode> {
    // Check permissions
    if !has_permission(&user, &Permission::ReadTradingSignals) {
        warn!(user_id = %user.id, "Permission denied for trading signals access");
        return Err(StatusCode::FORBIDDEN);
    }

    info!(
        user_id = %user.id,
        instrument_id = ?params.instrument_id,
        strategy_name = ?params.strategy_name,
        "Trading signals requested"
    );

    let limit = params.limit.unwrap_or(50).min(1000);
    let offset = params.offset.unwrap_or(0);

    // In a real implementation, this would query the database for signals
    // For now, we'll return mock data
    let signals = generate_mock_signals(&params);
    let total_count = signals.len() as u64;

    let response = SignalsResponse {
        signals,
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

/// Get latest trading signals
pub async fn get_latest_signals(
    Extension(_state): Extension<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<ApiResponse<LatestSignalsResponse>>, StatusCode> {
    // Check permissions
    if !has_permission(&user, &Permission::ReadTradingSignals) {
        return Err(StatusCode::FORBIDDEN);
    }

    info!(user_id = %user.id, "Latest trading signals requested");

    // In a real implementation, this would get the latest signal for each instrument
    let signals = generate_latest_mock_signals();

    let response = LatestSignalsResponse {
        signals,
        timestamp: Utc::now(),
        total_instruments: 5, // Our default forex pairs
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Get signal performance metrics
pub async fn get_signal_performance(
    Query(params): Query<SignalPerformanceQuery>,
    Extension(_state): Extension<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<ApiResponse<Vec<SignalPerformanceResponse>>>, StatusCode> {
    // Check permissions
    if !has_permission(&user, &Permission::ReadTradingSignals) {
        return Err(StatusCode::FORBIDDEN);
    }

    info!(
        user_id = %user.id,
        strategy_name = ?params.strategy_name,
        "Signal performance requested"
    );

    // In a real implementation, this would calculate performance from historical data
    let performance_metrics = vec![
        SignalPerformanceResponse {
            strategy_name: "MeanReversion".to_string(),
            total_signals: 125,
            successful_signals: 78,
            failed_signals: 47,
            success_rate: 0.624,
            average_return: 0.025,
            total_return: 3.125,
            sharpe_ratio: 1.45,
            max_drawdown: -0.08,
            win_rate: 0.624,
            average_win: 0.045,
            average_loss: -0.028,
            profit_factor: 1.61,
            average_confidence: 0.72,
            confidence_accuracy: 0.68,
        },
        SignalPerformanceResponse {
            strategy_name: "TrendFollowing".to_string(),
            total_signals: 89,
            successful_signals: 58,
            failed_signals: 31,
            success_rate: 0.652,
            average_return: 0.032,
            total_return: 2.848,
            sharpe_ratio: 1.28,
            max_drawdown: -0.12,
            win_rate: 0.652,
            average_win: 0.055,
            average_loss: -0.035,
            profit_factor: 1.57,
            average_confidence: 0.68,
            confidence_accuracy: 0.71,
        },
    ];

    Ok(Json(ApiResponse::success(performance_metrics)))
}

/// Get signal analytics
pub async fn get_signal_analytics(
    Extension(_state): Extension<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<ApiResponse<SignalAnalyticsResponse>>, StatusCode> {
    // Check permissions
    if !has_permission(&user, &Permission::ReadTradingSignals) {
        return Err(StatusCode::FORBIDDEN);
    }

    info!(user_id = %user.id, "Signal analytics requested");

    let analytics = SignalAnalyticsResponse {
        total_signals_today: 23,
        signals_by_type: SignalTypeBreakdown {
            buy_signals: 12,
            sell_signals: 8,
            hold_signals: 3,
        },
        signals_by_strategy: vec![
            StrategySignalCount {
                strategy_name: "MeanReversion".to_string(),
                signal_count: 14,
                average_confidence: 0.74,
            },
            StrategySignalCount {
                strategy_name: "TrendFollowing".to_string(),
                signal_count: 9,
                average_confidence: 0.69,
            },
        ],
        average_confidence: 0.72,
        high_confidence_signals: 15, // Signals with confidence > 0.8
        signals_executed: 18,
        execution_rate: 0.78, // 78% of signals were executed
    };

    Ok(Json(ApiResponse::success(analytics)))
}

// Helper functions for mock data generation

fn generate_mock_signals(_params: &SignalsQuery) -> Vec<SignalResponse> {
    // Generate mock signals based on query parameters
    vec![
        SignalResponse {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            instrument_id: Uuid::new_v4(),
            strategy_name: "MeanReversion".to_string(),
            signal_type: SignalType::Buy,
            signal_strength: 0.75,
            confidence_score: 0.82,
            recommended_size: 1000.0,
            entry_price: Some(1.0850),
            stop_loss: Some(1.0820),
            take_profit: Some(1.0920),
            time_horizon: Some(240), // 4 hours
            expected_return: Some(0.0065),
            risk_score: 0.25,
            microstructure_score: 0.78,
            ai_prediction_score: 0.85,
            regime_score: 0.72,
        },
        SignalResponse {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            instrument_id: Uuid::new_v4(),
            strategy_name: "TrendFollowing".to_string(),
            signal_type: SignalType::Sell,
            signal_strength: 0.68,
            confidence_score: 0.74,
            recommended_size: 1500.0,
            entry_price: Some(1.2150),
            stop_loss: Some(1.2180),
            take_profit: Some(1.2050),
            time_horizon: Some(360), // 6 hours
            expected_return: Some(0.0082),
            risk_score: 0.32,
            microstructure_score: 0.71,
            ai_prediction_score: 0.76,
            regime_score: 0.69,
        },
    ]
}

fn generate_latest_mock_signals() -> Vec<SignalResponse> {
    // Generate latest signals for each instrument
    vec![
        SignalResponse {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            instrument_id: Uuid::new_v4(),
            strategy_name: "MeanReversion".to_string(),
            signal_type: SignalType::Buy,
            signal_strength: 0.72,
            confidence_score: 0.79,
            recommended_size: 800.0,
            entry_price: Some(1.0865),
            stop_loss: Some(1.0835),
            take_profit: Some(1.0935),
            time_horizon: Some(180),
            expected_return: Some(0.0064),
            risk_score: 0.28,
            microstructure_score: 0.75,
            ai_prediction_score: 0.82,
            regime_score: 0.74,
        },
    ]
}
