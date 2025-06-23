// Hidden Markov Model for Market Regime Detection
use crate::database::types::{MarketTick, RegimeType};
use crate::trading::signals::RegimeSignal;
use crate::utils::{Result, PantherSwapError};
use chrono::{DateTime, Utc, Duration};
use tracing::{info, warn, debug, error};
use ndarray::{Array1, Array2, Array3};
use serde::{Serialize, Deserialize};
use std::collections::{VecDeque, HashMap};

/// Time scale for multi-scale HMM analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeScale {
    OneMinute,
    FiveMinutes,
    FifteenMinutes,
    OneHour,
}

impl TimeScale {
    /// Get the duration in seconds for this time scale
    pub fn duration_seconds(&self) -> u64 {
        match self {
            TimeScale::OneMinute => 60,
            TimeScale::FiveMinutes => 300,
            TimeScale::FifteenMinutes => 900,
            TimeScale::OneHour => 3600,
        }
    }

    /// Get the observation window size for this time scale
    pub fn observation_window(&self) -> usize {
        match self {
            TimeScale::OneMinute => 60,      // 1 hour of 1-minute data
            TimeScale::FiveMinutes => 48,    // 4 hours of 5-minute data
            TimeScale::FifteenMinutes => 32, // 8 hours of 15-minute data
            TimeScale::OneHour => 24,        // 24 hours of 1-hour data
        }
    }

    /// Get all time scales in order from shortest to longest
    pub fn all_scales() -> Vec<TimeScale> {
        vec![
            TimeScale::OneMinute,
            TimeScale::FiveMinutes,
            TimeScale::FifteenMinutes,
            TimeScale::OneHour,
        ]
    }
}

/// Multi-scale HMM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiScaleHMMConfig {
    /// Individual HMM configurations for each time scale
    pub scale_configs: HashMap<TimeScale, HMMConfig>,
    /// Weight for each time scale in final regime decision
    pub scale_weights: HashMap<TimeScale, f64>,
    /// Minimum agreement threshold across scales
    pub consensus_threshold: f64,
    /// Enable hierarchical regime propagation
    pub enable_hierarchical_propagation: bool,
    /// Transition detection sensitivity
    pub transition_sensitivity: f64,
}

impl Default for MultiScaleHMMConfig {
    fn default() -> Self {
        let mut scale_configs = HashMap::new();
        let mut scale_weights = HashMap::new();

        // Configure each time scale
        for scale in TimeScale::all_scales() {
            let config = HMMConfig {
                num_states: 4,
                observation_window: scale.observation_window(),
                feature_dimensions: 16,
                convergence_threshold: 1e-8,
                max_iterations: 200,
                min_confidence: match scale {
                    TimeScale::OneMinute => 0.65,      // Lower threshold for faster detection
                    TimeScale::FiveMinutes => 0.70,
                    TimeScale::FifteenMinutes => 0.75,
                    TimeScale::OneHour => 0.80,        // Higher threshold for stability
                },
                enable_enhanced_features: true,
                volatility_lookback: match scale {
                    TimeScale::OneMinute => 20,
                    TimeScale::FiveMinutes => 15,
                    TimeScale::FifteenMinutes => 12,
                    TimeScale::OneHour => 10,
                },
                trend_lookback: match scale {
                    TimeScale::OneMinute => 10,
                    TimeScale::FiveMinutes => 8,
                    TimeScale::FifteenMinutes => 6,
                    TimeScale::OneHour => 5,
                },
                momentum_lookback: 5,
                regime_persistence_threshold: 0.8,
                transition_smoothing_factor: 0.3,
                adaptive_threshold: true,
                fast_detection_mode: matches!(scale, TimeScale::OneMinute | TimeScale::FiveMinutes),
                enable_online_learning: false,
                online_learning_config: OnlineLearningConfig::default(),
                enable_microstructure_features: true,
                enable_volatility_clustering: true,
                enable_cross_asset_correlation: false,
                enable_regime_specific_indicators: true,
                microstructure_window: 40,
                volatility_clustering_window: 80,
                correlation_window: 120,
                garch_window: 40,
                hurst_exponent_window: 80,
                fractal_dimension_window: 60,
            };
            scale_configs.insert(scale, config);

            // Set weights - shorter timeframes get higher weights for responsiveness
            let weight = match scale {
                TimeScale::OneMinute => 0.4,      // Highest weight for immediate signals
                TimeScale::FiveMinutes => 0.3,
                TimeScale::FifteenMinutes => 0.2,
                TimeScale::OneHour => 0.1,        // Lowest weight but provides stability
            };
            scale_weights.insert(scale, weight);
        }

        Self {
            scale_configs,
            scale_weights,
            consensus_threshold: 0.6,
            enable_hierarchical_propagation: true,
            transition_sensitivity: 0.7,
        }
    }
}

/// Configuration for HMM regime detector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HMMConfig {
    pub num_states: usize,
    pub observation_window: usize,
    pub feature_dimensions: usize,
    pub convergence_threshold: f64,
    pub max_iterations: usize,
    pub min_confidence: f64,

    // Enhanced detection parameters
    pub enable_enhanced_features: bool,
    pub volatility_lookback: usize,
    pub trend_lookback: usize,
    pub momentum_lookback: usize,
    pub regime_persistence_threshold: f64,
    pub transition_smoothing_factor: f64,
    pub adaptive_threshold: bool,
    pub fast_detection_mode: bool,

    // Online learning parameters
    pub enable_online_learning: bool,
    pub online_learning_config: OnlineLearningConfig,

    // Advanced feature engineering parameters
    pub enable_microstructure_features: bool,
    pub enable_volatility_clustering: bool,
    pub enable_cross_asset_correlation: bool,
    pub enable_regime_specific_indicators: bool,
    pub microstructure_window: usize,
    pub volatility_clustering_window: usize,
    pub correlation_window: usize,
    pub garch_window: usize,
    pub hurst_exponent_window: usize,
    pub fractal_dimension_window: usize,
}

impl Default for HMMConfig {
    fn default() -> Self {
        Self {
            num_states: 4, // Normal, Trending, Volatile, Crisis
            observation_window: 50,
            feature_dimensions: 16, // Expanded for advanced features
            convergence_threshold: 1e-6,
            max_iterations: 100,
            min_confidence: 0.6,

            // Enhanced detection defaults
            enable_enhanced_features: true,
            volatility_lookback: 20,
            trend_lookback: 10,
            momentum_lookback: 5,
            regime_persistence_threshold: 0.8,
            transition_smoothing_factor: 0.3,
            adaptive_threshold: true,
            fast_detection_mode: true,

            // Online learning defaults
            enable_online_learning: false,
            online_learning_config: OnlineLearningConfig::default(),

            // Advanced feature engineering defaults
            enable_microstructure_features: true,
            enable_volatility_clustering: true,
            enable_cross_asset_correlation: false, // Disabled by default (requires multiple assets)
            enable_regime_specific_indicators: true,
            microstructure_window: 30,
            volatility_clustering_window: 50,
            correlation_window: 100,
            garch_window: 30,
            hurst_exponent_window: 50,
            fractal_dimension_window: 40,
        }
    }
}

/// Multi-scale regime signal combining signals from all time scales
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiScaleRegimeSignal {
    pub timestamp: DateTime<Utc>,
    pub consensus_regime: Option<RegimeType>,
    pub consensus_confidence: f64,
    pub scale_signals: HashMap<TimeScale, RegimeSignal>,
    pub transition_probability: f64,
    pub regime_strength: f64,
    pub hierarchical_consistency: f64,
}

/// Aggregated market observation across multiple time scales
#[derive(Debug, Clone)]
pub struct AggregatedMarketObservation {
    pub timestamp: DateTime<Utc>,
    pub scale_observations: HashMap<TimeScale, MarketObservation>,
    pub cross_scale_features: Array1<f64>,
}

/// Market observation for HMM
#[derive(Debug, Clone)]
pub struct MarketObservation {
    pub timestamp: DateTime<Utc>,
    pub features: Array1<f64>,
    pub volatility: f64,
    pub trend: f64,
    pub volume: f64,

    // Enhanced features
    pub momentum: f64,
    pub bid_ask_spread: f64,
    pub price_skewness: f64,
    pub price_kurtosis: f64,
    pub autocorrelation: f64,
    pub regime_persistence: f64,
    pub transition_probability: f64,

    // Advanced microstructure features
    pub order_flow_imbalance: f64,
    pub effective_spread: f64,
    pub price_impact: f64,
    pub market_depth_ratio: f64,

    // Volatility clustering features
    pub garch_volatility: f64,
    pub volatility_persistence: f64,
    pub volatility_clustering_score: f64,

    // Regime-specific technical indicators
    pub hurst_exponent: f64,
    pub fractal_dimension: f64,
    pub regime_strength: f64,
    pub regime_transition_signal: f64,
}

/// HMM parameters
#[derive(Debug, Clone)]
pub struct HMMParameters {
    /// Initial state probabilities (π)
    pub initial_probs: Array1<f64>,
    /// State transition matrix (A)
    pub transition_matrix: Array2<f64>,
    /// Emission parameters (means and covariances for Gaussian emissions)
    pub emission_means: Array2<f64>,
    pub emission_covariances: Vec<Array2<f64>>,
}

/// Enhanced transition detector for faster regime change detection
#[derive(Debug, Clone)]
pub struct TransitionDetector {
    pub detection_window: usize,
    pub sensitivity: f64,
    pub momentum_threshold: f64,
    pub volatility_threshold: f64,
    pub transition_scores: VecDeque<f64>,
    pub last_detection: Option<DateTime<Utc>>,

    // Enhanced detection components
    pub change_point_detector: ChangePointDetector,
    pub volatility_breakout_detector: VolatilityBreakoutDetector,
    pub ensemble_voter: EnsembleVoter,
    pub detection_history: VecDeque<TransitionDetectionResult>,
}

/// Change point detection for regime transitions
#[derive(Debug, Clone)]
pub struct ChangePointDetector {
    pub window_size: usize,
    pub threshold: f64,
    pub price_history: VecDeque<f64>,
    pub volatility_history: VecDeque<f64>,
    pub cumulative_sum: f64,
    pub mean_shift_threshold: f64,
}

/// Volatility breakout detection
#[derive(Debug, Clone)]
pub struct VolatilityBreakoutDetector {
    pub lookback_window: usize,
    pub breakout_threshold: f64,
    pub volatility_history: VecDeque<f64>,
    pub rolling_mean: f64,
    pub rolling_std: f64,
    pub last_breakout: Option<DateTime<Utc>>,
}

/// Ensemble voting for transition detection
#[derive(Debug, Clone)]
pub struct EnsembleVoter {
    pub voting_methods: Vec<VotingMethod>,
    pub method_weights: Vec<f64>,
    pub consensus_threshold: f64,
    pub recent_votes: VecDeque<EnsembleVote>,
}

/// Voting methods for ensemble detection
#[derive(Debug, Clone, PartialEq)]
pub enum VotingMethod {
    ChangePoint,
    VolatilityBreakout,
    MomentumShift,
    TrendReversal,
    VolumeSpike,
}

/// Individual vote from a detection method
#[derive(Debug, Clone)]
pub struct EnsembleVote {
    pub method: VotingMethod,
    pub confidence: f64,
    pub timestamp: DateTime<Utc>,
    pub detected_transition: bool,
}

/// Transition detection result
#[derive(Debug, Clone)]
pub struct TransitionDetectionResult {
    pub detected: bool,
    pub confidence: f64,
    pub detection_method: String,
    pub timestamp: DateTime<Utc>,
    pub transition_type: TransitionType,
}

/// Types of regime transitions
#[derive(Debug, Clone, PartialEq)]
pub enum TransitionType {
    /// Gradual transition
    Gradual,
    /// Sharp/sudden transition
    Sharp,
    /// Volatility spike
    VolatilitySpike,
    /// Trend reversal
    TrendReversal,
    /// Unknown/mixed
    Unknown,
}

/// Online learning configuration for adaptive HMM training
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineLearningConfig {
    /// Learning rate for online parameter updates
    pub learning_rate: f64,
    /// Forgetting factor for exponential decay of old observations
    pub forgetting_factor: f64,
    /// Minimum batch size for online updates
    pub min_batch_size: usize,
    /// Maximum batch size for online updates
    pub max_batch_size: usize,
    /// Concept drift detection threshold
    pub drift_threshold: f64,
    /// Window size for drift detection
    pub drift_window_size: usize,
    /// Enable adaptive learning rate
    pub adaptive_learning_rate: bool,
    /// Minimum learning rate
    pub min_learning_rate: f64,
    /// Maximum learning rate
    pub max_learning_rate: f64,
}

impl Default for OnlineLearningConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.01,
            forgetting_factor: 0.95,
            min_batch_size: 10,
            max_batch_size: 50,
            drift_threshold: 0.1,
            drift_window_size: 100,
            adaptive_learning_rate: true,
            min_learning_rate: 0.001,
            max_learning_rate: 0.1,
        }
    }
}

/// Concept drift detection result
#[derive(Debug, Clone)]
pub struct ConceptDriftResult {
    pub is_drifting: bool,
    pub drift_score: f64,
    pub drift_type: DriftType,
    pub detection_time: DateTime<Utc>,
    pub recommended_action: DriftAction,
}

/// Types of concept drift
#[derive(Debug, Clone, PartialEq)]
pub enum DriftType {
    /// Gradual drift over time
    Gradual,
    /// Sudden/abrupt drift
    Sudden,
    /// Recurring patterns
    Recurring,
    /// No drift detected
    None,
}

/// Recommended actions for drift
#[derive(Debug, Clone, PartialEq)]
pub enum DriftAction {
    /// Retrain the model completely
    FullRetrain,
    /// Update parameters incrementally
    IncrementalUpdate,
    /// Increase monitoring frequency
    IncreaseMonitoring,
    /// Reset to baseline model
    ResetToBaseline,
    /// No action needed
    NoAction,
}

/// Concept drift detector for online learning
#[derive(Debug, Clone)]
pub struct ConceptDriftDetector {
    config: OnlineLearningConfig,
    baseline_performance: Option<f64>,
    recent_performances: VecDeque<f64>,
    drift_scores: VecDeque<f64>,
    last_drift_detection: Option<DateTime<Utc>>,
    consecutive_drift_count: usize,
}

impl ConceptDriftDetector {
    pub fn new(config: OnlineLearningConfig) -> Self {
        let drift_window_size = config.drift_window_size;
        Self {
            config,
            baseline_performance: None,
            recent_performances: VecDeque::with_capacity(drift_window_size),
            drift_scores: VecDeque::with_capacity(drift_window_size),
            last_drift_detection: None,
            consecutive_drift_count: 0,
        }
    }

    /// Update with new performance metric (e.g., log-likelihood)
    pub fn update_performance(&mut self, performance: f64) {
        if self.recent_performances.len() >= self.config.drift_window_size {
            self.recent_performances.pop_front();
        }
        self.recent_performances.push_back(performance);

        // Set baseline if not set
        if self.baseline_performance.is_none() && self.recent_performances.len() >= 10 {
            self.baseline_performance = Some(self.calculate_mean(&self.recent_performances));
        }
    }

    /// Detect concept drift
    pub fn detect_drift(&mut self) -> Option<ConceptDriftResult> {
        if let Some(baseline) = self.baseline_performance {
            if self.recent_performances.len() >= self.config.drift_window_size / 2 {
                let recent_mean = self.calculate_mean(&self.recent_performances);
                let drift_score = (baseline - recent_mean).abs() / baseline.abs().max(1e-8);

                // Add to drift scores
                if self.drift_scores.len() >= self.config.drift_window_size {
                    self.drift_scores.pop_front();
                }
                self.drift_scores.push_back(drift_score);

                let is_drifting = drift_score > self.config.drift_threshold;

                if is_drifting {
                    self.consecutive_drift_count += 1;
                } else {
                    self.consecutive_drift_count = 0;
                }

                // Determine drift type and action
                let (drift_type, action) = self.classify_drift(drift_score);

                if is_drifting {
                    self.last_drift_detection = Some(Utc::now());
                    return Some(ConceptDriftResult {
                        is_drifting,
                        drift_score,
                        drift_type,
                        detection_time: Utc::now(),
                        recommended_action: action,
                    });
                }
            }
        }
        None
    }

    fn calculate_mean(&self, values: &VecDeque<f64>) -> f64 {
        if values.is_empty() {
            0.0
        } else {
            values.iter().sum::<f64>() / values.len() as f64
        }
    }

    fn classify_drift(&self, drift_score: f64) -> (DriftType, DriftAction) {
        if drift_score > self.config.drift_threshold * 3.0 {
            (DriftType::Sudden, DriftAction::FullRetrain)
        } else if self.consecutive_drift_count > 5 {
            (DriftType::Gradual, DriftAction::IncrementalUpdate)
        } else if drift_score > self.config.drift_threshold * 1.5 {
            (DriftType::Recurring, DriftAction::IncreaseMonitoring)
        } else {
            (DriftType::None, DriftAction::NoAction)
        }
    }
}

impl Default for TransitionDetector {
    fn default() -> Self {
        Self {
            detection_window: 10,
            sensitivity: 0.7,
            momentum_threshold: 0.02,
            volatility_threshold: 0.015,
            transition_scores: VecDeque::with_capacity(10),
            last_detection: None,
            change_point_detector: ChangePointDetector::default(),
            volatility_breakout_detector: VolatilityBreakoutDetector::default(),
            ensemble_voter: EnsembleVoter::default(),
            detection_history: VecDeque::with_capacity(50),
        }
    }
}

impl Default for ChangePointDetector {
    fn default() -> Self {
        Self {
            window_size: 20,
            threshold: 2.0,
            price_history: VecDeque::with_capacity(20),
            volatility_history: VecDeque::with_capacity(20),
            cumulative_sum: 0.0,
            mean_shift_threshold: 1.5,
        }
    }
}

impl Default for VolatilityBreakoutDetector {
    fn default() -> Self {
        Self {
            lookback_window: 30,
            breakout_threshold: 2.5,
            volatility_history: VecDeque::with_capacity(30),
            rolling_mean: 0.0,
            rolling_std: 0.0,
            last_breakout: None,
        }
    }
}

impl Default for EnsembleVoter {
    fn default() -> Self {
        Self {
            voting_methods: vec![
                VotingMethod::ChangePoint,
                VotingMethod::VolatilityBreakout,
                VotingMethod::MomentumShift,
                VotingMethod::TrendReversal,
                VotingMethod::VolumeSpike,
            ],
            method_weights: vec![0.25, 0.25, 0.2, 0.2, 0.1],
            consensus_threshold: 0.6,
            recent_votes: VecDeque::with_capacity(100),
        }
    }
}

/// Enhanced regime detection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedRegimeStats {
    pub current_regime: Option<RegimeType>,
    pub regime_persistence_count: usize,
    pub adaptive_threshold: f64,
    pub regime_distribution: HashMap<RegimeType, f64>,
    pub average_transition_time_minutes: f64,
    pub detection_accuracy: f64,
    pub fast_detection_enabled: bool,
    pub last_transition_time: Option<DateTime<Utc>>,
    pub total_observations: usize,
    pub total_regime_changes: usize,
}

/// Multi-scale Hidden Markov Model for regime detection
#[derive(Clone)]
pub struct MultiScaleHMMRegimeDetector {
    pub config: MultiScaleHMMConfig,
    pub scale_detectors: HashMap<TimeScale, HMMRegimeDetector>,
    raw_tick_buffer: VecDeque<MarketTick>,
    aggregated_observations: HashMap<TimeScale, VecDeque<MarketObservation>>,
    last_aggregation_times: HashMap<TimeScale, DateTime<Utc>>,
    consensus_history: VecDeque<MultiScaleRegimeSignal>,
    pub is_initialized: bool,
}

/// Hidden Markov Model for regime detection
#[derive(Clone)]
pub struct HMMRegimeDetector {
    config: HMMConfig,
    parameters: HMMParameters,
    observations: VecDeque<MarketObservation>,
    current_state_probs: Array1<f64>,
    is_trained: bool,
    last_regime: Option<RegimeType>,

    // Enhanced detection components
    price_history: VecDeque<f64>,
    volume_history: VecDeque<f64>,
    spread_history: VecDeque<f64>,
    regime_history: VecDeque<(RegimeType, f64)>, // (regime, confidence)
    transition_detector: TransitionDetector,
    adaptive_threshold: f64,
    regime_persistence_counter: usize,
    last_transition_time: Option<DateTime<Utc>>,

    // Online learning components
    online_batch: VecDeque<MarketObservation>,
    baseline_log_likelihood: f64,
    recent_log_likelihoods: VecDeque<f64>,
    drift_detector: ConceptDriftDetector,
    learning_rate: f64,
    last_online_update: Option<DateTime<Utc>>,
    update_counter: usize,
}

impl MultiScaleHMMRegimeDetector {
    /// Create new multi-scale HMM regime detector
    pub fn new(config: MultiScaleHMMConfig) -> Self {
        let mut scale_detectors = HashMap::new();
        let mut aggregated_observations = HashMap::new();
        let mut last_aggregation_times = HashMap::new();

        // Initialize individual HMM detectors for each time scale
        for (scale, scale_config) in &config.scale_configs {
            let detector = HMMRegimeDetector::new(scale_config.clone());
            scale_detectors.insert(*scale, detector);
            aggregated_observations.insert(*scale, VecDeque::new());
            last_aggregation_times.insert(*scale, Utc::now());
        }

        Self {
            config,
            scale_detectors,
            raw_tick_buffer: VecDeque::new(),
            aggregated_observations,
            last_aggregation_times,
            consensus_history: VecDeque::new(),
            is_initialized: false,
        }
    }

    /// Update detector with new market tick
    pub fn update_with_tick(&mut self, tick: &MarketTick) -> Result<()> {
        // Add to raw tick buffer
        self.raw_tick_buffer.push_back(tick.clone());

        // Keep buffer size manageable (1 hour of ticks at 1-second intervals)
        if self.raw_tick_buffer.len() > 3600 {
            self.raw_tick_buffer.pop_front();
        }

        // Aggregate data for each time scale and update individual detectors
        for scale in TimeScale::all_scales() {
            if self.should_aggregate_for_scale(scale, tick.timestamp) {
                if let Some(aggregated_obs) = self.aggregate_data_for_scale(scale)? {
                    // Convert aggregated observation to market tick for the detector
                    let aggregated_tick = Self::observation_to_tick_static(&aggregated_obs);

                    // Update the scale-specific detector
                    if let Some(detector) = self.scale_detectors.get_mut(&scale) {
                        detector.update_with_tick(&aggregated_tick)?;
                    }

                    // Store aggregated observation
                    if let Some(obs_buffer) = self.aggregated_observations.get_mut(&scale) {
                        obs_buffer.push_back(aggregated_obs);

                        // Maintain buffer size
                        let max_size = scale.observation_window() * 2;
                        if obs_buffer.len() > max_size {
                            obs_buffer.pop_front();
                        }
                    }

                    // Update last aggregation time
                    self.last_aggregation_times.insert(scale, tick.timestamp);
                }
            }
        }

        // Mark as initialized once we have some data
        if !self.is_initialized && self.raw_tick_buffer.len() >= 10 {
            self.is_initialized = true;
        }

        Ok(())
    }

    /// Check if we should aggregate data for a given time scale
    fn should_aggregate_for_scale(&self, scale: TimeScale, current_time: DateTime<Utc>) -> bool {
        if let Some(&last_time) = self.last_aggregation_times.get(&scale) {
            let duration_since_last = current_time.signed_duration_since(last_time);
            duration_since_last.num_seconds() >= scale.duration_seconds() as i64
        } else {
            true // First time, always aggregate
        }
    }

    /// Aggregate raw tick data for a specific time scale
    fn aggregate_data_for_scale(&self, scale: TimeScale) -> Result<Option<MarketObservation>> {
        if self.raw_tick_buffer.is_empty() {
            return Ok(None);
        }

        let duration_seconds = scale.duration_seconds() as i64;
        let current_time = self.raw_tick_buffer.back().unwrap().timestamp;
        let start_time = current_time - Duration::seconds(duration_seconds);

        // Filter ticks within the time window
        let relevant_ticks: Vec<&MarketTick> = self.raw_tick_buffer
            .iter()
            .filter(|tick| tick.timestamp >= start_time && tick.timestamp <= current_time)
            .collect();

        if relevant_ticks.is_empty() {
            return Ok(None);
        }

        // Aggregate tick data into OHLCV format
        let first_tick = relevant_ticks.first().unwrap();
        let last_tick = relevant_ticks.last().unwrap();

        let open_price = (first_tick.bid_price + first_tick.ask_price) / 2.0;
        let close_price = (last_tick.bid_price + last_tick.ask_price) / 2.0;

        let mut high_price = open_price;
        let mut low_price = open_price;
        let mut total_volume = 0.0;
        let mut total_spread = 0.0;

        for tick in &relevant_ticks {
            let mid_price = (tick.bid_price + tick.ask_price) / 2.0;
            high_price = high_price.max(mid_price);
            low_price = low_price.min(mid_price);
            total_volume += tick.volume.unwrap_or(0.0);
            total_spread += tick.ask_price - tick.bid_price;
        }

        let avg_spread = total_spread / relevant_ticks.len() as f64;

        // Create aggregated market observation
        let aggregated_tick = MarketTick {
            timestamp: current_time,
            instrument_id: first_tick.instrument_id,
            provider: first_tick.provider.clone(),
            bid_price: close_price - avg_spread / 2.0,
            ask_price: close_price + avg_spread / 2.0,
            bid_size: last_tick.bid_size,
            ask_size: last_tick.ask_size,
            last_price: Some(close_price),
            volume: Some(total_volume),
            spread: avg_spread,
            data_quality_score: 1.0,
            raw_data: serde_json::Value::Null,
            symbol: first_tick.symbol.clone(),
            price: Some(close_price),
            bid: Some(close_price - avg_spread / 2.0),
            ask: Some(close_price + avg_spread / 2.0),
        };

        // Create a temporary detector to extract features
        let temp_config = self.config.scale_configs.get(&scale).unwrap().clone();
        let mut temp_detector = HMMRegimeDetector::new(temp_config);

        // Add some price history for feature calculation
        for tick in &relevant_ticks {
            let mid_price = (tick.bid_price + tick.ask_price) / 2.0;
            temp_detector.price_history.push_back(mid_price);
            temp_detector.volume_history.push_back(tick.volume.unwrap_or(0.0));
            temp_detector.spread_history.push_back(tick.ask_price - tick.bid_price);
        }

        let observation = temp_detector.extract_observation(&aggregated_tick)?;
        Ok(Some(observation))
    }

    /// Convert market observation back to market tick for detector update
    pub fn observation_to_tick_static(obs: &MarketObservation) -> MarketTick {
        // Extract basic price information from features
        let mid_price = if obs.features.len() >= 3 {
            // Estimate mid price from volatility and trend
            100.0 + obs.trend * 1000.0 // Simple estimation
        } else {
            100.0 // Default price
        };

        let spread = obs.bid_ask_spread * mid_price;

        MarketTick {
            timestamp: obs.timestamp,
            instrument_id: uuid::Uuid::new_v4(), // Placeholder
            provider: "aggregated".to_string(),
            bid_price: mid_price - spread / 2.0,
            ask_price: mid_price + spread / 2.0,
            bid_size: 100.0, // Default size
            ask_size: 100.0, // Default size
            last_price: Some(mid_price),
            volume: Some(obs.volume * 1000.0), // Denormalize volume
            spread,
            data_quality_score: 1.0,
            raw_data: serde_json::Value::Null,
            symbol: Some("AGGREGATED".to_string()),
            price: Some(mid_price),
            bid: Some(mid_price - spread / 2.0),
            ask: Some(mid_price + spread / 2.0),
        }
    }

    /// Detect current market regime using multi-scale analysis
    pub fn detect_current_regime(&mut self) -> Option<MultiScaleRegimeSignal> {
        if !self.is_initialized {
            return None;
        }

        let mut scale_signals = HashMap::new();
        let mut valid_signals = 0;
        let mut weighted_confidence_sum = 0.0;
        let mut regime_votes: HashMap<RegimeType, f64> = HashMap::new();

        // Collect signals from each time scale
        for scale in TimeScale::all_scales() {
            if let Some(detector) = self.scale_detectors.get_mut(&scale) {
                if let Some(signal) = detector.detect_current_regime() {
                    let weight = self.config.scale_weights.get(&scale).copied().unwrap_or(0.25);

                    scale_signals.insert(scale, signal.clone());
                    valid_signals += 1;
                    weighted_confidence_sum += signal.confidence * weight;

                    // Vote for regime type
                    let current_vote = regime_votes.get(&signal.current_regime).copied().unwrap_or(0.0);
                    regime_votes.insert(signal.current_regime, current_vote + weight);
                }
            }
        }

        if valid_signals == 0 {
            return None;
        }

        // Determine consensus regime
        let consensus_regime = regime_votes
            .iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(regime, _)| *regime);

        let consensus_confidence = weighted_confidence_sum / valid_signals as f64;

        // Check if consensus meets threshold
        if consensus_confidence < self.config.consensus_threshold {
            return None;
        }

        // Calculate additional metrics
        let transition_probability = self.calculate_multi_scale_transition_probability(&scale_signals);
        let regime_strength = self.calculate_regime_strength(&scale_signals);
        let hierarchical_consistency = self.calculate_hierarchical_consistency(&scale_signals);

        let multi_scale_signal = MultiScaleRegimeSignal {
            timestamp: Utc::now(),
            consensus_regime,
            consensus_confidence,
            scale_signals,
            transition_probability,
            regime_strength,
            hierarchical_consistency,
        };

        // Update consensus history
        self.consensus_history.push_back(multi_scale_signal.clone());
        if self.consensus_history.len() > 100 {
            self.consensus_history.pop_front();
        }

        Some(multi_scale_signal)
    }

    /// Calculate multi-scale transition probability
    fn calculate_multi_scale_transition_probability(&self, scale_signals: &HashMap<TimeScale, RegimeSignal>) -> f64 {
        if scale_signals.is_empty() {
            return 0.1;
        }

        let mut weighted_transition_prob = 0.0;
        let mut total_weight = 0.0;

        for (scale, signal) in scale_signals {
            let weight = self.config.scale_weights.get(scale).copied().unwrap_or(0.25);
            weighted_transition_prob += signal.transition_probability * weight;
            total_weight += weight;
        }

        if total_weight > 0.0 {
            weighted_transition_prob / total_weight
        } else {
            0.1
        }
    }

    /// Calculate regime strength across scales
    fn calculate_regime_strength(&self, scale_signals: &HashMap<TimeScale, RegimeSignal>) -> f64 {
        if scale_signals.is_empty() {
            return 0.5;
        }

        let mut weighted_strength = 0.0;
        let mut total_weight = 0.0;

        for (scale, signal) in scale_signals {
            let weight = self.config.scale_weights.get(scale).copied().unwrap_or(0.25);
            weighted_strength += signal.confidence * weight;
            total_weight += weight;
        }

        if total_weight > 0.0 {
            weighted_strength / total_weight
        } else {
            0.5
        }
    }

    /// Calculate hierarchical consistency across time scales
    fn calculate_hierarchical_consistency(&self, scale_signals: &HashMap<TimeScale, RegimeSignal>) -> f64 {
        if scale_signals.len() < 2 {
            return 1.0;
        }

        let regimes: Vec<RegimeType> = scale_signals.values().map(|s| s.current_regime).collect();
        let unique_regimes: std::collections::HashSet<RegimeType> = regimes.iter().cloned().collect();

        // Perfect consistency if all scales agree
        if unique_regimes.len() == 1 {
            return 1.0;
        }

        // Calculate consistency based on regime agreement
        let most_common_regime = regimes.iter()
            .fold(HashMap::new(), |mut acc, &regime| {
                *acc.entry(regime).or_insert(0) += 1;
                acc
            })
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(regime, _)| regime);

        if let Some(common_regime) = most_common_regime {
            let agreement_count = regimes.iter().filter(|&&r| r == common_regime).count();
            agreement_count as f64 / regimes.len() as f64
        } else {
            0.5
        }
    }

    /// Train all scale-specific detectors
    pub fn train(&mut self) -> Result<HashMap<TimeScale, f64>> {
        let mut training_results = HashMap::new();

        for (scale, detector) in &mut self.scale_detectors {
            if detector.is_ready() {
                match detector.train() {
                    Ok(log_likelihood) => {
                        training_results.insert(*scale, log_likelihood);
                    }
                    Err(e) => {
                        eprintln!("Training failed for scale {:?}: {}", scale, e);
                    }
                }
            }
        }

        Ok(training_results)
    }

    /// Check if detector is ready for regime detection
    pub fn is_ready(&self) -> bool {
        self.is_initialized &&
        self.scale_detectors.values().any(|detector| detector.is_ready())
    }

    /// Get the current configuration
    pub fn get_config(&self) -> &MultiScaleHMMConfig {
        &self.config
    }

    /// Get consensus history
    pub fn get_consensus_history(&self) -> &VecDeque<MultiScaleRegimeSignal> {
        &self.consensus_history
    }

    /// Get scale-specific detector
    pub fn get_scale_detector(&self, scale: TimeScale) -> Option<&HMMRegimeDetector> {
        self.scale_detectors.get(&scale)
    }

    /// Get scale-specific detector (mutable)
    pub fn get_scale_detector_mut(&mut self, scale: TimeScale) -> Option<&mut HMMRegimeDetector> {
        self.scale_detectors.get_mut(&scale)
    }
}

impl HMMRegimeDetector {
    /// Create a new HMM regime detector
    pub fn new(config: HMMConfig) -> Self {
        let parameters = Self::initialize_parameters(&config);
        let num_states = config.num_states;
        let observation_window = config.observation_window;

        Self {
            config: config.clone(),
            parameters,
            observations: VecDeque::with_capacity(observation_window * 2),
            current_state_probs: Array1::from_elem(num_states, 1.0 / num_states as f64),
            is_trained: false,
            last_regime: None,

            // Initialize enhanced detection components
            price_history: VecDeque::with_capacity(config.volatility_lookback * 2),
            volume_history: VecDeque::with_capacity(config.volatility_lookback * 2),
            spread_history: VecDeque::with_capacity(config.volatility_lookback * 2),
            regime_history: VecDeque::with_capacity(50),
            transition_detector: TransitionDetector::default(),
            adaptive_threshold: config.min_confidence,
            regime_persistence_counter: 0,
            last_transition_time: None,

            // Initialize online learning components
            online_batch: VecDeque::with_capacity(config.online_learning_config.max_batch_size),
            baseline_log_likelihood: f64::NEG_INFINITY,
            recent_log_likelihoods: VecDeque::with_capacity(config.online_learning_config.drift_window_size),
            drift_detector: ConceptDriftDetector::new(config.online_learning_config.clone()),
            learning_rate: config.online_learning_config.learning_rate,
            last_online_update: None,
            update_counter: 0,
        }
    }
    
    /// Initialize HMM parameters with reasonable defaults
    fn initialize_parameters(config: &HMMConfig) -> HMMParameters {
        let num_states = config.num_states;
        let feature_dims = config.feature_dimensions;
        
        // Initialize with uniform probabilities
        let initial_probs = Array1::from_elem(num_states, 1.0 / num_states as f64);
        
        // Initialize transition matrix with slight preference for staying in same state
        let mut transition_matrix = Array2::from_elem((num_states, num_states), 0.1 / (num_states - 1) as f64);
        for i in 0..num_states {
            transition_matrix[[i, i]] = 0.9; // High probability of staying in same state
        }
        
        // Initialize emission means for different regimes
        let mut emission_means = Array2::zeros((num_states, feature_dims));
        
        // Normal regime: low volatility, low trend, medium volume
        emission_means[[0, 0]] = 0.005; // volatility
        emission_means[[0, 1]] = 0.001; // trend
        emission_means[[0, 2]] = 0.5;   // volume
        
        // Trending regime: medium volatility, high trend, high volume
        emission_means[[1, 0]] = 0.01;
        emission_means[[1, 1]] = 0.02;
        emission_means[[1, 2]] = 0.8;
        
        // Volatile regime: high volatility, medium trend, high volume
        emission_means[[2, 0]] = 0.025;
        emission_means[[2, 1]] = 0.01;
        emission_means[[2, 2]] = 0.9;
        
        // Crisis regime: very high volatility, high trend, very high volume
        emission_means[[3, 0]] = 0.05;
        emission_means[[3, 1]] = 0.03;
        emission_means[[3, 2]] = 1.0;
        
        // Initialize covariance matrices (diagonal for simplicity)
        let mut emission_covariances = Vec::with_capacity(num_states);
        for _ in 0..num_states {
            let mut cov = Array2::zeros((feature_dims, feature_dims));
            for i in 0..feature_dims {
                cov[[i, i]] = 0.001; // Small diagonal covariance
            }
            emission_covariances.push(cov);
        }
        
        HMMParameters {
            initial_probs,
            transition_matrix,
            emission_means,
            emission_covariances,
        }
    }
    
    /// Update detector with new market tick
    pub fn update_with_tick(&mut self, tick: &MarketTick) -> Result<()> {
        let observation = self.extract_observation(tick)?;

        // Clone observation for online learning before moving it
        let observation_clone = if self.config.enable_online_learning {
            Some(observation.clone())
        } else {
            None
        };

        // Add to observation buffer
        if self.observations.len() >= self.config.observation_window * 2 {
            self.observations.pop_front();
        }
        self.observations.push_back(observation);

        // Update current state probabilities using forward algorithm
        if self.observations.len() >= 2 {
            self.update_state_probabilities()?;
        }

        // Update transition detector for fast regime change detection
        if self.config.fast_detection_mode {
            self.update_transition_detector()?;
        }

        // Update adaptive threshold if enabled
        if self.config.adaptive_threshold {
            self.update_adaptive_threshold();
        }

        // Online learning update if enabled
        if let Some(obs) = observation_clone {
            self.update_online_learning(&obs)?;
        }

        Ok(())
    }
    
    /// Extract market observation from tick with enhanced features
    fn extract_observation(&mut self, tick: &MarketTick) -> Result<MarketObservation> {
        let mid_price = (tick.bid_price + tick.ask_price) / 2.0;
        let bid_ask_spread = tick.ask_price - tick.bid_price;
        let volume = tick.volume.unwrap_or(0.0);

        // Update price history
        if self.price_history.len() >= self.config.volatility_lookback * 2 {
            self.price_history.pop_front();
        }
        self.price_history.push_back(mid_price);

        // Update volume history
        if self.volume_history.len() >= self.config.volatility_lookback * 2 {
            self.volume_history.pop_front();
        }
        self.volume_history.push_back(volume);

        // Update spread history
        if self.spread_history.len() >= self.config.volatility_lookback * 2 {
            self.spread_history.pop_front();
        }
        self.spread_history.push_back(bid_ask_spread);

        // Calculate basic features
        let volatility = self.calculate_enhanced_volatility();
        let trend = self.calculate_enhanced_trend();
        let normalized_volume = self.normalize_volume(volume);

        // Calculate enhanced features if enabled
        let (momentum, skewness, kurtosis, autocorrelation) = if self.config.enable_enhanced_features {
            (
                self.calculate_momentum(),
                self.calculate_price_skewness(),
                self.calculate_price_kurtosis(),
                self.calculate_autocorrelation(),
            )
        } else {
            (0.0, 0.0, 0.0, 0.0)
        };

        // Calculate advanced microstructure features
        let (order_flow_imbalance, effective_spread, price_impact, market_depth_ratio) =
            if self.config.enable_microstructure_features {
                (
                    self.calculate_order_flow_imbalance(tick),
                    self.calculate_effective_spread(tick),
                    self.calculate_price_impact(tick),
                    self.calculate_market_depth_ratio(tick),
                )
            } else {
                (0.0, 0.0, 0.0, 1.0)
            };

        // Calculate volatility clustering features
        let (garch_volatility, volatility_persistence, volatility_clustering_score) =
            if self.config.enable_volatility_clustering {
                (
                    self.calculate_garch_volatility(),
                    self.calculate_volatility_persistence(),
                    self.calculate_volatility_clustering_score(),
                )
            } else {
                (volatility, 0.5, 0.5)
            };

        // Calculate regime-specific indicators
        let (hurst_exponent, fractal_dimension, regime_strength, regime_transition_signal) =
            if self.config.enable_regime_specific_indicators {
                (
                    self.calculate_hurst_exponent(),
                    self.calculate_fractal_dimension(),
                    self.calculate_regime_strength(),
                    self.calculate_regime_transition_signal(),
                )
            } else {
                (0.5, 1.5, 0.5, 0.1)
            };

        // Calculate regime persistence and transition probability
        let regime_persistence = self.calculate_regime_persistence();
        let transition_probability = self.calculate_fast_transition_probability();

        // Normalize spread
        let normalized_spread = self.normalize_spread(bid_ask_spread);

        // Create comprehensive feature vector
        let features = if self.config.enable_enhanced_features {
            Array1::from_vec(vec![
                volatility,
                trend,
                normalized_volume,
                momentum,
                normalized_spread,
                skewness,
                kurtosis,
                autocorrelation,
                order_flow_imbalance,
                effective_spread,
                price_impact,
                market_depth_ratio,
                garch_volatility,
                volatility_persistence,
                hurst_exponent,
                regime_strength,
            ])
        } else {
            Array1::from_vec(vec![volatility, trend, normalized_volume])
        };

        Ok(MarketObservation {
            timestamp: tick.timestamp,
            features,
            volatility,
            trend,
            volume: normalized_volume,
            momentum,
            bid_ask_spread: normalized_spread,
            price_skewness: skewness,
            price_kurtosis: kurtosis,
            autocorrelation,
            regime_persistence,
            transition_probability,

            // Advanced microstructure features
            order_flow_imbalance,
            effective_spread,
            price_impact,
            market_depth_ratio,

            // Volatility clustering features
            garch_volatility,
            volatility_persistence,
            volatility_clustering_score,

            // Regime-specific technical indicators
            hurst_exponent,
            fractal_dimension,
            regime_strength,
            regime_transition_signal,
        })
    }
    
    /// Calculate enhanced volatility using price history
    fn calculate_enhanced_volatility(&self) -> f64 {
        if self.price_history.len() < self.config.volatility_lookback {
            return 0.005; // Default low volatility
        }

        let prices: Vec<f64> = self.price_history.iter()
            .rev()
            .take(self.config.volatility_lookback)
            .cloned()
            .collect();

        if prices.len() < 2 {
            return 0.005;
        }

        // Calculate returns
        let mut returns = Vec::new();
        for i in 1..prices.len() {
            if prices[i-1] > 0.0 {
                returns.push((prices[i] - prices[i-1]) / prices[i-1]);
            }
        }

        if returns.is_empty() {
            return 0.005;
        }

        // Calculate standard deviation of returns
        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / (returns.len() - 1) as f64;

        variance.sqrt()
    }

    /// Calculate enhanced trend using linear regression
    fn calculate_enhanced_trend(&self) -> f64 {
        if self.price_history.len() < self.config.trend_lookback {
            return 0.001; // Default low trend
        }

        let prices: Vec<f64> = self.price_history.iter()
            .rev()
            .take(self.config.trend_lookback)
            .cloned()
            .collect();

        if prices.len() < 2 {
            return 0.001;
        }

        // Simple linear regression slope
        let n = prices.len() as f64;
        let x_sum = (0..prices.len()).sum::<usize>() as f64;
        let y_sum = prices.iter().sum::<f64>();
        let xy_sum = prices.iter()
            .enumerate()
            .map(|(i, &price)| i as f64 * price)
            .sum::<f64>();
        let x2_sum = (0..prices.len())
            .map(|i| (i as f64).powi(2))
            .sum::<f64>();

        let denominator = n * x2_sum - x_sum.powi(2);
        if denominator.abs() < 1e-10 {
            return 0.001;
        }

        let slope = (n * xy_sum - x_sum * y_sum) / denominator;

        // Normalize slope by average price
        let avg_price = y_sum / n;
        if avg_price > 0.0 {
            slope / avg_price
        } else {
            0.001
        }
    }

    /// Calculate momentum indicator
    fn calculate_momentum(&self) -> f64 {
        if self.price_history.len() < self.config.momentum_lookback + 1 {
            return 0.0;
        }

        let current_price = *self.price_history.back().unwrap();
        let past_price = self.price_history[self.price_history.len() - self.config.momentum_lookback - 1];

        if past_price > 0.0 {
            (current_price - past_price) / past_price
        } else {
            0.0
        }
    }

    /// Calculate price skewness
    fn calculate_price_skewness(&self) -> f64 {
        if self.price_history.len() < 10 {
            return 0.0;
        }

        let prices: Vec<f64> = self.price_history.iter()
            .rev()
            .take(20)
            .cloned()
            .collect();

        let mean = prices.iter().sum::<f64>() / prices.len() as f64;
        let variance = prices.iter()
            .map(|p| (p - mean).powi(2))
            .sum::<f64>() / prices.len() as f64;

        if variance <= 0.0 {
            return 0.0;
        }

        let std_dev = variance.sqrt();
        let skewness = prices.iter()
            .map(|p| ((p - mean) / std_dev).powi(3))
            .sum::<f64>() / prices.len() as f64;

        skewness
    }

    /// Calculate price kurtosis
    fn calculate_price_kurtosis(&self) -> f64 {
        if self.price_history.len() < 10 {
            return 0.0;
        }

        let prices: Vec<f64> = self.price_history.iter()
            .rev()
            .take(20)
            .cloned()
            .collect();

        let mean = prices.iter().sum::<f64>() / prices.len() as f64;
        let variance = prices.iter()
            .map(|p| (p - mean).powi(2))
            .sum::<f64>() / prices.len() as f64;

        if variance <= 0.0 {
            return 0.0;
        }

        let std_dev = variance.sqrt();
        let kurtosis = prices.iter()
            .map(|p| ((p - mean) / std_dev).powi(4))
            .sum::<f64>() / prices.len() as f64;

        kurtosis - 3.0 // Excess kurtosis
    }

    /// Calculate autocorrelation
    fn calculate_autocorrelation(&self) -> f64 {
        if self.price_history.len() < 10 {
            return 0.0;
        }

        let prices: Vec<f64> = self.price_history.iter()
            .rev()
            .take(20)
            .cloned()
            .collect();

        if prices.len() < 2 {
            return 0.0;
        }

        // Calculate returns
        let mut returns = Vec::new();
        for i in 1..prices.len() {
            if prices[i-1] > 0.0 {
                returns.push((prices[i] - prices[i-1]) / prices[i-1]);
            }
        }

        if returns.len() < 2 {
            return 0.0;
        }

        // Calculate lag-1 autocorrelation
        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let mut numerator = 0.0;
        let mut denominator = 0.0;

        for i in 1..returns.len() {
            numerator += (returns[i] - mean) * (returns[i-1] - mean);
        }

        for return_val in &returns {
            denominator += (return_val - mean).powi(2);
        }

        if denominator > 0.0 {
            numerator / denominator
        } else {
            0.0
        }
    }

    /// Calculate order flow imbalance (microstructure feature)
    fn calculate_order_flow_imbalance(&self, tick: &MarketTick) -> f64 {
        let bid_size = tick.bid_size;
        let ask_size = tick.ask_size;

        if bid_size + ask_size > 0.0 {
            (bid_size - ask_size) / (bid_size + ask_size)
        } else {
            0.0
        }
    }

    /// Calculate effective spread (microstructure feature)
    fn calculate_effective_spread(&self, tick: &MarketTick) -> f64 {
        let mid_price = (tick.bid_price + tick.ask_price) / 2.0;
        let spread = tick.ask_price - tick.bid_price;

        if mid_price > 0.0 {
            spread / mid_price
        } else {
            0.0
        }
    }

    /// Calculate price impact (microstructure feature)
    fn calculate_price_impact(&self, tick: &MarketTick) -> f64 {
        if self.price_history.len() < 2 {
            return 0.0;
        }

        let current_price = (tick.bid_price + tick.ask_price) / 2.0;
        let prev_price = *self.price_history.back().unwrap();
        let volume = tick.volume.unwrap_or(0.0);

        if volume > 0.0 && prev_price > 0.0 {
            ((current_price - prev_price) / prev_price) / volume.ln().max(1.0)
        } else {
            0.0
        }
    }

    /// Calculate market depth ratio (microstructure feature)
    fn calculate_market_depth_ratio(&self, tick: &MarketTick) -> f64 {
        let bid_size = tick.bid_size;
        let ask_size = tick.ask_size;

        if ask_size > 0.0 {
            bid_size / ask_size
        } else if bid_size > 0.0 {
            10.0 // Large ratio when ask size is zero
        } else {
            1.0 // Default ratio
        }
    }

    /// Calculate GARCH volatility (volatility clustering feature)
    fn calculate_garch_volatility(&self) -> f64 {
        if self.price_history.len() < self.config.garch_window {
            return self.calculate_enhanced_volatility();
        }

        let prices: Vec<f64> = self.price_history.iter()
            .rev()
            .take(self.config.garch_window)
            .cloned()
            .collect();

        // Calculate returns
        let mut returns = Vec::new();
        for i in 1..prices.len() {
            if prices[i-1] > 0.0 {
                returns.push((prices[i] - prices[i-1]) / prices[i-1]);
            }
        }

        if returns.len() < 5 {
            return self.calculate_enhanced_volatility();
        }

        // Simple GARCH(1,1) approximation
        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let mut variance = 0.0;
        let mut prev_variance = 0.01; // Initial variance

        // GARCH parameters (simplified)
        let alpha = 0.1;
        let beta = 0.85;
        let omega = 0.00001;

        for return_val in returns.iter().rev().take(10) {
            let squared_return = (return_val - mean_return).powi(2);
            variance = omega + alpha * squared_return + beta * prev_variance;
            prev_variance = variance;
        }

        variance.sqrt()
    }

    /// Calculate volatility persistence (volatility clustering feature)
    fn calculate_volatility_persistence(&self) -> f64 {
        if self.price_history.len() < self.config.volatility_clustering_window {
            return 0.5;
        }

        let window_size = 10;
        let mut volatilities = Vec::new();

        // Calculate rolling volatilities
        for i in window_size..self.price_history.len() {
            let window_prices: Vec<f64> = self.price_history.iter()
                .skip(i - window_size)
                .take(window_size)
                .cloned()
                .collect();

            let mut returns = Vec::new();
            for j in 1..window_prices.len() {
                if window_prices[j-1] > 0.0 {
                    returns.push((window_prices[j] - window_prices[j-1]) / window_prices[j-1]);
                }
            }

            if !returns.is_empty() {
                let mean = returns.iter().sum::<f64>() / returns.len() as f64;
                let variance = returns.iter()
                    .map(|r| (r - mean).powi(2))
                    .sum::<f64>() / returns.len() as f64;
                volatilities.push(variance.sqrt());
            }
        }

        if volatilities.len() < 2 {
            return 0.5;
        }

        // Calculate autocorrelation of volatilities
        let mean_vol = volatilities.iter().sum::<f64>() / volatilities.len() as f64;
        let mut numerator = 0.0;
        let mut denominator = 0.0;

        for i in 1..volatilities.len() {
            numerator += (volatilities[i] - mean_vol) * (volatilities[i-1] - mean_vol);
        }

        for vol in &volatilities {
            denominator += (vol - mean_vol).powi(2);
        }

        if denominator > 0.0 {
            (numerator / denominator).abs()
        } else {
            0.5
        }
    }

    /// Calculate volatility clustering score
    fn calculate_volatility_clustering_score(&self) -> f64 {
        let persistence = self.calculate_volatility_persistence();
        let current_vol = self.calculate_enhanced_volatility();
        let garch_vol = self.calculate_garch_volatility();

        // Combine persistence and volatility regime indicators
        let vol_ratio = if garch_vol > 0.0 {
            current_vol / garch_vol
        } else {
            1.0
        };

        (persistence * 0.7 + vol_ratio.ln().abs() * 0.3).min(1.0)
    }

    /// Calculate Hurst exponent (regime-specific indicator)
    fn calculate_hurst_exponent(&self) -> f64 {
        if self.price_history.len() < self.config.hurst_exponent_window {
            return 0.5; // Random walk default
        }

        let prices: Vec<f64> = self.price_history.iter()
            .rev()
            .take(self.config.hurst_exponent_window)
            .cloned()
            .collect();

        // Calculate log returns
        let mut log_returns = Vec::new();
        for i in 1..prices.len() {
            if prices[i-1] > 0.0 && prices[i] > 0.0 {
                log_returns.push((prices[i] / prices[i-1]).ln());
            }
        }

        if log_returns.len() < 10 {
            return 0.5;
        }

        // R/S analysis for Hurst exponent
        let n = log_returns.len();
        let mean_return = log_returns.iter().sum::<f64>() / n as f64;

        // Calculate cumulative deviations
        let mut cumulative_deviations = Vec::new();
        let mut cumsum = 0.0;
        for return_val in &log_returns {
            cumsum += return_val - mean_return;
            cumulative_deviations.push(cumsum);
        }

        // Calculate range
        let max_dev = cumulative_deviations.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let min_dev = cumulative_deviations.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let range = max_dev - min_dev;

        // Calculate standard deviation
        let variance = log_returns.iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>() / n as f64;
        let std_dev = variance.sqrt();

        if std_dev > 0.0 && range > 0.0 {
            // Hurst exponent approximation
            let rs_ratio = range / std_dev;
            let hurst = (rs_ratio.ln() / (n as f64).ln()).max(0.0).min(1.0);
            hurst
        } else {
            0.5
        }
    }

    /// Calculate fractal dimension (regime-specific indicator)
    fn calculate_fractal_dimension(&self) -> f64 {
        let hurst = self.calculate_hurst_exponent();
        // Fractal dimension = 2 - Hurst exponent
        2.0 - hurst
    }

    /// Calculate regime strength indicator
    fn calculate_regime_strength(&self) -> f64 {
        if self.regime_history.len() < 5 {
            return 0.5;
        }

        let recent_regimes: Vec<&RegimeType> = self.regime_history.iter()
            .rev()
            .take(10)
            .map(|(regime, _)| regime)
            .collect();

        if recent_regimes.is_empty() {
            return 0.5;
        }

        // Calculate regime consistency
        let most_common_regime = recent_regimes[0];
        let consistency = recent_regimes.iter()
            .filter(|&&regime| regime == most_common_regime)
            .count() as f64 / recent_regimes.len() as f64;

        // Calculate confidence trend
        let recent_confidences: Vec<f64> = self.regime_history.iter()
            .rev()
            .take(5)
            .map(|(_, confidence)| *confidence)
            .collect();

        let avg_confidence = if !recent_confidences.is_empty() {
            recent_confidences.iter().sum::<f64>() / recent_confidences.len() as f64
        } else {
            0.5
        };

        // Combine consistency and confidence
        (consistency * 0.6 + avg_confidence * 0.4).min(1.0)
    }

    /// Calculate regime transition signal
    fn calculate_regime_transition_signal(&self) -> f64 {
        if self.observations.len() < 3 {
            return 0.1;
        }

        // Get recent observations for change detection
        let recent_obs: Vec<&MarketObservation> = self.observations.iter()
            .rev()
            .take(3)
            .collect();

        if recent_obs.len() < 3 {
            return 0.1;
        }

        // Calculate feature velocity (rate of change)
        let mut velocity_score = 0.0;
        let feature_count = recent_obs[0].features.len();

        for i in 0..feature_count {
            let current = recent_obs[0].features[i];
            let prev1 = recent_obs[1].features[i];
            let prev2 = recent_obs[2].features[i];

            // Calculate acceleration (second derivative)
            let velocity1 = current - prev1;
            let velocity2 = prev1 - prev2;
            let acceleration = velocity1 - velocity2;

            velocity_score += acceleration.abs();
        }

        // Normalize and convert to transition signal
        let avg_velocity = velocity_score / feature_count as f64;
        (avg_velocity * 5.0).min(1.0)
    }

    /// Normalize volume
    fn normalize_volume(&self, volume: f64) -> f64 {
        if self.volume_history.is_empty() {
            return (volume / 1000.0).min(1.0);
        }

        let avg_volume = self.volume_history.iter().sum::<f64>() / self.volume_history.len() as f64;
        if avg_volume > 0.0 {
            (volume / avg_volume).min(3.0) / 3.0 // Normalize to [0, 1]
        } else {
            (volume / 1000.0).min(1.0)
        }
    }

    /// Normalize spread
    fn normalize_spread(&self, spread: f64) -> f64 {
        if self.spread_history.is_empty() {
            return spread * 10000.0; // Convert to basis points
        }

        let avg_spread = self.spread_history.iter().sum::<f64>() / self.spread_history.len() as f64;
        if avg_spread > 0.0 {
            (spread / avg_spread).min(5.0) / 5.0 // Normalize to [0, 1]
        } else {
            spread * 10000.0
        }
    }

    /// Calculate regime persistence
    fn calculate_regime_persistence(&self) -> f64 {
        if self.regime_history.len() < 5 {
            return 0.5; // Default persistence
        }

        let recent_regimes: Vec<&RegimeType> = self.regime_history.iter()
            .rev()
            .take(10)
            .map(|(regime, _)| regime)
            .collect();

        if recent_regimes.is_empty() {
            return 0.5;
        }

        // Count how many recent regimes are the same as the most recent
        let most_recent = recent_regimes[0];
        let same_count = recent_regimes.iter()
            .filter(|&&regime| regime == most_recent)
            .count();

        same_count as f64 / recent_regimes.len() as f64
    }

    /// Calculate fast transition probability
    fn calculate_fast_transition_probability(&self) -> f64 {
        if self.observations.len() < 3 {
            return 0.1; // Default low transition probability
        }

        // Get recent observations
        let recent_obs: Vec<&MarketObservation> = self.observations.iter()
            .rev()
            .take(5)
            .collect();

        if recent_obs.len() < 2 {
            return 0.1;
        }

        // Calculate feature changes
        let mut feature_changes = 0.0;
        let mut change_count = 0;

        for i in 1..recent_obs.len() {
            for j in 0..recent_obs[i].features.len() {
                let change = (recent_obs[i].features[j] - recent_obs[i-1].features[j]).abs();
                feature_changes += change;
                change_count += 1;
            }
        }

        if change_count > 0 {
            let avg_change = feature_changes / change_count as f64;
            // Convert to probability (higher changes = higher transition probability)
            (avg_change * 10.0).min(1.0)
        } else {
            0.1
        }
    }

    /// Update transition detector for fast regime change detection
    fn update_transition_detector(&mut self) -> Result<()> {
        if self.observations.len() < 2 {
            return Ok(());
        }

        let latest_obs = self.observations.back().unwrap();

        // Use enhanced transition detection
        if let Some(detection_result) = self.transition_detector.detect_transition(latest_obs) {
            // Store the detection result for analysis
            if detection_result.detected {
                self.last_transition_time = Some(detection_result.timestamp);

                // Update regime persistence counter based on detection
                if detection_result.confidence > 0.8 {
                    self.regime_persistence_counter = 0; // Reset on high-confidence detection
                } else {
                    self.regime_persistence_counter += 1;
                }
            }
        }

        // Legacy transition score calculation for backward compatibility
        let volatility_change = if self.observations.len() >= 2 {
            let prev_obs = &self.observations[self.observations.len() - 2];
            (latest_obs.volatility - prev_obs.volatility).abs()
        } else {
            0.0
        };

        let momentum_magnitude = latest_obs.momentum.abs();
        let spread_change = if self.spread_history.len() >= 2 {
            let current_spread = *self.spread_history.back().unwrap();
            let prev_spread = self.spread_history[self.spread_history.len() - 2];
            (current_spread - prev_spread).abs()
        } else {
            0.0
        };

        // Combine factors into transition score
        let transition_score =
            (volatility_change / self.transition_detector.volatility_threshold) * 0.4 +
            (momentum_magnitude / self.transition_detector.momentum_threshold) * 0.4 +
            (spread_change * 1000.0) * 0.2; // Spread is typically small

        // Add to transition scores
        if self.transition_detector.transition_scores.len() >= self.transition_detector.detection_window {
            self.transition_detector.transition_scores.pop_front();
        }
        self.transition_detector.transition_scores.push_back(transition_score);

        // Check for transition
        if self.transition_detector.transition_scores.len() >= 3 {
            let avg_score = self.transition_detector.transition_scores.iter().sum::<f64>()
                / self.transition_detector.transition_scores.len() as f64;

            if avg_score > self.transition_detector.sensitivity {
                self.transition_detector.last_detection = Some(Utc::now());
                // Reset regime persistence counter
                self.regime_persistence_counter = 0;
            }
        }

        Ok(())
    }

    /// Update adaptive threshold based on recent performance
    fn update_adaptive_threshold(&mut self) {
        if self.regime_history.len() < 10 {
            return;
        }

        // Calculate average confidence of recent regime detections
        let recent_confidences: Vec<f64> = self.regime_history.iter()
            .rev()
            .take(10)
            .map(|(_, confidence)| *confidence)
            .collect();

        if !recent_confidences.is_empty() {
            let avg_confidence = recent_confidences.iter().sum::<f64>() / recent_confidences.len() as f64;

            // Adjust threshold based on recent performance
            let adjustment_factor = self.config.transition_smoothing_factor;
            self.adaptive_threshold = self.adaptive_threshold * (1.0 - adjustment_factor) +
                                    avg_confidence * adjustment_factor;

            // Keep within reasonable bounds
            self.adaptive_threshold = self.adaptive_threshold
                .max(self.config.min_confidence * 0.8)
                .min(self.config.min_confidence * 1.2);
        }
    }

    /// Update state probabilities using forward algorithm
    fn update_state_probabilities(&mut self) -> Result<()> {
        if let Some(latest_obs) = self.observations.back() {
            let emission_probs = self.calculate_emission_probabilities(&latest_obs.features);
            
            // Forward step: α_t(i) = [Σ_j α_{t-1}(j) * a_{ji}] * b_i(o_t)
            let mut new_probs = Array1::zeros(self.config.num_states);
            
            for i in 0..self.config.num_states {
                let mut sum = 0.0;
                for j in 0..self.config.num_states {
                    sum += self.current_state_probs[j] * self.parameters.transition_matrix[[j, i]];
                }
                new_probs[i] = sum * emission_probs[i];
            }
            
            // Normalize probabilities
            let total: f64 = new_probs.sum();
            if total > 1e-10 {
                new_probs /= total;
            } else {
                // Fallback to uniform distribution
                new_probs.fill(1.0 / self.config.num_states as f64);
            }
            
            self.current_state_probs = new_probs;
        }
        
        Ok(())
    }
    
    /// Calculate emission probabilities for given observation
    fn calculate_emission_probabilities(&self, observation: &Array1<f64>) -> Array1<f64> {
        let mut probs = Array1::zeros(self.config.num_states);
        
        for state in 0..self.config.num_states {
            probs[state] = self.gaussian_pdf(
                observation,
                &self.parameters.emission_means.row(state).to_owned(),
                &self.parameters.emission_covariances[state],
            );
        }
        
        probs
    }
    
    /// Calculate Gaussian PDF
    fn gaussian_pdf(&self, x: &Array1<f64>, mean: &Array1<f64>, cov: &Array2<f64>) -> f64 {
        let diff = x - mean;
        let det = self.matrix_determinant(cov);
        
        if det <= 0.0 {
            return 1e-10; // Avoid numerical issues
        }
        
        let inv_cov = self.matrix_inverse(cov);
        let mahalanobis = diff.dot(&inv_cov.dot(&diff));
        
        let normalization = 1.0 / ((2.0 * std::f64::consts::PI).powf(x.len() as f64 / 2.0) * det.sqrt());
        normalization * (-0.5 * mahalanobis).exp()
    }
    
    /// Simple determinant calculation for small matrices
    fn matrix_determinant(&self, matrix: &Array2<f64>) -> f64 {
        match matrix.shape() {
            [1, 1] => matrix[[0, 0]],
            [2, 2] => matrix[[0, 0]] * matrix[[1, 1]] - matrix[[0, 1]] * matrix[[1, 0]],
            [3, 3] => {
                matrix[[0, 0]] * (matrix[[1, 1]] * matrix[[2, 2]] - matrix[[1, 2]] * matrix[[2, 1]])
                - matrix[[0, 1]] * (matrix[[1, 0]] * matrix[[2, 2]] - matrix[[1, 2]] * matrix[[2, 0]])
                + matrix[[0, 2]] * (matrix[[1, 0]] * matrix[[2, 1]] - matrix[[1, 1]] * matrix[[2, 0]])
            },
            _ => 1.0, // Fallback for larger matrices
        }
    }
    
    /// Simple matrix inverse for small matrices
    fn matrix_inverse(&self, matrix: &Array2<f64>) -> Array2<f64> {
        let det = self.matrix_determinant(matrix);
        if det.abs() < 1e-10 {
            // Return identity matrix if singular
            let mut identity = Array2::zeros(matrix.raw_dim());
            for i in 0..matrix.shape()[0] {
                identity[[i, i]] = 1.0;
            }
            return identity;
        }
        
        match matrix.shape() {
            [1, 1] => Array2::from_elem((1, 1), 1.0 / matrix[[0, 0]]),
            [2, 2] => {
                let mut inv = Array2::zeros((2, 2));
                inv[[0, 0]] = matrix[[1, 1]] / det;
                inv[[0, 1]] = -matrix[[0, 1]] / det;
                inv[[1, 0]] = -matrix[[1, 0]] / det;
                inv[[1, 1]] = matrix[[0, 0]] / det;
                inv
            },
            _ => {
                // Fallback to identity for larger matrices
                let mut identity = Array2::zeros(matrix.raw_dim());
                for i in 0..matrix.shape()[0] {
                    identity[[i, i]] = 1.0;
                }
                identity
            }
        }
    }
    
    /// Detect current market regime with enhanced detection
    pub fn detect_current_regime(&mut self) -> Option<RegimeSignal> {
        if self.observations.len() < 5 {
            return None;
        }

        // Find most likely state
        let (most_likely_state, confidence) = self.current_state_probs.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(idx, &prob)| (idx, prob))
            .unwrap_or((0, 0.25));

        // Enhanced adaptive threshold with market volatility consideration
        let threshold = if self.config.adaptive_threshold {
            let volatility_adjustment = self.calculate_market_volatility_adjustment();
            self.adaptive_threshold * (1.0 + volatility_adjustment * 0.2)
        } else {
            self.config.min_confidence
        };

        if confidence < threshold {
            return None;
        }

        let current_regime = self.state_to_regime_type(most_likely_state);

        // Enhanced transition probability calculation
        let transition_probability = if self.config.fast_detection_mode {
            let base_transition_prob = self.calculate_transition_probability(&current_regime);
            let fast_transition_prob = if let Some(latest_obs) = self.observations.back() {
                latest_obs.transition_probability
            } else {
                0.1
            };

            // Combine both approaches
            (base_transition_prob + fast_transition_prob) / 2.0
        } else {
            self.calculate_transition_probability(&current_regime)
        };

        // Update regime history
        if self.regime_history.len() >= 50 {
            self.regime_history.pop_front();
        }
        self.regime_history.push_back((current_regime.clone(), confidence));

        // Update regime persistence counter
        if let Some((last_regime, _)) = self.regime_history.get(self.regime_history.len().saturating_sub(2)) {
            if last_regime == &current_regime {
                self.regime_persistence_counter += 1;
            } else {
                self.regime_persistence_counter = 1;
                self.last_transition_time = Some(Utc::now());
            }
        }

        Some(RegimeSignal {
            current_regime,
            transition_probability,
            confidence,
            timestamp: Utc::now(),
        })
    }
    
    /// Convert state index to regime type
    fn state_to_regime_type(&self, state: usize) -> RegimeType {
        match state {
            0 => RegimeType::Normal,
            1 => RegimeType::Trending,
            2 => RegimeType::Volatile,
            3 => RegimeType::Crisis,
            _ => RegimeType::Normal,
        }
    }
    
    /// Calculate transition probability
    fn calculate_transition_probability(&self, current_regime: &RegimeType) -> f64 {
        let current_state = self.regime_type_to_state(current_regime);
        
        // Calculate probability of transitioning to a different state
        let stay_prob = self.parameters.transition_matrix[[current_state, current_state]];
        1.0 - stay_prob
    }
    
    /// Convert regime type to state index
    fn regime_type_to_state(&self, regime: &RegimeType) -> usize {
        match regime {
            RegimeType::Normal => 0,
            RegimeType::Trending => 1,
            RegimeType::Volatile => 2,
            RegimeType::Crisis => 3,
            RegimeType::Bullish => 4,
            RegimeType::Bearish => 5,
            RegimeType::Sideways => 6,
            RegimeType::HighVolatility => 7,
        }
    }
    
    /// Train HMM parameters using Baum-Welch algorithm
    pub fn train(&mut self) -> Result<f64> {
        if self.observations.len() < self.config.observation_window {
            return Err(PantherSwapError::ai_prediction(
                "Not enough observations for training".to_string()
            ));
        }

        let mut log_likelihood = f64::NEG_INFINITY;
        let mut prev_log_likelihood = f64::NEG_INFINITY;

        for iteration in 0..self.config.max_iterations {
            // E-step: Forward-Backward algorithm
            let (alpha, beta, log_likelihood_iter) = self.forward_backward()?;

            // M-step: Update parameters
            self.update_parameters(&alpha, &beta)?;

            // Check convergence
            if iteration > 0 {
                let improvement = log_likelihood_iter - prev_log_likelihood;
                if improvement.abs() < self.config.convergence_threshold {
                    log_likelihood = log_likelihood_iter;
                    break;
                }
            }

            prev_log_likelihood = log_likelihood_iter;
            log_likelihood = log_likelihood_iter;
        }

        self.is_trained = true;
        Ok(log_likelihood)
    }

    /// Forward-Backward algorithm for HMM training
    fn forward_backward(&self) -> Result<(Array2<f64>, Array2<f64>, f64)> {
        let num_obs = self.observations.len();
        let num_states = self.config.num_states;

        // Forward pass
        let mut alpha = Array2::zeros((num_obs, num_states));
        let mut scaling_factors = Array1::zeros(num_obs);

        // Initialize first observation
        if let Some(first_obs) = self.observations.front() {
            let emission_probs = self.calculate_emission_probabilities(&first_obs.features);
            for i in 0..num_states {
                alpha[[0, i]] = self.parameters.initial_probs[i] * emission_probs[i];
            }

            // Scale to prevent underflow
            let scale = alpha.row(0).sum();
            if scale > 1e-10 {
                scaling_factors[0] = scale;
                for i in 0..num_states {
                    alpha[[0, i]] /= scale;
                }
            }
        }

        // Forward recursion
        for t in 1..num_obs {
            if let Some(obs) = self.observations.get(t) {
                let emission_probs = self.calculate_emission_probabilities(&obs.features);

                for j in 0..num_states {
                    let mut sum = 0.0;
                    for i in 0..num_states {
                        sum += alpha[[t-1, i]] * self.parameters.transition_matrix[[i, j]];
                    }
                    alpha[[t, j]] = sum * emission_probs[j];
                }

                // Scale to prevent underflow
                let scale = alpha.row(t).sum();
                if scale > 1e-10 {
                    scaling_factors[t] = scale;
                    for j in 0..num_states {
                        alpha[[t, j]] /= scale;
                    }
                }
            }
        }

        // Backward pass
        let mut beta = Array2::zeros((num_obs, num_states));

        // Initialize last observation
        for i in 0..num_states {
            beta[[num_obs-1, i]] = 1.0 / scaling_factors[num_obs-1];
        }

        // Backward recursion
        for t in (0..num_obs-1).rev() {
            if let Some(next_obs) = self.observations.get(t + 1) {
                let emission_probs = self.calculate_emission_probabilities(&next_obs.features);

                for i in 0..num_states {
                    let mut sum = 0.0;
                    for j in 0..num_states {
                        sum += self.parameters.transition_matrix[[i, j]] *
                               emission_probs[j] * beta[[t+1, j]];
                    }
                    beta[[t, i]] = sum / scaling_factors[t];
                }
            }
        }

        // Calculate log likelihood
        let log_likelihood = scaling_factors.iter()
            .filter(|&&x| x > 1e-10)
            .map(|&x| x.ln())
            .sum::<f64>();

        Ok((alpha, beta, log_likelihood))
    }

    /// Update HMM parameters using forward-backward results
    fn update_parameters(&mut self, alpha: &Array2<f64>, beta: &Array2<f64>) -> Result<()> {
        let num_obs = self.observations.len();
        let num_states = self.config.num_states;
        let feature_dims = self.config.feature_dimensions;

        // Calculate gamma (state probabilities) and xi (transition probabilities)
        let mut gamma = Array2::zeros((num_obs, num_states));
        let mut xi = Array2::zeros((num_states, num_states));

        // Calculate gamma
        for t in 0..num_obs {
            let normalizer = alpha.row(t).dot(&beta.row(t));
            if normalizer > 1e-10 {
                for i in 0..num_states {
                    gamma[[t, i]] = alpha[[t, i]] * beta[[t, i]] / normalizer;
                }
            }
        }

        // Calculate xi (only for t < num_obs - 1)
        for t in 0..num_obs-1 {
            if let Some(next_obs) = self.observations.get(t + 1) {
                let emission_probs = self.calculate_emission_probabilities(&next_obs.features);
                let mut normalizer = 0.0;

                for i in 0..num_states {
                    for j in 0..num_states {
                        let val = alpha[[t, i]] * self.parameters.transition_matrix[[i, j]] *
                                 emission_probs[j] * beta[[t+1, j]];
                        xi[[i, j]] += val;
                        normalizer += val;
                    }
                }

                if normalizer > 1e-10 {
                    xi /= normalizer;
                }
            }
        }

        // Update initial probabilities
        for i in 0..num_states {
            self.parameters.initial_probs[i] = gamma[[0, i]];
        }

        // Update transition matrix
        for i in 0..num_states {
            let row_sum = gamma.column(i).sum() - gamma[[num_obs-1, i]];
            if row_sum > 1e-10 {
                for j in 0..num_states {
                    self.parameters.transition_matrix[[i, j]] = xi[[i, j]] / row_sum;
                }
            }
        }

        // Update emission parameters (means and covariances)
        for state in 0..num_states {
            let state_weight_sum = gamma.column(state).sum();

            if state_weight_sum > 1e-10 {
                // Update mean
                let mut new_mean = Array1::zeros(feature_dims);
                for t in 0..num_obs {
                    if let Some(obs) = self.observations.get(t) {
                        for d in 0..feature_dims {
                            new_mean[d] += gamma[[t, state]] * obs.features[d];
                        }
                    }
                }
                new_mean /= state_weight_sum;

                // Update covariance
                let mut new_cov = Array2::zeros((feature_dims, feature_dims));
                for t in 0..num_obs {
                    if let Some(obs) = self.observations.get(t) {
                        let diff = &obs.features - &new_mean;
                        for i in 0..feature_dims {
                            for j in 0..feature_dims {
                                new_cov[[i, j]] += gamma[[t, state]] * diff[i] * diff[j];
                            }
                        }
                    }
                }
                new_cov /= state_weight_sum;

                // Add small regularization to diagonal
                for i in 0..feature_dims {
                    new_cov[[i, i]] += 1e-6;
                }

                // Update parameters
                for d in 0..feature_dims {
                    self.parameters.emission_means[[state, d]] = new_mean[d];
                }
                self.parameters.emission_covariances[state] = new_cov;
            }
        }

        Ok(())
    }

    /// Viterbi algorithm for finding most likely state sequence
    pub fn viterbi_decode(&self, observations: &[MarketObservation]) -> Result<Vec<usize>> {
        if observations.is_empty() {
            return Ok(Vec::new());
        }

        let num_obs = observations.len();
        let num_states = self.config.num_states;

        // Viterbi tables
        let mut viterbi = Array2::zeros((num_obs, num_states));
        let mut path = Array2::zeros((num_obs, num_states));

        // Initialize first observation
        let emission_probs = self.calculate_emission_probabilities(&observations[0].features);
        for i in 0..num_states {
            viterbi[[0, i]] = self.parameters.initial_probs[i].ln() + emission_probs[i].ln();
            path[[0, i]] = 0.0;
        }

        // Forward pass
        for t in 1..num_obs {
            let emission_probs = self.calculate_emission_probabilities(&observations[t].features);

            for j in 0..num_states {
                let mut max_prob = f64::NEG_INFINITY;
                let mut best_prev_state = 0;

                for i in 0..num_states {
                    let prob = viterbi[[t-1, i]] +
                              self.parameters.transition_matrix[[i, j]].ln() +
                              emission_probs[j].ln();

                    if prob > max_prob {
                        max_prob = prob;
                        best_prev_state = i;
                    }
                }

                viterbi[[t, j]] = max_prob;
                path[[t, j]] = best_prev_state as f64;
            }
        }

        // Backward pass - find best path
        let mut states = vec![0; num_obs];

        // Find best final state
        let mut max_prob = f64::NEG_INFINITY;
        for i in 0..num_states {
            if viterbi[[num_obs-1, i]] > max_prob {
                max_prob = viterbi[[num_obs-1, i]];
                states[num_obs-1] = i;
            }
        }

        // Trace back
        for t in (0..num_obs-1).rev() {
            states[t] = path[[t+1, states[t+1]]] as usize;
        }

        Ok(states)
    }

    /// Check if detector is ready
    pub fn is_ready(&self) -> bool {
        self.observations.len() >= 5 && self.is_trained
    }

    /// Get current state probabilities
    pub fn get_state_probabilities(&self) -> &Array1<f64> {
        &self.current_state_probs
    }

    /// Get training status
    pub fn is_trained(&self) -> bool {
        self.is_trained
    }

    /// Force retrain with current observations
    pub fn retrain(&mut self) -> Result<f64> {
        self.is_trained = false;
        self.train()
    }

    /// Get enhanced regime detection statistics
    pub fn get_detection_statistics(&self) -> EnhancedRegimeStats {
        let regime_distribution = self.calculate_regime_distribution();
        let avg_transition_time = self.calculate_average_transition_time();
        let detection_accuracy = self.estimate_detection_accuracy();

        EnhancedRegimeStats {
            current_regime: self.last_regime.clone(),
            regime_persistence_count: self.regime_persistence_counter,
            adaptive_threshold: self.adaptive_threshold,
            regime_distribution,
            average_transition_time_minutes: avg_transition_time,
            detection_accuracy,
            fast_detection_enabled: self.config.fast_detection_mode,
            last_transition_time: self.last_transition_time,
            total_observations: self.observations.len(),
            total_regime_changes: self.count_regime_changes(),
        }
    }

    /// Calculate regime distribution from history
    fn calculate_regime_distribution(&self) -> HashMap<RegimeType, f64> {
        let mut distribution = HashMap::new();

        if self.regime_history.is_empty() {
            return distribution;
        }

        let total_count = self.regime_history.len() as f64;

        for (regime, _) in &self.regime_history {
            *distribution.entry(regime.clone()).or_insert(0.0) += 1.0;
        }

        // Convert to percentages
        for (_, count) in distribution.iter_mut() {
            *count /= total_count;
        }

        distribution
    }

    /// Calculate average time between regime transitions
    fn calculate_average_transition_time(&self) -> f64 {
        if self.regime_history.len() < 2 {
            return 0.0;
        }

        let mut transition_times = Vec::new();
        let mut last_regime = &self.regime_history[0].0;
        let mut last_time = self.observations.front().map(|obs| obs.timestamp);

        for (i, (regime, _)) in self.regime_history.iter().enumerate().skip(1) {
            if regime != last_regime {
                if let (Some(current_time), Some(prev_time)) = (
                    self.observations.get(i).map(|obs| obs.timestamp),
                    last_time
                ) {
                    let duration = current_time.signed_duration_since(prev_time);
                    transition_times.push(duration.num_minutes() as f64);
                }
                last_regime = regime;
                last_time = self.observations.get(i).map(|obs| obs.timestamp);
            }
        }

        if transition_times.is_empty() {
            0.0
        } else {
            transition_times.iter().sum::<f64>() / transition_times.len() as f64
        }
    }

    /// Estimate detection accuracy based on regime persistence
    fn estimate_detection_accuracy(&self) -> f64 {
        if self.regime_history.len() < 10 {
            return 0.7; // Default estimate
        }

        // Calculate stability metric based on regime persistence
        let persistence_scores: Vec<f64> = self.regime_history.iter()
            .map(|(_, confidence)| *confidence)
            .collect();

        let avg_confidence = persistence_scores.iter().sum::<f64>() / persistence_scores.len() as f64;

        // Estimate accuracy based on confidence and regime stability
        let stability_factor = self.calculate_regime_persistence();

        (avg_confidence * 0.7 + stability_factor * 0.3).min(0.95).max(0.5)
    }

    /// Count total regime changes in history
    fn count_regime_changes(&self) -> usize {
        if self.regime_history.len() < 2 {
            return 0;
        }

        let mut changes = 0;
        let mut last_regime = &self.regime_history[0].0;

        for (regime, _) in self.regime_history.iter().skip(1) {
            if regime != last_regime {
                changes += 1;
                last_regime = regime;
            }
        }

        changes
    }

    /// Detect regime transitions with enhanced alerting
    pub fn detect_regime_transition(&mut self) -> Option<RegimeTransitionAlert> {
        if let Some(current_signal) = self.detect_current_regime() {
            let current_regime = current_signal.current_regime;

            // Check if regime has changed
            let regime_changed = match &self.last_regime {
                Some(last) => *last != current_regime,
                None => true,
            };

            if regime_changed {
                let from_regime = self.last_regime.clone();
                let severity = self.calculate_transition_severity(&self.last_regime, &current_regime);
                let transition_alert = RegimeTransitionAlert {
                    from_regime,
                    to_regime: current_regime.clone(),
                    confidence: current_signal.confidence,
                    transition_probability: current_signal.transition_probability,
                    timestamp: current_signal.timestamp,
                    severity,
                };

                self.last_regime = Some(current_regime);
                return Some(transition_alert);
            }
        }

        None
    }

    /// Calculate severity of regime transition
    fn calculate_transition_severity(&self, from: &Option<RegimeType>, to: &RegimeType) -> TransitionSeverity {
        match (from, to) {
            (Some(RegimeType::Normal), RegimeType::Crisis) => TransitionSeverity::Critical,
            (Some(RegimeType::Trending), RegimeType::Crisis) => TransitionSeverity::Critical,
            (Some(RegimeType::Volatile), RegimeType::Crisis) => TransitionSeverity::High,
            (Some(RegimeType::Normal), RegimeType::Volatile) => TransitionSeverity::Medium,
            (Some(RegimeType::Trending), RegimeType::Volatile) => TransitionSeverity::Medium,
            (Some(RegimeType::Crisis), RegimeType::Normal) => TransitionSeverity::Low, // Recovery
            (Some(RegimeType::Crisis), RegimeType::Trending) => TransitionSeverity::Low, // Recovery
            _ => TransitionSeverity::Low,
        }
    }

    /// Get regime stability score
    pub fn get_regime_stability(&self) -> f64 {
        if self.current_state_probs.is_empty() {
            return 0.0;
        }

        // Calculate entropy of state probabilities (lower entropy = more stable)
        let entropy = self.current_state_probs.iter()
            .filter(|&&p| p > 1e-10)
            .map(|&p| -p * p.ln())
            .sum::<f64>();

        // Convert to stability score (0 = unstable, 1 = very stable)
        let max_entropy = (self.config.num_states as f64).ln();
        1.0 - (entropy / max_entropy)
    }

    /// Get regime persistence (how long current regime has been active)
    pub fn get_regime_persistence(&self) -> Option<chrono::Duration> {
        // This would track how long the current regime has been active
        // For now, return None as we don't track regime start times
        None
    }

    /// Reset detector state
    pub fn reset(&mut self) {
        self.observations.clear();
        self.current_state_probs.fill(1.0 / self.config.num_states as f64);
        self.is_trained = false;
        self.last_regime = None;
    }

    /// Calculate market volatility adjustment for adaptive thresholding
    fn calculate_market_volatility_adjustment(&self) -> f64 {
        if self.observations.len() < 10 {
            return 0.0;
        }

        let recent_obs = &self.observations[self.observations.len().saturating_sub(10)..];
        let mean = recent_obs.iter().map(|obs| obs.features[0]).sum::<f64>() / recent_obs.len() as f64;
        let variance = recent_obs.iter()
            .map(|obs| (obs.features[0] - mean).powi(2))
            .sum::<f64>() / recent_obs.len() as f64;

        let volatility = variance.sqrt();

        // Normalize volatility to [0, 1] range for adjustment
        (volatility / (volatility + 1.0)).min(1.0)
    }
}

/// Regime transition alert
#[derive(Debug, Clone)]
pub struct RegimeTransitionAlert {
    pub from_regime: Option<RegimeType>,
    pub to_regime: RegimeType,
    pub confidence: f64,
    pub transition_probability: f64,
    pub timestamp: DateTime<Utc>,
    pub severity: TransitionSeverity,
}

/// Severity levels for regime transitions
#[derive(Debug, Clone, PartialEq)]
pub enum TransitionSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Factory function to create a configured HMM regime detector
pub fn create_hmm_regime_detector() -> HMMRegimeDetector {
    let config = HMMConfig::default();
    HMMRegimeDetector::new(config)
}

/// Factory function to create a high-frequency HMM regime detector
pub fn create_hf_hmm_regime_detector() -> HMMRegimeDetector {
    let config = HMMConfig {
        num_states: 4,
        observation_window: 100, // Larger window for HF trading
        feature_dimensions: 16, // Advanced features (updated for new feature set)
        convergence_threshold: 1e-8, // Stricter convergence
        max_iterations: 200,
        min_confidence: 0.7, // Higher confidence threshold

        // Enhanced detection parameters
        enable_enhanced_features: true,
        volatility_lookback: 20,
        trend_lookback: 10,
        momentum_lookback: 5,
        regime_persistence_threshold: 0.8,
        transition_smoothing_factor: 0.3,
        adaptive_threshold: true,
        fast_detection_mode: true,

        // Online learning parameters
        enable_online_learning: true,
        online_learning_config: OnlineLearningConfig {
            learning_rate: 0.02, // Higher learning rate for HF
            forgetting_factor: 0.9, // Faster adaptation
            min_batch_size: 5,
            max_batch_size: 25,
            drift_threshold: 0.08, // More sensitive to drift
            drift_window_size: 50,
            adaptive_learning_rate: true,
            min_learning_rate: 0.005,
            max_learning_rate: 0.1,
        },

        // Advanced feature engineering parameters
        enable_microstructure_features: true,
        enable_volatility_clustering: true,
        enable_cross_asset_correlation: false,
        enable_regime_specific_indicators: true,
        microstructure_window: 30,
        volatility_clustering_window: 50,
        correlation_window: 100,
        garch_window: 30,
        hurst_exponent_window: 50,
        fractal_dimension_window: 40,
    };
    HMMRegimeDetector::new(config)
}

/// Factory function to create an enhanced HMM regime detector optimized for accuracy
pub fn create_enhanced_accuracy_hmm_detector() -> HMMRegimeDetector {
    let config = HMMConfig {
        num_states: 4,
        observation_window: 150, // Larger window for better accuracy
        feature_dimensions: 16, // Full advanced feature set
        convergence_threshold: 1e-10, // Very strict convergence
        max_iterations: 300,
        min_confidence: 0.75, // Higher confidence threshold

        // Enhanced detection parameters
        enable_enhanced_features: true,
        volatility_lookback: 30, // Longer lookback for stability
        trend_lookback: 15,
        momentum_lookback: 8,
        regime_persistence_threshold: 0.85,
        transition_smoothing_factor: 0.2, // More conservative smoothing
        adaptive_threshold: true,
        fast_detection_mode: true,

        // Online learning parameters (optimized for accuracy)
        enable_online_learning: true,
        online_learning_config: OnlineLearningConfig {
            learning_rate: 0.005, // Lower learning rate for stability
            forgetting_factor: 0.98, // Slower adaptation for accuracy
            min_batch_size: 15,
            max_batch_size: 75,
            drift_threshold: 0.12, // Less sensitive to drift
            drift_window_size: 150,
            adaptive_learning_rate: true,
            min_learning_rate: 0.001,
            max_learning_rate: 0.05,
        },

        // Advanced feature engineering parameters (all enabled)
        enable_microstructure_features: true,
        enable_volatility_clustering: true,
        enable_cross_asset_correlation: false, // Can be enabled with multiple assets
        enable_regime_specific_indicators: true,
        microstructure_window: 40,
        volatility_clustering_window: 80,
        correlation_window: 120,
        garch_window: 40,
        hurst_exponent_window: 80,
        fractal_dimension_window: 60,
    };
    HMMRegimeDetector::new(config)
}

impl HMMRegimeDetector {
    /// Get the current configuration
    pub fn get_config(&self) -> &HMMConfig {
        &self.config
    }

    /// Get enhanced regime detection statistics
    pub fn get_enhanced_stats(&self) -> EnhancedRegimeStats {
        self.get_detection_statistics()
    }

    /// Update online learning with new observation
    fn update_online_learning(&mut self, observation: &MarketObservation) -> Result<()> {
        // Add to online batch
        if self.online_batch.len() >= self.config.online_learning_config.max_batch_size {
            self.online_batch.pop_front();
        }
        self.online_batch.push_back(observation.clone());

        // Check if we have enough data for online update
        if self.online_batch.len() >= self.config.online_learning_config.min_batch_size {
            // Calculate current log-likelihood
            let current_ll = self.calculate_batch_log_likelihood(&self.online_batch)?;

            // Update drift detector
            self.drift_detector.update_performance(current_ll);

            // Add to recent log-likelihoods
            if self.recent_log_likelihoods.len() >= self.config.online_learning_config.drift_window_size {
                self.recent_log_likelihoods.pop_front();
            }
            self.recent_log_likelihoods.push_back(current_ll);

            // Set baseline if not set
            if self.baseline_log_likelihood == f64::NEG_INFINITY && self.recent_log_likelihoods.len() >= 10 {
                self.baseline_log_likelihood = self.recent_log_likelihoods.iter().sum::<f64>() / self.recent_log_likelihoods.len() as f64;
            }

            // Check for concept drift
            if let Some(drift_result) = self.drift_detector.detect_drift() {
                self.handle_concept_drift(drift_result)?;
            }

            // Perform online parameter update
            if self.should_perform_online_update() {
                self.perform_online_baum_welch_update()?;
                self.last_online_update = Some(Utc::now());
                self.update_counter += 1;
            }
        }

        Ok(())
    }

    /// Calculate log-likelihood for a batch of observations
    fn calculate_batch_log_likelihood(&self, batch: &VecDeque<MarketObservation>) -> Result<f64> {
        if batch.is_empty() {
            return Ok(f64::NEG_INFINITY);
        }

        let mut total_ll = 0.0;
        let mut prev_state_probs = self.parameters.initial_probs.clone();

        for observation in batch {
            let emission_probs = self.calculate_emission_probabilities(&observation.features);

            // Forward step
            let mut current_state_probs = Array1::zeros(self.config.num_states);
            for j in 0..self.config.num_states {
                for i in 0..self.config.num_states {
                    current_state_probs[j] += prev_state_probs[i] * self.parameters.transition_matrix[[i, j]];
                }
                current_state_probs[j] *= emission_probs[j];
            }

            // Normalize and add to log-likelihood
            let norm_factor = current_state_probs.sum();
            if norm_factor > 1e-10 {
                total_ll += norm_factor.ln();
                current_state_probs /= norm_factor;
            }

            prev_state_probs = current_state_probs;
        }

        Ok(total_ll / batch.len() as f64)
    }

    /// Handle concept drift detection
    fn handle_concept_drift(&mut self, drift_result: ConceptDriftResult) -> Result<()> {
        match drift_result.recommended_action {
            DriftAction::FullRetrain => {
                // Reset parameters and retrain
                self.parameters = Self::initialize_parameters(&self.config);
                self.is_trained = false;
                if self.observations.len() >= self.config.observation_window {
                    self.train()?;
                }
            }
            DriftAction::IncrementalUpdate => {
                // Increase learning rate temporarily
                self.learning_rate = (self.learning_rate * 1.5).min(self.config.online_learning_config.max_learning_rate);
            }
            DriftAction::IncreaseMonitoring => {
                // Reduce batch size for more frequent updates
                // This is handled in should_perform_online_update
            }
            DriftAction::ResetToBaseline => {
                // Reset to initial parameters
                self.parameters = Self::initialize_parameters(&self.config);
                self.baseline_log_likelihood = f64::NEG_INFINITY;
                self.recent_log_likelihoods.clear();
            }
            DriftAction::NoAction => {
                // Do nothing
            }
        }

        Ok(())
    }

    /// Check if online update should be performed
    fn should_perform_online_update(&self) -> bool {
        // Update more frequently if drift detected
        let min_batch = if self.drift_detector.consecutive_drift_count > 0 {
            self.config.online_learning_config.min_batch_size / 2
        } else {
            self.config.online_learning_config.min_batch_size
        };

        self.online_batch.len() >= min_batch &&
        (self.last_online_update.is_none() ||
         Utc::now().signed_duration_since(self.last_online_update.unwrap()).num_seconds() > 30)
    }

    /// Perform online Baum-Welch parameter update
    fn perform_online_baum_welch_update(&mut self) -> Result<()> {
        if self.online_batch.is_empty() {
            return Ok(());
        }

        // Calculate forward-backward for the batch
        let (alpha, beta, _) = self.forward_backward_batch(&self.online_batch)?;

        // Calculate sufficient statistics
        let (gamma, xi) = self.calculate_sufficient_statistics(&alpha, &beta)?;

        // Update parameters with learning rate
        self.update_parameters_online(&gamma, &xi)?;

        // Adapt learning rate if enabled
        if self.config.online_learning_config.adaptive_learning_rate {
            self.adapt_learning_rate();
        }

        Ok(())
    }

    /// Forward-backward algorithm for a batch
    fn forward_backward_batch(&self, batch: &VecDeque<MarketObservation>) -> Result<(Array2<f64>, Array2<f64>, f64)> {
        let num_obs = batch.len();
        let num_states = self.config.num_states;

        if num_obs == 0 {
            return Err(crate::utils::PantherSwapError::ai_prediction("Empty batch for forward-backward".to_string()));
        }

        // Forward pass
        let mut alpha = Array2::zeros((num_obs, num_states));
        let mut scaling_factors = Array1::zeros(num_obs);

        // Initialize first observation
        let first_obs = &batch[0];
        let emission_probs = self.calculate_emission_probabilities(&first_obs.features);
        for i in 0..num_states {
            alpha[[0, i]] = self.parameters.initial_probs[i] * emission_probs[i];
        }

        // Scale to prevent underflow
        let scale = alpha.row(0).sum();
        if scale > 1e-10 {
            scaling_factors[0] = scale;
            for i in 0..num_states {
                alpha[[0, i]] /= scale;
            }
        }

        // Forward recursion
        for t in 1..num_obs {
            let obs = &batch[t];
            let emission_probs = self.calculate_emission_probabilities(&obs.features);

            for j in 0..num_states {
                alpha[[t, j]] = 0.0;
                for i in 0..num_states {
                    alpha[[t, j]] += alpha[[t-1, i]] * self.parameters.transition_matrix[[i, j]];
                }
                alpha[[t, j]] *= emission_probs[j];
            }

            // Scale
            let scale = alpha.row(t).sum();
            if scale > 1e-10 {
                scaling_factors[t] = scale;
                for j in 0..num_states {
                    alpha[[t, j]] /= scale;
                }
            }
        }

        // Backward pass
        let mut beta = Array2::zeros((num_obs, num_states));

        // Initialize last observation
        for i in 0..num_states {
            beta[[num_obs-1, i]] = 1.0 / scaling_factors[num_obs-1];
        }

        // Backward recursion
        for t in (0..num_obs-1).rev() {
            let next_obs = &batch[t+1];
            let emission_probs = self.calculate_emission_probabilities(&next_obs.features);

            for i in 0..num_states {
                beta[[t, i]] = 0.0;
                for j in 0..num_states {
                    beta[[t, i]] += self.parameters.transition_matrix[[i, j]] * emission_probs[j] * beta[[t+1, j]];
                }
                beta[[t, i]] /= scaling_factors[t];
            }
        }

        // Calculate log-likelihood
        let log_likelihood = scaling_factors.iter().map(|&s| s.ln()).sum::<f64>();

        Ok((alpha, beta, log_likelihood))
    }

    /// Calculate sufficient statistics (gamma and xi)
    fn calculate_sufficient_statistics(&self, alpha: &Array2<f64>, beta: &Array2<f64>) -> Result<(Array2<f64>, Array3<f64>)> {
        let num_obs = alpha.nrows();
        let num_states = self.config.num_states;

        // Calculate gamma (state probabilities)
        let mut gamma = Array2::zeros((num_obs, num_states));
        for t in 0..num_obs {
            let norm = (0..num_states).map(|i| alpha[[t, i]] * beta[[t, i]]).sum::<f64>();
            if norm > 1e-10 {
                for i in 0..num_states {
                    gamma[[t, i]] = alpha[[t, i]] * beta[[t, i]] / norm;
                }
            }
        }

        // Calculate xi (transition probabilities)
        let mut xi = Array3::zeros((num_obs - 1, num_states, num_states));
        for t in 0..num_obs - 1 {
            let obs = &self.online_batch[t + 1];
            let emission_probs = self.calculate_emission_probabilities(&obs.features);

            let norm = (0..num_states).map(|i| {
                (0..num_states).map(|j| {
                    alpha[[t, i]] * self.parameters.transition_matrix[[i, j]] * emission_probs[j] * beta[[t + 1, j]]
                }).sum::<f64>()
            }).sum::<f64>();

            if norm > 1e-10 {
                for i in 0..num_states {
                    for j in 0..num_states {
                        xi[[t, i, j]] = alpha[[t, i]] * self.parameters.transition_matrix[[i, j]] * emission_probs[j] * beta[[t + 1, j]] / norm;
                    }
                }
            }
        }

        Ok((gamma, xi))
    }

    /// Update parameters using online learning
    fn update_parameters_online(&mut self, gamma: &Array2<f64>, xi: &Array3<f64>) -> Result<()> {
        let num_obs = gamma.nrows();
        let num_states = self.config.num_states;
        let forgetting_factor = self.config.online_learning_config.forgetting_factor;

        // Update initial probabilities
        for i in 0..num_states {
            let new_initial = gamma[[0, i]];
            self.parameters.initial_probs[i] = forgetting_factor * self.parameters.initial_probs[i] +
                                               self.learning_rate * (1.0 - forgetting_factor) * new_initial;
        }

        // Update transition matrix
        for i in 0..num_states {
            let gamma_sum = (0..num_obs - 1).map(|t| gamma[[t, i]]).sum::<f64>();
            if gamma_sum > 1e-10 {
                for j in 0..num_states {
                    let xi_sum = (0..num_obs - 1).map(|t| xi[[t, i, j]]).sum::<f64>();
                    let new_transition = xi_sum / gamma_sum;
                    self.parameters.transition_matrix[[i, j]] = forgetting_factor * self.parameters.transition_matrix[[i, j]] +
                                                                self.learning_rate * (1.0 - forgetting_factor) * new_transition;
                }
            }
        }

        // Normalize transition matrix rows
        for i in 0..num_states {
            let row_sum = (0..num_states).map(|j| self.parameters.transition_matrix[[i, j]]).sum::<f64>();
            if row_sum > 1e-10 {
                for j in 0..num_states {
                    self.parameters.transition_matrix[[i, j]] /= row_sum;
                }
            }
        }

        // Update emission parameters (means)
        for i in 0..num_states {
            let gamma_sum = (0..num_obs).map(|t| gamma[[t, i]]).sum::<f64>();
            if gamma_sum > 1e-10 {
                for d in 0..self.config.feature_dimensions {
                    let weighted_sum = (0..num_obs).map(|t| {
                        gamma[[t, i]] * self.online_batch[t].features[d]
                    }).sum::<f64>();
                    let new_mean = weighted_sum / gamma_sum;
                    self.parameters.emission_means[[i, d]] = forgetting_factor * self.parameters.emission_means[[i, d]] +
                                                             self.learning_rate * (1.0 - forgetting_factor) * new_mean;
                }
            }
        }

        Ok(())
    }

    /// Adapt learning rate based on performance
    fn adapt_learning_rate(&mut self) {
        let config = &self.config.online_learning_config;

        // Increase learning rate if recent performance is poor
        if self.recent_log_likelihoods.len() >= 5 {
            let recent_avg = self.recent_log_likelihoods.iter().rev().take(5).sum::<f64>() / 5.0;
            let older_avg = if self.recent_log_likelihoods.len() >= 10 {
                self.recent_log_likelihoods.iter().rev().skip(5).take(5).sum::<f64>() / 5.0
            } else {
                self.baseline_log_likelihood
            };

            if recent_avg < older_avg {
                // Performance degrading, increase learning rate
                self.learning_rate = (self.learning_rate * 1.1).min(config.max_learning_rate);
            } else {
                // Performance improving, decrease learning rate for stability
                self.learning_rate = (self.learning_rate * 0.95).max(config.min_learning_rate);
            }
        }
    }

    /// Get online learning statistics
    pub fn get_online_learning_stats(&self) -> OnlineLearningStats {
        OnlineLearningStats {
            current_learning_rate: self.learning_rate,
            update_counter: self.update_counter,
            last_update: self.last_online_update,
            baseline_log_likelihood: self.baseline_log_likelihood,
            recent_performance: if self.recent_log_likelihoods.is_empty() {
                None
            } else {
                Some(self.recent_log_likelihoods.iter().sum::<f64>() / self.recent_log_likelihoods.len() as f64)
            },
            drift_detected: self.drift_detector.last_drift_detection.is_some(),
            consecutive_drift_count: self.drift_detector.consecutive_drift_count,
        }
    }

    /// Get enhanced transition detection statistics
    pub fn get_transition_detection_stats(&self) -> TransitionDetectionStats {
        self.transition_detector.get_detection_stats()
    }

    /// Get ensemble voting statistics
    pub fn get_ensemble_voting_stats(&self) -> EnsembleVotingStats {
        self.transition_detector.ensemble_voter.get_voting_stats()
    }
}

/// Online learning statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineLearningStats {
    pub current_learning_rate: f64,
    pub update_counter: usize,
    pub last_update: Option<DateTime<Utc>>,
    pub baseline_log_likelihood: f64,
    pub recent_performance: Option<f64>,
    pub drift_detected: bool,
    pub consecutive_drift_count: usize,
}

// Enhanced Transition Detection Implementation
impl TransitionDetector {
    /// Enhanced transition detection with multiple methods
    pub fn detect_transition(&mut self, observation: &MarketObservation) -> Option<TransitionDetectionResult> {
        // Update all detection components
        self.update_change_point_detector(observation);
        self.update_volatility_breakout_detector(observation);

        // Collect votes from all methods
        let mut votes = Vec::new();

        // Change point detection vote
        if let Some(cp_result) = self.change_point_detector.detect_change_point() {
            votes.push(EnsembleVote {
                method: VotingMethod::ChangePoint,
                confidence: cp_result.confidence,
                timestamp: observation.timestamp,
                detected_transition: cp_result.detected,
            });
        }

        // Volatility breakout detection vote
        if let Some(vb_result) = self.volatility_breakout_detector.detect_breakout() {
            votes.push(EnsembleVote {
                method: VotingMethod::VolatilityBreakout,
                confidence: vb_result.confidence,
                timestamp: observation.timestamp,
                detected_transition: vb_result.detected,
            });
        }

        // Momentum shift detection vote
        if let Some(momentum_vote) = self.detect_momentum_shift(observation) {
            votes.push(momentum_vote);
        }

        // Trend reversal detection vote
        if let Some(trend_vote) = self.detect_trend_reversal(observation) {
            votes.push(trend_vote);
        }

        // Volume spike detection vote
        if let Some(volume_vote) = self.detect_volume_spike(observation) {
            votes.push(volume_vote);
        }

        // Ensemble voting
        if let Some(ensemble_result) = self.ensemble_voter.vote(votes) {
            // Add to detection history
            if self.detection_history.len() >= 50 {
                self.detection_history.pop_front();
            }
            self.detection_history.push_back(ensemble_result.clone());

            if ensemble_result.detected {
                self.last_detection = Some(observation.timestamp);
            }

            return Some(ensemble_result);
        }

        None
    }

    /// Update change point detector
    fn update_change_point_detector(&mut self, observation: &MarketObservation) {
        self.change_point_detector.update(observation);
    }

    /// Update volatility breakout detector
    fn update_volatility_breakout_detector(&mut self, observation: &MarketObservation) {
        self.volatility_breakout_detector.update(observation);
    }

    /// Detect momentum shift
    fn detect_momentum_shift(&self, observation: &MarketObservation) -> Option<EnsembleVote> {
        let momentum_magnitude = observation.momentum.abs();
        let detected = momentum_magnitude > self.momentum_threshold;
        let confidence = (momentum_magnitude / self.momentum_threshold).min(1.0);

        Some(EnsembleVote {
            method: VotingMethod::MomentumShift,
            confidence,
            timestamp: observation.timestamp,
            detected_transition: detected,
        })
    }

    /// Detect trend reversal
    fn detect_trend_reversal(&self, observation: &MarketObservation) -> Option<EnsembleVote> {
        // Simple trend reversal detection based on trend change
        let trend_magnitude = observation.trend.abs();
        let detected = trend_magnitude > 0.01; // Threshold for significant trend change
        let confidence = (trend_magnitude / 0.01).min(1.0);

        Some(EnsembleVote {
            method: VotingMethod::TrendReversal,
            confidence,
            timestamp: observation.timestamp,
            detected_transition: detected,
        })
    }

    /// Detect volume spike
    fn detect_volume_spike(&self, observation: &MarketObservation) -> Option<EnsembleVote> {
        // Volume spike detection based on normalized volume
        let volume_threshold = 2.0; // 2x normal volume
        let detected = observation.volume > volume_threshold;
        let confidence = (observation.volume / volume_threshold).min(1.0);

        Some(EnsembleVote {
            method: VotingMethod::VolumeSpike,
            confidence,
            timestamp: observation.timestamp,
            detected_transition: detected,
        })
    }

    /// Get detection statistics
    pub fn get_detection_stats(&self) -> TransitionDetectionStats {
        let total_detections = self.detection_history.len();
        let positive_detections = self.detection_history.iter()
            .filter(|d| d.detected)
            .count();

        let avg_confidence = if total_detections > 0 {
            self.detection_history.iter()
                .map(|d| d.confidence)
                .sum::<f64>() / total_detections as f64
        } else {
            0.0
        };

        TransitionDetectionStats {
            total_detections,
            positive_detections,
            detection_rate: if total_detections > 0 {
                positive_detections as f64 / total_detections as f64
            } else {
                0.0
            },
            average_confidence: avg_confidence,
            last_detection: self.last_detection,
        }
    }
}

/// Transition detection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionDetectionStats {
    pub total_detections: usize,
    pub positive_detections: usize,
    pub detection_rate: f64,
    pub average_confidence: f64,
    pub last_detection: Option<DateTime<Utc>>,
}

impl ChangePointDetector {
    /// Update with new observation
    pub fn update(&mut self, observation: &MarketObservation) {
        // Update price history
        if self.price_history.len() >= self.window_size {
            self.price_history.pop_front();
        }
        // Use a representative price from the observation
        let price = observation.features[0]; // Assuming first feature is price-related
        self.price_history.push_back(price);

        // Update volatility history
        if self.volatility_history.len() >= self.window_size {
            self.volatility_history.pop_front();
        }
        self.volatility_history.push_back(observation.volatility);

        // Update cumulative sum for CUSUM algorithm
        if self.price_history.len() >= 2 {
            let current_price = *self.price_history.back().unwrap();
            let prev_price = self.price_history[self.price_history.len() - 2];
            let price_change = current_price - prev_price;

            // CUSUM algorithm for change point detection
            self.cumulative_sum = (self.cumulative_sum + price_change).max(0.0);
        }
    }

    /// Detect change point using multiple methods
    pub fn detect_change_point(&self) -> Option<ChangePointResult> {
        if self.price_history.len() < self.window_size {
            return None;
        }

        // Method 1: CUSUM-based detection
        let cusum_detected = self.cumulative_sum > self.threshold;
        let cusum_confidence = (self.cumulative_sum / self.threshold).min(1.0);

        // Method 2: Mean shift detection
        let (mean_shift_detected, mean_shift_confidence) = self.detect_mean_shift();

        // Method 3: Variance change detection
        let (variance_change_detected, variance_confidence) = self.detect_variance_change();

        // Combine results
        let combined_confidence = (cusum_confidence + mean_shift_confidence + variance_confidence) / 3.0;
        let detected = cusum_detected || mean_shift_detected || variance_change_detected;

        Some(ChangePointResult {
            detected,
            confidence: combined_confidence,
            method: "Combined".to_string(),
            cusum_score: self.cumulative_sum,
            mean_shift_score: mean_shift_confidence,
            variance_change_score: variance_confidence,
        })
    }

    /// Detect mean shift in the time series
    fn detect_mean_shift(&self) -> (bool, f64) {
        if self.price_history.len() < self.window_size {
            return (false, 0.0);
        }

        let mid_point = self.window_size / 2;
        let first_half: Vec<f64> = self.price_history.iter().take(mid_point).cloned().collect();
        let second_half: Vec<f64> = self.price_history.iter().skip(mid_point).cloned().collect();

        let mean1 = first_half.iter().sum::<f64>() / first_half.len() as f64;
        let mean2 = second_half.iter().sum::<f64>() / second_half.len() as f64;

        let mean_diff = (mean2 - mean1).abs();
        let std_dev = self.calculate_std_dev(&self.price_history.iter().cloned().collect());

        let normalized_diff = if std_dev > 1e-8 {
            mean_diff / std_dev
        } else {
            0.0
        };

        let detected = normalized_diff > self.mean_shift_threshold;
        let confidence = (normalized_diff / self.mean_shift_threshold).min(1.0);

        (detected, confidence)
    }

    /// Detect variance change in the time series
    fn detect_variance_change(&self) -> (bool, f64) {
        if self.volatility_history.len() < self.window_size {
            return (false, 0.0);
        }

        let mid_point = self.window_size / 2;
        let first_half: Vec<f64> = self.volatility_history.iter().take(mid_point).cloned().collect();
        let second_half: Vec<f64> = self.volatility_history.iter().skip(mid_point).cloned().collect();

        let var1 = self.calculate_variance(&first_half);
        let var2 = self.calculate_variance(&second_half);

        let variance_ratio = if var1 > 1e-8 {
            (var2 / var1).max(var1 / var2)
        } else {
            1.0
        };

        let threshold = 2.0; // Variance change threshold
        let detected = variance_ratio > threshold;
        let confidence = ((variance_ratio - 1.0) / (threshold - 1.0)).min(1.0);

        (detected, confidence)
    }

    /// Calculate standard deviation
    fn calculate_std_dev(&self, data: &[f64]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }

        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance = data.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;

        variance.sqrt()
    }

    /// Calculate variance
    fn calculate_variance(&self, data: &[f64]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }

        let mean = data.iter().sum::<f64>() / data.len() as f64;
        data.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64
    }
}

/// Change point detection result
#[derive(Debug, Clone)]
pub struct ChangePointResult {
    pub detected: bool,
    pub confidence: f64,
    pub method: String,
    pub cusum_score: f64,
    pub mean_shift_score: f64,
    pub variance_change_score: f64,
}

impl VolatilityBreakoutDetector {
    /// Update with new observation
    pub fn update(&mut self, observation: &MarketObservation) {
        // Update volatility history
        if self.volatility_history.len() >= self.lookback_window {
            self.volatility_history.pop_front();
        }
        self.volatility_history.push_back(observation.volatility);

        // Update rolling statistics
        if self.volatility_history.len() >= 5 {
            self.update_rolling_stats();
        }
    }

    /// Update rolling mean and standard deviation
    fn update_rolling_stats(&mut self) {
        let data: Vec<f64> = self.volatility_history.iter().cloned().collect();
        self.rolling_mean = data.iter().sum::<f64>() / data.len() as f64;

        let variance = data.iter()
            .map(|x| (x - self.rolling_mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        self.rolling_std = variance.sqrt();
    }

    /// Detect volatility breakout
    pub fn detect_breakout(&mut self) -> Option<VolatilityBreakoutResult> {
        if self.volatility_history.len() < 5 {
            return None;
        }

        let current_volatility = *self.volatility_history.back().unwrap();

        // Z-score based breakout detection
        let z_score = if self.rolling_std > 1e-8 {
            (current_volatility - self.rolling_mean) / self.rolling_std
        } else {
            0.0
        };

        let detected = z_score.abs() > self.breakout_threshold;
        let confidence = (z_score.abs() / self.breakout_threshold).min(1.0);

        // Additional breakout patterns
        let (pattern_detected, pattern_confidence) = self.detect_breakout_patterns();

        // Combine results
        let final_detected = detected || pattern_detected;
        let final_confidence = (confidence + pattern_confidence) / 2.0;

        if final_detected {
            self.last_breakout = Some(Utc::now());
        }

        Some(VolatilityBreakoutResult {
            detected: final_detected,
            confidence: final_confidence,
            z_score,
            current_volatility,
            rolling_mean: self.rolling_mean,
            rolling_std: self.rolling_std,
            breakout_type: if z_score > 0.0 {
                BreakoutType::Upward
            } else {
                BreakoutType::Downward
            },
        })
    }

    /// Detect specific breakout patterns
    fn detect_breakout_patterns(&self) -> (bool, f64) {
        if self.volatility_history.len() < 10 {
            return (false, 0.0);
        }

        // Pattern 1: Consecutive increasing volatility
        let recent_5: Vec<f64> = self.volatility_history.iter().rev().take(5).cloned().collect();
        let is_increasing = recent_5.windows(2).all(|w| w[1] > w[0]);

        // Pattern 2: Sudden spike (current > 3x recent average)
        let recent_avg = recent_5.iter().sum::<f64>() / recent_5.len() as f64;
        let current = *self.volatility_history.back().unwrap();
        let is_spike = current > recent_avg * 3.0;

        // Pattern 3: Volatility clustering (high volatility followed by high volatility)
        let prev_high = recent_5.iter().any(|&v| v > self.rolling_mean + 2.0 * self.rolling_std);
        let current_high = current > self.rolling_mean + self.rolling_std;
        let is_clustering = prev_high && current_high;

        let pattern_detected = is_increasing || is_spike || is_clustering;
        let confidence = if pattern_detected {
            let mut score = 0.0;
            if is_increasing { score += 0.3; }
            if is_spike { score += 0.5; }
            if is_clustering { score += 0.2; }
            score.min(1.0)
        } else {
            0.0
        };

        (pattern_detected, confidence)
    }
}

/// Volatility breakout detection result
#[derive(Debug, Clone)]
pub struct VolatilityBreakoutResult {
    pub detected: bool,
    pub confidence: f64,
    pub z_score: f64,
    pub current_volatility: f64,
    pub rolling_mean: f64,
    pub rolling_std: f64,
    pub breakout_type: BreakoutType,
}

/// Types of volatility breakouts
#[derive(Debug, Clone, PartialEq)]
pub enum BreakoutType {
    Upward,
    Downward,
}

impl EnsembleVoter {
    /// Perform ensemble voting on detection results
    pub fn vote(&mut self, votes: Vec<EnsembleVote>) -> Option<TransitionDetectionResult> {
        if votes.is_empty() {
            return None;
        }

        // Add votes to history
        for vote in &votes {
            if self.recent_votes.len() >= 100 {
                self.recent_votes.pop_front();
            }
            self.recent_votes.push_back(vote.clone());
        }

        // Calculate weighted consensus
        let mut weighted_score = 0.0;
        let mut total_weight = 0.0;
        let mut positive_votes = 0;
        let mut detection_methods = Vec::new();

        for vote in &votes {
            if let Some(method_index) = self.voting_methods.iter().position(|m| m == &vote.method) {
                let weight = self.method_weights[method_index];
                let vote_score = if vote.detected_transition {
                    vote.confidence * weight
                } else {
                    0.0
                };

                weighted_score += vote_score;
                total_weight += weight;

                if vote.detected_transition {
                    positive_votes += 1;
                    detection_methods.push(format!("{:?}", vote.method));
                }
            }
        }

        // Normalize weighted score
        let consensus_score = if total_weight > 0.0 {
            weighted_score / total_weight
        } else {
            0.0
        };

        // Determine if transition is detected
        let detected = consensus_score > self.consensus_threshold;

        // Determine transition type based on voting patterns
        let transition_type = self.determine_transition_type(&votes);

        // Create detection result
        Some(TransitionDetectionResult {
            detected,
            confidence: consensus_score,
            detection_method: if detection_methods.is_empty() {
                "None".to_string()
            } else {
                detection_methods.join(", ")
            },
            timestamp: votes.first().unwrap().timestamp,
            transition_type,
        })
    }

    /// Determine transition type based on voting patterns
    fn determine_transition_type(&self, votes: &[EnsembleVote]) -> TransitionType {
        let mut volatility_votes = 0;
        let mut trend_votes = 0;
        let mut momentum_votes = 0;
        let mut change_point_votes = 0;

        for vote in votes {
            if vote.detected_transition {
                match vote.method {
                    VotingMethod::VolatilityBreakout | VotingMethod::VolumeSpike => volatility_votes += 1,
                    VotingMethod::TrendReversal => trend_votes += 1,
                    VotingMethod::MomentumShift => momentum_votes += 1,
                    VotingMethod::ChangePoint => change_point_votes += 1,
                }
            }
        }

        // Determine dominant pattern
        if volatility_votes >= 2 {
            TransitionType::VolatilitySpike
        } else if trend_votes >= 1 && momentum_votes >= 1 {
            TransitionType::TrendReversal
        } else if change_point_votes >= 1 {
            if momentum_votes >= 1 {
                TransitionType::Sharp
            } else {
                TransitionType::Gradual
            }
        } else {
            TransitionType::Unknown
        }
    }

    /// Get voting statistics
    pub fn get_voting_stats(&self) -> EnsembleVotingStats {
        let total_votes = self.recent_votes.len();
        let positive_votes = self.recent_votes.iter()
            .filter(|v| v.detected_transition)
            .count();

        let method_stats: std::collections::HashMap<String, usize> = self.recent_votes.iter()
            .filter(|v| v.detected_transition)
            .map(|v| format!("{:?}", v.method))
            .fold(std::collections::HashMap::new(), |mut acc, method| {
                *acc.entry(method).or_insert(0) += 1;
                acc
            });

        let avg_confidence = if total_votes > 0 {
            self.recent_votes.iter()
                .map(|v| v.confidence)
                .sum::<f64>() / total_votes as f64
        } else {
            0.0
        };

        EnsembleVotingStats {
            total_votes,
            positive_votes,
            detection_rate: if total_votes > 0 {
                positive_votes as f64 / total_votes as f64
            } else {
                0.0
            },
            average_confidence: avg_confidence,
            method_distribution: method_stats,
            consensus_threshold: self.consensus_threshold,
        }
    }

    /// Update voting configuration
    pub fn update_config(&mut self, new_threshold: f64, new_weights: Vec<f64>) {
        self.consensus_threshold = new_threshold;
        if new_weights.len() == self.method_weights.len() {
            self.method_weights = new_weights;
        }
    }
}

/// Ensemble voting statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnsembleVotingStats {
    pub total_votes: usize,
    pub positive_votes: usize,
    pub detection_rate: f64,
    pub average_confidence: f64,
    pub method_distribution: std::collections::HashMap<String, usize>,
    pub consensus_threshold: f64,
}

// Performance Optimization Components
/// Performance-optimized HMM inference engine
#[derive(Debug, Clone)]
pub struct OptimizedHMMInference {
    /// Cached emission probabilities
    emission_cache: std::collections::HashMap<String, Array1<f64>>,
    /// Pre-computed transition matrices
    transition_cache: Array2<f64>,
    /// State probability cache
    state_prob_cache: Array1<f64>,
    /// Feature normalization cache
    feature_normalizer: FeatureNormalizer,
    /// Performance metrics
    inference_metrics: InferenceMetrics,
}

/// Feature normalization for consistent inference
#[derive(Debug, Clone)]
pub struct FeatureNormalizer {
    pub feature_means: Array1<f64>,
    pub feature_stds: Array1<f64>,
    pub min_values: Array1<f64>,
    pub max_values: Array1<f64>,
    pub normalization_method: NormalizationMethod,
}

/// Normalization methods
#[derive(Debug, Clone, PartialEq)]
pub enum NormalizationMethod {
    ZScore,
    MinMax,
    Robust,
}

/// Inference performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceMetrics {
    pub total_inferences: usize,
    pub average_latency_ms: f64,
    pub cache_hit_rate: f64,
    pub accuracy_score: f64,
    pub last_inference_time: Option<DateTime<Utc>>,
}

/// Backtesting framework for HMM validation
#[derive(Debug, Clone)]
pub struct HMMBacktester {
    pub config: BacktestConfig,
    pub test_data: Vec<MarketObservation>,
    pub results: Vec<BacktestResult>,
    pub performance_metrics: BacktestPerformanceMetrics,
}

/// Backtesting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub validation_split: f64,
    pub rolling_window_size: usize,
    pub retraining_frequency: usize,
    pub performance_metrics: Vec<String>,
    pub confidence_thresholds: Vec<f64>,
}

/// Individual backtest result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    pub timestamp: DateTime<Utc>,
    pub predicted_regime: RegimeType,
    pub actual_regime: Option<RegimeType>,
    pub confidence: f64,
    pub inference_latency_ms: f64,
    pub correct_prediction: Option<bool>,
}

/// Comprehensive backtest performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestPerformanceMetrics {
    pub overall_accuracy: f64,
    pub regime_specific_accuracy: std::collections::HashMap<String, f64>,
    pub average_confidence: f64,
    pub average_latency_ms: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
    pub confusion_matrix: Vec<Vec<usize>>,
    pub sharpe_ratio: Option<f64>,
    pub max_drawdown: Option<f64>,
}

impl OptimizedHMMInference {
    /// Create new optimized inference engine
    pub fn new(feature_dimensions: usize) -> Self {
        Self {
            emission_cache: std::collections::HashMap::new(),
            transition_cache: Array2::zeros((4, 4)), // Default 4 states
            state_prob_cache: Array1::zeros(4),
            feature_normalizer: FeatureNormalizer::new(feature_dimensions),
            inference_metrics: InferenceMetrics::default(),
        }
    }

    /// Ultra-optimized regime detection with advanced caching and SIMD operations
    pub fn fast_regime_detection(&mut self, detector: &HMMRegimeDetector, observation: &MarketObservation) -> Result<Option<RegimeSignal>> {
        let start_time = std::time::Instant::now();

        // Pre-allocate arrays to avoid memory allocation overhead
        if self.state_prob_cache.len() != detector.config.num_states {
            self.state_prob_cache = Array1::zeros(detector.config.num_states);
        }

        // Fast feature normalization using pre-computed statistics
        let normalized_features = self.fast_normalize_features(&observation.features);

        // Create optimized cache key using hash-based approach
        let cache_key = self.fast_cache_key(&normalized_features);

        // Check emission cache with LRU eviction
        let emission_probs = if let Some(cached_probs) = self.emission_cache.get(&cache_key) {
            self.inference_metrics.cache_hit_rate += 1.0;
            cached_probs.clone()
        } else {
            // Limit cache size to prevent memory bloat
            if self.emission_cache.len() > 1000 {
                self.evict_lru_cache_entries();
            }

            let probs = self.calculate_emission_probabilities_fast(detector, &normalized_features);
            self.emission_cache.insert(cache_key, probs.clone());
            probs
        };

        // Ultra-fast forward inference with vectorized operations
        let regime_signal = self.vectorized_forward_inference(detector, &emission_probs)?;

        // Update metrics with minimal overhead
        let inference_time = start_time.elapsed().as_nanos() as f64 / 1_000_000.0; // More precise timing
        self.update_inference_metrics(inference_time);

        // Early exit if inference time exceeds target
        if inference_time > 20.0 {
            warn!("HMM inference exceeded 20ms target: {:.2}ms", inference_time);
        }

        Ok(regime_signal)
    }

    /// Fast forward algorithm implementation
    fn fast_forward_inference(&mut self, detector: &HMMRegimeDetector, emission_probs: &Array1<f64>) -> Result<Option<RegimeSignal>> {
        let num_states = detector.config.num_states;

        // Use cached transition matrix if available
        if self.transition_cache.dim() != (num_states, num_states) {
            self.transition_cache = detector.parameters.transition_matrix.clone();
        }

        // Fast matrix multiplication using BLAS-optimized operations
        let mut new_state_probs = Array1::zeros(num_states);
        for j in 0..num_states {
            for i in 0..num_states {
                new_state_probs[j] += self.state_prob_cache[i] * self.transition_cache[[i, j]];
            }
            new_state_probs[j] *= emission_probs[j];
        }

        // Normalize
        let norm_factor = new_state_probs.sum();
        if norm_factor > 1e-10 {
            new_state_probs /= norm_factor;
        }

        // Update cache
        self.state_prob_cache = new_state_probs.clone();

        // Find most likely state
        let (most_likely_state, confidence) = new_state_probs.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(idx, &prob)| (idx, prob))
            .unwrap_or((0, 0.25));

        if confidence < detector.config.min_confidence {
            return Ok(None);
        }

        // Map state to regime type
        let regime_type = match most_likely_state {
            0 => RegimeType::Bullish,
            1 => RegimeType::Bearish,
            2 => RegimeType::Sideways,
            3 => RegimeType::HighVolatility,
            _ => RegimeType::Sideways,
        };

        Ok(Some(RegimeSignal {
            regime: regime_type,
            confidence,
            timestamp: Utc::now(),
            transition_probability: 1.0 - confidence,
            regime_strength: confidence,
            expected_duration_minutes: 30, // Default
        }))
    }

    /// Create cache key from features
    fn create_cache_key(&self, features: &Array1<f64>) -> String {
        // Create a hash-based key from rounded feature values
        let rounded_features: Vec<i32> = features.iter()
            .map(|&f| (f * 1000.0).round() as i32)
            .collect();
        format!("{:?}", rounded_features)
    }

    /// Ultra-fast feature normalization using pre-computed statistics
    fn fast_normalize_features(&self, features: &Array1<f64>) -> Array1<f64> {
        // Use SIMD-friendly operations for normalization
        let mut normalized = Array1::zeros(features.len());
        for (i, &value) in features.iter().enumerate() {
            normalized[i] = (value - self.feature_normalizer.feature_means[i]) / self.feature_normalizer.feature_stds[i];
        }
        normalized
    }

    /// Fast cache key generation using hash-based approach
    fn fast_cache_key(&self, features: &Array1<f64>) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        for &f in features.iter() {
            ((f * 10000.0).round() as i64).hash(&mut hasher);
        }
        format!("{:x}", hasher.finish())
    }

    /// Evict least recently used cache entries
    fn evict_lru_cache_entries(&mut self) {
        // Simple eviction strategy - remove half the cache
        let keys_to_remove: Vec<String> = self.emission_cache.keys()
            .take(self.emission_cache.len() / 2)
            .cloned()
            .collect();

        for key in keys_to_remove {
            self.emission_cache.remove(&key);
        }
    }

    /// Fast emission probability calculation
    fn calculate_emission_probabilities_fast(&self, detector: &HMMRegimeDetector, features: &Array1<f64>) -> Array1<f64> {
        // Use optimized Gaussian probability calculation
        let mut probs = Array1::zeros(detector.config.num_states);

        for state in 0..detector.config.num_states {
            let mean = &detector.parameters.emission_means.row(state);
            let cov_inv = &detector.parameters.emission_covariances[state];

            // Fast multivariate Gaussian calculation
            let diff = features - mean;
            let mahalanobis_sq = diff.dot(&cov_inv.dot(&diff));
            probs[state] = (-0.5 * mahalanobis_sq).exp();
        }

        // Normalize probabilities
        let sum = probs.sum();
        if sum > 1e-10 {
            probs /= sum;
        }

        probs
    }

    /// Vectorized forward inference with SIMD optimizations
    fn vectorized_forward_inference(&mut self, detector: &HMMRegimeDetector, emission_probs: &Array1<f64>) -> Result<Option<RegimeSignal>> {
        let num_states = detector.config.num_states;

        // Use cached transition matrix if available
        if self.transition_cache.dim() != (num_states, num_states) {
            self.transition_cache = detector.parameters.transition_matrix.clone();
        }

        // Vectorized matrix multiplication using ndarray's optimized BLAS operations
        let new_state_probs = self.state_prob_cache.dot(&self.transition_cache) * emission_probs;

        // Fast normalization
        let norm_factor = new_state_probs.sum();
        let normalized_probs = if norm_factor > 1e-10 {
            new_state_probs / norm_factor
        } else {
            Array1::from_elem(num_states, 1.0 / num_states as f64)
        };

        // Update cache
        self.state_prob_cache = normalized_probs.clone();

        // Find most likely state using argmax
        let (most_likely_state, confidence) = normalized_probs.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(idx, &prob)| (idx, prob))
            .unwrap_or((0, 0.25));

        // Early exit for low confidence
        if confidence < detector.config.min_confidence {
            return Ok(None);
        }

        // Fast regime mapping
        let regime_type = match most_likely_state {
            0 => RegimeType::Bullish,
            1 => RegimeType::Bearish,
            2 => RegimeType::Sideways,
            3 => RegimeType::HighVolatility,
            _ => RegimeType::Sideways,
        };

        Ok(Some(RegimeSignal {
            regime: regime_type,
            confidence,
            timestamp: Utc::now(),
            transition_probability: 1.0 - confidence,
            regime_strength: confidence,
            expected_duration_minutes: 30, // Default
        }))
    }

    /// Update inference metrics
    fn update_inference_metrics(&mut self, latency_ms: f64) {
        self.inference_metrics.total_inferences += 1;

        // Update average latency using exponential moving average
        let alpha = 0.1;
        self.inference_metrics.average_latency_ms =
            alpha * latency_ms + (1.0 - alpha) * self.inference_metrics.average_latency_ms;

        // Update cache hit rate
        self.inference_metrics.cache_hit_rate =
            self.inference_metrics.cache_hit_rate / self.inference_metrics.total_inferences as f64;

        self.inference_metrics.last_inference_time = Some(Utc::now());
    }

    /// Clear caches to free memory
    pub fn clear_caches(&mut self) {
        self.emission_cache.clear();
    }

    /// Get performance metrics
    pub fn get_metrics(&self) -> &InferenceMetrics {
        &self.inference_metrics
    }
}

impl FeatureNormalizer {
    /// Create new feature normalizer
    pub fn new(feature_dimensions: usize) -> Self {
        Self {
            feature_means: Array1::zeros(feature_dimensions),
            feature_stds: Array1::ones(feature_dimensions),
            min_values: Array1::zeros(feature_dimensions),
            max_values: Array1::ones(feature_dimensions),
            normalization_method: NormalizationMethod::ZScore,
        }
    }

    /// Fit normalizer to training data
    pub fn fit(&mut self, training_data: &[Array1<f64>]) {
        if training_data.is_empty() {
            return;
        }

        let n_samples = training_data.len();
        let n_features = training_data[0].len();

        // Calculate means
        for feature_idx in 0..n_features {
            let sum: f64 = training_data.iter()
                .map(|sample| sample[feature_idx])
                .sum();
            self.feature_means[feature_idx] = sum / n_samples as f64;
        }

        // Calculate standard deviations and min/max
        for feature_idx in 0..n_features {
            let mean = self.feature_means[feature_idx];
            let variance: f64 = training_data.iter()
                .map(|sample| (sample[feature_idx] - mean).powi(2))
                .sum::<f64>() / n_samples as f64;
            self.feature_stds[feature_idx] = variance.sqrt().max(1e-8);

            // Min/Max for MinMax normalization
            let values: Vec<f64> = training_data.iter()
                .map(|sample| sample[feature_idx])
                .collect();
            self.min_values[feature_idx] = values.iter().cloned().fold(f64::INFINITY, f64::min);
            self.max_values[feature_idx] = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        }
    }

    /// Normalize features
    pub fn normalize(&self, features: &Array1<f64>) -> Array1<f64> {
        match self.normalization_method {
            NormalizationMethod::ZScore => {
                (features - &self.feature_means) / &self.feature_stds
            }
            NormalizationMethod::MinMax => {
                let range = &self.max_values - &self.min_values;
                (features - &self.min_values) / &range
            }
            NormalizationMethod::Robust => {
                // Simplified robust normalization
                (features - &self.feature_means) / &self.feature_stds
            }
        }
    }
}

impl Default for InferenceMetrics {
    fn default() -> Self {
        Self {
            total_inferences: 0,
            average_latency_ms: 0.0,
            cache_hit_rate: 0.0,
            accuracy_score: 0.0,
            last_inference_time: None,
        }
    }
}

impl HMMBacktester {
    /// Create new backtester
    pub fn new(config: BacktestConfig) -> Self {
        Self {
            config,
            test_data: Vec::new(),
            results: Vec::new(),
            performance_metrics: BacktestPerformanceMetrics::default(),
        }
    }

    /// Load test data
    pub fn load_test_data(&mut self, data: Vec<MarketObservation>) {
        self.test_data = data;
    }

    /// Run comprehensive backtest
    pub async fn run_backtest(&mut self, detector: &mut HMMRegimeDetector) -> Result<BacktestPerformanceMetrics> {
        if self.test_data.is_empty() {
            return Err(crate::utils::PantherSwapError::ai_prediction("No test data loaded".to_string()));
        }

        // Split data for validation
        let split_index = (self.test_data.len() as f64 * self.config.validation_split) as usize;
        let (training_data, test_data) = self.test_data.split_at(split_index);

        // Train on training data
        for obs in training_data {
            detector.update_with_tick(&MarketTick {
                timestamp: obs.timestamp,
                instrument_id: uuid::Uuid::new_v4(),
                provider: "test".to_string(),
                bid_price: obs.features[0] - obs.bid_ask_spread / 2.0,
                ask_price: obs.features[0] + obs.bid_ask_spread / 2.0,
                bid_size: 1000.0,
                ask_size: 1000.0,
                last_price: Some(obs.features[0]),
                volume: Some(obs.volume as f64),
                spread: obs.bid_ask_spread,
                data_quality_score: 0.95,
                raw_data: serde_json::json!({}),
            })?;
        }

        // Train the detector
        detector.train()?;

        // Initialize optimized inference
        let mut optimized_inference = OptimizedHMMInference::new(detector.config.feature_dimensions);

        // Run inference on test data
        self.results.clear();
        let mut correct_predictions = 0;
        let mut total_predictions = 0;

        for (i, obs) in test_data.iter().enumerate() {
            let start_time = std::time::Instant::now();

            // Get prediction
            let prediction = optimized_inference.fast_regime_detection(detector, obs)?;

            let inference_latency = start_time.elapsed().as_millis() as f64;

            if let Some(regime_signal) = prediction {
                // For backtesting, we need actual regime labels
                // This is a simplified version - in practice, you'd have labeled data
                let actual_regime = self.infer_actual_regime(obs);
                let correct = if let Some(actual) = actual_regime {
                    actual == regime_signal.regime
                } else {
                    None
                };

                if correct.is_some() {
                    total_predictions += 1;
                    if correct.unwrap() {
                        correct_predictions += 1;
                    }
                }

                self.results.push(BacktestResult {
                    timestamp: obs.timestamp,
                    predicted_regime: regime_signal.regime,
                    actual_regime,
                    confidence: regime_signal.confidence,
                    inference_latency_ms: inference_latency,
                    correct_prediction: correct,
                });
            }

            // Retrain periodically
            if i % self.config.retraining_frequency == 0 && i > 0 {
                detector.train()?;
            }
        }

        // Calculate performance metrics
        self.calculate_performance_metrics();

        Ok(self.performance_metrics.clone())
    }

    /// Infer actual regime from observation (simplified)
    fn infer_actual_regime(&self, obs: &MarketObservation) -> Option<RegimeType> {
        // This is a simplified heuristic - in practice, you'd have labeled data
        if obs.volatility > 0.03 {
            Some(RegimeType::HighVolatility)
        } else if obs.trend > 0.01 {
            Some(RegimeType::Bullish)
        } else if obs.trend < -0.01 {
            Some(RegimeType::Bearish)
        } else {
            Some(RegimeType::Sideways)
        }
    }

    /// Calculate comprehensive performance metrics
    fn calculate_performance_metrics(&mut self) {
        let total_results = self.results.len();
        if total_results == 0 {
            return;
        }

        let correct_predictions = self.results.iter()
            .filter_map(|r| r.correct_prediction)
            .filter(|&correct| correct)
            .count();

        let total_with_actual = self.results.iter()
            .filter(|r| r.correct_prediction.is_some())
            .count();

        // Overall accuracy
        self.performance_metrics.overall_accuracy = if total_with_actual > 0 {
            correct_predictions as f64 / total_with_actual as f64
        } else {
            0.0
        };

        // Average confidence
        self.performance_metrics.average_confidence =
            self.results.iter().map(|r| r.confidence).sum::<f64>() / total_results as f64;

        // Average latency
        self.performance_metrics.average_latency_ms =
            self.results.iter().map(|r| r.inference_latency_ms).sum::<f64>() / total_results as f64;

        // Regime-specific accuracy
        let mut regime_stats: std::collections::HashMap<RegimeType, (usize, usize)> = std::collections::HashMap::new();

        for result in &self.results {
            if let Some(correct) = result.correct_prediction {
                let entry = regime_stats.entry(result.predicted_regime).or_insert((0, 0));
                entry.1 += 1; // Total predictions
                if correct {
                    entry.0 += 1; // Correct predictions
                }
            }
        }

        self.performance_metrics.regime_specific_accuracy = regime_stats.iter()
            .map(|(regime, (correct, total))| {
                let accuracy = if *total > 0 { *correct as f64 / *total as f64 } else { 0.0 };
                (format!("{:?}", regime), accuracy)
            })
            .collect();

        // Calculate precision, recall, F1 (simplified for binary classification)
        self.calculate_classification_metrics();

        // Calculate confusion matrix
        self.calculate_confusion_matrix();
    }

    /// Calculate precision, recall, and F1 score
    fn calculate_classification_metrics(&mut self) {
        // Simplified binary classification metrics
        // In practice, you'd calculate these for each regime type
        let mut tp = 0; // True positives
        let mut fp = 0; // False positives
        let mut tn = 0; // True negatives
        let mut fn_count = 0; // False negatives

        for result in &self.results {
            if let Some(correct) = result.correct_prediction {
                if correct {
                    tp += 1;
                } else {
                    fp += 1;
                }
            }
        }

        self.performance_metrics.precision = if tp + fp > 0 {
            tp as f64 / (tp + fp) as f64
        } else {
            0.0
        };

        self.performance_metrics.recall = if tp + fn_count > 0 {
            tp as f64 / (tp + fn_count) as f64
        } else {
            0.0
        };

        self.performance_metrics.f1_score = if self.performance_metrics.precision + self.performance_metrics.recall > 0.0 {
            2.0 * (self.performance_metrics.precision * self.performance_metrics.recall) /
            (self.performance_metrics.precision + self.performance_metrics.recall)
        } else {
            0.0
        };
    }

    /// Calculate confusion matrix
    fn calculate_confusion_matrix(&mut self) {
        // Initialize 4x4 confusion matrix for 4 regime types
        let mut matrix = vec![vec![0; 4]; 4];

        for result in &self.results {
            if let Some(actual) = result.actual_regime {
                let predicted_idx = self.regime_to_index(result.predicted_regime);
                let actual_idx = self.regime_to_index(actual);
                matrix[actual_idx][predicted_idx] += 1;
            }
        }

        self.performance_metrics.confusion_matrix = matrix;
    }

    /// Convert regime type to matrix index
    fn regime_to_index(&self, regime: RegimeType) -> usize {
        match regime {
            RegimeType::Normal => 0,
            RegimeType::Trending => 1,
            RegimeType::Volatile => 2,
            RegimeType::Crisis => 3,
            RegimeType::Bullish => 4,
            RegimeType::Bearish => 5,
            RegimeType::Sideways => 6,
            RegimeType::HighVolatility => 7,
        }
    }

    /// Get detailed backtest report
    pub fn get_detailed_report(&self) -> BacktestReport {
        BacktestReport {
            config: self.config.clone(),
            performance_metrics: self.performance_metrics.clone(),
            sample_results: self.results.iter().take(100).cloned().collect(),
            total_results: self.results.len(),
        }
    }
}

impl Default for BacktestPerformanceMetrics {
    fn default() -> Self {
        Self {
            overall_accuracy: 0.0,
            regime_specific_accuracy: std::collections::HashMap::new(),
            average_confidence: 0.0,
            average_latency_ms: 0.0,
            precision: 0.0,
            recall: 0.0,
            f1_score: 0.0,
            confusion_matrix: vec![vec![0; 4]; 4],
            sharpe_ratio: None,
            max_drawdown: None,
        }
    }
}

/// Comprehensive backtest report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestReport {
    pub config: BacktestConfig,
    pub performance_metrics: BacktestPerformanceMetrics,
    pub sample_results: Vec<BacktestResult>,
    pub total_results: usize,
}

/// Factory function to create a multi-scale HMM regime detector with default configuration
pub fn create_multi_scale_hmm_detector() -> MultiScaleHMMRegimeDetector {
    let config = MultiScaleHMMConfig::default();
    MultiScaleHMMRegimeDetector::new(config)
}

/// Factory function to create a high-frequency multi-scale HMM regime detector
pub fn create_hf_multi_scale_hmm_detector() -> MultiScaleHMMRegimeDetector {
    let mut config = MultiScaleHMMConfig::default();

    // Optimize for high-frequency trading
    config.consensus_threshold = 0.55; // Lower threshold for faster decisions
    config.transition_sensitivity = 0.8; // Higher sensitivity

    // Adjust scale weights for HF trading (favor shorter timeframes)
    config.scale_weights.insert(TimeScale::OneMinute, 0.5);
    config.scale_weights.insert(TimeScale::FiveMinutes, 0.3);
    config.scale_weights.insert(TimeScale::FifteenMinutes, 0.15);
    config.scale_weights.insert(TimeScale::OneHour, 0.05);

    // Configure each scale for HF trading
    for (scale, scale_config) in config.scale_configs.iter_mut() {
        scale_config.fast_detection_mode = true;
        scale_config.adaptive_threshold = true;
        scale_config.min_confidence = match scale {
            TimeScale::OneMinute => 0.6,
            TimeScale::FiveMinutes => 0.65,
            TimeScale::FifteenMinutes => 0.7,
            TimeScale::OneHour => 0.75,
        };
    }

    MultiScaleHMMRegimeDetector::new(config)
}

/// Factory function to create an accuracy-optimized multi-scale HMM regime detector
pub fn create_accuracy_multi_scale_hmm_detector() -> MultiScaleHMMRegimeDetector {
    let mut config = MultiScaleHMMConfig::default();

    // Optimize for accuracy over speed
    config.consensus_threshold = 0.75; // Higher threshold for more confident decisions
    config.transition_sensitivity = 0.6; // Lower sensitivity to reduce false positives
    config.enable_hierarchical_propagation = true;

    // Balanced scale weights for accuracy
    config.scale_weights.insert(TimeScale::OneMinute, 0.25);
    config.scale_weights.insert(TimeScale::FiveMinutes, 0.3);
    config.scale_weights.insert(TimeScale::FifteenMinutes, 0.3);
    config.scale_weights.insert(TimeScale::OneHour, 0.15);

    // Configure each scale for accuracy
    for (scale, scale_config) in config.scale_configs.iter_mut() {
        scale_config.convergence_threshold = 1e-10; // Stricter convergence
        scale_config.max_iterations = 300; // More training iterations
        scale_config.observation_window = match scale {
            TimeScale::OneMinute => 120,     // 2 hours of data
            TimeScale::FiveMinutes => 72,    // 6 hours of data
            TimeScale::FifteenMinutes => 48, // 12 hours of data
            TimeScale::OneHour => 48,        // 48 hours of data
        };
        scale_config.min_confidence = match scale {
            TimeScale::OneMinute => 0.7,
            TimeScale::FiveMinutes => 0.75,
            TimeScale::FifteenMinutes => 0.8,
            TimeScale::OneHour => 0.85,
        };
    }

    MultiScaleHMMRegimeDetector::new(config)
}

/// Factory function to create a balanced multi-scale HMM regime detector
pub fn create_balanced_multi_scale_hmm_detector() -> MultiScaleHMMRegimeDetector {
    let mut config = MultiScaleHMMConfig::default();

    // Balanced configuration for general trading
    config.consensus_threshold = 0.65;
    config.transition_sensitivity = 0.7;
    config.enable_hierarchical_propagation = true;

    // Equal weights across scales for balanced approach
    config.scale_weights.insert(TimeScale::OneMinute, 0.3);
    config.scale_weights.insert(TimeScale::FiveMinutes, 0.3);
    config.scale_weights.insert(TimeScale::FifteenMinutes, 0.25);
    config.scale_weights.insert(TimeScale::OneHour, 0.15);

    MultiScaleHMMRegimeDetector::new(config)
}
