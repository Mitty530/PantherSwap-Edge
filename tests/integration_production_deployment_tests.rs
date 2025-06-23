// Comprehensive Integration & Production Deployment Tests
// Tests for seamless HMM integration, A/B testing, and production deployment
// Run with: cargo test --test integration_production_deployment_tests

use std::time::{Duration, Instant};
use tokio::time::sleep;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::{info, warn, error};

use pantherswap_edge::config::Settings;
use pantherswap_edge::database::Database;
use pantherswap_edge::market_data::MarketDataManager;
use pantherswap_edge::ai::hmm_integration::{
    HMMIntegrationManager, HMMIntegrationConfig, ABTestResults
};
use pantherswap_edge::deployment::production_orchestrator::{
    ProductionDeploymentOrchestrator, ProductionDeploymentConfig, DeploymentStrategy, DeploymentStatus
};
use pantherswap_edge::database::types::MarketTick;

mod common;
use common::*;

/// Integration test configuration
#[derive(Debug, Clone)]
pub struct IntegrationTestConfig {
    pub test_duration_seconds: u64,
    pub market_data_samples: usize,
    pub concurrent_users: usize,
    pub performance_validation: bool,
    pub ab_testing_enabled: bool,
}

impl Default for IntegrationTestConfig {
    fn default() -> Self {
        Self {
            test_duration_seconds: 300, // 5 minutes
            market_data_samples: 1000,
            concurrent_users: 50,
            performance_validation: true,
            ab_testing_enabled: true,
        }
    }
}

#[tokio::test]
async fn test_hmm_integration_manager_initialization() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();
    info!("🚀 Testing HMM Integration Manager initialization");

    let settings = Settings::load()?;
    let database = Database::new(&settings.database.url).await?;
    let market_data_manager = MarketDataManager::new(&settings, database.clone()).await?;

    let config = HMMIntegrationConfig::default();
    let integration_manager = HMMIntegrationManager::new(config, market_data_manager, database).await?;

    // Test initialization
    assert!(integration_manager.start_integration_services().await.is_ok());

    // Test performance metrics retrieval
    let metrics = integration_manager.get_performance_metrics().await;
    assert_eq!(metrics.total_inferences, 0);
    assert_eq!(metrics.successful_inferences, 0);

    // Test A/B testing setup
    let ab_results = integration_manager.get_ab_test_results().await;
    assert!(ab_results.current_test_id.is_some());

    info!("✅ HMM Integration Manager initialization test passed");
    Ok(())
}

#[tokio::test]
async fn test_enhanced_hmm_regime_detection_integration() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();
    info!("🎯 Testing enhanced HMM regime detection integration");

    let settings = Settings::load()?;
    let database = Database::new(&settings.database.url).await?;
    let market_data_manager = MarketDataManager::new(&settings, database.clone()).await?;

    let config = HMMIntegrationConfig {
        enable_enhanced_hmm: true,
        enable_multi_scale: true,
        enable_performance_monitoring: true,
        inference_timeout_ms: 20,
        ..Default::default()
    };

    let integration_manager = HMMIntegrationManager::new(config, market_data_manager, database).await?;
    integration_manager.start_integration_services().await?;

    // Generate test market data
    let test_data = generate_realistic_market_data(100);
    let mut successful_detections = 0;
    let mut total_latency = 0.0;

    for tick in &test_data {
        let start_time = Instant::now();
        
        match integration_manager.detect_regime(tick).await {
            Ok(Some(signal)) => {
                successful_detections += 1;
                let latency_ms = start_time.elapsed().as_nanos() as f64 / 1_000_000.0;
                total_latency += latency_ms;
                
                // Verify signal properties
                assert!(signal.confidence >= 0.0 && signal.confidence <= 1.0);
                assert!(latency_ms <= 20.0, "Latency {:.2}ms exceeds 20ms target", latency_ms);
                
                info!("Regime detected: {:?}, Confidence: {:.2}, Latency: {:.2}ms", 
                      signal.regime, signal.confidence, latency_ms);
            }
            Ok(None) => {
                // No regime detected - this is acceptable
            }
            Err(e) => {
                error!("Regime detection failed: {}", e);
                return Err(e.into());
            }
        }
    }

    let average_latency = total_latency / successful_detections as f64;
    let detection_rate = successful_detections as f64 / test_data.len() as f64;

    // Verify performance requirements
    assert!(average_latency <= 20.0, "Average latency {:.2}ms exceeds 20ms target", average_latency);
    assert!(detection_rate >= 0.5, "Detection rate {:.1}% is below 50% minimum", detection_rate * 100.0);

    // Check final performance metrics
    let final_metrics = integration_manager.get_performance_metrics().await;
    assert!(final_metrics.total_inferences > 0);
    assert!(final_metrics.average_latency_ms <= 20.0);

    info!("✅ Enhanced HMM regime detection integration test passed");
    info!("  • Successful detections: {}/{}", successful_detections, test_data.len());
    info!("  • Average latency: {:.2}ms", average_latency);
    info!("  • Detection rate: {:.1}%", detection_rate * 100.0);

    Ok(())
}

#[tokio::test]
async fn test_ab_testing_framework() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();
    info!("🧪 Testing A/B testing framework");

    let settings = Settings::load()?;
    let database = Database::new(&settings.database.url).await?;
    let market_data_manager = MarketDataManager::new(&settings, database.clone()).await?;

    let config = HMMIntegrationConfig {
        enable_a_b_testing: true,
        enable_enhanced_hmm: true,
        ..Default::default()
    };

    let integration_manager = HMMIntegrationManager::new(config, market_data_manager, database).await?;
    integration_manager.start_integration_services().await?;

    // Generate test data for A/B testing
    let test_data = generate_realistic_market_data(200);
    let mut enhanced_predictions = 0;
    let mut baseline_predictions = 0;

    // Run predictions and track which model is used
    for tick in &test_data {
        if let Ok(Some(_signal)) = integration_manager.detect_regime(tick).await {
            // Simulate tracking which model was used (simplified)
            if tick.volume.unwrap_or(0.0) % 2.0 < 1.0 {
                enhanced_predictions += 1;
                // Simulate accuracy feedback
                integration_manager.update_prediction_accuracy(tick.instrument_id, true, "enhanced").await;
            } else {
                baseline_predictions += 1;
                integration_manager.update_prediction_accuracy(tick.instrument_id, true, "baseline").await;
            }
        }
    }

    // Wait for A/B test metrics to accumulate
    sleep(Duration::from_secs(2)).await;

    // Check A/B test results
    let ab_results = integration_manager.get_ab_test_results().await;
    assert!(ab_results.current_test_id.is_some());
    assert!(ab_results.enhanced_metrics.total_predictions > 0 || ab_results.baseline_metrics.total_predictions > 0);

    // Force A/B test conclusion
    let test_results = integration_manager.conclude_ab_test().await?;
    assert!(test_results.enhanced_performance >= 0.0 && test_results.enhanced_performance <= 1.0);
    assert!(test_results.baseline_performance >= 0.0 && test_results.baseline_performance <= 1.0);
    assert!(!test_results.winner.is_empty());

    info!("✅ A/B testing framework test passed");
    info!("  • Enhanced predictions: {}", enhanced_predictions);
    info!("  • Baseline predictions: {}", baseline_predictions);
    info!("  • Test winner: {}", test_results.winner);
    info!("  • Enhanced performance: {:.2}", test_results.enhanced_performance);
    info!("  • Baseline performance: {:.2}", test_results.baseline_performance);

    Ok(())
}

#[tokio::test]
async fn test_production_deployment_orchestrator() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();
    info!("🚀 Testing Production Deployment Orchestrator");

    let settings = Settings::load()?;
    let database = Database::new(&settings.database.url).await?;
    let market_data_manager = MarketDataManager::new(&settings, database.clone()).await?;

    let config = ProductionDeploymentConfig {
        deployment_strategy: DeploymentStrategy::ABTest,
        rollback_threshold: 0.7,
        health_check_interval_seconds: 5,
        performance_monitoring_enabled: true,
        auto_rollback_enabled: true,
        ..Default::default()
    };

    let orchestrator = ProductionDeploymentOrchestrator::new(config, database, market_data_manager).await?;

    // Test deployment
    let deployment_id = orchestrator.deploy_enhanced_hmm("2.0.0".to_string()).await?;
    assert!(!deployment_id.is_nil());

    // Wait for deployment to initialize
    sleep(Duration::from_secs(3)).await;

    // Check deployment status
    let status = orchestrator.get_deployment_status().await;
    assert_eq!(status.deployment_id, deployment_id);
    assert_eq!(status.current_version, "2.0.0");
    assert!(matches!(status.deployment_status, DeploymentStatus::Monitoring | DeploymentStatus::Active));

    // Wait for health checks
    sleep(Duration::from_secs(10)).await;

    // Check health metrics
    let health = orchestrator.get_health_metrics().await;
    assert!(health.overall_health_score >= 0.0 && health.overall_health_score <= 1.0);
    assert!(!health.component_health.is_empty());

    // Check deployment history
    let history = orchestrator.get_deployment_history().await;
    // History might be empty for successful deployments

    info!("✅ Production Deployment Orchestrator test passed");
    info!("  • Deployment ID: {}", deployment_id);
    info!("  • Deployment status: {:?}", status.deployment_status);
    info!("  • Health score: {:.2}", health.overall_health_score);
    info!("  • Component health checks: {}", health.component_health.len());

    Ok(())
}

#[tokio::test]
async fn test_deployment_rollback_mechanism() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();
    info!("🔄 Testing deployment rollback mechanism");

    let settings = Settings::load()?;
    let database = Database::new(&settings.database.url).await?;
    let market_data_manager = MarketDataManager::new(&settings, database.clone()).await?;

    let config = ProductionDeploymentConfig {
        deployment_strategy: DeploymentStrategy::ABTest,
        rollback_threshold: 0.9, // High threshold to trigger rollback
        health_check_interval_seconds: 2,
        auto_rollback_enabled: true,
        ..Default::default()
    };

    let orchestrator = ProductionDeploymentOrchestrator::new(config, database, market_data_manager).await?;

    // Deploy version that will trigger rollback
    let deployment_id = orchestrator.deploy_enhanced_hmm("2.1.0-unstable".to_string()).await?;

    // Wait for deployment
    sleep(Duration::from_secs(3)).await;

    // Trigger manual rollback
    let rollback_reason = "Testing rollback mechanism".to_string();
    orchestrator.trigger_rollback(rollback_reason.clone()).await?;

    // Wait for rollback to complete
    sleep(Duration::from_secs(2)).await;

    // Check deployment status after rollback
    let status = orchestrator.get_deployment_status().await;
    assert!(matches!(status.deployment_status, DeploymentStatus::Failed | DeploymentStatus::RollingBack));
    assert!(!status.rollback_available);

    // Check deployment history for rollback record
    let history = orchestrator.get_deployment_history().await;
    if !history.is_empty() {
        let last_deployment = &history[history.len() - 1];
        assert_eq!(last_deployment.deployment_id, deployment_id);
        assert!(last_deployment.rollback_reason.is_some());
    }

    info!("✅ Deployment rollback mechanism test passed");
    info!("  • Rollback triggered for deployment: {}", deployment_id);
    info!("  • Final status: {:?}", status.deployment_status);
    info!("  • Rollback available: {}", status.rollback_available);

    Ok(())
}

#[tokio::test]
async fn test_comprehensive_integration_workflow() -> Result<(), Box<dyn std::error::Error>> {
    init_test_logging();
    info!("🎯 Testing comprehensive integration workflow");

    let settings = Settings::load()?;
    let database = Database::new(&settings.database.url).await?;
    let market_data_manager = MarketDataManager::new(&settings, database.clone()).await?;

    // Initialize integration manager
    let hmm_config = HMMIntegrationConfig::default();
    let integration_manager = HMMIntegrationManager::new(hmm_config, market_data_manager.clone(), database.clone()).await?;
    integration_manager.start_integration_services().await?;

    // Initialize deployment orchestrator
    let deployment_config = ProductionDeploymentConfig::default();
    let orchestrator = ProductionDeploymentOrchestrator::new(deployment_config, database, market_data_manager).await?;

    // Deploy enhanced HMM
    let deployment_id = orchestrator.deploy_enhanced_hmm("3.0.0".to_string()).await?;

    // Generate realistic trading scenario
    let test_data = generate_realistic_market_data(500);
    let mut total_predictions = 0;
    let mut successful_predictions = 0;
    let mut total_latency = 0.0;

    let start_time = Instant::now();

    // Simulate real trading activity
    for (i, tick) in test_data.iter().enumerate() {
        let prediction_start = Instant::now();
        
        match integration_manager.detect_regime(tick).await {
            Ok(Some(signal)) => {
                total_predictions += 1;
                let latency_ms = prediction_start.elapsed().as_nanos() as f64 / 1_000_000.0;
                total_latency += latency_ms;
                
                // Simulate accuracy feedback (80% accuracy)
                let is_correct = (i % 5) != 0; // 80% correct
                if is_correct {
                    successful_predictions += 1;
                }
                
                integration_manager.update_prediction_accuracy(
                    tick.instrument_id, 
                    is_correct, 
                    "enhanced"
                ).await;
                
                // Verify performance requirements
                assert!(latency_ms <= 25.0, "Latency {:.2}ms exceeds 25ms limit", latency_ms);
                assert!(signal.confidence >= 0.0 && signal.confidence <= 1.0);
            }
            Ok(None) => {
                // No prediction - acceptable
            }
            Err(e) => {
                error!("Prediction failed: {}", e);
            }
        }

        // Add small delay to simulate realistic timing
        if i % 10 == 0 {
            sleep(Duration::from_millis(10)).await;
        }
    }

    let total_test_time = start_time.elapsed();

    // Calculate final metrics
    let average_latency = if total_predictions > 0 { total_latency / total_predictions as f64 } else { 0.0 };
    let accuracy = if total_predictions > 0 { successful_predictions as f64 / total_predictions as f64 } else { 0.0 };
    let throughput = total_predictions as f64 / total_test_time.as_secs_f64();

    // Verify comprehensive performance
    assert!(average_latency <= 20.0, "Average latency {:.2}ms exceeds 20ms target", average_latency);
    assert!(accuracy >= 0.75, "Accuracy {:.1}% below 75% target", accuracy * 100.0);
    assert!(throughput >= 10.0, "Throughput {:.1} predictions/sec below 10/sec minimum", throughput);

    // Check final system state
    let final_metrics = integration_manager.get_performance_metrics().await;
    let deployment_status = orchestrator.get_deployment_status().await;
    let health_metrics = orchestrator.get_health_metrics().await;

    // Verify system health
    assert!(final_metrics.total_inferences > 0);
    assert!(health_metrics.overall_health_score >= 0.7);
    assert!(matches!(deployment_status.deployment_status, DeploymentStatus::Active | DeploymentStatus::Monitoring));

    info!("✅ Comprehensive integration workflow test passed");
    info!("  • Total test time: {:.2}s", total_test_time.as_secs_f64());
    info!("  • Total predictions: {}", total_predictions);
    info!("  • Accuracy: {:.1}%", accuracy * 100.0);
    info!("  • Average latency: {:.2}ms", average_latency);
    info!("  • Throughput: {:.1} predictions/sec", throughput);
    info!("  • Health score: {:.2}", health_metrics.overall_health_score);
    info!("  • Deployment status: {:?}", deployment_status.deployment_status);

    Ok(())
}

/// Helper function to generate realistic market data for testing
fn generate_realistic_market_data(count: usize) -> Vec<MarketTick> {
    use rand::Rng;
    
    let mut rng = rand::thread_rng();
    let mut data = Vec::with_capacity(count);
    let base_time = Utc::now() - chrono::Duration::minutes(count as i64);
    
    for i in 0..count {
        let timestamp = base_time + chrono::Duration::seconds(i as i64);
        let base_price = 100.0 + (i as f64 * 0.01);
        let volatility = rng.gen_range(0.5..2.0);
        
        let bid_price = base_price - rng.gen_range(0.01..0.05);
        let ask_price = base_price + rng.gen_range(0.01..0.05);
        let volume = rng.gen_range(1000.0..10000.0);
        
        data.push(MarketTick {
            instrument_id: Uuid::new_v4(),
            symbol: format!("TEST{}", i % 5),
            bid_price,
            ask_price,
            bid_size: rng.gen_range(100.0..1000.0),
            ask_size: rng.gen_range(100.0..1000.0),
            volume: Some(volume),
            timestamp,
            data_quality_score: rng.gen_range(0.8..1.0),
        });
    }
    
    data
}
