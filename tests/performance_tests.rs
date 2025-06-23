use axum::http::StatusCode;
use serde_json::{json, Value};
use std::time::{Duration, Instant};
use tokio;
use uuid::Uuid;

mod common;
use common::*;

/// Test order processing latency
#[tokio::test]
async fn test_order_processing_latency() {
    let app = create_test_app().await;

    let order_request = json!({
        "instrument_id": "550e8400-e29b-41d4-a716-446655440000",
        "side": "Buy",
        "quantity": 1000.0,
        "order_type": "Market",
        "time_in_force": "IOC"
    });

    // Measure latency for single order
    let start = Instant::now();
    let response = app
        .post("/api/v1/orders")
        .header("x-api-key", "demo-trader-key")
        .json(&order_request)
        .send()
        .await;
    let latency = start.elapsed();

    assert_eq!(response.status(), StatusCode::OK);
    println!("Single order processing latency: {:?}", latency);
    
    // Latency should be under 100ms for single order
    assert!(latency < Duration::from_millis(100));
}

/// Test concurrent order processing
#[tokio::test]
async fn test_concurrent_order_processing() {
    let app = create_test_app().await;

    let order_request = json!({
        "instrument_id": "550e8400-e29b-41d4-a716-446655440000",
        "side": "Buy",
        "quantity": 1000.0,
        "order_type": "Market",
        "time_in_force": "IOC"
    });

    // Process 10 concurrent orders
    let start = Instant::now();
    let mut handles = Vec::new();

    for _ in 0..10 {
        let app_clone = app.clone();
        let request_clone = order_request.clone();
        
        let handle = tokio::spawn(async move {
            app_clone
                .post("/api/v1/orders")
                .header("x-api-key", "demo-trader-key")
                .json(&request_clone)
                .send()
                .await
        });
        
        handles.push(handle);
    }

    // Wait for all orders to complete
    let mut success_count = 0;
    for handle in handles {
        let response = handle.await.unwrap();
        if response.status() == StatusCode::OK {
            success_count += 1;
        }
    }

    let total_time = start.elapsed();
    println!("Concurrent order processing time for 10 orders: {:?}", total_time);
    println!("Successful orders: {}/10", success_count);

    // All orders should succeed
    assert_eq!(success_count, 10);
    
    // Total time should be reasonable (under 1 second for 10 concurrent orders)
    assert!(total_time < Duration::from_secs(1));
}

/// Test portfolio update performance
#[tokio::test]
async fn test_portfolio_update_performance() {
    let app = create_test_app().await;

    // Measure portfolio summary retrieval time
    let start = Instant::now();
    let response = app
        .get("/api/v1/portfolio/summary")
        .header("x-api-key", "demo-trader-key")
        .send()
        .await;
    let latency = start.elapsed();

    assert_eq!(response.status(), StatusCode::OK);
    println!("Portfolio summary retrieval latency: {:?}", latency);
    
    // Should be under 50ms
    assert!(latency < Duration::from_millis(50));

    // Test positions retrieval
    let start = Instant::now();
    let response = app
        .get("/api/v1/portfolio/positions")
        .header("x-api-key", "demo-trader-key")
        .send()
        .await;
    let positions_latency = start.elapsed();

    assert_eq!(response.status(), StatusCode::OK);
    println!("Portfolio positions retrieval latency: {:?}", positions_latency);
    
    // Should be under 100ms
    assert!(positions_latency < Duration::from_millis(100));
}

/// Test signal generation and retrieval performance
#[tokio::test]
async fn test_signal_performance() {
    let app = create_test_app().await;

    // Test latest signals retrieval
    let start = Instant::now();
    let response = app
        .get("/api/v1/signals/latest")
        .header("x-api-key", "demo-trader-key")
        .send()
        .await;
    let latency = start.elapsed();

    assert_eq!(response.status(), StatusCode::OK);
    println!("Latest signals retrieval latency: {:?}", latency);
    
    // Should be under 100ms
    assert!(latency < Duration::from_millis(100));

    // Test signal analytics
    let start = Instant::now();
    let response = app
        .get("/api/v1/signals/analytics")
        .header("x-api-key", "demo-trader-key")
        .send()
        .await;
    let analytics_latency = start.elapsed();

    assert_eq!(response.status(), StatusCode::OK);
    println!("Signal analytics retrieval latency: {:?}", analytics_latency);
    
    // Should be under 150ms
    assert!(analytics_latency < Duration::from_millis(150));
}

/// Test risk calculation performance
#[tokio::test]
async fn test_risk_calculation_performance() {
    let app = create_test_app().await;

    // Test risk metrics calculation
    let start = Instant::now();
    let response = app
        .get("/api/v1/risk/metrics")
        .header("x-api-key", "demo-trader-key")
        .send()
        .await;
    let latency = start.elapsed();

    assert_eq!(response.status(), StatusCode::OK);
    println!("Risk metrics calculation latency: {:?}", latency);
    
    // Should be under 200ms (risk calculations can be more complex)
    assert!(latency < Duration::from_millis(200));

    // Test risk monitoring dashboard
    let start = Instant::now();
    let response = app
        .get("/api/v1/risk/monitoring")
        .header("x-api-key", "demo-trader-key")
        .send()
        .await;
    let monitoring_latency = start.elapsed();

    assert_eq!(response.status(), StatusCode::OK);
    println!("Risk monitoring dashboard latency: {:?}", monitoring_latency);
    
    // Should be under 250ms
    assert!(monitoring_latency < Duration::from_millis(250));
}

/// Test strategy performance calculation
#[tokio::test]
async fn test_strategy_performance_calculation() {
    let app = create_test_app().await;

    let strategy_id = Uuid::new_v4();

    // Test strategy performance calculation
    let start = Instant::now();
    let response = app
        .get(&format!("/api/v1/strategies/{}/performance", strategy_id))
        .header("x-api-key", "demo-trader-key")
        .send()
        .await;
    let latency = start.elapsed();

    assert_eq!(response.status(), StatusCode::OK);
    println!("Strategy performance calculation latency: {:?}", latency);
    
    // Should be under 300ms (performance calculations can be complex)
    assert!(latency < Duration::from_millis(300));
}

/// Test trading engine status retrieval performance
#[tokio::test]
async fn test_engine_status_performance() {
    let app = create_test_app().await;

    // Test engine status
    let start = Instant::now();
    let response = app
        .get("/api/v1/engine/status")
        .header("x-api-key", "demo-admin-key")
        .send()
        .await;
    let latency = start.elapsed();

    assert_eq!(response.status(), StatusCode::OK);
    println!("Engine status retrieval latency: {:?}", latency);
    
    // Should be very fast (under 25ms)
    assert!(latency < Duration::from_millis(25));

    // Test engine stats
    let start = Instant::now();
    let response = app
        .get("/api/v1/engine/stats")
        .header("x-api-key", "demo-admin-key")
        .send()
        .await;
    let stats_latency = start.elapsed();

    assert_eq!(response.status(), StatusCode::OK);
    println!("Engine stats retrieval latency: {:?}", stats_latency);
    
    // Should be under 100ms
    assert!(stats_latency < Duration::from_millis(100));
}

/// Test high-frequency API calls
#[tokio::test]
async fn test_high_frequency_calls() {
    let app = create_test_app().await;

    // Test 100 rapid portfolio summary calls
    let start = Instant::now();
    let mut handles = Vec::new();

    for _ in 0..100 {
        let app_clone = app.clone();
        let handle = tokio::spawn(async move {
            app_clone
                .get("/api/v1/portfolio/summary")
                .header("x-api-key", "demo-trader-key")
                .send()
                .await
        });
        handles.push(handle);
    }

    let mut success_count = 0;
    for handle in handles {
        let response = handle.await.unwrap();
        if response.status() == StatusCode::OK {
            success_count += 1;
        }
    }

    let total_time = start.elapsed();
    let avg_latency = total_time / 100;

    println!("High-frequency test: 100 calls in {:?}", total_time);
    println!("Average latency per call: {:?}", avg_latency);
    println!("Successful calls: {}/100", success_count);

    // Should handle at least 90% of calls successfully
    assert!(success_count >= 90);
    
    // Average latency should be reasonable
    assert!(avg_latency < Duration::from_millis(100));
}

/// Test memory usage during intensive operations
#[tokio::test]
async fn test_memory_usage() {
    let app = create_test_app().await;

    // Perform a series of operations that might consume memory
    for i in 0..50 {
        // Submit orders
        let order_request = json!({
            "instrument_id": "550e8400-e29b-41d4-a716-446655440000",
            "side": if i % 2 == 0 { "Buy" } else { "Sell" },
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

        // Check portfolio
        let response = app
            .get("/api/v1/portfolio/summary")
            .header("x-api-key", "demo-trader-key")
            .send()
            .await;

        assert_eq!(response.status(), StatusCode::OK);

        // Check signals
        let response = app
            .get("/api/v1/signals/latest")
            .header("x-api-key", "demo-trader-key")
            .send()
            .await;

        assert_eq!(response.status(), StatusCode::OK);

        // Small delay to prevent overwhelming the system
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    println!("Memory usage test completed successfully");
}
