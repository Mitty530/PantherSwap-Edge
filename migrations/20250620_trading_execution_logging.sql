-- Trading execution logging tables
-- This migration creates comprehensive tables for logging all trading activities

-- Orders table for tracking all order lifecycle
CREATE TABLE IF NOT EXISTS orders (
    id UUID PRIMARY KEY,
    instrument_id UUID NOT NULL REFERENCES instruments(id),
    side VARCHAR(10) NOT NULL, -- 'BUY' or 'SELL'
    quantity DECIMAL(20, 10) NOT NULL,
    filled_quantity DECIMAL(20, 10) NOT NULL DEFAULT 0,
    remaining_quantity DECIMAL(20, 10) NOT NULL,
    order_type VARCHAR(20) NOT NULL, -- 'MARKET', 'LIMIT', 'STOP', etc.
    price DECIMAL(20, 10),
    stop_price DECIMAL(20, 10),
    time_in_force VARCHAR(10) NOT NULL, -- 'GTC', 'IOC', 'FOK', 'DAY'
    execution_style VARCHAR(20) NOT NULL, -- 'AGGRESSIVE', 'PASSIVE'
    status VARCHAR(20) NOT NULL, -- 'PENDING', 'PARTIALLY_FILLED', 'FILLED', 'CANCELLED', 'REJECTED'
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    strategy_name VARCHAR(100),
    metadata JSONB
);

-- Fills table for tracking individual fill events
CREATE TABLE IF NOT EXISTS fills (
    id UUID PRIMARY KEY,
    order_id UUID NOT NULL REFERENCES orders(id),
    quantity DECIMAL(20, 10) NOT NULL,
    price DECIMAL(20, 10) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    commission DECIMAL(20, 10) NOT NULL DEFAULT 0,
    venue VARCHAR(50) NOT NULL DEFAULT 'internal',
    liquidity_flag VARCHAR(20) NOT NULL DEFAULT 'unknown', -- 'MAKER', 'TAKER', 'UNKNOWN'
    metadata JSONB
);

-- Position updates table for tracking position changes
CREATE TABLE IF NOT EXISTS position_updates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMPTZ NOT NULL,
    instrument_id UUID NOT NULL REFERENCES instruments(id),
    strategy_name VARCHAR(100) NOT NULL,
    size DECIMAL(20, 10) NOT NULL,
    entry_price DECIMAL(20, 10) NOT NULL,
    entry_time TIMESTAMPTZ NOT NULL,
    stop_loss DECIMAL(20, 10),
    take_profit DECIMAL(20, 10),
    unrealized_pnl DECIMAL(20, 10) NOT NULL DEFAULT 0,
    realized_pnl DECIMAL(20, 10) NOT NULL DEFAULT 0,
    risk_score DECIMAL(5, 4) NOT NULL,
    metadata JSONB
);

-- P&L records table for tracking profit and loss
CREATE TABLE IF NOT EXISTS pnl_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMPTZ NOT NULL,
    instrument_id UUID NOT NULL REFERENCES instruments(id),
    strategy_name VARCHAR(100) NOT NULL,
    realized_pnl DECIMAL(20, 10) NOT NULL,
    unrealized_pnl DECIMAL(20, 10) NOT NULL,
    total_pnl DECIMAL(20, 10) NOT NULL,
    trade_count INTEGER NOT NULL DEFAULT 0,
    win_rate DECIMAL(5, 4) NOT NULL DEFAULT 0,
    sharpe_ratio DECIMAL(8, 6),
    max_drawdown DECIMAL(8, 6),
    metadata JSONB
);

-- Convert time-series tables to hypertables
SELECT create_hypertable('fills', 'timestamp',
    chunk_time_interval => INTERVAL '1 hour',
    if_not_exists => TRUE);

SELECT create_hypertable('position_updates', 'timestamp',
    chunk_time_interval => INTERVAL '1 hour',
    if_not_exists => TRUE);

SELECT create_hypertable('pnl_records', 'timestamp',
    chunk_time_interval => INTERVAL '1 day',
    if_not_exists => TRUE);

-- Create indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_orders_instrument_id ON orders (instrument_id);
CREATE INDEX IF NOT EXISTS idx_orders_status ON orders (status);
CREATE INDEX IF NOT EXISTS idx_orders_strategy ON orders (strategy_name);
CREATE INDEX IF NOT EXISTS idx_orders_created_at ON orders (created_at DESC);

CREATE INDEX IF NOT EXISTS idx_fills_order_id ON fills (order_id);
CREATE INDEX IF NOT EXISTS idx_fills_timestamp ON fills (timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_fills_venue ON fills (venue);

CREATE INDEX IF NOT EXISTS idx_position_updates_instrument ON position_updates (instrument_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_position_updates_strategy ON position_updates (strategy_name, timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_pnl_records_instrument ON pnl_records (instrument_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_pnl_records_strategy ON pnl_records (strategy_name, timestamp DESC);

-- Add compression policies for older data
SELECT add_compression_policy('fills', INTERVAL '24 hours');
SELECT add_compression_policy('position_updates', INTERVAL '7 days');
SELECT add_compression_policy('pnl_records', INTERVAL '30 days');

-- Add retention policies for very old data
SELECT add_retention_policy('fills', INTERVAL '90 days');
SELECT add_retention_policy('position_updates', INTERVAL '1 year');
SELECT add_retention_policy('pnl_records', INTERVAL '5 years');
