-- Market data tables and hypertables
-- This migration creates time-series tables for market data

-- Market ticks table (main market data)
CREATE TABLE market_ticks (
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

-- Convert to hypertable
SELECT create_hypertable('market_ticks', 'timestamp',
    chunk_time_interval => INTERVAL '1 hour',
    if_not_exists => TRUE);

-- Order book snapshots table
CREATE TABLE order_book_snapshots (
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

-- Convert to hypertable
SELECT create_hypertable('order_book_snapshots', 'timestamp',
    chunk_time_interval => INTERVAL '30 minutes',
    if_not_exists => TRUE);

-- Create indexes for market data tables
CREATE INDEX IF NOT EXISTS idx_market_ticks_instrument_time
    ON market_ticks (instrument_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_market_ticks_provider
    ON market_ticks (provider);
CREATE INDEX IF NOT EXISTS idx_market_ticks_quality
    ON market_ticks (data_quality_score) WHERE data_quality_score >= 0.8;

CREATE INDEX IF NOT EXISTS idx_order_book_instrument_time
    ON order_book_snapshots (instrument_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_order_book_venue_side
    ON order_book_snapshots (venue, side);
CREATE INDEX IF NOT EXISTS idx_order_book_price
    ON order_book_snapshots (price) WHERE quantity > 0;
