// Production Monitoring Integration Test
// Tests the comprehensive production monitoring system

use pantherswap_edge::monitoring::{ProductionMonitor, ProductionMonitoringConfig};
use pantherswap_edge::ai::monitoring::create_ai_performance_monitor;
use pantherswap_edge::database::{Database, health_monitor::DatabaseHealthMonitor};
use pantherswap_edge::trading::engine::{TradingEngine, TradingEngineConfig};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_production_monitoring_integration() {
    // Initialize test database
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/pantherswap_test".to_string());
    
    let database = Database::new(&database_url).await.expect("Failed to connect to test database");
    
    // Initialize components for monitoring
    let ai_monitor = Arc::new(create_ai_performance_monitor(database.clone()));
    let db_monitor = Arc::new(DatabaseHealthMonitor::with_defaults(database.pool.clone()));
    
    let trading_config = TradingEngineConfig::default();
    let trading_engine = TradingEngine::new(trading_config, database.clone()).await
        .expect("Failed to create trading engine");
    let trading_engine_arc = Arc::new(RwLock::new(trading_engine));
    
    // Create production monitor
    let monitoring_config = ProductionMonitoringConfig {
        health_check_interval_seconds: 1, // Fast for testing
        metrics_collection_interval_seconds: 1,
        alert_aggregation_window_seconds: 5,
        enable_auto_recovery: true,
        enable_failover: false, // Disable for testing
        max_consecutive_failures: 2,
        system_health_threshold: 0.8,
        enable_performance_profiling: true,
        enable_predictive_alerts: false, // Disable for testing
    };
    
    let production_monitor = Arc::new(ProductionMonitor::new(
        monitoring_config,
        ai_monitor,
        db_monitor,
        trading_engine_arc,
    ));
    
    // Start monitoring
    production_monitor.start_monitoring().await
        .expect("Failed to start production monitoring");
    
    println!("✅ Production monitoring started successfully");
    
    // Wait for a few monitoring cycles
    sleep(Duration::from_secs(3)).await;
    
    // Test health status retrieval
    let system_health = production_monitor.get_system_health().await;
    println!("📊 System Health Score: {:.2}%", system_health.overall_health_score * 100.0);
    
    assert!(system_health.overall_health_score > 0.0, "System health score should be positive");
    assert!(system_health.overall_health_score <= 1.0, "System health score should not exceed 100%");
    
    // Test component health retrieval
    let component_health = production_monitor.get_all_component_health().await;
    println!("🔧 Component Health:");
    for (component, health) in &component_health {
        println!("  - {}: {:.2}% ({:?})", component, health.health_score * 100.0, health.status);
    }
    
    assert!(!component_health.is_empty(), "Should have component health data");
    assert!(component_health.contains_key("database"), "Should monitor database health");
    assert!(component_health.contains_key("ai_engine"), "Should monitor AI engine health");
    assert!(component_health.contains_key("trading_engine"), "Should monitor trading engine health");
    
    // Test active alerts
    let active_alerts = production_monitor.get_active_alerts().await;
    println!("🚨 Active Alerts: {}", active_alerts.len());
    
    // Test performance metrics
    let performance_metrics = production_monitor.get_performance_metrics().await;
    println!("📈 Performance Metrics History: {} entries", performance_metrics.len());
    
    println!("✅ Production monitoring test completed successfully");
}

#[tokio::test]
async fn test_health_check_endpoints() {
    use pantherswap_edge::api::{create_app, AppState};
    use axum::http::{Request, StatusCode};
    use axum::body::Body;
    use tower::ServiceExt;
    
    // Setup test environment (simplified)
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/pantherswap_test".to_string());
    
    let database = Database::new(&database_url).await.expect("Failed to connect to test database");
    
    // Create minimal components for testing
    let ai_monitor = Arc::new(create_ai_performance_monitor(database.clone()));
    let db_monitor = Arc::new(DatabaseHealthMonitor::with_defaults(database.pool.clone()));
    
    let trading_config = TradingEngineConfig::default();
    let trading_engine = TradingEngine::new(trading_config, database.clone()).await
        .expect("Failed to create trading engine");
    let trading_engine_arc = Arc::new(RwLock::new(trading_engine));
    
    let monitoring_config = ProductionMonitoringConfig::default();
    let production_monitor = Arc::new(ProductionMonitor::new(
        monitoring_config,
        ai_monitor,
        db_monitor,
        trading_engine_arc.clone(),
    ));
    
    // Create AI engine for AppState
    let ai_engine = pantherswap_edge::ai::AIEngine::new(database.clone()).await
        .expect("Failed to create AI engine");
    
    let app_state = AppState {
        database,
        trading_engine: trading_engine_arc,
        ai_engine: Arc::new(tokio::sync::Mutex::new(ai_engine)),
        production_monitor,
    };
    
    let app = create_app(app_state).await;
    
    // Test basic health endpoint
    let request = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    println!("✅ /health endpoint working");
    
    // Test liveness endpoint
    let request = Request::builder()
        .uri("/health/liveness")
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    println!("✅ /health/liveness endpoint working");
    
    // Test readiness endpoint
    let request = Request::builder()
        .uri("/health/readiness")
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    println!("✅ /health/readiness endpoint working");
    
    // Test system status endpoint
    let request = Request::builder()
        .uri("/status")
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    println!("✅ /status endpoint working");
    
    // Test production monitoring endpoint
    let request = Request::builder()
        .uri("/monitoring")
        .body(Body::empty())
        .unwrap();
    
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    println!("✅ /monitoring endpoint working");
    
    println!("✅ All health check endpoints test completed successfully");
}
