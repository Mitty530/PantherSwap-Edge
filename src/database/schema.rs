// Database schema definitions for TimescaleDB hypertables
// Optimized for high-frequency trading data

// ============================================================================
// REFERENCE TABLES (Regular PostgreSQL tables)
// ============================================================================

pub const CREATE_INSTRUMENTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS instruments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    symbol VARCHAR(20) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    instrument_type VARCHAR(20) NOT NULL,
    base_currency VARCHAR(10) NOT NULL,
    quote_currency VARCHAR(10) NOT NULL,
    tick_size DECIMAL(20, 10) NOT NULL,
    lot_size DECIMAL(20, 10) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
"#;

// ============================================================================
// TIME-SERIES TABLES (TimescaleDB Hypertables)
// ============================================================================

pub const CREATE_MARKET_TICKS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS market_ticks (
    timestamp TIMESTAMPTZ NOT NULL,
    instrument_id UUID NOT NULL REFERENCES instruments(id),
    provider VARCHAR(50) NOT NULL,
    bid_price DECIMAL(20, 10) NOT NULL,
    ask_price DECIMAL(20, 10) NOT NULL,
    bid_size DECIMAL(20, 10) NOT NULL,
    ask_size DECIMAL(20, 10) NOT NULL,
    last_price DECIMAL(20, 10),
    volume DECIMAL(20, 10),
    spread DECIMAL(20, 10) NOT NULL,
    data_quality_score DECIMAL(3, 2) NOT NULL,
    raw_data JSONB NOT NULL
);
"#;

pub const CREATE_AI_PREDICTIONS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS ai_predictions (
    timestamp TIMESTAMPTZ NOT NULL,
    instrument_id UUID NOT NULL REFERENCES instruments(id),
    model_type VARCHAR(50) NOT NULL,
    model_version VARCHAR(20) NOT NULL,
    prediction_horizon_minutes INTEGER NOT NULL,
    predicted_price DECIMAL(20, 10) NOT NULL,
    predicted_volatility DECIMAL(8, 6),
    confidence_score DECIMAL(5, 4) NOT NULL,
    prediction_intervals JSONB,
    feature_importance JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
"#;

pub const CREATE_MICROSTRUCTURE_ANALYSIS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS microstructure_analysis (
    timestamp TIMESTAMPTZ NOT NULL,
    instrument_id UUID NOT NULL REFERENCES instruments(id),
    order_book_imbalance DECIMAL(8, 6) NOT NULL,
    bid_ask_spread DECIMAL(20, 10) NOT NULL,
    market_depth DECIMAL(20, 10) NOT NULL,
    price_impact DECIMAL(8, 6) NOT NULL,
    liquidity_score DECIMAL(5, 4) NOT NULL,
    volatility_regime VARCHAR(30) NOT NULL,
    market_maker_presence DECIMAL(5, 4) NOT NULL,
    analysis_data JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
"#;

pub const CREATE_TRADING_SIGNALS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS trading_signals (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL,
    instrument_id UUID NOT NULL REFERENCES instruments(id),
    strategy_name VARCHAR(100) NOT NULL,
    signal_type VARCHAR(20) NOT NULL,
    signal_strength DECIMAL(8, 6) NOT NULL,
    confidence_score DECIMAL(8, 6) NOT NULL,
    recommended_size DECIMAL(20, 10) NOT NULL,
    entry_price DECIMAL(20, 10),
    stop_loss DECIMAL(20, 10),
    take_profit DECIMAL(20, 10),
    time_horizon INTERVAL,
    expected_return DECIMAL(8, 6),
    risk_metrics JSONB
);
"#;

pub const CREATE_ORDER_BOOK_SNAPSHOTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS order_book_snapshots (
    timestamp TIMESTAMPTZ NOT NULL,
    instrument_id UUID NOT NULL REFERENCES instruments(id),
    venue VARCHAR(50) NOT NULL,
    side VARCHAR(10) NOT NULL,
    price DECIMAL(20, 10) NOT NULL,
    quantity DECIMAL(20, 10) NOT NULL,
    order_count INTEGER,
    market_maker_id VARCHAR(50),
    order_type VARCHAR(20),
    created_at TIMESTAMPTZ DEFAULT NOW()
);
"#;

pub const CREATE_RISK_METRICS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS risk_metrics (
    timestamp TIMESTAMPTZ NOT NULL,
    instrument_id UUID REFERENCES instruments(id),
    portfolio_var DECIMAL(8, 6) NOT NULL,
    position_size DECIMAL(20, 10) NOT NULL,
    leverage DECIMAL(8, 4) NOT NULL,
    drawdown DECIMAL(8, 6) NOT NULL,
    sharpe_ratio DECIMAL(8, 6),
    max_loss_24h DECIMAL(20, 10) NOT NULL,
    risk_score DECIMAL(5, 4) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
"#;

pub const CREATE_TRADE_EXECUTIONS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS trade_executions (
    timestamp TIMESTAMPTZ NOT NULL,
    instrument_id UUID NOT NULL REFERENCES instruments(id),
    signal_id UUID,
    action VARCHAR(20) NOT NULL,
    quantity DECIMAL(20, 10) NOT NULL,
    price DECIMAL(20, 10) NOT NULL,
    execution_time_ms INTEGER,
    slippage_bps DECIMAL(8, 4),
    fees DECIMAL(20, 10),
    pnl DECIMAL(20, 10),
    confidence_score DECIMAL(5, 4),
    strategy_name VARCHAR(100),
    created_at TIMESTAMPTZ DEFAULT NOW()
);
"#;

// ============================================================================
// ALPACA TRADING TABLES (Regular PostgreSQL tables for order tracking)
// ============================================================================

pub const CREATE_ALPACA_ORDERS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS alpaca_orders (
    id SERIAL PRIMARY KEY,
    alpaca_order_id VARCHAR(255) UNIQUE NOT NULL,
    internal_order_id UUID NOT NULL,
    symbol VARCHAR(20) NOT NULL,
    side VARCHAR(10) NOT NULL,
    quantity DECIMAL(20, 8) NOT NULL,
    order_type VARCHAR(20) NOT NULL,
    status VARCHAR(20) NOT NULL,
    submitted_at TIMESTAMPTZ NOT NULL,
    filled_at TIMESTAMPTZ,
    filled_qty DECIMAL(20, 8) DEFAULT 0,
    filled_avg_price DECIMAL(20, 8),
    time_in_force VARCHAR(20),
    limit_price DECIMAL(20, 8),
    stop_price DECIMAL(20, 8),
    execution_time_ms INTEGER,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
"#;

pub const CREATE_ALPACA_ACCOUNT_SNAPSHOTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS alpaca_account_snapshots (
    id SERIAL PRIMARY KEY,
    account_id VARCHAR(255) NOT NULL,
    equity DECIMAL(20, 8),
    cash DECIMAL(20, 8),
    buying_power DECIMAL(20, 8),
    portfolio_value DECIMAL(20, 8),
    day_trade_buying_power DECIMAL(20, 8),
    regt_buying_power DECIMAL(20, 8),
    account_data JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
"#;

pub const CREATE_ALPACA_POSITIONS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS alpaca_positions (
    id SERIAL PRIMARY KEY,
    symbol VARCHAR(20) NOT NULL,
    qty DECIMAL(20, 8) NOT NULL,
    side VARCHAR(10),
    market_value DECIMAL(20, 8),
    cost_basis DECIMAL(20, 8),
    unrealized_pl DECIMAL(20, 8),
    unrealized_plpc DECIMAL(8, 6),
    current_price DECIMAL(20, 8),
    position_data JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
"#;

pub const CREATE_ALPACA_PERFORMANCE_METRICS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS alpaca_performance_metrics (
    id SERIAL PRIMARY KEY,
    total_trades BIGINT NOT NULL,
    profitable_trades BIGINT NOT NULL,
    total_pnl DECIMAL(20, 8) NOT NULL,
    total_volume DECIMAL(20, 8) NOT NULL,
    max_drawdown DECIMAL(8, 6),
    sharpe_ratio DECIMAL(8, 6),
    win_rate DECIMAL(8, 6),
    average_trade_duration_minutes DECIMAL(10, 2),
    daily_pnl JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
"#;

pub const CREATE_ALPACA_EXECUTION_STATS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS alpaca_execution_stats (
    id SERIAL PRIMARY KEY,
    total_orders BIGINT NOT NULL,
    filled_orders BIGINT NOT NULL,
    cancelled_orders BIGINT NOT NULL,
    rejected_orders BIGINT NOT NULL,
    total_volume DECIMAL(20, 8) NOT NULL,
    average_fill_time_ms DECIMAL(10, 2),
    slippage_bps DECIMAL(8, 4),
    created_at TIMESTAMPTZ DEFAULT NOW()
);
"#;

pub const CREATE_ALPACA_MARKET_EVENTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS alpaca_market_events (
    id SERIAL PRIMARY KEY,
    symbol VARCHAR(20) NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    event_data JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
"#;

// ============================================================================
// HYPERTABLE CREATION QUERIES
// ============================================================================

pub const CREATE_MARKET_TICKS_HYPERTABLE: &str = r#"
SELECT create_hypertable('market_ticks', 'timestamp',
    chunk_time_interval => INTERVAL '1 hour',
    if_not_exists => TRUE);
"#;

pub const CREATE_AI_PREDICTIONS_HYPERTABLE: &str = r#"
SELECT create_hypertable('ai_predictions', 'timestamp',
    chunk_time_interval => INTERVAL '1 hour',
    if_not_exists => TRUE);
"#;

pub const CREATE_MICROSTRUCTURE_ANALYSIS_HYPERTABLE: &str = r#"
SELECT create_hypertable('microstructure_analysis', 'timestamp',
    chunk_time_interval => INTERVAL '1 hour',
    if_not_exists => TRUE);
"#;

pub const CREATE_TRADING_SIGNALS_HYPERTABLE: &str = r#"
SELECT create_hypertable('trading_signals', 'timestamp',
    chunk_time_interval => INTERVAL '1 hour',
    if_not_exists => TRUE);
"#;

pub const CREATE_ORDER_BOOK_SNAPSHOTS_HYPERTABLE: &str = r#"
SELECT create_hypertable('order_book_snapshots', 'timestamp',
    chunk_time_interval => INTERVAL '30 minutes',
    if_not_exists => TRUE);
"#;

pub const CREATE_RISK_METRICS_HYPERTABLE: &str = r#"
SELECT create_hypertable('risk_metrics', 'timestamp',
    chunk_time_interval => INTERVAL '1 hour',
    if_not_exists => TRUE);
"#;

pub const CREATE_TRADE_EXECUTIONS_HYPERTABLE: &str = r#"
SELECT create_hypertable('trade_executions', 'timestamp',
    chunk_time_interval => INTERVAL '1 hour',
    if_not_exists => TRUE);
"#;

// ============================================================================
// COMPRESSION POLICIES FOR HISTORICAL DATA
// ============================================================================

pub const ENABLE_COMPRESSION_MARKET_TICKS: &str = r#"
ALTER TABLE market_ticks SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'instrument_id, provider',
    timescaledb.compress_orderby = 'timestamp DESC'
);
"#;

pub const ENABLE_COMPRESSION_AI_PREDICTIONS: &str = r#"
ALTER TABLE ai_predictions SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'instrument_id, model_type',
    timescaledb.compress_orderby = 'timestamp DESC'
);
"#;

pub const ENABLE_COMPRESSION_MICROSTRUCTURE: &str = r#"
ALTER TABLE microstructure_analysis SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'instrument_id',
    timescaledb.compress_orderby = 'timestamp DESC'
);
"#;

pub const ENABLE_COMPRESSION_ORDER_BOOK: &str = r#"
ALTER TABLE order_book_snapshots SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'instrument_id, venue, side',
    timescaledb.compress_orderby = 'timestamp DESC'
);
"#;

// ============================================================================
// COMPRESSION POLICIES (Compress data older than 24 hours)
// ============================================================================

pub const ADD_COMPRESSION_POLICY_MARKET_TICKS: &str = r#"
SELECT add_compression_policy('market_ticks', INTERVAL '24 hours');
"#;

pub const ADD_COMPRESSION_POLICY_AI_PREDICTIONS: &str = r#"
SELECT add_compression_policy('ai_predictions', INTERVAL '24 hours');
"#;

pub const ADD_COMPRESSION_POLICY_MICROSTRUCTURE: &str = r#"
SELECT add_compression_policy('microstructure_analysis', INTERVAL '24 hours');
"#;

pub const ADD_COMPRESSION_POLICY_ORDER_BOOK: &str = r#"
SELECT add_compression_policy('order_book_snapshots', INTERVAL '12 hours');
"#;

// ============================================================================
// RETENTION POLICIES (Delete data older than specified period)
// ============================================================================

pub const ADD_RETENTION_POLICY_MARKET_TICKS: &str = r#"
SELECT add_retention_policy('market_ticks', INTERVAL '90 days');
"#;

pub const ADD_RETENTION_POLICY_ORDER_BOOK: &str = r#"
SELECT add_retention_policy('order_book_snapshots', INTERVAL '30 days');
"#;

// ============================================================================
// PERFORMANCE INDEXES
// ============================================================================

// Individual index creation queries (one per constant to avoid multi-statement issues)
pub const CREATE_INDEX_INSTRUMENTS_SYMBOL: &str = r#"
CREATE INDEX IF NOT EXISTS idx_instruments_symbol ON instruments(symbol);
"#;

pub const CREATE_INDEX_INSTRUMENTS_TYPE: &str = r#"
CREATE INDEX IF NOT EXISTS idx_instruments_type ON instruments(instrument_type);
"#;

pub const CREATE_INDEX_INSTRUMENTS_ACTIVE: &str = r#"
CREATE INDEX IF NOT EXISTS idx_instruments_active ON instruments(is_active) WHERE is_active = true;
"#;

pub const CREATE_INDEX_MARKET_TICKS_INSTRUMENT_TIME: &str = r#"
CREATE INDEX IF NOT EXISTS idx_market_ticks_instrument_time
    ON market_ticks (instrument_id, timestamp DESC);
"#;

pub const CREATE_INDEX_MARKET_TICKS_PROVIDER: &str = r#"
CREATE INDEX IF NOT EXISTS idx_market_ticks_provider
    ON market_ticks (provider);
"#;

pub const CREATE_INDEX_MARKET_TICKS_QUALITY: &str = r#"
CREATE INDEX IF NOT EXISTS idx_market_ticks_quality
    ON market_ticks (data_quality_score) WHERE data_quality_score >= 0.8;
"#;

pub const CREATE_INDEX_AI_PREDICTIONS_INSTRUMENT_TIME: &str = r#"
CREATE INDEX IF NOT EXISTS idx_ai_predictions_instrument_time
    ON ai_predictions (instrument_id, timestamp DESC);
"#;

pub const CREATE_INDEX_AI_PREDICTIONS_MODEL: &str = r#"
CREATE INDEX IF NOT EXISTS idx_ai_predictions_model
    ON ai_predictions (model_type, model_version);
"#;

pub const CREATE_INDEX_AI_PREDICTIONS_CONFIDENCE: &str = r#"
CREATE INDEX IF NOT EXISTS idx_ai_predictions_confidence
    ON ai_predictions (confidence_score) WHERE confidence_score >= 0.7;
"#;

pub const CREATE_INDEX_MICROSTRUCTURE_INSTRUMENT_TIME: &str = r#"
CREATE INDEX IF NOT EXISTS idx_microstructure_instrument_time
    ON microstructure_analysis (instrument_id, timestamp DESC);
"#;

pub const CREATE_INDEX_MICROSTRUCTURE_REGIME: &str = r#"
CREATE INDEX IF NOT EXISTS idx_microstructure_regime
    ON microstructure_analysis (volatility_regime);
"#;

pub const CREATE_INDEX_MICROSTRUCTURE_LIQUIDITY: &str = r#"
CREATE INDEX IF NOT EXISTS idx_microstructure_liquidity
    ON microstructure_analysis (liquidity_score) WHERE liquidity_score >= 0.5;
"#;

pub const CREATE_INDEX_TRADING_SIGNALS_INSTRUMENT_TIME: &str = r#"
CREATE INDEX IF NOT EXISTS idx_trading_signals_instrument_time
    ON trading_signals (instrument_id, timestamp DESC);
"#;

pub const CREATE_INDEX_TRADING_SIGNALS_STRATEGY: &str = r#"
CREATE INDEX IF NOT EXISTS idx_trading_signals_strategy
    ON trading_signals (strategy_name);
"#;

pub const CREATE_INDEX_TRADING_SIGNALS_CONFIDENCE: &str = r#"
CREATE INDEX IF NOT EXISTS idx_trading_signals_confidence
    ON trading_signals (confidence_score) WHERE confidence_score >= 0.6;
"#;

pub const CREATE_INDEX_ORDER_BOOK_INSTRUMENT_TIME: &str = r#"
CREATE INDEX IF NOT EXISTS idx_order_book_instrument_time
    ON order_book_snapshots (instrument_id, timestamp DESC);
"#;

pub const CREATE_INDEX_ORDER_BOOK_VENUE_SIDE: &str = r#"
CREATE INDEX IF NOT EXISTS idx_order_book_venue_side
    ON order_book_snapshots (venue, side);
"#;

pub const CREATE_INDEX_ORDER_BOOK_PRICE: &str = r#"
CREATE INDEX IF NOT EXISTS idx_order_book_price
    ON order_book_snapshots (price) WHERE quantity > 0;
"#;

pub const CREATE_INDEX_TRADE_EXECUTIONS_INSTRUMENT_TIME: &str = r#"
CREATE INDEX IF NOT EXISTS idx_trade_executions_instrument_time
    ON trade_executions (instrument_id, timestamp DESC);
"#;

pub const CREATE_INDEX_TRADE_EXECUTIONS_STRATEGY: &str = r#"
CREATE INDEX IF NOT EXISTS idx_trade_executions_strategy
    ON trade_executions (strategy_name);
"#;

pub const CREATE_INDEX_TRADE_EXECUTIONS_PNL: &str = r#"
CREATE INDEX IF NOT EXISTS idx_trade_executions_pnl
    ON trade_executions (pnl) WHERE pnl IS NOT NULL;
"#;

// ============================================================================
// ALPACA TRADING INDEXES (Optimized for high-frequency operations)
// ============================================================================

pub const CREATE_INDEX_ALPACA_ORDERS_SYMBOL_CREATED: &str = r#"
CREATE INDEX IF NOT EXISTS idx_alpaca_orders_symbol_created
    ON alpaca_orders(symbol, created_at DESC);
"#;

pub const CREATE_INDEX_ALPACA_ORDERS_STATUS: &str = r#"
CREATE INDEX IF NOT EXISTS idx_alpaca_orders_status
    ON alpaca_orders(status);
"#;

pub const CREATE_INDEX_ALPACA_ORDERS_ALPACA_ID: &str = r#"
CREATE INDEX IF NOT EXISTS idx_alpaca_orders_alpaca_id
    ON alpaca_orders(alpaca_order_id);
"#;

pub const CREATE_INDEX_ALPACA_ORDERS_INTERNAL_ID: &str = r#"
CREATE INDEX IF NOT EXISTS idx_alpaca_orders_internal_id
    ON alpaca_orders(internal_order_id);
"#;

pub const CREATE_INDEX_ALPACA_ORDERS_EXECUTION_TIME: &str = r#"
CREATE INDEX IF NOT EXISTS idx_alpaca_orders_execution_time
    ON alpaca_orders(execution_time_ms) WHERE execution_time_ms IS NOT NULL;
"#;

pub const CREATE_INDEX_ALPACA_POSITIONS_SYMBOL: &str = r#"
CREATE INDEX IF NOT EXISTS idx_alpaca_positions_symbol
    ON alpaca_positions(symbol, created_at DESC);
"#;

pub const CREATE_INDEX_ALPACA_POSITIONS_SIDE: &str = r#"
CREATE INDEX IF NOT EXISTS idx_alpaca_positions_side
    ON alpaca_positions(side) WHERE side IS NOT NULL;
"#;

pub const CREATE_INDEX_ALPACA_ACCOUNT_SNAPSHOTS_ACCOUNT: &str = r#"
CREATE INDEX IF NOT EXISTS idx_alpaca_account_snapshots_account
    ON alpaca_account_snapshots(account_id, created_at DESC);
"#;

pub const CREATE_INDEX_ALPACA_MARKET_EVENTS_SYMBOL: &str = r#"
CREATE INDEX IF NOT EXISTS idx_alpaca_market_events_symbol
    ON alpaca_market_events(symbol, created_at DESC);
"#;

pub const CREATE_INDEX_ALPACA_MARKET_EVENTS_TYPE: &str = r#"
CREATE INDEX IF NOT EXISTS idx_alpaca_market_events_type
    ON alpaca_market_events(event_type);
"#;

pub const CREATE_INDEX_ALPACA_PERFORMANCE_METRICS_CREATED: &str = r#"
CREATE INDEX IF NOT EXISTS idx_alpaca_performance_metrics_created
    ON alpaca_performance_metrics(created_at DESC);
"#;

pub const CREATE_INDEX_ALPACA_EXECUTION_STATS_CREATED: &str = r#"
CREATE INDEX IF NOT EXISTS idx_alpaca_execution_stats_created
    ON alpaca_execution_stats(created_at DESC);
"#;

// ============================================================================
// BACKWARD COMPATIBILITY
// ============================================================================

// Keep the old constant name for backward compatibility
pub const CREATE_HYPERTABLE: &str = CREATE_MARKET_TICKS_HYPERTABLE;
