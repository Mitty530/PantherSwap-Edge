# Database Schema Reference

## Overview

This document provides a comprehensive reference for the PantherSwap Edge database schema, including all tables, indexes, constraints, and TimescaleDB-specific configurations.

## Table of Contents

1. [Reference Tables](#reference-tables)
2. [Time-Series Tables](#time-series-tables)
3. [Indexes](#indexes)
4. [Constraints](#constraints)
5. [TimescaleDB Configuration](#timescaledb-configuration)
6. [Data Types](#data-types)

## Reference Tables

### Instruments

**Purpose**: Master table for all tradeable instruments

```sql
CREATE TABLE instruments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    symbol VARCHAR(20) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    asset_class VARCHAR(50) NOT NULL,
    exchange VARCHAR(50) NOT NULL,
    currency VARCHAR(10) NOT NULL,
    tick_size DECIMAL(20, 10) NOT NULL,
    lot_size DECIMAL(20, 10) NOT NULL,
    trading_hours JSONB,
    metadata JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
```

**Indexes**:
- `PRIMARY KEY (id)`
- `UNIQUE INDEX idx_instruments_symbol (symbol)`
- `INDEX idx_instruments_asset_class (asset_class)`
- `INDEX idx_instruments_exchange (exchange)`

**Sample Data**:
```sql
INSERT INTO instruments (symbol, name, asset_class, exchange, currency, tick_size, lot_size) VALUES
('BTC/USD', 'Bitcoin to US Dollar', 'cryptocurrency', 'binance', 'USD', 0.01, 0.001),
('ETH/USD', 'Ethereum to US Dollar', 'cryptocurrency', 'binance', 'USD', 0.01, 0.001),
('AAPL', 'Apple Inc.', 'equity', 'nasdaq', 'USD', 0.01, 1.0);
```

## Time-Series Tables

### Market Ticks

**Purpose**: Real-time market data with bid/ask prices and volumes

```sql
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
SELECT create_hypertable('market_ticks', 'timestamp', chunk_time_interval => INTERVAL '1 hour');
```

**Key Fields**:
- `timestamp`: Microsecond-precision timestamp
- `spread`: Calculated bid-ask spread
- `data_quality_score`: 0.0-1.0 quality assessment
- `raw_data`: Original provider data in JSONB format

### AI Predictions

**Purpose**: Machine learning model predictions for price movements

```sql
CREATE TABLE ai_predictions (
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

-- Convert to hypertable
SELECT create_hypertable('ai_predictions', 'timestamp', chunk_time_interval => INTERVAL '6 hours');
```

**Key Fields**:
- `model_type`: 'lstm', 'transformer', 'ensemble', etc.
- `prediction_horizon_minutes`: Forecast time horizon
- `confidence_score`: 0.0-1.0 model confidence
- `prediction_intervals`: Confidence intervals in JSONB
- `feature_importance`: Feature weights in JSONB

### Trading Signals

**Purpose**: Generated trading signals from various strategies

```sql
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
SELECT create_hypertable('trading_signals', 'timestamp', chunk_time_interval => INTERVAL '1 hour');
```

**Key Fields**:
- `strategy_type`: 'momentum', 'mean_reversion', 'arbitrage', etc.
- `signal_type`: 'BUY', 'SELL', 'HOLD'
- `risk_score`: 0.0-1.0 risk assessment
- `time_horizon`: Expected signal duration

### Order Book Snapshots

**Purpose**: Market depth data snapshots

```sql
CREATE TABLE order_book_snapshots (
    timestamp TIMESTAMPTZ NOT NULL,
    instrument_id UUID NOT NULL REFERENCES instruments(id),
    provider VARCHAR(50) NOT NULL,
    bids JSONB NOT NULL,
    asks JSONB NOT NULL,
    sequence_number BIGINT,
    checksum VARCHAR(64),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Convert to hypertable
SELECT create_hypertable('order_book_snapshots', 'timestamp', chunk_time_interval => INTERVAL '30 minutes');
```

**JSONB Structure for bids/asks**:
```json
[
  {"price": "50000.00", "size": "1.5", "orders": 3},
  {"price": "49999.50", "size": "2.1", "orders": 5},
  {"price": "49999.00", "size": "0.8", "orders": 2}
]
```

### Microstructure Analysis

**Purpose**: Market microstructure metrics and analysis

```sql
CREATE TABLE microstructure_analysis (
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

-- Convert to hypertable
SELECT create_hypertable('microstructure_analysis', 'timestamp', chunk_time_interval => INTERVAL '2 hours');
```

**Key Fields**:
- `order_book_imbalance`: -1.0 to 1.0 (negative = sell pressure)
- `volatility_regime`: 'low', 'medium', 'high', 'extreme'
- `market_maker_presence`: 0.0-1.0 market maker activity score

### Trade Executions

**Purpose**: Actual trade execution records

```sql
CREATE TABLE trade_executions (
    timestamp TIMESTAMPTZ NOT NULL,
    instrument_id UUID NOT NULL REFERENCES instruments(id),
    trade_id UUID NOT NULL DEFAULT uuid_generate_v4(),
    signal_id UUID REFERENCES trading_signals(id),
    side VARCHAR(10) NOT NULL,
    quantity DECIMAL(20, 10) NOT NULL,
    price DECIMAL(20, 10) NOT NULL,
    commission DECIMAL(20, 10) NOT NULL,
    slippage DECIMAL(8, 6),
    execution_venue VARCHAR(50) NOT NULL,
    execution_quality DECIMAL(5, 4),
    metadata JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Convert to hypertable
SELECT create_hypertable('trade_executions', 'timestamp', chunk_time_interval => INTERVAL '1 hour');
```

### Risk Metrics

**Purpose**: Real-time risk monitoring and metrics

```sql
CREATE TABLE risk_metrics (
    timestamp TIMESTAMPTZ NOT NULL,
    instrument_id UUID REFERENCES instruments(id),
    portfolio_id UUID,
    var_1d DECIMAL(20, 10),
    var_5d DECIMAL(20, 10),
    expected_shortfall DECIMAL(20, 10),
    beta DECIMAL(8, 6),
    sharpe_ratio DECIMAL(8, 6),
    max_drawdown DECIMAL(8, 6),
    volatility DECIMAL(8, 6),
    correlation_matrix JSONB,
    risk_factors JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Convert to hypertable
SELECT create_hypertable('risk_metrics', 'timestamp', chunk_time_interval => INTERVAL '4 hours');
```

## Indexes

### Primary Indexes

**Time-Series Performance Indexes**:
```sql
-- Market ticks optimized for trading queries
CREATE INDEX CONCURRENTLY idx_market_ticks_instrument_time
ON market_ticks (instrument_id, timestamp DESC);

CREATE INDEX CONCURRENTLY idx_market_ticks_provider_time
ON market_ticks (provider, timestamp DESC);

-- AI predictions for model performance analysis
CREATE INDEX CONCURRENTLY idx_ai_predictions_model_time
ON ai_predictions (model_type, timestamp DESC);

CREATE INDEX CONCURRENTLY idx_ai_predictions_confidence
ON ai_predictions (confidence_score DESC) WHERE confidence_score > 0.8;

-- Trading signals for strategy analysis
CREATE INDEX CONCURRENTLY idx_trading_signals_strategy_time
ON trading_signals (strategy_type, timestamp DESC);

CREATE INDEX CONCURRENTLY idx_trading_signals_confidence
ON trading_signals (confidence_score DESC, signal_type);
```

### Specialized Indexes

**BRIN Indexes for Large Tables**:
```sql
-- Block Range Indexes for time-series data
CREATE INDEX CONCURRENTLY idx_market_ticks_timestamp_brin
ON market_ticks USING BRIN (timestamp);

CREATE INDEX CONCURRENTLY idx_order_book_timestamp_brin
ON order_book_snapshots USING BRIN (timestamp);
```

**GIN Indexes for JSONB**:
```sql
-- JSONB indexes for metadata searches
CREATE INDEX CONCURRENTLY idx_market_ticks_raw_data_gin
ON market_ticks USING GIN (raw_data);

CREATE INDEX CONCURRENTLY idx_ai_predictions_features_gin
ON ai_predictions USING GIN (feature_importance);
```

**Partial Indexes for Performance**:
```sql
-- High-confidence predictions only
CREATE INDEX CONCURRENTLY idx_ai_predictions_high_confidence
ON ai_predictions (timestamp DESC, predicted_price)
WHERE confidence_score > 0.9;

-- Recent trading signals
CREATE INDEX CONCURRENTLY idx_trading_signals_recent
ON trading_signals (instrument_id, timestamp DESC)
WHERE timestamp > NOW() - INTERVAL '24 hours';

-- Large trades only
CREATE INDEX CONCURRENTLY idx_trade_executions_large
ON trade_executions (timestamp DESC, quantity)
WHERE quantity > 1000;
```

## Constraints

### Foreign Key Constraints

```sql
-- Instrument references
ALTER TABLE market_ticks
ADD CONSTRAINT fk_market_ticks_instrument
FOREIGN KEY (instrument_id) REFERENCES instruments(id);

ALTER TABLE ai_predictions
ADD CONSTRAINT fk_ai_predictions_instrument
FOREIGN KEY (instrument_id) REFERENCES instruments(id);

ALTER TABLE trading_signals
ADD CONSTRAINT fk_trading_signals_instrument
FOREIGN KEY (instrument_id) REFERENCES instruments(id);

-- Signal to execution tracking
ALTER TABLE trade_executions
ADD CONSTRAINT fk_trade_executions_signal
FOREIGN KEY (signal_id) REFERENCES trading_signals(id);
```

### Check Constraints

```sql
-- Data quality constraints
ALTER TABLE market_ticks
ADD CONSTRAINT chk_market_ticks_quality_score
CHECK (data_quality_score >= 0.0 AND data_quality_score <= 1.0);

ALTER TABLE market_ticks
ADD CONSTRAINT chk_market_ticks_spread_positive
CHECK (spread >= 0);

-- Confidence score constraints
ALTER TABLE ai_predictions
ADD CONSTRAINT chk_ai_predictions_confidence
CHECK (confidence_score >= 0.0 AND confidence_score <= 1.0);

ALTER TABLE trading_signals
ADD CONSTRAINT chk_trading_signals_confidence
CHECK (confidence_score >= 0.0 AND confidence_score <= 1.0);

-- Trade execution constraints
ALTER TABLE trade_executions
ADD CONSTRAINT chk_trade_executions_side
CHECK (side IN ('BUY', 'SELL'));

ALTER TABLE trade_executions
ADD CONSTRAINT chk_trade_executions_quantity_positive
CHECK (quantity > 0);
```

## TimescaleDB Configuration

### Hypertable Settings

```sql
-- Market ticks: High-frequency data, 1-hour chunks
SELECT create_hypertable('market_ticks', 'timestamp',
    chunk_time_interval => INTERVAL '1 hour',
    create_default_indexes => FALSE);

-- Order book: Very high-frequency, 30-minute chunks
SELECT create_hypertable('order_book_snapshots', 'timestamp',
    chunk_time_interval => INTERVAL '30 minutes',
    create_default_indexes => FALSE);

-- Trading signals: Moderate frequency, 1-hour chunks
SELECT create_hypertable('trading_signals', 'timestamp',
    chunk_time_interval => INTERVAL '1 hour');

-- AI predictions: Lower frequency, 6-hour chunks
SELECT create_hypertable('ai_predictions', 'timestamp',
    chunk_time_interval => INTERVAL '6 hours');
```

### Compression Policies

```sql
-- Compress data older than specified intervals
SELECT add_compression_policy('market_ticks', INTERVAL '1 day');
SELECT add_compression_policy('order_book_snapshots', INTERVAL '6 hours');
SELECT add_compression_policy('trading_signals', INTERVAL '1 day');
SELECT add_compression_policy('ai_predictions', INTERVAL '7 days');
SELECT add_compression_policy('microstructure_analysis', INTERVAL '2 days');
SELECT add_compression_policy('trade_executions', INTERVAL '1 day');
SELECT add_compression_policy('risk_metrics', INTERVAL '7 days');
```

### Retention Policies

```sql
-- Automatic data cleanup
SELECT add_retention_policy('market_ticks', INTERVAL '90 days');
SELECT add_retention_policy('order_book_snapshots', INTERVAL '30 days');
SELECT add_retention_policy('trading_signals', INTERVAL '180 days');
SELECT add_retention_policy('ai_predictions', INTERVAL '365 days');
SELECT add_retention_policy('microstructure_analysis', INTERVAL '180 days');
SELECT add_retention_policy('trade_executions', INTERVAL '2555 days'); -- 7 years for compliance
SELECT add_retention_policy('risk_metrics', INTERVAL '1095 days'); -- 3 years
```

### Continuous Aggregates

```sql
-- 1-minute OHLCV aggregates
CREATE MATERIALIZED VIEW market_ticks_1min
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('1 minute', timestamp) AS bucket,
    instrument_id,
    provider,
    FIRST(last_price, timestamp) AS open,
    MAX(last_price) AS high,
    MIN(last_price) AS low,
    LAST(last_price, timestamp) AS close,
    SUM(volume) AS volume,
    AVG(spread) AS avg_spread,
    COUNT(*) AS tick_count
FROM market_ticks
GROUP BY bucket, instrument_id, provider;

-- 5-minute aggregates
CREATE MATERIALIZED VIEW market_ticks_5min
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('5 minutes', timestamp) AS bucket,
    instrument_id,
    provider,
    FIRST(last_price, timestamp) AS open,
    MAX(last_price) AS high,
    MIN(last_price) AS low,
    LAST(last_price, timestamp) AS close,
    SUM(volume) AS volume,
    AVG(spread) AS avg_spread,
    COUNT(*) AS tick_count
FROM market_ticks
GROUP BY bucket, instrument_id, provider;
```

## Data Types

### Custom Types

```sql
-- Enum types for standardized values
CREATE TYPE signal_type AS ENUM ('BUY', 'SELL', 'HOLD');
CREATE TYPE trade_side AS ENUM ('BUY', 'SELL');
CREATE TYPE volatility_regime AS ENUM ('low', 'medium', 'high', 'extreme');
CREATE TYPE asset_class AS ENUM ('equity', 'cryptocurrency', 'forex', 'commodity', 'bond');
```

### JSONB Schemas

**Trading Signal Metadata**:
```json
{
  "strategy_params": {
    "lookback_period": 20,
    "threshold": 0.02,
    "risk_multiplier": 1.5
  },
  "market_conditions": {
    "volatility": "medium",
    "trend": "bullish",
    "volume_profile": "above_average"
  },
  "execution_instructions": {
    "order_type": "limit",
    "time_in_force": "GTC",
    "iceberg_size": 100
  }
}
```

**AI Prediction Feature Importance**:
```json
{
  "price_features": {
    "sma_20": 0.15,
    "rsi_14": 0.12,
    "bollinger_position": 0.08
  },
  "volume_features": {
    "volume_sma_ratio": 0.10,
    "volume_profile": 0.07
  },
  "market_structure": {
    "order_book_imbalance": 0.18,
    "spread_normalized": 0.09
  },
  "external_factors": {
    "market_sentiment": 0.11,
    "correlation_spy": 0.10
  }
}
```

---

This schema reference provides the complete structure for the PantherSwap Edge database, optimized for high-frequency trading and real-time analytics.
