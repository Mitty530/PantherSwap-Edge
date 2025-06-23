-- Advanced Database Optimization for High-Frequency Trading
-- This migration adds specialized indexes, performance monitoring, and optimization features

-- ============================================================================
-- ENABLE REQUIRED EXTENSIONS
-- ============================================================================

-- Enable pg_stat_statements for query performance monitoring
CREATE EXTENSION IF NOT EXISTS pg_stat_statements;

-- Enable btree_gin for advanced indexing
CREATE EXTENSION IF NOT EXISTS btree_gin;

-- Enable pg_trgm for text search optimization
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- ============================================================================
-- ADVANCED TRADING PERFORMANCE INDEXES
-- ============================================================================

-- Multi-column index for latest market data by instrument and provider
CREATE INDEX IF NOT EXISTS idx_market_ticks_latest_by_provider
ON market_ticks (instrument_id, provider, timestamp DESC);

-- Index for high-confidence trading signals
CREATE INDEX IF NOT EXISTS idx_trading_signals_high_confidence
ON trading_signals (instrument_id, timestamp DESC, confidence_score)
WHERE confidence_score >= 0.8;

-- Index for active AI predictions
CREATE INDEX IF NOT EXISTS idx_ai_predictions_active
ON ai_predictions (instrument_id, model_type, timestamp DESC);

-- Index for recent trade executions with PnL
CREATE INDEX IF NOT EXISTS idx_trade_executions_pnl_recent
ON trade_executions (instrument_id, timestamp DESC, pnl)
WHERE pnl IS NOT NULL;

-- ============================================================================
-- REAL-TIME ANALYTICS INDEXES
-- ============================================================================

-- Composite index for OHLCV calculations
CREATE INDEX IF NOT EXISTS idx_market_ticks_ohlcv
ON market_ticks (instrument_id, timestamp, last_price, volume)
WHERE last_price IS NOT NULL AND volume IS NOT NULL;

-- Index for spread analysis
CREATE INDEX IF NOT EXISTS idx_market_ticks_spread_analysis
ON market_ticks (instrument_id, timestamp, spread, data_quality_score)
WHERE data_quality_score >= 0.7;

-- Index for liquidity analysis
CREATE INDEX IF NOT EXISTS idx_microstructure_liquidity
ON microstructure_analysis (instrument_id, timestamp, liquidity_score, market_depth)
WHERE liquidity_score >= 0.5;

-- Index for order book depth analysis
CREATE INDEX IF NOT EXISTS idx_order_book_depth
ON order_book_snapshots (instrument_id, venue, timestamp, side, price, quantity)
WHERE quantity > 0;

-- ============================================================================
-- JSONB INDEXES FOR METADATA QUERIES
-- ============================================================================

-- GIN index for market tick metadata
CREATE INDEX IF NOT EXISTS idx_market_ticks_metadata_gin
ON market_ticks USING GIN (raw_data);

-- Specific JSONB path indexes for common queries
CREATE INDEX IF NOT EXISTS idx_market_ticks_source
ON market_ticks ((raw_data->>'source'))
WHERE raw_data->>'source' IS NOT NULL;

-- AI prediction intervals index
CREATE INDEX IF NOT EXISTS idx_ai_predictions_intervals_gin
ON ai_predictions USING GIN (prediction_intervals)
WHERE prediction_intervals IS NOT NULL;

-- Trading signal metadata index
CREATE INDEX IF NOT EXISTS idx_trading_signals_metadata_gin
ON trading_signals USING GIN (metadata);

-- Strategy-specific metadata index
CREATE INDEX IF NOT EXISTS idx_trading_signals_strategy_metadata
ON trading_signals ((metadata->>'strategy'), strategy_type, timestamp DESC)
WHERE metadata->>'strategy' IS NOT NULL;

-- ============================================================================
-- HOT DATA PARTIAL INDEXES
-- ============================================================================

-- Hot market data index
CREATE INDEX IF NOT EXISTS idx_market_ticks_hot
ON market_ticks (instrument_id, timestamp DESC, last_price);

-- Recent high-quality ticks
CREATE INDEX IF NOT EXISTS idx_market_ticks_high_quality_recent
ON market_ticks (instrument_id, provider, timestamp DESC)
WHERE data_quality_score >= 0.9;

-- Active trading signals
CREATE INDEX IF NOT EXISTS idx_trading_signals_active
ON trading_signals (instrument_id, strategy_type, timestamp DESC);

-- Recent AI predictions with high confidence
CREATE INDEX IF NOT EXISTS idx_ai_predictions_recent_confident
ON ai_predictions (instrument_id, model_type, timestamp DESC)
WHERE confidence_score >= 0.8;

-- ============================================================================
-- COVERING INDEXES TO AVOID TABLE LOOKUPS
-- ============================================================================

-- Covering index for latest market data queries
CREATE INDEX IF NOT EXISTS idx_market_ticks_latest_covering
ON market_ticks (instrument_id, timestamp DESC)
INCLUDE (bid_price, ask_price, last_price, volume, spread, data_quality_score);

-- Covering index for trading signal analysis
CREATE INDEX IF NOT EXISTS idx_trading_signals_analysis_covering
ON trading_signals (instrument_id, strategy_type, timestamp DESC)
INCLUDE (signal_type, confidence_score, target_price, risk_score);

-- Covering index for AI model performance
CREATE INDEX IF NOT EXISTS idx_ai_predictions_performance_covering
ON ai_predictions (model_type, model_version, timestamp DESC)
INCLUDE (instrument_id, predicted_price, confidence_score, prediction_horizon_minutes);

-- ============================================================================
-- MATERIALIZED VIEWS FOR COMMON AGGREGATIONS
-- ============================================================================

-- Latest market data view for real-time queries
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_latest_market_data AS
SELECT DISTINCT ON (instrument_id, provider)
    instrument_id,
    provider,
    timestamp,
    bid_price,
    ask_price,
    last_price,
    volume,
    spread,
    data_quality_score
FROM market_ticks
ORDER BY instrument_id, provider, timestamp DESC;

-- Create index on materialized view
CREATE UNIQUE INDEX IF NOT EXISTS idx_mv_latest_market_data
ON mv_latest_market_data (instrument_id, provider);

-- Trading performance summary view
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_trading_performance_summary AS
SELECT
    instrument_id,
    strategy_type,
    COUNT(*) as signal_count,
    AVG(confidence_score) as avg_confidence,
    COUNT(*) FILTER (WHERE confidence_score >= 0.8) as high_confidence_signals,
    MAX(timestamp) as last_signal_time
FROM trading_signals
GROUP BY instrument_id, strategy_type;

-- Create index on trading performance view
CREATE INDEX IF NOT EXISTS idx_mv_trading_performance_summary
ON mv_trading_performance_summary (instrument_id, strategy_type);

-- ============================================================================
-- PERFORMANCE MONITORING FUNCTIONS
-- ============================================================================

-- Function to refresh materialized views
CREATE OR REPLACE FUNCTION refresh_performance_views()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW mv_latest_market_data;
    REFRESH MATERIALIZED VIEW mv_trading_performance_summary;
END;
$$ LANGUAGE plpgsql;

-- Function to get index usage statistics
CREATE OR REPLACE FUNCTION get_index_usage_stats()
RETURNS TABLE (
    table_name text,
    index_name text,
    index_scans bigint,
    tuples_read bigint,
    tuples_fetched bigint,
    index_size text,
    usage_score numeric
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        schemaname||'.'||tablename as table_name,
        indexname as index_name,
        idx_scan as index_scans,
        idx_tup_read as tuples_read,
        idx_tup_fetch as tuples_fetched,
        pg_size_pretty(pg_relation_size(indexrelid)) as index_size,
        CASE 
            WHEN idx_scan > 0 THEN 
                ROUND((LN(idx_scan::numeric) / 10.0 + LN(idx_tup_read::numeric) / 15.0)::numeric, 3)
            ELSE 0
        END as usage_score
    FROM pg_stat_user_indexes 
    WHERE schemaname = 'public'
    ORDER BY idx_scan DESC, idx_tup_read DESC;
END;
$$ LANGUAGE plpgsql;

-- Function to analyze table bloat
CREATE OR REPLACE FUNCTION analyze_table_bloat()
RETURNS TABLE (
    table_name text,
    live_tuples bigint,
    dead_tuples bigint,
    bloat_ratio numeric,
    recommendation text
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        schemaname||'.'||tablename as table_name,
        n_live_tup as live_tuples,
        n_dead_tup as dead_tuples,
        CASE 
            WHEN n_live_tup > 0 THEN 
                ROUND((n_dead_tup::numeric / n_live_tup::numeric * 100)::numeric, 2)
            ELSE 0
        END as bloat_ratio,
        CASE 
            WHEN n_live_tup > 0 AND (n_dead_tup::numeric / n_live_tup::numeric) > 0.1 THEN
                'VACUUM ANALYZE ' || schemaname||'.'||tablename
            ELSE 'No action needed'
        END as recommendation
    FROM pg_stat_user_tables 
    WHERE schemaname = 'public'
    ORDER BY n_dead_tup DESC;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- AUTOMATIC MAINTENANCE JOBS
-- ============================================================================

-- Schedule materialized view refresh (requires pg_cron extension)
-- SELECT cron.schedule('refresh-views', '*/5 * * * *', 'SELECT refresh_performance_views();');

-- ============================================================================
-- PERFORMANCE OPTIMIZATION SETTINGS
-- ============================================================================

-- Optimize PostgreSQL settings for high-frequency trading
-- These are recommendations - actual values should be tuned based on hardware

-- Memory settings
-- shared_buffers = 25% of RAM
-- effective_cache_size = 75% of RAM
-- work_mem = 256MB (for complex queries)
-- maintenance_work_mem = 2GB

-- Checkpoint settings for write-heavy workloads
-- checkpoint_completion_target = 0.9
-- wal_buffers = 16MB
-- checkpoint_timeout = 15min

-- Connection settings
-- max_connections = 200 (adjust based on pool configuration)
-- max_prepared_transactions = 100

-- Query planner settings
-- random_page_cost = 1.1 (for SSD storage)
-- effective_io_concurrency = 200 (for SSD)

-- Logging for performance monitoring
-- log_min_duration_statement = 1000 (log queries > 1 second)
-- log_checkpoints = on
-- log_connections = on
-- log_disconnections = on
-- log_lock_waits = on

-- TimescaleDB specific settings
-- timescaledb.max_background_workers = 8
-- max_worker_processes = 16
