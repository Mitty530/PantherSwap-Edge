// Advanced indexing strategies for high-frequency trading optimization
// Specialized indexes for trading patterns, real-time analytics, and performance

use crate::utils::Result;
use sqlx::{PgPool, Row};
use tracing::info;

/// Advanced indexing manager for high-frequency trading optimization
pub struct AdvancedIndexManager {
    pool: PgPool,
}

impl AdvancedIndexManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create all advanced indexes for high-frequency trading
    pub async fn create_advanced_indexes(&self) -> Result<()> {
        info!("Creating advanced indexes for high-frequency trading...");

        // Create specialized trading indexes
        self.create_trading_performance_indexes().await?;

        // Create real-time analytics indexes
        self.create_realtime_analytics_indexes().await?;

        // Create JSONB indexes for metadata queries
        self.create_jsonb_indexes().await?;

        // Create partial indexes for hot data
        self.create_hot_data_indexes().await?;

        // Create covering indexes for common queries
        self.create_covering_indexes().await?;

        // Create new optimized composite indexes
        self.create_optimized_composite_indexes().await?;

        // Create hash indexes for exact lookups
        self.create_hash_indexes().await?;

        // Create expression indexes for computed queries
        self.create_expression_indexes().await?;

        // Update table statistics for optimal query planning
        self.update_table_statistics().await?;

        info!("Advanced indexes created successfully");
        Ok(())
    }

    /// Create indexes optimized for trading performance queries
    async fn create_trading_performance_indexes(&self) -> Result<()> {
        let indexes = [
            // Multi-column index for latest market data by instrument and provider
            r#"
            CREATE INDEX IF NOT EXISTS idx_market_ticks_latest_by_provider
            ON market_ticks (instrument_id, provider, timestamp DESC)
            WHERE timestamp >= NOW() - INTERVAL '1 hour'
            "#,

            // Index for high-confidence trading signals
            r#"
            CREATE INDEX IF NOT EXISTS idx_trading_signals_high_confidence
            ON trading_signals (instrument_id, timestamp DESC, confidence_score)
            WHERE confidence_score >= 0.8 AND timestamp >= NOW() - INTERVAL '24 hours'
            "#,

            // Index for active AI predictions
            r#"
            CREATE INDEX IF NOT EXISTS idx_ai_predictions_active
            ON ai_predictions (instrument_id, model_type, timestamp DESC)
            WHERE timestamp >= NOW() - INTERVAL '6 hours'
            "#,

            // Index for recent trade executions with PnL
            r#"
            CREATE INDEX IF NOT EXISTS idx_trade_executions_pnl_recent
            ON trade_executions (instrument_id, timestamp DESC, pnl)
            WHERE pnl IS NOT NULL AND timestamp >= NOW() - INTERVAL '24 hours'
            "#,

            // Index for microstructure analysis by volatility regime
            r#"
            CREATE INDEX IF NOT EXISTS idx_microstructure_volatility_regime
            ON microstructure_analysis (volatility_regime, instrument_id, timestamp DESC)
            WHERE timestamp >= NOW() - INTERVAL '12 hours'
            "#,
        ];

        for index_sql in indexes {
            match sqlx::query(index_sql).execute(&self.pool).await {
                Ok(_) => {},
                Err(e) => {
                    // Log warning but continue - index might already exist
                    info!("Index creation warning: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Create indexes for real-time analytics
    async fn create_realtime_analytics_indexes(&self) -> Result<()> {
        let indexes = [
            // Composite index for OHLCV calculations
            r#"
            CREATE INDEX IF NOT EXISTS idx_market_ticks_ohlcv
            ON market_ticks (instrument_id, timestamp, last_price, volume)
            WHERE last_price IS NOT NULL AND volume IS NOT NULL
            "#,

            // Index for spread analysis
            r#"
            CREATE INDEX IF NOT EXISTS idx_market_ticks_spread_analysis
            ON market_ticks (instrument_id, timestamp, spread, data_quality_score)
            WHERE data_quality_score >= 0.7
            "#,

            // Index for liquidity analysis
            r#"
            CREATE INDEX IF NOT EXISTS idx_microstructure_liquidity
            ON microstructure_analysis (instrument_id, timestamp, liquidity_score, market_depth)
            WHERE liquidity_score >= 0.5
            "#,

            // Index for order book depth analysis
            r#"
            CREATE INDEX IF NOT EXISTS idx_order_book_depth
            ON order_book_snapshots (instrument_id, venue, timestamp, side, price, quantity)
            WHERE quantity > 0
            "#,

            // Index for risk metrics monitoring
            r#"
            CREATE INDEX IF NOT EXISTS idx_risk_metrics_monitoring
            ON risk_metrics (timestamp DESC, risk_score, portfolio_var)
            WHERE timestamp >= NOW() - INTERVAL '24 hours'
            "#,
        ];

        for index_sql in indexes {
            match sqlx::query(index_sql).execute(&self.pool).await {
                Ok(_) => {},
                Err(e) => {
                    info!("Index creation warning: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Create JSONB indexes for metadata queries
    async fn create_jsonb_indexes(&self) -> Result<()> {
        let indexes = [
            // GIN index for market tick metadata
            r#"
            CREATE INDEX IF NOT EXISTS idx_market_ticks_metadata_gin
            ON market_ticks USING GIN (raw_data)
            "#,

            // Specific JSONB path indexes for common queries
            r#"
            CREATE INDEX IF NOT EXISTS idx_market_ticks_source
            ON market_ticks ((raw_data->>'source'))
            WHERE raw_data->>'source' IS NOT NULL
            "#,

            // AI prediction intervals index
            r#"
            CREATE INDEX IF NOT EXISTS idx_ai_predictions_intervals_gin
            ON ai_predictions USING GIN (prediction_intervals)
            WHERE prediction_intervals IS NOT NULL
            "#,

            // Trading signal metadata index
            r#"
            CREATE INDEX IF NOT EXISTS idx_trading_signals_metadata_gin
            ON trading_signals USING GIN (metadata)
            "#,

            // Strategy-specific metadata index
            r#"
            CREATE INDEX IF NOT EXISTS idx_trading_signals_strategy_metadata
            ON trading_signals ((metadata->>'strategy'), strategy_type, timestamp DESC)
            WHERE metadata->>'strategy' IS NOT NULL
            "#,
        ];

        for index_sql in indexes {
            match sqlx::query(index_sql).execute(&self.pool).await {
                Ok(_) => {},
                Err(e) => {
                    info!("Index creation warning: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Create partial indexes for frequently accessed "hot" data
    async fn create_hot_data_indexes(&self) -> Result<()> {
        let indexes = [
            // Hot market data (last 15 minutes)
            r#"
            CREATE INDEX IF NOT EXISTS idx_market_ticks_hot
            ON market_ticks (instrument_id, timestamp DESC, last_price)
            WHERE timestamp >= NOW() - INTERVAL '15 minutes'
            "#,

            // Recent high-quality ticks
            r#"
            CREATE INDEX IF NOT EXISTS idx_market_ticks_high_quality_recent
            ON market_ticks (instrument_id, provider, timestamp DESC)
            WHERE data_quality_score >= 0.9 AND timestamp >= NOW() - INTERVAL '1 hour'
            "#,

            // Active trading signals (last 4 hours)
            r#"
            CREATE INDEX IF NOT EXISTS idx_trading_signals_active
            ON trading_signals (instrument_id, strategy_type, timestamp DESC)
            WHERE timestamp >= NOW() - INTERVAL '4 hours'
            "#,

            // Recent AI predictions with high confidence
            r#"
            CREATE INDEX IF NOT EXISTS idx_ai_predictions_recent_confident
            ON ai_predictions (instrument_id, model_type, timestamp DESC)
            WHERE confidence_score >= 0.8 AND timestamp >= NOW() - INTERVAL '2 hours'
            "#,

            // Current volatility regime analysis
            r#"
            CREATE INDEX IF NOT EXISTS idx_microstructure_current_regime
            ON microstructure_analysis (instrument_id, volatility_regime, timestamp DESC)
            WHERE timestamp >= NOW() - INTERVAL '30 minutes'
            "#,
        ];

        for index_sql in indexes {
            match sqlx::query(index_sql).execute(&self.pool).await {
                Ok(_) => {},
                Err(e) => {
                    info!("Index creation warning: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Create covering indexes to avoid table lookups
    async fn create_covering_indexes(&self) -> Result<()> {
        let indexes = [
            // Covering index for latest market data queries (simplified for TimescaleDB)
            r#"
            CREATE INDEX IF NOT EXISTS idx_market_ticks_latest_covering
            ON market_ticks (instrument_id, timestamp DESC, bid_price, ask_price, last_price, volume)
            WHERE timestamp >= NOW() - INTERVAL '1 hour'
            "#,

            // Covering index for trading signal analysis
            r#"
            CREATE INDEX IF NOT EXISTS idx_trading_signals_analysis_covering
            ON trading_signals (instrument_id, strategy_type, timestamp DESC, signal_type, confidence_score)
            WHERE timestamp >= NOW() - INTERVAL '24 hours'
            "#,

            // Covering index for AI model performance
            r#"
            CREATE INDEX IF NOT EXISTS idx_ai_predictions_performance_covering
            ON ai_predictions (model_type, model_version, timestamp DESC, instrument_id, predicted_price)
            WHERE timestamp >= NOW() - INTERVAL '24 hours'
            "#,

            // Covering index for order book analysis
            r#"
            CREATE INDEX IF NOT EXISTS idx_order_book_analysis_covering
            ON order_book_snapshots (instrument_id, venue, timestamp DESC, side, price, quantity)
            WHERE timestamp >= NOW() - INTERVAL '2 hours'
            "#,
        ];

        for index_sql in indexes {
            match sqlx::query(index_sql).execute(&self.pool).await {
                Ok(_) => {},
                Err(e) => {
                    info!("Index creation warning: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Create specialized indexes for specific trading strategies
    pub async fn create_strategy_specific_indexes(&self, strategy_name: &str) -> Result<()> {
        info!("Creating strategy-specific indexes for: {}", strategy_name);

        match strategy_name {
            "momentum" => self.create_momentum_strategy_indexes().await?,
            "arbitrage" => self.create_arbitrage_strategy_indexes().await?,
            "market_making" => self.create_market_making_indexes().await?,
            _ => {
                info!("No specific indexes defined for strategy: {}", strategy_name);
            }
        }

        Ok(())
    }

    /// Create indexes optimized for momentum trading strategies
    async fn create_momentum_strategy_indexes(&self) -> Result<()> {
        let indexes = [
            // Price momentum analysis
            r#"
            CREATE INDEX IF NOT EXISTS idx_market_ticks_momentum
            ON market_ticks (instrument_id, timestamp, last_price, volume)
            WHERE last_price IS NOT NULL AND volume > 0
            AND timestamp >= NOW() - INTERVAL '4 hours'
            "#,

            // Momentum signals
            r#"
            CREATE INDEX IF NOT EXISTS idx_trading_signals_momentum
            ON trading_signals (instrument_id, timestamp DESC, confidence_score)
            WHERE strategy_type = 'momentum' AND confidence_score >= 0.7
            "#,
        ];

        for index_sql in indexes {
            sqlx::query(index_sql).execute(&self.pool).await?;
        }

        Ok(())
    }

    /// Create indexes optimized for arbitrage strategies
    async fn create_arbitrage_strategy_indexes(&self) -> Result<()> {
        let indexes = [
            // Cross-venue price comparison
            r#"
            CREATE INDEX IF NOT EXISTS idx_market_ticks_arbitrage
            ON market_ticks (instrument_id, provider, timestamp DESC, bid_price, ask_price)
            WHERE timestamp >= NOW() - INTERVAL '5 minutes'
            "#,

            // Order book arbitrage opportunities
            r#"
            CREATE INDEX IF NOT EXISTS idx_order_book_arbitrage
            ON order_book_snapshots (instrument_id, venue, side, price, timestamp DESC)
            WHERE quantity > 0 AND timestamp >= NOW() - INTERVAL '5 minutes'
            "#,
        ];

        for index_sql in indexes {
            sqlx::query(index_sql).execute(&self.pool).await?;
        }

        Ok(())
    }

    /// Create indexes optimized for market making strategies
    async fn create_market_making_indexes(&self) -> Result<()> {
        let indexes = [
            // Spread analysis for market making
            r#"
            CREATE INDEX IF NOT EXISTS idx_market_ticks_market_making
            ON market_ticks (instrument_id, timestamp DESC, spread, bid_size, ask_size)
            WHERE spread > 0 AND timestamp >= NOW() - INTERVAL '30 minutes'
            "#,

            // Liquidity analysis
            r#"
            CREATE INDEX IF NOT EXISTS idx_microstructure_market_making
            ON microstructure_analysis (instrument_id, timestamp DESC, liquidity_score, market_maker_presence)
            WHERE timestamp >= NOW() - INTERVAL '1 hour'
            "#,
        ];

        for index_sql in indexes {
            sqlx::query(index_sql).execute(&self.pool).await?;
        }

        Ok(())
    }

    /// Drop unused or inefficient indexes
    pub async fn cleanup_unused_indexes(&self, min_usage_threshold: f64) -> Result<Vec<String>> {
        let unused_indexes = sqlx::query(
            r#"
            SELECT
                schemaname||'.'||indexrelname as index_name,
                idx_scan,
                pg_size_pretty(pg_relation_size(indexrelid)) as size
            FROM pg_stat_user_indexes
            WHERE schemaname = 'public'
            AND idx_scan < $1
            AND indexrelname NOT LIKE '%_pkey'  -- Don't drop primary keys
            AND indexrelname NOT LIKE '%_unique%'  -- Don't drop unique constraints
            ORDER BY idx_scan ASC, pg_relation_size(indexrelid) DESC
            "#
        )
        .bind(min_usage_threshold as i64)
        .fetch_all(&self.pool)
        .await?;

        let mut dropped_indexes = Vec::new();
        for row in unused_indexes {
            let index_name: String = row.get("index_name");
            let scans: Option<i64> = row.get("idx_scan");
            let size: String = row.get("size");

            if scans.unwrap_or(0) == 0 {
                // Only drop completely unused indexes for safety
                match sqlx::query(&format!("DROP INDEX IF EXISTS {}", index_name))
                    .execute(&self.pool)
                    .await
                {
                    Ok(_) => {
                        dropped_indexes.push(format!("{} (size: {})", index_name, size));
                        info!("Dropped unused index: {}", index_name);
                    }
                    Err(e) => {
                        info!("Failed to drop index {}: {}", index_name, e);
                    }
                }
            }
        }

        Ok(dropped_indexes)
    }

    /// Create optimized composite indexes for common query patterns
    async fn create_optimized_composite_indexes(&self) -> Result<()> {
        info!("Creating optimized composite indexes for HFT performance...");

        let indexes = [
            // Market ticks: instrument + time + quality (most critical for trading)
            r#"
            CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_market_ticks_instrument_time_quality_opt
            ON market_ticks (instrument_id, timestamp DESC, data_quality_score)
            WHERE data_quality_score >= 0.8 AND timestamp >= NOW() - INTERVAL '4 hours'
            "#,

            // AI predictions: instrument + model + confidence for high-quality predictions
            r#"
            CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_ai_predictions_instrument_model_confidence_opt
            ON ai_predictions (instrument_id, model_type, confidence_score DESC, timestamp DESC)
            WHERE confidence_score >= 0.7 AND timestamp >= NOW() - INTERVAL '6 hours'
            "#,

            // Trading signals: recent high-confidence signals (critical for execution)
            r#"
            CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_trading_signals_recent_high_confidence_opt
            ON trading_signals (instrument_id, timestamp DESC, confidence_score DESC, strategy_type)
            WHERE timestamp >= NOW() - INTERVAL '1 hour'
            AND confidence_score >= 0.8
            "#,

            // Market ticks: provider + time for data source analysis
            r#"
            CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_market_ticks_provider_time_instrument_opt
            ON market_ticks (provider, timestamp DESC, instrument_id, last_price)
            WHERE timestamp >= NOW() - INTERVAL '24 hours' AND last_price IS NOT NULL
            "#,

            // Trading signals: strategy performance analysis
            r#"
            CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_trading_signals_strategy_performance_opt
            ON trading_signals (strategy_type, timestamp DESC, confidence_score, risk_score)
            WHERE timestamp >= NOW() - INTERVAL '7 days'
            "#,
        ];

        for index_sql in indexes {
            match sqlx::query(index_sql).execute(&self.pool).await {
                Ok(_) => {},
                Err(e) => {
                    info!("Optimized composite index creation warning: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Create hash indexes for exact equality lookups
    async fn create_hash_indexes(&self) -> Result<()> {
        info!("Creating hash indexes for exact lookups...");

        let indexes = [
            // Instrument symbol lookups (exact matches)
            r#"
            CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_instruments_symbol_hash_opt
            ON instruments USING HASH (symbol)
            WHERE is_active = true
            "#,

            // Provider lookups for data source routing
            r#"
            CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_market_ticks_provider_hash_opt
            ON market_ticks USING HASH (provider)
            WHERE timestamp >= NOW() - INTERVAL '24 hours'
            "#,

            // Model type lookups for AI predictions
            r#"
            CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_ai_predictions_model_hash_opt
            ON ai_predictions USING HASH (model_type)
            WHERE timestamp >= NOW() - INTERVAL '24 hours'
            "#,

            // Strategy type lookups for trading signals
            r#"
            CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_trading_signals_strategy_hash_opt
            ON trading_signals USING HASH (strategy_type)
            WHERE timestamp >= NOW() - INTERVAL '24 hours'
            "#,
        ];

        for index_sql in indexes {
            match sqlx::query(index_sql).execute(&self.pool).await {
                Ok(_) => {},
                Err(e) => {
                    info!("Hash index creation warning: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Create expression indexes for computed queries
    async fn create_expression_indexes(&self) -> Result<()> {
        info!("Creating expression indexes for computed queries...");

        let indexes = [
            // Spread percentage for market efficiency analysis
            r#"
            CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_market_ticks_spread_percentage_opt
            ON market_ticks ((spread / NULLIF(last_price, 0) * 100), timestamp DESC, instrument_id)
            WHERE timestamp >= NOW() - INTERVAL '24 hours'
            AND last_price > 0
            AND spread > 0
            "#,

            // Volume-weighted average price calculation
            r#"
            CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_market_ticks_vwap_opt
            ON market_ticks (instrument_id, timestamp DESC, (last_price * volume))
            WHERE timestamp >= NOW() - INTERVAL '12 hours'
            AND last_price IS NOT NULL
            AND volume > 0
            "#,
        ];

        for index_sql in indexes {
            match sqlx::query(index_sql).execute(&self.pool).await {
                Ok(_) => {},
                Err(e) => {
                    info!("Expression index creation warning: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Update table statistics for optimal query planning
    async fn update_table_statistics(&self) -> Result<()> {
        info!("Updating table statistics for optimal query planning...");

        let tables = [
            "instruments",
            "market_ticks",
            "ai_predictions",
            "trading_signals",
            "microstructure_analysis",
            "order_book_snapshots",
            "trade_executions",
            "risk_metrics"
        ];

        for table in tables {
            match sqlx::query(&format!("ANALYZE {}", table))
                .execute(&self.pool)
                .await
            {
                Ok(_) => info!("✅ Updated statistics for table: {}", table),
                Err(e) => info!("❌ Failed to update statistics for {}: {}", table, e),
            }
        }

        Ok(())
    }

    /// Get comprehensive index usage statistics
    pub async fn get_index_usage_statistics(&self) -> Result<Vec<IndexUsageStats>> {
        let stats = sqlx::query(
            r#"
            SELECT
                schemaname,
                tablename,
                indexname,
                idx_scan,
                idx_tup_read,
                idx_tup_fetch,
                pg_size_pretty(pg_relation_size(indexrelid)) as index_size,
                pg_relation_size(indexrelid) as size_bytes,
                CASE
                    WHEN idx_scan = 0 THEN 'UNUSED'
                    WHEN idx_scan < 100 THEN 'LOW_USAGE'
                    WHEN idx_scan < 1000 THEN 'MEDIUM_USAGE'
                    ELSE 'HIGH_USAGE'
                END as usage_category,
                CASE
                    WHEN idx_scan > 0 THEN idx_tup_read::float / idx_scan::float
                    ELSE 0
                END as avg_tuples_per_scan
            FROM pg_stat_user_indexes
            WHERE schemaname = 'public'
            AND indexname LIKE 'idx_%'
            ORDER BY idx_scan DESC, pg_relation_size(indexrelid) DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut usage_stats = Vec::new();
        for row in stats {
            usage_stats.push(IndexUsageStats {
                schema_name: row.get("schemaname"),
                table_name: row.get("tablename"),
                index_name: row.get("indexname"),
                scans: row.get::<i64, _>("idx_scan"),
                tuples_read: row.get::<i64, _>("idx_tup_read"),
                tuples_fetched: row.get::<i64, _>("idx_tup_fetch"),
                size_pretty: row.get("index_size"),
                size_bytes: row.get::<i64, _>("size_bytes"),
                usage_category: row.get("usage_category"),
                avg_tuples_per_scan: row.get::<f64, _>("avg_tuples_per_scan"),
            });
        }

        Ok(usage_stats)
    }
}

/// Index usage statistics for monitoring and optimization
#[derive(Debug)]
pub struct IndexUsageStats {
    pub schema_name: String,
    pub table_name: String,
    pub index_name: String,
    pub scans: i64,
    pub tuples_read: i64,
    pub tuples_fetched: i64,
    pub size_pretty: String,
    pub size_bytes: i64,
    pub usage_category: String,
    pub avg_tuples_per_scan: f64,
}
