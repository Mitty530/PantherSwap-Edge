// Common test utilities for PantherSwap Edge integration tests

use pantherswap_edge::config::Settings;
use pantherswap_edge::database::Database;
use pantherswap_edge::trading::{TradingEngine, TradingEngineConfig};
use pantherswap_edge::ai::AIEngine;
use pantherswap_edge::api::{AppState, create_app};
use axum::Router;
use std::sync::Arc;
use tokio::sync::{Mutex, OnceCell};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Global test app instance to avoid recreating for each test
static TEST_APP: OnceCell<Router> = OnceCell::const_new();

/// Initialize logging for tests (called once)
pub fn init_test_logging() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "warn,pantherswap_edge=info".into()),
            )
            .with(tracing_subscriber::fmt::layer().with_test_writer())
            .init();
    });
}

/// Create a test application instance
pub async fn setup_test_app() -> Router {
    create_test_app().await
}

/// Create a test application instance (alias for compatibility)
pub async fn create_test_app() -> Router {
    init_test_logging();

    TEST_APP
        .get_or_init(|| async {
            create_test_app_internal().await
        })
        .await
        .clone()
}

/// Internal function to create the test app
async fn create_test_app_internal() -> Router {
    // Load test configuration
    let settings = create_test_settings();

    // Create test database (mock if real DB not available)
    let database = create_test_database(&settings).await;

    // Initialize test trading engine
    let trading_config = TradingEngineConfig::default();
    let trading_engine = Arc::new(Mutex::new(
        TradingEngine::new(trading_config, database.clone())
            .await
            .expect("Failed to create trading engine")
    ));

    // Initialize test AI engine
    let ai_engine = Arc::new(Mutex::new(
        AIEngine::new(database.clone())
            .await
            .expect("Failed to create AI engine")
    ));

    // Create application state
    let app_state = AppState {
        database,
        trading_engine,
        ai_engine,
    };

    // Create the application
    create_app(app_state).await
}

/// Create test settings
pub fn create_test_settings() -> Settings {
    // Try to load from environment, fallback to defaults
    match Settings::load() {
        Ok(settings) => settings,
        Err(_) => {
            // Create minimal test settings with all required fields
            Settings {
                debug: true,
                server: pantherswap_edge::config::settings::ServerConfig {
                    host: "127.0.0.1".to_string(),
                    port: 8080,
                    workers: 1,
                },
                database: pantherswap_edge::config::settings::DatabaseConfig {
                    url: "postgresql://localhost/test_db".to_string(),
                    pool_size: 5,
                    connection_timeout_ms: 5000,
                    query_timeout_ms: 10000,
                },
                market_data: pantherswap_edge::config::settings::MarketDataConfig {
                    providers: vec!["alpha_vantage".to_string()],
                    instruments: vec![
                        "EURUSD".to_string(),
                        "GBPUSD".to_string(),
                        "USDJPY".to_string(),
                    ],
                    update_interval_ms: 60000,
                    max_retries: 3,
                    timeout_ms: 10000,
                    quality_threshold: 0.7,
                    alpha_vantage_api_key: "test_key".to_string(),
                    iex_cloud_api_key: "test_key".to_string(),
                },
                trading: pantherswap_edge::config::settings::TradingConfig {
                    enable_live_trading: false,
                    signal_generation_interval_ms: 1000,
                    risk_check_interval_ms: 500,
                    max_position_size: 1000000.0,
                    confidence_threshold: 0.7,
                    strategy_types: vec!["test_strategy".to_string()],
                },
                ai: pantherswap_edge::config::settings::AIConfig {
                    enable_predictions: true,
                    model_path: "test_model".to_string(),
                    model_update_interval_hours: 24,
                    prediction_horizons_minutes: vec![60],
                    enable_training: false,
                    training_batch_size: 64,
                },
                risk: pantherswap_edge::config::settings::RiskConfig {
                    max_portfolio_var: 0.02,
                    max_leverage: 3.0,
                    confidence_threshold: 0.7,
                    max_daily_loss: 50000.0,
                    drawdown_limit: 0.1,
                },
                api: pantherswap_edge::config::settings::APIConfig {
                    rate_limit_per_minute: 1000,
                    enable_cors: true,
                    max_request_size: 1024 * 1024,
                    api_key_header: "X-API-Key".to_string(),
                },
            }
        }
    }
}

/// Create test database connection
async fn create_test_database(settings: &Settings) -> Database {
    // Try to connect to real database first
    match Database::new_testing(&settings.database.url).await {
        Ok(db) => {
            tracing::info!("Connected to test database successfully");
            db
        }
        Err(e) => {
            tracing::warn!("Failed to connect to test database: {}", e);
            tracing::info!("Creating mock database for testing");
            
            // Create a mock database connection for testing
            // This will allow tests to run even without a real database
            create_mock_database().await
        }
    }
}

/// Create a mock database for testing when real DB is not available
async fn create_mock_database() -> Database {
    // Use an in-memory SQLite database for testing
    let mock_url = "sqlite::memory:";
    
    match Database::new_testing(mock_url).await {
        Ok(db) => {
            tracing::info!("Created in-memory test database");
            db
        }
        Err(_) => {
            // If even SQLite fails, create a minimal mock
            // This is a fallback - in practice you'd want proper mocking
            tracing::warn!("Creating minimal mock database");
            
            // For now, try with a dummy PostgreSQL URL
            // The tests will handle database errors gracefully
            Database::new_testing("postgresql://localhost/nonexistent")
                .await
                .unwrap_or_else(|_| {
                    panic!("Cannot create any database connection for testing")
                })
        }
    }
}

/// Test data generators
pub mod test_data {
    use serde_json::{json, Value};
    use uuid::Uuid;

    /// Generate test instrument data
    pub fn create_test_instrument() -> Value {
        json!({
            "symbol": format!("TEST{}", Uuid::new_v4().to_string()[..8].to_uppercase()),
            "name": "Test Currency Pair",
            "instrument_type": "forex",
            "base_currency": "TST",
            "quote_currency": "USD",
            "tick_size": 0.0001,
            "lot_size": 100000.0
        })
    }

    /// Generate invalid instrument data for testing validation
    pub fn create_invalid_instrument() -> Value {
        json!({
            "symbol": "", // Invalid: empty symbol
            "name": "Test",
            "instrument_type": "invalid_type", // Invalid type
            "base_currency": "TOOLONG", // Invalid: too long
            "quote_currency": "USD",
            "tick_size": -1.0, // Invalid: negative
            "lot_size": 100000.0
        })
    }

    /// Generate test market tick data
    pub fn create_test_market_tick() -> Value {
        json!({
            "instrument_id": Uuid::new_v4(),
            "provider": "test_provider",
            "bid_price": 1.0850,
            "ask_price": 1.0852,
            "bid_size": 1000000.0,
            "ask_size": 1000000.0,
            "last_price": 1.0851,
            "volume": 5000000.0,
            "spread": 0.0002,
            "data_quality_score": 0.95
        })
    }
}

/// Test assertion helpers
pub mod assertions {
    use axum::http::StatusCode;
    use serde_json::Value;

    /// Assert that a response is a successful API response
    pub fn assert_api_success(json: &Value) {
        assert_eq!(json["success"], true, "API response should be successful");
        assert!(json["data"].is_object() || json["data"].is_array(), 
                "API response should have data field");
    }

    /// Assert that a response is an API error
    pub fn assert_api_error(json: &Value, expected_code: Option<&str>) {
        assert_eq!(json["success"], false, "API response should be an error");
        assert!(json["error"].is_object(), "API response should have error field");
        
        if let Some(code) = expected_code {
            assert_eq!(json["error"]["code"], code, 
                      "API error should have expected code");
        }
    }

    /// Assert that response has required fields
    pub fn assert_has_fields(json: &Value, fields: &[&str]) {
        for field in fields {
            assert!(json[field].is_object() || json[field].is_array() || 
                   json[field].is_string() || json[field].is_number() || 
                   json[field].is_boolean(),
                   "Response should have field: {}", field);
        }
    }

    /// Assert status code is in expected range
    pub fn assert_status_in_range(status: StatusCode, min: u16, max: u16) {
        let code = status.as_u16();
        assert!(code >= min && code <= max, 
                "Status code {} should be between {} and {}", code, min, max);
    }
}

/// API key constants for testing
pub mod api_keys {
    pub const ADMIN_KEY: &str = "demo-admin-key";
    pub const TRADER_KEY: &str = "demo-trader-key";
    pub const READONLY_KEY: &str = "demo-readonly-key";
    pub const INVALID_KEY: &str = "invalid-test-key";
}

/// Common test endpoints
pub mod endpoints {
    pub const HEALTH: &str = "/health";
    pub const STATUS: &str = "/status";
    pub const METRICS: &str = "/metrics";
    pub const LIVENESS: &str = "/health/liveness";
    pub const READINESS: &str = "/health/readiness";
    
    pub const INSTRUMENTS: &str = "/api/v1/instruments";
    pub const MARKET_DATA_LATEST: &str = "/api/v1/market-data/latest";
    pub const MARKET_DATA_TICKS: &str = "/api/v1/market-data/ticks";
    pub const MARKET_DATA_OHLC: &str = "/api/v1/market-data/ohlc";
    pub const MARKET_DATA_STATS: &str = "/api/v1/market-data/stats";
}

/// Rate limiting test helpers
pub mod rate_limiting {
    use std::time::Duration;
    use tokio::time::sleep;

    /// Wait for rate limit window to reset
    pub async fn wait_for_rate_limit_reset() {
        sleep(Duration::from_secs(61)).await; // Wait just over a minute
    }

    /// Make rapid requests to test rate limiting
    pub async fn make_rapid_requests<F, Fut>(request_fn: F, count: usize) -> Vec<axum::http::StatusCode>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = axum::http::StatusCode>,
    {
        let mut results = Vec::new();
        
        for _ in 0..count {
            let status = request_fn().await;
            results.push(status);
            
            // Small delay to avoid overwhelming the system
            sleep(Duration::from_millis(10)).await;
        }
        
        results
    }
}
