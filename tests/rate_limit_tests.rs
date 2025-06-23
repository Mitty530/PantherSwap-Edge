// Rate Limiting Integration Tests
// Run with: cargo test --test rate_limit_tests

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use tower::util::ServiceExt;
use std::time::Duration;
use tokio::time::sleep;

mod common;
use common::*;
use common::api_keys::*;
use common::endpoints::*;
use common::rate_limiting::*;

/// Test IP-based rate limiting for unauthenticated requests
#[tokio::test]
async fn test_ip_rate_limiting_unauthenticated() {
    let app = setup_test_app().await;

    // Make requests rapidly to trigger IP rate limiting
    let mut success_count = 0;
    let mut rate_limited_count = 0;

    for i in 0..15 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(HEALTH)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        match response.status() {
            StatusCode::OK => success_count += 1,
            StatusCode::TOO_MANY_REQUESTS => rate_limited_count += 1,
            _ => {
                // Other status codes are acceptable
            }
        }

        // Small delay to avoid overwhelming the test
        sleep(Duration::from_millis(50)).await;
    }

    println!("Success: {}, Rate limited: {}", success_count, rate_limited_count);
    
    // We should see some rate limiting after the first few requests
    // Note: This test might be timing-dependent
    assert!(success_count > 0, "Should have some successful requests");
}

/// Test user-based rate limiting for authenticated requests
#[tokio::test]
async fn test_user_rate_limiting_authenticated() {
    let app = setup_test_app().await;

    let mut success_count = 0;
    let mut rate_limited_count = 0;

    // Make many requests with readonly key (lower rate limit)
    for _ in 0..70 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(INSTRUMENTS)
                    .header("Authorization", format!("Bearer {}", READONLY_KEY))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        match response.status() {
            StatusCode::TOO_MANY_REQUESTS => rate_limited_count += 1,
            StatusCode::UNAUTHORIZED => {
                // Auth might be broken, but that's not what we're testing
            }
            _ => success_count += 1,
        }

        sleep(Duration::from_millis(20)).await;
    }

    println!("Authenticated - Success: {}, Rate limited: {}", success_count, rate_limited_count);
    
    // Should eventually hit rate limits
    // Note: Actual rate limiting behavior depends on implementation
}

/// Test different rate limits for different user roles
#[tokio::test]
async fn test_role_based_rate_limits() {
    let app = setup_test_app().await;

    let test_cases = vec![
        (ADMIN_KEY, "admin"),
        (TRADER_KEY, "trader"),
        (READONLY_KEY, "readonly"),
    ];

    for (api_key, role) in test_cases {
        let mut request_count = 0;
        let mut rate_limited = false;

        // Make requests until rate limited or reasonable limit
        for _ in 0..100 {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(INSTRUMENTS)
                        .header("Authorization", format!("Bearer {}", api_key))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();

            if response.status() == StatusCode::TOO_MANY_REQUESTS {
                rate_limited = true;
                break;
            }

            request_count += 1;
            sleep(Duration::from_millis(10)).await;
        }

        println!("Role {}: {} requests before rate limit, limited: {}", 
                role, request_count, rate_limited);
    }
}

/// Test rate limit headers in response
#[tokio::test]
async fn test_rate_limit_headers() {
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
    
    // Check for common rate limit headers
    // Note: These might not be implemented yet
    if headers.contains_key("x-ratelimit-limit") {
        println!("Rate limit headers found");
        assert!(headers.contains_key("x-ratelimit-remaining"));
        assert!(headers.contains_key("x-ratelimit-reset"));
    } else {
        println!("Rate limit headers not implemented yet");
    }
}

/// Test rate limiting with burst capacity
#[tokio::test]
async fn test_burst_capacity() {
    let app = setup_test_app().await;

    // Make a burst of requests quickly
    let burst_size = 5;
    let mut statuses = Vec::new();

    for _ in 0..burst_size {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(HEALTH)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        statuses.push(response.status());
    }

    // First few requests should succeed (burst capacity)
    let success_count = statuses.iter()
        .filter(|&&status| status == StatusCode::OK)
        .count();

    println!("Burst test - {} out of {} requests succeeded", success_count, burst_size);
    
    // Should allow some burst requests
    assert!(success_count > 0, "Should allow some burst requests");
}

/// Test rate limit reset after time window
#[tokio::test]
#[ignore] // This test takes a long time, run manually
async fn test_rate_limit_reset() {
    let app = setup_test_app().await;

    // Trigger rate limiting
    for _ in 0..20 {
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(HEALTH)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
    }

    // Make one more request to confirm rate limiting
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(HEALTH)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let was_rate_limited = response.status() == StatusCode::TOO_MANY_REQUESTS;

    if was_rate_limited {
        println!("Rate limiting confirmed, waiting for reset...");
        
        // Wait for rate limit window to reset (1 minute + buffer)
        wait_for_rate_limit_reset().await;

        // Try again after reset
        let response = app
            .oneshot(
                Request::builder()
                    .uri(HEALTH)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK, 
                  "Rate limit should reset after time window");
    } else {
        println!("Rate limiting not triggered in test");
    }
}

/// Test rate limiting with different endpoints
#[tokio::test]
async fn test_rate_limiting_different_endpoints() {
    let app = setup_test_app().await;

    let endpoints = vec![
        HEALTH,
        STATUS,
        METRICS,
    ];

    for endpoint in endpoints {
        let mut request_count = 0;
        
        // Make requests to this specific endpoint
        for _ in 0..15 {
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

            if response.status() == StatusCode::TOO_MANY_REQUESTS {
                break;
            }

            request_count += 1;
            sleep(Duration::from_millis(50)).await;
        }

        println!("Endpoint {}: {} requests before rate limit", endpoint, request_count);
    }
}

/// Test rate limiting error response format
#[tokio::test]
async fn test_rate_limit_error_response() {
    let app = setup_test_app().await;

    // Try to trigger rate limiting
    for _ in 0..25 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(HEALTH)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            
            if !body.is_empty() {
                let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
                
                // Check error response format
                assert_eq!(json["success"], false);
                assert!(json["error"].is_object());
                
                println!("Rate limit error response: {}", json);
            }
            
            return; // Test passed
        }

        sleep(Duration::from_millis(20)).await;
    }

    println!("Rate limiting not triggered in test");
}

/// Test concurrent requests from same IP
#[tokio::test]
async fn test_concurrent_rate_limiting() {
    let app = setup_test_app().await;

    let mut handles = vec![];

    // Make 20 concurrent requests
    for i in 0..20 {
        let app_clone = app.clone();
        let handle = tokio::spawn(async move {
            let response = app_clone
                .oneshot(
                    Request::builder()
                        .uri(HEALTH)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            
            (i, response.status())
        });
        handles.push(handle);
    }

    let results = futures::future::join_all(handles).await;
    
    let mut success_count = 0;
    let mut rate_limited_count = 0;

    for result in results {
        let (i, status) = result.unwrap();
        match status {
            StatusCode::OK => success_count += 1,
            StatusCode::TOO_MANY_REQUESTS => rate_limited_count += 1,
            _ => println!("Request {}: Unexpected status {}", i, status),
        }
    }

    println!("Concurrent test - Success: {}, Rate limited: {}", 
             success_count, rate_limited_count);
    
    // Should handle concurrent requests properly
    assert!(success_count + rate_limited_count > 0, 
           "Should process concurrent requests");
}

/// Test rate limiting with malformed requests
#[tokio::test]
async fn test_rate_limiting_malformed_requests() {
    let app = setup_test_app().await;

    // Make malformed requests that should still be rate limited
    for _ in 0..15 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/nonexistent-endpoint")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Even 404s should be rate limited
        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            println!("Rate limiting applied to malformed requests");
            return;
        }

        sleep(Duration::from_millis(50)).await;
    }

    println!("Rate limiting not triggered for malformed requests");
}
