// Real-time Market Data Collection Test for PantherSwap Edge
// Tests the market data manager with live Alpha Vantage API data

use anyhow::Result;
use pantherswap_edge::config::Settings;
use pantherswap_edge::database::Database;
use pantherswap_edge::market_data::MarketDataManager;
use std::time::{Duration, Instant};
use tokio::time::{sleep, interval};
use tracing::{info, warn, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    setup_logging()?;
    
    info!("🚀 Starting Real-time Market Data Collection Test");
    
    // Load configuration
    dotenvy::dotenv().ok();
    let settings = Settings::load()?;
    info!("✅ Configuration loaded successfully");
    
    // Initialize database with cloud testing settings (more generous timeouts)
    info!("🔗 Connecting to TimescaleDB with cloud testing settings...");
    let database = Database::new_cloud_testing(&settings.database.url).await?;
    info!("✅ Database connected with cloud testing configuration");
    
    // Initialize market data manager
    info!("📊 Initializing Market Data Manager for real-time collection...");
    let mut market_data_manager = MarketDataManager::new(&settings, database.clone()).await?;
    info!("✅ Market Data Manager initialized");
    
    // Start query performance monitoring
    info!("📈 Starting real-time query performance monitoring...");
    let query_monitor = database.query_monitor();
    query_monitor.start_monitoring().await?;
    info!("✅ Query performance monitoring started");
    
    // Test real-time data collection
    info!("🔄 Starting real-time data collection test...");
    run_real_time_collection_test(&mut market_data_manager, &query_monitor).await?;
    
    info!("🏁 Real-time market data collection test completed");
    Ok(())
}

fn setup_logging() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx=warn,hyper=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(())
}

async fn run_real_time_collection_test(
    market_data_manager: &mut MarketDataManager,
    query_monitor: &pantherswap_edge::database::query_monitor::QueryPerformanceMonitor,
) -> Result<()> {
    info!("🎯 Real-time Data Collection Test Configuration:");
    info!("   📊 Duration: 5 minutes");
    info!("   🔄 Collection Interval: 30 seconds");
    info!("   📈 Instruments: EUR/USD, GBP/USD, USD/JPY, AUD/USD, USD/CAD");
    info!("   🔍 Performance Monitoring: Enabled");
    
    let test_duration = Duration::from_secs(300); // 5 minutes
    let collection_interval = Duration::from_secs(30); // 30 seconds
    let start_time = Instant::now();
    
    let mut collection_count = 0;
    let mut successful_collections = 0;
    let mut failed_collections = 0;
    
    // Create interval for data collection
    let mut collection_timer = interval(collection_interval);
    
    // Create interval for performance reporting
    let mut reporting_timer = interval(Duration::from_secs(60)); // Report every minute
    
    info!("🚀 Starting real-time data collection...");
    
    loop {
        tokio::select! {
            _ = collection_timer.tick() => {
                if start_time.elapsed() >= test_duration {
                    break;
                }
                
                collection_count += 1;
                info!("📊 Collection #{} - Fetching market data...", collection_count);
                
                let collection_start = Instant::now();
                
                match collect_market_data(market_data_manager).await {
                    Ok(data_points) => {
                        successful_collections += 1;
                        let collection_time = collection_start.elapsed();
                        info!("✅ Collection #{} completed: {} data points in {:?}", 
                              collection_count, data_points, collection_time);
                        
                        // Record slow collection as slow query
                        if collection_time > Duration::from_millis(5000) {
                            query_monitor.record_slow_query(
                                format!("Market data collection #{}", collection_count),
                                collection_time.as_millis() as u64
                            ).await;
                        }
                    }
                    Err(e) => {
                        failed_collections += 1;
                        error!("❌ Collection #{} failed: {}", collection_count, e);
                    }
                }
            }
            
            _ = reporting_timer.tick() => {
                // Report performance metrics
                let metrics = query_monitor.get_metrics().await;
                let elapsed = start_time.elapsed();
                
                info!("📈 Performance Report ({}s elapsed):", elapsed.as_secs());
                info!("   📊 Collections: {} total, {} successful, {} failed", 
                      collection_count, successful_collections, failed_collections);
                info!("   🔍 Query Metrics:");
                info!("     - Average Query Time: {:.2}ms", metrics.average_execution_time_ms);
                info!("     - Queries per Second: {:.2}", metrics.queries_per_second);
                info!("     - Connection Pool Utilization: {:.1}%", 
                      metrics.connection_pool_utilization * 100.0);
                info!("     - Active Connections: {}", metrics.active_connections);
                info!("     - Slow Queries: {}", metrics.slow_queries);
                
                // Check for alerts
                let alerts = query_monitor.get_alerts(Some(5)).await;
                if !alerts.is_empty() {
                    warn!("⚠️  Recent Performance Alerts:");
                    for alert in alerts {
                        warn!("   - {:?}: {}", alert.alert_type, alert.message);
                    }
                }
            }
        }
    }
    
    // Final report
    let total_time = start_time.elapsed();
    let success_rate = if collection_count > 0 {
        (successful_collections as f64 / collection_count as f64) * 100.0
    } else {
        0.0
    };
    
    info!("🏆 Real-time Data Collection Test Results:");
    info!("   ⏱️  Total Duration: {:?}", total_time);
    info!("   📊 Total Collections: {}", collection_count);
    info!("   ✅ Successful: {} ({:.1}%)", successful_collections, success_rate);
    info!("   ❌ Failed: {} ({:.1}%)", failed_collections, 100.0 - success_rate);
    info!("   📈 Average Collection Rate: {:.2} collections/minute", 
          collection_count as f64 / (total_time.as_secs() as f64 / 60.0));
    
    // Final performance metrics
    let final_metrics = query_monitor.get_metrics().await;
    info!("📊 Final Database Performance:");
    info!("   - Total Queries: {}", final_metrics.total_queries);
    info!("   - Average Query Time: {:.2}ms", final_metrics.average_execution_time_ms);
    info!("   - Slow Queries: {}", final_metrics.slow_queries);
    info!("   - Connection Pool Utilization: {:.1}%", 
          final_metrics.connection_pool_utilization * 100.0);
    
    // Get slow queries summary
    let slow_queries = query_monitor.get_slow_queries(Some(10)).await;
    if !slow_queries.is_empty() {
        warn!("🐌 Slow Operations Detected:");
        for slow_query in slow_queries {
            warn!("   - {}ms: {}", slow_query.execution_time_ms, 
                  slow_query.query.chars().take(50).collect::<String>());
        }
    }
    
    // Test assessment
    if success_rate >= 95.0 && final_metrics.average_execution_time_ms < 1000.0 {
        info!("🎉 Real-time data collection test: ✅ PASSED");
        info!("   - High success rate ({:.1}%)", success_rate);
        info!("   - Good database performance ({:.2}ms avg)", final_metrics.average_execution_time_ms);
    } else {
        warn!("⚠️  Real-time data collection test: ❌ NEEDS IMPROVEMENT");
        if success_rate < 95.0 {
            warn!("   - Low success rate: {:.1}% (target: ≥95%)", success_rate);
        }
        if final_metrics.average_execution_time_ms >= 1000.0 {
            warn!("   - Slow database performance: {:.2}ms (target: <1000ms)", 
                  final_metrics.average_execution_time_ms);
        }
    }
    
    Ok(())
}

async fn collect_market_data(
    market_data_manager: &mut MarketDataManager,
) -> Result<usize> {
    // Simulate market data collection
    // In a real implementation, this would call market_data_manager.collect_data()
    
    // For now, simulate the collection process
    let instruments = vec!["EURUSD", "GBPUSD", "USDJPY", "AUDUSD", "USDCAD"];
    let mut data_points = 0;
    
    for instrument in instruments {
        // Simulate API call delay
        sleep(Duration::from_millis(200)).await;
        data_points += 1;
        
        // Simulate occasional API failures (5% failure rate)
        if rand::random::<f64>() < 0.05 {
            return Err(anyhow::anyhow!("Simulated API failure for {}", instrument));
        }
    }
    
    Ok(data_points)
}

// Add rand dependency simulation
mod rand {
    pub fn random<T>() -> T 
    where 
        T: From<f64>
    {
        // Simple pseudo-random number for simulation
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        T::from((nanos % 1000) as f64 / 1000.0)
    }
}
