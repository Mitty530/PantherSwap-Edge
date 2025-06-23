// Health check endpoints for production monitoring
// Provides comprehensive system health status and metrics

use crate::monitoring::production::{ProductionMonitor, SystemHealthStatus, ComponentHealth};
use crate::utils::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub uptime_seconds: u64,
    pub version: String,
    pub environment: String,
    pub overall_health_score: f64,
    pub components: HashMap<String, ComponentHealthResponse>,
}

/// Component health response
#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentHealthResponse {
    pub status: String,
    pub health_score: f64,
    pub last_check: DateTime<Utc>,
    pub error_count: u32,
    pub metrics: HashMap<String, f64>,
}

/// Detailed health response with metrics
#[derive(Debug, Serialize, Deserialize)]
pub struct DetailedHealthResponse {
    pub basic_health: HealthResponse,
    pub performance_metrics: PerformanceMetricsResponse,
    pub active_alerts: Vec<AlertResponse>,
    pub system_resources: SystemResourcesResponse,
}

/// Performance metrics response
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceMetricsResponse {
    pub requests_per_second: f64,
    pub average_response_time_ms: f64,
    pub error_rate_percentage: f64,
    pub ai_inference_latency_ms: f64,
    pub trading_latency_ms: f64,
    pub database_latency_ms: f64,
}

/// Alert response
#[derive(Debug, Serialize, Deserialize)]
pub struct AlertResponse {
    pub id: String,
    pub component: String,
    pub alert_type: String,
    pub severity: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

/// System resources response
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemResourcesResponse {
    pub memory_usage_percentage: f64,
    pub cpu_usage_percentage: f64,
    pub disk_usage_percentage: f64,
    pub network_throughput_mbps: f64,
    pub active_connections: u32,
    pub thread_count: u32,
}

/// Application state for health endpoints
#[derive(Clone)]
pub struct HealthState {
    pub production_monitor: Arc<ProductionMonitor>,
    pub version: String,
    pub environment: String,
}

/// Create health check router
pub fn create_health_router(state: HealthState) -> Router {
    Router::new()
        .route("/health", get(basic_health_check))
        .route("/health/detailed", get(detailed_health_check))
        .route("/health/components", get(component_health_check))
        .route("/health/metrics", get(metrics_health_check))
        .route("/health/alerts", get(alerts_health_check))
        .route("/ready", get(readiness_check))
        .route("/live", get(liveness_check))
        .with_state(state)
}

/// Basic health check endpoint
pub async fn basic_health_check(
    State(state): State<HealthState>,
) -> Result<(StatusCode, Json<HealthResponse>), (StatusCode, Json<serde_json::Value>)> {
    match get_basic_health(&state).await {
        Ok(health) => {
            let status_code = if health.overall_health_score >= 0.8 {
                StatusCode::OK
            } else if health.overall_health_score >= 0.5 {
                StatusCode::PARTIAL_CONTENT
            } else {
                StatusCode::SERVICE_UNAVAILABLE
            };
            Ok((status_code, Json(health)))
        }
        Err(e) => {
            let error_response = serde_json::json!({
                "error": "Health check failed",
                "message": e.to_string(),
                "timestamp": Utc::now()
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Detailed health check endpoint
pub async fn detailed_health_check(
    State(state): State<HealthState>,
) -> Result<(StatusCode, Json<DetailedHealthResponse>), (StatusCode, Json<serde_json::Value>)> {
    match get_detailed_health(&state).await {
        Ok(health) => {
            let status_code = if health.basic_health.overall_health_score >= 0.8 {
                StatusCode::OK
            } else {
                StatusCode::SERVICE_UNAVAILABLE
            };
            Ok((status_code, Json(health)))
        }
        Err(e) => {
            let error_response = serde_json::json!({
                "error": "Detailed health check failed",
                "message": e.to_string(),
                "timestamp": Utc::now()
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Component health check endpoint
pub async fn component_health_check(
    State(state): State<HealthState>,
) -> Result<(StatusCode, Json<HashMap<String, ComponentHealthResponse>>), (StatusCode, Json<serde_json::Value>)> {
    match get_component_health(&state).await {
        Ok(components) => Ok((StatusCode::OK, Json(components))),
        Err(e) => {
            let error_response = serde_json::json!({
                "error": "Component health check failed",
                "message": e.to_string(),
                "timestamp": Utc::now()
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Metrics health check endpoint
pub async fn metrics_health_check(
    State(state): State<HealthState>,
) -> Result<(StatusCode, Json<PerformanceMetricsResponse>), (StatusCode, Json<serde_json::Value>)> {
    match get_performance_metrics(&state).await {
        Ok(metrics) => Ok((StatusCode::OK, Json(metrics))),
        Err(e) => {
            let error_response = serde_json::json!({
                "error": "Metrics health check failed",
                "message": e.to_string(),
                "timestamp": Utc::now()
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Alerts health check endpoint
pub async fn alerts_health_check(
    State(state): State<HealthState>,
) -> Result<(StatusCode, Json<Vec<AlertResponse>>), (StatusCode, Json<serde_json::Value>)> {
    match get_active_alerts(&state).await {
        Ok(alerts) => Ok((StatusCode::OK, Json(alerts))),
        Err(e) => {
            let error_response = serde_json::json!({
                "error": "Alerts health check failed",
                "message": e.to_string(),
                "timestamp": Utc::now()
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Readiness check endpoint (for Kubernetes)
pub async fn readiness_check(
    State(state): State<HealthState>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    match get_basic_health(&state).await {
        Ok(health) => {
            if health.overall_health_score >= 0.8 {
                Ok((StatusCode::OK, Json(serde_json::json!({
                    "status": "ready",
                    "timestamp": Utc::now()
                }))))
            } else {
                Err((StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
                    "status": "not ready",
                    "health_score": health.overall_health_score,
                    "timestamp": Utc::now()
                }))))
            }
        }
        Err(e) => {
            Err((StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
                "status": "not ready",
                "error": e.to_string(),
                "timestamp": Utc::now()
            }))))
        }
    }
}

/// Liveness check endpoint (for Kubernetes)
pub async fn liveness_check(
    State(_state): State<HealthState>,
) -> (StatusCode, Json<serde_json::Value>) {
    // Simple liveness check - if we can respond, we're alive
    (StatusCode::OK, Json(serde_json::json!({
        "status": "alive",
        "timestamp": Utc::now()
    })))
}

/// Get basic health information
async fn get_basic_health(state: &HealthState) -> Result<HealthResponse> {
    let system_health = state.production_monitor.get_system_health().await;
    
    let mut components = HashMap::new();
    for (name, component) in &system_health.component_health {
        components.insert(name.clone(), ComponentHealthResponse {
            status: format!("{:?}", component.status),
            health_score: component.health_score,
            last_check: component.last_check,
            error_count: component.error_count,
            metrics: component.performance_metrics.clone(),
        });
    }
    
    Ok(HealthResponse {
        status: if system_health.overall_health_score >= 0.8 {
            "healthy".to_string()
        } else if system_health.overall_health_score >= 0.5 {
            "degraded".to_string()
        } else {
            "unhealthy".to_string()
        },
        timestamp: system_health.timestamp,
        uptime_seconds: system_health.uptime_seconds,
        version: state.version.clone(),
        environment: state.environment.clone(),
        overall_health_score: system_health.overall_health_score,
        components,
    })
}

/// Get detailed health information
async fn get_detailed_health(state: &HealthState) -> Result<DetailedHealthResponse> {
    let basic_health = get_basic_health(state).await?;
    let performance_metrics = get_performance_metrics(state).await?;
    let active_alerts = get_active_alerts(state).await?;
    let system_resources = get_system_resources(state).await?;
    
    Ok(DetailedHealthResponse {
        basic_health,
        performance_metrics,
        active_alerts,
        system_resources,
    })
}

/// Get component health information
async fn get_component_health(state: &HealthState) -> Result<HashMap<String, ComponentHealthResponse>> {
    let system_health = state.production_monitor.get_system_health().await;
    
    let mut components = HashMap::new();
    for (name, component) in &system_health.component_health {
        components.insert(name.clone(), ComponentHealthResponse {
            status: format!("{:?}", component.status),
            health_score: component.health_score,
            last_check: component.last_check,
            error_count: component.error_count,
            metrics: component.performance_metrics.clone(),
        });
    }
    
    Ok(components)
}

/// Get performance metrics
async fn get_performance_metrics(state: &HealthState) -> Result<PerformanceMetricsResponse> {
    let system_health = state.production_monitor.get_system_health().await;
    let metrics = &system_health.performance_metrics;
    
    Ok(PerformanceMetricsResponse {
        requests_per_second: metrics.total_requests_per_second,
        average_response_time_ms: metrics.average_response_time_ms,
        error_rate_percentage: metrics.error_rate_percentage,
        ai_inference_latency_ms: 50.0, // Mock - would get from AI monitor
        trading_latency_ms: 8.5, // Mock - would get from trading engine
        database_latency_ms: 25.0, // Mock - would get from DB monitor
    })
}

/// Get active alerts
async fn get_active_alerts(state: &HealthState) -> Result<Vec<AlertResponse>> {
    let alerts = state.production_monitor.get_active_alerts().await;
    
    let alert_responses = alerts.into_iter().map(|alert| AlertResponse {
        id: alert.id.to_string(),
        component: alert.component,
        alert_type: format!("{:?}", alert.alert_type),
        severity: format!("{:?}", alert.severity),
        message: alert.message,
        timestamp: alert.timestamp,
    }).collect();
    
    Ok(alert_responses)
}

/// Get system resources information
async fn get_system_resources(state: &HealthState) -> Result<SystemResourcesResponse> {
    let system_health = state.production_monitor.get_system_health().await;
    let metrics = &system_health.performance_metrics;
    
    Ok(SystemResourcesResponse {
        memory_usage_percentage: metrics.memory_usage_percentage,
        cpu_usage_percentage: metrics.cpu_usage_percentage,
        disk_usage_percentage: metrics.disk_usage_percentage,
        network_throughput_mbps: metrics.network_throughput_mbps,
        active_connections: 150, // Mock - would get from server metrics
        thread_count: 32, // Mock - would get from runtime metrics
    })
}
