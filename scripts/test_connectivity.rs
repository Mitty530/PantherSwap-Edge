#!/usr/bin/env rust-script
//! Simple connectivity test for PantherSwap Edge
//! Tests database and IG API connectivity without complex dependencies

use std::env;
use std::time::Instant;
use reqwest::Client;
use serde_json::{json, Value};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    println!("🚀 PantherSwap Edge Connectivity Test");
    println!("=====================================");

    // Test 1: Database Connectivity
    println!("\n📊 Testing TimescaleDB Connectivity...");
    let db_result = test_database_connectivity().await;
    match db_result {
        Ok(latency) => println!("✅ Database: Connected ({:.2}ms)", latency),
        Err(e) => println!("❌ Database: Failed - {}", e),
    }

    // Test 2: IG Trading API Connectivity
    println!("\n🔌 Testing IG Trading API Connectivity...");
    let ig_result = test_ig_api_connectivity().await;
    match ig_result {
        Ok(latency) => println!("✅ IG API: Connected ({:.2}ms)", latency),
        Err(e) => println!("❌ IG API: Failed - {}", e),
    }

    // Test 3: Environment Variables
    println!("\n🔧 Testing Environment Configuration...");
    test_environment_variables();

    println!("\n🎯 Connectivity Test Complete");
    Ok(())
}

async fn test_database_connectivity() -> Result<f64, Box<dyn std::error::Error>> {
    let database_url = env::var("PANTHERSWAP_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://tsdbadmin:sz2eu577bgqi5767@jqrbtbc5nw.w0mq2s13iy.tsdb.cloud.timescale.com:35762/tsdb?sslmode=require".to_string());

    let start = Instant::now();
    
    // Use sqlx to test connection
    let pool = sqlx::PgPool::connect(&database_url).await?;
    
    // Test with a simple query
    let row: (i32,) = sqlx::query_as("SELECT 1")
        .fetch_one(&pool)
        .await?;
    
    let latency = start.elapsed().as_millis() as f64;
    
    if row.0 == 1 {
        Ok(latency)
    } else {
        Err("Database query returned unexpected result".into())
    }
}

async fn test_ig_api_connectivity() -> Result<f64, Box<dyn std::error::Error>> {
    let api_key = env::var("PANTHERSWAP_MARKET_DATA_IG_TRADING_API_KEY")
        .unwrap_or_else(|_| "3ded3ba7db96187488bf8773b86bdf3e8fc42e9b".to_string());
    
    let security_token = env::var("PANTHERSWAP_MARKET_DATA_IG_TRADING_SECURITY_TOKEN")
        .unwrap_or_else(|_| "1206a1630c34bcc90fdcc1b62fc5920fa7ed3a216ae09933430d3de2c6bcf6CD01112".to_string());
    
    let cst = env::var("PANTHERSWAP_MARKET_DATA_IG_TRADING_CST")
        .unwrap_or_else(|_| "48417021199921da08b95b210d8f9492c36614232983a9f1f3e1a8f0748ce33CC01113".to_string());

    let client = Client::new();
    let start = Instant::now();

    // Test IG API session endpoint
    let base_url = "https://demo-api.ig.com/gateway/deal";
    let session_url = format!("{}/session", base_url);

    let auth_body = json!({
        "identifier": "",
        "password": "",
        "encryptedPassword": false
    });

    let response = client
        .post(&session_url)
        .header("Content-Type", "application/json; charset=UTF-8")
        .header("Accept", "application/json; charset=UTF-8")
        .header("X-IG-API-KEY", &api_key)
        .header("Version", "2")
        .json(&auth_body)
        .send()
        .await?;

    let latency = start.elapsed().as_millis() as f64;

    if response.status().is_success() || response.status().as_u16() == 401 {
        // 401 is expected without proper credentials, but it means API is reachable
        Ok(latency)
    } else {
        Err(format!("IG API returned status: {}", response.status()).into())
    }
}

fn test_environment_variables() {
    let required_vars = vec![
        "PANTHERSWAP_DATABASE_URL",
        "PANTHERSWAP_MARKET_DATA_IG_TRADING_API_KEY",
        "PANTHERSWAP_MARKET_DATA_IG_TRADING_SECURITY_TOKEN",
        "PANTHERSWAP_MARKET_DATA_IG_TRADING_CST",
    ];

    for var in required_vars {
        match env::var(var) {
            Ok(value) => {
                let masked_value = if value.len() > 8 {
                    format!("{}...{}", &value[..4], &value[value.len()-4..])
                } else {
                    "***".to_string()
                };
                println!("✅ {}: {}", var, masked_value);
            }
            Err(_) => println!("❌ {}: Not set", var),
        }
    }
}
