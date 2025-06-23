// Simplified database query functions that compile without database connection
// This provides the core functionality for database operations

use crate::utils::Result;
use crate::database::types::*;
use sqlx::{PgPool, Row};
use chrono::{DateTime, Utc};
use uuid::Uuid;

// ============================================================================
// BASIC CRUD OPERATIONS
// ============================================================================

/// Insert a new instrument (simplified version)
pub async fn insert_instrument_simple(pool: &PgPool, instrument: &Instrument) -> Result<Uuid> {
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

/// Get instrument by symbol (simplified version)
pub async fn get_instrument_by_symbol_simple(pool: &PgPool, symbol: &str) -> Result<Option<Instrument>> {
    let row = sqlx::query(
        r#"
        SELECT id, symbol, name, instrument_type, base_currency, quote_currency,
               tick_size::FLOAT8 as tick_size, lot_size::FLOAT8 as lot_size,
               is_active, created_at, updated_at
        FROM instruments WHERE symbol = $1
        "#
    )
        .bind(symbol)
        .fetch_optional(pool)
        .await?;

    match row {
        Some(row) => {
            let instrument = Instrument {
                id: row.get("id"),
                symbol: row.get("symbol"),
                name: row.get("name"),
                instrument_type: row.get("instrument_type"),
                base_currency: row.get("base_currency"),
                quote_currency: row.get("quote_currency"),
                tick_size: row.get("tick_size"),
                lot_size: row.get("lot_size"),
                is_active: row.get("is_active"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Some(instrument))
        }
        None => Ok(None),
    }
}

/// Get instrument by ID (simplified version)
pub async fn get_instrument_by_id_simple(pool: &PgPool, id: Uuid) -> Result<Option<Instrument>> {
    let row = sqlx::query(
        r#"
        SELECT id, symbol, name, instrument_type, base_currency, quote_currency,
               tick_size::FLOAT8 as tick_size, lot_size::FLOAT8 as lot_size,
               is_active, created_at, updated_at
        FROM instruments WHERE id = $1
        "#
    )
        .bind(id)
        .fetch_optional(pool)
        .await?;

    match row {
        Some(row) => {
            let instrument = Instrument {
                id: row.get("id"),
                symbol: row.get("symbol"),
                name: row.get("name"),
                instrument_type: row.get("instrument_type"),
                base_currency: row.get("base_currency"),
                quote_currency: row.get("quote_currency"),
                tick_size: row.get("tick_size"),
                lot_size: row.get("lot_size"),
                is_active: row.get("is_active"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Some(instrument))
        }
        None => Ok(None),
    }
}

/// Get all active instruments (simplified version)
pub async fn get_active_instruments_simple(pool: &PgPool) -> Result<Vec<Instrument>> {
    let rows = sqlx::query(
        r#"
        SELECT id, symbol, name, instrument_type, base_currency, quote_currency,
               tick_size::FLOAT8 as tick_size, lot_size::FLOAT8 as lot_size,
               is_active, created_at, updated_at
        FROM instruments WHERE is_active = true ORDER BY symbol
        "#
    )
        .fetch_all(pool)
        .await?;

    let mut instruments = Vec::new();
    for row in rows {
        let instrument = Instrument {
            id: row.get("id"),
            symbol: row.get("symbol"),
            name: row.get("name"),
            instrument_type: row.get("instrument_type"),
            base_currency: row.get("base_currency"),
            quote_currency: row.get("quote_currency"),
            tick_size: row.get("tick_size"),
            lot_size: row.get("lot_size"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };
        instruments.push(instrument);
    }

    Ok(instruments)
}

/// Insert market tick data (simplified version)
pub async fn insert_market_tick_simple(pool: &PgPool, tick: &MarketTick) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO market_ticks 
        (timestamp, instrument_id, provider, bid_price, ask_price, bid_size, ask_size,
         last_price, volume, spread, data_quality_score, raw_data)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#
    )
    .bind(tick.timestamp)
    .bind(tick.instrument_id)
    .bind(&tick.provider)
    .bind(tick.bid_price)
    .bind(tick.ask_price)
    .bind(tick.bid_size)
    .bind(tick.ask_size)
    .bind(tick.last_price)
    .bind(tick.volume)
    .bind(tick.spread)
    .bind(tick.data_quality_score)
    .bind(&tick.raw_data)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get latest market tick for an instrument (simplified version)
pub async fn get_latest_market_tick_simple(pool: &PgPool, instrument_id: Uuid) -> Result<Option<MarketTick>> {
    let row = sqlx::query(
        r#"
        SELECT timestamp, instrument_id, provider,
               bid_price::FLOAT8 as bid_price, ask_price::FLOAT8 as ask_price,
               bid_size::FLOAT8 as bid_size, ask_size::FLOAT8 as ask_size,
               last_price::FLOAT8 as last_price, volume::FLOAT8 as volume,
               spread::FLOAT8 as spread, data_quality_score::FLOAT8 as data_quality_score,
               raw_data
        FROM market_ticks
        WHERE instrument_id = $1
        ORDER BY timestamp DESC
        LIMIT 1
        "#
    )
    .bind(instrument_id)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(row) => {
            let tick = MarketTick {
                timestamp: row.get("timestamp"),
                instrument_id: row.get("instrument_id"),
                provider: row.get("provider"),
                bid_price: row.get("bid_price"),
                ask_price: row.get("ask_price"),
                bid_size: row.get("bid_size"),
                ask_size: row.get("ask_size"),
                last_price: row.get("last_price"),
                volume: row.get("volume"),
                spread: row.get("spread"),
                data_quality_score: row.get("data_quality_score"),
                raw_data: row.get("raw_data"),
            };
            Ok(Some(tick))
        }
        None => Ok(None),
    }
}

/// Batch insert market ticks for high-frequency data (simplified version)
pub async fn batch_insert_market_ticks_simple(pool: &PgPool, ticks: &[MarketTick]) -> Result<u64> {
    if ticks.is_empty() {
        return Ok(0);
    }

    let mut inserted_count = 0;

    // Insert in batches to avoid overwhelming the database
    for chunk in ticks.chunks(100) {
        for tick in chunk {
            match insert_market_tick_simple(pool, tick).await {
                Ok(_) => inserted_count += 1,
                Err(e) => {
                    tracing::warn!("Failed to insert market tick: {}", e);
                }
            }
        }
    }

    Ok(inserted_count)
}

/// Get market ticks within time range (simplified version)
pub async fn get_market_ticks_range_simple(
    pool: &PgPool,
    instrument_id: Uuid,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    limit: Option<i64>
) -> Result<Vec<MarketTick>> {
    let limit = limit.unwrap_or(1000);

    let rows = sqlx::query(
        r#"
        SELECT * FROM market_ticks
        WHERE instrument_id = $1
        AND timestamp >= $2
        AND timestamp <= $3
        ORDER BY timestamp DESC
        LIMIT $4
        "#
    )
    .bind(instrument_id)
    .bind(start_time)
    .bind(end_time)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let mut ticks = Vec::new();
    for row in rows {
        let tick = MarketTick {
            timestamp: row.get("timestamp"),
            instrument_id: row.get("instrument_id"),
            provider: row.get("provider"),
            bid_price: row.get("bid_price"),
            ask_price: row.get("ask_price"),
            bid_size: row.get("bid_size"),
            ask_size: row.get("ask_size"),
            last_price: row.get("last_price"),
            volume: row.get("volume"),
            spread: row.get("spread"),
            data_quality_score: row.get("data_quality_score"),
            raw_data: row.get("raw_data"),
        };
        ticks.push(tick);
    }

    Ok(ticks)
}

/// Insert AI prediction (simplified version)
pub async fn insert_ai_prediction_simple(pool: &PgPool, prediction: &AIPrediction) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO ai_predictions 
        (timestamp, instrument_id, model_type, model_version, prediction_horizon_minutes,
         predicted_price, predicted_volatility, confidence_score, prediction_intervals,
         feature_importance)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#
    )
    .bind(prediction.timestamp)
    .bind(prediction.instrument_id)
    .bind(&prediction.model_type)
    .bind(&prediction.model_version)
    .bind(prediction.prediction_horizon_minutes)
    .bind(prediction.predicted_price)
    .bind(prediction.predicted_volatility)
    .bind(prediction.confidence_score)
    .bind(&prediction.prediction_intervals)
    .bind(&prediction.feature_importance)
    .execute(pool)
    .await?;

    Ok(())
}

/// Insert trading signal (simplified version)
pub async fn insert_trading_signal_simple(pool: &PgPool, signal: &TradingSignal) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO trading_signals
        (id, timestamp, instrument_id, strategy_name, signal_type, signal_strength,
         confidence_score, recommended_size, entry_price, stop_loss, take_profit,
         time_horizon, expected_return, risk_metrics)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        "#
    )
    .bind(signal.id)
    .bind(signal.timestamp)
    .bind(signal.instrument_id)
    .bind(&signal.strategy_name)
    .bind(&signal.signal_type)
    .bind(signal.signal_strength)
    .bind(signal.confidence_score)
    .bind(signal.recommended_size)
    .bind(signal.entry_price)
    .bind(signal.stop_loss)
    .bind(signal.take_profit)
    .bind(signal.time_horizon)
    .bind(signal.expected_return)
    .bind(&signal.risk_metrics)
    .execute(pool)
    .await?;

    Ok(())
}

// ============================================================================
// TRADING EXECUTION LOGGING
// ============================================================================

/// Insert order into database
pub async fn insert_order(pool: &PgPool, order: &crate::trading::execution::Order) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO orders
        (id, instrument_id, side, quantity, filled_quantity, remaining_quantity,
         order_type, price, stop_price, time_in_force, execution_style, status,
         created_at, updated_at, strategy_name)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        ON CONFLICT (id) DO UPDATE SET
            filled_quantity = EXCLUDED.filled_quantity,
            remaining_quantity = EXCLUDED.remaining_quantity,
            status = EXCLUDED.status,
            updated_at = EXCLUDED.updated_at
        "#
    )
    .bind(order.id)
    .bind(order.instrument_id)
    .bind(order.side.to_string())
    .bind(order.quantity)
    .bind(order.filled_quantity)
    .bind(order.remaining_quantity)
    .bind(order.order_type.to_string())
    .bind(order.price)
    .bind(order.stop_price)
    .bind(order.time_in_force.to_string())
    .bind(order.execution_style.to_string())
    .bind(order.status.to_string())
    .bind(order.created_at)
    .bind(order.updated_at)
    .bind(&order.strategy_name)
    .execute(pool)
    .await?;

    Ok(())
}

/// Insert fill into database
pub async fn insert_fill(pool: &PgPool, fill: &crate::trading::execution::Fill) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO fills
        (id, order_id, quantity, price, timestamp, commission, venue, liquidity_flag)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#
    )
    .bind(fill.id)
    .bind(fill.order_id)
    .bind(fill.quantity)
    .bind(fill.price)
    .bind(fill.timestamp)
    .bind(fill.commission.unwrap_or(0.0))
    .bind(&fill.venue.as_ref().unwrap_or(&"internal".to_string()))
    .bind(&fill.liquidity_flag.as_ref().unwrap_or(&"unknown".to_string()))
    .execute(pool)
    .await?;

    Ok(())
}

/// Insert trade execution into database
pub async fn insert_trade_execution(
    pool: &PgPool,
    execution: &crate::trading::signals::ExecutionResult,
    signal_id: Option<Uuid>,
    execution_time_ms: Option<i32>,
    slippage_bps: Option<f64>,
    fees: Option<f64>
) -> Result<()> {
    let pnl = execution.realized_pnl.unwrap_or(0.0) + execution.unrealized_pnl.unwrap_or(0.0);

    sqlx::query(
        r#"
        INSERT INTO trade_executions
        (timestamp, instrument_id, signal_id, action, quantity, price,
         execution_time_ms, slippage_bps, fees, pnl, confidence_score, strategy_name)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#
    )
    .bind(execution.execution_time)
    .bind(execution.instrument_id)
    .bind(signal_id)
    .bind(if execution.filled_quantity > 0.0 { "BUY" } else { "SELL" })
    .bind(execution.filled_quantity.abs())
    .bind(execution.average_price)
    .bind(execution_time_ms)
    .bind(slippage_bps)
    .bind(fees)
    .bind(pnl)
    .bind(execution.confidence_score)
    .bind(&execution.strategy_name)
    .execute(pool)
    .await?;

    Ok(())
}

/// Insert position update into database
pub async fn insert_position_update(
    pool: &PgPool,
    position: &crate::trading::signals::Position
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO position_updates
        (timestamp, instrument_id, strategy_name, size, entry_price, entry_time,
         stop_loss, take_profit, unrealized_pnl, realized_pnl, risk_score)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#
    )
    .bind(chrono::Utc::now())
    .bind(position.instrument_id)
    .bind(&position.strategy_name)
    .bind(position.size)
    .bind(position.entry_price)
    .bind(position.entry_time)
    .bind(position.stop_loss)
    .bind(position.take_profit)
    .bind(position.unrealized_pnl)
    .bind(position.realized_pnl)
    .bind(position.risk_metrics.risk_score)
    .execute(pool)
    .await?;

    Ok(())
}

/// Insert risk metrics into database
pub async fn insert_risk_metrics(
    pool: &PgPool,
    instrument_id: Option<Uuid>,
    portfolio_var: f64,
    position_size: f64,
    leverage: f64,
    drawdown: f64,
    sharpe_ratio: Option<f64>,
    max_loss_24h: f64,
    risk_score: f64
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO risk_metrics
        (timestamp, instrument_id, portfolio_var, position_size, leverage,
         drawdown, sharpe_ratio, max_loss_24h, risk_score)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#
    )
    .bind(chrono::Utc::now())
    .bind(instrument_id)
    .bind(portfolio_var)
    .bind(position_size)
    .bind(leverage)
    .bind(drawdown)
    .bind(sharpe_ratio)
    .bind(max_loss_24h)
    .bind(risk_score)
    .execute(pool)
    .await?;

    Ok(())
}

/// Insert P&L record into database
pub async fn insert_pnl_record(
    pool: &PgPool,
    instrument_id: Uuid,
    strategy_name: &str,
    realized_pnl: f64,
    unrealized_pnl: f64,
    total_pnl: f64,
    trade_count: i32,
    win_rate: f64,
    sharpe_ratio: Option<f64>
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO pnl_records
        (timestamp, instrument_id, strategy_name, realized_pnl, unrealized_pnl,
         total_pnl, trade_count, win_rate, sharpe_ratio)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#
    )
    .bind(chrono::Utc::now())
    .bind(instrument_id)
    .bind(strategy_name)
    .bind(realized_pnl)
    .bind(unrealized_pnl)
    .bind(total_pnl)
    .bind(trade_count)
    .bind(win_rate)
    .bind(sharpe_ratio)
    .execute(pool)
    .await?;

    Ok(())
}

// ============================================================================
// ANALYTICAL QUERIES
// ============================================================================

/// Get OHLCV data with time bucketing (simplified version)
pub async fn get_ohlcv_data_simple(
    pool: &PgPool,
    instrument_id: Uuid,
    bucket_size: &str,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>
) -> Result<Vec<(DateTime<Utc>, f64, f64, f64, f64, f64)>> {
    let rows = sqlx::query(
        r#"
        SELECT 
            time_bucket($1, timestamp) as time_bucket,
            first(last_price, timestamp) as open_price,
            max(last_price) as high_price,
            min(last_price) as low_price,
            last(last_price, timestamp) as close_price,
            sum(volume) as total_volume
        FROM market_ticks
        WHERE instrument_id = $2
        AND timestamp >= $3
        AND timestamp <= $4
        AND last_price IS NOT NULL
        GROUP BY time_bucket
        ORDER BY time_bucket
        "#
    )
    .bind(bucket_size)
    .bind(instrument_id)
    .bind(start_time)
    .bind(end_time)
    .fetch_all(pool)
    .await?;

    let mut result = Vec::new();
    for row in rows {
        let time_bucket: DateTime<Utc> = row.get("time_bucket");
        let open: Option<f64> = row.get("open_price");
        let high: Option<f64> = row.get("high_price");
        let low: Option<f64> = row.get("low_price");
        let close: Option<f64> = row.get("close_price");
        let volume: Option<f64> = row.get("total_volume");
        
        result.push((
            time_bucket,
            open.unwrap_or(0.0),
            high.unwrap_or(0.0),
            low.unwrap_or(0.0),
            close.unwrap_or(0.0),
            volume.unwrap_or(0.0),
        ));
    }

    Ok(result)
}

/// Get database health check (simplified version)
pub async fn database_health_check_simple(pool: &PgPool) -> Result<bool> {
    let result = sqlx::query("SELECT 1 as health_check")
        .fetch_one(pool)
        .await;

    Ok(result.is_ok())
}

/// Simple query manager for basic operations
pub struct SimpleQueryManager {
    pool: PgPool,
}

impl SimpleQueryManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Instrument operations
    pub async fn insert_instrument(&self, instrument: &Instrument) -> Result<Uuid> {
        insert_instrument_simple(&self.pool, instrument).await
    }

    pub async fn get_instrument_by_symbol(&self, symbol: &str) -> Result<Option<Instrument>> {
        get_instrument_by_symbol_simple(&self.pool, symbol).await
    }

    pub async fn get_instrument_by_id(&self, id: Uuid) -> Result<Option<Instrument>> {
        get_instrument_by_id_simple(&self.pool, id).await
    }

    pub async fn get_active_instruments(&self) -> Result<Vec<Instrument>> {
        get_active_instruments_simple(&self.pool).await
    }

    // Market data operations
    pub async fn insert_market_tick(&self, tick: &MarketTick) -> Result<()> {
        insert_market_tick_simple(&self.pool, tick).await
    }

    pub async fn batch_insert_market_ticks(&self, ticks: &[MarketTick]) -> Result<u64> {
        batch_insert_market_ticks_simple(&self.pool, ticks).await
    }

    pub async fn get_latest_market_tick(&self, instrument_id: Uuid) -> Result<Option<MarketTick>> {
        get_latest_market_tick_simple(&self.pool, instrument_id).await
    }

    pub async fn get_market_ticks_range(
        &self,
        instrument_id: Uuid,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        limit: Option<i64>
    ) -> Result<Vec<MarketTick>> {
        get_market_ticks_range_simple(&self.pool, instrument_id, start_time, end_time, limit).await
    }

    // AI and trading operations
    pub async fn insert_ai_prediction(&self, prediction: &AIPrediction) -> Result<()> {
        insert_ai_prediction_simple(&self.pool, prediction).await
    }

    pub async fn insert_trading_signal(&self, signal: &TradingSignal) -> Result<()> {
        insert_trading_signal_simple(&self.pool, signal).await
    }

    // Analytics operations
    pub async fn get_ohlcv_data(
        &self,
        instrument_id: Uuid,
        bucket_size: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>
    ) -> Result<Vec<(DateTime<Utc>, f64, f64, f64, f64, f64)>> {
        get_ohlcv_data_simple(&self.pool, instrument_id, bucket_size, start_time, end_time).await
    }

    // Health and utility operations
    pub async fn health_check(&self) -> Result<bool> {
        database_health_check_simple(&self.pool).await
    }

    /// Test basic database operations (for development/testing)
    pub async fn test_basic_operations(&self) -> Result<()> {
        tracing::info!("Testing basic database operations...");

        // Test 1: Health check
        let is_healthy = self.health_check().await?;
        if !is_healthy {
            return Err(crate::utils::errors::PantherSwapError::internal("Database health check failed"));
        }
        tracing::info!("✅ Database health check passed");

        // Test 2: Create a test instrument
        let test_instrument = Instrument {
            id: uuid::Uuid::new_v4(), // This will be overridden by the database
            symbol: "TEST_EURUSD".to_string(),
            name: "Test EUR/USD".to_string(),
            instrument_type: "forex".to_string(),
            base_currency: "EUR".to_string(),
            quote_currency: "USD".to_string(),
            tick_size: 0.00001,
            lot_size: 100000.0,
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let instrument_id = self.insert_instrument(&test_instrument).await?;
        tracing::info!("✅ Test instrument created with ID: {}", instrument_id);

        // Test 3: Retrieve the instrument
        let retrieved_instrument = self.get_instrument_by_id(instrument_id).await?;
        if retrieved_instrument.is_none() {
            return Err(crate::utils::errors::PantherSwapError::internal("Failed to retrieve test instrument"));
        }
        tracing::info!("✅ Test instrument retrieved successfully");

        // Test 4: Create a test market tick
        let test_tick = MarketTick {
            timestamp: chrono::Utc::now(),
            instrument_id,
            provider: "test_provider".to_string(),
            bid_price: 1.0850,
            ask_price: 1.0852,
            bid_size: 1000000.0,
            ask_size: 1000000.0,
            last_price: Some(1.0851),
            volume: Some(5000000.0),
            spread: 0.0002,
            data_quality_score: 0.95,
            raw_data: serde_json::json!({"test": true, "source": "test"}),
        };

        self.insert_market_tick(&test_tick).await?;
        tracing::info!("✅ Test market tick inserted successfully");

        // Test 5: Retrieve the latest market tick
        let latest_tick = self.get_latest_market_tick(instrument_id).await?;
        if latest_tick.is_none() {
            return Err(crate::utils::errors::PantherSwapError::internal("Failed to retrieve test market tick"));
        }
        tracing::info!("✅ Test market tick retrieved successfully");

        tracing::info!("🎉 All basic database operations completed successfully!");
        Ok(())
    }

    pub async fn get_latest_market_ticks(&self, instrument_id: Option<Uuid>, limit: Option<i64>) -> Result<Vec<MarketTick>> {
        let limit = limit.unwrap_or(100);

        if let Some(id) = instrument_id {
            sqlx::query_as::<_, MarketTick>(
                "SELECT timestamp, instrument_id, provider,
                        bid_price::FLOAT8 as bid_price, ask_price::FLOAT8 as ask_price,
                        bid_size::FLOAT8 as bid_size, ask_size::FLOAT8 as ask_size,
                        last_price::FLOAT8 as last_price, volume::FLOAT8 as volume,
                        spread::FLOAT8 as spread, data_quality_score::FLOAT8 as data_quality_score,
                        raw_data
                 FROM market_ticks
                 WHERE instrument_id = $1
                 ORDER BY timestamp DESC
                 LIMIT $2"
            )
            .bind(id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
        } else {
            sqlx::query_as::<_, MarketTick>(
                "SELECT timestamp, instrument_id, provider,
                        bid_price::FLOAT8 as bid_price, ask_price::FLOAT8 as ask_price,
                        bid_size::FLOAT8 as bid_size, ask_size::FLOAT8 as ask_size,
                        last_price::FLOAT8 as last_price, volume::FLOAT8 as volume,
                        spread::FLOAT8 as spread, data_quality_score::FLOAT8 as data_quality_score,
                        raw_data
                 FROM market_ticks
                 ORDER BY timestamp DESC
                 LIMIT $1"
            )
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
        }
    }

    pub async fn get_market_ticks_for_instrument(
        &self,
        instrument_id: Uuid,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: Option<i64>,
    ) -> Result<Vec<MarketTick>> {
        let limit = limit.unwrap_or(100);
        let end_time = end_time.unwrap_or_else(Utc::now);
        let start_time = start_time.unwrap_or_else(|| end_time - chrono::Duration::hours(24));

        sqlx::query_as::<_, MarketTick>(
            "SELECT timestamp, instrument_id, provider,
                    bid_price::FLOAT8 as bid_price, ask_price::FLOAT8 as ask_price,
                    bid_size::FLOAT8 as bid_size, ask_size::FLOAT8 as ask_size,
                    last_price::FLOAT8 as last_price, volume::FLOAT8 as volume,
                    spread::FLOAT8 as spread, data_quality_score::FLOAT8 as data_quality_score,
                    raw_data
             FROM market_ticks
             WHERE instrument_id = $1
               AND timestamp >= $2
               AND timestamp <= $3
             ORDER BY timestamp DESC
             LIMIT $4"
        )
        .bind(instrument_id)
        .bind(start_time)
        .bind(end_time)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn get_instruments_with_filters(
        &self,
        _instrument_type: Option<&str>,
        _is_active: Option<bool>,
        _base_currency: Option<&str>,
        _quote_currency: Option<&str>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Instrument>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        // Simplified version - in production you'd want proper dynamic query building
        sqlx::query_as::<_, Instrument>(
            "SELECT id, symbol, name, instrument_type, base_currency, quote_currency,
                    tick_size::FLOAT8 as tick_size, lot_size::FLOAT8 as lot_size,
                    is_active, created_at, updated_at
             FROM instruments
             ORDER BY created_at DESC
             LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn update_instrument(&self, instrument: &Instrument) -> Result<()> {
        sqlx::query(
            "UPDATE instruments
             SET name = $2, tick_size = $3, lot_size = $4, is_active = $5, updated_at = $6
             WHERE id = $1"
        )
        .bind(instrument.id)
        .bind(&instrument.name)
        .bind(instrument.tick_size)
        .bind(instrument.lot_size)
        .bind(instrument.is_active)
        .bind(instrument.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
