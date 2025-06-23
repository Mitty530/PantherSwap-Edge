// Database query functions for TimescaleDB hypertables and optimization
// Comprehensive CRUD operations, time-series queries, and analytical functions

use crate::utils::Result;
use crate::database::types::*;
use sqlx::{PgPool, Row};
use tracing::info;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Set up the complete database schema with all tables and hypertables
pub async fn setup_database_schema(pool: &PgPool) -> Result<()> {
    info!("Setting up database schema...");

    // Create reference tables first
    create_reference_tables(pool).await?;

    // Create Alpaca trading tables
    create_alpaca_tables(pool).await?;

    // Create time-series tables
    create_time_series_tables(pool).await?;

    // Convert to hypertables
    create_hypertables(pool).await?;

    // Create indexes for performance
    create_performance_indexes(pool).await?;

    // Set up compression policies
    setup_compression_policies(pool).await?;

    // Set up retention policies
    setup_retention_policies(pool).await?;

    info!("Database schema setup completed successfully");
    Ok(())
}

/// Create reference tables (regular PostgreSQL tables)
async fn create_reference_tables(pool: &PgPool) -> Result<()> {
    info!("Creating reference tables...");

    sqlx::query(crate::database::schema::CREATE_INSTRUMENTS_TABLE)
        .execute(pool)
        .await?;

    info!("Reference tables created successfully");
    Ok(())
}

/// Create Alpaca trading tables (regular PostgreSQL tables for order tracking)
async fn create_alpaca_tables(pool: &PgPool) -> Result<()> {
    info!("Creating Alpaca trading tables...");

    // Create Alpaca order tracking table
    sqlx::query(crate::database::schema::CREATE_ALPACA_ORDERS_TABLE)
        .execute(pool)
        .await?;

    // Create Alpaca account snapshots table
    sqlx::query(crate::database::schema::CREATE_ALPACA_ACCOUNT_SNAPSHOTS_TABLE)
        .execute(pool)
        .await?;

    // Create Alpaca positions table
    sqlx::query(crate::database::schema::CREATE_ALPACA_POSITIONS_TABLE)
        .execute(pool)
        .await?;

    // Create Alpaca performance metrics table
    sqlx::query(crate::database::schema::CREATE_ALPACA_PERFORMANCE_METRICS_TABLE)
        .execute(pool)
        .await?;

    // Create Alpaca execution stats table
    sqlx::query(crate::database::schema::CREATE_ALPACA_EXECUTION_STATS_TABLE)
        .execute(pool)
        .await?;

    // Create Alpaca market events table
    sqlx::query(crate::database::schema::CREATE_ALPACA_MARKET_EVENTS_TABLE)
        .execute(pool)
        .await?;

    info!("Alpaca trading tables created successfully");
    Ok(())
}

/// Create time-series tables (before converting to hypertables)
async fn create_time_series_tables(pool: &PgPool) -> Result<()> {
    info!("Creating time-series tables...");

    // Market data tables
    sqlx::query(crate::database::schema::CREATE_MARKET_TICKS_TABLE)
        .execute(pool)
        .await?;

    sqlx::query(crate::database::schema::CREATE_ORDER_BOOK_SNAPSHOTS_TABLE)
        .execute(pool)
        .await?;

    // AI and analytics tables
    sqlx::query(crate::database::schema::CREATE_AI_PREDICTIONS_TABLE)
        .execute(pool)
        .await?;

    sqlx::query(crate::database::schema::CREATE_MICROSTRUCTURE_ANALYSIS_TABLE)
        .execute(pool)
        .await?;

    // Trading tables
    sqlx::query(crate::database::schema::CREATE_TRADING_SIGNALS_TABLE)
        .execute(pool)
        .await?;

    sqlx::query(crate::database::schema::CREATE_TRADE_EXECUTIONS_TABLE)
        .execute(pool)
        .await?;

    // Risk management tables
    sqlx::query(crate::database::schema::CREATE_RISK_METRICS_TABLE)
        .execute(pool)
        .await?;

    info!("Time-series tables created successfully");
    Ok(())
}

/// Convert time-series tables to TimescaleDB hypertables
async fn create_hypertables(pool: &PgPool) -> Result<()> {
    info!("Converting tables to TimescaleDB hypertables...");

    // Create hypertables with optimized chunk intervals
    let hypertable_queries = [
        crate::database::schema::CREATE_MARKET_TICKS_HYPERTABLE,
        crate::database::schema::CREATE_AI_PREDICTIONS_HYPERTABLE,
        crate::database::schema::CREATE_MICROSTRUCTURE_ANALYSIS_HYPERTABLE,
        crate::database::schema::CREATE_TRADING_SIGNALS_HYPERTABLE,
        crate::database::schema::CREATE_ORDER_BOOK_SNAPSHOTS_HYPERTABLE,
        crate::database::schema::CREATE_RISK_METRICS_HYPERTABLE,
        crate::database::schema::CREATE_TRADE_EXECUTIONS_HYPERTABLE,
    ];

    for query in hypertable_queries {
        match sqlx::query(query).execute(pool).await {
            Ok(_) => {},
            Err(e) => {
                // Log warning but continue - table might already be a hypertable
                info!("Hypertable creation warning (may already exist): {}", e);
            }
        }
    }

    info!("Hypertables created successfully");
    Ok(())
}

/// Create performance indexes for all tables
async fn create_performance_indexes(pool: &PgPool) -> Result<()> {
    info!("Creating performance indexes...");

    let index_queries = [
        // Instruments indexes
        crate::database::schema::CREATE_INDEX_INSTRUMENTS_SYMBOL,
        crate::database::schema::CREATE_INDEX_INSTRUMENTS_TYPE,
        crate::database::schema::CREATE_INDEX_INSTRUMENTS_ACTIVE,

        // Market ticks indexes
        crate::database::schema::CREATE_INDEX_MARKET_TICKS_INSTRUMENT_TIME,
        crate::database::schema::CREATE_INDEX_MARKET_TICKS_PROVIDER,
        crate::database::schema::CREATE_INDEX_MARKET_TICKS_QUALITY,

        // AI predictions indexes
        crate::database::schema::CREATE_INDEX_AI_PREDICTIONS_INSTRUMENT_TIME,
        crate::database::schema::CREATE_INDEX_AI_PREDICTIONS_MODEL,
        crate::database::schema::CREATE_INDEX_AI_PREDICTIONS_CONFIDENCE,

        // Microstructure analysis indexes
        crate::database::schema::CREATE_INDEX_MICROSTRUCTURE_INSTRUMENT_TIME,
        crate::database::schema::CREATE_INDEX_MICROSTRUCTURE_REGIME,
        crate::database::schema::CREATE_INDEX_MICROSTRUCTURE_LIQUIDITY,

        // Trading signals indexes
        crate::database::schema::CREATE_INDEX_TRADING_SIGNALS_INSTRUMENT_TIME,
        crate::database::schema::CREATE_INDEX_TRADING_SIGNALS_STRATEGY,
        crate::database::schema::CREATE_INDEX_TRADING_SIGNALS_CONFIDENCE,

        // Order book indexes
        crate::database::schema::CREATE_INDEX_ORDER_BOOK_INSTRUMENT_TIME,
        crate::database::schema::CREATE_INDEX_ORDER_BOOK_VENUE_SIDE,
        crate::database::schema::CREATE_INDEX_ORDER_BOOK_PRICE,

        // Trade executions indexes
        crate::database::schema::CREATE_INDEX_TRADE_EXECUTIONS_INSTRUMENT_TIME,
        crate::database::schema::CREATE_INDEX_TRADE_EXECUTIONS_STRATEGY,
        crate::database::schema::CREATE_INDEX_TRADE_EXECUTIONS_PNL,

        // Alpaca trading indexes
        crate::database::schema::CREATE_INDEX_ALPACA_ORDERS_SYMBOL_CREATED,
        crate::database::schema::CREATE_INDEX_ALPACA_ORDERS_STATUS,
        crate::database::schema::CREATE_INDEX_ALPACA_ORDERS_ALPACA_ID,
        crate::database::schema::CREATE_INDEX_ALPACA_ORDERS_INTERNAL_ID,
        crate::database::schema::CREATE_INDEX_ALPACA_ORDERS_EXECUTION_TIME,
        crate::database::schema::CREATE_INDEX_ALPACA_POSITIONS_SYMBOL,
        crate::database::schema::CREATE_INDEX_ALPACA_POSITIONS_SIDE,
        crate::database::schema::CREATE_INDEX_ALPACA_ACCOUNT_SNAPSHOTS_ACCOUNT,
        crate::database::schema::CREATE_INDEX_ALPACA_MARKET_EVENTS_SYMBOL,
        crate::database::schema::CREATE_INDEX_ALPACA_MARKET_EVENTS_TYPE,
        crate::database::schema::CREATE_INDEX_ALPACA_PERFORMANCE_METRICS_CREATED,
        crate::database::schema::CREATE_INDEX_ALPACA_EXECUTION_STATS_CREATED,
    ];

    for query in index_queries {
        sqlx::query(query).execute(pool).await?;
    }

    info!("Performance indexes created successfully");
    Ok(())
}

/// Set up compression policies for historical data optimization
async fn setup_compression_policies(pool: &PgPool) -> Result<()> {
    info!("Setting up compression policies...");

    // Enable compression on hypertables
    let compression_enable_queries = [
        crate::database::schema::ENABLE_COMPRESSION_MARKET_TICKS,
        crate::database::schema::ENABLE_COMPRESSION_AI_PREDICTIONS,
        crate::database::schema::ENABLE_COMPRESSION_MICROSTRUCTURE,
        crate::database::schema::ENABLE_COMPRESSION_ORDER_BOOK,
    ];

    for query in compression_enable_queries {
        match sqlx::query(query).execute(pool).await {
            Ok(_) => {},
            Err(e) => {
                info!("Compression enable warning (may already be enabled): {}", e);
            }
        }
    }

    // Add compression policies
    let compression_policy_queries = [
        crate::database::schema::ADD_COMPRESSION_POLICY_MARKET_TICKS,
        crate::database::schema::ADD_COMPRESSION_POLICY_AI_PREDICTIONS,
        crate::database::schema::ADD_COMPRESSION_POLICY_MICROSTRUCTURE,
        crate::database::schema::ADD_COMPRESSION_POLICY_ORDER_BOOK,
    ];

    for query in compression_policy_queries {
        match sqlx::query(query).execute(pool).await {
            Ok(_) => {},
            Err(e) => {
                info!("Compression policy warning (may already exist): {}", e);
            }
        }
    }

    info!("Compression policies set up successfully");
    Ok(())
}

/// Set up retention policies for automatic data cleanup
async fn setup_retention_policies(pool: &PgPool) -> Result<()> {
    info!("Setting up retention policies...");

    let retention_queries = [
        crate::database::schema::ADD_RETENTION_POLICY_MARKET_TICKS,
        crate::database::schema::ADD_RETENTION_POLICY_ORDER_BOOK,
    ];

    for query in retention_queries {
        match sqlx::query(query).execute(pool).await {
            Ok(_) => {},
            Err(e) => {
                info!("Retention policy warning (may already exist): {}", e);
            }
        }
    }

    info!("Retention policies set up successfully");
    Ok(())
}

// ============================================================================
// CRUD OPERATIONS FOR INSTRUMENTS (Reference Data)
// ============================================================================

/// Insert a new instrument
pub async fn insert_instrument(pool: &PgPool, instrument: &Instrument) -> Result<Uuid> {
    let row = sqlx::query(
        r#"
        INSERT INTO instruments
        (symbol, name, instrument_type, base_currency, quote_currency,
         tick_size, lot_size, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id
        "#
    )
    .bind(&instrument.symbol)
    .bind(&instrument.name)
    .bind(&instrument.instrument_type)
    .bind(&instrument.base_currency)
    .bind(&instrument.quote_currency)
    .bind(instrument.tick_size)
    .bind(instrument.lot_size)
    .bind(instrument.is_active)
    .fetch_one(pool)
    .await?;

    let id: Uuid = row.get("id");
    Ok(id)
}

/// Get instrument by ID
pub async fn get_instrument_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Instrument>> {
    let instrument = sqlx::query_as!(
        Instrument,
        "SELECT * FROM instruments WHERE id = $1",
        id
    )
    .fetch_optional(pool)
    .await?;

    Ok(instrument)
}

/// Get instrument by symbol
pub async fn get_instrument_by_symbol(pool: &PgPool, symbol: &str) -> Result<Option<Instrument>> {
    let instrument = sqlx::query_as!(
        Instrument,
        "SELECT * FROM instruments WHERE symbol = $1",
        symbol
    )
    .fetch_optional(pool)
    .await?;

    Ok(instrument)
}

/// Get all active instruments
pub async fn get_active_instruments(pool: &PgPool) -> Result<Vec<Instrument>> {
    let instruments = sqlx::query_as!(
        Instrument,
        "SELECT * FROM instruments WHERE is_active = true ORDER BY symbol"
    )
    .fetch_all(pool)
    .await?;

    Ok(instruments)
}

/// Update instrument
pub async fn update_instrument(pool: &PgPool, id: Uuid, instrument: &Instrument) -> Result<()> {
    sqlx::query!(
        r#"
        UPDATE instruments
        SET name = $2, instrument_type = $3, base_currency = $4, quote_currency = $5,
            tick_size = $6, lot_size = $7, is_active = $8, updated_at = NOW()
        WHERE id = $1
        "#,
        id,
        instrument.name,
        instrument.instrument_type,
        instrument.base_currency,
        instrument.quote_currency,
        instrument.tick_size,
        instrument.lot_size,
        instrument.is_active
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Delete instrument (soft delete by setting inactive)
pub async fn deactivate_instrument(pool: &PgPool, id: Uuid) -> Result<()> {
    sqlx::query!(
        "UPDATE instruments SET is_active = false, updated_at = NOW() WHERE id = $1",
        id
    )
    .execute(pool)
    .await?;

    Ok(())
}

// ============================================================================
// CRUD OPERATIONS FOR MARKET DATA (Time-Series)
// ============================================================================

/// Insert market tick data
pub async fn insert_market_tick(pool: &PgPool, tick: &MarketTick) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO market_ticks
        (timestamp, instrument_id, provider, bid_price, ask_price, bid_size, ask_size,
         last_price, volume, spread, data_quality_score, raw_data)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#,
        tick.timestamp,
        tick.instrument_id,
        tick.provider,
        tick.bid_price,
        tick.ask_price,
        tick.bid_size,
        tick.ask_size,
        tick.last_price,
        tick.volume,
        tick.spread,
        tick.data_quality_score,
        tick.raw_data
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Batch insert market ticks for high-frequency data
pub async fn batch_insert_market_ticks(pool: &PgPool, ticks: &[MarketTick]) -> Result<u64> {
    if ticks.is_empty() {
        return Ok(0);
    }

    let mut query_builder = sqlx::QueryBuilder::new(
        "INSERT INTO market_ticks (timestamp, instrument_id, provider, bid_price, ask_price, bid_size, ask_size, last_price, volume, spread, data_quality_score, raw_data) "
    );

    query_builder.push_values(ticks, |mut b, tick| {
        b.push_bind(tick.timestamp)
         .push_bind(tick.instrument_id)
         .push_bind(&tick.provider)
         .push_bind(tick.bid_price)
         .push_bind(tick.ask_price)
         .push_bind(tick.bid_size)
         .push_bind(tick.ask_size)
         .push_bind(tick.last_price)
         .push_bind(tick.volume)
         .push_bind(tick.spread)
         .push_bind(tick.data_quality_score)
         .push_bind(&tick.raw_data);
    });

    let result = query_builder.build().execute(pool).await?;
    Ok(result.rows_affected())
}

/// Get latest market tick for an instrument
pub async fn get_latest_market_tick(pool: &PgPool, instrument_id: Uuid) -> Result<Option<MarketTick>> {
    let tick = sqlx::query_as!(
        MarketTick,
        r#"
        SELECT * FROM market_ticks
        WHERE instrument_id = $1
        ORDER BY timestamp DESC
        LIMIT 1
        "#,
        instrument_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(tick)
}

/// Get market ticks within time range
pub async fn get_market_ticks_range(
    pool: &PgPool,
    instrument_id: Uuid,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    limit: Option<i64>
) -> Result<Vec<MarketTick>> {
    let limit = limit.unwrap_or(1000);

    let ticks = sqlx::query_as!(
        MarketTick,
        r#"
        SELECT * FROM market_ticks
        WHERE instrument_id = $1
        AND timestamp >= $2
        AND timestamp <= $3
        ORDER BY timestamp DESC
        LIMIT $4
        "#,
        instrument_id,
        start_time,
        end_time,
        limit
    )
    .fetch_all(pool)
    .await?;

    Ok(ticks)
}

/// Get market ticks with quality filter
pub async fn get_high_quality_market_ticks(
    pool: &PgPool,
    instrument_id: Uuid,
    min_quality: f64,
    hours_back: i32
) -> Result<Vec<MarketTick>> {
    let ticks = sqlx::query_as!(
        MarketTick,
        r#"
        SELECT * FROM market_ticks
        WHERE instrument_id = $1
        AND data_quality_score >= $2
        AND timestamp >= NOW() - INTERVAL '%s hours'
        ORDER BY timestamp DESC
        LIMIT 1000
        "#,
        instrument_id,
        min_quality,
        hours_back.to_string()
    )
    .fetch_all(pool)
    .await?;

    Ok(ticks)
}

// ============================================================================
// CRUD OPERATIONS FOR AI PREDICTIONS
// ============================================================================

/// Insert AI prediction
pub async fn insert_ai_prediction(pool: &PgPool, prediction: &AIPrediction) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO ai_predictions
        (timestamp, instrument_id, model_type, model_version, prediction_horizon_minutes,
         predicted_price, predicted_volatility, confidence_score, prediction_intervals,
         feature_importance)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
        prediction.timestamp,
        prediction.instrument_id,
        prediction.model_type,
        prediction.model_version,
        prediction.prediction_horizon_minutes,
        prediction.predicted_price,
        prediction.predicted_volatility,
        prediction.confidence_score,
        prediction.prediction_intervals,
        prediction.feature_importance
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Get latest AI predictions for an instrument
pub async fn get_latest_ai_predictions(
    pool: &PgPool,
    instrument_id: Uuid,
    model_type: Option<&str>,
    limit: Option<i64>
) -> Result<Vec<AIPrediction>> {
    let limit = limit.unwrap_or(10);

    let predictions = match model_type {
        Some(model) => {
            sqlx::query_as!(
                AIPrediction,
                r#"
                SELECT * FROM ai_predictions
                WHERE instrument_id = $1 AND model_type = $2
                ORDER BY timestamp DESC
                LIMIT $3
                "#,
                instrument_id,
                model,
                limit
            )
            .fetch_all(pool)
            .await?
        }
        None => {
            sqlx::query_as!(
                AIPrediction,
                r#"
                SELECT * FROM ai_predictions
                WHERE instrument_id = $1
                ORDER BY timestamp DESC
                LIMIT $2
                "#,
                instrument_id,
                limit
            )
            .fetch_all(pool)
            .await?
        }
    };

    Ok(predictions)
}

/// Get high-confidence AI predictions
pub async fn get_high_confidence_predictions(
    pool: &PgPool,
    instrument_id: Uuid,
    min_confidence: f64,
    hours_back: i32
) -> Result<Vec<AIPrediction>> {
    let predictions = sqlx::query_as!(
        AIPrediction,
        r#"
        SELECT * FROM ai_predictions
        WHERE instrument_id = $1
        AND confidence_score >= $2
        AND timestamp >= NOW() - INTERVAL '%s hours'
        ORDER BY confidence_score DESC, timestamp DESC
        LIMIT 50
        "#,
        instrument_id,
        min_confidence,
        hours_back.to_string()
    )
    .fetch_all(pool)
    .await?;

    Ok(predictions)
}

// ============================================================================
// CRUD OPERATIONS FOR TRADING SIGNALS
// ============================================================================

/// Insert trading signal
pub async fn insert_trading_signal(pool: &PgPool, signal: &TradingSignal) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO trading_signals
        (timestamp, instrument_id, strategy_type, signal_type, confidence_score,
         target_price, stop_loss, take_profit, position_size, risk_score,
         time_horizon, metadata)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#,
        signal.timestamp,
        signal.instrument_id,
        signal.strategy_type,
        signal.signal_type,
        signal.confidence_score,
        signal.target_price,
        signal.stop_loss,
        signal.take_profit,
        signal.position_size,
        signal.risk_score,
        signal.time_horizon.map(|d| d.to_std().ok()).flatten(),
        signal.metadata
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Get active trading signals
pub async fn get_active_trading_signals(
    pool: &PgPool,
    instrument_id: Option<Uuid>,
    strategy_type: Option<&str>,
    min_confidence: Option<f64>
) -> Result<Vec<TradingSignal>> {
    let mut query = "SELECT * FROM trading_signals WHERE 1=1".to_string();
    let mut params: Vec<Box<dyn sqlx::Encode<sqlx::Postgres> + Send + Sync>> = Vec::new();
    let mut param_count = 1;

    if let Some(id) = instrument_id {
        query.push_str(&format!(" AND instrument_id = ${}", param_count));
        params.push(Box::new(id));
        param_count += 1;
    }

    if let Some(strategy) = strategy_type {
        query.push_str(&format!(" AND strategy_type = ${}", param_count));
        params.push(Box::new(strategy.to_string()));
        param_count += 1;
    }

    if let Some(confidence) = min_confidence {
        query.push_str(&format!(" AND confidence_score >= ${}", param_count));
        params.push(Box::new(confidence));
        param_count += 1;
    }

    query.push_str(" AND timestamp >= NOW() - INTERVAL '24 hours' ORDER BY timestamp DESC LIMIT 100");

    // For now, use a simpler approach with fixed parameters
    let signals = match (instrument_id, strategy_type, min_confidence) {
        (Some(id), Some(strategy), Some(confidence)) => {
            sqlx::query_as!(
                TradingSignal,
                r#"
                SELECT timestamp, instrument_id, strategy_type, signal_type, confidence_score,
                       target_price, stop_loss, take_profit, position_size, risk_score,
                       time_horizon, metadata, created_at
                FROM trading_signals
                WHERE instrument_id = $1 AND strategy_type = $2 AND confidence_score >= $3
                AND timestamp >= NOW() - INTERVAL '24 hours'
                ORDER BY timestamp DESC LIMIT 100
                "#,
                id,
                strategy,
                confidence
            )
            .fetch_all(pool)
            .await?
        }
        (Some(id), None, None) => {
            sqlx::query_as!(
                TradingSignal,
                r#"
                SELECT timestamp, instrument_id, strategy_type, signal_type, confidence_score,
                       target_price, stop_loss, take_profit, position_size, risk_score,
                       time_horizon, metadata, created_at
                FROM trading_signals
                WHERE instrument_id = $1
                AND timestamp >= NOW() - INTERVAL '24 hours'
                ORDER BY timestamp DESC LIMIT 100
                "#,
                id
            )
            .fetch_all(pool)
            .await?
        }
        _ => {
            sqlx::query_as!(
                TradingSignal,
                r#"
                SELECT timestamp, instrument_id, strategy_type, signal_type, confidence_score,
                       target_price, stop_loss, take_profit, position_size, risk_score,
                       time_horizon, metadata, created_at
                FROM trading_signals
                WHERE timestamp >= NOW() - INTERVAL '24 hours'
                ORDER BY timestamp DESC LIMIT 100
                "#
            )
            .fetch_all(pool)
            .await?
        }
    };

    Ok(signals)
}

// ============================================================================
// TIME-SERIES ANALYTICAL QUERIES
// ============================================================================

/// Market data aggregation result
#[derive(Debug, sqlx::FromRow)]
pub struct MarketDataAggregation {
    pub time_bucket: DateTime<Utc>,
    pub instrument_id: Uuid,
    pub open_price: Option<f64>,
    pub high_price: Option<f64>,
    pub low_price: Option<f64>,
    pub close_price: Option<f64>,
    pub avg_price: Option<f64>,
    pub total_volume: Option<f64>,
    pub avg_spread: Option<f64>,
    pub tick_count: Option<i64>,
}

/// Get OHLCV data with time bucketing
pub async fn get_ohlcv_data(
    pool: &PgPool,
    instrument_id: Uuid,
    bucket_size: &str, // e.g., "1 minute", "5 minutes", "1 hour"
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>
) -> Result<Vec<MarketDataAggregation>> {
    let data = sqlx::query_as!(
        MarketDataAggregation,
        r#"
        SELECT
            time_bucket($1, timestamp) as time_bucket,
            instrument_id,
            first(last_price, timestamp) as open_price,
            max(last_price) as high_price,
            min(last_price) as low_price,
            last(last_price, timestamp) as close_price,
            avg(last_price) as avg_price,
            sum(volume) as total_volume,
            avg(spread) as avg_spread,
            count(*) as tick_count
        FROM market_ticks
        WHERE instrument_id = $2
        AND timestamp >= $3
        AND timestamp <= $4
        AND last_price IS NOT NULL
        GROUP BY time_bucket, instrument_id
        ORDER BY time_bucket
        "#,
        bucket_size,
        instrument_id,
        start_time,
        end_time
    )
    .fetch_all(pool)
    .await?;

    Ok(data)
}

/// Volatility calculation result
#[derive(Debug, sqlx::FromRow)]
pub struct VolatilityMetrics {
    pub time_bucket: DateTime<Utc>,
    pub instrument_id: Uuid,
    pub price_volatility: Option<f64>,
    pub spread_volatility: Option<f64>,
    pub volume_volatility: Option<f64>,
    pub returns_std: Option<f64>,
}

/// Calculate volatility metrics over time
pub async fn get_volatility_metrics(
    pool: &PgPool,
    instrument_id: Uuid,
    bucket_size: &str,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>
) -> Result<Vec<VolatilityMetrics>> {
    let metrics = sqlx::query_as!(
        VolatilityMetrics,
        r#"
        SELECT
            time_bucket($1, timestamp) as time_bucket,
            instrument_id,
            stddev(last_price) as price_volatility,
            stddev(spread) as spread_volatility,
            stddev(volume) as volume_volatility,
            stddev(
                (last_price - lag(last_price) OVER (ORDER BY timestamp)) /
                lag(last_price) OVER (ORDER BY timestamp)
            ) as returns_std
        FROM market_ticks
        WHERE instrument_id = $2
        AND timestamp >= $3
        AND timestamp <= $4
        AND last_price IS NOT NULL
        GROUP BY time_bucket, instrument_id
        ORDER BY time_bucket
        "#,
        bucket_size,
        instrument_id,
        start_time,
        end_time
    )
    .fetch_all(pool)
    .await?;

    Ok(metrics)
}

/// Liquidity metrics result
#[derive(Debug, sqlx::FromRow)]
pub struct LiquidityMetrics {
    pub time_bucket: DateTime<Utc>,
    pub instrument_id: Uuid,
    pub avg_bid_ask_spread: Option<f64>,
    pub avg_market_depth: Option<f64>,
    pub total_volume: Option<f64>,
    pub tick_frequency: Option<f64>,
}

/// Calculate liquidity metrics
pub async fn get_liquidity_metrics(
    pool: &PgPool,
    instrument_id: Uuid,
    bucket_size: &str,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>
) -> Result<Vec<LiquidityMetrics>> {
    let metrics = sqlx::query_as!(
        LiquidityMetrics,
        r#"
        SELECT
            time_bucket($1, timestamp) as time_bucket,
            instrument_id,
            avg(spread) as avg_bid_ask_spread,
            avg(bid_size + ask_size) as avg_market_depth,
            sum(volume) as total_volume,
            count(*)::float / extract(epoch from $1::interval) as tick_frequency
        FROM market_ticks
        WHERE instrument_id = $2
        AND timestamp >= $3
        AND timestamp <= $4
        GROUP BY time_bucket, instrument_id
        ORDER BY time_bucket
        "#,
        bucket_size,
        instrument_id,
        start_time,
        end_time
    )
    .fetch_all(pool)
    .await?;

    Ok(metrics)
}

// ============================================================================
// PERFORMANCE AND RISK ANALYTICS
// ============================================================================

/// Trading performance metrics
#[derive(Debug, sqlx::FromRow)]
pub struct TradingPerformance {
    pub strategy_type: String,
    pub total_signals: Option<i64>,
    pub avg_confidence: Option<f64>,
    pub high_confidence_signals: Option<i64>,
    pub avg_risk_score: Option<f64>,
}

/// Get trading strategy performance
pub async fn get_strategy_performance(
    pool: &PgPool,
    instrument_id: Option<Uuid>,
    days_back: i32
) -> Result<Vec<TradingPerformance>> {
    let performance = match instrument_id {
        Some(id) => {
            sqlx::query_as!(
                TradingPerformance,
                r#"
                SELECT
                    strategy_type,
                    count(*) as total_signals,
                    avg(confidence_score) as avg_confidence,
                    count(*) FILTER (WHERE confidence_score >= 0.8) as high_confidence_signals,
                    avg(risk_score) as avg_risk_score
                FROM trading_signals
                WHERE instrument_id = $1
                AND timestamp >= NOW() - INTERVAL '%s days'
                GROUP BY strategy_type
                ORDER BY total_signals DESC
                "#,
                id,
                days_back.to_string()
            )
            .fetch_all(pool)
            .await?
        }
        None => {
            sqlx::query_as!(
                TradingPerformance,
                r#"
                SELECT
                    strategy_type,
                    count(*) as total_signals,
                    avg(confidence_score) as avg_confidence,
                    count(*) FILTER (WHERE confidence_score >= 0.8) as high_confidence_signals,
                    avg(risk_score) as avg_risk_score
                FROM trading_signals
                WHERE timestamp >= NOW() - INTERVAL '%s days'
                GROUP BY strategy_type
                ORDER BY total_signals DESC
                "#,
                days_back.to_string()
            )
            .fetch_all(pool)
            .await?
        }
    };

    Ok(performance)
}

/// AI model performance metrics
#[derive(Debug, sqlx::FromRow)]
pub struct AIModelPerformance {
    pub model_type: String,
    pub model_version: String,
    pub total_predictions: Option<i64>,
    pub avg_confidence: Option<f64>,
    pub high_confidence_predictions: Option<i64>,
    pub prediction_horizons: Vec<Option<i32>>,
}

/// Get AI model performance statistics
pub async fn get_ai_model_performance(
    pool: &PgPool,
    instrument_id: Option<Uuid>,
    days_back: i32
) -> Result<Vec<AIModelPerformance>> {
    let performance = match instrument_id {
        Some(id) => {
            sqlx::query_as!(
                AIModelPerformance,
                r#"
                SELECT
                    model_type,
                    model_version,
                    count(*) as total_predictions,
                    avg(confidence_score) as avg_confidence,
                    count(*) FILTER (WHERE confidence_score >= 0.8) as high_confidence_predictions,
                    array_agg(DISTINCT prediction_horizon_minutes) as prediction_horizons
                FROM ai_predictions
                WHERE instrument_id = $1
                AND timestamp >= NOW() - INTERVAL '%s days'
                GROUP BY model_type, model_version
                ORDER BY total_predictions DESC
                "#,
                id,
                days_back.to_string()
            )
            .fetch_all(pool)
            .await?
        }
        None => {
            sqlx::query_as!(
                AIModelPerformance,
                r#"
                SELECT
                    model_type,
                    model_version,
                    count(*) as total_predictions,
                    avg(confidence_score) as avg_confidence,
                    count(*) FILTER (WHERE confidence_score >= 0.8) as high_confidence_predictions,
                    array_agg(DISTINCT prediction_horizon_minutes) as prediction_horizons
                FROM ai_predictions
                WHERE timestamp >= NOW() - INTERVAL '%s days'
                GROUP BY model_type, model_version
                ORDER BY total_predictions DESC
                "#,
                days_back.to_string()
            )
            .fetch_all(pool)
            .await?
        }
    };

    Ok(performance)
}

/// Data quality metrics
#[derive(Debug, sqlx::FromRow)]
pub struct DataQualityMetrics {
    pub time_bucket: DateTime<Utc>,
    pub provider: String,
    pub avg_quality_score: Option<f64>,
    pub min_quality_score: Option<f64>,
    pub max_quality_score: Option<f64>,
    pub total_ticks: Option<i64>,
    pub high_quality_ticks: Option<i64>,
}

/// Get data quality metrics by provider
pub async fn get_data_quality_metrics(
    pool: &PgPool,
    instrument_id: Uuid,
    bucket_size: &str,
    hours_back: i32
) -> Result<Vec<DataQualityMetrics>> {
    let metrics = sqlx::query_as!(
        DataQualityMetrics,
        r#"
        SELECT
            time_bucket($1, timestamp) as time_bucket,
            provider,
            avg(data_quality_score) as avg_quality_score,
            min(data_quality_score) as min_quality_score,
            max(data_quality_score) as max_quality_score,
            count(*) as total_ticks,
            count(*) FILTER (WHERE data_quality_score >= 0.8) as high_quality_ticks
        FROM market_ticks
        WHERE instrument_id = $2
        AND timestamp >= NOW() - INTERVAL '%s hours'
        GROUP BY time_bucket, provider
        ORDER BY time_bucket, provider
        "#,
        bucket_size,
        instrument_id,
        hours_back.to_string()
    )
    .fetch_all(pool)
    .await?;

    Ok(metrics)
}

// ============================================================================
// UTILITY AND MONITORING FUNCTIONS
// ============================================================================

/// Database statistics
#[derive(Debug, sqlx::FromRow)]
pub struct DatabaseStats {
    pub table_name: String,
    pub row_count: Option<i64>,
    pub table_size: Option<String>,
    pub index_size: Option<String>,
}

/// Get database table statistics
pub async fn get_database_stats(pool: &PgPool) -> Result<Vec<DatabaseStats>> {
    let stats = sqlx::query_as!(
        DatabaseStats,
        r#"
        SELECT
            schemaname||'.'||tablename as table_name,
            n_tup_ins + n_tup_upd as row_count,
            pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as table_size,
            pg_size_pretty(pg_indexes_size(schemaname||'.'||tablename)) as index_size
        FROM pg_stat_user_tables
        WHERE schemaname = 'public'
        ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(stats)
}

/// Hypertable information
#[derive(Debug, sqlx::FromRow)]
pub struct HypertableInfo {
    pub table_name: String,
    pub chunk_time_interval: Option<String>,
    pub compression_enabled: Option<bool>,
    pub total_chunks: Option<i64>,
    pub compressed_chunks: Option<i64>,
}

/// Get TimescaleDB hypertable information
pub async fn get_hypertable_info(pool: &PgPool) -> Result<Vec<HypertableInfo>> {
    let info = sqlx::query_as!(
        HypertableInfo,
        r#"
        SELECT
            h.table_name,
            h.chunk_time_interval::text as chunk_time_interval,
            h.compression_enabled,
            (SELECT count(*) FROM timescaledb_information.chunks c
             WHERE c.hypertable_name = h.table_name) as total_chunks,
            (SELECT count(*) FROM timescaledb_information.chunks c
             WHERE c.hypertable_name = h.table_name AND c.is_compressed = true) as compressed_chunks
        FROM timescaledb_information.hypertables h
        WHERE h.schema_name = 'public'
        ORDER BY h.table_name
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(info)
}

/// Recent activity summary
#[derive(Debug, sqlx::FromRow)]
pub struct RecentActivity {
    pub table_name: String,
    pub recent_records: Option<i64>,
    pub last_insert: Option<DateTime<Utc>>,
    pub avg_records_per_hour: Option<f64>,
}

/// Get recent database activity
pub async fn get_recent_activity(pool: &PgPool, hours_back: i32) -> Result<Vec<RecentActivity>> {
    // This is a simplified version - in production you'd want more sophisticated monitoring
    let activity = sqlx::query_as!(
        RecentActivity,
        r#"
        SELECT
            'market_ticks' as table_name,
            count(*) as recent_records,
            max(timestamp) as last_insert,
            count(*)::float / $1 as avg_records_per_hour
        FROM market_ticks
        WHERE timestamp >= NOW() - INTERVAL '%s hours'

        UNION ALL

        SELECT
            'trading_signals' as table_name,
            count(*) as recent_records,
            max(timestamp) as last_insert,
            count(*)::float / $1 as avg_records_per_hour
        FROM trading_signals
        WHERE timestamp >= NOW() - INTERVAL '%s hours'

        UNION ALL

        SELECT
            'ai_predictions' as table_name,
            count(*) as recent_records,
            max(timestamp) as last_insert,
            count(*)::float / $1 as avg_records_per_hour
        FROM ai_predictions
        WHERE timestamp >= NOW() - INTERVAL '%s hours'

        ORDER BY recent_records DESC
        "#,
        hours_back as f64,
        hours_back.to_string(),
        hours_back.to_string(),
        hours_back.to_string()
    )
    .fetch_all(pool)
    .await?;

    Ok(activity)
}

/// Clean up old data based on retention policies
pub async fn cleanup_old_data(pool: &PgPool, days_to_keep: i32) -> Result<u64> {
    let mut total_deleted = 0u64;

    // Clean up old market ticks (keep only high-quality recent data)
    let deleted = sqlx::query!(
        r#"
        DELETE FROM market_ticks
        WHERE timestamp < NOW() - INTERVAL '%s days'
        AND data_quality_score < 0.7
        "#,
        days_to_keep.to_string()
    )
    .execute(pool)
    .await?;

    total_deleted += deleted.rows_affected();

    // Clean up old low-confidence predictions
    let deleted = sqlx::query!(
        r#"
        DELETE FROM ai_predictions
        WHERE timestamp < NOW() - INTERVAL '%s days'
        AND confidence_score < 0.5
        "#,
        days_to_keep.to_string()
    )
    .execute(pool)
    .await?;

    total_deleted += deleted.rows_affected();

    Ok(total_deleted)
}

/// Database health check with detailed metrics
pub async fn comprehensive_health_check(pool: &PgPool) -> Result<serde_json::Value> {
    // Basic connectivity
    let connection_ok = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(pool)
        .await
        .is_ok();

    // Check TimescaleDB extension
    let timescale_ok = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM pg_extension WHERE extname = 'timescaledb')"
    )
    .fetch_one(pool)
    .await
    .unwrap_or(false);

    // Get table counts
    let table_counts = sqlx::query!(
        r#"
        SELECT
            'instruments' as table_name, count(*) as count FROM instruments
        UNION ALL
        SELECT
            'market_ticks' as table_name, count(*) as count FROM market_ticks
        UNION ALL
        SELECT
            'trading_signals' as table_name, count(*) as count FROM trading_signals
        UNION ALL
        SELECT
            'ai_predictions' as table_name, count(*) as count FROM ai_predictions
        "#
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let health_data = serde_json::json!({
        "connection_ok": connection_ok,
        "timescale_extension": timescale_ok,
        "table_counts": table_counts.into_iter().map(|row| {
            serde_json::json!({
                "table": row.table_name,
                "count": row.count
            })
        }).collect::<Vec<_>>(),
        "timestamp": Utc::now()
    });

    Ok(health_data)
}

// ============================================================================
// QUERY MANAGER - Organized interface for all database operations
// ============================================================================

/// Centralized query manager for database operations
pub struct QueryManager {
    pool: PgPool,
}

impl QueryManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Instrument operations
    pub async fn insert_instrument(&self, instrument: &Instrument) -> Result<Uuid> {
        insert_instrument(&self.pool, instrument).await
    }

    pub async fn get_instrument_by_id(&self, id: Uuid) -> Result<Option<Instrument>> {
        get_instrument_by_id(&self.pool, id).await
    }

    pub async fn get_instrument_by_symbol(&self, symbol: &str) -> Result<Option<Instrument>> {
        get_instrument_by_symbol(&self.pool, symbol).await
    }

    pub async fn get_active_instruments(&self) -> Result<Vec<Instrument>> {
        get_active_instruments(&self.pool).await
    }

    // Market data operations
    pub async fn insert_market_tick(&self, tick: &MarketTick) -> Result<()> {
        insert_market_tick(&self.pool, tick).await
    }

    pub async fn batch_insert_market_ticks(&self, ticks: &[MarketTick]) -> Result<u64> {
        batch_insert_market_ticks(&self.pool, ticks).await
    }

    pub async fn get_latest_market_tick(&self, instrument_id: Uuid) -> Result<Option<MarketTick>> {
        get_latest_market_tick(&self.pool, instrument_id).await
    }

    pub async fn get_market_ticks_range(
        &self,
        instrument_id: Uuid,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        limit: Option<i64>
    ) -> Result<Vec<MarketTick>> {
        get_market_ticks_range(&self.pool, instrument_id, start_time, end_time, limit).await
    }

    // AI predictions
    pub async fn insert_ai_prediction(&self, prediction: &AIPrediction) -> Result<()> {
        insert_ai_prediction(&self.pool, prediction).await
    }

    pub async fn get_latest_ai_predictions(
        &self,
        instrument_id: Uuid,
        model_type: Option<&str>,
        limit: Option<i64>
    ) -> Result<Vec<AIPrediction>> {
        get_latest_ai_predictions(&self.pool, instrument_id, model_type, limit).await
    }

    // Trading signals
    pub async fn insert_trading_signal(&self, signal: &TradingSignal) -> Result<()> {
        insert_trading_signal(&self.pool, signal).await
    }

    pub async fn get_active_trading_signals(
        &self,
        instrument_id: Option<Uuid>,
        strategy_type: Option<&str>,
        min_confidence: Option<f64>
    ) -> Result<Vec<TradingSignal>> {
        get_active_trading_signals(&self.pool, instrument_id, strategy_type, min_confidence).await
    }

    // Analytics
    pub async fn get_ohlcv_data(
        &self,
        instrument_id: Uuid,
        bucket_size: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>
    ) -> Result<Vec<MarketDataAggregation>> {
        get_ohlcv_data(&self.pool, instrument_id, bucket_size, start_time, end_time).await
    }

    pub async fn get_volatility_metrics(
        &self,
        instrument_id: Uuid,
        bucket_size: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>
    ) -> Result<Vec<VolatilityMetrics>> {
        get_volatility_metrics(&self.pool, instrument_id, bucket_size, start_time, end_time).await
    }

    pub async fn get_strategy_performance(
        &self,
        instrument_id: Option<Uuid>,
        days_back: i32
    ) -> Result<Vec<TradingPerformance>> {
        get_strategy_performance(&self.pool, instrument_id, days_back).await
    }

    // Monitoring
    pub async fn get_database_stats(&self) -> Result<Vec<DatabaseStats>> {
        get_database_stats(&self.pool).await
    }

    pub async fn get_hypertable_info(&self) -> Result<Vec<HypertableInfo>> {
        get_hypertable_info(&self.pool).await
    }

    pub async fn comprehensive_health_check(&self) -> Result<serde_json::Value> {
        comprehensive_health_check(&self.pool).await
    }

    pub async fn cleanup_old_data(&self, days_to_keep: i32) -> Result<u64> {
        cleanup_old_data(&self.pool, days_to_keep).await
    }

    // ============================================================================
    // ALPACA TRADING OPERATIONS
    // ============================================================================

    /// Insert Alpaca order record
    pub async fn insert_alpaca_order(&self, order: &crate::market_data::alpaca::AlpacaOrderInfo) -> Result<()> {
        insert_alpaca_order(&self.pool, order).await
    }

    /// Update Alpaca order status
    pub async fn update_alpaca_order_status(&self, alpaca_order_id: &str, status: &str, filled_qty: Option<f64>, filled_avg_price: Option<f64>) -> Result<()> {
        update_alpaca_order_status(&self.pool, alpaca_order_id, status, filled_qty, filled_avg_price).await
    }

    /// Get Alpaca orders by status
    pub async fn get_alpaca_orders_by_status(&self, status: &str, limit: Option<i64>) -> Result<Vec<crate::market_data::alpaca::AlpacaOrderInfo>> {
        get_alpaca_orders_by_status(&self.pool, status, limit).await
    }

    /// Insert Alpaca position snapshot
    pub async fn insert_alpaca_position(&self, position: &crate::market_data::alpaca::AlpacaPosition) -> Result<()> {
        insert_alpaca_position(&self.pool, position).await
    }

    /// Get current Alpaca positions
    pub async fn get_current_alpaca_positions(&self) -> Result<Vec<crate::market_data::alpaca::AlpacaPosition>> {
        get_current_alpaca_positions(&self.pool).await
    }

    /// Insert Alpaca execution statistics
    pub async fn insert_alpaca_execution_stats(&self, stats: &crate::market_data::alpaca::AlpacaExecutionStats) -> Result<()> {
        insert_alpaca_execution_stats(&self.pool, stats).await
    }

    /// Get latest Alpaca execution statistics
    pub async fn get_latest_alpaca_execution_stats(&self) -> Result<Option<crate::market_data::alpaca::AlpacaExecutionStats>> {
        get_latest_alpaca_execution_stats(&self.pool).await
    }
}

// ============================================================================
// ALPACA TRADING CRUD OPERATIONS
// ============================================================================

/// Insert Alpaca order record
pub async fn insert_alpaca_order(pool: &PgPool, order: &crate::market_data::alpaca::AlpacaOrderInfo) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO alpaca_orders
        (alpaca_order_id, internal_order_id, symbol, side, quantity, order_type, status,
         submitted_at, filled_at, filled_qty, filled_avg_price, time_in_force, limit_price, stop_price)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        "#,
        order.alpaca_order_id,
        order.internal_order_id,
        order.symbol,
        order.side,
        order.quantity,
        order.order_type,
        order.status,
        order.submitted_at,
        order.filled_at,
        order.filled_qty,
        order.filled_avg_price,
        order.time_in_force,
        order.limit_price,
        order.stop_price
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Update Alpaca order status
pub async fn update_alpaca_order_status(
    pool: &PgPool,
    alpaca_order_id: &str,
    status: &str,
    filled_qty: Option<f64>,
    filled_avg_price: Option<f64>
) -> Result<()> {
    sqlx::query!(
        r#"
        UPDATE alpaca_orders
        SET status = $2, filled_qty = $3, filled_avg_price = $4, updated_at = NOW()
        WHERE alpaca_order_id = $1
        "#,
        alpaca_order_id,
        status,
        filled_qty,
        filled_avg_price
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Get Alpaca orders by status
pub async fn get_alpaca_orders_by_status(
    pool: &PgPool,
    status: &str,
    limit: Option<i64>
) -> Result<Vec<crate::market_data::alpaca::AlpacaOrderInfo>> {
    let limit = limit.unwrap_or(100);

    let orders = sqlx::query!(
        r#"
        SELECT alpaca_order_id, internal_order_id, symbol, side, quantity, order_type, status,
               submitted_at, filled_at, filled_qty, filled_avg_price, time_in_force, limit_price, stop_price
        FROM alpaca_orders
        WHERE status = $1
        ORDER BY created_at DESC
        LIMIT $2
        "#,
        status,
        limit
    )
    .fetch_all(pool)
    .await?;

    let mut result = Vec::new();
    for row in orders {
        result.push(crate::market_data::alpaca::AlpacaOrderInfo {
            alpaca_order_id: row.alpaca_order_id,
            internal_order_id: row.internal_order_id,
            symbol: row.symbol,
            side: row.side,
            quantity: row.quantity,
            order_type: row.order_type,
            status: row.status,
            submitted_at: row.submitted_at,
            filled_at: row.filled_at,
            filled_qty: row.filled_qty,
            filled_avg_price: row.filled_avg_price,
            time_in_force: row.time_in_force,
            limit_price: row.limit_price,
            stop_price: row.stop_price,
        });
    }

    Ok(result)
}

/// Insert Alpaca position snapshot
pub async fn insert_alpaca_position(pool: &PgPool, position: &crate::market_data::alpaca::AlpacaPosition) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO alpaca_positions
        (symbol, qty, side, market_value, cost_basis, unrealized_pl, unrealized_plpc, current_price,
         position_data)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
        position.symbol,
        position.qty,
        position.side,
        position.market_value,
        position.cost_basis,
        position.unrealized_pl,
        position.unrealized_plpc,
        position.current_price,
        serde_json::json!(position)
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Get current Alpaca positions
pub async fn get_current_alpaca_positions(pool: &PgPool) -> Result<Vec<crate::market_data::alpaca::AlpacaPosition>> {
    let positions = sqlx::query!(
        r#"
        SELECT DISTINCT ON (symbol) symbol, qty, side, market_value, cost_basis,
               unrealized_pl, unrealized_plpc, current_price, position_data
        FROM alpaca_positions
        ORDER BY symbol, created_at DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    let mut result = Vec::new();
    for row in positions {
        result.push(crate::market_data::alpaca::AlpacaPosition {
            symbol: row.symbol,
            qty: row.qty,
            side: row.side,
            market_value: row.market_value,
            cost_basis: row.cost_basis,
            unrealized_pl: row.unrealized_pl,
            unrealized_plpc: row.unrealized_plpc,
            current_price: row.current_price,
            lastday_price: 0.0, // Not stored in this table
            change_today: 0.0,  // Not stored in this table
        });
    }

    Ok(result)
}

/// Insert Alpaca execution statistics
pub async fn insert_alpaca_execution_stats(pool: &PgPool, stats: &crate::market_data::alpaca::AlpacaExecutionStats) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO alpaca_execution_stats
        (total_orders, filled_orders, cancelled_orders, rejected_orders, total_volume,
         average_fill_time_ms, slippage_bps)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        stats.total_orders as i64,
        stats.filled_orders as i64,
        stats.cancelled_orders as i64,
        stats.rejected_orders as i64,
        stats.total_volume,
        stats.average_fill_time_ms,
        stats.slippage_bps
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Get latest Alpaca execution statistics
pub async fn get_latest_alpaca_execution_stats(pool: &PgPool) -> Result<Option<crate::market_data::alpaca::AlpacaExecutionStats>> {
    let stats = sqlx::query!(
        r#"
        SELECT total_orders, filled_orders, cancelled_orders, rejected_orders, total_volume,
               average_fill_time_ms, slippage_bps, created_at
        FROM alpaca_execution_stats
        ORDER BY created_at DESC
        LIMIT 1
        "#
    )
    .fetch_optional(pool)
    .await?;

    if let Some(row) = stats {
        Ok(Some(crate::market_data::alpaca::AlpacaExecutionStats {
            total_orders: row.total_orders as u64,
            filled_orders: row.filled_orders as u64,
            cancelled_orders: row.cancelled_orders as u64,
            rejected_orders: row.rejected_orders as u64,
            total_volume: row.total_volume,
            average_fill_time_ms: row.average_fill_time_ms,
            slippage_bps: row.slippage_bps,
            last_updated: row.created_at,
        }))
    } else {
        Ok(None)
    }
}
