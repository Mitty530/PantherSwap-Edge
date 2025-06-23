// Production Monitoring Demonstration
// Shows the comprehensive production monitoring system in action

use pantherswap_edge::monitoring::{ProductionMonitor, ProductionMonitoringConfig};
use pantherswap_edge::ai::monitoring::create_ai_performance_monitor;
use pantherswap_edge::database::{Database, health_monitor::DatabaseHealthMonitor};
use pantherswap_edge::trading::engine::{TradingEngine, TradingEngineConfig};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("🚀 Starting PantherSwap Edge Production Monitoring Demo");
    
    // Load database URL from environment or use default
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://tsdbadmin:sz2eu577bgqi5767@jqrbtbc5nw.w0mq2s13iy.tsdb.cloud.timescale.com:35762/tsdb?sslmode=require".to_string());
    
    info!("📊 Connecting to TimescaleDB...");
    let database = Database::new(&database_url).await?;
    info!("✅ Database connected successfully");
    
    // Initialize monitoring components
    info!("🔧 Initializing monitoring components...");
    let ai_monitor = Arc::new(create_ai_performance_monitor(database.clone()));
    let db_monitor = Arc::new(DatabaseHealthMonitor::with_defaults(database.pool.clone()));
    
    let trading_config = TradingEngineConfig::default();
    let trading_engine = TradingEngine::new(trading_config, database.clone()).await?;
    let trading_engine_arc = Arc::new(RwLock::new(trading_engine));
    
    // Configure production monitoring for demo
    let monitoring_config = ProductionMonitoringConfig {
        health_check_interval_seconds: 5,  // Check every 5 seconds for demo
        metrics_collection_interval_seconds: 3,  // Collect metrics every 3 seconds
        alert_aggregation_window_seconds: 30,
        enable_auto_recovery: true,
        enable_failover: true,
        max_consecutive_failures: 2,
        system_health_threshold: 0.8,
        enable_performance_profiling: true,
        enable_predictive_alerts: true,
    };
    
    info!("📊 Creating production monitor...");
    let production_monitor = Arc::new(ProductionMonitor::new(
        monitoring_config,
        ai_monitor,
        db_monitor,
        trading_engine_arc,
    ));
    
    // Start production monitoring
    info!("🚀 Starting production monitoring system...");
    production_monitor.start_monitoring().await?;
    info!("✅ Production monitoring started successfully");
    
    // Demo loop - show monitoring data every 10 seconds
    info!("📈 Starting monitoring demonstration...");
    for cycle in 1..=6 {  // Run for 6 cycles (1 minute)
        info!("📊 === Monitoring Cycle {} ===", cycle);
        
        // Get and display system health
        let system_health = production_monitor.get_system_health().await;
        info!("🏥 Overall System Health: {:.1}%", system_health.overall_health_score * 100.0);
        info!("⏰ System Uptime: {} seconds", system_health.uptime_seconds);
        
        // Get and display component health
        let component_health = production_monitor.get_all_component_health().await;
        info!("🔧 Component Health Status:");
        for (component, health) in &component_health {
            let status_emoji = match health.status {
                pantherswap_edge::monitoring::ComponentStatus::Healthy => "✅",
                pantherswap_edge::monitoring::ComponentStatus::Degraded => "⚠️",
                pantherswap_edge::monitoring::ComponentStatus::Critical => "🚨",
                pantherswap_edge::monitoring::ComponentStatus::Offline => "❌",
            };
            info!("  {} {}: {:.1}% - {:?}", status_emoji, component, health.health_score * 100.0, health.status);
            
            // Show performance metrics for each component
            if !health.performance_metrics.is_empty() {
                info!("    📈 Performance Metrics:");
                for (metric, value) in &health.performance_metrics {
                    info!("      - {}: {:.2}", metric, value);
                }
            }
        }
        
        // Get and display active alerts
        let active_alerts = production_monitor.get_active_alerts().await;
        if !active_alerts.is_empty() {
            warn!("🚨 Active Alerts ({}):", active_alerts.len());
            for alert in &active_alerts {
                let severity_emoji = match alert.severity {
                    pantherswap_edge::monitoring::AlertSeverity::Info => "ℹ️",
                    pantherswap_edge::monitoring::AlertSeverity::Warning => "⚠️",
                    pantherswap_edge::monitoring::AlertSeverity::Critical => "🚨",
                    pantherswap_edge::monitoring::AlertSeverity::Emergency => "🆘",
                };
                warn!("  {} [{}] {}: {}", severity_emoji, alert.component, alert.alert_type, alert.message);
            }
        } else {
            info!("✅ No active alerts");
        }
        
        // Get and display performance metrics history
        let performance_metrics = production_monitor.get_performance_metrics().await;
        info!("📊 Performance Metrics History: {} entries", performance_metrics.len());
        
        if let Some(latest_metrics) = performance_metrics.last() {
            info!("📈 Latest Performance Metrics:");
            info!("  - Requests/sec: {:.1}", latest_metrics.total_requests_per_second);
            info!("  - Avg Response Time: {:.1}ms", latest_metrics.average_response_time_ms);
            info!("  - Error Rate: {:.2}%", latest_metrics.error_rate_percentage);
            info!("  - Memory Usage: {:.1}%", latest_metrics.memory_usage_percentage);
            info!("  - CPU Usage: {:.1}%", latest_metrics.cpu_usage_percentage);
            info!("  - Network Throughput: {:.1} Mbps", latest_metrics.network_throughput_mbps);
        }
        
        info!("💤 Waiting 10 seconds for next cycle...");
        sleep(Duration::from_secs(10)).await;
    }
    
    info!("🎯 Production monitoring demonstration completed successfully!");
    info!("📊 Key Features Demonstrated:");
    info!("  ✅ Real-time health monitoring");
    info!("  ✅ Component-level health tracking");
    info!("  ✅ Performance metrics collection");
    info!("  ✅ Alert management system");
    info!("  ✅ Auto-recovery capabilities");
    info!("  ✅ Production-ready monitoring");
    
    info!("🌐 Health check endpoints available at:");
    info!("  - GET /health - Basic health check");
    info!("  - GET /health/liveness - Kubernetes liveness probe");
    info!("  - GET /health/readiness - Kubernetes readiness probe");
    info!("  - GET /status - Detailed system status");
    info!("  - GET /metrics - Performance metrics");
    info!("  - GET /monitoring - Comprehensive production monitoring data");
    
    Ok(())
}
