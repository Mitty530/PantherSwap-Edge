// Simple database health monitoring demo for PantherSwap Edge
// Demonstrates basic health monitoring capabilities with cloud-friendly settings
// Run with: DATABASE_URL="..." cargo run --example simple_health_monitoring_demo

use pantherswap_edge::database::{Database, HealthMonitorConfig, AlertConfig};
use pantherswap_edge::config::Settings;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🏥 PantherSwap Edge Simple Health Monitoring Demo");
    println!("=================================================");
    
    // Load configuration
    let settings = Settings::load()?;
    let database_url = &settings.database.url;
    
    // Test 1: Basic Health Check
    println!("\n🔍 Testing Basic Health Check...");
    
    let database = Database::new_cloud(database_url).await?;
    println!("✅ Database connection established");
    
    // Basic health check
    let is_healthy = database.health_check().await?;
    println!("✅ Basic health check: {}", if is_healthy { "HEALTHY ✅" } else { "UNHEALTHY ❌" });
    
    // Pool health check
    let pool_health = database.pool_health_check().await?;
    println!("✅ Pool health check:");
    println!("   - Status: {}", if pool_health.is_healthy { "HEALTHY ✅" } else { "UNHEALTHY ❌" });
    println!("   - Connectivity time: {:?}", pool_health.connectivity_time);
    println!("   - Pool utilization: {:.1}%", pool_health.utilization_percent);
    println!("   - Performance rating: {}", pool_health.performance_status);
    println!("   - Pool size: {}/{}", pool_health.pool_stats.size, pool_health.pool_stats.max_size);
    println!("   - Active connections: {}", pool_health.pool_stats.active);
    println!("   - Idle connections: {}", pool_health.pool_stats.idle);

    // Test 2: Comprehensive Health Monitoring
    println!("\n📊 Testing Comprehensive Health Monitoring...");
    
    let health_report = database.comprehensive_health_check().await?;
    println!("✅ Comprehensive health check completed:");
    println!("   - Overall status: {:?}", health_report.overall_status);
    println!("   - Health score: {:.2}/100", health_report.overall_score);
    println!("   - Total alerts: {}", health_report.current_metrics.alerts.len());
    println!("   - Critical alerts: {}", health_report.critical_alerts.len());
    
    // Display connectivity metrics
    let conn_metrics = &health_report.current_metrics.connectivity;
    println!("   - Connection status: {}", if conn_metrics.is_connected { "Connected ✅" } else { "Disconnected ❌" });
    println!("   - Connection time: {}ms", conn_metrics.connection_time_ms);
    println!("   - Pool utilization: {:.1}%", conn_metrics.connection_utilization_percent);
    println!("   - Active/Max connections: {}/{}", conn_metrics.active_connections, conn_metrics.max_connections);
    
    // Display performance metrics
    let perf_metrics = &health_report.current_metrics.performance;
    println!("   - Query response time: {}ms", perf_metrics.query_response_time_ms);
    println!("   - Cache hit ratio: {:.1}%", perf_metrics.cache_hit_ratio);
    println!("   - Transactions/sec: {:.2}", perf_metrics.transactions_per_second);
    println!("   - Index usage ratio: {:.1}%", perf_metrics.index_usage_ratio);
    println!("   - Slow queries: {}", perf_metrics.slow_queries_count);
    println!("   - Blocked queries: {}", perf_metrics.blocked_queries_count);
    println!("   - Deadlocks: {}", perf_metrics.deadlocks_count);
    
    // Display resource metrics
    let resource_metrics = &health_report.current_metrics.resource_usage;
    println!("   - CPU usage: {:.1}%", resource_metrics.cpu_usage_percent);
    println!("   - Memory usage: {:.1}%", resource_metrics.memory_usage_percent);
    println!("   - Disk usage: {:.1}%", resource_metrics.disk_usage_percent);
    println!("   - Disk I/O read: {:.2} MB/s", resource_metrics.disk_io_read_mb_per_sec);
    println!("   - Disk I/O write: {:.2} MB/s", resource_metrics.disk_io_write_mb_per_sec);
    
    // Display database statistics
    let db_stats = &health_report.current_metrics.database_stats;
    println!("   - Database size: {:.2} MB", db_stats.total_size_mb);
    println!("   - Tables: {}", db_stats.table_count);
    println!("   - Indexes: {}", db_stats.index_count);
    println!("   - Total transactions: {}", db_stats.total_transactions);
    println!("   - Committed transactions: {}", db_stats.committed_transactions);
    println!("   - Rolled back transactions: {}", db_stats.rolled_back_transactions);
    println!("   - Cache efficiency: {:.1}%", 
             if db_stats.blocks_read + db_stats.blocks_hit > 0 {
                 (db_stats.blocks_hit as f64 / (db_stats.blocks_read + db_stats.blocks_hit) as f64) * 100.0
             } else {
                 100.0
             });
    
    // Display TimescaleDB metrics if available
    if let Some(ts_metrics) = &health_report.current_metrics.timescale_metrics {
        println!("   - TimescaleDB status: Available ✅");
        println!("   - Hypertables: {}", ts_metrics.hypertables_count);
        println!("   - Total chunks: {}", ts_metrics.total_chunks);
        println!("   - Compressed chunks: {}", ts_metrics.compressed_chunks);
        println!("   - Compression ratio: {:.1}%", ts_metrics.compression_ratio);
        println!("   - Retention policy: {}", if ts_metrics.retention_policy_active { "Active ✅" } else { "Inactive ⚠️" });
    } else {
        println!("   - TimescaleDB status: Not available or not detected");
    }

    // Test 3: Health Alerts Analysis
    println!("\n🚨 Testing Health Alerts Analysis...");
    
    if !health_report.current_metrics.alerts.is_empty() {
        println!("✅ Active alerts detected:");
        for (i, alert) in health_report.current_metrics.alerts.iter().enumerate() {
            let severity_icon = match alert.severity {
                pantherswap_edge::database::AlertSeverity::Emergency => "🚨",
                pantherswap_edge::database::AlertSeverity::Critical => "🔴",
                pantherswap_edge::database::AlertSeverity::Warning => "⚠️",
                pantherswap_edge::database::AlertSeverity::Info => "ℹ️",
            };
            println!("   {}. {} [{:?}] {:?}: {}", 
                     i + 1, severity_icon, alert.severity, alert.alert_type, alert.message);
            println!("      Value: {:.2}, Threshold: {:.2}", alert.metric_value, alert.threshold);
        }
    } else {
        println!("✅ No active alerts - system is healthy");
    }
    
    if !health_report.critical_alerts.is_empty() {
        println!("🔴 Critical alerts requiring immediate attention:");
        for (i, alert) in health_report.critical_alerts.iter().enumerate() {
            println!("   {}. {:?}: {}", i + 1, alert.alert_type, alert.message);
        }
    }

    // Test 4: Health Recommendations
    println!("\n💡 Testing Health Recommendations...");
    
    if !health_report.recommendations.is_empty() {
        println!("✅ Health recommendations:");
        for (i, recommendation) in health_report.recommendations.iter().enumerate() {
            println!("   {}. {}", i + 1, recommendation);
        }
    } else {
        println!("✅ No recommendations - system is optimally configured");
    }

    // Test 5: Health Trends (if available)
    println!("\n📈 Testing Health Trends...");
    
    println!("✅ Health trends analysis:");
    println!("   - Performance trend: {:?}", health_report.trends.performance_trend);
    println!("   - Resource usage trend: {:?}", health_report.trends.resource_usage_trend);
    println!("   - Connection trend: {:?}", health_report.trends.connection_trend);
    println!("   - Error rate trend: {:?}", health_report.trends.error_rate_trend);

    // Test 6: Custom Health Monitor Configuration
    println!("\n⚙️  Testing Custom Health Monitor Configuration...");
    
    let custom_config = HealthMonitorConfig {
        check_interval_seconds: 30,
        metrics_retention_hours: 24,
        enable_continuous_monitoring: false, // Disabled for demo
        enable_alerting: true,
        max_history_size: 100,
    };
    
    let custom_monitor = database.health_monitor_with_config(custom_config);
    println!("✅ Custom health monitor created with 30s intervals");
    
    // Get current thresholds
    let thresholds = custom_monitor.get_thresholds();
    println!("✅ Current alert thresholds:");
    println!("   - Connection time warning: {}ms", thresholds.connection_time_warning_ms);
    println!("   - Connection time critical: {}ms", thresholds.connection_time_critical_ms);
    println!("   - Query response warning: {}ms", thresholds.query_response_warning_ms);
    println!("   - Query response critical: {}ms", thresholds.query_response_critical_ms);
    println!("   - Cache hit ratio warning: {:.1}%", thresholds.cache_hit_ratio_warning_percent);
    println!("   - CPU usage warning: {:.1}%", thresholds.cpu_usage_warning_percent);
    println!("   - Memory usage warning: {:.1}%", thresholds.memory_usage_warning_percent);

    // Test 7: Alert Manager Configuration
    println!("\n📢 Testing Alert Manager Configuration...");
    
    let alert_config = AlertConfig {
        enable_notifications: true,
        alert_cooldown_minutes: 5,
        max_alerts_per_hour: 20,
        escalation_enabled: true,
        escalation_threshold_minutes: 15,
        alert_retention_hours: 168, // 7 days
    };
    
    let alert_manager = database.alert_manager_with_config(alert_config);
    println!("✅ Alert manager configured:");
    println!("   - Notifications: enabled");
    println!("   - Cooldown period: 5 minutes");
    println!("   - Rate limit: 20 alerts/hour");
    println!("   - Escalation: enabled (15 min threshold)");
    println!("   - Retention: 7 days");
    
    // Get alert statistics
    let alert_stats = alert_manager.get_alert_statistics();
    println!("✅ Alert statistics:");
    println!("   - Total alerts: {}", alert_stats.total_alerts);
    println!("   - Alerts last hour: {}", alert_stats.alerts_last_hour);
    println!("   - Alerts last day: {}", alert_stats.alerts_last_day);
    println!("   - Critical alerts last day: {}", alert_stats.critical_alerts_last_day);
    println!("   - Unresolved alerts: {}", alert_stats.unresolved_alerts);
    println!("   - Notification channels: {}", alert_stats.notification_channels_count);

    // Test 8: Health Status Summary
    println!("\n📋 Health Status Summary...");
    
    let status_icon = match health_report.overall_status {
        pantherswap_edge::database::HealthStatus::Healthy => "✅",
        pantherswap_edge::database::HealthStatus::Warning => "⚠️",
        pantherswap_edge::database::HealthStatus::Critical => "🔴",
        pantherswap_edge::database::HealthStatus::Emergency => "🚨",
    };
    
    println!("✅ Overall Database Health Status:");
    println!("   {} Status: {:?}", status_icon, health_report.overall_status);
    println!("   📊 Health Score: {:.1}/100", health_report.overall_score);
    println!("   🔗 Connectivity: {} ({}ms)", 
             if conn_metrics.is_connected { "✅" } else { "❌" },
             conn_metrics.connection_time_ms);
    println!("   ⚡ Performance: {} ({}ms query time)", 
             if perf_metrics.query_response_time_ms < 100 { "✅" } else { "⚠️" },
             perf_metrics.query_response_time_ms);
    println!("   💾 Resources: CPU {:.1}%, Memory {:.1}%, Disk {:.1}%",
             resource_metrics.cpu_usage_percent,
             resource_metrics.memory_usage_percent,
             resource_metrics.disk_usage_percent);
    println!("   📈 Database: {:.1}MB, {} tables, {} indexes",
             db_stats.total_size_mb,
             db_stats.table_count,
             db_stats.index_count);

    // Cleanup
    println!("\n🧹 Cleaning up...");
    
    // Close database connection
    database.close().await;
    println!("✅ Database connection closed");

    println!("\n🎉 Simple Health Monitoring Demo Completed Successfully!");
    println!("========================================================");
    println!("✅ Basic health checks working");
    println!("✅ Comprehensive health monitoring operational");
    println!("✅ Health alerts analysis functional");
    println!("✅ Health recommendations generated");
    println!("✅ Health trends tracking working");
    println!("✅ Custom health monitor configuration tested");
    println!("✅ Alert manager configuration validated");
    println!("✅ Health status summary generated");
    
    Ok(())
}
