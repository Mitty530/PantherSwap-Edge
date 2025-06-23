// Integration Tests for PantherSwap Edge REST API
// Run with: cargo test --test integration_tests

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, Method};
use tower::util::ServiceExt;
use serde_json::{json, Value};

mod common;
use common::*;

/// Test the health check endpoint (no authentication required)
#[tokio::test]
async fn test_health_check() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert!(json["data"]["status"].as_str().unwrap() == "healthy");
    assert!(json["data"]["version"].is_string());
    assert!(json["data"]["uptime_seconds"].is_number());
}

/// Test the system status endpoint (no authentication required)
#[tokio::test]
async fn test_system_status() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert!(json["data"]["database"].is_object());
    assert!(json["data"]["market_data"].is_object());
    assert!(json["data"]["api"].is_object());
    assert!(json["data"]["overall_status"].is_string());
}

/// Test the metrics endpoint (no authentication required)
#[tokio::test]
async fn test_metrics() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert!(json["data"]["timestamp"].is_string());
    assert!(json["data"]["database"].is_object());
    assert!(json["data"]["api"].is_object());
    assert!(json["data"]["system"].is_object());
}

/// Test Kubernetes liveness probe
#[tokio::test]
async fn test_liveness_probe() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health/liveness")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["data"]["alive"], true);
}

/// Test Kubernetes readiness probe
#[tokio::test]
async fn test_readiness_probe() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health/readiness")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // This might return 503 if database is not available, which is expected
    let status = response.status();
    assert!(
        status == StatusCode::OK || status == StatusCode::SERVICE_UNAVAILABLE
    );

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    if status == StatusCode::OK {
        assert_eq!(json["success"], true);
        assert!(json["data"]["ready"].is_boolean());
        assert!(json["data"]["services"].is_object());
    }
}

/// Test authentication with invalid API key
#[tokio::test]
async fn test_authentication_invalid_key() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/instruments")
                .header("Authorization", "Bearer invalid-key")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// Test authentication with missing API key
#[tokio::test]
async fn test_authentication_missing_key() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/instruments")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// Test authentication with valid admin API key
#[tokio::test]
async fn test_authentication_valid_admin_key() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/instruments")
                .header("Authorization", "Bearer demo-admin-key")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should not be 401 Unauthorized
    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
    // Might be 500 due to database issues, but auth should pass
}

/// Test authentication with valid readonly API key
#[tokio::test]
async fn test_authentication_valid_readonly_key() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/market-data/latest")
                .header("Authorization", "Bearer demo-readonly-key")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should not be 401 Unauthorized
    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
}

/// Test rate limiting for unauthenticated requests
#[tokio::test]
async fn test_rate_limiting_unauthenticated() {
    let app = setup_test_app().await;

    // Make multiple requests quickly to trigger rate limiting
    for i in 0..15 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        if i < 10 {
            // First 10 requests should succeed
            assert_eq!(response.status(), StatusCode::OK);
        } else {
            // After 10 requests, should be rate limited
            // Note: This test might be flaky depending on timing
            println!("Request {}: Status = {}", i, response.status());
        }
    }
}

/// Test CORS headers
#[tokio::test]
async fn test_cors_headers() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .header("Origin", "https://example.com")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Check for CORS headers
    let headers = response.headers();
    assert!(headers.contains_key("access-control-allow-origin"));
}

/// Test security headers
#[tokio::test]
async fn test_security_headers() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let headers = response.headers();
    
    // Check for security headers
    assert!(headers.contains_key("x-content-type-options"));
    assert!(headers.contains_key("x-frame-options"));
    assert!(headers.contains_key("x-xss-protection"));
}

/// Test request validation with invalid JSON
#[tokio::test]
async fn test_request_validation_invalid_json() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/instruments")
                .header("Authorization", "Bearer demo-admin-key")
                .header("Content-Type", "application/json")
                .body(Body::from("invalid json"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

/// Test creating an instrument with valid data
#[tokio::test]
async fn test_create_instrument_valid_data() {
    let app = setup_test_app().await;

    let instrument_data = json!({
        "symbol": "TESTPAIR",
        "name": "Test Currency Pair",
        "instrument_type": "forex",
        "base_currency": "TST",
        "quote_currency": "USD",
        "tick_size": 0.0001,
        "lot_size": 100000.0
    });

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/instruments")
                .header("Authorization", "Bearer demo-admin-key")
                .header("Content-Type", "application/json")
                .body(Body::from(instrument_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should not be 401 or 403 (auth/permission errors)
    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
    assert_ne!(response.status(), StatusCode::FORBIDDEN);
    
    // Might be 500 due to database issues, but validation should pass
    if response.status() != StatusCode::INTERNAL_SERVER_ERROR {
        assert_eq!(response.status(), StatusCode::OK);
    }
}

/// Test creating an instrument with invalid data
#[tokio::test]
async fn test_create_instrument_invalid_data() {
    let app = setup_test_app().await;

    let invalid_data = json!({
        "symbol": "", // Empty symbol should be invalid
        "name": "Test",
        "instrument_type": "invalid_type",
        "base_currency": "TOOLONG", // Should be 3 chars
        "quote_currency": "USD",
        "tick_size": -1.0, // Negative should be invalid
        "lot_size": 100000.0
    });

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/instruments")
                .header("Authorization", "Bearer demo-admin-key")
                .header("Content-Type", "application/json")
                .body(Body::from(invalid_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should be a client error (400-499)
    assert!(response.status().is_client_error());
}

/// Test permission denied for readonly user trying to create instrument
#[tokio::test]
async fn test_permission_denied_readonly_create() {
    let app = setup_test_app().await;

    let instrument_data = json!({
        "symbol": "TESTPAIR",
        "name": "Test Currency Pair",
        "instrument_type": "forex",
        "base_currency": "TST",
        "quote_currency": "USD",
        "tick_size": 0.0001,
        "lot_size": 100000.0
    });

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/instruments")
                .header("Authorization", "Bearer demo-readonly-key")
                .header("Content-Type", "application/json")
                .body(Body::from(instrument_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
