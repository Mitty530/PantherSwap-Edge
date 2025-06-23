// Comprehensive HMM Performance Validation Tests
// Tests for <20ms inference latency, accuracy improvements, and real market data validation
// Run with: cargo test --test hmm_performance_validation_tests

use std::time::{Duration, Instant};
use tokio::time::sleep;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::{info, warn, error};

use pantherswap_edge::config::Settings;
use pantherswap_edge::database::Database;
use pantherswap_edge::market_data::MarketDataManager;
use pantherswap_edge::ai::hmm_performance_validator::{
    HMMPerformanceValidator, PerformanceValidationConfig, ValidationResults
};
use pantherswap_edge::ai::hmm_regime::{
    create_enhanced_accuracy_hmm_detector, create_hf_hmm_regime_detector, 
    OptimizedHMMInference, HMMRegimeDetector
};

mod common;
use common::*;

/// Test configuration for HMM performance validation
#[derive(Debug, Clone)]
pub struct HMMPerformanceTestConfig {
    pub latency_target_ms: f64,
    pub accuracy_target: f64,
    pub test_samples: usize,
    pub concurrent_tests: usize,
    pub real_data_enabled: bool,
}

impl Default for HMMPerformanceTestConfig {
    fn default() -> Self {
        Self {
            latency_target_ms: 20.0,
            accuracy_target: 0.75,
            test_samples: 5000,
            concurrent_tests: 50,
            real_data_enabled: true,
        }
    }
}

#[tokio::test]
async fn test_hmm_inference_latency_optimization() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();
    info!("🚀 Testing HMM inference latency optimization (<20ms target)");

    let settings = Settings::load()?;
    let database = Database::new(&settings.database.url).await?;
    let market_data_manager = MarketDataManager::new(&settings, database.clone()).await?;

    let config = PerformanceValidationConfig {
        target_latency_ms: 20.0,
        target_accuracy: 0.75,
        validation_samples: 1000,
        warmup_samples: 100,
        real_data_validation: false, // Use synthetic data for speed
        stress_test_enabled: true,
        concurrent_inference_tests: 25,
    };

    let mut validator = HMMPerformanceValidator::new(config, market_data_manager);
    let results = validator.validate_performance().await?;

    // Verify latency requirements
    assert!(results.latency_metrics.p95_latency_ms <= 20.0, 
        "P95 latency {:.2}ms exceeds 20ms target", results.latency_metrics.p95_latency_ms);
    
    assert!(results.latency_metrics.average_latency_ms <= 15.0,
        "Average latency {:.2}ms exceeds 15ms target", results.latency_metrics.average_latency_ms);

    // Verify cache performance
    assert!(results.latency_metrics.cache_hit_rate >= 0.7,
        "Cache hit rate {:.1}% is below 70% target", results.latency_metrics.cache_hit_rate * 100.0);

    info!("✅ Latency optimization test passed");
    info!("  • P95 latency: {:.2}ms", results.latency_metrics.p95_latency_ms);
    info!("  • Average latency: {:.2}ms", results.latency_metrics.average_latency_ms);
    info!("  • Cache hit rate: {:.1}%", results.latency_metrics.cache_hit_rate * 100.0);

    Ok(())
}

#[tokio::test]
async fn test_hmm_accuracy_validation() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();
    info!("🎯 Testing HMM accuracy validation and improvements");

    let settings = Settings::load()?;
    let database = Database::new(&settings.database.url).await?;
    let market_data_manager = MarketDataManager::new(&settings, database.clone()).await?;

    let config = PerformanceValidationConfig {
        target_latency_ms: 20.0,
        target_accuracy: 0.75,
        validation_samples: 2000,
        warmup_samples: 200,
        real_data_validation: false,
        stress_test_enabled: false,
        concurrent_inference_tests: 10,
    };

    let mut validator = HMMPerformanceValidator::new(config, market_data_manager);
    let results = validator.validate_performance().await?;

    // Verify accuracy requirements
    assert!(results.accuracy_metrics.overall_accuracy >= 0.75,
        "Overall accuracy {:.1}% is below 75% target", results.accuracy_metrics.overall_accuracy * 100.0);

    assert!(results.accuracy_metrics.f1_score >= 0.7,
        "F1 score {:.3} is below 0.7 target", results.accuracy_metrics.f1_score);

    // Check regime-specific accuracy
    for (regime, accuracy) in &results.accuracy_metrics.regime_specific_accuracy {
        assert!(*accuracy >= 0.6,
            "Regime {} accuracy {:.1}% is below 60% minimum", regime, accuracy * 100.0);
    }

    info!("✅ Accuracy validation test passed");
    info!("  • Overall accuracy: {:.1}%", results.accuracy_metrics.overall_accuracy * 100.0);
    info!("  • F1 score: {:.3}", results.accuracy_metrics.f1_score);
    info!("  • Precision: {:.1}%", results.accuracy_metrics.precision * 100.0);
    info!("  • Recall: {:.1}%", results.accuracy_metrics.recall * 100.0);

    Ok(())
}

#[tokio::test]
async fn test_hmm_stress_testing() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();
    info!("💪 Testing HMM stress testing and concurrent inference");

    let settings = Settings::load()?;
    let database = Database::new(&settings.database.url).await?;
    let market_data_manager = MarketDataManager::new(&settings, database.clone()).await?;

    let config = PerformanceValidationConfig {
        target_latency_ms: 20.0,
        target_accuracy: 0.70,
        validation_samples: 500,
        warmup_samples: 50,
        real_data_validation: false,
        stress_test_enabled: true,
        concurrent_inference_tests: 100, // High concurrency
    };

    let mut validator = HMMPerformanceValidator::new(config, market_data_manager);
    let results = validator.validate_performance().await?;

    // Verify stress test performance
    assert!(results.stress_test_results.concurrent_inference_success_rate >= 0.95,
        "Concurrent inference success rate {:.1}% is below 95% target", 
        results.stress_test_results.concurrent_inference_success_rate * 100.0);

    assert!(results.stress_test_results.throughput_inferences_per_second >= 500.0,
        "Throughput {:.0} inferences/sec is below 500/sec target",
        results.stress_test_results.throughput_inferences_per_second);

    assert!(results.stress_test_results.degradation_under_load <= 0.1,
        "Performance degradation under load {:.1}% exceeds 10% limit",
        results.stress_test_results.degradation_under_load * 100.0);

    info!("✅ Stress testing passed");
    info!("  • Concurrent success rate: {:.1}%", results.stress_test_results.concurrent_inference_success_rate * 100.0);
    info!("  • Throughput: {:.0} inferences/sec", results.stress_test_results.throughput_inferences_per_second);
    info!("  • Memory usage: {:.1} MB", results.stress_test_results.memory_usage_mb);

    Ok(())
}

#[tokio::test]
async fn test_hmm_real_data_validation() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();
    info!("📊 Testing HMM validation with real market data");

    let settings = Settings::load()?;
    let database = Database::new(&settings.database.url).await?;
    let market_data_manager = MarketDataManager::new(&settings, database.clone()).await?;

    let config = PerformanceValidationConfig {
        target_latency_ms: 20.0,
        target_accuracy: 0.70, // Slightly lower for real data
        validation_samples: 1000,
        warmup_samples: 100,
        real_data_validation: true, // Enable real data
        stress_test_enabled: false,
        concurrent_inference_tests: 10,
    };

    let mut validator = HMMPerformanceValidator::new(config, market_data_manager);
    
    // This test may fail if Alpha Vantage API is not available
    match validator.validate_performance().await {
        Ok(results) => {
            // Verify real data validation results
            assert!(results.real_data_validation.market_data_samples > 0,
                "No real market data samples loaded");

            assert!(results.real_data_validation.regime_detection_accuracy >= 0.6,
                "Real data regime detection accuracy {:.1}% is below 60% minimum",
                results.real_data_validation.regime_detection_accuracy * 100.0);

            assert!(results.real_data_validation.false_positive_rate <= 0.3,
                "False positive rate {:.1}% exceeds 30% limit",
                results.real_data_validation.false_positive_rate * 100.0);

            info!("✅ Real data validation passed");
            info!("  • Market data samples: {}", results.real_data_validation.market_data_samples);
            info!("  • Regime detection accuracy: {:.1}%", results.real_data_validation.regime_detection_accuracy * 100.0);
            info!("  • False positive rate: {:.1}%", results.real_data_validation.false_positive_rate * 100.0);
            info!("  • False negative rate: {:.1}%", results.real_data_validation.false_negative_rate * 100.0);
        }
        Err(e) => {
            warn!("Real data validation failed (possibly due to API limits): {}", e);
            // Don't fail the test if it's just an API issue
            if e.to_string().contains("No real market data available") {
                info!("⚠️ Skipping real data validation due to API unavailability");
            } else {
                return Err(e);
            }
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_hmm_comprehensive_performance_validation() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();
    info!("🎯 Running comprehensive HMM performance validation");

    let settings = Settings::load()?;
    let database = Database::new(&settings.database.url).await?;
    let market_data_manager = MarketDataManager::new(&settings, database.clone()).await?;

    let config = PerformanceValidationConfig::default();
    let mut validator = HMMPerformanceValidator::new(config, market_data_manager);
    
    let start_time = Instant::now();
    let results = validator.validate_performance().await?;
    let total_time = start_time.elapsed();

    // Verify overall performance score
    assert!(results.overall_performance_score >= 0.8,
        "Overall performance score {:.2} is below 0.8 target", results.overall_performance_score);

    // Verify that all major requirements are met
    assert!(results.meets_requirements,
        "Performance requirements not met. Check recommendations: {:?}", results.recommendations);

    // Export results for analysis
    let results_json = validator.export_results_json()?;
    info!("📊 Performance validation results exported");

    info!("✅ Comprehensive performance validation completed in {:.2}s", total_time.as_secs_f64());
    info!("  • Overall performance score: {:.2}", results.overall_performance_score);
    info!("  • Requirements met: {}", results.meets_requirements);
    info!("  • Recommendations: {}", results.recommendations.len());

    // Log key metrics
    info!("📈 Key Performance Metrics:");
    info!("  • P95 Latency: {:.2}ms (Target: <20ms)", results.latency_metrics.p95_latency_ms);
    info!("  • Overall Accuracy: {:.1}% (Target: ≥75%)", results.accuracy_metrics.overall_accuracy * 100.0);
    info!("  • Concurrent Success Rate: {:.1}%", results.stress_test_results.concurrent_inference_success_rate * 100.0);
    info!("  • Cache Hit Rate: {:.1}%", results.latency_metrics.cache_hit_rate * 100.0);

    Ok(())
}

#[tokio::test]
async fn test_hmm_optimization_comparison() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();
    info!("⚡ Testing HMM optimization improvements vs baseline");

    // Test baseline HMM detector
    let mut baseline_detector = create_enhanced_accuracy_hmm_detector();
    let mut baseline_inference = OptimizedHMMInference::new(baseline_detector.config.feature_dimensions);

    // Test high-frequency optimized detector
    let mut hf_detector = create_hf_hmm_regime_detector();
    let mut hf_inference = OptimizedHMMInference::new(hf_detector.config.feature_dimensions);

    // Generate test data
    let test_data = generate_synthetic_market_data(1000);

    // Benchmark baseline performance
    let baseline_latencies = benchmark_detector_latency(&mut baseline_detector, &mut baseline_inference, &test_data).await?;
    
    // Benchmark optimized performance
    let hf_latencies = benchmark_detector_latency(&mut hf_detector, &mut hf_inference, &test_data).await?;

    // Calculate improvements
    let baseline_avg = baseline_latencies.iter().sum::<f64>() / baseline_latencies.len() as f64;
    let hf_avg = hf_latencies.iter().sum::<f64>() / hf_latencies.len() as f64;
    let improvement_percent = ((baseline_avg - hf_avg) / baseline_avg) * 100.0;

    info!("⚡ Optimization comparison results:");
    info!("  • Baseline average latency: {:.2}ms", baseline_avg);
    info!("  • Optimized average latency: {:.2}ms", hf_avg);
    info!("  • Performance improvement: {:.1}%", improvement_percent);

    // Verify that optimization provides meaningful improvement
    assert!(improvement_percent >= 10.0,
        "Optimization improvement {:.1}% is below 10% minimum", improvement_percent);

    assert!(hf_avg <= 20.0,
        "Optimized average latency {:.2}ms exceeds 20ms target", hf_avg);

    info!("✅ Optimization comparison test passed");

    Ok(())
}

/// Helper function to generate synthetic market data for testing
fn generate_synthetic_market_data(count: usize) -> Vec<pantherswap_edge::database::types::MarketTick> {
    use pantherswap_edge::database::types::MarketTick;
    use rand::Rng;

    let mut rng = rand::thread_rng();
    let mut data = Vec::with_capacity(count);
    let base_time = Utc::now() - chrono::Duration::hours(1);

    for i in 0..count {
        let timestamp = base_time + chrono::Duration::seconds(i as i64);
        let price = 100.0 + (i as f64 * 0.01) + rng.gen_range(-2.0..2.0);
        let volume = rng.gen_range(1000..5000);

        data.push(MarketTick {
            symbol: "TEST".to_string(),
            price,
            volume,
            timestamp,
            bid: price - 0.005,
            ask: price + 0.005,
        });
    }

    data
}

/// Helper function to benchmark detector latency
async fn benchmark_detector_latency(
    detector: &mut HMMRegimeDetector,
    inference: &mut OptimizedHMMInference,
    test_data: &[pantherswap_edge::database::types::MarketTick],
) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
    use pantherswap_edge::ai::hmm_regime::MarketObservation;
    use ndarray::Array1;

    // Train detector with some data
    for tick in &test_data[..100.min(test_data.len())] {
        detector.update_with_tick(tick)?;
    }
    detector.train()?;

    let mut latencies = Vec::new();

    // Benchmark inference latency
    for tick in test_data {
        let observation = MarketObservation {
            timestamp: tick.timestamp,
            features: Array1::from(vec![tick.price, tick.volume as f64, tick.ask - tick.bid]),
            volume: tick.volume as f64,
            volatility: 0.02,
            trend: 0.0,
            bid_ask_spread: tick.ask - tick.bid,
            transition_probability: 0.1,
        };

        let start_time = Instant::now();
        let _ = inference.fast_regime_detection(detector, &observation)?;
        let latency_ms = start_time.elapsed().as_nanos() as f64 / 1_000_000.0;

        latencies.push(latency_ms);
    }

    Ok(latencies)
}
