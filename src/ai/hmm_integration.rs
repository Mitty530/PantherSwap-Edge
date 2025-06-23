// Enhanced HMM Integration Module for Production Trading Engine
// Seamless integration of optimized HMM regime detection with trading systems

use crate::ai::hmm_regime::{
    HMMRegimeDetector, OptimizedHMMInference, create_enhanced_accuracy_hmm_detector,
    create_hf_hmm_regime_detector, MultiScaleHMMRegimeDetector, create_multi_scale_hmm_detector
};
use crate::ai::hmm_performance_validator::{HMMPerformanceValidator, PerformanceValidationConfig};
use crate::trading::signals::RegimeSignal;
use crate::database::types::MarketTick;
use crate::market_data::MarketDataManager;
use crate::utils::{Result, PantherSwapError};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;
use tracing::{info, error};
use std::time::Instant;

/// HMM Integration Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HMMIntegrationConfig {
    pub enable_enhanced_hmm: bool,
    pub enable_multi_scale: bool,
    pub enable_performance_monitoring: bool,
    pub enable_a_b_testing: bool,
    pub inference_timeout_ms: u64,
    pub max_concurrent_inferences: usize,
    pub cache_size: usize,
    pub validation_interval_minutes: u64,
    pub fallback_to_baseline: bool,
    pub performance_threshold: f64,
}

impl Default for HMMIntegrationConfig {
    fn default() -> Self {
        Self {
            enable_enhanced_hmm: true,
            enable_multi_scale: true,
            enable_performance_monitoring: true,
            enable_a_b_testing: true,
            inference_timeout_ms: 20,
            max_concurrent_inferences: 100,
            cache_size: 1000,
            validation_interval_minutes: 60,
            fallback_to_baseline: true,
            performance_threshold: 0.75,
        }
    }
}

/// HMM Integration Manager for Production Trading
#[derive(Clone)]
pub struct HMMIntegrationManager {
    config: HMMIntegrationConfig,
    
    // Enhanced HMM Components
    enhanced_detector: Arc<RwLock<HMMRegimeDetector>>,
    multi_scale_detector: Arc<RwLock<MultiScaleHMMRegimeDetector>>,
    optimized_inference: Arc<RwLock<OptimizedHMMInference>>,
    
    // Baseline for A/B Testing
    baseline_detector: Arc<RwLock<HMMRegimeDetector>>,
    
    // Performance Monitoring
    performance_validator: Arc<RwLock<HMMPerformanceValidator>>,
    performance_metrics: Arc<RwLock<IntegrationPerformanceMetrics>>,
    
    // A/B Testing Framework
    ab_test_manager: Arc<RwLock<ABTestManager>>,
    
    // Event System
    event_sender: mpsc::UnboundedSender<HMMIntegrationEvent>,
    
    // Market Data Integration
    market_data_manager: MarketDataManager,
    database: crate::database::Database,
}

/// Integration Performance Metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationPerformanceMetrics {
    pub total_inferences: u64,
    pub successful_inferences: u64,
    pub average_latency_ms: f64,
    pub accuracy_score: f64,
    pub cache_hit_rate: f64,
    pub error_rate: f64,
    pub last_validation: Option<DateTime<Utc>>,
    pub performance_trend: Vec<PerformanceDataPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceDataPoint {
    pub timestamp: DateTime<Utc>,
    pub latency_ms: f64,
    pub accuracy: f64,
    pub throughput: f64,
}

/// A/B Testing Manager
#[derive(Debug, Clone)]
pub struct ABTestManager {
    pub test_config: ABTestConfig,
    pub enhanced_metrics: ABTestMetrics,
    pub baseline_metrics: ABTestMetrics,
    pub current_test_id: Option<Uuid>,
    pub test_start_time: Option<DateTime<Utc>>,
    pub traffic_split: f64, // 0.0 = all baseline, 1.0 = all enhanced
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestConfig {
    pub test_duration_hours: u64,
    pub traffic_split_percentage: f64,
    pub significance_threshold: f64,
    pub min_sample_size: usize,
    pub enable_automatic_winner_selection: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestMetrics {
    pub total_predictions: u64,
    pub correct_predictions: u64,
    pub average_latency_ms: f64,
    pub error_count: u64,
    pub confidence_scores: Vec<f64>,
    pub regime_detection_accuracy: f64,
}

/// HMM Integration Events
#[derive(Debug, Clone)]
pub enum HMMIntegrationEvent {
    InferenceCompleted {
        instrument_id: Uuid,
        latency_ms: f64,
        accuracy: Option<f64>,
        model_type: String,
    },
    PerformanceAlert {
        alert_type: String,
        message: String,
        severity: AlertSeverity,
    },
    ABTestUpdate {
        test_id: Uuid,
        enhanced_performance: f64,
        baseline_performance: f64,
    },
    ValidationCompleted {
        validation_results: ValidationSummary,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    pub overall_score: f64,
    pub latency_score: f64,
    pub accuracy_score: f64,
    pub meets_requirements: bool,
    pub recommendations: Vec<String>,
}

impl HMMIntegrationManager {
    /// Create new HMM Integration Manager
    pub async fn new(
        config: HMMIntegrationConfig,
        market_data_manager: MarketDataManager,
        database: crate::database::Database,
    ) -> Result<Self> {
        info!("🚀 Initializing HMM Integration Manager for production deployment");

        // Initialize enhanced HMM components
        let enhanced_detector = Arc::new(RwLock::new(create_enhanced_accuracy_hmm_detector()));
        let multi_scale_detector = Arc::new(RwLock::new(create_multi_scale_hmm_detector()));
        let optimized_inference = Arc::new(RwLock::new(OptimizedHMMInference::new(16))); // 16 features

        // Initialize baseline detector for A/B testing
        let baseline_detector = Arc::new(RwLock::new(create_hf_hmm_regime_detector()));

        // Initialize performance validator
        let validation_config = PerformanceValidationConfig {
            target_latency_ms: config.inference_timeout_ms as f64,
            target_accuracy: config.performance_threshold,
            validation_samples: 1000,
            warmup_samples: 100,
            real_data_validation: true,
            stress_test_enabled: true,
            concurrent_inference_tests: config.max_concurrent_inferences,
        };
        let performance_validator = Arc::new(RwLock::new(
            HMMPerformanceValidator::new(validation_config, market_data_manager.clone())
        ));

        // Initialize A/B testing
        let ab_test_config = ABTestConfig {
            test_duration_hours: 24,
            traffic_split_percentage: 50.0,
            significance_threshold: 0.05,
            min_sample_size: 1000,
            enable_automatic_winner_selection: true,
        };
        let ab_test_manager = Arc::new(RwLock::new(ABTestManager {
            test_config: ab_test_config,
            enhanced_metrics: ABTestMetrics::default(),
            baseline_metrics: ABTestMetrics::default(),
            current_test_id: None,
            test_start_time: None,
            traffic_split: 0.5,
        }));

        // Initialize performance metrics
        let performance_metrics = Arc::new(RwLock::new(IntegrationPerformanceMetrics::default()));

        // Create event system
        let (event_sender, _event_receiver) = mpsc::unbounded_channel();

        let manager = Self {
            config,
            enhanced_detector,
            multi_scale_detector,
            optimized_inference,
            baseline_detector,
            performance_validator,
            performance_metrics,
            ab_test_manager,
            event_sender,
            market_data_manager,
            database,
        };

        info!("✅ HMM Integration Manager initialized successfully");
        Ok(manager)
    }

    /// Start HMM integration services
    pub async fn start_integration_services(&self) -> Result<()> {
        info!("🔧 Starting HMM integration services...");

        // Start performance monitoring
        if self.config.enable_performance_monitoring {
            self.start_performance_monitoring().await?;
        }

        // Start A/B testing if enabled
        if self.config.enable_a_b_testing {
            self.start_ab_testing().await?;
        }

        // Start validation loop
        self.start_validation_loop().await?;

        info!("✅ All HMM integration services started successfully");
        Ok(())
    }

    /// Perform regime detection with enhanced HMM
    pub async fn detect_regime(&self, market_tick: &MarketTick) -> Result<Option<RegimeSignal>> {
        let start_time = Instant::now();

        // Determine which model to use based on A/B testing
        let use_enhanced = if self.config.enable_a_b_testing {
            self.should_use_enhanced_model().await
        } else {
            self.config.enable_enhanced_hmm
        };

        let result = if use_enhanced {
            self.detect_regime_enhanced(market_tick).await
        } else {
            self.detect_regime_baseline(market_tick).await
        };

        // Record performance metrics
        let latency_ms = start_time.elapsed().as_nanos() as f64 / 1_000_000.0;
        self.record_inference_metrics(market_tick.instrument_id, latency_ms, use_enhanced).await;

        // Send event
        let model_type = if use_enhanced { "enhanced" } else { "baseline" }.to_string();
        let _ = self.event_sender.send(HMMIntegrationEvent::InferenceCompleted {
            instrument_id: market_tick.instrument_id,
            latency_ms,
            accuracy: None, // Will be calculated later
            model_type,
        });

        result
    }

    /// Enhanced regime detection using optimized HMM
    async fn detect_regime_enhanced(&self, market_tick: &MarketTick) -> Result<Option<RegimeSignal>> {
        // Convert market tick to observation
        let observation = self.create_market_observation(market_tick).await?;

        if self.config.enable_multi_scale {
            // Use multi-scale detector for better accuracy
            let mut detector = self.multi_scale_detector.write().await;
            detector.update_with_tick(market_tick)?;

            // Convert MultiScaleRegimeSignal to RegimeSignal if available
            Ok(detector.detect_current_regime().and_then(|multi_scale_signal| {
                multi_scale_signal.consensus_regime.map(|consensus_regime| RegimeSignal {
                    current_regime: consensus_regime,
                    regime: consensus_regime,
                    confidence: multi_scale_signal.consensus_confidence,
                    timestamp: multi_scale_signal.timestamp,
                    transition_probability: multi_scale_signal.transition_probability,
                    regime_strength: multi_scale_signal.regime_strength,
                    expected_duration_minutes: 30, // Default duration
                })
            }))
        } else {
            // Use enhanced single-scale detector with optimized inference
            let mut detector = self.enhanced_detector.write().await;
            let mut inference = self.optimized_inference.write().await;

            detector.update_with_tick(market_tick)?;
            Ok(inference.fast_regime_detection(&*detector, &observation)?)
        }
    }

    /// Baseline regime detection for A/B testing
    async fn detect_regime_baseline(&self, market_tick: &MarketTick) -> Result<Option<RegimeSignal>> {
        let mut detector = self.baseline_detector.write().await;
        detector.update_with_tick(market_tick)?;
        Ok(detector.detect_current_regime())
    }

    /// Determine if enhanced model should be used based on A/B testing
    async fn should_use_enhanced_model(&self) -> bool {
        let ab_manager = self.ab_test_manager.read().await;

        // Use traffic split to determine model selection
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let random_value: f64 = rng.gen();

        random_value < ab_manager.traffic_split
    }

    /// Create market observation from market tick
    async fn create_market_observation(&self, tick: &MarketTick) -> Result<crate::ai::hmm_regime::MarketObservation> {
        use crate::ai::hmm_regime::MarketObservation;
        use ndarray::Array1;

        // Calculate basic features
        let mid_price = (tick.bid_price + tick.ask_price) / 2.0;
        let spread = tick.ask_price - tick.bid_price;
        let volume = tick.volume.unwrap_or(0.0);

        // Create comprehensive feature vector
        let features = Array1::from(vec![
            mid_price,
            volume,
            spread,
            mid_price.ln(),
            spread / mid_price, // Relative spread
            tick.bid_size,
            tick.ask_size,
            tick.data_quality_score,
            // Additional features for enhanced detection
            0.02, // volatility (placeholder)
            0.0,  // trend (placeholder)
            0.0,  // momentum (placeholder)
            0.0,  // price_skewness (placeholder)
            0.0,  // price_kurtosis (placeholder)
            0.0,  // autocorrelation (placeholder)
            0.0,  // order_flow_imbalance (placeholder)
            0.0,  // effective_spread (placeholder)
        ]);

        Ok(MarketObservation {
            timestamp: tick.timestamp,
            features,
            volume,
            volatility: 0.02, // Simplified
            trend: 0.0,
            bid_ask_spread: spread,
            transition_probability: 0.1,
            // Additional fields with default values
            momentum: 0.0,
            price_skewness: 0.0,  // Fixed field name
            price_kurtosis: 0.0,  // Fixed field name
            autocorrelation: 0.0,
            regime_persistence: 0.5,  // Added missing field
            order_flow_imbalance: 0.0,
            effective_spread: spread,
            price_impact: 0.0,
            market_depth_ratio: tick.bid_size / (tick.ask_size + 1e-8),
            garch_volatility: 0.02,
            volatility_persistence: 0.5,
            volatility_clustering_score: 0.0,  // Added missing field
            hurst_exponent: 0.5,
            fractal_dimension: 1.5,
            regime_strength: 0.5,
            regime_transition_signal: 0.0,  // Added missing field
        })
    }

    /// Record inference performance metrics
    async fn record_inference_metrics(&self, _instrument_id: Uuid, latency_ms: f64, is_enhanced: bool) {
        let mut metrics = self.performance_metrics.write().await;

        metrics.total_inferences += 1;

        // Update average latency using exponential moving average
        let alpha = 0.1;
        metrics.average_latency_ms = alpha * latency_ms + (1.0 - alpha) * metrics.average_latency_ms;

        // Add performance data point
        metrics.performance_trend.push(PerformanceDataPoint {
            timestamp: Utc::now(),
            latency_ms,
            accuracy: 0.0, // Will be updated later
            throughput: 1.0 / (latency_ms / 1000.0), // Inferences per second
        });

        // Keep only last 1000 data points
        if metrics.performance_trend.len() > 1000 {
            metrics.performance_trend.remove(0);
        }

        // Update A/B testing metrics
        let mut ab_manager = self.ab_test_manager.write().await;
        if is_enhanced {
            ab_manager.enhanced_metrics.total_predictions += 1;
            ab_manager.enhanced_metrics.average_latency_ms =
                alpha * latency_ms + (1.0 - alpha) * ab_manager.enhanced_metrics.average_latency_ms;
        } else {
            ab_manager.baseline_metrics.total_predictions += 1;
            ab_manager.baseline_metrics.average_latency_ms =
                alpha * latency_ms + (1.0 - alpha) * ab_manager.baseline_metrics.average_latency_ms;
        }
    }

    /// Start performance monitoring loop
    async fn start_performance_monitoring(&self) -> Result<()> {
        info!("📊 Starting HMM performance monitoring...");

        let validator = self.performance_validator.clone();
        let event_sender = self.event_sender.clone();
        let validation_interval = self.config.validation_interval_minutes;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_secs(validation_interval * 60)
            );

            loop {
                interval.tick().await;

                match validator.write().await.validate_performance().await {
                    Ok(results) => {
                        let summary = ValidationSummary {
                            overall_score: results.overall_performance_score,
                            latency_score: if results.latency_metrics.meets_target { 1.0 } else { 0.5 },
                            accuracy_score: if results.accuracy_metrics.meets_target { 1.0 } else { 0.5 },
                            meets_requirements: results.meets_requirements,
                            recommendations: results.recommendations,
                        };

                        let _ = event_sender.send(HMMIntegrationEvent::ValidationCompleted {
                            validation_results: summary,
                        });

                        info!("✅ HMM validation completed - Score: {:.2}", results.overall_performance_score);
                    }
                    Err(e) => {
                        error!("❌ HMM validation failed: {}", e);
                        let _ = event_sender.send(HMMIntegrationEvent::PerformanceAlert {
                            alert_type: "ValidationFailure".to_string(),
                            message: format!("HMM validation failed: {}", e),
                            severity: AlertSeverity::Critical,
                        });
                    }
                }
            }
        });

        Ok(())
    }

    /// Start A/B testing framework
    async fn start_ab_testing(&self) -> Result<()> {
        info!("🧪 Starting HMM A/B testing framework...");

        let ab_manager = self.ab_test_manager.clone();
        let event_sender = self.event_sender.clone();

        // Initialize new A/B test
        {
            let mut manager = ab_manager.write().await;
            manager.current_test_id = Some(Uuid::new_v4());
            manager.test_start_time = Some(Utc::now());
            manager.traffic_split = manager.test_config.traffic_split_percentage / 100.0;

            info!("🧪 A/B Test started - ID: {:?}, Traffic Split: {:.1}%",
                  manager.current_test_id, manager.test_config.traffic_split_percentage);
        }

        // Start A/B test monitoring loop
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600)); // Check hourly

            loop {
                interval.tick().await;

                let manager = ab_manager.read().await;
                if let (Some(test_id), Some(start_time)) = (manager.current_test_id, manager.test_start_time) {
                    let test_duration = Utc::now().signed_duration_since(start_time);

                    // Check if test should end
                    if test_duration.num_hours() >= manager.test_config.test_duration_hours as i64 {
                        // Calculate performance scores
                        let enhanced_performance = Self::calculate_ab_performance(&manager.enhanced_metrics);
                        let baseline_performance = Self::calculate_ab_performance(&manager.baseline_metrics);

                        let _ = event_sender.send(HMMIntegrationEvent::ABTestUpdate {
                            test_id,
                            enhanced_performance,
                            baseline_performance,
                        });

                        info!("🧪 A/B Test completed - Enhanced: {:.2}, Baseline: {:.2}",
                              enhanced_performance, baseline_performance);
                    }
                }
            }
        });

        Ok(())
    }

    /// Calculate A/B test performance score
    fn calculate_ab_performance(metrics: &ABTestMetrics) -> f64 {
        if metrics.total_predictions == 0 {
            return 0.0;
        }

        let accuracy = metrics.correct_predictions as f64 / metrics.total_predictions as f64;
        let latency_score = if metrics.average_latency_ms <= 20.0 { 1.0 } else { 20.0 / metrics.average_latency_ms };
        let error_rate = metrics.error_count as f64 / metrics.total_predictions as f64;

        // Weighted performance score
        (accuracy * 0.5 + latency_score * 0.3 + (1.0 - error_rate) * 0.2).max(0.0).min(1.0)
    }

    /// Start validation loop
    async fn start_validation_loop(&self) -> Result<()> {
        info!("🔍 Starting HMM validation loop...");

        let performance_metrics = self.performance_metrics.clone();
        let event_sender = self.event_sender.clone();
        let performance_threshold = self.config.performance_threshold;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // 5 minutes

            loop {
                interval.tick().await;

                let metrics = performance_metrics.read().await;

                // Check performance thresholds
                if metrics.average_latency_ms > 20.0 {
                    let _ = event_sender.send(HMMIntegrationEvent::PerformanceAlert {
                        alert_type: "HighLatency".to_string(),
                        message: format!("Average latency {:.2}ms exceeds 20ms threshold", metrics.average_latency_ms),
                        severity: AlertSeverity::Warning,
                    });
                }

                if metrics.accuracy_score < performance_threshold {
                    let _ = event_sender.send(HMMIntegrationEvent::PerformanceAlert {
                        alert_type: "LowAccuracy".to_string(),
                        message: format!("Accuracy {:.2} below threshold {:.2}", metrics.accuracy_score, performance_threshold),
                        severity: AlertSeverity::Critical,
                    });
                }

                if metrics.error_rate > 0.05 {
                    let _ = event_sender.send(HMMIntegrationEvent::PerformanceAlert {
                        alert_type: "HighErrorRate".to_string(),
                        message: format!("Error rate {:.2}% exceeds 5% threshold", metrics.error_rate * 100.0),
                        severity: AlertSeverity::Warning,
                    });
                }
            }
        });

        Ok(())
    }

    /// Get current performance metrics
    pub async fn get_performance_metrics(&self) -> IntegrationPerformanceMetrics {
        self.performance_metrics.read().await.clone()
    }

    /// Get A/B testing results
    pub async fn get_ab_test_results(&self) -> ABTestManager {
        self.ab_test_manager.read().await.clone()
    }

    /// Force A/B test conclusion
    pub async fn conclude_ab_test(&self) -> Result<ABTestResults> {
        let mut manager = self.ab_test_manager.write().await;

        if manager.current_test_id.is_none() {
            return Err(PantherSwapError::ai_prediction("No active A/B test".to_string()));
        }

        let enhanced_performance = Self::calculate_ab_performance(&manager.enhanced_metrics);
        let baseline_performance = Self::calculate_ab_performance(&manager.baseline_metrics);

        let winner = if enhanced_performance > baseline_performance {
            "enhanced".to_string()
        } else {
            "baseline".to_string()
        };

        let results = ABTestResults {
            test_id: manager.current_test_id.unwrap(),
            enhanced_performance,
            baseline_performance,
            winner: winner.clone(),
            confidence_level: 0.95, // Simplified
            sample_size_enhanced: manager.enhanced_metrics.total_predictions,
            sample_size_baseline: manager.baseline_metrics.total_predictions,
        };

        // Reset for next test
        manager.current_test_id = None;
        manager.test_start_time = None;
        manager.enhanced_metrics = ABTestMetrics::default();
        manager.baseline_metrics = ABTestMetrics::default();

        info!("🏆 A/B Test concluded - Winner: {}, Enhanced: {:.2}, Baseline: {:.2}",
              winner, enhanced_performance, baseline_performance);

        Ok(results)
    }

    /// Update accuracy for completed predictions
    pub async fn update_prediction_accuracy(&self, _instrument_id: Uuid, was_correct: bool, model_type: &str) {
        let mut metrics = self.performance_metrics.write().await;

        if was_correct {
            metrics.successful_inferences += 1;
        }

        // Update accuracy score using exponential moving average
        let current_accuracy = if metrics.total_inferences > 0 {
            metrics.successful_inferences as f64 / metrics.total_inferences as f64
        } else {
            0.0
        };

        let alpha = 0.1;
        metrics.accuracy_score = alpha * current_accuracy + (1.0 - alpha) * metrics.accuracy_score;

        // Update A/B testing metrics
        let mut ab_manager = self.ab_test_manager.write().await;
        if model_type == "enhanced" {
            if was_correct {
                ab_manager.enhanced_metrics.correct_predictions += 1;
            }
        } else {
            if was_correct {
                ab_manager.baseline_metrics.correct_predictions += 1;
            }
        }
    }
}

/// A/B Test Results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestResults {
    pub test_id: Uuid,
    pub enhanced_performance: f64,
    pub baseline_performance: f64,
    pub winner: String,
    pub confidence_level: f64,
    pub sample_size_enhanced: u64,
    pub sample_size_baseline: u64,
}

/// Default implementations
impl Default for IntegrationPerformanceMetrics {
    fn default() -> Self {
        Self {
            total_inferences: 0,
            successful_inferences: 0,
            average_latency_ms: 0.0,
            accuracy_score: 0.0,
            cache_hit_rate: 0.0,
            error_rate: 0.0,
            last_validation: None,
            performance_trend: Vec::new(),
        }
    }
}

impl Default for ABTestMetrics {
    fn default() -> Self {
        Self {
            total_predictions: 0,
            correct_predictions: 0,
            average_latency_ms: 0.0,
            error_count: 0,
            confidence_scores: Vec::new(),
            regime_detection_accuracy: 0.0,
        }
    }
}
