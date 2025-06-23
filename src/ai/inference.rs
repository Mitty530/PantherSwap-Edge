// AI Inference Engine with Real-time Processing and Optimization
use crate::utils::{Result, PantherSwapError};
use crate::database::types::MarketTick;
use crate::trading::signals::{AISignal, PredictionResult, RegimeSignal, RLRecommendation};
use crate::ai::time_series::LSTMTimeSeriesModel;
use crate::ai::rl_agent::{RLTradingAgent, TradingAction};
use crate::ai::hmm_regime::HMMRegimeDetector;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;
use tracing::{info, warn, error, debug};
use std::time::{Instant, Duration};

/// Configuration for the inference engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    pub max_batch_size: usize,
    pub batch_timeout_ms: u64,
    pub max_latency_ms: u64,
    pub enable_caching: bool,
    pub cache_ttl_seconds: u64,
    pub parallel_processing: bool,
    pub max_concurrent_inferences: usize,
    pub enable_quantization: bool,
    pub quantization_bits: u8,
    pub enable_model_compression: bool,
    pub enable_prefetching: bool,
    pub prefetch_window_size: usize,
    pub enable_adaptive_batching: bool,
    pub warmup_iterations: usize,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 32,
            batch_timeout_ms: 10, // 10ms for real-time trading
            max_latency_ms: 100,  // 100ms max latency requirement
            enable_caching: true,
            cache_ttl_seconds: 5,
            parallel_processing: true,
            max_concurrent_inferences: 8,
            enable_quantization: true,
            quantization_bits: 8, // 8-bit quantization for speed
            enable_model_compression: true,
            enable_prefetching: true,
            prefetch_window_size: 10,
            enable_adaptive_batching: true,
            warmup_iterations: 100,
        }
    }
}

/// Inference request for batch processing
#[derive(Debug, Clone)]
pub struct InferenceRequest {
    pub request_id: Uuid,
    pub instrument_id: Uuid,
    pub market_data: Vec<MarketTick>,
    pub timestamp: DateTime<Utc>,
    pub priority: InferencePriority,
}

/// Priority levels for inference requests
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum InferencePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Inference result with performance metrics
#[derive(Debug, Clone)]
pub struct InferenceResult {
    pub request_id: Uuid,
    pub instrument_id: Uuid,
    pub ai_signal: AISignal,
    pub inference_latency_ms: f64,
    pub model_confidence: f64,
    pub timestamp: DateTime<Utc>,
}

/// Cached inference result
#[derive(Debug, Clone)]
struct CachedResult {
    result: InferenceResult,
    expires_at: DateTime<Utc>,
}

/// Model quantizer for performance optimization
#[derive(Debug)]
pub struct ModelQuantizer {
    pub quantization_enabled: bool,
    pub quantization_bits: u8,
    pub quantization_cache: HashMap<String, Vec<u8>>,
}

/// Prefetch buffer for predictive loading
#[derive(Debug)]
pub struct PrefetchBuffer {
    pub buffer: VecDeque<InferenceRequest>,
    pub prefetch_predictions: HashMap<Uuid, Vec<f64>>,
    pub last_prefetch_time: Option<DateTime<Utc>>,
}

/// Adaptive batcher for dynamic batch sizing
#[derive(Debug)]
pub struct AdaptiveBatcher {
    pub current_batch_size: usize,
    pub optimal_batch_size: usize,
    pub latency_history: VecDeque<f64>,
    pub throughput_history: VecDeque<f64>,
    pub last_adjustment: Option<DateTime<Utc>>,
}

/// Performance optimizer for runtime optimization
#[derive(Debug)]
pub struct PerformanceOptimizer {
    pub warmup_completed: bool,
    pub warmup_iterations: usize,
    pub optimization_level: u8,
    pub performance_profile: HashMap<String, f64>,
    pub auto_scaling_enabled: bool,
}

/// Performance metrics for the inference engine
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InferenceMetrics {
    pub total_requests: u64,
    pub successful_inferences: u64,
    pub failed_inferences: u64,
    pub average_latency_ms: f64,
    pub max_latency_ms: f64,
    pub min_latency_ms: f64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub batch_processing_count: u64,
    pub last_updated: DateTime<Utc>,
}

/// Dynamic ensemble weights for AI models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicEnsembleWeights {
    pub lstm_weight: f64,
    pub hmm_weight: f64,
    pub rl_weight: f64,
    pub last_updated: DateTime<Utc>,
    pub performance_scores: ModelPerformanceScores,
}

impl Default for DynamicEnsembleWeights {
    fn default() -> Self {
        Self {
            lstm_weight: 0.4,
            hmm_weight: 0.3,
            rl_weight: 0.3,
            last_updated: Utc::now(),
            performance_scores: ModelPerformanceScores::default(),
        }
    }
}

/// Performance scores for individual AI models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformanceScores {
    pub lstm_score: f64,
    pub hmm_score: f64,
    pub rl_score: f64,
}

impl Default for ModelPerformanceScores {
    fn default() -> Self {
        Self {
            lstm_score: 0.7, // Default 70% performance
            hmm_score: 0.7,
            rl_score: 0.7,
        }
    }
}

/// Enhanced performance tracker for AI models
#[derive(Debug, Default)]
pub struct EnhancedPerformanceTracker {
    pub lstm_accuracy_history: VecDeque<f64>,
    pub hmm_accuracy_history: VecDeque<f64>,
    pub rl_success_history: VecDeque<f64>,
    pub current_regime: Option<crate::database::types::RegimeType>,
    pub regime_transition_count: u64,
    pub last_performance_update: DateTime<Utc>,
    pub performance_window_size: usize,
}

/// Performance statistics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub lstm_avg_accuracy: f64,
    pub hmm_avg_accuracy: f64,
    pub rl_avg_success_rate: f64,
    pub current_regime: Option<crate::database::types::RegimeType>,
    pub regime_transitions: u64,
    pub last_update: DateTime<Utc>,
}

impl EnhancedPerformanceTracker {
    pub fn new(window_size: usize) -> Self {
        Self {
            lstm_accuracy_history: VecDeque::with_capacity(window_size),
            hmm_accuracy_history: VecDeque::with_capacity(window_size),
            rl_success_history: VecDeque::with_capacity(window_size),
            current_regime: None,
            regime_transition_count: 0,
            last_performance_update: Utc::now(),
            performance_window_size: window_size,
        }
    }

    pub fn get_lstm_performance_score(&self) -> f64 {
        if self.lstm_accuracy_history.is_empty() {
            return 0.7; // Default score
        }
        self.lstm_accuracy_history.iter().sum::<f64>() / self.lstm_accuracy_history.len() as f64
    }

    pub fn get_hmm_performance_score(&self) -> f64 {
        if self.hmm_accuracy_history.is_empty() {
            return 0.7; // Default score
        }
        self.hmm_accuracy_history.iter().sum::<f64>() / self.hmm_accuracy_history.len() as f64
    }

    pub fn get_rl_performance_score(&self) -> f64 {
        if self.rl_success_history.is_empty() {
            return 0.7; // Default score
        }
        self.rl_success_history.iter().sum::<f64>() / self.rl_success_history.len() as f64
    }

    pub fn get_current_regime(&self) -> Option<crate::database::types::RegimeType> {
        self.current_regime.clone()
    }

    pub fn update_lstm_performance(&mut self, accuracy: f64) {
        if self.lstm_accuracy_history.len() >= self.performance_window_size {
            self.lstm_accuracy_history.pop_front();
        }
        self.lstm_accuracy_history.push_back(accuracy);
        self.last_performance_update = Utc::now();
    }

    pub fn update_hmm_performance(&mut self, accuracy: f64) {
        if self.hmm_accuracy_history.len() >= self.performance_window_size {
            self.hmm_accuracy_history.pop_front();
        }
        self.hmm_accuracy_history.push_back(accuracy);
        self.last_performance_update = Utc::now();
    }

    pub fn update_rl_performance(&mut self, success_rate: f64) {
        if self.rl_success_history.len() >= self.performance_window_size {
            self.rl_success_history.pop_front();
        }
        self.rl_success_history.push_back(success_rate);
        self.last_performance_update = Utc::now();
    }

    pub fn update_regime(&mut self, new_regime: crate::database::types::RegimeType) {
        if self.current_regime.is_some() && self.current_regime != Some(new_regime.clone()) {
            self.regime_transition_count += 1;
        }
        self.current_regime = Some(new_regime);
    }
}

/// Real-time AI inference engine with advanced optimizations
pub struct InferenceEngine {
    config: InferenceConfig,

    // Model references
    lstm_models: Arc<RwLock<HashMap<Uuid, LSTMTimeSeriesModel>>>,
    rl_agents: Arc<RwLock<HashMap<Uuid, RLTradingAgent>>>,
    hmm_detectors: Arc<RwLock<HashMap<Uuid, HMMRegimeDetector>>>,

    // Batch processing
    request_queue: Arc<RwLock<VecDeque<InferenceRequest>>>,
    result_cache: Arc<RwLock<HashMap<String, CachedResult>>>,

    // Performance tracking
    metrics: Arc<RwLock<InferenceMetrics>>,
    performance_tracker: Arc<RwLock<EnhancedPerformanceTracker>>,

    // Communication channels
    request_sender: mpsc::UnboundedSender<InferenceRequest>,
    result_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<InferenceResult>>>>,

    // Advanced optimization components
    model_quantizer: Arc<RwLock<ModelQuantizer>>,
    prefetch_buffer: Arc<RwLock<PrefetchBuffer>>,
    adaptive_batcher: Arc<RwLock<AdaptiveBatcher>>,
    performance_optimizer: Arc<RwLock<PerformanceOptimizer>>,
}

impl InferenceEngine {
    /// Create a new inference engine
    pub fn new(config: InferenceConfig) -> Self {
        let (request_sender, _request_receiver) = mpsc::unbounded_channel();
        let (_result_sender, result_receiver) = mpsc::unbounded_channel();

        Self {
            config: config.clone(),
            lstm_models: Arc::new(RwLock::new(HashMap::new())),
            rl_agents: Arc::new(RwLock::new(HashMap::new())),
            hmm_detectors: Arc::new(RwLock::new(HashMap::new())),
            request_queue: Arc::new(RwLock::new(VecDeque::new())),
            result_cache: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(InferenceMetrics::default())),
            performance_tracker: Arc::new(RwLock::new(EnhancedPerformanceTracker::new(100))), // 100 sample window
            request_sender,
            result_receiver: Arc::new(RwLock::new(Some(result_receiver))),
            model_quantizer: Arc::new(RwLock::new(ModelQuantizer {
                quantization_enabled: config.enable_quantization,
                quantization_bits: config.quantization_bits,
                quantization_cache: HashMap::new(),
            })),
            prefetch_buffer: Arc::new(RwLock::new(PrefetchBuffer {
                buffer: VecDeque::new(),
                prefetch_predictions: HashMap::new(),
                last_prefetch_time: None,
            })),
            adaptive_batcher: Arc::new(RwLock::new(AdaptiveBatcher {
                current_batch_size: config.max_batch_size,
                optimal_batch_size: config.max_batch_size,
                latency_history: VecDeque::new(),
                throughput_history: VecDeque::new(),
                last_adjustment: None,
            })),
            performance_optimizer: Arc::new(RwLock::new(PerformanceOptimizer {
                warmup_completed: false,
                warmup_iterations: 0,
                optimization_level: 1,
                performance_profile: HashMap::new(),
                auto_scaling_enabled: true,
            })),
        }
    }

    /// Register an LSTM model for inference
    pub async fn register_lstm_model(&self, instrument_id: Uuid, model: LSTMTimeSeriesModel) {
        let mut models = self.lstm_models.write().await;
        models.insert(instrument_id, model);
        info!("Registered LSTM model for instrument: {}", instrument_id);
    }

    /// Register an RL agent for inference
    pub async fn register_rl_agent(&self, instrument_id: Uuid, agent: RLTradingAgent) {
        let mut agents = self.rl_agents.write().await;
        agents.insert(instrument_id, agent);
        info!("Registered RL agent for instrument: {}", instrument_id);
    }

    /// Register an HMM regime detector for inference
    pub async fn register_hmm_detector(&self, instrument_id: Uuid, detector: HMMRegimeDetector) {
        let mut detectors = self.hmm_detectors.write().await;
        detectors.insert(instrument_id, detector);
        info!("Registered HMM detector for instrument: {}", instrument_id);
    }

    /// Submit an inference request
    pub async fn submit_request(&self, request: InferenceRequest) -> Result<()> {
        // Check cache first if enabled
        if self.config.enable_caching {
            let cache_key = self.generate_cache_key(&request);
            let cache = self.result_cache.read().await;

            if let Some(cached) = cache.get(&cache_key) {
                if cached.expires_at > Utc::now() {
                    // Cache hit - update metrics and return
                    let mut metrics = self.metrics.write().await;
                    metrics.cache_hits += 1;
                    debug!("Cache hit for request: {}", request.request_id);
                    return Ok(());
                }
            }
        }

        // Add to processing queue
        self.request_sender.send(request)
            .map_err(|e| PantherSwapError::ai_prediction(format!("Failed to submit inference request: {}", e)))?;

        Ok(())
    }

    /// Process a single inference request
    pub async fn process_request(&self, request: InferenceRequest) -> Result<InferenceResult> {
        let start_time = Instant::now();

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_requests += 1;
        }

        // Perform inference based on available models
        let ai_signal = self.run_inference(&request).await?;

        let inference_latency = start_time.elapsed().as_millis() as f64;

        // Check latency requirement
        if inference_latency > self.config.max_latency_ms as f64 {
            warn!("Inference latency exceeded limit: {}ms > {}ms",
                  inference_latency, self.config.max_latency_ms);
        }

        let result = InferenceResult {
            request_id: request.request_id,
            instrument_id: request.instrument_id,
            ai_signal,
            inference_latency_ms: inference_latency,
            model_confidence: 0.8, // Placeholder - would be calculated from model outputs
            timestamp: Utc::now(),
        };

        // Update performance metrics
        self.update_metrics(inference_latency, true).await;

        // Cache result if enabled
        if self.config.enable_caching {
            self.cache_result(&request, &result).await;
        }

        Ok(result)
    }

    /// Run inference using available models
    async fn run_inference(&self, request: &InferenceRequest) -> Result<AISignal> {
        let instrument_id = request.instrument_id;
        let market_data = &request.market_data;

        // Initialize AI signal
        let mut ai_signal = AISignal {
            instrument_id,
            timestamp: Utc::now(),
            price_predictions: Vec::new(),
            regime_signal: None,
            rl_recommendation: None,
            confidence_score: 0.0,
        };

        // LSTM Time Series Prediction
        if let Some(lstm_model) = self.lstm_models.read().await.get(&instrument_id) {
            match lstm_model.predict_multi_horizon(instrument_id).await {
                Ok(predictions) => {
                    // Convert PricePrediction to PredictionResult
                    ai_signal.price_predictions = predictions.into_iter().map(|p| {
                        crate::trading::signals::PredictionResult {
                            horizon: std::time::Duration::from_secs(p.horizon_seconds as u64),
                            predicted_price: p.predicted_price,
                            confidence_score: p.confidence_score,
                            prediction_interval: (p.prediction_interval_lower, p.prediction_interval_upper),
                        }
                    }).collect();
                    debug!("LSTM predictions generated for instrument: {}", instrument_id);
                }
                Err(e) => {
                    warn!("LSTM prediction failed for instrument {}: {}", instrument_id, e);
                }
            }
        }

        // HMM Regime Detection
        if let Some(hmm_detector) = self.hmm_detectors.read().await.get(&instrument_id) {
            if let Some(regime_signal) = hmm_detector.detect_current_regime() {
                ai_signal.regime_signal = Some(regime_signal);
                debug!("HMM regime signal generated for instrument: {}", instrument_id);
            }
        }

        // RL Agent Recommendation
        if let Some(rl_agent) = self.rl_agents.write().await.get_mut(&instrument_id) {
            match rl_agent.get_recommendation(market_data, &ai_signal) {
                Ok(recommendation) => {
                    ai_signal.rl_recommendation = Some(recommendation);
                    debug!("RL recommendation generated for instrument: {}", instrument_id);
                }
                Err(e) => {
                    warn!("RL recommendation failed for instrument {}: {}", instrument_id, e);
                }
            }
        }

        // Apply ensemble decision making and conflict resolution
        let ensemble_signal = self.apply_ensemble_logic(&ai_signal).await?;

        Ok(ensemble_signal)
    }

    /// Apply ensemble decision making and conflict resolution
    async fn apply_ensemble_logic(&self, ai_signal: &AISignal) -> Result<AISignal> {
        let mut enhanced_signal = ai_signal.clone();

        // Calculate ensemble confidence score
        enhanced_signal.confidence_score = self.calculate_ensemble_confidence(&enhanced_signal).await;

        // Apply conflict resolution
        enhanced_signal = self.resolve_ai_conflicts(enhanced_signal).await?;

        // Apply ensemble weighting
        enhanced_signal = self.apply_ensemble_weighting(enhanced_signal).await;

        Ok(enhanced_signal)
    }

    /// Calculate ensemble confidence score from all AI components with dynamic weighting
    async fn calculate_ensemble_confidence(&self, signal: &AISignal) -> f64 {
        let mut confidence_scores = Vec::new();
        let mut weights = Vec::new();

        // Get dynamic weights based on recent performance
        let dynamic_weights = self.calculate_dynamic_ensemble_weights().await;

        // LSTM confidence (based on prediction intervals)
        if !signal.price_predictions.is_empty() {
            let lstm_confidence = signal.price_predictions.iter()
                .map(|p| p.confidence_score)
                .sum::<f64>() / signal.price_predictions.len() as f64;
            confidence_scores.push(lstm_confidence);
            weights.push(dynamic_weights.lstm_weight);
        }

        // HMM regime confidence
        if let Some(regime_signal) = &signal.regime_signal {
            confidence_scores.push(regime_signal.confidence);
            weights.push(dynamic_weights.hmm_weight);
        }

        // RL agent confidence
        if let Some(rl_rec) = &signal.rl_recommendation {
            confidence_scores.push(rl_rec.confidence);
            weights.push(dynamic_weights.rl_weight);
        }

        // Calculate weighted average
        if confidence_scores.is_empty() {
            return 0.5; // Default confidence
        }

        let total_weight: f64 = weights.iter().sum();
        if total_weight == 0.0 {
            return confidence_scores.iter().sum::<f64>() / confidence_scores.len() as f64;
        }

        confidence_scores.iter()
            .zip(weights.iter())
            .map(|(score, weight)| score * weight)
            .sum::<f64>() / total_weight
    }

    /// Calculate dynamic ensemble weights based on recent performance
    async fn calculate_dynamic_ensemble_weights(&self) -> DynamicEnsembleWeights {
        let metrics = self.metrics.read().await;
        let performance_tracker = self.performance_tracker.read().await;

        // Get recent performance metrics for each model type
        let lstm_performance = performance_tracker.get_lstm_performance_score();
        let hmm_performance = performance_tracker.get_hmm_performance_score();
        let rl_performance = performance_tracker.get_rl_performance_score();

        // Calculate base weights with performance adjustment
        let mut lstm_weight = 0.4 * (1.0 + (lstm_performance - 0.7) * 0.5); // Base 40%, adjust by performance
        let mut hmm_weight = 0.3 * (1.0 + (hmm_performance - 0.7) * 0.5);   // Base 30%, adjust by performance
        let mut rl_weight = 0.3 * (1.0 + (rl_performance - 0.7) * 0.5);     // Base 30%, adjust by performance

        // Apply regime-based adjustments
        if let Some(current_regime) = performance_tracker.get_current_regime() {
            match current_regime {
                crate::database::types::RegimeType::Trending => {
                    // LSTM performs better in trending markets
                    lstm_weight *= 1.2;
                    hmm_weight *= 0.9;
                    rl_weight *= 0.9;
                },
                crate::database::types::RegimeType::Volatile => {
                    // RL agent adapts better to volatility
                    lstm_weight *= 0.8;
                    hmm_weight *= 1.1;
                    rl_weight *= 1.3;
                },
                crate::database::types::RegimeType::Crisis => {
                    // HMM regime detection is crucial in crisis
                    lstm_weight *= 0.7;
                    hmm_weight *= 1.4;
                    rl_weight *= 1.1;
                },
                crate::database::types::RegimeType::Normal => {
                    // Balanced approach in normal markets
                    // No adjustment needed
                },
                crate::database::types::RegimeType::Bullish => {
                    // In bullish markets, slightly favor LSTM
                    lstm_weight *= 1.1;
                    hmm_weight *= 0.95;
                },
                crate::database::types::RegimeType::Bearish => {
                    // In bearish markets, slightly favor RL
                    rl_weight *= 1.1;
                    hmm_weight *= 1.05;
                },
                crate::database::types::RegimeType::Sideways => {
                    // In sideways markets, balanced approach
                    // No adjustment needed
                },
                crate::database::types::RegimeType::HighVolatility => {
                    // Similar to volatile markets
                    lstm_weight *= 0.8;
                    hmm_weight *= 1.1;
                    rl_weight *= 1.3;
                },
            }
        }

        // Apply confidence decay for poor performing models
        if lstm_performance < 0.6 {
            lstm_weight *= 0.7;
        }
        if hmm_performance < 0.6 {
            hmm_weight *= 0.7;
        }
        if rl_performance < 0.6 {
            rl_weight *= 0.7;
        }

        // Normalize weights to sum to 1.0
        let total_weight = lstm_weight + hmm_weight + rl_weight;
        if total_weight > 0.0 {
            lstm_weight /= total_weight;
            hmm_weight /= total_weight;
            rl_weight /= total_weight;
        } else {
            // Fallback to default weights
            lstm_weight = 0.4;
            hmm_weight = 0.3;
            rl_weight = 0.3;
        }

        // Apply minimum weight constraints to prevent complete model exclusion
        lstm_weight = lstm_weight.max(0.15).min(0.7);
        hmm_weight = hmm_weight.max(0.1).min(0.5);
        rl_weight = rl_weight.max(0.1).min(0.5);

        // Final normalization
        let final_total = lstm_weight + hmm_weight + rl_weight;
        DynamicEnsembleWeights {
            lstm_weight: lstm_weight / final_total,
            hmm_weight: hmm_weight / final_total,
            rl_weight: rl_weight / final_total,
            last_updated: chrono::Utc::now(),
            performance_scores: ModelPerformanceScores {
                lstm_score: lstm_performance,
                hmm_score: hmm_performance,
                rl_score: rl_performance,
            },
        }
    }

    /// Calculate overall confidence score from individual model outputs (legacy method)
    fn calculate_confidence_score(&self, ai_signal: &AISignal) -> f64 {
        let mut total_confidence = 0.0;
        let mut model_count = 0;

        // LSTM confidence
        if !ai_signal.price_predictions.is_empty() {
            let avg_confidence = ai_signal.price_predictions.iter()
                .map(|p| p.confidence_score)
                .sum::<f64>() / ai_signal.price_predictions.len() as f64;
            total_confidence += avg_confidence;
            model_count += 1;
        }

        // HMM confidence
        if let Some(regime_signal) = &ai_signal.regime_signal {
            total_confidence += regime_signal.confidence;
            model_count += 1;
        }

        // RL confidence
        if let Some(rl_rec) = &ai_signal.rl_recommendation {
            total_confidence += rl_rec.confidence;
            model_count += 1;
        }

        if model_count > 0 {
            total_confidence / model_count as f64
        } else {
            0.0
        }
    }

    /// Resolve conflicts between AI components
    async fn resolve_ai_conflicts(&self, mut signal: AISignal) -> Result<AISignal> {
        // Check for conflicts between RL recommendation and LSTM predictions
        if let (Some(rl_rec), false) = (&signal.rl_recommendation, signal.price_predictions.is_empty()) {
            let lstm_direction = self.get_lstm_direction(&signal.price_predictions);
            let rl_direction = self.get_rl_direction_from_string(&rl_rec.action);

            // If directions conflict, reduce confidence and apply conflict resolution
            if lstm_direction != rl_direction && lstm_direction != 0 && rl_direction != 0 {
                signal.confidence_score *= 0.7; // Reduce confidence due to conflict

                // Apply regime-based conflict resolution
                if let Some(regime_signal) = signal.regime_signal.clone() {
                    signal = self.apply_regime_based_resolution(signal, &regime_signal).await;
                }
            }
        }

        // Check for regime transition conflicts
        if let Some(regime_signal) = &signal.regime_signal {
            if regime_signal.transition_probability > 0.7 {
                // High transition probability - reduce confidence in predictions
                signal.confidence_score *= 0.8;

                // Adjust RL exploration if regime is changing
                if let Some(rl_rec) = &mut signal.rl_recommendation {
                    // Reduce confidence during regime transitions
                    signal.rl_recommendation = Some(RLRecommendation {
                        action: rl_rec.action.clone(),
                        confidence: rl_rec.confidence * 0.9,
                        expected_reward: rl_rec.expected_reward * 0.8,
                    });
                }
            }
        }

        Ok(signal)
    }

    /// Apply ensemble weighting based on recent performance
    async fn apply_ensemble_weighting(&self, mut signal: AISignal) -> AISignal {
        // Get recent performance metrics for each component
        let metrics = self.metrics.read().await;

        // Adjust LSTM predictions based on recent accuracy
        if !signal.price_predictions.is_empty() {
            let lstm_performance_factor = self.get_lstm_performance_factor(&metrics).await;
            for prediction in &mut signal.price_predictions {
                prediction.confidence_score *= lstm_performance_factor;
            }
        }

        // Adjust RL recommendation based on recent success rate
        if let Some(rl_rec) = &mut signal.rl_recommendation {
            let rl_performance_factor = self.get_rl_performance_factor(&metrics).await;
            signal.rl_recommendation = Some(RLRecommendation {
                action: rl_rec.action.clone(),
                confidence: rl_rec.confidence * rl_performance_factor,
                expected_reward: rl_rec.expected_reward,
            });
        }

        // Adjust regime signal based on stability
        if let Some(regime_signal) = &mut signal.regime_signal {
            let hmm_performance_factor = self.get_hmm_performance_factor(&metrics).await;
            signal.regime_signal = Some(RegimeSignal {
                current_regime: regime_signal.current_regime.clone(),
                regime: regime_signal.current_regime.clone(), // Alias for backward compatibility
                transition_probability: regime_signal.transition_probability,
                confidence: regime_signal.confidence * hmm_performance_factor,
                regime_strength: regime_signal.confidence * hmm_performance_factor, // Alias for confidence
                expected_duration_minutes: 30, // Default duration
                timestamp: regime_signal.timestamp,
            });
        }

        signal
    }

    /// Get LSTM direction from predictions
    fn get_lstm_direction(&self, predictions: &[crate::trading::signals::PredictionResult]) -> i8 {
        if predictions.is_empty() {
            return 0;
        }

        // Use the shortest horizon prediction for direction
        if let Some(short_pred) = predictions.iter().min_by_key(|p| p.horizon.as_secs()) {
            // Compare predicted price with current price (would need current price)
            // For now, use a simplified approach based on prediction confidence
            if short_pred.confidence_score > 0.6 {
                1 // Bullish
            } else if short_pred.confidence_score < 0.4 {
                -1 // Bearish
            } else {
                0 // Neutral
            }
        } else {
            0
        }
    }

    /// Get RL direction from action string
    fn get_rl_direction_from_string(&self, action: &str) -> i8 {
        match action.to_uppercase().as_str() {
            "BUY_SMALL" | "BUY_LARGE" | "BUY" => 1,
            "SELL_SMALL" | "SELL_LARGE" | "SELL" => -1,
            "HOLD" => 0,
            _ => 0,
        }
    }

    /// Get RL direction from action (legacy method for TradingAction enum)
    fn get_rl_direction(&self, action: &TradingAction) -> i8 {
        match action {
            TradingAction::BuySmall | TradingAction::BuyLarge => 1,
            TradingAction::SellSmall | TradingAction::SellLarge => -1,
            TradingAction::Hold => 0,
        }
    }

    /// Apply regime-based conflict resolution
    async fn apply_regime_based_resolution(&self, mut signal: AISignal, regime_signal: &RegimeSignal) -> AISignal {
        use crate::database::types::RegimeType;
        match regime_signal.current_regime {
            RegimeType::Crisis => {
                // In crisis, prioritize RL agent (more adaptive)
                if let Some(rl_rec) = &mut signal.rl_recommendation {
                    signal.rl_recommendation = Some(RLRecommendation {
                        action: rl_rec.action.clone(),
                        confidence: rl_rec.confidence * 1.2,
                        expected_reward: rl_rec.expected_reward,
                    });
                }
                // Reduce LSTM confidence in crisis
                for prediction in &mut signal.price_predictions {
                    prediction.confidence_score *= 0.7;
                }
            },
            RegimeType::Trending => {
                // In trending markets, prioritize LSTM predictions
                for prediction in &mut signal.price_predictions {
                    prediction.confidence_score *= 1.1;
                }
            },
            RegimeType::Volatile => {
                // In volatile markets, reduce all confidences
                signal.confidence_score *= 0.8;
            },
            RegimeType::Normal => {
                // In normal markets, use balanced approach (no adjustment)
            },
            RegimeType::Bullish => {
                // In bullish markets, slightly favor LSTM predictions
                for prediction in &mut signal.price_predictions {
                    prediction.confidence_score *= 1.05;
                }
            },
            RegimeType::Bearish => {
                // In bearish markets, slightly favor RL agent
                if let Some(rl_rec) = &mut signal.rl_recommendation {
                    signal.rl_recommendation = Some(RLRecommendation {
                        action: rl_rec.action.clone(),
                        confidence: rl_rec.confidence * 1.05,
                        expected_reward: rl_rec.expected_reward,
                    });
                }
            },
            RegimeType::Sideways => {
                // In sideways markets, reduce all confidences slightly
                signal.confidence_score *= 0.95;
            },
            RegimeType::HighVolatility => {
                // Similar to volatile markets
                signal.confidence_score *= 0.8;
            },
        }

        signal
    }

    /// Get LSTM performance factor based on recent accuracy
    async fn get_lstm_performance_factor(&self, _metrics: &InferenceMetrics) -> f64 {
        // In production, this would track LSTM prediction accuracy
        // For now, return a default factor
        1.0
    }

    /// Get RL performance factor based on recent success rate
    async fn get_rl_performance_factor(&self, _metrics: &InferenceMetrics) -> f64 {
        // In production, this would track RL agent success rate
        // For now, return a default factor
        1.0
    }

    /// Get HMM performance factor based on regime detection accuracy
    async fn get_hmm_performance_factor(&self, _metrics: &InferenceMetrics) -> f64 {
        // In production, this would track HMM regime detection accuracy
        // For now, return a default factor
        1.0
    }

    /// Update performance metrics
    async fn update_metrics(&self, latency_ms: f64, success: bool) {
        let mut metrics = self.metrics.write().await;

        if success {
            metrics.successful_inferences += 1;
        } else {
            metrics.failed_inferences += 1;
        }

        // Update latency statistics
        if metrics.successful_inferences == 1 {
            metrics.min_latency_ms = latency_ms;
            metrics.max_latency_ms = latency_ms;
            metrics.average_latency_ms = latency_ms;
        } else {
            metrics.min_latency_ms = metrics.min_latency_ms.min(latency_ms);
            metrics.max_latency_ms = metrics.max_latency_ms.max(latency_ms);

            // Update running average
            let total_requests = metrics.successful_inferences as f64;
            metrics.average_latency_ms =
                (metrics.average_latency_ms * (total_requests - 1.0) + latency_ms) / total_requests;
        }

        metrics.last_updated = Utc::now();
    }

    /// Cache inference result
    async fn cache_result(&self, request: &InferenceRequest, result: &InferenceResult) {
        let cache_key = self.generate_cache_key(request);
        let expires_at = Utc::now() + chrono::Duration::seconds(self.config.cache_ttl_seconds as i64);

        let cached_result = CachedResult {
            result: result.clone(),
            expires_at,
        };

        let mut cache = self.result_cache.write().await;
        cache.insert(cache_key, cached_result);

        // Update cache metrics
        let mut metrics = self.metrics.write().await;
        metrics.cache_misses += 1;
    }

    /// Generate cache key for request
    fn generate_cache_key(&self, request: &InferenceRequest) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        request.instrument_id.hash(&mut hasher);

        // Hash recent market data (simplified)
        if let Some(latest_tick) = request.market_data.last() {
            latest_tick.timestamp.hash(&mut hasher);
            ((latest_tick.bid_price * 10000.0) as u64).hash(&mut hasher);
            ((latest_tick.ask_price * 10000.0) as u64).hash(&mut hasher);
        }

        format!("inference_{:x}", hasher.finish())
    }

    /// Clean expired cache entries
    pub async fn cleanup_cache(&self) {
        let mut cache = self.result_cache.write().await;
        let now = Utc::now();

        cache.retain(|_, cached| cached.expires_at > now);

        debug!("Cache cleanup completed, {} entries remaining", cache.len());
    }

    /// Get current performance metrics
    pub async fn get_metrics(&self) -> InferenceMetrics {
        self.metrics.read().await.clone()
    }

    /// Get current dynamic ensemble weights
    pub async fn get_dynamic_weights(&self) -> DynamicEnsembleWeights {
        self.calculate_dynamic_ensemble_weights().await
    }

    /// Update LSTM model performance
    pub async fn update_lstm_performance(&self, accuracy: f64) {
        let mut tracker = self.performance_tracker.write().await;
        tracker.update_lstm_performance(accuracy);
        info!("Updated LSTM performance: {:.3}", accuracy);
    }

    /// Update HMM model performance
    pub async fn update_hmm_performance(&self, accuracy: f64) {
        let mut tracker = self.performance_tracker.write().await;
        tracker.update_hmm_performance(accuracy);
        info!("Updated HMM performance: {:.3}", accuracy);
    }

    /// Update RL agent performance
    pub async fn update_rl_performance(&self, success_rate: f64) {
        let mut tracker = self.performance_tracker.write().await;
        tracker.update_rl_performance(success_rate);
        info!("Updated RL performance: {:.3}", success_rate);
    }

    /// Update current market regime
    pub async fn update_market_regime(&self, regime: crate::database::types::RegimeType) {
        let mut tracker = self.performance_tracker.write().await;
        tracker.update_regime(regime.clone());
        info!("Updated market regime: {:?}", regime);
    }

    /// Get performance tracker statistics
    pub async fn get_performance_stats(&self) -> PerformanceStats {
        let tracker = self.performance_tracker.read().await;
        PerformanceStats {
            lstm_avg_accuracy: tracker.get_lstm_performance_score(),
            hmm_avg_accuracy: tracker.get_hmm_performance_score(),
            rl_avg_success_rate: tracker.get_rl_performance_score(),
            current_regime: tracker.get_current_regime(),
            regime_transitions: tracker.regime_transition_count,
            last_update: tracker.last_performance_update,
        }
    }

    /// Check if inference engine is ready
    pub async fn is_ready(&self) -> bool {
        let lstm_count = self.lstm_models.read().await.len();
        let rl_count = self.rl_agents.read().await.len();
        let hmm_count = self.hmm_detectors.read().await.len();

        lstm_count > 0 || rl_count > 0 || hmm_count > 0
    }

    /// Start background processing tasks
    pub async fn start_background_tasks(&self) -> Result<()> {
        // Start cache cleanup task
        let cache_cleanup_interval = Duration::from_secs(60); // Cleanup every minute
        let cache = self.result_cache.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cache_cleanup_interval);
            loop {
                interval.tick().await;
                let mut cache_guard = cache.write().await;
                let now = Utc::now();
                cache_guard.retain(|_, cached| cached.expires_at > now);
            }
        });

        info!("Inference engine background tasks started");
        Ok(())
    }

    /// Process batch of requests with adaptive optimization
    pub async fn process_batch(&self, requests: Vec<InferenceRequest>) -> Vec<Result<InferenceResult>> {
        let start_time = Instant::now();
        let mut results = Vec::with_capacity(requests.len());

        // Apply adaptive batching optimization
        let optimized_requests = self.optimize_batch(requests).await;

        if self.config.parallel_processing {
            // Process requests in parallel with optimizations
            let futures: Vec<_> = optimized_requests.into_iter()
                .map(|req| self.process_request_optimized(req))
                .collect();

            results = futures::future::join_all(futures).await;
        } else {
            // Process requests sequentially with optimizations
            for request in optimized_requests {
                let result = self.process_request_optimized(request).await;
                results.push(result);
            }
        }

        let batch_latency = start_time.elapsed().as_millis() as f64;

        // Update adaptive batcher with performance feedback
        self.update_adaptive_batcher(batch_latency, results.len()).await;

        // Update batch processing metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.batch_processing_count += 1;
        }

        debug!("Processed optimized batch of {} requests in {}ms", results.len(), batch_latency);
        results
    }

    /// Process a single request with advanced optimizations
    async fn process_request_optimized(&self, request: InferenceRequest) -> Result<InferenceResult> {
        let start_time = Instant::now();

        // Check if warmup is needed
        if !self.is_warmed_up().await {
            self.perform_warmup().await?;
        }

        // Clone request for later use
        let request_clone = request.clone();

        // Apply quantization if enabled
        let quantized_request = if self.config.enable_quantization {
            self.apply_quantization(&request).await?
        } else {
            request
        };

        // Check prefetch buffer first
        if let Some(prefetched_result) = self.check_prefetch_buffer(&quantized_request).await {
            return Ok(prefetched_result);
        }

        // Process with standard pipeline
        let result = self.process_request(quantized_request).await?;

        // Update prefetch buffer for future requests
        if self.config.enable_prefetching {
            self.update_prefetch_buffer(&request_clone, &result).await;
        }

        let processing_time = start_time.elapsed().as_millis() as f64;

        // Update performance optimizer
        self.update_performance_optimizer(processing_time).await;

        Ok(result)
    }

    /// Optimize batch for better performance
    async fn optimize_batch(&self, requests: Vec<InferenceRequest>) -> Vec<InferenceRequest> {
        if !self.config.enable_adaptive_batching {
            return requests;
        }

        let batcher = self.adaptive_batcher.read().await;
        let optimal_size = batcher.optimal_batch_size;
        drop(batcher);

        // Reorder requests by priority and similarity
        let mut optimized = requests;
        optimized.sort_by(|a, b| {
            // Sort by priority first, then by instrument similarity
            b.priority.cmp(&a.priority)
                .then_with(|| a.instrument_id.cmp(&b.instrument_id))
        });

        // Limit to optimal batch size
        if optimized.len() > optimal_size {
            optimized.truncate(optimal_size);
        }

        optimized
    }

    /// Apply model quantization for faster inference
    async fn apply_quantization(&self, request: &InferenceRequest) -> Result<InferenceRequest> {
        let quantizer = self.model_quantizer.read().await;

        if !quantizer.quantization_enabled {
            return Ok(request.clone());
        }

        // Simulate quantization (in production, this would quantize model weights)
        let mut quantized_request = request.clone();

        // Apply quantization to market data (simplified)
        for tick in &mut quantized_request.market_data {
            // Quantize prices to reduce precision for faster processing
            let price_scale = 10000.0; // 4 decimal places
            tick.bid_price = (tick.bid_price * price_scale).round() / price_scale;
            tick.ask_price = (tick.ask_price * price_scale).round() / price_scale;
        }

        Ok(quantized_request)
    }

    /// Check prefetch buffer for cached predictions
    async fn check_prefetch_buffer(&self, request: &InferenceRequest) -> Option<InferenceResult> {
        if !self.config.enable_prefetching {
            return None;
        }

        let buffer = self.prefetch_buffer.read().await;

        // Check if we have a prefetched prediction for this instrument
        if let Some(predictions) = buffer.prefetch_predictions.get(&request.instrument_id) {
            if !predictions.is_empty() {
                // Create a result from prefetched data
                let ai_signal = AISignal {
                    instrument_id: request.instrument_id,
                    timestamp: Utc::now(),
                    price_predictions: vec![], // Would populate from prefetched data
                    regime_signal: None,
                    rl_recommendation: None,
                    confidence_score: 0.8, // Prefetched confidence
                };

                return Some(InferenceResult {
                    request_id: request.request_id,
                    instrument_id: request.instrument_id,
                    ai_signal,
                    inference_latency_ms: 0.1, // Very fast prefetched result
                    model_confidence: 0.8,
                    timestamp: Utc::now(),
                });
            }
        }

        None
    }

    /// Update prefetch buffer with new predictions
    async fn update_prefetch_buffer(&self, request: &InferenceRequest, result: &InferenceResult) {
        if !self.config.enable_prefetching {
            return;
        }

        let mut buffer = self.prefetch_buffer.write().await;

        // Store prediction for future use
        let predictions = vec![result.model_confidence]; // Simplified
        buffer.prefetch_predictions.insert(request.instrument_id, predictions);

        // Update prefetch time
        buffer.last_prefetch_time = Some(Utc::now());

        // Limit buffer size
        if buffer.prefetch_predictions.len() > self.config.prefetch_window_size {
            // Remove oldest entries (simplified - would use LRU in production)
            let keys_to_remove: Vec<_> = buffer.prefetch_predictions.keys().take(1).cloned().collect();
            for key in keys_to_remove {
                buffer.prefetch_predictions.remove(&key);
            }
        }
    }
    /// Update adaptive batcher with performance feedback
    async fn update_adaptive_batcher(&self, batch_latency: f64, batch_size: usize) {
        let mut batcher = self.adaptive_batcher.write().await;

        // Add latency sample
        if batcher.latency_history.len() >= 100 {
            batcher.latency_history.pop_front();
        }
        batcher.latency_history.push_back(batch_latency);

        // Calculate throughput (requests per second)
        let throughput = if batch_latency > 0.0 {
            (batch_size as f64 * 1000.0) / batch_latency
        } else {
            0.0
        };

        if batcher.throughput_history.len() >= 100 {
            batcher.throughput_history.pop_front();
        }
        batcher.throughput_history.push_back(throughput);

        // Adjust batch size if we have enough samples
        if batcher.latency_history.len() >= 10 {
            let avg_latency: f64 = batcher.latency_history.iter().sum::<f64>() / batcher.latency_history.len() as f64;
            let avg_throughput: f64 = batcher.throughput_history.iter().sum::<f64>() / batcher.throughput_history.len() as f64;

            // Optimize for latency vs throughput trade-off
            if avg_latency > self.config.max_latency_ms as f64 * 0.8 {
                // Reduce batch size if latency is too high
                batcher.optimal_batch_size = ((batcher.optimal_batch_size as f64 * 0.9) as usize).max(1);
            } else if avg_latency < self.config.max_latency_ms as f64 * 0.5 && avg_throughput > 1000.0 {
                // Increase batch size if we have headroom
                batcher.optimal_batch_size = ((batcher.optimal_batch_size as f64 * 1.1) as usize).min(self.config.max_batch_size);
            }
        }

        batcher.last_adjustment = Some(Utc::now());
    }

    /// Check if the engine is warmed up
    async fn is_warmed_up(&self) -> bool {
        let optimizer = self.performance_optimizer.read().await;
        optimizer.warmup_completed
    }

    /// Perform warmup iterations for optimal performance
    async fn perform_warmup(&self) -> Result<()> {
        let mut optimizer = self.performance_optimizer.write().await;

        if optimizer.warmup_completed {
            return Ok(());
        }

        info!("Starting AI inference engine warmup...");

        // Simulate warmup iterations
        for i in 0..self.config.warmup_iterations {
            // Create dummy request for warmup
            let warmup_request = InferenceRequest {
                request_id: Uuid::new_v4(),
                instrument_id: Uuid::new_v4(),
                market_data: vec![], // Empty for warmup
                timestamp: Utc::now(),
                priority: InferencePriority::Low,
            };

            // Process warmup request (simplified)
            let _start = Instant::now();
            tokio::time::sleep(Duration::from_micros(100)).await; // Simulate processing
            let _elapsed = _start.elapsed();

            optimizer.warmup_iterations += 1;

            if i % 20 == 0 {
                debug!("Warmup progress: {}/{}", i + 1, self.config.warmup_iterations);
            }
        }

        optimizer.warmup_completed = true;
        optimizer.optimization_level = 3; // Fully optimized

        info!("AI inference engine warmup completed ({} iterations)", optimizer.warmup_iterations);
        Ok(())
    }

    /// Update performance optimizer with runtime metrics
    async fn update_performance_optimizer(&self, processing_time: f64) {
        let mut optimizer = self.performance_optimizer.write().await;

        // Update performance profile
        optimizer.performance_profile.insert("avg_latency".to_string(), processing_time);

        // Auto-scale optimization level based on performance
        if optimizer.auto_scaling_enabled {
            if processing_time > self.config.max_latency_ms as f64 * 0.8 {
                // Performance degrading - increase optimization
                optimizer.optimization_level = (optimizer.optimization_level + 1).min(5);
            } else if processing_time < self.config.max_latency_ms as f64 * 0.3 {
                // Performance good - can reduce optimization for accuracy
                optimizer.optimization_level = (optimizer.optimization_level.saturating_sub(1)).max(1);
            }
        }
    }

    /// Get current optimization status
    pub async fn get_optimization_status(&self) -> OptimizationStatus {
        let quantizer = self.model_quantizer.read().await;
        let batcher = self.adaptive_batcher.read().await;
        let optimizer = self.performance_optimizer.read().await;

        OptimizationStatus {
            quantization_enabled: quantizer.quantization_enabled,
            quantization_bits: quantizer.quantization_bits,
            current_batch_size: batcher.current_batch_size,
            optimal_batch_size: batcher.optimal_batch_size,
            warmup_completed: optimizer.warmup_completed,
            optimization_level: optimizer.optimization_level,
            auto_scaling_enabled: optimizer.auto_scaling_enabled,
        }
    }

    /// Enable/disable specific optimizations at runtime
    pub async fn configure_optimizations(&self, config: OptimizationConfig) -> Result<()> {
        if config.enable_quantization.is_some() {
            let mut quantizer = self.model_quantizer.write().await;
            quantizer.quantization_enabled = config.enable_quantization.unwrap();
        }

        if config.quantization_bits.is_some() {
            let mut quantizer = self.model_quantizer.write().await;
            quantizer.quantization_bits = config.quantization_bits.unwrap();
        }

        if config.enable_auto_scaling.is_some() {
            let mut optimizer = self.performance_optimizer.write().await;
            optimizer.auto_scaling_enabled = config.enable_auto_scaling.unwrap();
        }

        Ok(())
    }
}

/// Optimization status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationStatus {
    pub quantization_enabled: bool,
    pub quantization_bits: u8,
    pub current_batch_size: usize,
    pub optimal_batch_size: usize,
    pub warmup_completed: bool,
    pub optimization_level: u8,
    pub auto_scaling_enabled: bool,
}

/// Runtime optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    pub enable_quantization: Option<bool>,
    pub quantization_bits: Option<u8>,
    pub enable_auto_scaling: Option<bool>,
}

/// Factory function to create an inference engine with default configuration
pub fn create_inference_engine() -> InferenceEngine {
    let config = InferenceConfig::default();
    InferenceEngine::new(config)
}

/// Factory function to create a high-performance inference engine
pub fn create_optimized_inference_engine() -> InferenceEngine {
    let config = InferenceConfig {
        max_batch_size: 64,
        batch_timeout_ms: 5, // Aggressive batching
        max_latency_ms: 50,  // Stricter latency requirement
        enable_caching: true,
        cache_ttl_seconds: 3, // Shorter cache TTL for freshness
        parallel_processing: true,
        max_concurrent_inferences: 16, // More parallelism
        enable_quantization: true,
        quantization_bits: 8,
        enable_model_compression: true,
        enable_prefetching: true,
        prefetch_window_size: 20,
        enable_adaptive_batching: true,
        warmup_iterations: 200, // More warmup for better optimization
    };
    InferenceEngine::new(config)
}

/// Utility function to create a high-priority inference request
pub fn create_priority_request(
    instrument_id: Uuid,
    market_data: Vec<MarketTick>,
    priority: InferencePriority,
) -> InferenceRequest {
    InferenceRequest {
        request_id: Uuid::new_v4(),
        instrument_id,
        market_data,
        timestamp: Utc::now(),
        priority,
    }
}
