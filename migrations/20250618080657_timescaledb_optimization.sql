-- TimescaleDB optimization: compression and retention policies
-- This migration sets up compression and retention policies for optimal performance

-- Enable compression on hypertables
ALTER TABLE market_ticks SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'instrument_id, provider',
    timescaledb.compress_orderby = 'timestamp DESC'
);

ALTER TABLE ai_predictions SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'instrument_id, model_type',
    timescaledb.compress_orderby = 'timestamp DESC'
);

ALTER TABLE microstructure_analysis SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'instrument_id',
    timescaledb.compress_orderby = 'timestamp DESC'
);

ALTER TABLE order_book_snapshots SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'instrument_id, venue, side',
    timescaledb.compress_orderby = 'timestamp DESC'
);

-- Add compression policies (compress data older than 24 hours)
SELECT add_compression_policy('market_ticks', INTERVAL '24 hours');
SELECT add_compression_policy('ai_predictions', INTERVAL '24 hours');
SELECT add_compression_policy('microstructure_analysis', INTERVAL '24 hours');
SELECT add_compression_policy('order_book_snapshots', INTERVAL '12 hours');

-- Add retention policies (delete old data automatically)
SELECT add_retention_policy('market_ticks', INTERVAL '90 days');
SELECT add_retention_policy('order_book_snapshots', INTERVAL '30 days');
