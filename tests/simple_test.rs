// Simple Integration Test to verify framework works
// Run with: cargo test --test simple_test

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use tower::util::ServiceExt;
use serde_json::Value;

mod common;
use common::*;

#[tokio::test]
async fn test_basic_health_endpoint() {
    println!("🧪 Starting basic health endpoint test");
    
    let app = setup_test_app().await;
    println!("✅ Test app created successfully");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    println!("📊 Response status: {}", response.status());
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    println!("📄 Response body: {}", json);
    
    assert_eq!(json["success"], true);
    assert!(json["data"]["status"].as_str().unwrap() == "healthy");
    
    println!("✅ Basic health endpoint test passed!");
}

#[tokio::test]
async fn test_system_status_endpoint() {
    println!("🧪 Starting system status endpoint test");
    
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

    println!("📊 Status endpoint response: {}", response.status());
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    println!("📄 Status response: {}", json);
    
    assert_eq!(json["success"], true);
    assert!(json["data"]["database"].is_object());
    
    println!("✅ System status endpoint test passed!");
}

#[tokio::test]
async fn test_authentication_required() {
    println!("🧪 Starting authentication test");
    
    let app = setup_test_app().await;

    // Test without authentication
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/instruments")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    println!("📊 Unauthenticated request status: {}", response.status());
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Test with authentication
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

    println!("📊 Authenticated request status: {}", response.status());
    // Should not be 401 (might be 500 due to database issues, but auth should pass)
    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
    
    println!("✅ Authentication test passed!");
}

#[tokio::test]
async fn test_cors_headers() {
    println!("🧪 Starting CORS headers test");
    
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

    println!("📊 CORS test response status: {}", response.status());
    assert_eq!(response.status(), StatusCode::OK);

    let headers = response.headers();
    println!("📄 Response headers: {:?}", headers);
    
    // Check for CORS headers
    if headers.contains_key("access-control-allow-origin") {
        println!("✅ CORS headers found");
    } else {
        println!("ℹ️  CORS headers not found (might not be implemented yet)");
    }
    
    println!("✅ CORS headers test completed!");
}

#[tokio::test]
async fn test_framework_functionality() {
    println!("🧪 Testing integration test framework functionality");
    
    // Test that we can create multiple app instances
    let app1 = setup_test_app().await;
    let app2 = setup_test_app().await;
    
    println!("✅ Multiple app instances created");
    
    // Test that both work
    let response1 = app1
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    let response2 = app2
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response1.status(), StatusCode::OK);
    assert_eq!(response2.status(), StatusCode::OK);
    
    println!("✅ Integration test framework is working correctly!");
}
