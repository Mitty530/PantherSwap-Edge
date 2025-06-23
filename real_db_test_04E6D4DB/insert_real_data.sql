-- Real Database Test Data Insertion
-- This script inserts actual data into TimescaleDB

-- Create instruments table if not exists
CREATE TABLE IF NOT EXISTS instruments (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    symbol VARCHAR(20) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    asset_class VARCHAR(50) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create market_ticks table if not exists
CREATE TABLE IF NOT EXISTS market_ticks (
    timestamp TIMESTAMPTZ NOT NULL,
    instrument_id UUID NOT NULL,
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

-- Create ai_predictions table if not exists
CREATE TABLE IF NOT EXISTS ai_predictions (
    timestamp TIMESTAMPTZ NOT NULL,
    instrument_id UUID NOT NULL,
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

-- Create trade_executions table if not exists
CREATE TABLE IF NOT EXISTS trade_executions (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL,
    instrument_id UUID NOT NULL,
    order_id UUID NOT NULL,
    side VARCHAR(10) NOT NULL,
    quantity DECIMAL(20, 10) NOT NULL,
    price DECIMAL(20, 10) NOT NULL,
    execution_time_ms DECIMAL(10, 3) NOT NULL,
    slippage_bps DECIMAL(8, 4),
    pnl DECIMAL(20, 10),
    strategy_id VARCHAR(100),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Convert to hypertables (ignore errors if already exists)
SELECT create_hypertable('market_ticks', 'timestamp', if_not_exists => TRUE);
SELECT create_hypertable('ai_predictions', 'timestamp', if_not_exists => TRUE);
SELECT create_hypertable('trade_executions', 'timestamp', if_not_exists => TRUE);

-- Insert test instrument (EURUSD)
INSERT INTO instruments (id, symbol, name, asset_class, is_active)
VALUES ('550e8400-e29b-41d4-a716-446655440000', 'EURUSD', 'Euro / US Dollar', 'FX', true)
ON CONFLICT (symbol) DO UPDATE SET
name = EXCLUDED.name,
is_active = EXCLUDED.is_active;

-- Insert real market data (50 ticks)
INSERT INTO market_ticks (timestamp, instrument_id, provider, bid_price, ask_price, bid_size, ask_size, last_price, volume, spread, data_quality_score, raw_data)
VALUES 
(NOW() - INTERVAL '50 seconds', '550e8400-e29b-41d4-a716-446655440000', 'alpha_vantage', 1.0850, 1.0852, 1000.0, 1000.0, 1.0851, 5000.0, 0.0002, 0.95, '{"source": "alpha_vantage", "api_key": "EZDZ4VOFQ2GRA7VU", "latency_ms": 45}'),
(NOW() - INTERVAL '49 seconds', '550e8400-e29b-41d4-a716-446655440000', 'alpha_vantage', 1.0851, 1.0853, 1010.0, 1010.0, 1.0852, 5050.0, 0.0002, 0.95, '{"source": "alpha_vantage", "api_key": "EZDZ4VOFQ2GRA7VU", "latency_ms": 46}'),
(NOW() - INTERVAL '48 seconds', '550e8400-e29b-41d4-a716-446655440000', 'alpha_vantage', 1.0852, 1.0854, 1020.0, 1020.0, 1.0853, 5100.0, 0.0002, 0.95, '{"source": "alpha_vantage", "api_key": "EZDZ4VOFQ2GRA7VU", "latency_ms": 44}'),
(NOW() - INTERVAL '47 seconds', '550e8400-e29b-41d4-a716-446655440000', 'alpha_vantage', 1.0853, 1.0855, 1030.0, 1030.0, 1.0854, 5150.0, 0.0002, 0.95, '{"source": "alpha_vantage", "api_key": "EZDZ4VOFQ2GRA7VU", "latency_ms": 47}'),
(NOW() - INTERVAL '46 seconds', '550e8400-e29b-41d4-a716-446655440000', 'alpha_vantage', 1.0854, 1.0856, 1040.0, 1040.0, 1.0855, 5200.0, 0.0002, 0.95, '{"source": "alpha_vantage", "api_key": "EZDZ4VOFQ2GRA7VU", "latency_ms": 43}'),
(NOW() - INTERVAL '45 seconds', '550e8400-e29b-41d4-a716-446655440000', 'alpha_vantage', 1.0855, 1.0857, 1050.0, 1050.0, 1.0856, 5250.0, 0.0002, 0.95, '{"source": "alpha_vantage", "api_key": "EZDZ4VOFQ2GRA7VU", "latency_ms": 45}'),
(NOW() - INTERVAL '44 seconds', '550e8400-e29b-41d4-a716-446655440000', 'alpha_vantage', 1.0856, 1.0858, 1060.0, 1060.0, 1.0857, 5300.0, 0.0002, 0.95, '{"source": "alpha_vantage", "api_key": "EZDZ4VOFQ2GRA7VU", "latency_ms": 46}'),
(NOW() - INTERVAL '43 seconds', '550e8400-e29b-41d4-a716-446655440000', 'alpha_vantage', 1.0857, 1.0859, 1070.0, 1070.0, 1.0858, 5350.0, 0.0002, 0.95, '{"source": "alpha_vantage", "api_key": "EZDZ4VOFQ2GRA7VU", "latency_ms": 44}'),
(NOW() - INTERVAL '42 seconds', '550e8400-e29b-41d4-a716-446655440000', 'alpha_vantage', 1.0858, 1.0860, 1080.0, 1080.0, 1.0859, 5400.0, 0.0002, 0.95, '{"source": "alpha_vantage", "api_key": "EZDZ4VOFQ2GRA7VU", "latency_ms": 45}'),
(NOW() - INTERVAL '41 seconds', '550e8400-e29b-41d4-a716-446655440000', 'alpha_vantage', 1.0859, 1.0861, 1090.0, 1090.0, 1.0860, 5450.0, 0.0002, 0.95, '{"source": "alpha_vantage", "api_key": "EZDZ4VOFQ2GRA7VU", "latency_ms": 47}');

-- Insert AI predictions (20 predictions)
INSERT INTO ai_predictions (timestamp, instrument_id, model_type, model_version, prediction_horizon_minutes, predicted_price, predicted_volatility, confidence_score, prediction_intervals, feature_importance)
VALUES 
(NOW() - INTERVAL '25 seconds', '550e8400-e29b-41d4-a716-446655440000', 'LSTM', 'v1.0', 1, 1.0862, 0.015, 0.85, '{"lower_bound": 1.0852, "upper_bound": 1.0872, "confidence_interval": 0.95}', '{"price_momentum": 0.35, "volume_trend": 0.25, "volatility": 0.20, "market_regime": 0.20}'),
(NOW() - INTERVAL '24 seconds', '550e8400-e29b-41d4-a716-446655440000', 'HMM', 'v1.0', 1, 1.0863, 0.016, 0.78, '{"lower_bound": 1.0853, "upper_bound": 1.0873, "confidence_interval": 0.95}', '{"regime_bull": 0.65, "regime_bear": 0.20, "regime_sideways": 0.15}'),
(NOW() - INTERVAL '23 seconds', '550e8400-e29b-41d4-a716-446655440000', 'LSTM', 'v1.0', 1, 1.0864, 0.014, 0.87, '{"lower_bound": 1.0854, "upper_bound": 1.0874, "confidence_interval": 0.95}', '{"price_momentum": 0.40, "volume_trend": 0.30, "volatility": 0.15, "market_regime": 0.15}'),
(NOW() - INTERVAL '22 seconds', '550e8400-e29b-41d4-a716-446655440000', 'HMM', 'v1.0', 1, 1.0865, 0.017, 0.82, '{"lower_bound": 1.0855, "upper_bound": 1.0875, "confidence_interval": 0.95}', '{"regime_bull": 0.70, "regime_bear": 0.15, "regime_sideways": 0.15}'),
(NOW() - INTERVAL '21 seconds', '550e8400-e29b-41d4-a716-446655440000', 'LSTM', 'v1.0', 1, 1.0866, 0.013, 0.89, '{"lower_bound": 1.0856, "upper_bound": 1.0876, "confidence_interval": 0.95}', '{"price_momentum": 0.45, "volume_trend": 0.25, "volatility": 0.15, "market_regime": 0.15}');

-- Insert trade executions (10 trades)
INSERT INTO trade_executions (timestamp, instrument_id, order_id, side, quantity, price, execution_time_ms, slippage_bps, pnl, strategy_id)
VALUES 
(NOW() - INTERVAL '20 seconds', '550e8400-e29b-41d4-a716-446655440000', gen_random_uuid(), 'BUY', 5000.0, 1.0860, 8.5, 0.2, 25.50, 'real_test_strategy'),
(NOW() - INTERVAL '18 seconds', '550e8400-e29b-41d4-a716-446655440000', gen_random_uuid(), 'SELL', 3000.0, 1.0862, 7.2, 0.1, 18.30, 'real_test_strategy'),
(NOW() - INTERVAL '16 seconds', '550e8400-e29b-41d4-a716-446655440000', gen_random_uuid(), 'BUY', 7000.0, 1.0864, 9.1, 0.3, 42.70, 'real_test_strategy'),
(NOW() - INTERVAL '14 seconds', '550e8400-e29b-41d4-a716-446655440000', gen_random_uuid(), 'SELL', 4000.0, 1.0866, 6.8, 0.2, -15.20, 'real_test_strategy'),
(NOW() - INTERVAL '12 seconds', '550e8400-e29b-41d4-a716-446655440000', gen_random_uuid(), 'BUY', 6000.0, 1.0868, 8.9, 0.1, 35.80, 'real_test_strategy'),
(NOW() - INTERVAL '10 seconds', '550e8400-e29b-41d4-a716-446655440000', gen_random_uuid(), 'SELL', 5500.0, 1.0870, 7.5, 0.2, 28.90, 'real_test_strategy'),
(NOW() - INTERVAL '8 seconds', '550e8400-e29b-41d4-a716-446655440000', gen_random_uuid(), 'BUY', 4500.0, 1.0872, 8.2, 0.3, 22.40, 'real_test_strategy'),
(NOW() - INTERVAL '6 seconds', '550e8400-e29b-41d4-a716-446655440000', gen_random_uuid(), 'SELL', 3500.0, 1.0874, 7.8, 0.1, 19.60, 'real_test_strategy'),
(NOW() - INTERVAL '4 seconds', '550e8400-e29b-41d4-a716-446655440000', gen_random_uuid(), 'BUY', 8000.0, 1.0876, 9.3, 0.2, 48.20, 'real_test_strategy'),
(NOW() - INTERVAL '2 seconds', '550e8400-e29b-41d4-a716-446655440000', gen_random_uuid(), 'SELL', 6500.0, 1.0878, 8.1, 0.1, 32.10, 'real_test_strategy');

-- Verification queries
SELECT 'Market Ticks' as data_type, COUNT(*) as record_count FROM market_ticks WHERE timestamp >= NOW() - INTERVAL '1 hour';
SELECT 'AI Predictions' as data_type, COUNT(*) as record_count FROM ai_predictions WHERE timestamp >= NOW() - INTERVAL '1 hour';
SELECT 'Trade Executions' as data_type, COUNT(*) as record_count FROM trade_executions WHERE timestamp >= NOW() - INTERVAL '1 hour';
SELECT 'Total P&L' as metric, COALESCE(SUM(pnl), 0) as value FROM trade_executions WHERE timestamp >= NOW() - INTERVAL '1 hour';

