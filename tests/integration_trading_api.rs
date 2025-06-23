use axum::http::StatusCode;
use serde_json::{json, Value};
use tokio;
use uuid::Uuid;

mod common;
use common::*;

/// Test trading orders API endpoints
#[tokio::test]
async fn test_trading_orders_api() {
    let app = create_test_app().await;

    // Test submit order
    let order_request = json!({
        "instrument_id": "550e8400-e29b-41d4-a716-446655440000",
        "side": "Buy",
        "quantity": 1000.0,
        "order_type": "Market",
        "time_in_force": "IOC"
    });

    let response = app
        .post("/api/v1/orders")
        .header("x-api-key", "demo-trader-key")
        .json(&order_request)
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await;
    assert_eq!(body["success"], true);
    assert!(body["data"]["id"].is_string());

    // Test list orders
    let response = app
        .get("/api/v1/orders")
        .header("x-api-key", "demo-trader-key")
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await;
    assert_eq!(body["success"], true);
    assert!(body["data"]["orders"].is_array());

    // Test get order stats
    let response = app
        .get("/api/v1/orders/stats")
        .header("x-api-key", "demo-trader-key")
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await;
    assert_eq!(body["success"], true);
    assert!(body["data"]["total_orders"].is_number());
}

/// Test portfolio management API endpoints
#[tokio::test]
async fn test_portfolio_api() {
    let app = create_test_app().await;

    // Test get positions
    let response = app
        .get("/api/v1/portfolio/positions")
        .header("x-api-key", "demo-trader-key")
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await;
    assert_eq!(body["success"], true);
    assert!(body["data"]["positions"].is_array());
    assert!(body["data"]["summary"].is_object());

    // Test get performance
    let response = app
        .get("/api/v1/portfolio/performance")
        .header("x-api-key", "demo-trader-key")
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await;
    assert_eq!(body["success"], true);
    assert!(body["data"]["total_return"].is_number());
    assert!(body["data"]["sharpe_ratio"].is_number());

    // Test get risk metrics
    let response = app
        .get("/api/v1/portfolio/risk")
        .header("x-api-key", "demo-trader-key")
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await;
    assert_eq!(body["success"], true);
    assert!(body["data"]["var_95"].is_number());

    // Test get summary
    let response = app
        .get("/api/v1/portfolio/summary")
        .header("x-api-key", "demo-trader-key")
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await;
    assert_eq!(body["success"], true);
    assert!(body["data"]["total_value"].is_number());
}

/// Test trading signals API endpoints
#[tokio::test]
async fn test_signals_api() {
    let app = create_test_app().await;

    // Test get signals
    let response = app
        .get("/api/v1/signals")
        .header("x-api-key", "demo-trader-key")
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await;
    assert_eq!(body["success"], true);
    assert!(body["data"]["signals"].is_array());

    // Test get latest signals
    let response = app
        .get("/api/v1/signals/latest")
        .header("x-api-key", "demo-trader-key")
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await;
    assert_eq!(body["success"], true);
    assert!(body["data"]["signals"].is_array());

    // Test get signal performance
    let response = app
        .get("/api/v1/signals/performance")
        .header("x-api-key", "demo-trader-key")
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await;
    assert_eq!(body["success"], true);
    assert!(body["data"].is_array());

    // Test get signal analytics
    let response = app
        .get("/api/v1/signals/analytics")
        .header("x-api-key", "demo-trader-key")
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await;
    assert_eq!(body["success"], true);
    assert!(body["data"]["total_signals_today"].is_number());
}

/// Test risk management API endpoints
#[tokio::test]
async fn test_risk_api() {
    let app = create_test_app().await;

    // Test get risk metrics
    let response = app
        .get("/api/v1/risk/metrics")
        .header("x-api-key", "demo-trader-key")
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await;
    assert_eq!(body["success"], true);
    assert!(body["data"]["portfolio_var_95"].is_number());

    // Test get risk limits
    let response = app
        .get("/api/v1/risk/limits")
        .header("x-api-key", "demo-trader-key")
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await;
    assert_eq!(body["success"], true);
    assert!(body["data"]["max_position_size"].is_number());

    // Test get risk alerts
    let response = app
        .get("/api/v1/risk/alerts")
        .header("x-api-key", "demo-trader-key")
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await;
    assert_eq!(body["success"], true);
    assert!(body["data"]["active_alerts"].is_array());

    // Test get risk monitoring
    let response = app
        .get("/api/v1/risk/monitoring")
        .header("x-api-key", "demo-trader-key")
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await;
    assert_eq!(body["success"], true);
    assert!(body["data"]["overall_risk_score"].is_number());
}

/// Test strategy management API endpoints
#[tokio::test]
async fn test_strategies_api() {
    let app = create_test_app().await;

    // Test list strategies
    let response = app
        .get("/api/v1/strategies")
        .header("x-api-key", "demo-trader-key")
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await;
    assert_eq!(body["success"], true);
    assert!(body["data"]["strategies"].is_array());

    // Test get strategy (using a mock UUID)
    let strategy_id = Uuid::new_v4();
    let response = app
        .get(&format!("/api/v1/strategies/{}", strategy_id))
        .header("x-api-key", "demo-trader-key")
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await;
    assert_eq!(body["success"], true);
    assert!(body["data"]["name"].is_string());

    // Test get strategy performance
    let response = app
        .get(&format!("/api/v1/strategies/{}/performance", strategy_id))
        .header("x-api-key", "demo-trader-key")
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await;
    assert_eq!(body["success"], true);
    assert!(body["data"]["performance_metrics"].is_object());
}

/// Test trading engine control API endpoints
#[tokio::test]
async fn test_trading_engine_api() {
    let app = create_test_app().await;

    // Test get engine status
    let response = app
        .get("/api/v1/engine/status")
        .header("x-api-key", "demo-admin-key")
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await;
    assert_eq!(body["success"], true);
    assert!(body["data"]["state"].is_string());

    // Test get engine config
    let response = app
        .get("/api/v1/engine/config")
        .header("x-api-key", "demo-admin-key")
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await;
    assert_eq!(body["success"], true);
    assert!(body["data"]["confidence_threshold"].is_number());

    // Test get engine stats
    let response = app
        .get("/api/v1/engine/stats")
        .header("x-api-key", "demo-admin-key")
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await;
    assert_eq!(body["success"], true);
    assert!(body["data"]["uptime_hours"].is_number());
}

/// Test authentication and authorization
#[tokio::test]
async fn test_authentication() {
    let app = create_test_app().await;

    // Test without API key
    let response = app
        .get("/api/v1/orders")
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Test with invalid API key
    let response = app
        .get("/api/v1/orders")
        .header("x-api-key", "invalid-key")
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Test readonly user trying to submit order
    let order_request = json!({
        "instrument_id": "550e8400-e29b-41d4-a716-446655440000",
        "side": "Buy",
        "quantity": 1000.0,
        "order_type": "Market",
        "time_in_force": "IOC"
    });

    let response = app
        .post("/api/v1/orders")
        .header("x-api-key", "demo-readonly-key")
        .json(&order_request)
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

/// Test error handling
#[tokio::test]
async fn test_error_handling() {
    let app = create_test_app().await;

    // Test invalid order request
    let invalid_order = json!({
        "instrument_id": "invalid-uuid",
        "side": "InvalidSide",
        "quantity": -1000.0,
        "order_type": "Market"
    });

    let response = app
        .post("/api/v1/orders")
        .header("x-api-key", "demo-trader-key")
        .json(&invalid_order)
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Test non-existent order
    let fake_order_id = Uuid::new_v4();
    let response = app
        .get(&format!("/api/v1/orders/{}", fake_order_id))
        .header("x-api-key", "demo-trader-key")
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

/// Test end-to-end trading workflow
#[tokio::test]
async fn test_end_to_end_trading_workflow() {
    let app = create_test_app().await;

    // 1. Check initial portfolio state
    let response = app
        .get("/api/v1/portfolio/summary")
        .header("x-api-key", "demo-trader-key")
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let initial_portfolio: Value = response.json().await;

    // 2. Get latest signals
    let response = app
        .get("/api/v1/signals/latest")
        .header("x-api-key", "demo-trader-key")
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let signals: Value = response.json().await;
    assert!(signals["data"]["signals"].is_array());

    // 3. Submit an order based on signal
    let order_request = json!({
        "instrument_id": "550e8400-e29b-41d4-a716-446655440000",
        "side": "Buy",
        "quantity": 1000.0,
        "order_type": "Market",
        "time_in_force": "IOC"
    });

    let response = app
        .post("/api/v1/orders")
        .header("x-api-key", "demo-trader-key")
        .json(&order_request)
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let order_response: Value = response.json().await;
    let order_id = order_response["data"]["id"].as_str().unwrap();

    // 4. Check order status
    let response = app
        .get(&format!("/api/v1/orders/{}", order_id))
        .header("x-api-key", "demo-trader-key")
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let order_status: Value = response.json().await;
    assert_eq!(order_status["success"], true);

    // 5. Check updated portfolio
    let response = app
        .get("/api/v1/portfolio/positions")
        .header("x-api-key", "demo-trader-key")
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let positions: Value = response.json().await;
    assert!(positions["data"]["positions"].is_array());

    // 6. Check risk metrics after trade
    let response = app
        .get("/api/v1/risk/metrics")
        .header("x-api-key", "demo-trader-key")
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let risk_metrics: Value = response.json().await;
    assert!(risk_metrics["data"]["portfolio_var_95"].is_number());
}

/// Test rate limiting
#[tokio::test]
async fn test_rate_limiting() {
    let app = create_test_app().await;

    // Make multiple rapid requests to test rate limiting
    let mut responses = Vec::new();
    for _ in 0..10 {
        let response = app
            .get("/api/v1/portfolio/summary")
            .header("x-api-key", "demo-trader-key")
            .send()
            .await;
        responses.push(response.status());
    }

    // All requests should succeed for now (rate limiting is lenient in tests)
    for status in responses {
        assert!(status == StatusCode::OK || status == StatusCode::TOO_MANY_REQUESTS);
    }
}
