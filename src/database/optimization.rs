// Advanced database optimization for high-frequency trading
// Provides performance monitoring, index optimization, and query tuning

use crate::utils::Result;
use sqlx::{PgPool, Row};
use tracing::info;
use chrono::{DateTime, Utc};
// use serde_json::{json, Value}; // Unused for now
use std::collections::HashMap;

/// Database optimization manager for high-frequency trading performance
pub struct OptimizationManager {
    pool: PgPool,
}

impl OptimizationManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Analyze and optimize database performance
    pub async fn optimize_database(&self) -> Result<OptimizationReport> {
        info!("Starting database optimization analysis...");

        let mut report = OptimizationReport::new();

        // Analyze table statistics
        report.table_stats = self.analyze_table_statistics().await?;
        
        // Analyze index usage
        report.index_analysis = self.analyze_index_usage().await?;
        
        // Check query performance
        report.slow_queries = self.identify_slow_queries().await?;
        
        // Analyze TimescaleDB specific metrics
        report.timescale_metrics = self.analyze_timescale_performance().await?;
        
        // Generate optimization recommendations
        report.recommendations = self.generate_recommendations(&report).await?;

        info!("Database optimization analysis completed");
        Ok(report)
    }

    /// Analyze table statistics for optimization insights
    async fn analyze_table_statistics(&self) -> Result<Vec<TableStatistics>> {
        let stats = sqlx::query(
            r#"
            SELECT
                schemaname||'.'||relname as table_name,
                n_tup_ins as inserts,
                n_tup_upd as updates,
                n_tup_del as deletes,
                n_live_tup as live_tuples,
                n_dead_tup as dead_tuples,
                last_vacuum,
                last_autovacuum,
                last_analyze,
                last_autoanalyze,
                pg_size_pretty(pg_total_relation_size(schemaname||'.'||relname)) as total_size,
                pg_size_pretty(pg_relation_size(schemaname||'.'||relname)) as table_size,
                pg_size_pretty(pg_indexes_size(schemaname||'.'||relname)) as index_size
            FROM pg_stat_user_tables
            WHERE schemaname = 'public'
            ORDER BY pg_total_relation_size(schemaname||'.'||relname) DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut table_stats = Vec::new();
        for row in stats {
            table_stats.push(TableStatistics {
                table_name: row.get("table_name"),
                inserts: row.get::<Option<i64>, _>("inserts").unwrap_or(0),
                updates: row.get::<Option<i64>, _>("updates").unwrap_or(0),
                deletes: row.get::<Option<i64>, _>("deletes").unwrap_or(0),
                live_tuples: row.get::<Option<i64>, _>("live_tuples").unwrap_or(0),
                dead_tuples: row.get::<Option<i64>, _>("dead_tuples").unwrap_or(0),
                last_vacuum: row.get("last_vacuum"),
                last_analyze: row.get("last_analyze"),
                total_size: row.get("total_size"),
                table_size: row.get("table_size"),
                index_size: row.get("index_size"),
            });
        }

        Ok(table_stats)
    }

    /// Analyze index usage and effectiveness
    async fn analyze_index_usage(&self) -> Result<Vec<IndexAnalysis>> {
        let indexes = sqlx::query(
            r#"
            SELECT
                schemaname||'.'||relname as table_name,
                indexrelname as indexname,
                idx_tup_read,
                idx_tup_fetch,
                idx_scan,
                pg_size_pretty(pg_relation_size(indexrelid)) as index_size,
                pg_get_indexdef(indexrelid) as index_definition
            FROM pg_stat_user_indexes
            WHERE schemaname = 'public'
            ORDER BY idx_scan DESC, idx_tup_read DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut index_analysis = Vec::new();
        for row in indexes {
            let scans = row.get::<Option<i64>, _>("idx_scan").unwrap_or(0);
            let reads = row.get::<Option<i64>, _>("idx_tup_read").unwrap_or(0);
            
            index_analysis.push(IndexAnalysis {
                table_name: row.get("table_name"),
                index_name: row.get("indexname"),
                scans,
                tuple_reads: reads,
                tuple_fetches: row.get::<Option<i64>, _>("idx_tup_fetch").unwrap_or(0),
                size: row.get("index_size"),
                definition: row.get("index_definition"),
                efficiency: if scans > 0 { reads as f64 / scans as f64 } else { 0.0 },
                usage_score: self.calculate_index_usage_score(scans, reads),
            });
        }

        Ok(index_analysis)
    }

    /// Identify slow queries that need optimization
    async fn identify_slow_queries(&self) -> Result<Vec<SlowQuery>> {
        // Check if pg_stat_statements is available
        let extension_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM pg_extension WHERE extname = 'pg_stat_statements')"
        )
        .fetch_one(&self.pool)
        .await
        .unwrap_or(false);

        if !extension_exists {
            info!("pg_stat_statements extension not available, skipping slow query analysis");
            return Ok(Vec::new());
        }

        let slow_queries = sqlx::query(
            r#"
            SELECT
                query,
                calls,
                total_exec_time,
                mean_exec_time,
                max_exec_time,
                rows,
                CASE
                    WHEN (shared_blks_hit + shared_blks_read) > 0 THEN
                        (100.0 * shared_blks_hit::float / (shared_blks_hit + shared_blks_read)::float)
                    ELSE 0.0
                END AS hit_percent
            FROM pg_stat_statements
            WHERE query NOT LIKE '%pg_stat_statements%'
            AND mean_exec_time > 10  -- queries taking more than 10ms on average
            ORDER BY mean_exec_time DESC
            LIMIT 20
            "#
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        let mut queries = Vec::new();
        for row in slow_queries {
            queries.push(SlowQuery {
                query: row.get("query"),
                calls: row.get::<Option<i64>, _>("calls").unwrap_or(0),
                total_time: row.get::<Option<f64>, _>("total_exec_time").unwrap_or(0.0),
                mean_time: row.get::<Option<f64>, _>("mean_exec_time").unwrap_or(0.0),
                max_time: row.get::<Option<f64>, _>("max_exec_time").unwrap_or(0.0),
                rows_affected: row.get::<Option<i64>, _>("rows").unwrap_or(0),
                cache_hit_ratio: row.get::<Option<f64>, _>("hit_percent").unwrap_or(0.0),
            });
        }

        Ok(queries)
    }

    /// Analyze TimescaleDB specific performance metrics
    async fn analyze_timescale_performance(&self) -> Result<TimescaleMetrics> {
        // Get hypertable information
        let hypertables = sqlx::query(
            r#"
            SELECT 
                table_name,
                chunk_time_interval,
                compression_enabled,
                (SELECT count(*) FROM timescaledb_information.chunks c 
                 WHERE c.hypertable_name = h.table_name) as total_chunks,
                (SELECT count(*) FROM timescaledb_information.chunks c 
                 WHERE c.hypertable_name = h.table_name AND c.is_compressed = true) as compressed_chunks
            FROM timescaledb_information.hypertables h
            WHERE h.schema_name = 'public'
            "#
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        let mut hypertable_info = Vec::new();
        for row in hypertables {
            hypertable_info.push(HypertableInfo {
                table_name: row.get("table_name"),
                chunk_interval: row.get("chunk_time_interval"),
                compression_enabled: row.get::<Option<bool>, _>("compression_enabled").unwrap_or(false),
                total_chunks: row.get::<Option<i64>, _>("total_chunks").unwrap_or(0),
                compressed_chunks: row.get::<Option<i64>, _>("compressed_chunks").unwrap_or(0),
            });
        }

        // Get compression ratio
        let compression_stats = sqlx::query(
            r#"
            SELECT 
                hypertable_name,
                before_compression_total_bytes,
                after_compression_total_bytes,
                CASE 
                    WHEN before_compression_total_bytes > 0 THEN
                        100.0 * (1.0 - after_compression_total_bytes::float / before_compression_total_bytes::float)
                    ELSE 0
                END as compression_ratio
            FROM timescaledb_information.compression_settings
            WHERE hypertable_schema = 'public'
            "#
        )
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        let mut compression_ratios = HashMap::new();
        for row in compression_stats {
            let table_name: String = row.get("hypertable_name");
            let ratio: Option<f64> = row.get("compression_ratio");
            compression_ratios.insert(table_name, ratio.unwrap_or(0.0));
        }

        Ok(TimescaleMetrics {
            hypertables: hypertable_info,
            compression_ratios,
        })
    }

    /// Generate optimization recommendations based on analysis
    async fn generate_recommendations(&self, report: &OptimizationReport) -> Result<Vec<OptimizationRecommendation>> {
        let mut recommendations = Vec::new();

        // Check for unused indexes
        for index in &report.index_analysis {
            if index.scans < 10 && index.usage_score < 0.1 {
                recommendations.push(OptimizationRecommendation {
                    category: "Index Optimization".to_string(),
                    priority: "Medium".to_string(),
                    description: format!("Consider dropping unused index: {}", index.index_name),
                    action: format!("DROP INDEX IF EXISTS {}", index.index_name),
                    impact: "Reduced storage and faster writes".to_string(),
                });
            }
        }

        // Check for tables needing vacuum
        for table in &report.table_stats {
            let dead_ratio = if table.live_tuples > 0 {
                table.dead_tuples as f64 / table.live_tuples as f64
            } else {
                0.0
            };

            if dead_ratio > 0.1 {
                recommendations.push(OptimizationRecommendation {
                    category: "Maintenance".to_string(),
                    priority: "High".to_string(),
                    description: format!("Table {} needs vacuum ({}% dead tuples)", table.table_name, (dead_ratio * 100.0) as i32),
                    action: format!("VACUUM ANALYZE {}", table.table_name),
                    impact: "Improved query performance and reduced bloat".to_string(),
                });
            }
        }

        // Check TimescaleDB compression opportunities
        for hypertable in &report.timescale_metrics.hypertables {
            if !hypertable.compression_enabled && hypertable.total_chunks > 5 {
                recommendations.push(OptimizationRecommendation {
                    category: "TimescaleDB Optimization".to_string(),
                    priority: "Medium".to_string(),
                    description: format!("Enable compression for hypertable: {}", hypertable.table_name),
                    action: format!("ALTER TABLE {} SET (timescaledb.compress)", hypertable.table_name),
                    impact: "Significant storage reduction and improved query performance".to_string(),
                });
            }
        }

        // Check for slow queries
        for query in &report.slow_queries {
            if query.mean_time > 100.0 {
                recommendations.push(OptimizationRecommendation {
                    category: "Query Optimization".to_string(),
                    priority: "High".to_string(),
                    description: format!("Slow query detected (avg: {:.2}ms)", query.mean_time),
                    action: "Review query execution plan and consider adding indexes".to_string(),
                    impact: "Faster query execution and reduced system load".to_string(),
                });
            }
        }

        Ok(recommendations)
    }

    /// Calculate index usage score (0.0 to 1.0)
    fn calculate_index_usage_score(&self, scans: i64, reads: i64) -> f64 {
        if scans == 0 {
            return 0.0;
        }
        
        let scan_score = (scans as f64).ln() / 10.0; // Logarithmic scaling
        let read_score = (reads as f64).ln() / 15.0;
        
        (scan_score + read_score).min(1.0).max(0.0)
    }

    /// Apply automatic optimizations
    pub async fn apply_auto_optimizations(&self) -> Result<Vec<String>> {
        let mut applied = Vec::new();

        // Update table statistics
        let tables = ["market_ticks", "ai_predictions", "trading_signals", "microstructure_analysis"];
        for table in tables {
            sqlx::query(&format!("ANALYZE {}", table))
                .execute(&self.pool)
                .await?;
            applied.push(format!("Updated statistics for {}", table));
        }

        // Refresh materialized views if any exist
        // (This would be added when we create materialized views)

        info!("Applied {} automatic optimizations", applied.len());
        Ok(applied)
    }
}

// Data structures for optimization reporting

#[derive(Debug)]
pub struct OptimizationReport {
    pub table_stats: Vec<TableStatistics>,
    pub index_analysis: Vec<IndexAnalysis>,
    pub slow_queries: Vec<SlowQuery>,
    pub timescale_metrics: TimescaleMetrics,
    pub recommendations: Vec<OptimizationRecommendation>,
}

impl OptimizationReport {
    fn new() -> Self {
        Self {
            table_stats: Vec::new(),
            index_analysis: Vec::new(),
            slow_queries: Vec::new(),
            timescale_metrics: TimescaleMetrics::default(),
            recommendations: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct TableStatistics {
    pub table_name: String,
    pub inserts: i64,
    pub updates: i64,
    pub deletes: i64,
    pub live_tuples: i64,
    pub dead_tuples: i64,
    pub last_vacuum: Option<DateTime<Utc>>,
    pub last_analyze: Option<DateTime<Utc>>,
    pub total_size: String,
    pub table_size: String,
    pub index_size: String,
}

#[derive(Debug)]
pub struct IndexAnalysis {
    pub table_name: String,
    pub index_name: String,
    pub scans: i64,
    pub tuple_reads: i64,
    pub tuple_fetches: i64,
    pub size: String,
    pub definition: String,
    pub efficiency: f64,
    pub usage_score: f64,
}

#[derive(Debug)]
pub struct SlowQuery {
    pub query: String,
    pub calls: i64,
    pub total_time: f64,
    pub mean_time: f64,
    pub max_time: f64,
    pub rows_affected: i64,
    pub cache_hit_ratio: f64,
}

#[derive(Debug, Default)]
pub struct TimescaleMetrics {
    pub hypertables: Vec<HypertableInfo>,
    pub compression_ratios: HashMap<String, f64>,
}

#[derive(Debug)]
pub struct HypertableInfo {
    pub table_name: String,
    pub chunk_interval: Option<String>,
    pub compression_enabled: bool,
    pub total_chunks: i64,
    pub compressed_chunks: i64,
}

#[derive(Debug)]
pub struct OptimizationRecommendation {
    pub category: String,
    pub priority: String,
    pub description: String,
    pub action: String,
    pub impact: String,
}
