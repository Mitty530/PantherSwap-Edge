pub mod time_series;
pub mod models;
pub mod inference;
pub mod rl_agent;
pub mod hmm_regime;
pub mod hmm_performance_validator;
pub mod hmm_integration;
pub mod monitoring;
pub mod optimization;

#[cfg(test)]
mod hmm_regime_test;

use crate::database::{Database, types::MarketTick};
use crate::utils::Result;
use crate::trading::signals::{AISignal, PredictionResult, RegimeSignal};
use crate::database::types::RegimeType;
use self::time_series::{LSTMTimeSeriesModel, create_forex_lstm_model};
use self::rl_agent::RLTradingAgent;
use self::hmm_regime::{HMMRegimeDetector, create_hmm_regime_detector};
use uuid::Uuid;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use std::time::Duration;
use tracing::{info, warn};
use serde::{Serialize, Deserialize};

/// Cached prediction for performance optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedPrediction {
    pub prediction: AISignal,
    pub timestamp: DateTime<Utc>,
    pub ttl_seconds: u64,
}

impl CachedPrediction {
    pub fn is_expired(&self) -> bool {
        let age = Utc::now() - self.timestamp;
        age.num_seconds() > self.ttl_seconds as i64
    }
}

/// Inference optimizer for <100ms latency requirements
#[derive(Debug, Clone)]
pub struct InferenceOptimizer {
    pub enable_caching: bool,
    pub cache_ttl_seconds: u64,
    pub enable_parallel_processing: bool,
    pub max_concurrent_inferences: usize,
    pub enable_batch_processing: bool,
    pub batch_timeout_ms: u64,
}

impl Default for InferenceOptimizer {
    fn default() -> Self {
        Self {
            enable_caching: true,
            cache_ttl_seconds: 5, // 5 second cache for real-time trading
            enable_parallel_processing: true,
            max_concurrent_inferences: 8,
            enable_batch_processing: true,
            batch_timeout_ms: 10, // 10ms batch timeout for low latency
        }
    }
}

/// Enhanced AI Engine with production ML models and database persistence
#[derive(Clone)]
pub struct AIEngine {
    database: Database,
    time_series_models: HashMap<Uuid, LSTMTimeSeriesModel>,
    regime_detectors: HashMap<Uuid, HMMRegimeDetector>,
    rl_agents: HashMap<Uuid, RLTradingAgent>,
    performance_metrics: AIPerformanceMetrics,
    prediction_cache: HashMap<Uuid, CachedPrediction>,
    enable_database_persistence: bool,
    inference_optimizer: InferenceOptimizer,
}



/// AI performance tracking
#[derive(Debug, Default, Clone)]
pub struct AIPerformanceMetrics {
    pub total_predictions: u64,
    pub successful_predictions: u64,
    pub average_latency_ms: f64,
    pub last_update: Option<DateTime<Utc>>,
}

impl AIEngine {
    pub async fn new(database: Database) -> Result<Self> {
        info!("Initializing AI Engine with production ML models and database persistence");

        Ok(Self {
            database,
            time_series_models: HashMap::new(),
            regime_detectors: HashMap::new(),
            rl_agents: HashMap::new(),
            performance_metrics: AIPerformanceMetrics::default(),
            prediction_cache: HashMap::new(),
            enable_database_persistence: true,
            inference_optimizer: InferenceOptimizer::default(),
        })
    }

    /// Create AI engine with custom inference optimization settings
    pub async fn new_with_optimization(database: Database, optimizer: InferenceOptimizer) -> Result<Self> {
        info!("Initializing optimized AI Engine for <100ms inference");

        Ok(Self {
            database,
            time_series_models: HashMap::new(),
            regime_detectors: HashMap::new(),
            rl_agents: HashMap::new(),
            performance_metrics: AIPerformanceMetrics::default(),
            prediction_cache: HashMap::new(),
            enable_database_persistence: true,
            inference_optimizer: optimizer,
        })
    }

    /// Enable or disable database persistence
    pub fn set_database_persistence(&mut self, enabled: bool) {
        self.enable_database_persistence = enabled;
        info!("Database persistence {}", if enabled { "enabled" } else { "disabled" });
    }

    /// Initialize AI models for a specific instrument
    pub async fn initialize_instrument(&mut self, instrument_id: Uuid) -> Result<()> {
        info!("Initializing AI models for instrument: {}", instrument_id);

        // Create LSTM time series model
        let lstm_model = create_forex_lstm_model()?;
        self.time_series_models.insert(instrument_id, lstm_model);

        // Create regime detector
        let regime_detector = create_hmm_regime_detector();
        self.regime_detectors.insert(instrument_id, regime_detector);

        info!("AI models initialized for instrument: {}", instrument_id);
        Ok(())
    }

    /// Process market data and generate AI signals with <100ms latency optimization
    pub async fn process_market_data(&mut self, ticks: &[MarketTick]) -> Result<Vec<AISignal>> {
        let start_time = std::time::Instant::now();
        let mut signals = Vec::new();

        // Check cache first if enabled
        if self.inference_optimizer.enable_caching {
            for tick in ticks {
                if let Some(cached) = self.prediction_cache.get(&tick.instrument_id) {
                    if !cached.is_expired() {
                        signals.push(cached.prediction.clone());
                        continue;
                    }
                }
            }
        }

        // Group ticks by instrument for batch processing
        let mut ticks_by_instrument: HashMap<Uuid, Vec<&MarketTick>> = HashMap::new();
        for tick in ticks {
            // Skip if we already have a cached result
            if self.inference_optimizer.enable_caching &&
               self.prediction_cache.get(&tick.instrument_id)
                   .map_or(false, |cached| !cached.is_expired()) {
                continue;
            }
            ticks_by_instrument.entry(tick.instrument_id).or_default().push(tick);
        }

        // Process instruments in parallel if enabled
        if self.inference_optimizer.enable_parallel_processing && ticks_by_instrument.len() > 1 {
            let mut tasks = Vec::new();

            for (instrument_id, instrument_ticks) in ticks_by_instrument {
                let owned_ticks: Vec<MarketTick> = instrument_ticks.iter().map(|&t| t.clone()).collect();
                // Note: In a real implementation, we'd use async tasks here
                // For now, process sequentially but with optimizations
                if let Ok(signal) = self.process_instrument_data_optimized(instrument_id, &owned_ticks).await {
                    signals.push(signal);
                }
            }
        } else {
            // Sequential processing with optimizations
            for (instrument_id, instrument_ticks) in ticks_by_instrument {
                let owned_ticks: Vec<MarketTick> = instrument_ticks.iter().map(|&t| t.clone()).collect();
                if let Ok(signal) = self.process_instrument_data_optimized(instrument_id, &owned_ticks).await {
                    signals.push(signal);
                }
            }
        }

        // Store predictions in database if enabled
        if self.enable_database_persistence && !signals.is_empty() {
            if let Err(e) = self.store_predictions_in_database(&signals).await {
                warn!("Failed to store AI predictions in database: {}", e);
            }
        }

        // Update performance metrics
        let latency_ms = start_time.elapsed().as_millis() as f64;
        self.update_performance_metrics(signals.len(), latency_ms);

        if latency_ms > 100.0 {
            warn!("AI inference latency {}ms exceeds target of 100ms", latency_ms);
        } else {
            info!("AI inference completed in {}ms for {} instruments", latency_ms, signals.len());
        }

        Ok(signals)
    }

    /// Process data for a specific instrument
    async fn process_instrument_data(
        &mut self,
        instrument_id: Uuid,
        ticks: &[&MarketTick],
    ) -> Result<AISignal> {
        // Ensure models are initialized
        if !self.time_series_models.contains_key(&instrument_id) {
            self.initialize_instrument(instrument_id).await?;
        }

        // Update time series model with new data
        if let Some(ts_model) = self.time_series_models.get_mut(&instrument_id) {
            let owned_ticks: Vec<MarketTick> = ticks.iter().map(|&t| t.clone()).collect();
            ts_model.add_market_data(&owned_ticks).await?;
        }

        // Update regime detector
        if let Some(regime_detector) = self.regime_detectors.get_mut(&instrument_id) {
            for &tick in ticks {
                regime_detector.update_with_tick(tick);
            }
        }

        // Generate predictions
        let price_predictions = if let Some(ts_model) = self.time_series_models.get(&instrument_id) {
            if ts_model.is_ready_for_prediction() {
                ts_model.predict_multi_horizon(instrument_id).await.unwrap_or_default()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        // Convert to trading signal format
        let prediction_results: Vec<PredictionResult> = price_predictions
            .into_iter()
            .map(|pred| PredictionResult {
                horizon: Duration::from_secs(pred.horizon_seconds as u64),
                predicted_price: pred.predicted_price,
                confidence_score: pred.confidence_score,
                prediction_interval: (pred.prediction_interval_lower, pred.prediction_interval_upper),
            })
            .collect();

        // Detect regime
        let regime_signal = if let Some(regime_detector) = self.regime_detectors.get_mut(&instrument_id) {
            regime_detector.detect_current_regime()
        } else {
            None
        };

        // Calculate overall confidence
        let confidence_score = if prediction_results.is_empty() {
            0.5 // Default confidence when no predictions available
        } else {
            prediction_results.iter()
                .map(|p| p.confidence_score)
                .sum::<f64>() / prediction_results.len() as f64
        };

        Ok(AISignal {
            instrument_id,
            timestamp: Utc::now(),
            price_predictions: prediction_results,
            regime_signal,
            rl_recommendation: None, // Will be implemented with RL agent
            confidence_score,
        })
    }

    /// Update performance metrics
    fn update_performance_metrics(&mut self, signal_count: usize, latency_ms: f64) {
        self.performance_metrics.total_predictions += signal_count as u64;

        // Update average latency with exponential moving average
        if self.performance_metrics.average_latency_ms == 0.0 {
            self.performance_metrics.average_latency_ms = latency_ms;
        } else {
            self.performance_metrics.average_latency_ms =
                0.9 * self.performance_metrics.average_latency_ms + 0.1 * latency_ms;
        }

        self.performance_metrics.last_update = Some(Utc::now());
    }

    /// Get current performance metrics
    pub fn get_performance_metrics(&self) -> &AIPerformanceMetrics {
        &self.performance_metrics
    }

    /// Optimized instrument data processing for <100ms latency
    async fn process_instrument_data_optimized(
        &mut self,
        instrument_id: Uuid,
        ticks: &[MarketTick],
    ) -> Result<AISignal> {
        let start_time = std::time::Instant::now();

        // Ensure models are initialized
        if !self.time_series_models.contains_key(&instrument_id) {
            self.initialize_instrument(instrument_id).await?;
        }

        // Fast path: Use only the latest tick for real-time inference
        let latest_tick = ticks.last().ok_or_else(|| {
            crate::utils::errors::PantherSwapError::ai_prediction("No ticks provided".to_string())
        })?;

        // Update models with minimal data for speed
        if let Some(ts_model) = self.time_series_models.get_mut(&instrument_id) {
            // Only add the latest tick to avoid processing overhead
            ts_model.add_market_data(&[latest_tick.clone()]).await?;
        }

        // Update regime detector with latest tick only
        if let Some(regime_detector) = self.regime_detectors.get_mut(&instrument_id) {
            regime_detector.update_with_tick(latest_tick);
        }

        // Generate fast predictions (single horizon for speed)
        let price_predictions = if let Some(ts_model) = self.time_series_models.get(&instrument_id) {
            if ts_model.is_ready_for_prediction() {
                // Use fast single prediction instead of multi-horizon
                if let Ok(prediction) = ts_model.predict_next_price(instrument_id).await {
                    vec![PredictionResult {
                        horizon: Duration::from_secs(60), // 1 minute horizon
                        predicted_price: prediction.predicted_price,
                        confidence_score: prediction.confidence_score,
                        prediction_interval: (prediction.prediction_interval_lower, prediction.prediction_interval_upper),
                    }]
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        // Fast regime detection
        let regime_signal = if let Some(regime_detector) = self.regime_detectors.get_mut(&instrument_id) {
            regime_detector.detect_current_regime()
        } else {
            None
        };

        // Calculate confidence
        let confidence_score = if price_predictions.is_empty() {
            0.5
        } else {
            price_predictions[0].confidence_score
        };

        let signal = AISignal {
            instrument_id,
            timestamp: Utc::now(),
            price_predictions,
            regime_signal,
            rl_recommendation: None,
            confidence_score,
        };

        // Cache the result if caching is enabled
        if self.inference_optimizer.enable_caching {
            let cached_prediction = CachedPrediction {
                prediction: signal.clone(),
                timestamp: Utc::now(),
                ttl_seconds: self.inference_optimizer.cache_ttl_seconds,
            };
            self.prediction_cache.insert(instrument_id, cached_prediction);
        }

        let processing_time = start_time.elapsed().as_millis() as f64;
        if processing_time > 50.0 { // Half of our 100ms target per instrument
            warn!("Instrument {} processing took {}ms", instrument_id, processing_time);
        }

        Ok(signal)
    }

    /// Store AI predictions in TimescaleDB
    async fn store_predictions_in_database(&self, signals: &[AISignal]) -> Result<()> {
        let query_manager = self.database.query_manager();

        for signal in signals {
            for prediction in &signal.price_predictions {
                let prediction_data = crate::database::types::AIPrediction {
                    timestamp: signal.timestamp,
                    instrument_id: signal.instrument_id,
                    model_type: "LSTM".to_string(),
                    model_version: "1.0.0".to_string(),
                    prediction_horizon_minutes: prediction.horizon.as_secs() / 60,
                    predicted_price: prediction.predicted_price,
                    predicted_volatility: None, // Could be added later
                    confidence_score: prediction.confidence_score,
                    prediction_intervals: Some(serde_json::json!({
                        "lower": prediction.prediction_interval.0,
                        "upper": prediction.prediction_interval.1
                    })),
                    feature_importance: None, // Could be added for model explainability
                    created_at: Utc::now(),
                };

                if let Err(e) = query_manager.insert_ai_prediction(&prediction_data).await {
                    warn!("Failed to store prediction for instrument {}: {}", signal.instrument_id, e);
                }
            }

            // Store regime detection results in microstructure analysis table
            if let Some(ref regime) = signal.regime_signal {
                let regime_data = crate::database::types::MicrostructureAnalysis {
                    timestamp: signal.timestamp,
                    instrument_id: signal.instrument_id,
                    order_book_imbalance: 0.0, // Default value
                    bid_ask_spread: 0.0, // Default value
                    market_depth: 0.0, // Default value
                    price_impact: 0.0, // Default value
                    liquidity_score: 0.5, // Default value
                    volatility_regime: format!("{:?}", regime.regime_type),
                    market_maker_presence: 0.5, // Default value
                    analysis_data: Some(serde_json::json!({
                        "regime_confidence": regime.confidence_score,
                        "transition_probability": regime.transition_probability,
                        "regime_duration_minutes": regime.regime_duration.map(|d| d.as_secs() / 60),
                        "volatility_estimate": regime.volatility_estimate,
                        "source": "HMM_regime_detector"
                    })),
                    created_at: Utc::now(),
                };

                if let Err(e) = query_manager.insert_microstructure_analysis(&regime_data).await {
                    warn!("Failed to store regime detection for instrument {}: {}", signal.instrument_id, e);
                }
            }
        }

        info!("Stored {} AI predictions in database", signals.len());
        Ok(())
    }

    /// Clear expired cache entries
    pub fn cleanup_cache(&mut self) {
        let before_count = self.prediction_cache.len();
        self.prediction_cache.retain(|_, cached| !cached.is_expired());
        let after_count = self.prediction_cache.len();

        if before_count != after_count {
            info!("Cleaned up {} expired cache entries", before_count - after_count);
        }
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();
        stats.insert("total_entries".to_string(), serde_json::json!(self.prediction_cache.len()));

        let expired_count = self.prediction_cache.values()
            .filter(|cached| cached.is_expired())
            .count();
        stats.insert("expired_entries".to_string(), serde_json::json!(expired_count));

        stats.insert("cache_enabled".to_string(), serde_json::json!(self.inference_optimizer.enable_caching));
        stats.insert("cache_ttl_seconds".to_string(), serde_json::json!(self.inference_optimizer.cache_ttl_seconds));

        stats
    }

    /// Check if AI engine is ready for production use
    pub fn is_ready(&self) -> bool {
        !self.time_series_models.is_empty() &&
        self.performance_metrics.average_latency_ms < 100.0
    }
}


