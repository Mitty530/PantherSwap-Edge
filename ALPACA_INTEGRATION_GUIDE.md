# PantherSwap Edge - Alpaca API Integration Guide

## Overview

This guide provides comprehensive instructions for integrating Alpaca API with PantherSwap Edge for live trading capabilities. The integration enables real-time market data, order execution, and comprehensive audit logging through TimescaleDB.

## 🚀 Quick Start

### Prerequisites

1. **Alpaca Account**: Sign up at [alpaca.markets](https://alpaca.markets/)
2. **API Credentials**: Generate paper trading API keys
3. **TimescaleDB**: Ensure database connection is configured
4. **Rust Environment**: Cargo and Rust toolchain installed

### Environment Setup

```bash
# Set Alpaca API credentials
export PANTHERSWAP_MARKET_DATA_ALPACA_API_KEY="your_api_key"
export PANTHERSWAP_MARKET_DATA_ALPACA_SECRET_KEY="your_secret_key"

# Optional: Set database URL (uses config default if not set)
export PANTHERSWAP_DATABASE_URL="your_timescale_db_url"
```

### Run Integration Demo

```bash
# Make demo script executable
chmod +x alpaca_live_trading_demo.sh

# Run comprehensive demo
./alpaca_live_trading_demo.sh
```

## 🏗️ Architecture Overview

### Core Components

1. **AlpacaProvider** (`src/market_data/alpaca.rs`)
   - Real-time market data streaming
   - Historical data retrieval
   - Market status monitoring
   - Rate limiting and error handling

2. **AlpacaExecutionEngine** (`src/trading/alpaca_execution.rs`)
   - Live order execution
   - Position management
   - Performance tracking
   - Risk controls

3. **AlpacaTradingEngine** (`src/trading/alpaca_trading_engine.rs`)
   - Integrated trading system
   - Signal generation and execution
   - Strategy management
   - Performance monitoring

4. **AlpacaLogger** (`src/database/alpaca_logging.rs`)
   - Comprehensive audit logging
   - Performance metrics storage
   - Query optimization
   - Compliance tracking

### Data Flow

```
Market Data (Alpaca) → AlpacaProvider → MarketDataManager → TradingEngine
                                                                    ↓
Database (TimescaleDB) ← AlpacaLogger ← ExecutionEngine ← Trading Signals
```

## 📊 Configuration

### Alpaca Configuration (`config/default.toml`)

```toml
[market_data.alpaca]
api_key = ""                # Set via environment variable
secret_key = ""             # Set via environment variable
base_url = "https://paper-api.alpaca.markets"  # Paper trading
data_url = "https://data.alpaca.markets"
paper_trading = true
enable_streaming = true
max_positions = 10
max_order_value = 10000.0
enable_fractional_shares = true
```

### Trading Configuration

```toml
[trading]
enable_live_trading = true
signal_generation_interval_ms = 1000
risk_check_interval_ms = 500
max_position_size = 100000.0
confidence_threshold = 0.7
```

## 🔧 API Usage Examples

### Market Data

```rust
use pantherswap_edge::market_data::AlpacaProvider;
use pantherswap_edge::config::AlpacaConfig;

// Create provider
let config = AlpacaConfig { /* ... */ };
let provider = AlpacaProvider::new(config)?;

// Get latest quote
let quote = provider.get_latest_quote("AAPL").await?;
println!("AAPL: ${:.2}", quote.exchange_rate);

// Start streaming
let mut stream = provider.start_streaming(vec!["AAPL".to_string()]).await?;
while let Some(event) = stream.recv().await {
    // Handle market data events
}
```

### Order Execution

```rust
use pantherswap_edge::trading::AlpacaExecutionEngine;

// Create execution engine
let engine = AlpacaExecutionEngine::new(config)?;

// Execute market order
let result = engine.market_buy("AAPL", 100.0).await?;
println!("Order executed: {}", result.execution_id);

// Monitor order
let order_info = engine.monitor_order(&result.execution_id, 30).await?;
println!("Order status: {}", order_info.status);
```

### Database Logging

```rust
use pantherswap_edge::database::Database;

// Setup database logging
let database = Database::new(&database_url).await?;
database.setup_alpaca_logging().await?;

// Get logger
let logger = database.alpaca_logger();

// Log performance metrics
logger.log_performance_metrics(&metrics).await?;

// Query performance summary
let summary = logger.get_performance_summary(7).await?;
```

## 📈 Performance Targets

### Latency Requirements

- **Market Data**: <100ms from Alpaca to internal processing
- **Order Execution**: <10ms from signal to order submission
- **Database Writes**: <10ms for audit logging
- **AI Inference**: <100ms for trading decisions

### Throughput Targets

- **Orders per Second**: >1000 TPS
- **Market Data Updates**: Real-time streaming
- **Database Operations**: >5000 writes/second
- **Concurrent Positions**: Up to 100 symbols

### Reliability Metrics

- **Uptime**: 99.9% availability
- **Data Quality**: >95% accuracy
- **Fill Rate**: >98% order execution
- **Slippage**: <5 basis points average

## 🛡️ Risk Management

### Position Limits

```rust
// Configure in AlpacaConfig
max_positions: 10,           // Maximum concurrent positions
max_order_value: 10000.0,    // Maximum single order value
enable_fractional_shares: true,
```

### Risk Controls

- **Pre-trade validation**: Order size and value limits
- **Real-time monitoring**: Position and P&L tracking
- **Circuit breakers**: Automatic trading halt on losses
- **Compliance logging**: Complete audit trail

## 🔍 Monitoring & Alerting

### Performance Monitoring

```rust
// Get execution statistics
let stats = engine.get_execution_stats().await;
println!("Fill rate: {:.2}%", stats.filled_orders as f64 / stats.total_orders as f64 * 100.0);
println!("Average latency: {:.2}ms", stats.average_fill_time_ms);
```

### Database Queries

```sql
-- Recent order performance
SELECT symbol, AVG(execution_time_ms) as avg_latency
FROM alpaca_orders 
WHERE created_at >= NOW() - INTERVAL '1 hour'
GROUP BY symbol;

-- Trading performance summary
SELECT 
    COUNT(*) as total_trades,
    AVG(filled_avg_price * filled_qty) as avg_trade_size,
    SUM(CASE WHEN side = 'buy' THEN filled_qty ELSE -filled_qty END) as net_position
FROM alpaca_orders 
WHERE status = 'filled';
```

## 🚀 Production Deployment

### Environment Configuration

1. **Live API Credentials**
   ```bash
   export PANTHERSWAP_MARKET_DATA_ALPACA_API_KEY="live_api_key"
   export PANTHERSWAP_MARKET_DATA_ALPACA_SECRET_KEY="live_secret_key"
   ```

2. **Trading Configuration**
   ```toml
   [market_data.alpaca]
   base_url = "https://api.alpaca.markets"  # Live trading
   paper_trading = false
   ```

3. **Risk Parameters**
   ```toml
   [trading]
   enable_live_trading = true
   max_position_size = 50000.0  # Adjust based on capital
   confidence_threshold = 0.8   # Higher threshold for live trading
   ```

### Deployment Checklist

- [ ] API credentials configured and validated
- [ ] Database tables created and indexed
- [ ] Risk limits configured appropriately
- [ ] Monitoring and alerting enabled
- [ ] Backup and recovery procedures tested
- [ ] Compliance requirements met
- [ ] Performance benchmarks validated

## 🧪 Testing

### Unit Tests

```bash
# Test individual components
cargo test alpaca_provider
cargo test alpaca_execution
cargo test alpaca_logging
```

### Integration Tests

```bash
# Test complete integration
cargo run --bin test_alpaca_integration

# End-to-end testing
cargo run --bin alpaca_end_to_end_test
```

### Performance Testing

```bash
# Run comprehensive demo
./alpaca_live_trading_demo.sh

# Performance benchmarks
cargo run --bin performance_benchmark
```

## 📚 API Reference

### AlpacaProvider Methods

- `new(config)` - Create new provider
- `validate_configuration()` - Test API connection
- `get_latest_quote(symbol)` - Get current market data
- `start_streaming(symbols)` - Begin real-time streaming
- `get_historical_bars()` - Retrieve historical data
- `is_market_open()` - Check market status

### AlpacaExecutionEngine Methods

- `execute_order(request)` - Submit order for execution
- `market_buy/sell()` - Convenience methods for market orders
- `limit_buy/sell()` - Convenience methods for limit orders
- `get_positions()` - Retrieve current positions
- `cancel_order()` - Cancel pending order
- `monitor_order()` - Track order status

### AlpacaLogger Methods

- `log_alpaca_order()` - Record order execution
- `log_account_snapshot()` - Store account state
- `log_performance_metrics()` - Save performance data
- `get_performance_summary()` - Query historical performance
- `create_tables()` - Initialize database schema

## 🔧 Troubleshooting

### Common Issues

1. **API Authentication Errors**
   - Verify API keys are correct
   - Check if using paper vs live endpoints
   - Ensure account is active

2. **Market Data Issues**
   - Verify market is open
   - Check symbol format (e.g., "AAPL" not "AAPL.US")
   - Monitor rate limits

3. **Database Connection**
   - Verify TimescaleDB URL
   - Check network connectivity
   - Ensure proper permissions

4. **Performance Issues**
   - Monitor connection pool utilization
   - Check database query performance
   - Verify network latency

### Debug Mode

```bash
# Enable debug logging
export RUST_LOG=debug
cargo run --bin test_alpaca_integration
```

## 📞 Support

For technical support and questions:

1. **Documentation**: Review this guide and code comments
2. **Testing**: Run the comprehensive test suite
3. **Logs**: Check application logs for detailed error information
4. **Performance**: Monitor metrics and database queries

## 🔄 Updates and Maintenance

### Regular Maintenance

- Monitor API rate limits and usage
- Review and optimize database queries
- Update risk parameters based on performance
- Backup critical configuration and data
- Test disaster recovery procedures

### Performance Optimization

- Tune database connection pools
- Optimize query patterns
- Implement caching strategies
- Monitor and reduce latency
- Scale infrastructure as needed

---

**Note**: This integration uses Alpaca's paper trading environment by default. Always test thoroughly before enabling live trading with real capital.
