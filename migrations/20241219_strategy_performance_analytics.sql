-- Strategy Performance Analytics Tables for Weight Optimization
-- Migration: 20241219_strategy_performance_analytics.sql

-- Strategy performance metrics table
CREATE TABLE IF NOT EXISTS strategy_performance_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    strategy_type TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Basic performance metrics
    total_trades BIGINT NOT NULL DEFAULT 0,
    winning_trades BIGINT NOT NULL DEFAULT 0,
    total_pnl DECIMAL(20,8) NOT NULL DEFAULT 0,
    sharpe_ratio DECIMAL(10,6) NOT NULL DEFAULT 0,
    max_drawdown DECIMAL(10,6) NOT NULL DEFAULT 0,
    avg_holding_period_seconds BIGINT NOT NULL DEFAULT 0,
    success_rate DECIMAL(10,6) NOT NULL DEFAULT 0,
    avg_return_per_trade DECIMAL(20,8) NOT NULL DEFAULT 0,
    
    -- Enhanced risk metrics
    sortino_ratio DECIMAL(10,6) NOT NULL DEFAULT 0,
    calmar_ratio DECIMAL(10,6) NOT NULL DEFAULT 0,
    information_ratio DECIMAL(10,6) NOT NULL DEFAULT 0,
    var_95 DECIMAL(20,8) NOT NULL DEFAULT 0,
    expected_shortfall DECIMAL(20,8) NOT NULL DEFAULT 0,
    profit_factor DECIMAL(10,6) NOT NULL DEFAULT 0,
    recovery_factor DECIMAL(10,6) NOT NULL DEFAULT 0,
    tail_ratio DECIMAL(10,6) NOT NULL DEFAULT 0,
    skewness DECIMAL(10,6) NOT NULL DEFAULT 0,
    kurtosis DECIMAL(10,6) NOT NULL DEFAULT 0,
    
    -- Rolling metrics
    rolling_sharpe_30d DECIMAL(10,6) NOT NULL DEFAULT 0,
    rolling_volatility_30d DECIMAL(10,6) NOT NULL DEFAULT 0,
    max_consecutive_losses INTEGER NOT NULL DEFAULT 0,
    avg_win_loss_ratio DECIMAL(10,6) NOT NULL DEFAULT 0,
    kelly_fraction DECIMAL(10,6) NOT NULL DEFAULT 0,
    
    -- Market correlation metrics
    correlation_to_market DECIMAL(10,6) NOT NULL DEFAULT 0,
    beta DECIMAL(10,6) NOT NULL DEFAULT 0,
    alpha DECIMAL(20,8) NOT NULL DEFAULT 0,
    tracking_error DECIMAL(10,6) NOT NULL DEFAULT 0,
    upside_capture DECIMAL(10,6) NOT NULL DEFAULT 0,
    downside_capture DECIMAL(10,6) NOT NULL DEFAULT 0,
    
    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Strategy weight allocations table
CREATE TABLE IF NOT EXISTS strategy_weight_allocations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Strategy weights (must sum to 1.0)
    predictive_market_making DECIMAL(10,6) NOT NULL DEFAULT 0.25,
    microstructure_momentum DECIMAL(10,6) NOT NULL DEFAULT 0.25,
    regime_arbitrage DECIMAL(10,6) NOT NULL DEFAULT 0.25,
    liquidity_harvesting DECIMAL(10,6) NOT NULL DEFAULT 0.25,
    
    -- Optimization metadata
    rebalance_reason TEXT NOT NULL DEFAULT 'Initial allocation',
    optimization_method TEXT NOT NULL DEFAULT 'manual',
    target_sharpe_ratio DECIMAL(10,6),
    achieved_sharpe_ratio DECIMAL(10,6),
    portfolio_volatility DECIMAL(10,6),
    diversification_ratio DECIMAL(10,6),
    
    -- Validation
    weights_sum DECIMAL(10,6) GENERATED ALWAYS AS (
        predictive_market_making + microstructure_momentum + 
        regime_arbitrage + liquidity_harvesting
    ) STORED,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Strategy analytics table for detailed performance tracking
CREATE TABLE IF NOT EXISTS strategy_analytics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    strategy_type TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Performance reference
    performance_metrics_id UUID REFERENCES strategy_performance_metrics(id),
    
    -- Risk-adjusted metrics
    risk_adjusted_return DECIMAL(20,8) NOT NULL DEFAULT 0,
    volatility_adjusted_sharpe DECIMAL(10,6) NOT NULL DEFAULT 0,
    
    -- Portfolio contribution metrics
    contribution_to_portfolio_risk DECIMAL(10,6) NOT NULL DEFAULT 0,
    marginal_var DECIMAL(20,8) NOT NULL DEFAULT 0,
    component_var DECIMAL(20,8) NOT NULL DEFAULT 0,
    diversification_ratio DECIMAL(10,6) NOT NULL DEFAULT 1.0,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Strategy regime performance table
CREATE TABLE IF NOT EXISTS strategy_regime_performance (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    strategy_type TEXT NOT NULL,
    regime_type TEXT NOT NULL, -- 'trending', 'volatile', 'normal', 'crisis'
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Performance in specific regime
    regime_return DECIMAL(20,8) NOT NULL DEFAULT 0,
    regime_volatility DECIMAL(10,6) NOT NULL DEFAULT 0,
    regime_sharpe DECIMAL(10,6) NOT NULL DEFAULT 0,
    regime_max_drawdown DECIMAL(10,6) NOT NULL DEFAULT 0,
    regime_win_rate DECIMAL(10,6) NOT NULL DEFAULT 0,
    
    -- Regime detection accuracy
    regime_detection_accuracy DECIMAL(10,6) NOT NULL DEFAULT 0,
    regime_transition_success DECIMAL(10,6) NOT NULL DEFAULT 0,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Strategy correlation matrix table
CREATE TABLE IF NOT EXISTS strategy_correlation_matrix (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Strategy pair correlation
    strategy_1 TEXT NOT NULL,
    strategy_2 TEXT NOT NULL,
    correlation_coefficient DECIMAL(10,6) NOT NULL DEFAULT 0,
    
    -- Time window for correlation calculation
    calculation_window_days INTEGER NOT NULL DEFAULT 30,
    data_points_used INTEGER NOT NULL DEFAULT 0,
    
    -- Statistical significance
    p_value DECIMAL(10,6),
    confidence_interval_lower DECIMAL(10,6),
    confidence_interval_upper DECIMAL(10,6),
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Ensure unique strategy pairs per timestamp
    UNIQUE(timestamp, strategy_1, strategy_2)
);

-- Daily strategy returns table for detailed analysis
CREATE TABLE IF NOT EXISTS strategy_daily_returns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    strategy_type TEXT NOT NULL,
    date DATE NOT NULL,
    
    -- Daily performance
    daily_return DECIMAL(20,8) NOT NULL DEFAULT 0,
    daily_volatility DECIMAL(10,6) NOT NULL DEFAULT 0,
    daily_trades INTEGER NOT NULL DEFAULT 0,
    daily_pnl DECIMAL(20,8) NOT NULL DEFAULT 0,
    
    -- Intraday metrics
    max_intraday_drawdown DECIMAL(10,6) NOT NULL DEFAULT 0,
    max_intraday_gain DECIMAL(10,6) NOT NULL DEFAULT 0,
    
    -- Market conditions
    market_regime TEXT,
    market_volatility DECIMAL(10,6),
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Ensure one record per strategy per day
    UNIQUE(strategy_type, date)
);

-- Optimization history table
CREATE TABLE IF NOT EXISTS strategy_optimization_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Optimization parameters
    optimization_method TEXT NOT NULL,
    target_function TEXT NOT NULL, -- 'sharpe', 'calmar', 'sortino', 'multi_objective'
    
    -- Before optimization
    weights_before JSONB NOT NULL,
    portfolio_metrics_before JSONB NOT NULL,
    
    -- After optimization
    weights_after JSONB NOT NULL,
    portfolio_metrics_after JSONB NOT NULL,
    
    -- Optimization results
    improvement_score DECIMAL(10,6) NOT NULL DEFAULT 0,
    convergence_iterations INTEGER NOT NULL DEFAULT 0,
    optimization_duration_ms INTEGER NOT NULL DEFAULT 0,
    
    -- Validation
    backtest_sharpe DECIMAL(10,6),
    backtest_max_drawdown DECIMAL(10,6),
    out_of_sample_performance DECIMAL(10,6),
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_strategy_performance_metrics_strategy_timestamp 
    ON strategy_performance_metrics(strategy_type, timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_strategy_weight_allocations_timestamp 
    ON strategy_weight_allocations(timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_strategy_analytics_strategy_timestamp 
    ON strategy_analytics(strategy_type, timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_strategy_regime_performance_strategy_regime 
    ON strategy_regime_performance(strategy_type, regime_type, timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_strategy_correlation_matrix_timestamp 
    ON strategy_correlation_matrix(timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_strategy_daily_returns_strategy_date 
    ON strategy_daily_returns(strategy_type, date DESC);

CREATE INDEX IF NOT EXISTS idx_strategy_optimization_history_timestamp 
    ON strategy_optimization_history(timestamp DESC);

-- Hypertables for time-series data (TimescaleDB)
SELECT create_hypertable('strategy_performance_metrics', 'timestamp', if_not_exists => TRUE);
SELECT create_hypertable('strategy_weight_allocations', 'timestamp', if_not_exists => TRUE);
SELECT create_hypertable('strategy_analytics', 'timestamp', if_not_exists => TRUE);
SELECT create_hypertable('strategy_regime_performance', 'timestamp', if_not_exists => TRUE);
SELECT create_hypertable('strategy_correlation_matrix', 'timestamp', if_not_exists => TRUE);
SELECT create_hypertable('strategy_daily_returns', 'date', if_not_exists => TRUE);
SELECT create_hypertable('strategy_optimization_history', 'timestamp', if_not_exists => TRUE);

-- Data retention policies (keep 1 year of detailed data)
SELECT add_retention_policy('strategy_performance_metrics', INTERVAL '1 year', if_not_exists => TRUE);
SELECT add_retention_policy('strategy_analytics', INTERVAL '1 year', if_not_exists => TRUE);
SELECT add_retention_policy('strategy_regime_performance', INTERVAL '1 year', if_not_exists => TRUE);
SELECT add_retention_policy('strategy_correlation_matrix', INTERVAL '1 year', if_not_exists => TRUE);
SELECT add_retention_policy('strategy_daily_returns', INTERVAL '2 years', if_not_exists => TRUE);
SELECT add_retention_policy('strategy_optimization_history', INTERVAL '2 years', if_not_exists => TRUE);

-- Keep weight allocations for longer (5 years) for historical analysis
SELECT add_retention_policy('strategy_weight_allocations', INTERVAL '5 years', if_not_exists => TRUE);
