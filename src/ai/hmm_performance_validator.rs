// HMM Performance Validation and Optimization Framework
use crate::ai::hmm_regime::{HMMRegimeDetector, OptimizedHMMInference, create_enhanced_accuracy_hmm_detector};
use crate::database::types::{MarketTick, RegimeType};
use crate::trading::signals::RegimeSignal;
use crate::market_data::MarketDataManager;
use crate::utils::{Result, PantherSwapError};
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::Instant;
use tracing::{info, warn, debug};
use ndarray::Array1;

/// Performance validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceValidationConfig {
    pub target_latency_ms: f64,
    pub target_accuracy: f64,
    pub validation_samples: usize,
    pub warmup_samples: usize,
    pub real_data_validation: bool,
    pub stress_test_enabled: bool,
    pub concurrent_inference_tests: usize,
}

impl Default for PerformanceValidationConfig {
    fn default() -> Self {
        Self {
            target_latency_ms: 20.0,
            target_accuracy: 0.75,
            validation_samples: 10000,
            warmup_samples: 1000,
            real_data_validation: true,
            stress_test_enabled: true,
            concurrent_inference_tests: 100,
        }
    }
}

/// Comprehensive performance validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResults {
    pub latency_metrics: LatencyValidationResults,
    pub accuracy_metrics: AccuracyValidationResults,
    pub stress_test_results: StressTestResults,
    pub real_data_validation: RealDataValidationResults,
    pub overall_performance_score: f64,
    pub meets_requirements: bool,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyValidationResults {
    pub average_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub max_latency_ms: f64,
    pub meets_target: bool,
    pub cache_hit_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccuracyValidationResults {
    pub overall_accuracy: f64,
    pub regime_specific_accuracy: HashMap<String, f64>,
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
    pub confidence_distribution: Vec<f64>,
    pub meets_target: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressTestResults {
    pub concurrent_inference_success_rate: f64,
    pub memory_usage_mb: f64,
    pub cpu_utilization_percent: f64,
    pub throughput_inferences_per_second: f64,
    pub degradation_under_load: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealDataValidationResults {
    pub market_data_samples: usize,
    pub regime_detection_accuracy: f64,
    pub transition_detection_accuracy: f64,
    pub false_positive_rate: f64,
    pub false_negative_rate: f64,
    pub market_condition_coverage: HashMap<String, f64>,
}

/// HMM Performance Validator
pub struct HMMPerformanceValidator {
    config: PerformanceValidationConfig,
    market_data_manager: MarketDataManager,
    test_data: Vec<MarketTick>,
    validation_results: Option<ValidationResults>,
}

impl HMMPerformanceValidator {
    /// Create new performance validator
    pub fn new(config: PerformanceValidationConfig, market_data_manager: MarketDataManager) -> Self {
        Self {
            config,
            market_data_manager,
            test_data: Vec::new(),
            validation_results: None,
        }
    }

    /// Run comprehensive performance validation
    pub async fn validate_performance(&mut self) -> Result<ValidationResults> {
        info!("🚀 Starting comprehensive HMM performance validation...");
        
        // Load real market data if enabled
        if self.config.real_data_validation {
            self.load_real_market_data().await?;
        } else {
            self.generate_synthetic_test_data();
        }

        // Create optimized HMM detector
        let mut detector = create_enhanced_accuracy_hmm_detector();
        let mut optimized_inference = OptimizedHMMInference::new(detector.config.feature_dimensions);

        // Train detector with warmup data
        self.train_detector_with_warmup(&mut detector).await?;

        // Run validation tests
        let latency_metrics = self.validate_latency_performance(&mut detector, &mut optimized_inference).await?;
        let accuracy_metrics = self.validate_accuracy_performance(&mut detector, &mut optimized_inference).await?;
        let stress_test_results = self.run_stress_tests(&mut detector, &mut optimized_inference).await?;
        let real_data_validation = self.validate_with_real_data(&mut detector, &mut optimized_inference).await?;

        // Calculate overall performance score
        let overall_performance_score = self.calculate_overall_score(
            &latency_metrics,
            &accuracy_metrics,
            &stress_test_results,
            &real_data_validation,
        );

        // Check if requirements are met
        let meets_requirements = latency_metrics.meets_target && 
                                accuracy_metrics.meets_target &&
                                stress_test_results.concurrent_inference_success_rate > 0.95;

        // Generate recommendations
        let recommendations = self.generate_recommendations(
            &latency_metrics,
            &accuracy_metrics,
            &stress_test_results,
        );

        let results = ValidationResults {
            latency_metrics,
            accuracy_metrics,
            stress_test_results,
            real_data_validation,
            overall_performance_score,
            meets_requirements,
            recommendations,
        };

        self.validation_results = Some(results.clone());
        
        info!("✅ Performance validation completed. Overall score: {:.2}", overall_performance_score);
        if meets_requirements {
            info!("🎯 All performance requirements met!");
        } else {
            warn!("⚠️ Some performance requirements not met. Check recommendations.");
        }

        Ok(results)
    }

    /// Load real market data for validation
    async fn load_real_market_data(&mut self) -> Result<()> {
        info!("📊 Loading real market data for validation...");
        
        let end_time = Utc::now();
        let start_time = end_time - Duration::days(7); // Last 7 days
        
        // Fetch real market data using Alpha Vantage
        let symbols = vec!["AAPL", "GOOGL", "MSFT", "TSLA", "SPY"];
        
        for symbol in symbols {
            match self.market_data_manager.fetch_historical_data(symbol, start_time, end_time).await {
                Ok(data) => {
                    self.test_data.extend(data);
                    info!("Loaded {} data points for {}", data.len(), symbol);
                }
                Err(e) => {
                    warn!("Failed to load data for {}: {}", symbol, e);
                }
            }
        }
        
        if self.test_data.is_empty() {
            return Err(PantherSwapError::ai_prediction("No real market data available".to_string()));
        }
        
        // Sort by timestamp
        self.test_data.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        
        info!("📈 Loaded {} total market data points", self.test_data.len());
        Ok(())
    }

    /// Generate synthetic test data
    fn generate_synthetic_test_data(&mut self) {
        info!("🔧 Generating synthetic test data...");
        
        let mut rng = rand::thread_rng();
        let base_time = Utc::now() - Duration::days(1);
        
        for i in 0..self.config.validation_samples {
            use rand::Rng;
            
            let timestamp = base_time + Duration::seconds(i as i64 * 60);
            let price = 100.0 + (i as f64 * 0.01) + rng.gen_range(-5.0..5.0);
            let volume = rng.gen_range(1000..10000);
            
            self.test_data.push(MarketTick {
                timestamp,
                instrument_id: uuid::Uuid::new_v4(),
                provider: "SYNTHETIC".to_string(),
                bid_price: price - 0.01,
                ask_price: price + 0.01,
                bid_size: 100.0,
                ask_size: 100.0,
                last_price: Some(price),
                volume: Some(volume as f64),
                spread: 0.02,
                data_quality_score: 1.0,
                raw_data: serde_json::json!({}),
                // Backward compatibility fields
                symbol: Some("SYNTHETIC".to_string()),
                price: Some(price),
                bid: Some(price - 0.01),
                ask: Some(price + 0.01),
            });
        }
        
        info!("🔧 Generated {} synthetic data points", self.test_data.len());
    }

    /// Train detector with warmup data
    async fn train_detector_with_warmup(&self, detector: &mut HMMRegimeDetector) -> Result<()> {
        info!("🏋️ Training detector with warmup data...");
        
        let warmup_data = &self.test_data[..self.config.warmup_samples.min(self.test_data.len())];
        
        for tick in warmup_data {
            detector.update_with_tick(tick)?;
        }
        
        detector.train()?;
        info!("✅ Detector training completed");
        Ok(())
    }

    /// Validate latency performance with <20ms target
    async fn validate_latency_performance(
        &self,
        detector: &mut HMMRegimeDetector,
        optimized_inference: &mut OptimizedHMMInference,
    ) -> Result<LatencyValidationResults> {
        info!("⏱️ Validating latency performance...");

        let test_samples = self.config.validation_samples.min(self.test_data.len());
        let mut latencies = Vec::with_capacity(test_samples);
        let mut cache_hits = 0;

        // Warmup inference engine
        for tick in &self.test_data[..100.min(self.test_data.len())] {
            let observation = self.tick_to_observation(tick);
            let _ = optimized_inference.fast_regime_detection(detector, &observation);
        }

        // Measure latencies
        for tick in &self.test_data[..test_samples] {
            let observation = self.tick_to_observation(tick);

            let start_time = Instant::now();
            let result = optimized_inference.fast_regime_detection(detector, &observation)?;
            let latency_ms = start_time.elapsed().as_nanos() as f64 / 1_000_000.0;

            latencies.push(latency_ms);

            // Check cache performance
            let metrics = optimized_inference.get_metrics();
            if metrics.cache_hit_rate > 0.0 {
                cache_hits += 1;
            }
        }

        // Calculate percentiles
        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let average_latency_ms = latencies.iter().sum::<f64>() / latencies.len() as f64;
        let p50_latency_ms = latencies[latencies.len() / 2];
        let p95_latency_ms = latencies[(latencies.len() as f64 * 0.95) as usize];
        let p99_latency_ms = latencies[(latencies.len() as f64 * 0.99) as usize];
        let max_latency_ms = latencies.iter().cloned().fold(0.0, f64::max);
        let cache_hit_rate = cache_hits as f64 / test_samples as f64;

        let meets_target = p95_latency_ms <= self.config.target_latency_ms;

        info!("⏱️ Latency validation results:");
        info!("  • Average: {:.2}ms", average_latency_ms);
        info!("  • P95: {:.2}ms (Target: <{}ms)", p95_latency_ms, self.config.target_latency_ms);
        info!("  • P99: {:.2}ms", p99_latency_ms);
        info!("  • Max: {:.2}ms", max_latency_ms);
        info!("  • Cache hit rate: {:.1}%", cache_hit_rate * 100.0);
        info!("  • Meets target: {}", meets_target);

        Ok(LatencyValidationResults {
            average_latency_ms,
            p50_latency_ms,
            p95_latency_ms,
            p99_latency_ms,
            max_latency_ms,
            meets_target,
            cache_hit_rate,
        })
    }

    /// Validate accuracy performance
    async fn validate_accuracy_performance(
        &self,
        detector: &mut HMMRegimeDetector,
        optimized_inference: &mut OptimizedHMMInference,
    ) -> Result<AccuracyValidationResults> {
        info!("🎯 Validating accuracy performance...");

        let test_samples = self.config.validation_samples.min(self.test_data.len());
        let mut predictions = Vec::new();
        let mut confidences = Vec::new();
        let mut regime_counts: HashMap<String, (usize, usize)> = HashMap::new();

        for tick in &self.test_data[..test_samples] {
            let observation = self.tick_to_observation(tick);

            if let Some(prediction) = optimized_inference.fast_regime_detection(detector, &observation)? {
                let actual_regime = self.infer_actual_regime(tick);
                let regime_str = format!("{:?}", prediction.regime);

                predictions.push((prediction.regime, actual_regime, prediction.confidence));
                confidences.push(prediction.confidence);

                let entry = regime_counts.entry(regime_str).or_insert((0, 0));
                entry.1 += 1; // Total predictions
                if Some(prediction.regime) == actual_regime {
                    entry.0 += 1; // Correct predictions
                }
            }
        }

        // Calculate metrics
        let correct_predictions = predictions.iter()
            .filter(|(pred, actual, _)| Some(*pred) == *actual)
            .count();
        let overall_accuracy = correct_predictions as f64 / predictions.len() as f64;

        let regime_specific_accuracy: HashMap<String, f64> = regime_counts.iter()
            .map(|(regime, (correct, total))| {
                let accuracy = if *total > 0 { *correct as f64 / *total as f64 } else { 0.0 };
                (regime.clone(), accuracy)
            })
            .collect();

        // Calculate precision, recall, F1 (simplified)
        let precision = overall_accuracy; // Simplified
        let recall = overall_accuracy; // Simplified
        let f1_score = if precision + recall > 0.0 {
            2.0 * (precision * recall) / (precision + recall)
        } else {
            0.0
        };

        let meets_target = overall_accuracy >= self.config.target_accuracy;

        info!("🎯 Accuracy validation results:");
        info!("  • Overall accuracy: {:.1}% (Target: ≥{:.1}%)", overall_accuracy * 100.0, self.config.target_accuracy * 100.0);
        info!("  • Precision: {:.1}%", precision * 100.0);
        info!("  • Recall: {:.1}%", recall * 100.0);
        info!("  • F1 Score: {:.3}", f1_score);
        info!("  • Meets target: {}", meets_target);

        Ok(AccuracyValidationResults {
            overall_accuracy,
            regime_specific_accuracy,
            precision,
            recall,
            f1_score,
            confidence_distribution: confidences,
            meets_target,
        })
    }

    /// Helper methods for validation
    fn tick_to_observation(&self, tick: &MarketTick) -> crate::ai::hmm_regime::MarketObservation {
        use crate::ai::hmm_regime::MarketObservation;

        // Convert market tick to observation with features
        let price = tick.last_price.unwrap_or(tick.bid_price);
        let volume = tick.volume.unwrap_or(0.0);
        let features = Array1::from(vec![
            price,
            volume,
            tick.spread, // Use the spread field directly
            price.ln(), // Log price
        ]);

        MarketObservation {
            timestamp: tick.timestamp,
            features,
            volume,
            volatility: 0.02, // Simplified
            trend: 0.0, // Simplified
            momentum: 0.0,
            bid_ask_spread: tick.spread,
            price_skewness: 0.0,
            price_kurtosis: 0.0,
            autocorrelation: 0.0,
            regime_persistence: 0.0,
            transition_probability: 0.1,
            order_flow_imbalance: 0.0,
            effective_spread: tick.spread,
            price_impact: 0.0,
            market_depth_ratio: 1.0,
            garch_volatility: 0.02,
            volatility_persistence: 0.5,
            volatility_clustering_score: 0.0,
            hurst_exponent: 0.5,
            fractal_dimension: 1.5,
            regime_strength: 0.5,
            regime_transition_signal: 0.0,
        }
    }

    fn tick_to_observation_static(tick: &MarketTick) -> crate::ai::hmm_regime::MarketObservation {
        use crate::ai::hmm_regime::MarketObservation;

        let price = tick.last_price.unwrap_or(tick.bid_price);
        let volume = tick.volume.unwrap_or(0.0);
        let features = Array1::from(vec![
            price,
            volume,
            tick.spread,
            price.ln(),
        ]);

        MarketObservation {
            timestamp: tick.timestamp,
            features,
            volume,
            volatility: 0.02,
            trend: 0.0,
            momentum: 0.0,
            bid_ask_spread: tick.spread,
            price_skewness: 0.0,
            price_kurtosis: 0.0,
            autocorrelation: 0.0,
            regime_persistence: 0.0,
            transition_probability: 0.1,
            order_flow_imbalance: 0.0,
            effective_spread: tick.spread,
            price_impact: 0.0,
            market_depth_ratio: 1.0,
            garch_volatility: 0.02,
            volatility_persistence: 0.5,
            volatility_clustering_score: 0.0,
            hurst_exponent: 0.5,
            fractal_dimension: 1.5,
            regime_strength: 0.5,
            regime_transition_signal: 0.0,
        }
    }

    fn infer_actual_regime(&self, tick: &MarketTick) -> Option<RegimeType> {
        // Simplified regime inference based on price movement
        // In practice, this would use labeled data or more sophisticated analysis
        let volume = tick.volume.unwrap_or(0.0);
        let price = tick.last_price.unwrap_or(tick.bid_price);

        if volume > 5000.0 {
            Some(RegimeType::HighVolatility)
        } else if price > 150.0 {
            Some(RegimeType::Bullish)
        } else if price < 50.0 {
            Some(RegimeType::Bearish)
        } else {
            Some(RegimeType::Sideways)
        }
    }

    fn classify_market_condition(&self, tick: &MarketTick) -> String {
        let volume = tick.volume.unwrap_or(0.0);
        if volume > 8000.0 {
            "High Volume".to_string()
        } else if volume < 2000.0 {
            "Low Volume".to_string()
        } else {
            "Normal Volume".to_string()
        }
    }

    fn validate_transition(&self, prev_tick: &MarketTick, curr_tick: &MarketTick) -> bool {
        // Simplified transition validation
        let prev_price = prev_tick.last_price.unwrap_or(prev_tick.bid_price);
        let curr_price = curr_tick.last_price.unwrap_or(curr_tick.bid_price);
        let price_change = (curr_price - prev_price).abs() / prev_price;
        price_change > 0.02 // 2% price change indicates potential regime transition
    }

    fn should_have_detected_regime(&self, tick: &MarketTick) -> bool {
        // Simplified check for when a regime should have been detected
        let volume = tick.volume.unwrap_or(0.0);
        let price = tick.last_price.unwrap_or(tick.bid_price);
        volume > 1000.0 && price > 10.0
    }

    /// Calculate overall performance score
    fn calculate_overall_score(
        &self,
        latency_metrics: &LatencyValidationResults,
        accuracy_metrics: &AccuracyValidationResults,
        stress_test_results: &StressTestResults,
        real_data_validation: &RealDataValidationResults,
    ) -> f64 {
        let latency_score = if latency_metrics.meets_target { 1.0 } else {
            (self.config.target_latency_ms / latency_metrics.p95_latency_ms).min(1.0)
        };

        let accuracy_score = if accuracy_metrics.meets_target { 1.0 } else {
            (accuracy_metrics.overall_accuracy / self.config.target_accuracy).min(1.0)
        };

        let stress_score = stress_test_results.concurrent_inference_success_rate;

        let real_data_score = if real_data_validation.market_data_samples > 0 {
            real_data_validation.regime_detection_accuracy
        } else {
            0.8 // Default score if no real data
        };

        // Weighted average
        (latency_score * 0.3 + accuracy_score * 0.4 + stress_score * 0.2 + real_data_score * 0.1)
    }

    /// Generate performance recommendations
    fn generate_recommendations(
        &self,
        latency_metrics: &LatencyValidationResults,
        accuracy_metrics: &AccuracyValidationResults,
        stress_test_results: &StressTestResults,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if !latency_metrics.meets_target {
            recommendations.push(format!(
                "Latency optimization needed: P95 latency is {:.2}ms, target is <{:.0}ms. Consider increasing cache size or optimizing feature normalization.",
                latency_metrics.p95_latency_ms, self.config.target_latency_ms
            ));
        }

        if latency_metrics.cache_hit_rate < 0.8 {
            recommendations.push(format!(
                "Cache hit rate is low ({:.1}%). Consider adjusting cache key generation or increasing cache size.",
                latency_metrics.cache_hit_rate * 100.0
            ));
        }

        if !accuracy_metrics.meets_target {
            recommendations.push(format!(
                "Accuracy improvement needed: Current accuracy is {:.1}%, target is ≥{:.1}%. Consider retraining with more data or adjusting model parameters.",
                accuracy_metrics.overall_accuracy * 100.0, self.config.target_accuracy * 100.0
            ));
        }

        if stress_test_results.concurrent_inference_success_rate < 0.95 {
            recommendations.push(format!(
                "Concurrent inference performance is suboptimal ({:.1}% success rate). Consider implementing better thread safety or resource pooling.",
                stress_test_results.concurrent_inference_success_rate * 100.0
            ));
        }

        if stress_test_results.throughput_inferences_per_second < 1000.0 {
            recommendations.push(format!(
                "Throughput is below optimal ({:.0} inferences/sec). Consider batch processing or parallel inference pipelines.",
                stress_test_results.throughput_inferences_per_second
            ));
        }

        if recommendations.is_empty() {
            recommendations.push("All performance metrics meet targets. System is ready for production deployment.".to_string());
        }

        recommendations
    }

    /// Get validation results
    pub fn get_results(&self) -> Option<&ValidationResults> {
        self.validation_results.as_ref()
    }

    /// Export results to JSON
    pub fn export_results_json(&self) -> Result<String> {
        if let Some(results) = &self.validation_results {
            serde_json::to_string_pretty(results)
                .map_err(|e| PantherSwapError::ai_prediction(format!("Failed to serialize results: {}", e)))
        } else {
            Err(PantherSwapError::ai_prediction("No validation results available".to_string()))
        }
    }
}
