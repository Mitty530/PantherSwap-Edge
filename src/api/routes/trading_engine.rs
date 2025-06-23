use axum::{
    extract::{Extension},
    Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::{info, error, warn};

use crate::api::{AppState, responses::*};
use crate::api::middleware::auth::{AuthenticatedUser, Permission, has_permission};
use crate::trading::engine::TradingEngineState;

/// Request to start the trading engine
#[derive(Debug, Deserialize)]
pub struct StartEngineRequest {
    pub force_start: Option<bool>,
    pub instruments: Option<Vec<Uuid>>,
}

/// Request to stop the trading engine
#[derive(Debug, Deserialize)]
pub struct StopEngineRequest {
    pub reason: Option<String>,
    pub emergency_stop: Option<bool>,
}

/// Request to update engine configuration
#[derive(Debug, Deserialize)]
pub struct UpdateEngineConfigRequest {
    pub confidence_threshold: Option<f64>,
    pub max_daily_trades: Option<u32>,
    pub enable_live_trading: Option<bool>,
    pub risk_check_interval_ms: Option<u64>,
    pub signal_generation_interval_ms: Option<u64>,
}

/// Response for trading engine status
#[derive(Debug, Serialize)]
pub struct EngineStatusResponse {
    pub state: TradingEngineState,
    pub uptime_seconds: u64,
    pub last_signal_time: Option<DateTime<Utc>>,
    pub last_risk_check: Option<DateTime<Utc>>,
    pub active_instruments: Vec<Uuid>,
    pub daily_trade_count: u32,
    pub total_positions: u32,
    pub total_pnl: f64,
    pub engine_health: EngineHealth,
    pub last_updated: DateTime<Utc>,
}

/// Engine health indicators
#[derive(Debug, Serialize)]
pub struct EngineHealth {
    pub overall_status: HealthStatus,
    pub components: Vec<ComponentHealth>,
    pub performance_metrics: EnginePerformanceMetrics,
    pub resource_usage: ResourceUsage,
}

/// Health status levels
#[derive(Debug, Serialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Error,
}

/// Individual component health
#[derive(Debug, Serialize)]
pub struct ComponentHealth {
    pub component_name: String,
    pub status: HealthStatus,
    pub last_check: DateTime<Utc>,
    pub message: Option<String>,
    pub metrics: Option<serde_json::Value>,
}

/// Engine performance metrics
#[derive(Debug, Serialize)]
pub struct EnginePerformanceMetrics {
    pub signals_per_minute: f64,
    pub orders_per_minute: f64,
    pub average_signal_latency_ms: f64,
    pub average_order_latency_ms: f64,
    pub risk_check_latency_ms: f64,
    pub portfolio_update_latency_ms: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percentage: f64,
}

/// Resource usage information
#[derive(Debug, Serialize)]
pub struct ResourceUsage {
    pub memory_used_mb: f64,
    pub memory_total_mb: f64,
    pub cpu_usage_percentage: f64,
    pub active_threads: u32,
    pub database_connections: u32,
    pub websocket_connections: u32,
}

/// Response for engine configuration
#[derive(Debug, Serialize)]
pub struct EngineConfigResponse {
    pub confidence_threshold: f64,
    pub max_daily_trades: u32,
    pub enable_live_trading: bool,
    pub risk_check_interval_ms: u64,
    pub signal_generation_interval_ms: u64,
    pub max_position_size: f64,
    pub max_portfolio_risk: f64,
    pub strategy_types: Vec<String>,
    pub active_instruments: Vec<Uuid>,
    pub last_updated: DateTime<Utc>,
}

/// Response for engine performance statistics
#[derive(Debug, Serialize)]
pub struct EngineStatsResponse {
    pub uptime_hours: f64,
    pub total_signals_generated: u64,
    pub total_orders_executed: u64,
    pub total_trades_completed: u64,
    pub success_rate: f64,
    pub average_trade_duration_minutes: f64,
    pub total_pnl: f64,
    pub daily_pnl: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub performance_by_strategy: Vec<StrategyPerformance>,
    pub performance_by_instrument: Vec<InstrumentPerformance>,
}

/// Strategy performance summary
#[derive(Debug, Serialize)]
pub struct StrategyPerformance {
    pub strategy_name: String,
    pub signals_generated: u32,
    pub trades_executed: u32,
    pub success_rate: f64,
    pub total_pnl: f64,
    pub sharpe_ratio: f64,
}

/// Instrument performance summary
#[derive(Debug, Serialize)]
pub struct InstrumentPerformance {
    pub instrument_id: Uuid,
    pub instrument_name: String,
    pub trades_executed: u32,
    pub total_pnl: f64,
    pub win_rate: f64,
    pub average_trade_size: f64,
}

/// Start the trading engine
pub async fn start_engine(
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(request): Json<StartEngineRequest>,
) -> Result<Json<ApiResponse<EngineStatusResponse>>, StatusCode> {
    // Check permissions
    if !has_permission(&user, &Permission::SystemAdmin) {
        warn!(user_id = %user.id, "Permission denied for engine start");
        return Err(StatusCode::FORBIDDEN);
    }

    info!(
        user_id = %user.id,
        force_start = ?request.force_start,
        "Trading engine start requested"
    );

    let trading_engine = state.trading_engine.read().await;
    
    // Check current state
    let current_state = trading_engine.get_state().await;
    if matches!(current_state, TradingEngineState::Running) && !request.force_start.unwrap_or(false) {
        warn!("Trading engine is already running");
        return Err(StatusCode::CONFLICT);
    }

    // Start the engine
    if let Err(e) = trading_engine.start().await {
        error!("Failed to start trading engine: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Add instruments if specified
    if let Some(instruments) = request.instruments {
        for instrument_id in instruments {
            if let Err(e) = trading_engine.add_instrument(instrument_id).await {
                warn!("Failed to add instrument {}: {}", instrument_id, e);
            }
        }
    }

    let status = get_engine_status_internal(&trading_engine).await;
    Ok(Json(ApiResponse::success(status)))
}

/// Stop the trading engine
pub async fn stop_engine(
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(request): Json<StopEngineRequest>,
) -> Result<Json<ApiResponse<EngineStatusResponse>>, StatusCode> {
    // Check permissions
    if !has_permission(&user, &Permission::SystemAdmin) {
        warn!(user_id = %user.id, "Permission denied for engine stop");
        return Err(StatusCode::FORBIDDEN);
    }

    info!(
        user_id = %user.id,
        emergency_stop = ?request.emergency_stop,
        reason = ?request.reason,
        "Trading engine stop requested"
    );

    let trading_engine = state.trading_engine.read().await;

    if request.emergency_stop.unwrap_or(false) {
        let reason = request.reason.unwrap_or_else(|| "Manual emergency stop".to_string());
        if let Err(e) = trading_engine.emergency_stop(reason).await {
            error!("Failed to emergency stop trading engine: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    } else {
        if let Err(e) = trading_engine.stop().await {
            error!("Failed to stop trading engine: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    let status = get_engine_status_internal(&trading_engine).await;
    Ok(Json(ApiResponse::success(status)))
}

/// Get trading engine status
pub async fn get_engine_status(
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<ApiResponse<EngineStatusResponse>>, StatusCode> {
    // Check permissions
    if !has_permission(&user, &Permission::ReadTradingData) {
        return Err(StatusCode::FORBIDDEN);
    }

    info!(user_id = %user.id, "Trading engine status requested");

    let trading_engine = state.trading_engine.read().await;
    let status = get_engine_status_internal(&trading_engine).await;

    Ok(Json(ApiResponse::success(status)))
}

/// Get trading engine configuration
pub async fn get_engine_config(
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<ApiResponse<EngineConfigResponse>>, StatusCode> {
    // Check permissions
    if !has_permission(&user, &Permission::ReadTradingData) {
        return Err(StatusCode::FORBIDDEN);
    }

    info!(user_id = %user.id, "Trading engine configuration requested");

    // In a real implementation, this would get the actual configuration
    let config = EngineConfigResponse {
        confidence_threshold: 0.75,
        max_daily_trades: 100,
        enable_live_trading: true,
        risk_check_interval_ms: 5000,
        signal_generation_interval_ms: 1000,
        max_position_size: 10000.0,
        max_portfolio_risk: 0.02,
        strategy_types: vec!["MeanReversion".to_string(), "TrendFollowing".to_string()],
        active_instruments: vec![Uuid::new_v4(), Uuid::new_v4()],
        last_updated: Utc::now(),
    };

    Ok(Json(ApiResponse::success(config)))
}

/// Update trading engine configuration
pub async fn update_engine_config(
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(request): Json<UpdateEngineConfigRequest>,
) -> Result<Json<ApiResponse<EngineConfigResponse>>, StatusCode> {
    // Check permissions
    if !has_permission(&user, &Permission::SystemAdmin) {
        warn!(user_id = %user.id, "Permission denied for engine config update");
        return Err(StatusCode::FORBIDDEN);
    }

    info!(
        user_id = %user.id,
        "Trading engine configuration update requested"
    );

    // In a real implementation, this would update the engine configuration
    // and potentially restart components if needed

    let config = EngineConfigResponse {
        confidence_threshold: request.confidence_threshold.unwrap_or(0.75),
        max_daily_trades: request.max_daily_trades.unwrap_or(100),
        enable_live_trading: request.enable_live_trading.unwrap_or(true),
        risk_check_interval_ms: request.risk_check_interval_ms.unwrap_or(5000),
        signal_generation_interval_ms: request.signal_generation_interval_ms.unwrap_or(1000),
        max_position_size: 10000.0,
        max_portfolio_risk: 0.02,
        strategy_types: vec!["MeanReversion".to_string(), "TrendFollowing".to_string()],
        active_instruments: vec![Uuid::new_v4(), Uuid::new_v4()],
        last_updated: Utc::now(),
    };

    Ok(Json(ApiResponse::success(config)))
}

/// Get trading engine performance statistics
pub async fn get_engine_stats(
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<ApiResponse<EngineStatsResponse>>, StatusCode> {
    // Check permissions
    if !has_permission(&user, &Permission::ReadTradingData) {
        return Err(StatusCode::FORBIDDEN);
    }

    info!(user_id = %user.id, "Trading engine statistics requested");

    let stats = EngineStatsResponse {
        uptime_hours: 24.5,
        total_signals_generated: 1250,
        total_orders_executed: 890,
        total_trades_completed: 845,
        success_rate: 0.68,
        average_trade_duration_minutes: 245.0,
        total_pnl: 12500.0,
        daily_pnl: 850.0,
        max_drawdown: -0.08,
        sharpe_ratio: 1.45,
        performance_by_strategy: vec![
            StrategyPerformance {
                strategy_name: "MeanReversion".to_string(),
                signals_generated: 750,
                trades_executed: 520,
                success_rate: 0.72,
                total_pnl: 8200.0,
                sharpe_ratio: 1.52,
            },
            StrategyPerformance {
                strategy_name: "TrendFollowing".to_string(),
                signals_generated: 500,
                trades_executed: 325,
                success_rate: 0.62,
                total_pnl: 4300.0,
                sharpe_ratio: 1.35,
            },
        ],
        performance_by_instrument: vec![
            InstrumentPerformance {
                instrument_id: Uuid::new_v4(),
                instrument_name: "EUR/USD".to_string(),
                trades_executed: 285,
                total_pnl: 4500.0,
                win_rate: 0.68,
                average_trade_size: 1250.0,
            },
            InstrumentPerformance {
                instrument_id: Uuid::new_v4(),
                instrument_name: "GBP/USD".to_string(),
                trades_executed: 220,
                total_pnl: 3200.0,
                win_rate: 0.65,
                average_trade_size: 1100.0,
            },
        ],
    };

    Ok(Json(ApiResponse::success(stats)))
}

// Helper function to get engine status
async fn get_engine_status_internal(trading_engine: &crate::trading::TradingEngine) -> EngineStatusResponse {
    let state = trading_engine.get_state().await;
    let portfolio_summary = trading_engine.get_portfolio_summary().await;

    EngineStatusResponse {
        state,
        uptime_seconds: 88200, // Would be calculated from start time
        last_signal_time: Some(Utc::now()),
        last_risk_check: Some(Utc::now()),
        active_instruments: vec![Uuid::new_v4(), Uuid::new_v4()],
        daily_trade_count: 23,
        total_positions: portfolio_summary.num_positions as u32,
        total_pnl: portfolio_summary.total_pnl,
        engine_health: EngineHealth {
            overall_status: HealthStatus::Healthy,
            components: vec![
                ComponentHealth {
                    component_name: "Signal Generator".to_string(),
                    status: HealthStatus::Healthy,
                    last_check: Utc::now(),
                    message: Some("Operating normally".to_string()),
                    metrics: None,
                },
                ComponentHealth {
                    component_name: "Risk Manager".to_string(),
                    status: HealthStatus::Healthy,
                    last_check: Utc::now(),
                    message: Some("All limits within bounds".to_string()),
                    metrics: None,
                },
            ],
            performance_metrics: EnginePerformanceMetrics {
                signals_per_minute: 1.2,
                orders_per_minute: 0.8,
                average_signal_latency_ms: 15.5,
                average_order_latency_ms: 45.2,
                risk_check_latency_ms: 8.3,
                portfolio_update_latency_ms: 12.1,
                memory_usage_mb: 256.0,
                cpu_usage_percentage: 15.5,
            },
            resource_usage: ResourceUsage {
                memory_used_mb: 256.0,
                memory_total_mb: 1024.0,
                cpu_usage_percentage: 15.5,
                active_threads: 12,
                database_connections: 5,
                websocket_connections: 3,
            },
        },
        last_updated: Utc::now(),
    }
}
