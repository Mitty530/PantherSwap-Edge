-- Risk management tables
-- This migration creates tables for risk metrics and monitoring

-- Risk metrics table
CREATE TABLE risk_metrics (
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

-- Convert to hypertable
SELECT create_hypertable('risk_metrics', 'timestamp',
    chunk_time_interval => INTERVAL '1 hour',
    if_not_exists => TRUE);
