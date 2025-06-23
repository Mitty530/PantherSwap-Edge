-- AI predictions and analytics tables
-- This migration creates tables for AI model predictions and microstructure analysis

-- AI predictions table
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
SELECT create_hypertable('ai_predictions', 'timestamp',
    chunk_time_interval => INTERVAL '1 hour',
    if_not_exists => TRUE);

-- Microstructure analysis table
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
SELECT create_hypertable('microstructure_analysis', 'timestamp',
    chunk_time_interval => INTERVAL '1 hour',
    if_not_exists => TRUE);

-- Create indexes for AI and analytics tables
CREATE INDEX IF NOT EXISTS idx_ai_predictions_instrument_time
    ON ai_predictions (instrument_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_ai_predictions_model
    ON ai_predictions (model_type, model_version);
CREATE INDEX IF NOT EXISTS idx_ai_predictions_confidence
    ON ai_predictions (confidence_score) WHERE confidence_score >= 0.7;

CREATE INDEX IF NOT EXISTS idx_microstructure_instrument_time
    ON microstructure_analysis (instrument_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_microstructure_regime
    ON microstructure_analysis (volatility_regime);
CREATE INDEX IF NOT EXISTS idx_microstructure_liquidity
    ON microstructure_analysis (liquidity_score) WHERE liquidity_score >= 0.5;
