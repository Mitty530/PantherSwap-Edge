pub mod routes;
pub mod middleware;
pub mod responses;

use axum::{
    Router,
    serve,
    routing::{get, post, put},
    middleware as axum_middleware,
    Extension,
};
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
    compression::CompressionLayer,
    timeout::TimeoutLayer,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use crate::database::Database;
use crate::trading::TradingEngine;
use crate::ai::AIEngine;
use crate::monitoring::ProductionMonitor;
use crate::utils::Result;
use tokio::net::TcpListener;
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    pub database: Database,
    pub trading_engine: Arc<tokio::sync::RwLock<TradingEngine>>,
    pub ai_engine: Arc<Mutex<AIEngine>>,
    pub production_monitor: Arc<ProductionMonitor>,
}

pub async fn create_app(state: AppState) -> Router {
    info!("Creating API application with routes and middleware");

    // Start rate limiter cleanup task
    middleware::rate_limit::start_cleanup_task();

    Router::new()
        // Health and monitoring endpoints (no auth required)
        .route("/health", get(routes::health::health_check))
        .route("/health/liveness", get(routes::health::liveness))
        .route("/health/readiness", get(routes::health::readiness))
        .route("/status", get(routes::health::system_status))
        .route("/metrics", get(routes::health::metrics))
        .route("/monitoring", get(routes::health::production_monitoring))

        // API v1 routes (auth required)
        .route("/api/v1/instruments", get(routes::instruments::list_instruments))
        .route("/api/v1/instruments/:id", get(routes::instruments::get_instrument))
        .route("/api/v1/instruments", post(routes::instruments::create_instrument))
        .route("/api/v1/instruments/:id", put(routes::instruments::update_instrument))

        // Trading Orders
        .route("/api/v1/orders", post(routes::orders::submit_order))
        .route("/api/v1/orders", get(routes::orders::list_orders))
        .route("/api/v1/orders/:id", get(routes::orders::get_order))
        .route("/api/v1/orders/:id/cancel", post(routes::orders::cancel_order))
        .route("/api/v1/orders/stats", get(routes::orders::get_order_stats))

        // Portfolio Management
        .route("/api/v1/portfolio/positions", get(routes::portfolio::get_positions))
        .route("/api/v1/portfolio/performance", get(routes::portfolio::get_performance))
        .route("/api/v1/portfolio/risk", get(routes::portfolio::get_risk_metrics))
        .route("/api/v1/portfolio/summary", get(routes::portfolio::get_summary))

        // Trading Signals
        .route("/api/v1/signals", get(routes::signals::get_signals))
        .route("/api/v1/signals/latest", get(routes::signals::get_latest_signals))
        .route("/api/v1/signals/performance", get(routes::signals::get_signal_performance))
        .route("/api/v1/signals/analytics", get(routes::signals::get_signal_analytics))

        // Risk Management
        .route("/api/v1/risk/metrics", get(routes::risk::get_risk_metrics))
        .route("/api/v1/risk/limits", get(routes::risk::get_risk_limits))
        .route("/api/v1/risk/alerts", get(routes::risk::get_risk_alerts))
        .route("/api/v1/risk/monitoring", get(routes::risk::get_risk_monitoring))

        // Strategy Management
        .route("/api/v1/strategies", get(routes::strategies::list_strategies))
        .route("/api/v1/strategies/:id", get(routes::strategies::get_strategy))
        .route("/api/v1/strategies/:id", put(routes::strategies::update_strategy))
        .route("/api/v1/strategies/:id/performance", get(routes::strategies::get_strategy_performance))

        // Trading Engine Control
        .route("/api/v1/engine/start", post(routes::trading_engine::start_engine))
        .route("/api/v1/engine/stop", post(routes::trading_engine::stop_engine))
        .route("/api/v1/engine/status", get(routes::trading_engine::get_engine_status))
        .route("/api/v1/engine/config", get(routes::trading_engine::get_engine_config))
        .route("/api/v1/engine/config", put(routes::trading_engine::update_engine_config))
        .route("/api/v1/engine/stats", get(routes::trading_engine::get_engine_stats))

        .route("/api/v1/market-data/ticks", get(routes::market_data::get_ticks))
        .route("/api/v1/market-data/latest", get(routes::market_data::get_latest_ticks))
        .route("/api/v1/market-data/ohlc", get(routes::market_data::get_ohlc))
        .route("/api/v1/market-data/stats", get(routes::market_data::get_market_stats))

        // Add middleware layers
        .layer(
            ServiceBuilder::new()
                // Outer layers (applied first)
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()) // In production, use restrictive CORS
                .layer(CompressionLayer::new())
                .layer(TimeoutLayer::new(Duration::from_secs(30)))

                // Security and validation middleware
                .layer(axum_middleware::from_fn(middleware::cors::security_headers_middleware))
                .layer(axum_middleware::from_fn(middleware::cors::cors_middleware))
                .layer(axum_middleware::from_fn(middleware::validation::validation_middleware))

                // Authentication and rate limiting
                .layer(axum_middleware::from_fn(middleware::rate_limit::rate_limit_middleware))
                .layer(axum_middleware::from_fn(middleware::auth::auth_middleware))

                // Request processing middleware
                .layer(axum_middleware::from_fn(middleware::request_id_middleware))
                .layer(axum_middleware::from_fn(middleware::logging_middleware))

                // Application state
                .layer(Extension(state))
        )
}

pub async fn start_server(app: Router, port: u16) -> Result<()> {
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;

    info!("🚀 PantherSwap Edge API server starting on {}", addr);
    info!("📊 Health check available at: http://{}/health", addr);
    info!("📈 Market data API available at: http://{}/api/v1/market-data/", addr);
    info!("🔧 Instruments API available at: http://{}/api/v1/instruments", addr);

    serve(listener, app).await?;
    Ok(())
}
