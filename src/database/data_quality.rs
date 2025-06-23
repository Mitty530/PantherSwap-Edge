// Advanced data quality assessment for trading platform
// Provides statistical analysis, anomaly detection, and data quality scoring

use crate::utils::Result;
use crate::database::types::*;
use chrono::{DateTime, Utc, Duration};
use std::collections::{HashMap, VecDeque};
use serde::{Serialize, Deserialize};
use tracing::{info, warn};

/// Data quality assessment engine
pub struct DataQualityAssessor {
    config: QualityConfig,
    historical_data: HashMap<String, VecDeque<QualityMetric>>,
    anomaly_detector: AnomalyDetector,
}

#[derive(Debug, Clone)]
pub struct QualityConfig {
    pub window_size: usize,
    pub anomaly_threshold: f64,
    pub min_samples_for_analysis: usize,
    pub price_volatility_threshold: f64,
    pub volume_spike_threshold: f64,
    pub spread_anomaly_threshold: f64,
}

impl Default for QualityConfig {
    fn default() -> Self {
        Self {
            window_size: 100,
            anomaly_threshold: 2.0, // 2 standard deviations
            min_samples_for_analysis: 10,
            price_volatility_threshold: 0.05, // 5%
            volume_spike_threshold: 3.0, // 3x average
            spread_anomaly_threshold: 2.0, // 2x normal spread
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetric {
    pub timestamp: DateTime<Utc>,
    pub price_quality: f64,
    pub volume_quality: f64,
    pub spread_quality: f64,
    pub timestamp_quality: f64,
    pub overall_quality: f64,
    pub anomaly_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityReport {
    pub instrument_id: String,
    pub provider: String,
    pub assessment_time: DateTime<Utc>,
    pub overall_score: f64,
    pub component_scores: ComponentScores,
    pub anomalies_detected: Vec<AnomalyAlert>,
    pub recommendations: Vec<String>,
    pub data_completeness: f64,
    pub temporal_consistency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentScores {
    pub price_accuracy: f64,
    pub volume_consistency: f64,
    pub spread_reasonableness: f64,
    pub timestamp_freshness: f64,
    pub provider_reliability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyAlert {
    pub alert_type: String,
    pub severity: String,
    pub description: String,
    pub detected_at: DateTime<Utc>,
    pub confidence: f64,
    pub affected_fields: Vec<String>,
}

/// Anomaly detection using statistical methods
pub struct AnomalyDetector {
    price_stats: StatisticalTracker,
    volume_stats: StatisticalTracker,
    spread_stats: StatisticalTracker,
}

#[derive(Debug, Clone)]
struct StatisticalTracker {
    values: VecDeque<f64>,
    mean: f64,
    variance: f64,
    max_size: usize,
}

impl StatisticalTracker {
    fn new(max_size: usize) -> Self {
        Self {
            values: VecDeque::new(),
            mean: 0.0,
            variance: 0.0,
            max_size,
        }
    }

    fn add_value(&mut self, value: f64) {
        self.values.push_back(value);
        if self.values.len() > self.max_size {
            self.values.pop_front();
        }
        self.update_statistics();
    }

    fn update_statistics(&mut self) {
        if self.values.is_empty() {
            return;
        }

        self.mean = self.values.iter().sum::<f64>() / self.values.len() as f64;
        
        if self.values.len() > 1 {
            let variance_sum: f64 = self.values
                .iter()
                .map(|x| (x - self.mean).powi(2))
                .sum();
            self.variance = variance_sum / (self.values.len() - 1) as f64;
        }
    }

    fn z_score(&self, value: f64) -> f64 {
        if self.variance == 0.0 {
            return 0.0;
        }
        (value - self.mean) / self.variance.sqrt()
    }

    fn is_anomaly(&self, value: f64, threshold: f64) -> bool {
        self.z_score(value).abs() > threshold
    }
}

impl AnomalyDetector {
    fn new(window_size: usize) -> Self {
        Self {
            price_stats: StatisticalTracker::new(window_size),
            volume_stats: StatisticalTracker::new(window_size),
            spread_stats: StatisticalTracker::new(window_size),
        }
    }

    fn detect_anomalies(&mut self, tick: &MarketTick, threshold: f64) -> Vec<AnomalyAlert> {
        let mut anomalies = Vec::new();

        // Check price anomalies
        let mid_price = (tick.bid_price + tick.ask_price) / 2.0;
        if self.price_stats.is_anomaly(mid_price, threshold) {
            anomalies.push(AnomalyAlert {
                alert_type: "price_anomaly".to_string(),
                severity: "medium".to_string(),
                description: format!("Unusual price detected: {:.4}", mid_price),
                detected_at: Utc::now(),
                confidence: self.price_stats.z_score(mid_price).abs() / threshold,
                affected_fields: vec!["bid_price".to_string(), "ask_price".to_string()],
            });
        }

        // Check volume anomalies
        if let Some(volume) = tick.volume {
            if volume > 0.0 && self.volume_stats.is_anomaly(volume, threshold) {
                anomalies.push(AnomalyAlert {
                    alert_type: "volume_anomaly".to_string(),
                    severity: "low".to_string(),
                    description: format!("Unusual volume detected: {:.2}", volume),
                    detected_at: Utc::now(),
                    confidence: self.volume_stats.z_score(volume).abs() / threshold,
                    affected_fields: vec!["volume".to_string()],
                });
            }
        }

        // Check spread anomalies
        if self.spread_stats.is_anomaly(tick.spread, threshold) {
            anomalies.push(AnomalyAlert {
                alert_type: "spread_anomaly".to_string(),
                severity: "high".to_string(),
                description: format!("Unusual spread detected: {:.4}", tick.spread),
                detected_at: Utc::now(),
                confidence: self.spread_stats.z_score(tick.spread).abs() / threshold,
                affected_fields: vec!["spread".to_string()],
            });
        }

        // Update statistics with new values
        self.price_stats.add_value(mid_price);
        if let Some(volume) = tick.volume {
            if volume > 0.0 {
                self.volume_stats.add_value(volume);
            }
        }
        self.spread_stats.add_value(tick.spread);

        anomalies
    }
}

impl DataQualityAssessor {
    pub fn new(config: QualityConfig) -> Self {
        Self {
            config: config.clone(),
            historical_data: HashMap::new(),
            anomaly_detector: AnomalyDetector::new(config.window_size),
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(QualityConfig::default())
    }

    /// Assess the quality of market tick data
    pub fn assess_market_tick_quality(&mut self, tick: &MarketTick) -> Result<QualityReport> {
        let key = format!("{}_{}", tick.instrument_id, tick.provider);

        // Calculate component scores
        let price_quality = self.assess_price_quality(tick);
        let volume_quality = self.assess_volume_quality(tick);
        let spread_quality = self.assess_spread_quality(tick);
        let timestamp_quality = self.assess_timestamp_quality(tick);
        let provider_reliability = self.assess_provider_reliability(&tick.provider);

        // Calculate overall quality score
        let overall_quality = (price_quality + volume_quality + spread_quality + 
                             timestamp_quality + provider_reliability) / 5.0;

        // Detect anomalies
        let anomalies = self.anomaly_detector.detect_anomalies(tick, self.config.anomaly_threshold);

        // Create quality metric
        let metric = QualityMetric {
            timestamp: tick.timestamp,
            price_quality,
            volume_quality,
            spread_quality,
            timestamp_quality,
            overall_quality,
            anomaly_score: anomalies.len() as f64,
        };

        // Store historical data
        let history = self.historical_data.entry(key.clone()).or_insert_with(VecDeque::new);
        history.push_back(metric.clone());
        if history.len() > self.config.window_size {
            history.pop_front();
        }

        // Generate recommendations
        let recommendations = self.generate_recommendations(tick, &anomalies, overall_quality);

        // Calculate additional metrics
        let data_completeness = self.calculate_data_completeness(tick);
        let temporal_consistency = self.calculate_temporal_consistency(&key);

        Ok(QualityReport {
            instrument_id: tick.instrument_id.to_string(),
            provider: tick.provider.clone(),
            assessment_time: Utc::now(),
            overall_score: overall_quality,
            component_scores: ComponentScores {
                price_accuracy: price_quality,
                volume_consistency: volume_quality,
                spread_reasonableness: spread_quality,
                timestamp_freshness: timestamp_quality,
                provider_reliability,
            },
            anomalies_detected: anomalies,
            recommendations,
            data_completeness,
            temporal_consistency,
        })
    }

    /// Assess trading signal quality
    pub fn assess_trading_signal_quality(&self, signal: &TradingSignal) -> Result<QualityReport> {
        let confidence_quality = signal.confidence_score;
        let risk_quality = 1.0 - signal.risk_score; // Lower risk = higher quality
        let timestamp_quality = self.assess_signal_timestamp_quality(signal);
        let strategy_quality = self.assess_strategy_quality(&signal.strategy_type);
        let price_level_quality = self.assess_price_level_quality(signal);

        let overall_quality = (confidence_quality + risk_quality + timestamp_quality + 
                             strategy_quality + price_level_quality) / 5.0;

        let mut anomalies = Vec::new();
        let mut recommendations = Vec::new();

        // Check for signal quality issues
        if signal.confidence_score < 0.6 {
            anomalies.push(AnomalyAlert {
                alert_type: "low_confidence".to_string(),
                severity: "medium".to_string(),
                description: format!("Low confidence signal: {:.2}", signal.confidence_score),
                detected_at: Utc::now(),
                confidence: 1.0 - signal.confidence_score,
                affected_fields: vec!["confidence_score".to_string()],
            });
            recommendations.push("Consider increasing confidence threshold".to_string());
        }

        if signal.risk_score > 0.8 {
            anomalies.push(AnomalyAlert {
                alert_type: "high_risk".to_string(),
                severity: "high".to_string(),
                description: format!("High risk signal: {:.2}", signal.risk_score),
                detected_at: Utc::now(),
                confidence: signal.risk_score,
                affected_fields: vec!["risk_score".to_string()],
            });
            recommendations.push("Review risk management parameters".to_string());
        }

        Ok(QualityReport {
            instrument_id: signal.instrument_id.to_string(),
            provider: "trading_engine".to_string(),
            assessment_time: Utc::now(),
            overall_score: overall_quality,
            component_scores: ComponentScores {
                price_accuracy: price_level_quality,
                volume_consistency: 1.0, // Not applicable for signals
                spread_reasonableness: 1.0, // Not applicable for signals
                timestamp_freshness: timestamp_quality,
                provider_reliability: strategy_quality,
            },
            anomalies_detected: anomalies,
            recommendations,
            data_completeness: self.calculate_signal_completeness(signal),
            temporal_consistency: 1.0, // Would need historical analysis
        })
    }

    // Private assessment methods

    fn assess_price_quality(&self, tick: &MarketTick) -> f64 {
        let mut score: f64 = 1.0;

        // Check for reasonable price levels
        if tick.bid_price <= 0.0 || tick.ask_price <= 0.0 {
            score -= 0.5;
        }

        // Check bid-ask relationship
        if tick.ask_price <= tick.bid_price {
            score -= 0.3;
        }

        // Check for extreme price movements (if we have historical data)
        // This would be enhanced with actual historical price analysis

        score.max(0.0)
    }

    fn assess_volume_quality(&self, tick: &MarketTick) -> f64 {
        let mut score: f64 = 1.0;

        // Check if volume data is present
        if tick.volume.is_none() {
            score -= 0.2;
        }

        // Check for negative volumes
        if let Some(volume) = tick.volume {
            if volume < 0.0 {
                score -= 0.5;
            }
        }

        // Check bid/ask sizes
        if tick.bid_size < 0.0 || tick.ask_size < 0.0 {
            score -= 0.3;
        }

        score.max(0.0)
    }

    fn assess_spread_quality(&self, tick: &MarketTick) -> f64 {
        let mut score: f64 = 1.0;

        // Calculate spread percentage
        let spread_pct = (tick.spread / tick.bid_price) * 100.0;

        // Penalize extremely wide spreads
        if spread_pct > 5.0 {
            score -= 0.4;
        } else if spread_pct > 2.0 {
            score -= 0.2;
        }

        // Penalize negative spreads
        if tick.spread < 0.0 {
            score -= 0.5;
        }

        score.max(0.0)
    }

    fn assess_timestamp_quality(&self, tick: &MarketTick) -> f64 {
        let now = Utc::now();
        let age = now.signed_duration_since(tick.timestamp);

        // Score based on data freshness
        if age < Duration::seconds(1) {
            1.0
        } else if age < Duration::seconds(10) {
            0.9
        } else if age < Duration::minutes(1) {
            0.7
        } else if age < Duration::minutes(5) {
            0.5
        } else {
            0.2
        }
    }

    fn assess_provider_reliability(&self, provider: &str) -> f64 {
        // This would be enhanced with actual provider reliability metrics
        match provider {
            "alpha_vantage" => 0.9,
            "iex_cloud" => 0.85,
            _ => 0.7,
        }
    }

    fn assess_signal_timestamp_quality(&self, signal: &TradingSignal) -> f64 {
        let now = Utc::now();
        let age = now.signed_duration_since(signal.timestamp);

        if age < Duration::seconds(5) {
            1.0
        } else if age < Duration::seconds(30) {
            0.8
        } else if age < Duration::minutes(5) {
            0.6
        } else {
            0.3
        }
    }

    fn assess_strategy_quality(&self, strategy_type: &str) -> f64 {
        // This would be enhanced with actual strategy performance metrics
        match strategy_type {
            "predictive_market_making" => 0.9,
            "microstructure_momentum" => 0.85,
            "regime_arbitrage" => 0.8,
            "liquidity_harvesting" => 0.75,
            _ => 0.7,
        }
    }

    fn assess_price_level_quality(&self, signal: &TradingSignal) -> f64 {
        let mut score: f64 = 1.0;

        // Check if price levels are reasonable
        if let Some(target) = signal.target_price {
            if target <= 0.0 {
                score -= 0.3;
            }
        }

        if let Some(stop_loss) = signal.stop_loss {
            if stop_loss <= 0.0 {
                score -= 0.3;
            }
        }

        // Check price level relationships
        if signal.signal_type == "BUY" {
            if let (Some(target), Some(stop)) = (signal.target_price, signal.stop_loss) {
                if target <= stop {
                    score -= 0.4;
                }
            }
        }

        score.max(0.0)
    }

    fn calculate_data_completeness(&self, tick: &MarketTick) -> f64 {
        let mut fields_present = 0;
        let total_fields = 7;

        // Required fields
        if tick.bid_price > 0.0 { fields_present += 1; }
        if tick.ask_price > 0.0 { fields_present += 1; }
        if tick.bid_size >= 0.0 { fields_present += 1; }
        if tick.ask_size >= 0.0 { fields_present += 1; }

        // Optional fields
        if tick.last_price.is_some() { fields_present += 1; }
        if tick.volume.is_some() { fields_present += 1; }
        if !tick.raw_data.is_null() { fields_present += 1; }

        fields_present as f64 / total_fields as f64
    }

    fn calculate_signal_completeness(&self, signal: &TradingSignal) -> f64 {
        let mut fields_present = 0;
        let total_fields = 6;

        // Core fields (always present)
        fields_present += 3; // confidence_score, risk_score, position_size

        // Optional fields
        if signal.target_price.is_some() { fields_present += 1; }
        if signal.stop_loss.is_some() { fields_present += 1; }
        if signal.time_horizon.is_some() { fields_present += 1; }

        fields_present as f64 / total_fields as f64
    }

    fn calculate_temporal_consistency(&self, key: &str) -> f64 {
        if let Some(history) = self.historical_data.get(key) {
            if history.len() < 2 {
                return 1.0;
            }

            let mut consistency_score = 0.0;
            let mut count = 0;

            for i in 1..history.len() {
                let current = &history[i];
                let previous = &history[i - 1];
                let time_diff = current.timestamp.signed_duration_since(previous.timestamp);
                let expected_interval = Duration::seconds(1); // Assuming 1-second intervals

                let deviation = (time_diff - expected_interval).num_milliseconds().abs() as f64;
                let max_acceptable_deviation = 5000.0; // 5 seconds

                let interval_score = (max_acceptable_deviation - deviation.min(max_acceptable_deviation)) 
                    / max_acceptable_deviation;
                
                consistency_score += interval_score;
                count += 1;
            }

            if count > 0 {
                consistency_score / count as f64
            } else {
                1.0
            }
        } else {
            1.0
        }
    }

    fn generate_recommendations(&self, tick: &MarketTick, anomalies: &[AnomalyAlert], overall_quality: f64) -> Vec<String> {
        let mut recommendations = Vec::new();

        if overall_quality < 0.6 {
            recommendations.push("Data quality is below acceptable threshold - consider data source review".to_string());
        }

        if tick.data_quality_score < 0.7 {
            recommendations.push("Provider data quality score is low - verify data source".to_string());
        }

        if !anomalies.is_empty() {
            recommendations.push(format!("Detected {} anomalies - review data for accuracy", anomalies.len()));
        }

        let spread_pct = (tick.spread / tick.bid_price) * 100.0;
        if spread_pct > 2.0 {
            recommendations.push("Wide spread detected - verify market conditions".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("Data quality is acceptable".to_string());
        }

        recommendations
    }

    /// Get quality statistics for a specific instrument and provider
    pub fn get_quality_statistics(&self, instrument_id: &str, provider: &str) -> Option<QualityStatistics> {
        let key = format!("{}_{}", instrument_id, provider);
        
        if let Some(history) = self.historical_data.get(&key) {
            if history.is_empty() {
                return None;
            }

            let count = history.len();
            let avg_quality = history.iter().map(|m| m.overall_quality).sum::<f64>() / count as f64;
            let min_quality = history.iter().map(|m| m.overall_quality).fold(f64::INFINITY, f64::min);
            let max_quality = history.iter().map(|m| m.overall_quality).fold(f64::NEG_INFINITY, f64::max);
            let total_anomalies = history.iter().map(|m| m.anomaly_score).sum::<f64>() as u64;

            Some(QualityStatistics {
                sample_count: count,
                average_quality: avg_quality,
                min_quality,
                max_quality,
                total_anomalies,
                last_assessment: history.back().unwrap().timestamp,
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityStatistics {
    pub sample_count: usize,
    pub average_quality: f64,
    pub min_quality: f64,
    pub max_quality: f64,
    pub total_anomalies: u64,
    pub last_assessment: DateTime<Utc>,
}
