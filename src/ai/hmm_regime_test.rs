//! Test module for Multi-Scale HMM Regime Detection

#[cfg(test)]
mod tests {
    use super::super::hmm_regime::*;
    use crate::database::types::{MarketTick, RegimeType};
    use chrono::{DateTime, Utc};
    use uuid::Uuid;

    fn create_test_market_tick(timestamp: DateTime<Utc>, price: f64, volume: f64) -> MarketTick {
        MarketTick {
            timestamp,
            instrument_id: Uuid::new_v4(),
            provider: "test".to_string(),
            bid_price: price - 0.01,
            ask_price: price + 0.01,
            bid_size: 100.0,
            ask_size: 100.0,
            last_price: Some(price),
            volume: Some(volume),
            spread: 0.02,
            data_quality_score: 1.0,
            raw_data: serde_json::Value::Null,
            // Backward compatibility fields
            symbol: Some("TEST".to_string()),
            price: Some(price),
            bid: Some(price - 0.01),
            ask: Some(price + 0.01),
        }
    }

    #[test]
    fn test_time_scale_duration() {
        assert_eq!(TimeScale::OneMinute.duration_seconds(), 60);
        assert_eq!(TimeScale::FiveMinutes.duration_seconds(), 300);
        assert_eq!(TimeScale::FifteenMinutes.duration_seconds(), 900);
        assert_eq!(TimeScale::OneHour.duration_seconds(), 3600);
    }

    #[test]
    fn test_time_scale_observation_window() {
        assert_eq!(TimeScale::OneMinute.observation_window(), 60);
        assert_eq!(TimeScale::FiveMinutes.observation_window(), 48);
        assert_eq!(TimeScale::FifteenMinutes.observation_window(), 32);
        assert_eq!(TimeScale::OneHour.observation_window(), 24);
    }

    #[test]
    fn test_multi_scale_hmm_config_default() {
        let config = MultiScaleHMMConfig::default();
        
        // Check that all time scales are configured
        assert_eq!(config.scale_configs.len(), 4);
        assert_eq!(config.scale_weights.len(), 4);
        
        // Check consensus threshold
        assert_eq!(config.consensus_threshold, 0.6);
        
        // Check that weights sum to 1.0 (approximately)
        let total_weight: f64 = config.scale_weights.values().sum();
        assert!((total_weight - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_multi_scale_hmm_detector_creation() {
        let config = MultiScaleHMMConfig::default();
        let detector = MultiScaleHMMRegimeDetector::new(config);
        
        // Check that detector is created with correct number of scale detectors
        assert_eq!(detector.scale_detectors.len(), 4);
        assert!(!detector.is_initialized);
        assert!(!detector.is_ready());
    }

    #[test]
    fn test_factory_functions() {
        // Test default factory
        let detector1 = create_multi_scale_hmm_detector();
        assert_eq!(detector1.scale_detectors.len(), 4);
        
        // Test HF factory
        let detector2 = create_hf_multi_scale_hmm_detector();
        assert_eq!(detector2.scale_detectors.len(), 4);
        assert_eq!(detector2.config.consensus_threshold, 0.55);
        
        // Test accuracy factory
        let detector3 = create_accuracy_multi_scale_hmm_detector();
        assert_eq!(detector3.scale_detectors.len(), 4);
        assert_eq!(detector3.config.consensus_threshold, 0.75);
        
        // Test balanced factory
        let detector4 = create_balanced_multi_scale_hmm_detector();
        assert_eq!(detector4.scale_detectors.len(), 4);
        assert_eq!(detector4.config.consensus_threshold, 0.65);
    }

    #[test]
    fn test_multi_scale_detector_update() {
        let mut detector = create_multi_scale_hmm_detector();
        let now = Utc::now();
        
        // Add some test ticks
        for i in 0..20 {
            let tick = create_test_market_tick(
                now + chrono::Duration::seconds(i * 60), // 1 minute intervals
                100.0 + (i as f64) * 0.1, // Slightly increasing price
                1000.0 + (i as f64) * 10.0, // Increasing volume
            );
            
            let result = detector.update_with_tick(&tick);
            assert!(result.is_ok());
        }
        
        // Check that detector becomes initialized
        assert!(detector.is_initialized);
    }

    #[test]
    fn test_observation_to_tick_conversion() {
        use ndarray::Array1;
        
        let obs = MarketObservation {
            timestamp: Utc::now(),
            features: Array1::from_vec(vec![0.01, 0.005, 0.5]), // volatility, trend, volume
            volatility: 0.01,
            trend: 0.005,
            volume: 0.5,
            momentum: 0.002,
            bid_ask_spread: 0.02,
            price_skewness: 0.1,
            price_kurtosis: 0.2,
            autocorrelation: 0.3,
            regime_persistence: 0.8,
            transition_probability: 0.1,
            order_flow_imbalance: 0.05,
            effective_spread: 0.015,
            price_impact: 0.001,
            market_depth_ratio: 1.2,
            garch_volatility: 0.012,
            volatility_persistence: 0.7,
            volatility_clustering_score: 0.6,
            hurst_exponent: 0.55,
            fractal_dimension: 1.45,
            regime_strength: 0.75,
            regime_transition_signal: 0.15,
        };
        
        let tick = MultiScaleHMMRegimeDetector::observation_to_tick_static(&obs);
        
        // Check that conversion produces valid tick
        assert!(tick.bid_price > 0.0);
        assert!(tick.ask_price > tick.bid_price);
        assert!(tick.spread > 0.0);
        assert_eq!(tick.provider, "aggregated");
    }

    #[test]
    fn test_scale_weights_configuration() {
        // Test HF configuration weights
        let hf_detector = create_hf_multi_scale_hmm_detector();
        let hf_weights = &hf_detector.config.scale_weights;
        
        // HF should favor shorter timeframes
        assert!(hf_weights[&TimeScale::OneMinute] > hf_weights[&TimeScale::OneHour]);
        assert_eq!(hf_weights[&TimeScale::OneMinute], 0.5);
        
        // Test accuracy configuration weights
        let acc_detector = create_accuracy_multi_scale_hmm_detector();
        let acc_weights = &acc_detector.config.scale_weights;
        
        // Accuracy should be more balanced
        assert!(acc_weights[&TimeScale::FiveMinutes] == acc_weights[&TimeScale::FifteenMinutes]);
        assert_eq!(acc_weights[&TimeScale::FiveMinutes], 0.3);
    }

    #[test]
    fn test_consensus_threshold_variations() {
        let hf_detector = create_hf_multi_scale_hmm_detector();
        let acc_detector = create_accuracy_multi_scale_hmm_detector();
        let balanced_detector = create_balanced_multi_scale_hmm_detector();
        
        // HF should have lower threshold for faster decisions
        assert!(hf_detector.config.consensus_threshold < balanced_detector.config.consensus_threshold);
        
        // Accuracy should have higher threshold for more confident decisions
        assert!(acc_detector.config.consensus_threshold > balanced_detector.config.consensus_threshold);
        
        // Check specific values
        assert_eq!(hf_detector.config.consensus_threshold, 0.55);
        assert_eq!(acc_detector.config.consensus_threshold, 0.75);
        assert_eq!(balanced_detector.config.consensus_threshold, 0.65);
    }

    #[test]
    fn test_hierarchical_propagation_setting() {
        let config = MultiScaleHMMConfig::default();
        assert!(config.enable_hierarchical_propagation);
        
        let acc_detector = create_accuracy_multi_scale_hmm_detector();
        assert!(acc_detector.config.enable_hierarchical_propagation);
        
        let balanced_detector = create_balanced_multi_scale_hmm_detector();
        assert!(balanced_detector.config.enable_hierarchical_propagation);
    }
}
