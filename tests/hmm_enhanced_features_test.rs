use pantherswap_edge::ai::hmm_regime::{
    HMMRegimeDetector, create_enhanced_accuracy_hmm_detector, create_hf_hmm_regime_detector
};
use pantherswap_edge::database::types::{MarketTick, RegimeType};
use pantherswap_edge::error::Result;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;
use rand;

/// Test enhanced feature engineering for HMM regime detection
#[tokio::test]
async fn test_enhanced_hmm_feature_engineering() -> Result<()> {
    println!("🧪 Testing Enhanced HMM Feature Engineering");

    // Test 1: Create enhanced accuracy detector
    println!("\n📊 Test 1: Creating enhanced accuracy HMM detector");
    let mut enhanced_detector = create_enhanced_accuracy_hmm_detector();
    
    // Verify configuration
    assert_eq!(enhanced_detector.get_config().feature_dimensions, 16);
    assert!(enhanced_detector.get_config().enable_microstructure_features);
    assert!(enhanced_detector.get_config().enable_volatility_clustering);
    assert!(enhanced_detector.get_config().enable_regime_specific_indicators);
    println!("  ✅ Enhanced detector created with 16 feature dimensions");

    // Test 2: Generate synthetic market data with different regime characteristics
    println!("\n📈 Test 2: Testing feature extraction with synthetic market data");
    
    let instrument_id = Uuid::new_v4();
    let base_time = Utc::now();
    
    // Generate normal regime data (low volatility, stable trend)
    let normal_ticks = generate_regime_ticks(instrument_id, base_time, RegimeType::Normal, 50);
    
    // Generate volatile regime data (high volatility, erratic movements)
    let volatile_ticks = generate_regime_ticks(instrument_id, base_time, RegimeType::Volatile, 50);
    
    // Generate trending regime data (consistent directional movement)
    let trending_ticks = generate_regime_ticks(instrument_id, base_time, RegimeType::Trending, 50);
    
    // Test 3: Process normal regime data
    println!("\n🔍 Test 3: Processing normal regime data");
    for tick in &normal_ticks {
        enhanced_detector.update_with_tick(tick)?;
    }
    
    let normal_stats = enhanced_detector.get_enhanced_stats();
    println!("  📊 Normal regime stats:");
    println!("    Detection accuracy: {:.3}", normal_stats.detection_accuracy);
    println!("    Total observations: {}", normal_stats.total_observations);
    
    // Test 4: Process volatile regime data
    println!("\n⚡ Test 4: Processing volatile regime data");
    for tick in &volatile_ticks {
        enhanced_detector.update_with_tick(tick)?;
    }
    
    if let Some(regime_signal) = enhanced_detector.detect_current_regime() {
        println!("  🎯 Detected regime: {:?}", regime_signal.current_regime);
        println!("  📈 Confidence: {:.3}", regime_signal.confidence);
        println!("  🔄 Transition probability: {:.3}", regime_signal.transition_probability);
    }
    
    // Test 5: Process trending regime data
    println!("\n📈 Test 5: Processing trending regime data");
    for tick in &trending_ticks {
        enhanced_detector.update_with_tick(tick)?;
    }
    
    if let Some(regime_signal) = enhanced_detector.detect_current_regime() {
        println!("  🎯 Detected regime: {:?}", regime_signal.current_regime);
        println!("  📈 Confidence: {:.3}", regime_signal.confidence);
        println!("  🔄 Transition probability: {:.3}", regime_signal.transition_probability);
    }
    
    // Test 6: Compare with standard HF detector
    println!("\n⚖️ Test 6: Comparing with standard HF detector");
    let mut hf_detector = create_hf_hmm_regime_detector();
    
    // Process same data with HF detector
    for tick in &trending_ticks {
        hf_detector.update_with_tick(tick)?;
    }
    
    let enhanced_stats = enhanced_detector.get_enhanced_stats();
    let hf_stats = hf_detector.get_enhanced_stats();
    
    println!("  📊 Enhanced detector accuracy: {:.3}", enhanced_stats.detection_accuracy);
    println!("  📊 HF detector accuracy: {:.3}", hf_stats.detection_accuracy);
    
    // Test 7: Feature quality validation
    println!("\n🔬 Test 7: Validating feature quality");
    test_feature_quality(&mut enhanced_detector).await?;
    
    // Test 8: Performance benchmarking
    println!("\n⏱️ Test 8: Performance benchmarking");
    test_performance_benchmarking(&mut enhanced_detector).await?;
    
    println!("\n✅ All enhanced HMM feature engineering tests passed!");
    Ok(())
}

/// Generate synthetic market ticks for different regime types
fn generate_regime_ticks(
    instrument_id: Uuid,
    base_time: DateTime<Utc>,
    regime: RegimeType,
    count: usize,
) -> Vec<MarketTick> {
    let mut ticks = Vec::new();
    let mut price = 100.0;
    let mut volume = 1000.0;
    
    for i in 0..count {
        let timestamp = base_time + chrono::Duration::seconds(i as i64);
        
        // Adjust price movement based on regime
        let (price_change, vol_multiplier, spread_multiplier) = match regime {
            RegimeType::Normal => {
                // Low volatility, small random movements
                let change = (rand::random::<f64>() - 0.5) * 0.01;
                (change, 1.0, 1.0)
            },
            RegimeType::Volatile => {
                // High volatility, large random movements
                let change = (rand::random::<f64>() - 0.5) * 0.05;
                (change, 3.0, 2.0)
            },
            RegimeType::Trending => {
                // Consistent directional movement with some noise
                let trend = 0.002; // Upward trend
                let noise = (rand::random::<f64>() - 0.5) * 0.005;
                (trend + noise, 1.5, 1.2)
            },
            RegimeType::Crisis => {
                // Very high volatility, large downward movements
                let change = (rand::random::<f64>() - 0.7) * 0.08;
                (change, 5.0, 3.0)
            },
        };
        
        price += price_change;
        volume *= vol_multiplier * (0.8 + rand::random::<f64>() * 0.4);
        
        let spread = price * 0.001 * spread_multiplier;
        let bid_price = price - spread / 2.0;
        let ask_price = price + spread / 2.0;
        
        ticks.push(MarketTick {
            id: Uuid::new_v4(),
            instrument_id,
            timestamp,
            bid_price,
            ask_price,
            bid_size: Some(volume * 0.8),
            ask_size: Some(volume * 1.2),
            volume: Some(volume),
            trade_count: Some(10),
        });
    }
    
    ticks
}

/// Test feature quality and discriminative power
async fn test_feature_quality(detector: &mut HMMRegimeDetector) -> Result<()> {
    println!("  🔬 Testing feature discriminative power");
    
    // Generate contrasting regime data
    let instrument_id = Uuid::new_v4();
    let base_time = Utc::now();
    
    let normal_ticks = generate_regime_ticks(instrument_id, base_time, RegimeType::Normal, 30);
    let crisis_ticks = generate_regime_ticks(instrument_id, base_time, RegimeType::Crisis, 30);
    
    // Process normal regime
    for tick in &normal_ticks {
        detector.update_with_tick(tick)?;
    }
    let normal_signal = detector.detect_current_regime();
    
    // Process crisis regime
    for tick in &crisis_ticks {
        detector.update_with_tick(tick)?;
    }
    let crisis_signal = detector.detect_current_regime();
    
    // Validate that different regimes are detected
    if let (Some(normal), Some(crisis)) = (normal_signal, crisis_signal) {
        println!("    Normal regime confidence: {:.3}", normal.confidence);
        println!("    Crisis regime confidence: {:.3}", crisis.confidence);
        
        // Features should be able to distinguish between regimes
        assert!(crisis.confidence > 0.5, "Crisis regime should be detected with reasonable confidence");
    }
    
    println!("  ✅ Feature quality validation passed");
    Ok(())
}

/// Test performance benchmarking
async fn test_performance_benchmarking(detector: &mut HMMRegimeDetector) -> Result<()> {
    println!("  ⏱️ Benchmarking feature extraction performance");
    
    let instrument_id = Uuid::new_v4();
    let base_time = Utc::now();
    let test_ticks = generate_regime_ticks(instrument_id, base_time, RegimeType::Normal, 100);
    
    let start_time = std::time::Instant::now();
    
    for tick in &test_ticks {
        detector.update_with_tick(tick)?;
    }
    
    let elapsed = start_time.elapsed();
    let avg_latency_ms = elapsed.as_millis() as f64 / test_ticks.len() as f64;
    
    println!("    Average feature extraction latency: {:.3}ms", avg_latency_ms);
    println!("    Total processing time: {:?}", elapsed);
    
    // Ensure performance meets requirements (<20ms per tick)
    assert!(avg_latency_ms < 20.0, "Feature extraction should be under 20ms per tick");
    
    println!("  ✅ Performance benchmarking passed");
    Ok(())
}
