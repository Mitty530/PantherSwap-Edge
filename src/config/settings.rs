use serde::{Deserialize, Serialize};
use config::{Config, ConfigError, Environment, File};
use std::env;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Settings {
    #[serde(default)]
    pub debug: bool,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub market_data: MarketDataConfig,
    pub trading: TradingConfig,
    pub ai: AIConfig,
    pub risk: RiskConfig,
    pub api: APIConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout: u64,
    pub query_timeout: u64,
    pub idle_timeout: u64,
    pub max_lifetime: u64,
    pub enable_real_time_monitoring: bool,
    pub slow_query_threshold_ms: u64,
    pub data_retention_days: u32,
    pub enable_performance_metrics: bool,
    pub connection_pool_monitoring: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MarketDataConfig {
    pub providers: Vec<String>,
    #[serde(default = "default_primary_provider")]
    pub primary_provider: String,
    pub instruments: Vec<String>,
    pub update_interval_ms: u64,
    pub max_retries: u32,
    pub timeout_ms: u64,
    pub quality_threshold: f64,
    pub alpha_vantage_api_key: String,
    pub iex_cloud_api_key: String,
    pub alpaca: AlpacaConfig,
    pub ig_trading: IGTradingConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IGTradingConfig {
    pub api_key: String,
    pub username: String,
    pub password: String,
    pub security_token: String,
    pub cst: String,
    pub version: String,
    pub base_url: String,
    pub content_type: String,
    pub accept: String,
    pub demo_mode: bool,
    pub rate_limit_per_minute: u32,
    pub connection_timeout_ms: u64,
    pub retry_attempts: u32,
}

fn default_primary_provider() -> String {
    "ig_trading".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AlpacaConfig {
    pub api_key: String,
    pub secret_key: String,
    pub base_url: String,
    pub data_url: String,
    pub paper_trading: bool,
    pub enable_streaming: bool,
    pub max_positions: u32,
    pub max_order_value: f64,
    pub enable_fractional_shares: bool,
    #[serde(default = "default_rate_limit")]
    pub rate_limit_per_minute: u32,
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout_ms: u64,
    #[serde(default = "default_retry_attempts")]
    pub retry_attempts: u32,
    #[serde(default = "default_enable_order_execution")]
    pub enable_order_execution: bool,
}

fn default_rate_limit() -> u32 { 200 }
fn default_connection_timeout() -> u64 { 5000 }
fn default_retry_attempts() -> u32 { 3 }
fn default_enable_order_execution() -> bool { true }

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TradingConfig {
    pub enable_live_trading: bool,
    pub signal_generation_interval_ms: u64,
    pub risk_check_interval_ms: u64,
    pub max_position_size: f64,
    pub confidence_threshold: f64,
    pub strategy_types: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AIConfig {
    pub enable_predictions: bool,
    pub model_path: String,
    pub model_update_interval_hours: u64,
    pub prediction_horizons_minutes: Vec<u64>,
    pub enable_training: bool,
    pub training_batch_size: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RiskConfig {
    pub max_portfolio_var: f64,
    pub max_leverage: f64,
    pub confidence_threshold: f64,
    pub max_daily_loss: f64,
    pub drawdown_limit: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct APIConfig {
    pub rate_limit_per_minute: u32,
    pub enable_cors: bool,
    pub max_request_size: usize,
    pub api_key_header: String,
}

impl Settings {
    pub fn load() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
        
        let s = Config::builder()
            // Start with default configuration file
            .add_source(File::with_name("config/default").required(false))
            // Add environment-specific configuration
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            // Add local configuration (gitignored)
            .add_source(File::with_name("config/local").required(false))
            // Add environment variables with prefix PANTHERSWAP
            .add_source(Environment::with_prefix("PANTHERSWAP").separator("_"))
            .build()?;
        
        let settings: Settings = s.try_deserialize()?;
        settings.validate()?;
        Ok(settings)
    }
    
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate configuration values
        if self.database.url.is_empty() {
            return Err(ConfigError::Message("Database URL is required".into()));
        }
        
        // Skip API key validation for Phase 1
        // if self.market_data.alpha_vantage_api_key.is_empty() {
        //     return Err(ConfigError::Message("Alpha Vantage API key is required".into()));
        // }
        
        if self.trading.confidence_threshold < 0.0 || self.trading.confidence_threshold > 1.0 {
            return Err(ConfigError::Message("Confidence threshold must be between 0.0 and 1.0".into()));
        }
        
        if self.risk.max_portfolio_var <= 0.0 {
            return Err(ConfigError::Message("Max portfolio VaR must be positive".into()));
        }
        
        if self.server.port == 0 {
            return Err(ConfigError::Message("Server port must be specified".into()));
        }
        
        Ok(())
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            debug: false,
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                workers: 4,
            },
            database: DatabaseConfig {
                url: "postgres://tsdbadmin:sz2eu577bgqi5767@jqrbtbc5nw.w0mq2s13iy.tsdb.cloud.timescale.com:35762/tsdb?sslmode=require".to_string(),
                max_connections: 75,  // Scaled to target range for HFT performance
                min_connections: 15,  // 20% of max for baseline performance
                connection_timeout: 30,
                query_timeout: 30,
                idle_timeout: 300,
                max_lifetime: 1800,
                enable_real_time_monitoring: true,
                slow_query_threshold_ms: 1000,
                data_retention_days: 14,
                enable_performance_metrics: true,
                connection_pool_monitoring: true,
            },
            market_data: MarketDataConfig {
                providers: vec!["ig_trading".to_string(), "alpaca".to_string(), "alpha_vantage".to_string()],
                primary_provider: "ig_trading".to_string(),
                instruments: vec![
                    "AAPL".to_string(),
                    "MSFT".to_string(),
                    "GOOGL".to_string(),
                    "TSLA".to_string(),
                    "NVDA".to_string(),
                    "SPY".to_string(),
                    "QQQ".to_string(),
                ],
                update_interval_ms: 1000, // 1 second for real-time trading
                max_retries: 3,
                timeout_ms: 5000,
                quality_threshold: 0.9,
                alpha_vantage_api_key: String::new(),
                iex_cloud_api_key: String::new(),
                alpaca: AlpacaConfig {
                    api_key: String::new(),
                    secret_key: String::new(),
                    base_url: "https://paper-api.alpaca.markets".to_string(), // Paper trading by default
                    data_url: "https://data.alpaca.markets".to_string(),
                    paper_trading: true,
                    enable_streaming: true,
                    max_positions: 100,
                    max_order_value: 100000.0,
                    enable_fractional_shares: true,
                    rate_limit_per_minute: 200,
                    connection_timeout_ms: 5000,
                    retry_attempts: 3,
                    enable_order_execution: true,
                },
                ig_trading: IGTradingConfig {
                    api_key: std::env::var("IG_TRADING_API_KEY").unwrap_or_else(|_| "3ded3ba7db96187488bf8773b86bdf3e8fc42e9b".to_string()),
                    username: std::env::var("IG_TRADING_USERNAME").unwrap_or_default(),
                    password: std::env::var("IG_TRADING_PASSWORD").unwrap_or_default(),
                    security_token: std::env::var("IG_TRADING_SECURITY_TOKEN").unwrap_or_else(|_| "1206a1630c34bcc90fdcc1b62fc5920fa7ed3a216ae09933430d3de2c6bcf6CD01112".to_string()),
                    cst: std::env::var("IG_TRADING_CST").unwrap_or_else(|_| "48417021199921da08b95b210d8f9492c36614232983a9f1f3e1a8f0748ce33CC01113".to_string()),
                    version: "2".to_string(),
                    base_url: "https://demo-api.ig.com/gateway/deal".to_string(),
                    content_type: "application/json; charset=UTF-8".to_string(),
                    accept: "application/json; charset=UTF-8".to_string(),
                    demo_mode: true,
                    rate_limit_per_minute: 100,
                    connection_timeout_ms: 5000,
                    retry_attempts: 3,
                },
            },
            trading: TradingConfig {
                enable_live_trading: false, // Paper trading for MVP
                signal_generation_interval_ms: 1000,
                risk_check_interval_ms: 500,
                max_position_size: 100000.0,
                confidence_threshold: 0.7,
                strategy_types: vec![
                    "predictive_market_making".to_string(),
                    "microstructure_momentum".to_string(),
                    "regime_arbitrage".to_string(),
                    "liquidity_harvesting".to_string(),
                ],
            },
            ai: AIConfig {
                enable_predictions: true,
                model_path: "./models/".to_string(),
                model_update_interval_hours: 24,
                prediction_horizons_minutes: vec![1, 5, 15, 60],
                enable_training: false, // Disable for MVP
                training_batch_size: 64,
            },
            risk: RiskConfig {
                max_portfolio_var: 0.02, // 2%
                max_leverage: 3.0,
                confidence_threshold: 0.7,
                max_daily_loss: 50000.0,
                drawdown_limit: 0.1, // 10%
            },
            api: APIConfig {
                rate_limit_per_minute: 1000,
                enable_cors: true,
                max_request_size: 1024 * 1024, // 1MB
                api_key_header: "X-API-Key".to_string(),
            },
        }
    }
}
