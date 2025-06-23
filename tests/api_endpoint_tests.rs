// API Endpoint Integration Tests
// Run with: cargo test --test api_endpoint_tests

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, Method};
use tower::util::ServiceExt;
use serde_json::{json, Value};

mod common;
use common::*;
use common::api_keys::*;
use common::endpoints::*;
use common::assertions::*;
use common::test_data::*;

/// Test all health and monitoring endpoints
#[tokio::test]
async fn test_health_endpoints() {
    let app = setup_test_app().await;

    let health_endpoints = vec![
        (HEALTH, "health check"),
        (STATUS, "system status"),
        (METRICS, "metrics"),
        (LIVENESS, "liveness probe"),
        (READINESS, "readiness probe"),
    ];

    for (endpoint, description) in health_endpoints {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(endpoint)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Health endpoints should be accessible without auth
        let status = response.status();
        assert!(
            status == StatusCode::OK ||
            status == StatusCode::SERVICE_UNAVAILABLE,
            "{} endpoint should return 200 or 503: {}", description, endpoint
        );

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();

        if !body.is_empty() {
            let json: Value = serde_json::from_slice(&body).unwrap();

            if status == StatusCode::OK {
                assert_api_success(&json);
            }
            
            println!("{} response: {}", description, json);
        }
    }
}

/// Test instruments API endpoints
#[tokio::test]
async fn test_instruments_endpoints() {
    let app = setup_test_app().await;

    // Test GET /api/v1/instruments
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(INSTRUMENTS)
                .header("Authorization", format!("Bearer {}", ADMIN_KEY))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should not be auth error
    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
    assert_ne!(response.status(), StatusCode::FORBIDDEN);

    println!("GET instruments status: {}", response.status());

    // Test POST /api/v1/instruments (create)
    let instrument_data = create_test_instrument();
    
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(INSTRUMENTS)
                .header("Authorization", format!("Bearer {}", ADMIN_KEY))
                .header("Content-Type", "application/json")
                .body(Body::from(instrument_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
    assert_ne!(response.status(), StatusCode::FORBIDDEN);
    
    println!("POST instruments status: {}", response.status());

    // If creation was successful, test GET specific instrument
    if response.status() == StatusCode::OK || response.status() == StatusCode::CREATED {
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        
        if !body.is_empty() {
            let json: Value = serde_json::from_slice(&body).unwrap();
            
            if let Some(instrument_id) = json["data"]["id"].as_str() {
                // Test GET /api/v1/instruments/{id}
                let response = app
                    .clone()
                    .oneshot(
                        Request::builder()
                            .uri(&format!("{}/{}", INSTRUMENTS, instrument_id))
                            .header("Authorization", format!("Bearer {}", READONLY_KEY))
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
                println!("GET specific instrument status: {}", response.status());
            }
        }
    }
}

/// Test market data API endpoints
#[tokio::test]
async fn test_market_data_endpoints() {
    let app = setup_test_app().await;

    let market_data_endpoints = vec![
        (MARKET_DATA_LATEST, "latest ticks"),
        (MARKET_DATA_TICKS, "market ticks"),
        (MARKET_DATA_STATS, "market statistics"),
    ];

    for (endpoint, description) in market_data_endpoints {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(endpoint)
                    .header("Authorization", format!("Bearer {}", READONLY_KEY))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_ne!(response.status(), StatusCode::UNAUTHORIZED, 
                  "{} should not return 401", description);
        assert_ne!(response.status(), StatusCode::FORBIDDEN,
                  "{} should not return 403", description);

        println!("{} endpoint status: {}", description, response.status());
    }

    // Test OHLC endpoint with parameters
    let ohlc_url = format!("{}?instrument_id={}&interval=1h", 
                          MARKET_DATA_OHLC, 
                          uuid::Uuid::new_v4());
    
    let response = app
        .oneshot(
            Request::builder()
                .uri(&ohlc_url)
                .header("Authorization", format!("Bearer {}", READONLY_KEY))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
    println!("OHLC endpoint status: {}", response.status());
}

/// Test request validation
#[tokio::test]
async fn test_request_validation() {
    let app = setup_test_app().await;

    // Test invalid JSON
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(INSTRUMENTS)
                .header("Authorization", format!("Bearer {}", ADMIN_KEY))
                .header("Content-Type", "application/json")
                .body(Body::from("invalid json"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_status_in_range(response.status(), 400, 499);

    // Test missing required fields
    let incomplete_data = json!({
        "symbol": "TEST"
        // Missing required fields
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(INSTRUMENTS)
                .header("Authorization", format!("Bearer {}", ADMIN_KEY))
                .header("Content-Type", "application/json")
                .body(Body::from(incomplete_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_status_in_range(response.status(), 400, 499);

    // Test invalid field values
    let invalid_data = create_invalid_instrument();

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(INSTRUMENTS)
                .header("Authorization", format!("Bearer {}", ADMIN_KEY))
                .header("Content-Type", "application/json")
                .body(Body::from(invalid_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_status_in_range(response.status(), 400, 499);
}

/// Test HTTP methods on endpoints
#[tokio::test]
async fn test_http_methods() {
    let app = setup_test_app().await;

    // Test unsupported methods on health endpoint
    let unsupported_methods = vec![
        Method::POST,
        Method::PUT,
        Method::DELETE,
        Method::PATCH,
    ];

    for method in unsupported_methods {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method.clone())
                    .uri(HEALTH)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should return 405 Method Not Allowed or 404 Not Found
        assert!(
            response.status() == StatusCode::METHOD_NOT_ALLOWED ||
            response.status() == StatusCode::NOT_FOUND,
            "Unsupported method {:?} should return 405 or 404", method
        );
    }
}

/// Test query parameters
#[tokio::test]
async fn test_query_parameters() {
    let app = setup_test_app().await;

    // Test instruments with query parameters
    let query_params = vec![
        "?limit=10",
        "?offset=0",
        "?instrument_type=forex",
        "?is_active=true",
        "?limit=5&offset=10",
    ];

    for params in query_params {
        let url = format!("{}{}", INSTRUMENTS, params);
        
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(&url)
                    .header("Authorization", format!("Bearer {}", READONLY_KEY))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
        println!("Query params {} status: {}", params, response.status());
    }

    // Test market data with query parameters
    let market_query_params = vec![
        "?limit=50",
        "?provider=alpha_vantage",
        "?min_quality=0.8",
    ];

    for params in market_query_params {
        let url = format!("{}{}", MARKET_DATA_TICKS, params);
        
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(&url)
                    .header("Authorization", format!("Bearer {}", READONLY_KEY))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
        println!("Market data query {} status: {}", params, response.status());
    }
}

/// Test response headers
#[tokio::test]
async fn test_response_headers() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri(HEALTH)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let headers = response.headers();

    // Check for required headers
    assert!(headers.contains_key("content-type"));
    
    // Check for security headers
    let security_headers = vec![
        "x-content-type-options",
        "x-frame-options", 
        "x-xss-protection",
    ];

    for header in security_headers {
        if headers.contains_key(header) {
            println!("Security header found: {}", header);
        }
    }

    // Check content type for JSON endpoints
    if let Some(content_type) = headers.get("content-type") {
        let content_type_str = content_type.to_str().unwrap();
        if content_type_str.contains("application/json") {
            println!("JSON content type confirmed");
        }
    }
}

/// Test error response format consistency
#[tokio::test]
async fn test_error_response_format() {
    let app = setup_test_app().await;

    // Test 404 error
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/nonexistent")
                .header("Authorization", format!("Bearer {}", ADMIN_KEY))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // Test 401 error
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(INSTRUMENTS)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Test 403 error
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(INSTRUMENTS)
                .header("Authorization", format!("Bearer {}", READONLY_KEY))
                .header("Content-Type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

/// Test large request handling
#[tokio::test]
async fn test_large_request_handling() {
    let app = setup_test_app().await;

    // Create a large JSON payload
    let large_data = json!({
        "symbol": "TEST",
        "name": "A".repeat(10000), // Very long name
        "instrument_type": "forex",
        "base_currency": "USD",
        "quote_currency": "EUR",
        "tick_size": 0.0001,
        "lot_size": 100000.0
    });

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(INSTRUMENTS)
                .header("Authorization", format!("Bearer {}", ADMIN_KEY))
                .header("Content-Type", "application/json")
                .body(Body::from(large_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should handle large requests gracefully
    assert!(response.status().is_client_error() || response.status().is_server_error());
}
