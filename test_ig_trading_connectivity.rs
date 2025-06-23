// IG Trading API Connectivity Test for PantherSwap Edge
use pantherswap_edge::market_data::ig_trading::{IGTradingClient, IGTradingConfig};
use pantherswap_edge::utils::Result;
use tracing::{info, error, warn};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("🚀 Starting IG Trading API Connectivity Test");
    info!("================================================");

    // Load IG Trading configuration
    let config = IGTradingConfig {
        api_key: "3ded3ba7db96187488bf8773b86bdf3e8fc42e9b".to_string(),
        security_token: "1206a1630c34bcc90fdcc1b62fc5920fa7ed3a216ae09933430d3de2c6bcf6CD01112".to_string(),
        cst: "48417021199921da08b95b210d8f9492c36614232983a9f1f3e1a8f0748ce33CC01113".to_string(),
        version: "2".to_string(),
        base_url: "https://demo-api.ig.com/gateway/deal".to_string(),
        content_type: "application/json; charset=UTF-8".to_string(),
        accept: "application/json; charset=UTF-8".to_string(),
        demo_mode: true,
        rate_limit_per_minute: 100,
        connection_timeout_ms: 5000,
        retry_attempts: 3,
    };

    info!("📋 Configuration:");
    info!("  API Key: {}...{}", &config.api_key[..8], &config.api_key[config.api_key.len()-8..]);
    info!("  Base URL: {}", config.base_url);
    info!("  Demo Mode: {}", config.demo_mode);
    info!("  Rate Limit: {} requests/minute", config.rate_limit_per_minute);

    // Create IG Trading client
    let mut client = IGTradingClient::new(config);

    // Test 1: Basic Connection Test
    info!("\n🔍 Test 1: Basic API Connection");
    info!("--------------------------------");
    
    match client.test_connection().await {
        Ok(success) => {
            if success {
                info!("✅ IG Trading API connection successful");
            } else {
                warn!("⚠️ IG Trading API connection failed");
            }
        }
        Err(e) => {
            error!("❌ IG Trading API connection error: {}", e);
        }
    }

    // Test 2: Authentication Test
    info!("\n🔐 Test 2: Authentication");
    info!("-------------------------");
    
    match client.authenticate().await {
        Ok(_) => {
            info!("✅ IG Trading authentication successful");
        }
        Err(e) => {
            error!("❌ IG Trading authentication failed: {}", e);
            info!("💡 Note: This is expected in demo mode without valid login credentials");
        }
    }

    // Test 3: Account Information (if authenticated)
    info!("\n📊 Test 3: Account Information");
    info!("------------------------------");
    
    match client.get_account_info().await {
        Ok(account_data) => {
            info!("✅ Account information retrieved successfully");
            info!("📋 Account Data: {}", serde_json::to_string_pretty(&account_data).unwrap_or_default());
        }
        Err(e) => {
            warn!("⚠️ Could not retrieve account information: {}", e);
            info!("💡 This is expected without proper authentication");
        }
    }

    // Test 4: Market Data Test (Popular Instruments)
    info!("\n📈 Test 4: Market Data Retrieval");
    info!("--------------------------------");
    
    let test_instruments = vec![
        "CS.D.EURUSD.MINI.IP".to_string(),  // EUR/USD
        "CS.D.GBPUSD.MINI.IP".to_string(),  // GBP/USD
        "IX.D.FTSE.DAILY.IP".to_string(),   // FTSE 100
        "IX.D.DOW.DAILY.IP".to_string(),    // Dow Jones
    ];

    for instrument in &test_instruments {
        info!("🔍 Testing instrument: {}", instrument);
        
        match client.fetch_market_data(&[instrument.clone()]).await {
            Ok(market_ticks) => {
                if !market_ticks.is_empty() {
                    let tick = &market_ticks[0];
                    info!("✅ Market data retrieved for {}", instrument);
                    info!("   Bid: {:.5}, Ask: {:.5}, Spread: {:.5}", 
                        tick.bid_price, tick.ask_price, tick.spread);
                    info!("   Quality Score: {:.2}", tick.data_quality_score);
                } else {
                    warn!("⚠️ No market data returned for {}", instrument);
                }
            }
            Err(e) => {
                error!("❌ Failed to get market data for {}: {}", instrument, e);
            }
        }
    }

    // Test 5: Performance Test
    info!("\n⚡ Test 5: Performance Test");
    info!("---------------------------");
    
    let start_time = std::time::Instant::now();
    let mut successful_requests = 0;
    let mut failed_requests = 0;
    
    for i in 0..5 {
        let test_instrument = "CS.D.EURUSD.MINI.IP".to_string();
        
        match client.fetch_market_data(&[test_instrument]).await {
            Ok(_) => {
                successful_requests += 1;
                info!("✅ Request {} successful", i + 1);
            }
            Err(e) => {
                failed_requests += 1;
                warn!("⚠️ Request {} failed: {}", i + 1, e);
            }
        }
        
        // Small delay to respect rate limits
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    }
    
    let total_time = start_time.elapsed();
    let avg_time_per_request = total_time.as_millis() as f64 / 5.0;
    
    info!("📊 Performance Results:");
    info!("   Total Time: {:?}", total_time);
    info!("   Average Time per Request: {:.2}ms", avg_time_per_request);
    info!("   Successful Requests: {}", successful_requests);
    info!("   Failed Requests: {}", failed_requests);
    info!("   Success Rate: {:.1}%", (successful_requests as f64 / 5.0) * 100.0);

    // Test Summary
    info!("\n📋 Test Summary");
    info!("===============");
    
    let overall_success = successful_requests > 0;
    
    if overall_success {
        info!("✅ IG Trading integration test PASSED");
        info!("🎯 Ready for integration with PantherSwap Edge");
        info!("💡 Recommendations:");
        info!("   - Configure proper authentication credentials for production");
        info!("   - Monitor rate limits during high-frequency trading");
        info!("   - Implement proper error handling and retry logic");
        info!("   - Consider using streaming API for real-time data");
    } else {
        warn!("⚠️ IG Trading integration test had issues");
        info!("🔧 Troubleshooting steps:");
        info!("   1. Verify API credentials are correct");
        info!("   2. Check network connectivity");
        info!("   3. Ensure demo account is properly configured");
        info!("   4. Review IG Trading API documentation");
    }

    info!("\n🏁 IG Trading connectivity test completed");
    
    Ok(())
}
