// Test Alpha Vantage API Integration for Production Readiness Assessment
use std::time::{Duration, Instant};
use tokio::time::sleep;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Testing Alpha Vantage API Integration for Production Readiness");
    println!("================================================================");
    
    let api_key = "EZDZ4VOFQ2GRA7VU";
    let client = reqwest::Client::new();
    
    // Test 1: Basic API connectivity
    println!("\n📡 Test 1: Basic API Connectivity");
    let start = Instant::now();
    
    let url = format!(
        "https://www.alphavantage.co/query?function=FX_INTRADAY&from_symbol=EUR&to_symbol=USD&interval=1min&apikey={}",
        api_key
    );
    
    match client.get(&url).send().await {
        Ok(response) => {
            let latency = start.elapsed();
            println!("✅ API Response received in {:?}", latency);
            println!("   Status: {}", response.status());
            
            if response.status().is_success() {
                let text = response.text().await?;
                if let Ok(json) = serde_json::from_str::<Value>(&text) {
                    if json.get("Error Message").is_some() {
                        println!("❌ API Error: {}", json["Error Message"]);
                    } else if json.get("Note").is_some() {
                        println!("⚠️  API Rate Limit: {}", json["Note"]);
                    } else if json.get("Time Series FX (1min)").is_some() {
                        println!("✅ Valid market data received");
                        let time_series = &json["Time Series FX (1min)"];
                        let data_points = time_series.as_object().unwrap().len();
                        println!("   Data points: {}", data_points);
                    } else {
                        println!("⚠️  Unexpected response format");
                    }
                } else {
                    println!("❌ Invalid JSON response");
                }
            } else {
                println!("❌ HTTP Error: {}", response.status());
            }
        }
        Err(e) => {
            println!("❌ Network Error: {}", e);
        }
    }
    
    // Test 2: Multiple currency pairs
    println!("\n💱 Test 2: Multiple Currency Pairs");
    let pairs = vec![
        ("EUR", "USD"),
        ("GBP", "USD"),
        ("USD", "JPY"),
        ("AUD", "USD"),
    ];
    
    for (from, to) in pairs {
        let start = Instant::now();
        let url = format!(
            "https://www.alphavantage.co/query?function=CURRENCY_EXCHANGE_RATE&from_currency={}&to_currency={}&apikey={}",
            from, to, api_key
        );
        
        match client.get(&url).send().await {
            Ok(response) => {
                let latency = start.elapsed();
                if response.status().is_success() {
                    let text = response.text().await?;
                    if let Ok(json) = serde_json::from_str::<Value>(&text) {
                        if let Some(rate_data) = json.get("Realtime Currency Exchange Rate") {
                            if let Some(rate) = rate_data.get("5. Exchange Rate") {
                                println!("✅ {}/{}: {} (latency: {:?})", from, to, rate, latency);
                            }
                        } else {
                            println!("❌ {}/{}: No rate data", from, to);
                        }
                    }
                } else {
                    println!("❌ {}/{}: HTTP {}", from, to, response.status());
                }
            }
            Err(e) => {
                println!("❌ {}/{}: {}", from, to, e);
            }
        }
        
        // Rate limiting - wait between requests
        sleep(Duration::from_millis(500)).await;
    }
    
    // Test 3: Data quality assessment
    println!("\n📊 Test 3: Data Quality Assessment");
    let url = format!(
        "https://www.alphavantage.co/query?function=FX_INTRADAY&from_symbol=EUR&to_symbol=USD&interval=5min&apikey={}",
        api_key
    );
    
    match client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let text = response.text().await?;
                if let Ok(json) = serde_json::from_str::<Value>(&text) {
                    if let Some(time_series) = json.get("Time Series FX (5min)") {
                        let data_points = time_series.as_object().unwrap();
                        let mut valid_points = 0;
                        let mut total_points = 0;
                        
                        for (timestamp, data) in data_points {
                            total_points += 1;
                            if let (Some(open), Some(high), Some(low), Some(close)) = (
                                data.get("1. open"),
                                data.get("2. high"),
                                data.get("3. low"),
                                data.get("4. close")
                            ) {
                                if let (Ok(o), Ok(h), Ok(l), Ok(c)) = (
                                    open.as_str().unwrap().parse::<f64>(),
                                    high.as_str().unwrap().parse::<f64>(),
                                    low.as_str().unwrap().parse::<f64>(),
                                    close.as_str().unwrap().parse::<f64>()
                                ) {
                                    // Basic OHLC validation
                                    if h >= o && h >= l && h >= c && l <= o && l <= c {
                                        valid_points += 1;
                                    }
                                }
                            }
                        }
                        
                        let quality_score = (valid_points as f64 / total_points as f64) * 100.0;
                        println!("✅ Data Quality Score: {:.1}% ({}/{} valid points)", 
                                quality_score, valid_points, total_points);
                        
                        if quality_score >= 95.0 {
                            println!("✅ Excellent data quality");
                        } else if quality_score >= 90.0 {
                            println!("⚠️  Good data quality");
                        } else {
                            println!("❌ Poor data quality");
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Data quality test failed: {}", e);
        }
    }
    
    // Test 4: Performance benchmarking
    println!("\n⚡ Test 4: Performance Benchmarking");
    let mut latencies = Vec::new();
    let test_count = 5;
    
    for i in 1..=test_count {
        let start = Instant::now();
        let url = format!(
            "https://www.alphavantage.co/query?function=CURRENCY_EXCHANGE_RATE&from_currency=EUR&to_currency=USD&apikey={}",
            api_key
        );
        
        match client.get(&url).send().await {
            Ok(response) => {
                let latency = start.elapsed();
                latencies.push(latency.as_millis() as f64);
                println!("   Request {}: {:?}", i, latency);
            }
            Err(e) => {
                println!("   Request {} failed: {}", i, e);
            }
        }
        
        if i < test_count {
            sleep(Duration::from_millis(1000)).await;
        }
    }
    
    if !latencies.is_empty() {
        let avg_latency = latencies.iter().sum::<f64>() / latencies.len() as f64;
        let min_latency = latencies.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_latency = latencies.iter().fold(0.0, |a, &b| a.max(b));
        
        println!("📈 Performance Summary:");
        println!("   Average latency: {:.1}ms", avg_latency);
        println!("   Min latency: {:.1}ms", min_latency);
        println!("   Max latency: {:.1}ms", max_latency);
        
        if avg_latency < 500.0 {
            println!("✅ Excellent API performance");
        } else if avg_latency < 1000.0 {
            println!("⚠️  Good API performance");
        } else {
            println!("❌ Poor API performance");
        }
    }
    
    println!("\n🎯 Alpha Vantage Integration Test Complete");
    Ok(())
}
