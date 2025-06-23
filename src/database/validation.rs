// Data validation layer for PantherSwap Edge trading platform
// Provides comprehensive validation for market data, trading signals, AI predictions, and data integrity

use crate::utils::Result;
use crate::database::types::*;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use serde_json::Value;
use std::collections::HashMap;
use tracing::{warn, error, info};

/// Validation error types for different data validation failures
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid price: {message}")]
    InvalidPrice { message: String },
    
    #[error("Invalid spread: {message}")]
    InvalidSpread { message: String },
    
    #[error("Invalid timestamp: {message}")]
    InvalidTimestamp { message: String },
    
    #[error("Invalid volume: {message}")]
    InvalidVolume { message: String },
    
    #[error("Data quality too low: {score}, minimum required: {threshold}")]
    DataQualityTooLow { score: f64, threshold: f64 },
    
    #[error("Invalid confidence score: {score}")]
    InvalidConfidence { score: f64 },
    
    #[error("Invalid risk score: {score}")]
    InvalidRisk { score: f64 },
    
    #[error("Missing required field: {field}")]
    MissingField { field: String },
    
    #[error("Invalid range: {field} value {value} not in range [{min}, {max}]")]
    InvalidRange { field: String, value: f64, min: f64, max: f64 },
    
    #[error("Constraint violation: {constraint}")]
    ConstraintViolation { constraint: String },
    
    #[error("Data integrity check failed: {message}")]
    DataIntegrityFailure { message: String },
}

/// Validation configuration for different data types
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    pub market_data: MarketDataValidationConfig,
    pub trading_signals: TradingSignalValidationConfig,
    pub ai_predictions: AIPredictionValidationConfig,
    pub general: GeneralValidationConfig,
}

#[derive(Debug, Clone)]
pub struct MarketDataValidationConfig {
    pub min_price: f64,
    pub max_price: f64,
    pub max_spread_percentage: f64,
    pub max_age_minutes: i64,
    pub min_data_quality: f64,
    pub max_volume: f64,
    pub required_providers: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TradingSignalValidationConfig {
    pub min_confidence: f64,
    pub max_confidence: f64,
    pub min_risk_score: f64,
    pub max_risk_score: f64,
    pub max_position_size: f64,
    pub allowed_signal_types: Vec<String>,
    pub allowed_strategy_types: Vec<String>,
    pub max_time_horizon_hours: i64,
}

#[derive(Debug, Clone)]
pub struct AIPredictionValidationConfig {
    pub min_confidence: f64,
    pub max_confidence: f64,
    pub allowed_model_types: Vec<String>,
    pub max_prediction_horizon_minutes: i32,
    pub min_prediction_horizon_minutes: i32,
    pub max_predicted_volatility: f64,
}

#[derive(Debug, Clone)]
pub struct GeneralValidationConfig {
    pub max_future_timestamp_minutes: i64,
    pub max_past_timestamp_minutes: i64,
    pub required_json_fields: HashMap<String, Vec<String>>,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            market_data: MarketDataValidationConfig {
                min_price: 0.0001,
                max_price: 1_000_000.0,
                max_spread_percentage: 10.0, // 10%
                max_age_minutes: 30,
                min_data_quality: 0.5,
                max_volume: 1_000_000_000.0,
                required_providers: vec!["alpha_vantage".to_string()],
            },
            trading_signals: TradingSignalValidationConfig {
                min_confidence: 0.0,
                max_confidence: 1.0,
                min_risk_score: 0.0,
                max_risk_score: 1.0,
                max_position_size: 1_000_000.0,
                allowed_signal_types: vec![
                    "BUY".to_string(),
                    "SELL".to_string(),
                    "HOLD".to_string(),
                ],
                allowed_strategy_types: vec![
                    "predictive_market_making".to_string(),
                    "microstructure_momentum".to_string(),
                    "regime_arbitrage".to_string(),
                    "liquidity_harvesting".to_string(),
                    "momentum".to_string(),
                    "arbitrage".to_string(),
                    "market_making".to_string(),
                ],
                max_time_horizon_hours: 168, // 1 week
            },
            ai_predictions: AIPredictionValidationConfig {
                min_confidence: 0.0,
                max_confidence: 1.0,
                allowed_model_types: vec![
                    "lstm".to_string(),
                    "transformer".to_string(),
                    "random_forest".to_string(),
                    "xgboost".to_string(),
                ],
                max_prediction_horizon_minutes: 1440, // 24 hours
                min_prediction_horizon_minutes: 1,
                max_predicted_volatility: 1.0, // 100%
            },
            general: GeneralValidationConfig {
                max_future_timestamp_minutes: 5,
                max_past_timestamp_minutes: 60,
                required_json_fields: HashMap::new(),
            },
        }
    }
}

/// Main data validator for the trading platform
pub struct DataValidator {
    config: ValidationConfig,
    validation_stats: ValidationStats,
}

#[derive(Debug, Default)]
pub struct ValidationStats {
    pub total_validations: u64,
    pub successful_validations: u64,
    pub failed_validations: u64,
    pub validation_errors: HashMap<String, u64>,
}

impl DataValidator {
    pub fn new(config: ValidationConfig) -> Self {
        Self {
            config,
            validation_stats: ValidationStats::default(),
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(ValidationConfig::default())
    }

    /// Validate market tick data
    pub fn validate_market_tick(&mut self, tick: &MarketTick) -> Result<()> {
        self.validation_stats.total_validations += 1;

        // Validate prices
        self.validate_prices(tick)?;
        
        // Validate spread
        self.validate_spread(tick)?;
        
        // Validate timestamp
        self.validate_timestamp(tick.timestamp, "market_tick")?;
        
        // Validate volumes and sizes
        self.validate_volumes_and_sizes(tick)?;
        
        // Validate data quality
        self.validate_data_quality(tick.data_quality_score)?;
        
        // Validate provider
        self.validate_provider(&tick.provider)?;
        
        // Validate JSON metadata
        self.validate_json_metadata(&tick.raw_data, "market_tick")?;

        self.validation_stats.successful_validations += 1;
        Ok(())
    }

    /// Validate trading signal data
    pub fn validate_trading_signal(&mut self, signal: &TradingSignal) -> Result<()> {
        self.validation_stats.total_validations += 1;

        // Validate confidence score
        self.validate_confidence_score(signal.confidence_score)?;
        
        // Validate risk score
        self.validate_risk_score(signal.risk_score)?;
        
        // Validate signal type
        self.validate_signal_type(&signal.signal_type)?;
        
        // Validate strategy type
        self.validate_strategy_type(&signal.strategy_type)?;
        
        // Validate position size
        self.validate_position_size(signal.position_size)?;
        
        // Validate timestamp
        self.validate_timestamp(signal.timestamp, "trading_signal")?;
        
        // Validate price levels
        self.validate_price_levels(signal)?;
        
        // Validate time horizon
        if let Some(horizon) = signal.time_horizon {
            self.validate_time_horizon(horizon)?;
        }
        
        // Validate JSON metadata
        self.validate_json_metadata(&signal.metadata, "trading_signal")?;

        self.validation_stats.successful_validations += 1;
        Ok(())
    }

    /// Validate AI prediction data
    pub fn validate_ai_prediction(&mut self, prediction: &AIPrediction) -> Result<()> {
        self.validation_stats.total_validations += 1;

        // Validate confidence score
        self.validate_confidence_score(prediction.confidence_score)?;
        
        // Validate model type
        self.validate_model_type(&prediction.model_type)?;
        
        // Validate prediction horizon
        self.validate_prediction_horizon(prediction.prediction_horizon_minutes)?;
        
        // Validate predicted price
        self.validate_predicted_price(prediction.predicted_price)?;
        
        // Validate predicted volatility
        if let Some(volatility) = prediction.predicted_volatility {
            self.validate_predicted_volatility(volatility)?;
        }
        
        // Validate timestamp
        self.validate_timestamp(prediction.timestamp, "ai_prediction")?;
        
        // Validate JSON fields
        if let Some(intervals) = &prediction.prediction_intervals {
            self.validate_json_metadata(intervals, "prediction_intervals")?;
        }
        
        if let Some(importance) = &prediction.feature_importance {
            self.validate_json_metadata(importance, "feature_importance")?;
        }

        self.validation_stats.successful_validations += 1;
        Ok(())
    }

    /// Validate instrument data
    pub fn validate_instrument(&mut self, instrument: &Instrument) -> Result<()> {
        self.validation_stats.total_validations += 1;

        // Validate symbol format
        if instrument.symbol.is_empty() || instrument.symbol.len() > 20 {
            return Err(ValidationError::ConstraintViolation {
                constraint: "Symbol must be 1-20 characters".to_string(),
            });
        }

        // Validate name
        if instrument.name.is_empty() || instrument.name.len() > 100 {
            return Err(ValidationError::ConstraintViolation {
                constraint: "Name must be 1-100 characters".to_string(),
            });
        }

        // Validate tick and lot sizes
        if instrument.tick_size <= 0.0 {
            return Err(ValidationError::InvalidPrice {
                message: "Tick size must be positive".to_string(),
            });
        }

        if instrument.lot_size <= 0.0 {
            return Err(ValidationError::InvalidVolume {
                message: "Lot size must be positive".to_string(),
            });
        }

        self.validation_stats.successful_validations += 1;
        Ok(())
    }

    // Private validation helper methods
    
    fn validate_prices(&self, tick: &MarketTick) -> Result<(), ValidationError> {
        let config = &self.config.market_data;
        
        if tick.bid_price <= 0.0 || tick.ask_price <= 0.0 {
            return Err(ValidationError::InvalidPrice {
                message: "Bid and ask prices must be positive".to_string(),
            });
        }
        
        if tick.bid_price < config.min_price || tick.ask_price < config.min_price {
            return Err(ValidationError::InvalidPrice {
                message: format!("Prices below minimum: {}", config.min_price),
            });
        }
        
        if tick.bid_price > config.max_price || tick.ask_price > config.max_price {
            return Err(ValidationError::InvalidPrice {
                message: format!("Prices above maximum: {}", config.max_price),
            });
        }
        
        if let Some(last_price) = tick.last_price {
            if last_price <= 0.0 || last_price < config.min_price || last_price > config.max_price {
                return Err(ValidationError::InvalidPrice {
                    message: "Invalid last price".to_string(),
                });
            }
        }
        
        Ok(())
    }
    
    fn validate_spread(&self, tick: &MarketTick) -> Result<(), ValidationError> {
        if tick.ask_price <= tick.bid_price {
            return Err(ValidationError::InvalidSpread {
                message: "Ask price must be greater than bid price".to_string(),
            });
        }
        
        let spread_percentage = ((tick.ask_price - tick.bid_price) / tick.bid_price) * 100.0;
        if spread_percentage > self.config.market_data.max_spread_percentage {
            return Err(ValidationError::InvalidSpread {
                message: format!("Spread too wide: {:.2}%", spread_percentage),
            });
        }
        
        Ok(())
    }
    
    fn validate_timestamp(&self, timestamp: DateTime<Utc>, context: &str) -> Result<(), ValidationError> {
        let now = Utc::now();
        let age = now.signed_duration_since(timestamp);
        let future_diff = timestamp.signed_duration_since(now);
        
        if age > Duration::minutes(self.config.general.max_past_timestamp_minutes) {
            return Err(ValidationError::InvalidTimestamp {
                message: format!("{} timestamp too old: {} minutes", context, age.num_minutes()),
            });
        }
        
        if future_diff > Duration::minutes(self.config.general.max_future_timestamp_minutes) {
            return Err(ValidationError::InvalidTimestamp {
                message: format!("{} timestamp too far in future: {} minutes", context, future_diff.num_minutes()),
            });
        }
        
        Ok(())
    }
    
    fn validate_volumes_and_sizes(&self, tick: &MarketTick) -> Result<(), ValidationError> {
        if tick.bid_size < 0.0 || tick.ask_size < 0.0 {
            return Err(ValidationError::InvalidVolume {
                message: "Bid and ask sizes must be non-negative".to_string(),
            });
        }
        
        if let Some(volume) = tick.volume {
            if volume < 0.0 {
                return Err(ValidationError::InvalidVolume {
                    message: "Volume must be non-negative".to_string(),
                });
            }
            
            if volume > self.config.market_data.max_volume {
                return Err(ValidationError::InvalidVolume {
                    message: format!("Volume exceeds maximum: {}", self.config.market_data.max_volume),
                });
            }
        }
        
        Ok(())
    }
    
    fn validate_data_quality(&self, score: f64) -> Result<(), ValidationError> {
        if score < 0.0 || score > 1.0 {
            return Err(ValidationError::InvalidRange {
                field: "data_quality_score".to_string(),
                value: score,
                min: 0.0,
                max: 1.0,
            });
        }
        
        if score < self.config.market_data.min_data_quality {
            return Err(ValidationError::DataQualityTooLow {
                score,
                threshold: self.config.market_data.min_data_quality,
            });
        }
        
        Ok(())
    }
    
    fn validate_provider(&self, provider: &str) -> Result<(), ValidationError> {
        if provider.is_empty() {
            return Err(ValidationError::MissingField {
                field: "provider".to_string(),
            });
        }
        
        // Note: In production, you might want to validate against allowed providers
        Ok(())
    }
    
    fn validate_confidence_score(&self, score: f64) -> Result<(), ValidationError> {
        if score < 0.0 || score > 1.0 {
            return Err(ValidationError::InvalidConfidence { score });
        }
        Ok(())
    }
    
    fn validate_risk_score(&self, score: f64) -> Result<(), ValidationError> {
        if score < 0.0 || score > 1.0 {
            return Err(ValidationError::InvalidRisk { score });
        }
        Ok(())
    }
    
    fn validate_signal_type(&self, signal_type: &str) -> Result<(), ValidationError> {
        if !self.config.trading_signals.allowed_signal_types.contains(&signal_type.to_string()) {
            return Err(ValidationError::ConstraintViolation {
                constraint: format!("Invalid signal type: {}", signal_type),
            });
        }
        Ok(())
    }
    
    fn validate_strategy_type(&self, strategy_type: &str) -> Result<(), ValidationError> {
        if !self.config.trading_signals.allowed_strategy_types.contains(&strategy_type.to_string()) {
            return Err(ValidationError::ConstraintViolation {
                constraint: format!("Invalid strategy type: {}", strategy_type),
            });
        }
        Ok(())
    }
    
    fn validate_position_size(&self, size: f64) -> Result<(), ValidationError> {
        if size <= 0.0 {
            return Err(ValidationError::InvalidVolume {
                message: "Position size must be positive".to_string(),
            });
        }
        
        if size > self.config.trading_signals.max_position_size {
            return Err(ValidationError::InvalidVolume {
                message: format!("Position size exceeds maximum: {}", self.config.trading_signals.max_position_size),
            });
        }
        
        Ok(())
    }
    
    fn validate_price_levels(&self, signal: &TradingSignal) -> Result<(), ValidationError> {
        if let Some(target) = signal.target_price {
            if target <= 0.0 {
                return Err(ValidationError::InvalidPrice {
                    message: "Target price must be positive".to_string(),
                });
            }
        }
        
        if let Some(stop_loss) = signal.stop_loss {
            if stop_loss <= 0.0 {
                return Err(ValidationError::InvalidPrice {
                    message: "Stop loss must be positive".to_string(),
                });
            }
        }
        
        if let Some(take_profit) = signal.take_profit {
            if take_profit <= 0.0 {
                return Err(ValidationError::InvalidPrice {
                    message: "Take profit must be positive".to_string(),
                });
            }
        }
        
        // Validate price level relationships for BUY signals
        if signal.signal_type == "BUY" {
            if let (Some(target), Some(stop_loss)) = (signal.target_price, signal.stop_loss) {
                if target <= stop_loss {
                    return Err(ValidationError::ConstraintViolation {
                        constraint: "For BUY signals, target price must be above stop loss".to_string(),
                    });
                }
            }
        }
        
        // Validate price level relationships for SELL signals
        if signal.signal_type == "SELL" {
            if let (Some(target), Some(stop_loss)) = (signal.target_price, signal.stop_loss) {
                if target >= stop_loss {
                    return Err(ValidationError::ConstraintViolation {
                        constraint: "For SELL signals, target price must be below stop loss".to_string(),
                    });
                }
            }
        }
        
        Ok(())
    }
    
    fn validate_time_horizon(&self, horizon: chrono::Duration) -> Result<(), ValidationError> {
        let hours = horizon.num_hours();
        if hours <= 0 || hours > self.config.trading_signals.max_time_horizon_hours {
            return Err(ValidationError::InvalidRange {
                field: "time_horizon".to_string(),
                value: hours as f64,
                min: 0.0,
                max: self.config.trading_signals.max_time_horizon_hours as f64,
            });
        }
        Ok(())
    }
    
    fn validate_model_type(&self, model_type: &str) -> Result<(), ValidationError> {
        if !self.config.ai_predictions.allowed_model_types.contains(&model_type.to_string()) {
            return Err(ValidationError::ConstraintViolation {
                constraint: format!("Invalid model type: {}", model_type),
            });
        }
        Ok(())
    }
    
    fn validate_prediction_horizon(&self, horizon_minutes: i32) -> Result<(), ValidationError> {
        let config = &self.config.ai_predictions;
        if horizon_minutes < config.min_prediction_horizon_minutes || horizon_minutes > config.max_prediction_horizon_minutes {
            return Err(ValidationError::InvalidRange {
                field: "prediction_horizon_minutes".to_string(),
                value: horizon_minutes as f64,
                min: config.min_prediction_horizon_minutes as f64,
                max: config.max_prediction_horizon_minutes as f64,
            });
        }
        Ok(())
    }
    
    fn validate_predicted_price(&self, price: f64) -> Result<(), ValidationError> {
        if price <= 0.0 {
            return Err(ValidationError::InvalidPrice {
                message: "Predicted price must be positive".to_string(),
            });
        }
        
        let config = &self.config.market_data;
        if price < config.min_price || price > config.max_price {
            return Err(ValidationError::InvalidPrice {
                message: format!("Predicted price out of range: [{}, {}]", config.min_price, config.max_price),
            });
        }
        
        Ok(())
    }
    
    fn validate_predicted_volatility(&self, volatility: f64) -> Result<(), ValidationError> {
        if volatility < 0.0 || volatility > self.config.ai_predictions.max_predicted_volatility {
            return Err(ValidationError::InvalidRange {
                field: "predicted_volatility".to_string(),
                value: volatility,
                min: 0.0,
                max: self.config.ai_predictions.max_predicted_volatility,
            });
        }
        Ok(())
    }
    
    fn validate_json_metadata(&self, json_data: &Value, context: &str) -> Result<(), ValidationError> {
        // Basic JSON structure validation
        if json_data.is_null() {
            return Err(ValidationError::MissingField {
                field: format!("{}_metadata", context),
            });
        }
        
        // Check for required fields if configured
        if let Some(required_fields) = self.config.general.required_json_fields.get(context) {
            for field in required_fields {
                if json_data.get(field).is_none() {
                    return Err(ValidationError::MissingField {
                        field: format!("{}.{}", context, field),
                    });
                }
            }
        }
        
        Ok(())
    }

    /// Get validation statistics
    pub fn get_stats(&self) -> &ValidationStats {
        &self.validation_stats
    }

    /// Reset validation statistics
    pub fn reset_stats(&mut self) {
        self.validation_stats = ValidationStats::default();
    }

    /// Record validation error for statistics
    fn record_validation_error(&mut self, error: &ValidationError) {
        self.validation_stats.failed_validations += 1;
        let error_type = match error {
            ValidationError::InvalidPrice { .. } => "invalid_price",
            ValidationError::InvalidSpread { .. } => "invalid_spread",
            ValidationError::InvalidTimestamp { .. } => "invalid_timestamp",
            ValidationError::InvalidVolume { .. } => "invalid_volume",
            ValidationError::DataQualityTooLow { .. } => "data_quality_too_low",
            ValidationError::InvalidConfidence { .. } => "invalid_confidence",
            ValidationError::InvalidRisk { .. } => "invalid_risk",
            ValidationError::MissingField { .. } => "missing_field",
            ValidationError::InvalidRange { .. } => "invalid_range",
            ValidationError::ConstraintViolation { .. } => "constraint_violation",
            ValidationError::DataIntegrityFailure { .. } => "data_integrity_failure",
        };

        *self.validation_stats.validation_errors.entry(error_type.to_string()).or_insert(0) += 1;
    }

    /// Validate data with error recording
    pub fn validate_with_stats<T, F>(&mut self, data: &T, validator: F) -> Result<(), ValidationError>
    where
        F: FnOnce(&mut Self, &T) -> Result<(), ValidationError>,
    {
        match validator(self, data) {
            Ok(()) => Ok(()),
            Err(error) => {
                self.record_validation_error(&error);
                Err(error)
            }
        }
    }
}
