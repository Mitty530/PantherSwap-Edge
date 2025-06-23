use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sqlx::{FromRow, Type};
use rust_decimal::Decimal;

// Trading Signal Types and Enums
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type, Default)]
#[sqlx(type_name = "text")]
pub enum SignalType {
    #[default]
    Buy,
    Sell,
    Hold,
    AI, // Added missing AI variant
}

impl std::fmt::Display for SignalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignalType::Buy => write!(f, "BUY"),
            SignalType::Sell => write!(f, "SELL"),
            SignalType::Hold => write!(f, "HOLD"),
            SignalType::AI => write!(f, "AI"),
        }
    }
}

impl std::str::FromStr for SignalType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "BUY" => Ok(SignalType::Buy),
            "SELL" => Ok(SignalType::Sell),
            "HOLD" => Ok(SignalType::Hold),
            _ => Err(format!("Invalid signal type: {}", s)),
        }
    }
}



#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Type)]
#[sqlx(type_name = "text")]
pub enum RegimeType {
    Normal,
    Trending,
    Volatile,
    Crisis,
    Bullish,
    Bearish,
    Sideways,
    HighVolatility,
}

impl std::fmt::Display for RegimeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegimeType::Normal => write!(f, "normal"),
            RegimeType::Trending => write!(f, "trending"),
            RegimeType::Volatile => write!(f, "volatile"),
            RegimeType::Crisis => write!(f, "crisis"),
            RegimeType::Bullish => write!(f, "bullish"),
            RegimeType::Bearish => write!(f, "bearish"),
            RegimeType::Sideways => write!(f, "sideways"),
            RegimeType::HighVolatility => write!(f, "high_volatility"),
        }
    }
}

impl std::str::FromStr for RegimeType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "normal" => Ok(RegimeType::Normal),
            "trending" => Ok(RegimeType::Trending),
            "volatile" => Ok(RegimeType::Volatile),
            "crisis" => Ok(RegimeType::Crisis),
            _ => Err(format!("Invalid regime type: {}", s)),
        }
    }
}



#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum OrderType {
    #[default]
    Market,
    Limit,
    Stop,
    StopLimit,
}

impl std::fmt::Display for OrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderType::Market => write!(f, "Market"),
            OrderType::Limit => write!(f, "Limit"),
            OrderType::Stop => write!(f, "Stop"),
            OrderType::StopLimit => write!(f, "StopLimit"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum TimeInForce {
    #[default]
    GTC, // Good Till Canceled
    IOC, // Immediate Or Cancel
    FOK, // Fill Or Kill
    DAY, // Day Order
}

impl std::fmt::Display for TimeInForce {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeInForce::GTC => write!(f, "GTC"),
            TimeInForce::IOC => write!(f, "IOC"),
            TimeInForce::FOK => write!(f, "FOK"),
            TimeInForce::DAY => write!(f, "DAY"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionStyle {
    Aggressive,    // Execute immediately
    Passive,       // Wait for better prices
    Iceberg,       // Hidden size execution
    TWAP,          // Time-weighted average price
}

impl std::fmt::Display for ExecutionStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionStyle::Aggressive => write!(f, "Aggressive"),
            ExecutionStyle::Passive => write!(f, "Passive"),
            ExecutionStyle::Iceberg => write!(f, "Iceberg"),
            ExecutionStyle::TWAP => write!(f, "TWAP"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MarketTick {
    pub timestamp: DateTime<Utc>,
    pub instrument_id: Uuid,
    pub provider: String,
    pub bid_price: f64,
    pub ask_price: f64,
    pub bid_size: f64,
    pub ask_size: f64,
    pub last_price: Option<f64>,
    pub volume: Option<f64>,
    pub spread: f64,
    pub data_quality_score: f64,
    pub raw_data: serde_json::Value,
    // Backward compatibility fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>, // Alias for last_price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bid: Option<f64>, // Alias for bid_price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ask: Option<f64>, // Alias for ask_price
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Instrument {
    pub id: Uuid,
    pub symbol: String,
    pub name: String,
    pub instrument_type: String,
    pub base_currency: String,
    pub quote_currency: String,
    pub tick_size: f64,
    pub lot_size: f64,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TradingSignal {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub instrument_id: Uuid,
    pub strategy_name: String,
    pub signal_type: SignalType,
    pub signal_strength: Decimal,
    pub confidence_score: Decimal,
    pub recommended_size: Decimal,
    pub entry_price: Option<Decimal>,
    pub stop_loss: Option<Decimal>,
    pub take_profit: Option<Decimal>,
    pub time_horizon: Option<chrono::Duration>,
    pub expected_return: Option<Decimal>,
    pub risk_metrics: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AIPrediction {
    pub timestamp: DateTime<Utc>,
    pub instrument_id: Uuid,
    pub model_type: String,
    pub model_version: String,
    pub prediction_horizon_minutes: i32,
    pub predicted_price: f64,
    pub predicted_volatility: Option<f64>,
    pub confidence_score: f64,
    pub prediction_intervals: Option<serde_json::Value>,
    pub feature_importance: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MicrostructureAnalysis {
    pub timestamp: DateTime<Utc>,
    pub instrument_id: Uuid,
    pub order_book_imbalance: f64,
    pub bid_ask_spread: f64,
    pub market_depth: f64,
    pub price_impact: f64,
    pub liquidity_score: f64,
    pub volatility_regime: String,
    pub market_maker_presence: f64,
    pub analysis_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RiskMetrics {
    pub timestamp: DateTime<Utc>,
    pub instrument_id: Option<Uuid>,
    pub portfolio_var: f64,
    pub position_size: f64,
    pub leverage: f64,
    pub drawdown: f64,
    pub sharpe_ratio: Option<f64>,
    pub max_loss_24h: f64,
    pub risk_score: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OrderBookSnapshot {
    pub timestamp: DateTime<Utc>,
    pub instrument_id: Uuid,
    pub venue: String,
    pub side: String, // 'bid' or 'ask'
    pub price: f64,
    pub quantity: f64,
    pub order_count: Option<i32>,
    pub market_maker_id: Option<String>,
    pub order_type: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TradeExecution {
    pub timestamp: DateTime<Utc>,
    pub instrument_id: Uuid,
    pub signal_id: Option<Uuid>,
    pub action: String, // BUY, SELL
    pub quantity: f64,
    pub price: f64,
    pub execution_time_ms: Option<i32>,
    pub slippage_bps: Option<f64>,
    pub fees: Option<f64>,
    pub pnl: Option<f64>,
    pub confidence_score: Option<f64>,
    pub strategy_name: Option<String>,
    pub created_at: DateTime<Utc>,
}
