-- PantherSwap Edge Database Data Verification Queries
-- Verify actual data stored during the 5-minute trading simulation

-- ============================================================================
-- 1. MARKET DATA VERIFICATION
-- ============================================================================

-- Check market ticks stored during simulation
SELECT 
    COUNT(*) as total_ticks,
    MIN(timestamp) as first_tick,
    MAX(timestamp) as last_tick,
    AVG(bid_price) as avg_bid,
    AVG(ask_price) as avg_ask,
    AVG(spread) as avg_spread,
    AVG(data_quality_score) as avg_quality
FROM market_ticks 
WHERE timestamp >= NOW() - INTERVAL '1 hour'
ORDER BY timestamp DESC;

-- Market data by provider
SELECT 
    provider,
    COUNT(*) as tick_count,
    AVG(bid_price) as avg_bid,
    AVG(ask_price) as avg_ask,
    MIN(timestamp) as first_tick,
    MAX(timestamp) as last_tick
FROM market_ticks 
WHERE timestamp >= NOW() - INTERVAL '1 hour'
GROUP BY provider;

-- Recent market ticks sample
SELECT 
    timestamp,
    provider,
    bid_price,
    ask_price,
    spread,
    data_quality_score
FROM market_ticks 
WHERE timestamp >= NOW() - INTERVAL '1 hour'
ORDER BY timestamp DESC 
LIMIT 10;

-- ============================================================================
-- 2. AI PREDICTIONS VERIFICATION
-- ============================================================================

-- Check AI predictions stored
SELECT 
    COUNT(*) as total_predictions,
    model_type,
    AVG(confidence_score) as avg_confidence,
    AVG(predicted_price) as avg_predicted_price,
    MIN(timestamp) as first_prediction,
    MAX(timestamp) as last_prediction
FROM ai_predictions 
WHERE timestamp >= NOW() - INTERVAL '1 hour'
GROUP BY model_type;

-- AI prediction accuracy analysis
SELECT 
    model_type,
    model_version,
    COUNT(*) as prediction_count,
    AVG(confidence_score) as avg_confidence,
    MIN(confidence_score) as min_confidence,
    MAX(confidence_score) as max_confidence,
    AVG(predicted_price) as avg_predicted_price
FROM ai_predictions 
WHERE timestamp >= NOW() - INTERVAL '1 hour'
GROUP BY model_type, model_version
ORDER BY prediction_count DESC;

-- Recent AI predictions sample
SELECT 
    timestamp,
    model_type,
    predicted_price,
    confidence_score,
    prediction_horizon_minutes
FROM ai_predictions 
WHERE timestamp >= NOW() - INTERVAL '1 hour'
ORDER BY timestamp DESC 
LIMIT 10;

-- ============================================================================
-- 3. TRADING SIGNALS VERIFICATION
-- ============================================================================

-- Check trading signals generated
SELECT 
    COUNT(*) as total_signals,
    signal_type,
    AVG(confidence_score) as avg_confidence,
    AVG(signal_strength) as avg_strength,
    MIN(timestamp) as first_signal,
    MAX(timestamp) as last_signal
FROM trading_signals 
WHERE timestamp >= NOW() - INTERVAL '1 hour'
GROUP BY signal_type;

-- Trading signals by strategy
SELECT 
    strategy_name,
    signal_type,
    COUNT(*) as signal_count,
    AVG(confidence_score) as avg_confidence,
    AVG(recommended_size) as avg_position_size
FROM trading_signals 
WHERE timestamp >= NOW() - INTERVAL '1 hour'
GROUP BY strategy_name, signal_type
ORDER BY signal_count DESC;

-- Recent trading signals sample
SELECT 
    timestamp,
    strategy_name,
    signal_type,
    confidence_score,
    recommended_size,
    entry_price
FROM trading_signals 
WHERE timestamp >= NOW() - INTERVAL '1 hour'
ORDER BY timestamp DESC 
LIMIT 10;

-- ============================================================================
-- 4. TRADE EXECUTIONS VERIFICATION
-- ============================================================================

-- Check trade executions
SELECT 
    COUNT(*) as total_trades,
    side,
    AVG(quantity) as avg_quantity,
    AVG(price) as avg_price,
    SUM(quantity * price) as total_volume,
    MIN(timestamp) as first_trade,
    MAX(timestamp) as last_trade
FROM trade_executions 
WHERE timestamp >= NOW() - INTERVAL '1 hour'
GROUP BY side;

-- Trade execution performance
SELECT 
    COUNT(*) as total_executions,
    AVG(execution_time_ms) as avg_execution_time,
    MIN(execution_time_ms) as min_execution_time,
    MAX(execution_time_ms) as max_execution_time,
    AVG(slippage_bps) as avg_slippage,
    SUM(pnl) as total_pnl
FROM trade_executions 
WHERE timestamp >= NOW() - INTERVAL '1 hour';

-- Recent trade executions sample
SELECT 
    timestamp,
    order_id,
    side,
    quantity,
    price,
    execution_time_ms,
    slippage_bps,
    pnl
FROM trade_executions 
WHERE timestamp >= NOW() - INTERVAL '1 hour'
ORDER BY timestamp DESC 
LIMIT 15;

-- ============================================================================
-- 5. RISK METRICS VERIFICATION
-- ============================================================================

-- Check risk metrics stored
SELECT 
    COUNT(*) as total_risk_records,
    AVG(portfolio_value) as avg_portfolio_value,
    AVG(var_95) as avg_var_95,
    AVG(max_drawdown) as avg_max_drawdown,
    MIN(timestamp) as first_record,
    MAX(timestamp) as last_record
FROM risk_metrics 
WHERE timestamp >= NOW() - INTERVAL '1 hour';

-- Risk metrics trends
SELECT 
    DATE_TRUNC('minute', timestamp) as minute_bucket,
    AVG(portfolio_value) as avg_portfolio_value,
    AVG(var_95) as avg_var,
    AVG(sharpe_ratio) as avg_sharpe
FROM risk_metrics 
WHERE timestamp >= NOW() - INTERVAL '1 hour'
GROUP BY minute_bucket
ORDER BY minute_bucket DESC
LIMIT 10;

-- ============================================================================
-- 6. SYSTEM PERFORMANCE VERIFICATION
-- ============================================================================

-- Database performance metrics
SELECT 
    schemaname,
    tablename,
    n_tup_ins as inserts,
    n_tup_upd as updates,
    n_tup_del as deletes,
    n_live_tup as live_rows,
    n_dead_tup as dead_rows
FROM pg_stat_user_tables 
WHERE schemaname = 'public'
AND tablename IN ('market_ticks', 'ai_predictions', 'trading_signals', 'trade_executions', 'risk_metrics')
ORDER BY n_tup_ins DESC;

-- Hypertable chunk information
SELECT 
    hypertable_name,
    chunk_name,
    range_start,
    range_end,
    is_compressed,
    chunk_size
FROM timescaledb_information.chunks 
WHERE hypertable_name IN ('market_ticks', 'ai_predictions', 'trading_signals', 'trade_executions')
ORDER BY range_start DESC
LIMIT 10;

-- Index usage statistics
SELECT 
    schemaname,
    tablename,
    indexname,
    idx_scan as index_scans,
    idx_tup_read as tuples_read,
    idx_tup_fetch as tuples_fetched
FROM pg_stat_user_indexes 
WHERE schemaname = 'public'
AND tablename IN ('market_ticks', 'ai_predictions', 'trading_signals', 'trade_executions')
ORDER BY idx_scan DESC
LIMIT 10;

-- ============================================================================
-- 7. DATA QUALITY VERIFICATION
-- ============================================================================

-- Check for data consistency
SELECT 
    'market_ticks' as table_name,
    COUNT(*) as total_records,
    COUNT(DISTINCT instrument_id) as unique_instruments,
    MIN(timestamp) as earliest_record,
    MAX(timestamp) as latest_record
FROM market_ticks 
WHERE timestamp >= NOW() - INTERVAL '1 hour'

UNION ALL

SELECT 
    'ai_predictions' as table_name,
    COUNT(*) as total_records,
    COUNT(DISTINCT instrument_id) as unique_instruments,
    MIN(timestamp) as earliest_record,
    MAX(timestamp) as latest_record
FROM ai_predictions 
WHERE timestamp >= NOW() - INTERVAL '1 hour'

UNION ALL

SELECT 
    'trading_signals' as table_name,
    COUNT(*) as total_records,
    COUNT(DISTINCT instrument_id) as unique_instruments,
    MIN(timestamp) as earliest_record,
    MAX(timestamp) as latest_record
FROM trading_signals 
WHERE timestamp >= NOW() - INTERVAL '1 hour'

UNION ALL

SELECT 
    'trade_executions' as table_name,
    COUNT(*) as total_records,
    COUNT(DISTINCT instrument_id) as unique_instruments,
    MIN(timestamp) as earliest_record,
    MAX(timestamp) as latest_record
FROM trade_executions 
WHERE timestamp >= NOW() - INTERVAL '1 hour';

-- Check for null values in critical fields
SELECT 
    'market_ticks' as table_name,
    COUNT(*) FILTER (WHERE bid_price IS NULL) as null_bid_price,
    COUNT(*) FILTER (WHERE ask_price IS NULL) as null_ask_price,
    COUNT(*) FILTER (WHERE timestamp IS NULL) as null_timestamp
FROM market_ticks 
WHERE timestamp >= NOW() - INTERVAL '1 hour'

UNION ALL

SELECT 
    'ai_predictions' as table_name,
    COUNT(*) FILTER (WHERE predicted_price IS NULL) as null_predicted_price,
    COUNT(*) FILTER (WHERE confidence_score IS NULL) as null_confidence,
    COUNT(*) FILTER (WHERE timestamp IS NULL) as null_timestamp
FROM ai_predictions 
WHERE timestamp >= NOW() - INTERVAL '1 hour';

-- ============================================================================
-- 8. STORAGE AND COMPRESSION ANALYSIS
-- ============================================================================

-- Table sizes and compression ratios
SELECT 
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as total_size,
    pg_size_pretty(pg_relation_size(schemaname||'.'||tablename)) as table_size,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename) - pg_relation_size(schemaname||'.'||tablename)) as index_size
FROM pg_tables 
WHERE schemaname = 'public'
AND tablename IN ('market_ticks', 'ai_predictions', 'trading_signals', 'trade_executions', 'risk_metrics')
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;

-- Compression statistics for hypertables
SELECT 
    hypertable_name,
    compression_status,
    uncompressed_heap_size,
    compressed_heap_size,
    uncompressed_index_size,
    compressed_index_size,
    compression_ratio
FROM timescaledb_information.compression_settings 
WHERE hypertable_name IN ('market_ticks', 'ai_predictions', 'trading_signals', 'trade_executions');
