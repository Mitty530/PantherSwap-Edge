use crate::database::Database;
use crate::trading::alpaca_execution::{AlpacaOrderInfo, ExecutionStats};
use crate::trading::alpaca_trading_engine::TradingPerformanceMetrics;
use crate::utils::Result;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::Row;
use tracing::{info, error};
use uuid::Uuid;

/// Enhanced database logging for Alpaca trading activities
pub struct AlpacaLogger {
    database: Database,
}

impl AlpacaLogger {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    /// Log Alpaca order execution with comprehensive details
    pub async fn log_alpaca_order(&self, order_info: &AlpacaOrderInfo, execution_time_ms: i64) -> Result<()> {
        let query = r#"
            INSERT INTO alpaca_orders (
                alpaca_order_id, internal_order_id, symbol, side, quantity, order_type,
                status, submitted_at, filled_at, filled_qty, filled_avg_price,
                time_in_force, limit_price, stop_price, execution_time_ms, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, NOW())
            ON CONFLICT (alpaca_order_id) DO UPDATE SET
                status = EXCLUDED.status,
                filled_at = EXCLUDED.filled_at,
                filled_qty = EXCLUDED.filled_qty,
                filled_avg_price = EXCLUDED.filled_avg_price,
                execution_time_ms = EXCLUDED.execution_time_ms,
                updated_at = NOW()
        "#;

        sqlx::query(query)
            .bind(&order_info.alpaca_order_id)
            .bind(&order_info.internal_order_id)
            .bind(&order_info.symbol)
            .bind(&order_info.side)
            .bind(&order_info.quantity)
            .bind(&order_info.order_type)
            .bind(&order_info.status)
            .bind(&order_info.submitted_at)
            .bind(&order_info.filled_at)
            .bind(&order_info.filled_qty)
            .bind(&order_info.filled_avg_price)
            .bind(&order_info.time_in_force)
            .bind(&order_info.limit_price)
            .bind(&order_info.stop_price)
            .bind(execution_time_ms)
            .execute(&self.database.pool)
            .await
            .map_err(|e| crate::utils::PantherSwapError::database(format!("Failed to log Alpaca order: {}", e)))?;

        info!("Logged Alpaca order: {} for {}", order_info.alpaca_order_id, order_info.symbol);
        Ok(())
    }

    /// Log Alpaca account status and portfolio changes
    pub async fn log_account_snapshot(&self, account_data: &Value) -> Result<()> {
        let query = r#"
            INSERT INTO alpaca_account_snapshots (
                account_id, equity, cash, buying_power, portfolio_value,
                day_trade_buying_power, regt_buying_power, account_data, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())
        "#;

        let account_id = account_data.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");
        let equity = account_data.get("equity").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let cash = account_data.get("cash").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let buying_power = account_data.get("buying_power").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let portfolio_value = account_data.get("portfolio_value").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let day_trade_buying_power = account_data.get("day_trade_buying_power").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let regt_buying_power = account_data.get("regt_buying_power").and_then(|v| v.as_f64()).unwrap_or(0.0);

        sqlx::query(query)
            .bind(account_id)
            .bind(equity)
            .bind(cash)
            .bind(buying_power)
            .bind(portfolio_value)
            .bind(day_trade_buying_power)
            .bind(regt_buying_power)
            .bind(account_data)
            .execute(&self.database.pool)
            .await
            .map_err(|e| crate::utils::PantherSwapError::database(format!("Failed to log account snapshot: {}", e)))?;

        info!("Logged account snapshot for account: {}", account_id);
        Ok(())
    }

    /// Log position changes
    pub async fn log_position_change(&self, symbol: &str, position_data: &Value) -> Result<()> {
        let query = r#"
            INSERT INTO alpaca_positions (
                symbol, qty, side, market_value, cost_basis, unrealized_pl,
                unrealized_plpc, current_price, position_data, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
        "#;

        let qty = position_data.get("qty").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let side = position_data.get("side").and_then(|v| v.as_str()).unwrap_or("unknown");
        let market_value = position_data.get("market_value").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let cost_basis = position_data.get("cost_basis").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let unrealized_pl = position_data.get("unrealized_pl").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let unrealized_plpc = position_data.get("unrealized_plpc").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let current_price = position_data.get("current_price").and_then(|v| v.as_f64()).unwrap_or(0.0);

        sqlx::query(query)
            .bind(symbol)
            .bind(qty)
            .bind(side)
            .bind(market_value)
            .bind(cost_basis)
            .bind(unrealized_pl)
            .bind(unrealized_plpc)
            .bind(current_price)
            .bind(position_data)
            .execute(&self.database.pool)
            .await
            .map_err(|e| crate::utils::PantherSwapError::database(format!("Failed to log position change: {}", e)))?;

        info!("Logged position change for {}: {} shares", symbol, qty);
        Ok(())
    }

    /// Log trading performance metrics
    pub async fn log_performance_metrics(&self, metrics: &TradingPerformanceMetrics) -> Result<()> {
        let query = r#"
            INSERT INTO alpaca_performance_metrics (
                total_trades, profitable_trades, total_pnl, total_volume,
                max_drawdown, sharpe_ratio, win_rate, average_trade_duration_minutes,
                daily_pnl, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
        "#;

        let daily_pnl_json = serde_json::to_value(&metrics.daily_pnl)
            .map_err(|e| crate::utils::PantherSwapError::database(format!("Failed to serialize daily PnL: {}", e)))?;

        sqlx::query(query)
            .bind(metrics.total_trades as i64)
            .bind(metrics.profitable_trades as i64)
            .bind(metrics.total_pnl)
            .bind(metrics.total_volume)
            .bind(metrics.max_drawdown)
            .bind(metrics.sharpe_ratio)
            .bind(metrics.win_rate)
            .bind(metrics.average_trade_duration_minutes)
            .bind(&daily_pnl_json)
            .execute(&self.database.pool)
            .await
            .map_err(|e| crate::utils::PantherSwapError::database(format!("Failed to log performance metrics: {}", e)))?;

        info!("Logged performance metrics: {} trades, {:.2}% win rate", 
            metrics.total_trades, metrics.win_rate * 100.0);
        Ok(())
    }

    /// Log execution statistics
    pub async fn log_execution_stats(&self, stats: &ExecutionStats) -> Result<()> {
        let query = r#"
            INSERT INTO alpaca_execution_stats (
                total_orders, filled_orders, cancelled_orders, rejected_orders,
                total_volume, average_fill_time_ms, slippage_bps, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
        "#;

        sqlx::query(query)
            .bind(stats.total_orders as i64)
            .bind(stats.filled_orders as i64)
            .bind(stats.cancelled_orders as i64)
            .bind(stats.rejected_orders as i64)
            .bind(stats.total_volume)
            .bind(stats.average_fill_time_ms)
            .bind(stats.slippage_bps)
            .execute(&self.database.pool)
            .await
            .map_err(|e| crate::utils::PantherSwapError::database(format!("Failed to log execution stats: {}", e)))?;

        info!("Logged execution stats: {} orders, {:.2}ms avg fill time", 
            stats.total_orders, stats.average_fill_time_ms);
        Ok(())
    }

    /// Log market data events from Alpaca
    pub async fn log_market_event(&self, symbol: &str, event_type: &str, event_data: &Value) -> Result<()> {
        let query = r#"
            INSERT INTO alpaca_market_events (
                symbol, event_type, event_data, created_at
            ) VALUES ($1, $2, $3, NOW())
        "#;

        sqlx::query(query)
            .bind(symbol)
            .bind(event_type)
            .bind(event_data)
            .execute(&self.database.pool)
            .await
            .map_err(|e| crate::utils::PantherSwapError::database(format!("Failed to log market event: {}", e)))?;

        Ok(())
    }

    /// Get trading performance summary from database
    pub async fn get_performance_summary(&self, days: i32) -> Result<Value> {
        let query = r#"
            SELECT 
                COUNT(*) as total_orders,
                COUNT(*) FILTER (WHERE status = 'filled') as filled_orders,
                AVG(execution_time_ms) as avg_execution_time,
                SUM(filled_qty * filled_avg_price) as total_volume,
                AVG(CASE WHEN filled_avg_price IS NOT NULL AND limit_price IS NOT NULL 
                    THEN ABS(filled_avg_price - limit_price) / limit_price * 10000 
                    ELSE 0 END) as avg_slippage_bps
            FROM alpaca_orders 
            WHERE created_at >= NOW() - INTERVAL '%d days'
        "#;

        let row = sqlx::query(&query.replace("%d", &days.to_string()))
            .fetch_one(&self.database.pool)
            .await
            .map_err(|e| crate::utils::PantherSwapError::database(format!("Failed to get performance summary: {}", e)))?;

        let total_orders: i64 = row.get("total_orders");
        let filled_orders: i64 = row.get("filled_orders");
        let avg_execution_time: Option<f64> = row.get("avg_execution_time");
        let total_volume: Option<f64> = row.get("total_volume");
        let avg_slippage_bps: Option<f64> = row.get("avg_slippage_bps");

        Ok(serde_json::json!({
            "period_days": days,
            "total_orders": total_orders,
            "filled_orders": filled_orders,
            "fill_rate": if total_orders > 0 { filled_orders as f64 / total_orders as f64 } else { 0.0 },
            "avg_execution_time_ms": avg_execution_time.unwrap_or(0.0),
            "total_volume": total_volume.unwrap_or(0.0),
            "avg_slippage_bps": avg_slippage_bps.unwrap_or(0.0),
        }))
    }

    /// Get recent orders for analysis
    pub async fn get_recent_orders(&self, limit: i32) -> Result<Vec<AlpacaOrderInfo>> {
        let query = r#"
            SELECT alpaca_order_id, internal_order_id, symbol, side, quantity, order_type,
                   status, submitted_at, filled_at, filled_qty, filled_avg_price,
                   time_in_force, limit_price, stop_price
            FROM alpaca_orders 
            ORDER BY created_at DESC 
            LIMIT $1
        "#;

        let rows = sqlx::query(query)
            .bind(limit)
            .fetch_all(&self.database.pool)
            .await
            .map_err(|e| crate::utils::PantherSwapError::database(format!("Failed to get recent orders: {}", e)))?;

        let mut orders = Vec::new();
        for row in rows {
            let order = AlpacaOrderInfo {
                alpaca_order_id: row.get("alpaca_order_id"),
                internal_order_id: row.get("internal_order_id"),
                symbol: row.get("symbol"),
                side: row.get("side"),
                quantity: row.get("quantity"),
                order_type: row.get("order_type"),
                status: row.get("status"),
                submitted_at: row.get("submitted_at"),
                filled_at: row.get("filled_at"),
                filled_qty: row.get("filled_qty"),
                filled_avg_price: row.get("filled_avg_price"),
                time_in_force: row.get("time_in_force"),
                limit_price: row.get("limit_price"),
                stop_price: row.get("stop_price"),
            };
            orders.push(order);
        }

        Ok(orders)
    }

    /// Create database tables for Alpaca logging if they don't exist
    pub async fn create_tables(&self) -> Result<()> {
        let tables = vec![
            // Alpaca orders table
            r#"
            CREATE TABLE IF NOT EXISTS alpaca_orders (
                id SERIAL PRIMARY KEY,
                alpaca_order_id VARCHAR(255) UNIQUE NOT NULL,
                internal_order_id UUID NOT NULL,
                symbol VARCHAR(20) NOT NULL,
                side VARCHAR(10) NOT NULL,
                quantity DECIMAL(20, 8) NOT NULL,
                order_type VARCHAR(20) NOT NULL,
                status VARCHAR(20) NOT NULL,
                submitted_at TIMESTAMPTZ NOT NULL,
                filled_at TIMESTAMPTZ,
                filled_qty DECIMAL(20, 8) DEFAULT 0,
                filled_avg_price DECIMAL(20, 8),
                time_in_force VARCHAR(20),
                limit_price DECIMAL(20, 8),
                stop_price DECIMAL(20, 8),
                execution_time_ms INTEGER,
                created_at TIMESTAMPTZ DEFAULT NOW(),
                updated_at TIMESTAMPTZ DEFAULT NOW()
            )
            "#,
            
            // Alpaca account snapshots
            r#"
            CREATE TABLE IF NOT EXISTS alpaca_account_snapshots (
                id SERIAL PRIMARY KEY,
                account_id VARCHAR(255) NOT NULL,
                equity DECIMAL(20, 8),
                cash DECIMAL(20, 8),
                buying_power DECIMAL(20, 8),
                portfolio_value DECIMAL(20, 8),
                day_trade_buying_power DECIMAL(20, 8),
                regt_buying_power DECIMAL(20, 8),
                account_data JSONB,
                created_at TIMESTAMPTZ DEFAULT NOW()
            )
            "#,
            
            // Alpaca positions
            r#"
            CREATE TABLE IF NOT EXISTS alpaca_positions (
                id SERIAL PRIMARY KEY,
                symbol VARCHAR(20) NOT NULL,
                qty DECIMAL(20, 8) NOT NULL,
                side VARCHAR(10),
                market_value DECIMAL(20, 8),
                cost_basis DECIMAL(20, 8),
                unrealized_pl DECIMAL(20, 8),
                unrealized_plpc DECIMAL(8, 6),
                current_price DECIMAL(20, 8),
                position_data JSONB,
                created_at TIMESTAMPTZ DEFAULT NOW()
            )
            "#,
            
            // Performance metrics
            r#"
            CREATE TABLE IF NOT EXISTS alpaca_performance_metrics (
                id SERIAL PRIMARY KEY,
                total_trades BIGINT NOT NULL,
                profitable_trades BIGINT NOT NULL,
                total_pnl DECIMAL(20, 8) NOT NULL,
                total_volume DECIMAL(20, 8) NOT NULL,
                max_drawdown DECIMAL(8, 6),
                sharpe_ratio DECIMAL(8, 6),
                win_rate DECIMAL(8, 6),
                average_trade_duration_minutes DECIMAL(10, 2),
                daily_pnl JSONB,
                created_at TIMESTAMPTZ DEFAULT NOW()
            )
            "#,
            
            // Execution statistics
            r#"
            CREATE TABLE IF NOT EXISTS alpaca_execution_stats (
                id SERIAL PRIMARY KEY,
                total_orders BIGINT NOT NULL,
                filled_orders BIGINT NOT NULL,
                cancelled_orders BIGINT NOT NULL,
                rejected_orders BIGINT NOT NULL,
                total_volume DECIMAL(20, 8) NOT NULL,
                average_fill_time_ms DECIMAL(10, 2),
                slippage_bps DECIMAL(8, 4),
                created_at TIMESTAMPTZ DEFAULT NOW()
            )
            "#,
            
            // Market events
            r#"
            CREATE TABLE IF NOT EXISTS alpaca_market_events (
                id SERIAL PRIMARY KEY,
                symbol VARCHAR(20) NOT NULL,
                event_type VARCHAR(50) NOT NULL,
                event_data JSONB,
                created_at TIMESTAMPTZ DEFAULT NOW()
            )
            "#,
        ];

        for table_sql in tables {
            sqlx::query(table_sql)
                .execute(&self.database.pool)
                .await
                .map_err(|e| crate::utils::PantherSwapError::database(format!("Failed to create table: {}", e)))?;
        }

        // Create indexes for better performance
        let indexes = vec![
            "CREATE INDEX IF NOT EXISTS idx_alpaca_orders_symbol_created ON alpaca_orders(symbol, created_at)",
            "CREATE INDEX IF NOT EXISTS idx_alpaca_orders_status ON alpaca_orders(status)",
            "CREATE INDEX IF NOT EXISTS idx_alpaca_positions_symbol ON alpaca_positions(symbol, created_at)",
            "CREATE INDEX IF NOT EXISTS idx_alpaca_market_events_symbol ON alpaca_market_events(symbol, created_at)",
        ];

        for index_sql in indexes {
            sqlx::query(index_sql)
                .execute(&self.database.pool)
                .await
                .map_err(|e| crate::utils::PantherSwapError::database(format!("Failed to create index: {}", e)))?;
        }

        info!("✅ Alpaca logging tables and indexes created successfully");
        Ok(())
    }
}
