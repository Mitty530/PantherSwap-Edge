use axum::{
    extract::Extension,
    Json,
    http::StatusCode,
};
use serde::Serialize;
use chrono::{DateTime, Utc};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, error};

use crate::api::{AppState, responses::*};
use crate::monitoring::{SystemHealthStatus, ComponentHealth, SystemAlert, SystemPerformanceMetrics};
use std::collections::HashMap;

/// Basic health check endpoint
pub async fn health_check() -> Result<Json<ApiResponse<HealthResponse>>, StatusCode> {
    let uptime = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let health_response = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime,
        timestamp: Utc::now(),
    };

    info!("Health check requested");
    Ok(Json(ApiResponse::success(health_response)))
}

/// Detailed system status endpoint with production monitoring
pub async fn system_status(
    Extension(state): Extension<AppState>,
) -> Result<Json<ApiResponse<SystemStatusResponse>>, StatusCode> {
    info!("System status check requested");

    // Get comprehensive health status from production monitor
    let system_health = state.production_monitor.get_system_health().await;
    let component_health = state.production_monitor.get_all_component_health().await;

    // Check database status
    let database_status = check_database_status(&state).await;

    // Check market data status
    let market_data_status = check_market_data_status(&state).await;

    // Check API status
    let api_status = check_api_status().await;

    // Determine overall status based on production monitor health score
    let overall_status = if system_health.overall_health_score >= 0.8 {
        "healthy".to_string()
    } else if system_health.overall_health_score >= 0.6 {
        "degraded".to_string()
    } else {
        "critical".to_string()
    };

    let system_status = SystemStatusResponse {
        database: database_status,
        market_data: market_data_status,
        api: api_status,
        overall_status,
    };

    Ok(Json(ApiResponse::success(system_status)))
}

/// Metrics endpoint for monitoring
pub async fn metrics(
    Extension(state): Extension<AppState>,
) -> Result<Json<ApiResponse<MetricsResponse>>, StatusCode> {
    info!("Metrics requested");

    let metrics = collect_metrics(&state).await;
    Ok(Json(ApiResponse::success(metrics)))
}

/// Check database connectivity and status
async fn check_database_status(state: &AppState) -> DatabaseStatus {
    match sqlx::query("SELECT 1").fetch_one(&state.database.pool).await {
        Ok(_) => {
            let pool_options = state.database.pool.options();
            DatabaseStatus {
                connected: true,
                pool_size: pool_options.get_max_connections(),
                active_connections: state.database.pool.size(),
                last_query_time: Some(Utc::now()),
            }
        }
        Err(e) => {
            error!("Database health check failed: {}", e);
            DatabaseStatus {
                connected: false,
                pool_size: 0,
                active_connections: 0,
                last_query_time: None,
            }
        }
    }
}

/// Check market data provider status
async fn check_market_data_status(state: &AppState) -> MarketDataStatus {
    // Try to get the latest market data to check if providers are working
    let query_manager = state.database.query_manager();
    
    match query_manager.get_latest_market_ticks(None, Some(1)).await {
        Ok(ticks) => {
            let last_update = ticks.first().map(|tick| tick.timestamp);
            let avg_quality = ticks.first()
                .map(|tick| tick.data_quality_score)
                .unwrap_or(0.0);
            
            MarketDataStatus {
                providers_active: 1, // We have Alpha Vantage
                last_update,
                total_instruments: 5, // Our default forex pairs
                data_quality_avg: avg_quality,
            }
        }
        Err(e) => {
            error!("Market data status check failed: {}", e);
            MarketDataStatus {
                providers_active: 0,
                last_update: None,
                total_instruments: 0,
                data_quality_avg: 0.0,
            }
        }
    }
}

/// Check API status
async fn check_api_status() -> ApiStatus {
    // In a real implementation, this would check actual metrics
    // For now, return mock data
    ApiStatus {
        requests_per_minute: 0, // Would be tracked by rate limiter
        active_connections: 1,  // Would be tracked by connection manager
        error_rate: 0.0,        // Would be calculated from error metrics
    }
}

/// Collect comprehensive metrics
async fn collect_metrics(state: &AppState) -> MetricsResponse {
    let database_metrics = collect_database_metrics(state).await;
    let api_metrics = collect_api_metrics().await;
    let system_metrics = collect_system_metrics().await;

    MetricsResponse {
        timestamp: Utc::now(),
        database: database_metrics,
        api: api_metrics,
        system: system_metrics,
    }
}

/// Collect database-specific metrics
async fn collect_database_metrics(state: &AppState) -> DatabaseMetrics {
    let pool = &state.database.pool;
    
    DatabaseMetrics {
        pool_size: pool.size(),
        active_connections: pool.size(),
        idle_connections: pool.options().get_max_connections() - pool.size(),
        total_queries: 0, // Would need to be tracked
        failed_queries: 0, // Would need to be tracked
        avg_query_time_ms: 0.0, // Would need to be tracked
    }
}

/// Collect API-specific metrics
async fn collect_api_metrics() -> ApiMetrics {
    // In a real implementation, these would be tracked by middleware
    ApiMetrics {
        total_requests: 0,
        requests_per_minute: 0,
        error_rate: 0.0,
        avg_response_time_ms: 0.0,
        active_connections: 1,
    }
}

/// Collect system-level metrics
async fn collect_system_metrics() -> SystemMetrics {
    let uptime = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    SystemMetrics {
        uptime_seconds: uptime,
        memory_usage_mb: 0.0,    // Would use system metrics crate
        cpu_usage_percent: 0.0,  // Would use system metrics crate
        disk_usage_percent: 0.0, // Would use system metrics crate
    }
}

/// Additional response types for metrics
#[derive(Debug, Serialize)]
pub struct MetricsResponse {
    pub timestamp: DateTime<Utc>,
    pub database: DatabaseMetrics,
    pub api: ApiMetrics,
    pub system: SystemMetrics,
}

#[derive(Debug, Serialize)]
pub struct DatabaseMetrics {
    pub pool_size: u32,
    pub active_connections: u32,
    pub idle_connections: u32,
    pub total_queries: u64,
    pub failed_queries: u64,
    pub avg_query_time_ms: f64,
}

#[derive(Debug, Serialize)]
pub struct ApiMetrics {
    pub total_requests: u64,
    pub requests_per_minute: u64,
    pub error_rate: f64,
    pub avg_response_time_ms: f64,
    pub active_connections: u32,
}

#[derive(Debug, Serialize)]
pub struct SystemMetrics {
    pub uptime_seconds: u64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub disk_usage_percent: f64,
}

/// Readiness probe for Kubernetes
pub async fn readiness(
    Extension(state): Extension<AppState>,
) -> Result<Json<ApiResponse<ReadinessResponse>>, StatusCode> {
    // Check if all critical services are ready
    let database_ready = check_database_ready(&state).await;
    
    let ready = database_ready;
    let status_code = if ready { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };
    
    let readiness_response = ReadinessResponse {
        ready,
        services: ReadinessServices {
            database: database_ready,
            market_data: true, // Alpha Vantage is external, assume ready
        },
    };

    if ready {
        Ok(Json(ApiResponse::success(readiness_response)))
    } else {
        Err(status_code)
    }
}

/// Liveness probe for Kubernetes
pub async fn liveness() -> Result<Json<ApiResponse<LivenessResponse>>, StatusCode> {
    // Simple liveness check - if we can respond, we're alive
    let liveness_response = LivenessResponse {
        alive: true,
        timestamp: Utc::now(),
    };

    Ok(Json(ApiResponse::success(liveness_response)))
}

/// Comprehensive production monitoring endpoint
pub async fn production_monitoring(
    Extension(state): Extension<AppState>,
) -> Result<Json<ApiResponse<ProductionMonitoringResponse>>, StatusCode> {
    info!("Production monitoring data requested");

    // Get comprehensive monitoring data
    let system_health = state.production_monitor.get_system_health().await;
    let component_health = state.production_monitor.get_all_component_health().await;
    let active_alerts = state.production_monitor.get_active_alerts().await;
    let performance_metrics = state.production_monitor.get_performance_metrics().await;

    let monitoring_response = ProductionMonitoringResponse {
        timestamp: Utc::now(),
        system_health,
        component_health,
        active_alerts,
        performance_metrics,
    };

    Ok(Json(ApiResponse::success(monitoring_response)))
}

/// Check if database is ready for queries
async fn check_database_ready(state: &AppState) -> bool {
    sqlx::query("SELECT 1")
        .fetch_one(&state.database.pool)
        .await
        .is_ok()
}

#[derive(Debug, Serialize)]
pub struct ReadinessResponse {
    pub ready: bool,
    pub services: ReadinessServices,
}

#[derive(Debug, Serialize)]
pub struct ReadinessServices {
    pub database: bool,
    pub market_data: bool,
}

#[derive(Debug, Serialize)]
pub struct LivenessResponse {
    pub alive: bool,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ProductionMonitoringResponse {
    pub timestamp: DateTime<Utc>,
    pub system_health: SystemHealthStatus,
    pub component_health: HashMap<String, ComponentHealth>,
    pub active_alerts: Vec<SystemAlert>,
    pub performance_metrics: Vec<SystemPerformanceMetrics>,
}
