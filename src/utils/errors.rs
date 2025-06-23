use thiserror::Error;

#[derive(Debug, Error)]
pub enum PantherSwapError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Database migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
    
    #[error("Market data error: {0}")]
    MarketData(String),
    
    #[error("AI prediction error: {0}")]
    AIPrediction(String),
    
    #[error("Trading error: {0}")]
    Trading(String),
    
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),
    
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Risk management error: {0}")]
    RiskManagement(String),
    
    #[error("Microstructure analysis error: {0}")]
    Microstructure(String),
    
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl PantherSwapError {
    pub fn market_data(msg: impl Into<String>) -> Self {
        Self::MarketData(msg.into())
    }
    
    pub fn ai_prediction(msg: impl Into<String>) -> Self {
        Self::AIPrediction(msg.into())
    }
    
    pub fn trading(msg: impl Into<String>) -> Self {
        Self::Trading(msg.into())
    }
    
    pub fn risk_management(msg: impl Into<String>) -> Self {
        Self::RiskManagement(msg.into())
    }
    
    pub fn microstructure(msg: impl Into<String>) -> Self {
        Self::Microstructure(msg.into())
    }

    pub fn performance(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
    
    pub fn authentication(msg: impl Into<String>) -> Self {
        Self::Authentication(msg.into())
    }
    
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }
    
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}

// Validation error conversion temporarily disabled

pub type Result<T> = std::result::Result<T, PantherSwapError>;
