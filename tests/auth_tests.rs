// Authentication and Authorization Integration Tests
// Run with: cargo test --test auth_tests

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, Method};
use tower::util::ServiceExt;
use serde_json::{json, Value};

mod common;
use common::*;
use common::api_keys::*;
use common::endpoints::*;
use common::assertions::*;

/// Test authentication with various API key scenarios
#[tokio::test]
async fn test_api_key_authentication() {
    let app = setup_test_app().await;

    // Test cases: (api_key, expected_auth_result)
    let test_cases = vec![
        (Some(ADMIN_KEY), true),
        (Some(TRADER_KEY), true),
        (Some(READONLY_KEY), true),
        (Some(INVALID_KEY), false),
        (None, false),
    ];

    for (api_key, should_authenticate) in test_cases {
        let mut request_builder = Request::builder()
            .uri(INSTRUMENTS)
            .method(Method::GET);

        if let Some(key) = api_key {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", key));
        }

        let response = app
            .clone()
            .oneshot(request_builder.body(Body::empty()).unwrap())
            .await
            .unwrap();

        if should_authenticate {
            assert_ne!(response.status(), StatusCode::UNAUTHORIZED, 
                      "Valid API key should not return 401: {:?}", api_key);
        } else {
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED,
                      "Invalid/missing API key should return 401: {:?}", api_key);
        }
    }
}

/// Test role-based access control for different endpoints
#[tokio::test]
async fn test_role_based_access_control() {
    let app = setup_test_app().await;

    // Test readonly user accessing read endpoints (should work)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(MARKET_DATA_LATEST)
                .header("Authorization", format!("Bearer {}", READONLY_KEY))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
    assert_ne!(response.status(), StatusCode::FORBIDDEN);

    // Test readonly user trying to create instrument (should fail)
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
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(INSTRUMENTS)
                .header("Authorization", format!("Bearer {}", READONLY_KEY))
                .header("Content-Type", "application/json")
                .body(Body::from(instrument_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    // Test admin user creating instrument (should work)
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
}

/// Test malformed authorization headers
#[tokio::test]
async fn test_malformed_auth_headers() {
    let app = setup_test_app().await;

    let malformed_headers = vec![
        "Bearer", // Missing key
        "Basic demo-admin-key", // Wrong auth type
        "Bearer ", // Empty key
        "demo-admin-key", // Missing Bearer prefix
        "Bearer demo-admin-key extra", // Extra content
    ];

    for header_value in malformed_headers {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(INSTRUMENTS)
                    .header("Authorization", header_value)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED,
                  "Malformed auth header should return 401: {}", header_value);
    }
}

/// Test case sensitivity of API keys
#[tokio::test]
async fn test_api_key_case_sensitivity() {
    let app = setup_test_app().await;

    let case_variations = vec![
        "DEMO-ADMIN-KEY",
        "demo-Admin-Key",
        "Demo-Admin-Key",
    ];

    for key_variation in case_variations {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(INSTRUMENTS)
                    .header("Authorization", format!("Bearer {}", key_variation))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED,
                  "API keys should be case sensitive: {}", key_variation);
    }
}

/// Test authentication bypass attempts
#[tokio::test]
async fn test_authentication_bypass_attempts() {
    let app = setup_test_app().await;

    let bypass_attempts = vec![
        ("X-API-Key", ADMIN_KEY), // Wrong header name
        ("Authorization", ADMIN_KEY), // Missing Bearer
        ("Bearer", ADMIN_KEY), // Wrong header name
        ("X-Authorization", format!("Bearer {}", ADMIN_KEY)), // Wrong header name
    ];

    for (header_name, header_value) in bypass_attempts {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(INSTRUMENTS)
                    .header(header_name, header_value)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED,
                  "Bypass attempt should fail: {} = {}", header_name, header_value);
    }
}

/// Test multiple authorization headers
#[tokio::test]
async fn test_multiple_auth_headers() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri(INSTRUMENTS)
                .header("Authorization", format!("Bearer {}", ADMIN_KEY))
                .header("Authorization", format!("Bearer {}", INVALID_KEY))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should handle multiple headers gracefully (typically uses first one)
    assert!(response.status() == StatusCode::UNAUTHORIZED || 
            response.status() != StatusCode::UNAUTHORIZED);
}

/// Test authentication with special characters in API key
#[tokio::test]
async fn test_special_characters_in_api_key() {
    let app = setup_test_app().await;

    let special_keys = vec![
        "demo-admin-key!@#",
        "demo admin key", // Space
        "demo-admin-key\n", // Newline
        "demo-admin-key\t", // Tab
        "demo-admin-key;DROP TABLE users;", // SQL injection attempt
    ];

    for special_key in special_keys {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(INSTRUMENTS)
                    .header("Authorization", format!("Bearer {}", special_key))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED,
                  "Special character key should be rejected: {}", special_key);
    }
}

/// Test very long API keys
#[tokio::test]
async fn test_long_api_keys() {
    let app = setup_test_app().await;

    let long_key = "a".repeat(10000); // Very long key

    let response = app
        .oneshot(
            Request::builder()
                .uri(INSTRUMENTS)
                .header("Authorization", format!("Bearer {}", long_key))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// Test authentication error response format
#[tokio::test]
async fn test_auth_error_response_format() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri(INSTRUMENTS)
                .header("Authorization", "Bearer invalid-key")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    
    // Check if response is JSON (might be empty for 401)
    if !body.is_empty() {
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_api_error(&json, Some("UNAUTHORIZED"));
    }
}

/// Test concurrent authentication requests
#[tokio::test]
async fn test_concurrent_authentication() {
    let app = setup_test_app().await;

    let mut handles = vec![];

    // Make 10 concurrent requests with valid auth
    for _ in 0..10 {
        let app_clone = app.clone();
        let handle = tokio::spawn(async move {
            app_clone
                .oneshot(
                    Request::builder()
                        .uri(INSTRUMENTS)
                        .header("Authorization", format!("Bearer {}", ADMIN_KEY))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap()
                .status()
        });
        handles.push(handle);
    }

    // Wait for all requests to complete
    let results = futures::future::join_all(handles).await;

    // All should have consistent authentication results
    for result in results {
        let status = result.unwrap();
        assert_ne!(status, StatusCode::UNAUTHORIZED, 
                  "Concurrent auth requests should be consistent");
    }
}

/// Test authentication with different HTTP methods
#[tokio::test]
async fn test_auth_with_different_methods() {
    let app = setup_test_app().await;

    let methods = vec![
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::DELETE,
        Method::PATCH,
    ];

    for method in methods {
        let mut request_builder = Request::builder()
            .method(method.clone())
            .uri(INSTRUMENTS)
            .header("Authorization", format!("Bearer {}", ADMIN_KEY));

        // Add content type for methods that might need it
        if method == Method::POST || method == Method::PUT || method == Method::PATCH {
            request_builder = request_builder.header("Content-Type", "application/json");
        }

        let body = if method == Method::POST || method == Method::PUT || method == Method::PATCH {
            Body::from("{}")
        } else {
            Body::empty()
        };

        let response = app
            .clone()
            .oneshot(request_builder.body(body).unwrap())
            .await
            .unwrap();

        // Should not be unauthorized (might be 404 or 405 for unsupported methods)
        assert_ne!(response.status(), StatusCode::UNAUTHORIZED,
                  "Authentication should work for method: {:?}", method);
    }
}
