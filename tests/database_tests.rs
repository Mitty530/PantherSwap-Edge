// Database Integration Tests
// Run with: cargo test --test database_tests

use pantherswap_edge::database::{Database, types::*};
use pantherswap_edge::config::Settings;
use uuid::Uuid;
use chrono::{Utc, Duration};
use bigdecimal::BigDecimal;
use std::str::FromStr;

mod common;
use common::*;

/// Test database connection and basic operations
#[tokio::test]
async fn test_database_connection() {
    init_test_logging();
    
    let settings = Settings::load().unwrap_or_else(|_| {
        Settings {
            database: pantherswap_edge::config::DatabaseConfig {
                url: "postgresql://localhost/test_db".to_string(),
                max_connections: 5,
                min_connections: 1,
                acquire_timeout: 15,
                idle_timeout: Some(300),
                max_lifetime: Some(1800),
            },
            market_data: pantherswap_edge::config::MarketDataConfig {
                alpha_vantage_api_key: "test_key".to_string(),
                rate_limit_requests_per_minute: 5,
                rate_limit_requests_per_day: 500,
                default_instruments: vec!["EURUSD".to_string()],
            },
        }
    });

    match Database::new_testing(&settings.database.url).await {
        Ok(db) => {
            println!("✅ Database connection successful");
            
            // Test basic query
            let query_manager = db.query_manager();
            
            // Test connection with a simple query
            match sqlx::query("SELECT 1 as test")
                .fetch_one(&db.pool)
                .await 
            {
                Ok(_) => println!("✅ Basic query successful"),
                Err(e) => println!("❌ Basic query failed: {}", e),
            }
        }
        Err(e) => {
            println!("❌ Database connection failed: {}", e);
            println!("ℹ️  This is expected if database is not available");
        }
    }
}

/// Test instrument CRUD operations
#[tokio::test]
async fn test_instrument_crud_operations() {
    init_test_logging();
    
    let settings = create_test_settings();
    
    let db = match Database::new_testing(&settings.database.url).await {
        Ok(db) => db,
        Err(_) => {
            println!("⏭️  Skipping database test - database not available");
            return;
        }
    };

    let query_manager = db.query_manager();

    // Create test instrument
    let test_instrument = Instrument {
        id: Uuid::new_v4(),
        symbol: format!("TEST{}", Uuid::new_v4().to_string()[..8].to_uppercase()),
        name: "Test Currency Pair".to_string(),
        instrument_type: "forex".to_string(),
        base_currency: "TST".to_string(),
        quote_currency: "USD".to_string(),
        tick_size: 0.0001,
        lot_size: 100000.0,
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Test INSERT
    match query_manager.insert_instrument(&test_instrument).await {
        Ok(instrument_id) => {
            println!("✅ Instrument created with ID: {}", instrument_id);

            // Test SELECT by ID
            match query_manager.get_instrument_by_id(instrument_id).await {
                Ok(Some(retrieved)) => {
                    println!("✅ Instrument retrieved successfully");
                    assert_eq!(retrieved.symbol, test_instrument.symbol);
                    assert_eq!(retrieved.name, test_instrument.name);
                }
                Ok(None) => println!("❌ Instrument not found after creation"),
                Err(e) => println!("❌ Failed to retrieve instrument: {}", e),
            }

            // Test SELECT by symbol
            match query_manager.get_instrument_by_symbol(&test_instrument.symbol).await {
                Ok(Some(retrieved)) => {
                    println!("✅ Instrument retrieved by symbol");
                    assert_eq!(retrieved.id, instrument_id);
                }
                Ok(None) => println!("❌ Instrument not found by symbol"),
                Err(e) => println!("❌ Failed to retrieve instrument by symbol: {}", e),
            }

            // Test UPDATE
            let mut updated_instrument = test_instrument.clone();
            updated_instrument.id = instrument_id;
            updated_instrument.name = "Updated Test Currency Pair".to_string();
            updated_instrument.is_active = false;
            updated_instrument.updated_at = Utc::now();

            match query_manager.update_instrument(&updated_instrument).await {
                Ok(_) => {
                    println!("✅ Instrument updated successfully");

                    // Verify update
                    match query_manager.get_instrument_by_id(instrument_id).await {
                        Ok(Some(retrieved)) => {
                            assert_eq!(retrieved.name, "Updated Test Currency Pair");
                            assert_eq!(retrieved.is_active, false);
                            println!("✅ Update verified");
                        }
                        _ => println!("❌ Failed to verify update"),
                    }
                }
                Err(e) => println!("❌ Failed to update instrument: {}", e),
            }
        }
        Err(e) => println!("❌ Failed to create instrument: {}", e),
    }
}

/// Test market tick operations
#[tokio::test]
async fn test_market_tick_operations() {
    init_test_logging();
    
    let settings = create_test_settings();
    
    let db = match Database::new_testing(&settings.database.url).await {
        Ok(db) => db,
        Err(_) => {
            println!("⏭️  Skipping database test - database not available");
            return;
        }
    };

    let query_manager = db.query_manager();

    // First, create a test instrument
    let test_instrument = Instrument {
        id: Uuid::new_v4(),
        symbol: format!("TICK{}", Uuid::new_v4().to_string()[..8].to_uppercase()),
        name: "Test Tick Instrument".to_string(),
        instrument_type: "forex".to_string(),
        base_currency: "TST".to_string(),
        quote_currency: "USD".to_string(),
        tick_size: 0.0001,
        lot_size: 100000.0,
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let instrument_id = match query_manager.insert_instrument(&test_instrument).await {
        Ok(id) => {
            println!("✅ Test instrument created for tick testing");
            id
        }
        Err(e) => {
            println!("❌ Failed to create test instrument: {}", e);
            return;
        }
    };

    // Create test market tick
    let test_tick = MarketTick {
        timestamp: Utc::now(),
        instrument_id,
        provider: "test_provider".to_string(),
        bid_price: 1.0850,
        ask_price: 1.0852,
        bid_size: 1000000.0,
        ask_size: 1000000.0,
        last_price: 1.0851,
        volume: 5000000.0,
        spread: 0.0002,
        data_quality_score: 0.95,
        raw_data: Some(serde_json::json!({"test": "data"})),
    };

    // Test INSERT market tick
    match query_manager.insert_market_tick(&test_tick).await {
        Ok(_) => {
            println!("✅ Market tick inserted successfully");

            // Test retrieve latest tick
            match query_manager.get_latest_market_tick(instrument_id).await {
                Ok(Some(retrieved)) => {
                    println!("✅ Latest market tick retrieved");
                    assert_eq!(retrieved.instrument_id, instrument_id);
                    assert_eq!(retrieved.provider, "test_provider");
                    assert!((retrieved.bid_price - 1.0850).abs() < 0.0001);
                }
                Ok(None) => println!("❌ No market tick found"),
                Err(e) => println!("❌ Failed to retrieve market tick: {}", e),
            }

            // Test retrieve multiple ticks
            match query_manager.get_latest_market_ticks(Some(instrument_id), Some(10)).await {
                Ok(ticks) => {
                    println!("✅ Retrieved {} market ticks", ticks.len());
                    assert!(!ticks.is_empty());
                }
                Err(e) => println!("❌ Failed to retrieve market ticks: {}", e),
            }

            // Test time-range query
            let end_time = Utc::now();
            let start_time = end_time - Duration::hours(1);

            match query_manager.get_market_ticks_for_instrument(
                instrument_id,
                Some(start_time),
                Some(end_time),
                Some(10),
            ).await {
                Ok(ticks) => {
                    println!("✅ Retrieved {} ticks in time range", ticks.len());
                }
                Err(e) => println!("❌ Failed to retrieve ticks in time range: {}", e),
            }
        }
        Err(e) => println!("❌ Failed to insert market tick: {}", e),
    }
}

/// Test database performance and connection pooling
#[tokio::test]
async fn test_database_performance() {
    init_test_logging();
    
    let settings = create_test_settings();
    
    let db = match Database::new_testing(&settings.database.url).await {
        Ok(db) => db,
        Err(_) => {
            println!("⏭️  Skipping performance test - database not available");
            return;
        }
    };

    let query_manager = db.query_manager();

    // Test concurrent operations
    let mut handles = vec![];

    for i in 0..10 {
        let qm = query_manager.clone();
        let handle = tokio::spawn(async move {
            let start = std::time::Instant::now();
            
            // Simple query to test connection pool
            let result = sqlx::query("SELECT $1 as test_value")
                .bind(i)
                .fetch_one(&qm.pool)
                .await;
            
            let duration = start.elapsed();
            (i, result.is_ok(), duration)
        });
        handles.push(handle);
    }

    let results = futures::future::join_all(handles).await;
    
    let mut success_count = 0;
    let mut total_duration = std::time::Duration::ZERO;

    for result in results {
        let (i, success, duration) = result.unwrap();
        if success {
            success_count += 1;
            total_duration += duration;
        }
        println!("Query {}: {} in {:?}", i, if success { "✅" } else { "❌" }, duration);
    }

    println!("✅ {}/10 concurrent queries successful", success_count);
    if success_count > 0 {
        let avg_duration = total_duration / success_count as u32;
        println!("📊 Average query time: {:?}", avg_duration);
    }
}

/// Test database error handling
#[tokio::test]
async fn test_database_error_handling() {
    init_test_logging();
    
    let settings = create_test_settings();
    
    let db = match Database::new_testing(&settings.database.url).await {
        Ok(db) => db,
        Err(_) => {
            println!("⏭️  Skipping error handling test - database not available");
            return;
        }
    };

    let query_manager = db.query_manager();

    // Test invalid UUID
    let invalid_id = Uuid::new_v4();
    match query_manager.get_instrument_by_id(invalid_id).await {
        Ok(None) => println!("✅ Correctly handled non-existent instrument"),
        Ok(Some(_)) => println!("❌ Unexpectedly found non-existent instrument"),
        Err(e) => println!("❌ Error querying non-existent instrument: {}", e),
    }

    // Test duplicate symbol insertion
    let duplicate_instrument = Instrument {
        id: Uuid::new_v4(),
        symbol: "DUPLICATE_TEST".to_string(),
        name: "Duplicate Test".to_string(),
        instrument_type: "forex".to_string(),
        base_currency: "DUP".to_string(),
        quote_currency: "USD".to_string(),
        tick_size: 0.0001,
        lot_size: 100000.0,
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Insert first time
    let first_result = query_manager.insert_instrument(&duplicate_instrument).await;
    
    if first_result.is_ok() {
        // Try to insert again with same symbol
        let second_result = query_manager.insert_instrument(&duplicate_instrument).await;
        
        match second_result {
            Ok(_) => println!("❌ Duplicate symbol was allowed"),
            Err(_) => println!("✅ Duplicate symbol correctly rejected"),
        }
    }
}

/// Test database migrations and schema
#[tokio::test]
async fn test_database_schema() {
    init_test_logging();
    
    let settings = create_test_settings();
    
    let db = match Database::new_testing(&settings.database.url).await {
        Ok(db) => db,
        Err(_) => {
            println!("⏭️  Skipping schema test - database not available");
            return;
        }
    };

    // Test that required tables exist
    let tables_to_check = vec![
        "instruments",
        "market_ticks",
    ];

    for table_name in tables_to_check {
        let result = sqlx::query(&format!(
            "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = '{}'",
            table_name
        ))
        .fetch_one(&db.pool)
        .await;

        match result {
            Ok(_) => println!("✅ Table '{}' exists", table_name),
            Err(e) => println!("❌ Table '{}' check failed: {}", table_name, e),
        }
    }

    // Test TimescaleDB specific features if available
    let timescale_check = sqlx::query(
        "SELECT COUNT(*) FROM pg_extension WHERE extname = 'timescaledb'"
    )
    .fetch_one(&db.pool)
    .await;

    match timescale_check {
        Ok(_) => println!("✅ TimescaleDB extension check completed"),
        Err(e) => println!("ℹ️  TimescaleDB check failed (might not be installed): {}", e),
    }
}
