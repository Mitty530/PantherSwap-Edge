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

/// Query parameters for risk metrics
#[derive(Debug, Deserialize)]
pub struct RiskMetricsQuery {
    pub instrument_id: Option<Uuid>,
    pub strategy_name: Option<String>,
    pub time_horizon: Option<String>, // "1d", "1w", "1m"
    pub confidence_level: Option<f64>, // 0.95, 0.99
}

/// Request to update risk limits
#[derive(Debug, Deserialize)]
pub struct UpdateRiskLimitsRequest {
    pub max_position_size: Option<f64>,
    pub max_daily_loss: Option<f64>,
    pub max_drawdown: Option<f64>,
    pub var_limit: Option<f64>,
    pub concentration_limit: Option<f64>,
    pub leverage_limit: Option<f64>,
}

/// Response for risk metrics
#[derive(Debug, Serialize)]
pub struct RiskMetricsResponse {
    pub portfolio_var_95: f64,
    pub portfolio_var_99: f64,
    pub expected_shortfall_95: f64,
    pub expected_shortfall_99: f64,
    pub max_drawdown: f64,
    pub current_drawdown: f64,
    pub portfolio_volatility: f64,
    pub beta_to_market: f64,
    pub correlation_to_market: f64,
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub calmar_ratio: f64,
    pub leverage_ratio: f64,
    pub concentration_risk: f64,
    pub liquidity_risk: f64,
    pub stress_test_results: StressTestResults,
    pub last_updated: DateTime<Utc>,
}

/// Stress test results
#[derive(Debug, Serialize)]
pub struct StressTestResults {
    pub market_crash_scenario: f64,
    pub interest_rate_shock: f64,
    pub currency_crisis: f64,
    pub liquidity_crisis: f64,
    pub worst_case_scenario: f64,
}

/// Response for risk limits
#[derive(Debug, Serialize)]
pub struct RiskLimitsResponse {
    pub max_position_size: f64,
    pub current_max_position: f64,
    pub max_daily_loss: f64,
    pub current_daily_loss: f64,
    pub max_drawdown: f64,
    pub current_drawdown: f64,
    pub var_limit: f64,
    pub current_var: f64,
    pub concentration_limit: f64,
    pub current_concentration: f64,
    pub leverage_limit: f64,
    pub current_leverage: f64,
    pub limits_status: Vec<LimitStatus>,
}

/// Status of individual risk limits
#[derive(Debug, Serialize)]
pub struct LimitStatus {
    pub limit_name: String,
    pub limit_value: f64,
    pub current_value: f64,
    pub utilization_percentage: f64,
    pub status: RiskStatus,
    pub last_breach: Option<DateTime<Utc>>,
}

/// Risk status levels
#[derive(Debug, Serialize)]
pub enum RiskStatus {
    Normal,
    Warning,
    Critical,
    Breached,
}

/// Response for risk alerts
#[derive(Debug, Serialize)]
pub struct RiskAlertsResponse {
    pub active_alerts: Vec<RiskAlert>,
    pub recent_alerts: Vec<RiskAlert>,
    pub alert_summary: AlertSummary,
}

/// Individual risk alert
#[derive(Debug, Serialize)]
pub struct RiskAlert {
    pub id: Uuid,
    pub alert_type: RiskAlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub instrument_id: Option<Uuid>,
    pub strategy_name: Option<String>,
    pub threshold_value: f64,
    pub current_value: f64,
    pub created_at: DateTime<Utc>,
    pub acknowledged: bool,
    pub acknowledged_by: Option<String>,
    pub acknowledged_at: Option<DateTime<Utc>>,
}

/// Types of risk alerts
#[derive(Debug, Serialize)]
pub enum RiskAlertType {
    VarBreach,
    DrawdownLimit,
    ConcentrationRisk,
    LeverageLimit,
    LiquidityRisk,
    VolatilitySpike,
    CorrelationBreakdown,
    StressTestFailure,
}

/// Alert severity levels
#[derive(Debug, Serialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Summary of alerts
#[derive(Debug, Serialize)]
pub struct AlertSummary {
    pub total_active_alerts: u32,
    pub critical_alerts: u32,
    pub high_alerts: u32,
    pub medium_alerts: u32,
    pub low_alerts: u32,
    pub alerts_today: u32,
    pub most_frequent_alert_type: Option<RiskAlertType>,
}

/// Response for risk monitoring dashboard
#[derive(Debug, Serialize)]
pub struct RiskMonitoringResponse {
    pub overall_risk_score: f64,
    pub risk_trend: RiskTrend,
    pub key_metrics: Vec<KeyRiskMetric>,
    pub risk_attribution: Vec<RiskAttribution>,
    pub recommendations: Vec<RiskRecommendation>,
    pub last_updated: DateTime<Utc>,
}

/// Risk trend direction
#[derive(Debug, Serialize)]
pub enum RiskTrend {
    Increasing,
    Stable,
    Decreasing,
}

/// Key risk metric for dashboard
#[derive(Debug, Serialize)]
pub struct KeyRiskMetric {
    pub name: String,
    pub value: f64,
    pub change_24h: f64,
    pub status: RiskStatus,
    pub description: String,
}

/// Risk attribution by source
#[derive(Debug, Serialize)]
pub struct RiskAttribution {
    pub source: String,
    pub contribution_percentage: f64,
    pub var_contribution: f64,
    pub description: String,
}

/// Risk management recommendation
#[derive(Debug, Serialize)]
pub struct RiskRecommendation {
    pub priority: String,
    pub action: String,
    pub description: String,
    pub expected_impact: String,
}

/// Get comprehensive risk metrics
pub async fn get_risk_metrics(
    Query(params): Query<RiskMetricsQuery>,
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<ApiResponse<RiskMetricsResponse>>, StatusCode> {
    // Check permissions
    if !has_permission(&user, &Permission::ReadPortfolio) {
        warn!(user_id = %user.id, "Permission denied for risk metrics access");
        return Err(StatusCode::FORBIDDEN);
    }

    info!(
        user_id = %user.id,
        instrument_id = ?params.instrument_id,
        time_horizon = ?params.time_horizon,
        "Risk metrics requested"
    );

    // In a real implementation, this would calculate risk metrics from current positions
    let risk_metrics = RiskMetricsResponse {
        portfolio_var_95: -1250.0,
        portfolio_var_99: -2100.0,
        expected_shortfall_95: -1800.0,
        expected_shortfall_99: -2800.0,
        max_drawdown: -0.08,
        current_drawdown: -0.02,
        portfolio_volatility: 0.18,
        beta_to_market: 0.85,
        correlation_to_market: 0.72,
        sharpe_ratio: 1.45,
        sortino_ratio: 1.82,
        calmar_ratio: 2.15,
        leverage_ratio: 1.2,
        concentration_risk: 0.25,
        liquidity_risk: 0.15,
        stress_test_results: StressTestResults {
            market_crash_scenario: -0.15,
            interest_rate_shock: -0.08,
            currency_crisis: -0.12,
            liquidity_crisis: -0.18,
            worst_case_scenario: -0.22,
        },
        last_updated: Utc::now(),
    };

    Ok(Json(ApiResponse::success(risk_metrics)))
}

/// Get current risk limits and their status
pub async fn get_risk_limits(
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<ApiResponse<RiskLimitsResponse>>, StatusCode> {
    // Check permissions
    if !has_permission(&user, &Permission::ReadPortfolio) {
        return Err(StatusCode::FORBIDDEN);
    }

    info!(user_id = %user.id, "Risk limits requested");

    let risk_limits = RiskLimitsResponse {
        max_position_size: 10000.0,
        current_max_position: 7500.0,
        max_daily_loss: -1000.0,
        current_daily_loss: -250.0,
        max_drawdown: -0.10,
        current_drawdown: -0.02,
        var_limit: -2000.0,
        current_var: -1250.0,
        concentration_limit: 0.30,
        current_concentration: 0.25,
        leverage_limit: 2.0,
        current_leverage: 1.2,
        limits_status: vec![
            LimitStatus {
                limit_name: "Position Size".to_string(),
                limit_value: 10000.0,
                current_value: 7500.0,
                utilization_percentage: 75.0,
                status: RiskStatus::Normal,
                last_breach: None,
            },
            LimitStatus {
                limit_name: "Daily Loss".to_string(),
                limit_value: 1000.0,
                current_value: 250.0,
                utilization_percentage: 25.0,
                status: RiskStatus::Normal,
                last_breach: None,
            },
        ],
    };

    Ok(Json(ApiResponse::success(risk_limits)))
}

/// Get risk alerts
pub async fn get_risk_alerts(
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<ApiResponse<RiskAlertsResponse>>, StatusCode> {
    // Check permissions
    if !has_permission(&user, &Permission::ReadPortfolio) {
        return Err(StatusCode::FORBIDDEN);
    }

    info!(user_id = %user.id, "Risk alerts requested");

    let alerts = RiskAlertsResponse {
        active_alerts: vec![
            RiskAlert {
                id: Uuid::new_v4(),
                alert_type: RiskAlertType::ConcentrationRisk,
                severity: AlertSeverity::Medium,
                message: "Portfolio concentration in EUR/USD exceeds 20%".to_string(),
                instrument_id: Some(Uuid::new_v4()),
                strategy_name: Some("MeanReversion".to_string()),
                threshold_value: 0.20,
                current_value: 0.25,
                created_at: Utc::now(),
                acknowledged: false,
                acknowledged_by: None,
                acknowledged_at: None,
            },
        ],
        recent_alerts: vec![],
        alert_summary: AlertSummary {
            total_active_alerts: 1,
            critical_alerts: 0,
            high_alerts: 0,
            medium_alerts: 1,
            low_alerts: 0,
            alerts_today: 3,
            most_frequent_alert_type: Some(RiskAlertType::ConcentrationRisk),
        },
    };

    Ok(Json(ApiResponse::success(alerts)))
}

/// Get risk monitoring dashboard
pub async fn get_risk_monitoring(
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<ApiResponse<RiskMonitoringResponse>>, StatusCode> {
    // Check permissions
    if !has_permission(&user, &Permission::ReadPortfolio) {
        return Err(StatusCode::FORBIDDEN);
    }

    info!(user_id = %user.id, "Risk monitoring dashboard requested");

    let monitoring = RiskMonitoringResponse {
        overall_risk_score: 6.5, // Out of 10
        risk_trend: RiskTrend::Stable,
        key_metrics: vec![
            KeyRiskMetric {
                name: "Portfolio VaR (95%)".to_string(),
                value: -1250.0,
                change_24h: -50.0,
                status: RiskStatus::Normal,
                description: "Value at Risk at 95% confidence level".to_string(),
            },
            KeyRiskMetric {
                name: "Current Drawdown".to_string(),
                value: -0.02,
                change_24h: 0.005,
                status: RiskStatus::Normal,
                description: "Current portfolio drawdown from peak".to_string(),
            },
        ],
        risk_attribution: vec![
            RiskAttribution {
                source: "EUR/USD".to_string(),
                contribution_percentage: 35.0,
                var_contribution: -437.5,
                description: "Largest position contributing to portfolio risk".to_string(),
            },
            RiskAttribution {
                source: "GBP/USD".to_string(),
                contribution_percentage: 25.0,
                var_contribution: -312.5,
                description: "Second largest risk contributor".to_string(),
            },
        ],
        recommendations: vec![
            RiskRecommendation {
                priority: "Medium".to_string(),
                action: "Reduce EUR/USD concentration".to_string(),
                description: "Consider reducing EUR/USD position size to below 20% of portfolio".to_string(),
                expected_impact: "Reduce portfolio VaR by approximately 8%".to_string(),
            },
        ],
        last_updated: Utc::now(),
    };

    Ok(Json(ApiResponse::success(monitoring)))
}
