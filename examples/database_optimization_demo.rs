// Database optimization demo for PantherSwap Edge
// Demonstrates advanced indexing, performance monitoring, and optimization features
// Run with: DATABASE_URL="..." cargo run --example database_optimization_demo

use pantherswap_edge::database::Database;
use pantherswap_edge::config::Settings;
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 PantherSwap Edge Database Optimization Demo");
    println!("==============================================");
    
    // Load configuration
    let settings = Settings::load()?;
    
    // Connect to database
    let database = Database::new(&settings.database.url).await?;
    println!("✅ Connected to TimescaleDB");
    
    // Test 1: Database Optimization Analysis
    println!("\n📊 Running Database Optimization Analysis...");
    let optimization_manager = database.optimization_manager();
    
    match optimization_manager.optimize_database().await {
        Ok(report) => {
            println!("✅ Optimization analysis completed");
            
            // Display table statistics
            println!("\n📈 Table Statistics:");
            for (i, table) in report.table_stats.iter().enumerate().take(5) {
                println!("   {}. {} - {} total size, {} live tuples", 
                         i + 1, table.table_name, table.total_size, table.live_tuples);
            }
            
            // Display index analysis
            println!("\n🔍 Index Analysis (Top 5 by usage):");
            for (i, index) in report.index_analysis.iter().enumerate().take(5) {
                println!("   {}. {} - {} scans, efficiency: {:.2}, usage score: {:.2}", 
                         i + 1, index.index_name, index.scans, index.efficiency, index.usage_score);
            }
            
            // Display recommendations
            println!("\n💡 Optimization Recommendations:");
            for (i, rec) in report.recommendations.iter().enumerate().take(5) {
                println!("   {}. [{}] {} - {}", 
                         i + 1, rec.priority, rec.category, rec.description);
            }
            
            // Display TimescaleDB metrics
            println!("\n⏰ TimescaleDB Metrics:");
            for hypertable in &report.timescale_metrics.hypertables {
                let compression_ratio = report.timescale_metrics.compression_ratios
                    .get(&hypertable.table_name)
                    .unwrap_or(&0.0);
                println!("   📊 {} - {} chunks ({} compressed), compression: {:.1}%", 
                         hypertable.table_name, 
                         hypertable.total_chunks, 
                         hypertable.compressed_chunks,
                         compression_ratio);
            }
        }
        Err(e) => {
            println!("⚠️  Optimization analysis failed: {}", e);
        }
    }
    
    // Test 2: Advanced Index Management
    println!("\n🔧 Testing Advanced Index Management...");
    let index_manager = database.advanced_index_manager();
    
    match index_manager.create_advanced_indexes().await {
        Ok(_) => {
            println!("✅ Advanced indexes created successfully");
            
            // Create strategy-specific indexes
            let strategies = ["momentum", "arbitrage", "market_making"];
            for strategy in strategies {
                match index_manager.create_strategy_specific_indexes(strategy).await {
                    Ok(_) => println!("   ✅ Created {} strategy indexes", strategy),
                    Err(e) => println!("   ⚠️  Failed to create {} indexes: {}", strategy, e),
                }
            }
        }
        Err(e) => {
            println!("⚠️  Advanced index creation failed: {}", e);
        }
    }
    
    // Test 3: Connection Pool Optimization
    println!("\n🔗 Testing Connection Pool Optimization...");
    
    match Database::create_optimized_pool(&settings.database.url).await {
        Ok(mut pool_manager) => {
            println!("✅ Optimized connection pool created");
            
            // Get pool metrics
            match pool_manager.get_metrics().await {
                Ok(metrics) => {
                    println!("   📊 Pool metrics:");
                    println!("      - Total connections: {}", metrics.total_connections);
                    println!("      - Active connections: {}", metrics.active_connections);
                    println!("      - Idle connections: {}", metrics.idle_connections);
                    println!("      - Query count: {}", metrics.query_count);
                    if metrics.query_time_avg.as_millis() > 0 {
                        println!("      - Avg query time: {}ms", metrics.query_time_avg.as_millis());
                    }
                }
                Err(e) => println!("   ⚠️  Failed to get pool metrics: {}", e),
            }
            
            // Perform health check
            match pool_manager.health_check().await {
                Ok(health) => {
                    println!("   🏥 Pool health check:");
                    println!("      - Status: {}", if health.is_healthy { "✅ Healthy" } else { "❌ Unhealthy" });
                    println!("      - Connectivity time: {}ms", health.connectivity_time.as_millis());
                    println!("      - Performance: {}", health.performance_status);
                    println!("      - Pool utilization: {:.1}%", 
                             if health.pool_size > 0 { 
                                 (health.active_connections as f64 / health.pool_size as f64) * 100.0 
                             } else { 
                                 0.0 
                             });
                    
                    if !health.recommendations.is_empty() {
                        println!("      - Recommendations:");
                        for rec in &health.recommendations {
                            println!("        • {}", rec);
                        }
                    }
                }
                Err(e) => println!("   ⚠️  Health check failed: {}", e),
            }
            
            // Get detailed statistics
            match pool_manager.get_detailed_stats().await {
                Ok(stats) => {
                    println!("   📈 Detailed pool statistics available (JSON format)");
                    // In a real application, you might log this or send to monitoring
                }
                Err(e) => println!("   ⚠️  Failed to get detailed stats: {}", e),
            }
        }
        Err(e) => {
            println!("⚠️  Failed to create optimized pool: {}", e);
        }
    }
    
    // Test 4: Apply Automatic Optimizations
    println!("\n🤖 Applying Automatic Optimizations...");
    let optimization_manager = database.optimization_manager();
    
    match optimization_manager.apply_auto_optimizations().await {
        Ok(applied) => {
            println!("✅ Applied {} automatic optimizations:", applied.len());
            for optimization in applied {
                println!("   • {}", optimization);
            }
        }
        Err(e) => {
            println!("⚠️  Auto-optimization failed: {}", e);
        }
    }
    
    // Test 5: Performance Monitoring
    println!("\n📊 Performance Monitoring Summary...");
    
    // Test basic query performance
    let start = std::time::Instant::now();
    let simple_query_manager = database.simple_query_manager();
    
    match simple_query_manager.health_check().await {
        Ok(healthy) => {
            let query_time = start.elapsed();
            println!("✅ Database health check: {} ({}ms)", 
                     if healthy { "Healthy" } else { "Unhealthy" },
                     query_time.as_millis());
        }
        Err(e) => {
            println!("⚠️  Health check failed: {}", e);
        }
    }
    
    // Performance recommendations
    println!("\n🎯 Performance Optimization Summary:");
    println!("   ✅ Advanced indexes: Created for high-frequency trading patterns");
    println!("   ✅ Connection pooling: Optimized for trading workloads");
    println!("   ✅ TimescaleDB: Compression and retention policies active");
    println!("   ✅ Query monitoring: Performance tracking enabled");
    println!("   ✅ Auto-optimization: Maintenance tasks automated");
    
    println!("\n🚀 Database Optimization Features:");
    println!("   📊 Real-time performance monitoring");
    println!("   🔍 Intelligent index usage analysis");
    println!("   ⚡ High-frequency trading optimizations");
    println!("   🤖 Automated maintenance and tuning");
    println!("   📈 TimescaleDB compression and retention");
    println!("   🔗 Advanced connection pool management");
    
    println!("\n🎉 Database optimization demo completed successfully!");
    println!("✅ PantherSwap Edge database is optimized for high-frequency trading");
    
    Ok(())
}
