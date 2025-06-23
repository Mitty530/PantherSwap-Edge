// Cargo.toml dependencies:
/*
[dependencies]
reqwest = { version = "0.11", features = ["json"] }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
*/

use reqwest;
use serde_json::Value;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Latest Alpaca paper trading API keys
    let api_key = "CK6ZEGH5TA1AU9MLZSPW";
    let secret_key = "aQzcjaf1VlqQyawUbXu6BbLVK47LWb6w2Qv634Ue";
    let base_url = "https://paper-api.alpaca.markets";  // Paper endpoint
    let data_url = "https://data.alpaca.markets";       // Data endpoint (same for paper/live)

    println!("🔄 Testing Alpaca Paper Trading API with Rust...");
    println!("API Key: {}", api_key);
    println!("Base URL: {}", base_url);
    println!("Data URL: {}", data_url);
    println!("Environment: Paper Trading");
    println!("=" .repeat(50));
    
    // Test account connection
    match test_account_connection(api_key, secret_key, base_url).await {
        Ok(_) => {
            println!("✅ Account connection successful!");
            
            // Test market data
            match test_market_data(api_key, secret_key, "AAPL").await {
                Ok(_) => {
                    println!("\n🎉 All tests passed! Your Alpaca API is working with Rust.");
                    
                    // Test additional symbols
                    println!("\n📊 Testing additional symbols...");
                    for symbol in ["MSFT", "GOOGL", "TSLA"] {
                        match test_market_data(api_key, secret_key, symbol).await {
                            Ok(_) => println!("✅ {} data retrieved successfully", symbol),
                            Err(e) => println!("⚠️  {} data failed: {}", symbol, e),
                        }
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await; // Rate limiting
                    }
                    
                    // Test market status
                    match test_market_status(api_key, secret_key, base_url).await {
                        Ok(_) => println!("✅ Market status retrieved successfully"),
                        Err(e) => println!("⚠️  Market status failed: {}", e),
                    }
                }
                Err(e) => println!("❌ Market data test failed: {}", e),
            }
        }
        Err(e) => {
            println!("❌ Account connection failed: {}", e);
            println!("Make sure you're using Paper Trading API keys from alpaca.markets");
            
            // Additional debugging info
            println!("\n🔍 Debugging Information:");
            println!("- Check if API keys are valid and active");
            println!("- Verify paper trading account is enabled");
            println!("- Ensure account is not suspended or restricted");
            println!("- Try logging into Alpaca web interface to verify account status");
        }
    }
    
    Ok(())
}

async fn test_account_connection(
    api_key: &str, 
    secret_key: &str, 
    base_url: &str
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 Testing account connection...");
    
    let client = reqwest::Client::new();
    
    let response = client
        .get(&format!("{}/v2/account", base_url))
        .header("APCA-API-KEY-ID", api_key)
        .header("APCA-API-SECRET-KEY", secret_key)
        .send()
        .await?;
    
    println!("Response Status: {}", response.status());
    
    if response.status().is_success() {
        let account_data: Value = response.json().await?;
        
        println!("✅ Account Details:");
        println!("   Account ID: {}", account_data["id"].as_str().unwrap_or("N/A"));
        println!("   Status: {}", account_data["status"].as_str().unwrap_or("N/A"));
        println!("   Trading Blocked: {}", account_data["trading_blocked"].as_bool().unwrap_or(false));
        println!("   Buying Power: ${}", account_data["buying_power"].as_str().unwrap_or("N/A"));
        println!("   Cash: ${}", account_data["cash"].as_str().unwrap_or("N/A"));
        println!("   Portfolio Value: ${}", account_data["portfolio_value"].as_str().unwrap_or("N/A"));
        
        Ok(())
    } else {
        let error_text = response.text().await?;
        Err(format!("HTTP {}: {}", response.status(), error_text).into())
    }
}

async fn test_market_data(
    api_key: &str,
    secret_key: &str,
    symbol: &str
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let data_url = "https://data.alpaca.markets";  // Data endpoint for paper trading
    
    println!("\n📊 Testing market data for {}...", symbol);
    
    let response = client
        .get(&format!("{}/v2/stocks/{}/quotes/latest", data_url, symbol))
        .header("APCA-API-KEY-ID", api_key)
        .header("APCA-API-SECRET-KEY", secret_key)
        .send()
        .await?;
    
    println!("   Response Status: {}", response.status());
    
    if response.status().is_success() {
        let quote_data: Value = response.json().await?;
        
        if let Some(quote) = quote_data["quote"].as_object() {
            println!("   ✅ {} Quote Data:", symbol);
            println!("      Bid Price: ${:.2}", quote["bp"].as_f64().unwrap_or(0.0));
            println!("      Ask Price: ${:.2}", quote["ap"].as_f64().unwrap_or(0.0));
            println!("      Bid Size: {}", quote["bs"].as_i64().unwrap_or(0));
            println!("      Ask Size: {}", quote["as"].as_i64().unwrap_or(0));
            
            if let Some(timestamp) = quote["t"].as_str() {
                println!("      Timestamp: {}", timestamp);
            }
        }
        
        Ok(())
    } else {
        let error_text = response.text().await?;
        Err(format!("HTTP {}: {}", response.status(), error_text).into())
    }
}

async fn test_market_status(
    api_key: &str, 
    secret_key: &str, 
    base_url: &str
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🕐 Testing market status...");
    
    let client = reqwest::Client::new();
    
    let response = client
        .get(&format!("{}/v2/clock", base_url))
        .header("APCA-API-KEY-ID", api_key)
        .header("APCA-API-SECRET-KEY", secret_key)
        .send()
        .await?;
    
    println!("   Response Status: {}", response.status());
    
    if response.status().is_success() {
        let clock_data: Value = response.json().await?;
        
        println!("   ✅ Market Status:");
        println!("      Current Time: {}", clock_data["timestamp"].as_str().unwrap_or("N/A"));
        println!("      Market Open: {}", clock_data["is_open"].as_bool().unwrap_or(false));
        println!("      Next Open: {}", clock_data["next_open"].as_str().unwrap_or("N/A"));
        println!("      Next Close: {}", clock_data["next_close"].as_str().unwrap_or("N/A"));
        
        Ok(())
    } else {
        let error_text = response.text().await?;
        Err(format!("HTTP {}: {}", response.status(), error_text).into())
    }
}

// Example function for placing a test order (commented out for safety)
/*
async fn place_test_order(
    api_key: &str, 
    secret_key: &str, 
    base_url: &str
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Testing order placement (PAPER TRADING ONLY)...");
    
    let client = reqwest::Client::new();
    
    let mut order_data = HashMap::new();
    order_data.insert("symbol", "AAPL");
    order_data.insert("qty", "1");
    order_data.insert("side", "buy");
    order_data.insert("type", "market");
    order_data.insert("time_in_force", "day");
    
    let response = client
        .post(&format!("{}/v2/orders", base_url))
        .header("APCA-API-KEY-ID", api_key)
        .header("APCA-API-SECRET-KEY", secret_key)
        .header("Content-Type", "application/json")
        .json(&order_data)
        .send()
        .await?;
    
    println!("   Response Status: {}", response.status());
    
    if response.status().is_success() {
        let order_response: Value = response.json().await?;
        println!("   ✅ Order placed successfully!");
        println!("      Order ID: {}", order_response["id"].as_str().unwrap_or("N/A"));
        println!("      Symbol: {}", order_response["symbol"].as_str().unwrap_or("N/A"));
        println!("      Quantity: {}", order_response["qty"].as_str().unwrap_or("N/A"));
        println!("      Side: {}", order_response["side"].as_str().unwrap_or("N/A"));
        println!("      Status: {}", order_response["status"].as_str().unwrap_or("N/A"));
        Ok(())
    } else {
        let error_text = response.text().await?;
        Err(format!("Order failed: HTTP {}: {}", response.status(), error_text).into())
    }
}
*/
