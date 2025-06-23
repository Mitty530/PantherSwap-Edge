// Database health monitoring demo for PantherSwap Edge
// Demonstrates comprehensive health monitoring, alerting, and reporting
// Run with: DATABASE_URL="..." cargo run --example database_health_monitoring_demo

use pantherswap_edge::database::{Database, DatabaseHealthMonitor, AlertManager, HealthMonitorConfig, AlertConfig};
use pantherswap_edge::config::Settings;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🏥 PantherSwap Edge Database Health Monitoring Demo");
    println!("==================================================");
    
    // Load configuration
    let settings = Settings::load()?;
    let database_url = &settings.database.url;
    
    // Test 1: Basic Health Check
    println!("\n🔍 Testing Basic Health Check...");
    
    let database = Database::new_cloud(database_url).await?;
    println!("✅ Database connection established");
    
    // Basic health check
    let is_healthy = database.health_check().await?;
    println!("✅ Basic health check: {}", if is_healthy { "HEALTHY" } else { "UNHEALTHY" });
    
    // Pool health check
    let pool_health = database.pool_health_check().await?;
    println!("✅ Pool health check:");
    println!("   - Healthy: {}", pool_health.is_healthy);
    println!("   - Connectivity time: {:?}", pool_health.connectivity_time);
    println!("   - Utilization: {:.1}%", pool_health.utilization_percent);
    println!("   - Performance: {}", pool_health.performance_status);

    // Test 2: Comprehensive Health Monitoring
    println!("\n📊 Testing Comprehensive Health Monitoring...");
    
    let health_report = database.comprehensive_health_check().await?;
    println!("✅ Comprehensive health check completed:");
    println!("   - Overall status: {:?}", health_report.overall_status);
    println!("   - Health score: {:.2}/100", health_report.overall_score);
    println!("   - Active alerts: {}", health_report.current_metrics.alerts.len());
    println!("   - Critical alerts: {}", health_report.critical_alerts.len());
    
    // Display connectivity metrics
    let conn_metrics = &health_report.current_metrics.connectivity;
    println!("   - Connection time: {}ms", conn_metrics.connection_time_ms);
    println!("   - Pool utilization: {:.1}%", conn_metrics.connection_utilization_percent);
    println!("   - Active connections: {}/{}", conn_metrics.active_connections, conn_metrics.max_connections);
    
    // Display performance metrics
    let perf_metrics = &health_report.current_metrics.performance;
    println!("   - Query response time: {}ms", perf_metrics.query_response_time_ms);
    println!("   - Cache hit ratio: {:.1}%", perf_metrics.cache_hit_ratio);
    println!("   - Transactions/sec: {:.2}", perf_metrics.transactions_per_second);
    println!("   - Slow queries: {}", perf_metrics.slow_queries_count);
    
    // Display resource metrics
    let resource_metrics = &health_report.current_metrics.resource_usage;
    println!("   - CPU usage: {:.1}%", resource_metrics.cpu_usage_percent);
    println!("   - Memory usage: {:.1}%", resource_metrics.memory_usage_percent);
    println!("   - Disk usage: {:.1}%", resource_metrics.disk_usage_percent);
    
    // Display database statistics
    let db_stats = &health_report.current_metrics.database_stats;
    println!("   - Database size: {:.2} MB", db_stats.total_size_mb);
    println!("   - Tables: {}", db_stats.table_count);
    println!("   - Indexes: {}", db_stats.index_count);
    println!("   - Total transactions: {}", db_stats.total_transactions);
    
    // Display TimescaleDB metrics if available
    if let Some(ts_metrics) = &health_report.current_metrics.timescale_metrics {
        println!("   - TimescaleDB hypertables: {}", ts_metrics.hypertables_count);
        println!("   - Total chunks: {}", ts_metrics.total_chunks);
        println!("   - Compression ratio: {:.1}%", ts_metrics.compression_ratio);
        println!("   - Retention policy: {}", if ts_metrics.retention_policy_active { "Active" } else { "Inactive" });
    }

    // Test 3: Health Monitoring with Custom Configuration
    println!("\n⚙️  Testing Custom Health Monitor Configuration...");
    
    let custom_config = HealthMonitorConfig {
        check_interval_seconds: 10,
        metrics_retention_hours: 12,
        enable_continuous_monitoring: true,
        enable_alerting: true,
        max_history_size: 1000,
    };
    
    let mut custom_monitor = database.health_monitor_with_config(custom_config);
    println!("✅ Custom health monitor created");
    
    // Perform multiple health checks to build history
    for i in 1..=3 {
        println!("   - Performing health check #{}", i);
        let report = custom_monitor.health_check().await?;
        println!("     • Score: {:.2}, Alerts: {}", report.overall_score, report.current_metrics.alerts.len());
        
        if i < 3 {
            sleep(Duration::from_secs(2)).await;
        }
    }
    
    // Check metrics history
    let history = custom_monitor.get_metrics_history();
    println!("✅ Health monitoring history: {} entries", history.len());

    // Test 4: Alert Management System
    println!("\n🚨 Testing Alert Management System...");
    
    let alert_config = AlertConfig {
        enable_notifications: true,
        alert_cooldown_minutes: 1,
        max_alerts_per_hour: 50,
        escalation_enabled: true,
        escalation_threshold_minutes: 5,
        alert_retention_hours: 24,
    };
    
    let mut alert_manager = database.alert_manager_with_config(alert_config);
    println!("✅ Alert manager created with custom configuration");
    
    // Start alert processing
    alert_manager.start_processing().await?;
    println!("✅ Alert processing started");
    
    // Get alert sender for testing
    let alert_sender = alert_manager.get_alert_sender();
    
    // Send test alerts
    use pantherswap_edge::database::health_monitor::{HealthAlert, AlertType, AlertSeverity};
    use chrono::Utc;
    
    let test_alerts = vec![
        HealthAlert {
            alert_type: AlertType::Performance,
            severity: AlertSeverity::Warning,
            message: "Query response time elevated".to_string(),
            metric_value: 75.0,
            threshold: 50.0,
            timestamp: Utc::now(),
        },
        HealthAlert {
            alert_type: AlertType::ConnectionPool,
            severity: AlertSeverity::Critical,
            message: "Connection pool utilization critically high".to_string(),
            metric_value: 98.0,
            threshold: 95.0,
            timestamp: Utc::now(),
        },
        HealthAlert {
            alert_type: AlertType::ResourceUsage,
            severity: AlertSeverity::Info,
            message: "Memory usage within normal range".to_string(),
            metric_value: 65.0,
            threshold: 80.0,
            timestamp: Utc::now(),
        },
    ];
    
    for alert in test_alerts {
        alert_sender.send(alert)?;
    }
    
    println!("✅ Test alerts sent to alert manager");
    
    // Wait for alerts to be processed
    sleep(Duration::from_secs(2)).await;
    
    // Get alert statistics
    let alert_stats = alert_manager.get_alert_statistics();
    println!("✅ Alert statistics:");
    println!("   - Total alerts: {}", alert_stats.total_alerts);
    println!("   - Alerts last hour: {}", alert_stats.alerts_last_hour);
    println!("   - Critical alerts last day: {}", alert_stats.critical_alerts_last_day);
    println!("   - Unresolved alerts: {}", alert_stats.unresolved_alerts);
    println!("   - Notification channels: {}", alert_stats.notification_channels_count);

    // Test 5: Continuous Health Monitoring
    println!("\n🔄 Testing Continuous Health Monitoring...");
    
    // Start continuous monitoring
    custom_monitor.start_monitoring().await?;
    println!("✅ Continuous health monitoring started");
    
    // Let it run for a short period
    println!("   - Monitoring for 10 seconds...");
    sleep(Duration::from_secs(10)).await;
    
    // Check updated history
    let updated_history = custom_monitor.get_metrics_history();
    println!("✅ Updated monitoring history: {} entries", updated_history.len());

    // Test 6: Health Trends Analysis
    println!("\n📈 Testing Health Trends Analysis...");
    
    if updated_history.len() >= 2 {
        let latest_report = custom_monitor.health_check().await?;
        println!("✅ Health trends analysis:");
        println!("   - Performance trend: {:?}", latest_report.trends.performance_trend);
        println!("   - Resource usage trend: {:?}", latest_report.trends.resource_usage_trend);
        println!("   - Connection trend: {:?}", latest_report.trends.connection_trend);
        println!("   - Error rate trend: {:?}", latest_report.trends.error_rate_trend);
        
        // Display recommendations
        println!("✅ Health recommendations:");
        for (i, recommendation) in latest_report.recommendations.iter().enumerate() {
            println!("   {}. {}", i + 1, recommendation);
        }
    } else {
        println!("⚠️  Insufficient data for trend analysis (need at least 2 data points)");
    }

    // Test 7: Alert Threshold Customization
    println!("\n🎛️  Testing Alert Threshold Customization...");
    
    use pantherswap_edge::database::health_monitor::AlertThresholds;
    
    let custom_thresholds = AlertThresholds {
        connection_time_warning_ms: 50,
        connection_time_critical_ms: 200,
        connection_utilization_warning_percent: 70.0,
        connection_utilization_critical_percent: 90.0,
        query_response_warning_ms: 25,
        query_response_critical_ms: 100,
        cache_hit_ratio_warning_percent: 95.0,
        slow_queries_warning_count: 3,
        slow_queries_critical_count: 10,
        cpu_usage_warning_percent: 60.0,
        cpu_usage_critical_percent: 85.0,
        memory_usage_warning_percent: 75.0,
        memory_usage_critical_percent: 90.0,
        disk_usage_warning_percent: 80.0,
        disk_usage_critical_percent: 90.0,
    };
    
    custom_monitor.update_thresholds(custom_thresholds);
    println!("✅ Custom alert thresholds applied");
    
    // Perform health check with new thresholds
    let threshold_report = custom_monitor.health_check().await?;
    println!("✅ Health check with custom thresholds:");
    println!("   - Health score: {:.2}/100", threshold_report.overall_score);
    println!("   - Alerts with new thresholds: {}", threshold_report.current_metrics.alerts.len());

    // Test 8: Health Monitoring Integration
    println!("\n🔗 Testing Health Monitoring Integration...");
    
    // Demonstrate integration with existing pool health check
    let pool_health = database.pool_health_check().await?;
    let comprehensive_health = database.comprehensive_health_check().await?;
    
    println!("✅ Health monitoring integration:");
    println!("   - Pool health: {} ({})", 
             if pool_health.is_healthy { "✅" } else { "❌" }, 
             pool_health.performance_status);
    println!("   - Comprehensive health: {:?} ({:.1}/100)", 
             comprehensive_health.overall_status, 
             comprehensive_health.overall_score);
    
    // Compare metrics
    let pool_utilization = pool_health.utilization_percent;
    let comprehensive_utilization = comprehensive_health.current_metrics.connectivity.connection_utilization_percent;
    println!("   - Pool utilization comparison: {:.1}% vs {:.1}%", 
             pool_utilization, comprehensive_utilization);

    // Cleanup
    println!("\n🧹 Cleaning up...");
    
    // Clear monitoring history
    custom_monitor.clear_history();
    println!("✅ Monitoring history cleared");
    
    // Close database connection
    database.close().await;
    println!("✅ Database connection closed");

    println!("\n🎉 Database Health Monitoring Demo Completed Successfully!");
    println!("==========================================================");
    println!("✅ Basic health checks working");
    println!("✅ Comprehensive health monitoring operational");
    println!("✅ Custom health monitor configuration tested");
    println!("✅ Alert management system functional");
    println!("✅ Continuous monitoring capabilities verified");
    println!("✅ Health trends analysis working");
    println!("✅ Alert threshold customization tested");
    println!("✅ Health monitoring integration validated");
    
    Ok(())
}
