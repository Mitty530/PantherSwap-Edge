-- Trading signals and execution tables
-- This migration creates tables for trading signals and trade executions

-- Trading signals table
CREATE TABLE trading_signals (
    timestamp TIMESTAMPTZ NOT NULL,
    instrument_id UUID NOT NULL REFERENCES instruments(id),
    strategy_type VARCHAR(50) NOT NULL,
    signal_type VARCHAR(20) NOT NULL,
    confidence_score DECIMAL(5, 4) NOT NULL,
    target_price DECIMAL(20, 10),
    stop_loss DECIMAL(20, 10),
    take_profit DECIMAL(20, 10),
    position_size DECIMAL(20, 10) NOT NULL,
    risk_score DECIMAL(5, 4) NOT NULL,
    time_horizon INTERVAL,
    metadata JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Convert to hypertable
SELECT create_hypertable('trading_signals', 'timestamp',
    chunk_time_interval => INTERVAL '1 hour',
    if_not_exists => TRUE);

-- Trade executions table
CREATE TABLE trade_executions (
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

-- Convert to hypertable
SELECT create_hypertable('trade_executions', 'timestamp',
    chunk_time_interval => INTERVAL '1 hour',
    if_not_exists => TRUE);

-- Create indexes for trading tables
CREATE INDEX IF NOT EXISTS idx_trading_signals_instrument_time
    ON trading_signals (instrument_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_trading_signals_strategy
    ON trading_signals (strategy_type);
CREATE INDEX IF NOT EXISTS idx_trading_signals_confidence
    ON trading_signals (confidence_score) WHERE confidence_score >= 0.6;

CREATE INDEX IF NOT EXISTS idx_trade_executions_instrument_time
    ON trade_executions (instrument_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_trade_executions_strategy
    ON trade_executions (strategy_name);
CREATE INDEX IF NOT EXISTS idx_trade_executions_pnl
    ON trade_executions (pnl) WHERE pnl IS NOT NULL;
