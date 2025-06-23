// AI Performance Monitoring and Model Drift Detection
use crate::utils::{Result, PantherSwapError};
use crate::database::Database;
use crate::ai::models::{ModelType, ModelMetadata, ModelPerformanceMetrics};
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;
use tracing::{info, warn, error, debug};
use tokio::sync::RwLock;
use std::sync::Arc;

/// Configuration for AI performance monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub drift_detection_window: usize,
    pub performance_window: usize,
    pub drift_threshold: f64,
    pub accuracy_threshold: f64,
    pub latency_threshold_ms: f64,
    pub monitoring_interval_seconds: u64,
    pub alert_cooldown_minutes: u64,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            drift_detection_window: 1000,
            performance_window: 100,
            drift_threshold: 0.15, // 15% drift threshold
            accuracy_threshold: 0.65, // 65% minimum accuracy
            latency_threshold_ms: 100.0,
            monitoring_interval_seconds: 60,
            alert_cooldown_minutes: 15,
        }
    }
}

/// Performance sample for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSample {
    pub timestamp: DateTime<Utc>,
    pub model_id: Uuid,
    pub prediction_accuracy: f64,
    pub inference_latency_ms: f64,
    pub confidence_score: f64,
    pub actual_value: Option<f64>,
    pub predicted_value: f64,
    pub error_magnitude: f64,
}

/// Model drift detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftDetectionResult {
    pub model_id: Uuid,
    pub drift_score: f64,
    pub is_drifting: bool,
    pub drift_type: DriftType,
    pub detection_timestamp: DateTime<Utc>,
    pub recommendation: DriftRecommendation,
}

/// Types of model drift
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DriftType {
    ConceptDrift,    // Relationship between features and target changes
    DataDrift,       // Input data distribution changes
    PerformanceDrift, // Model performance degrades
    LatencyDrift,    // Inference latency increases
}

/// Recommendations for handling drift
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DriftRecommendation {
    Retrain,
    RecalibrateThresholds,
    IncreaseMonitoring,
    ReplaceModel,
    NoAction,
}

/// Alert for model performance issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    pub id: Uuid,
    pub model_id: Uuid,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub metrics: HashMap<String, f64>,
}

/// Types of performance alerts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AlertType {
    AccuracyDrop,
    LatencyIncrease,
    ModelDrift,
    PredictionBias,
    ConfidenceCalibration,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Comprehensive AI performance monitor
pub struct AIPerformanceMonitor {
    config: MonitoringConfig,
    database: Database,
    
    // Performance tracking
    performance_samples: Arc<RwLock<HashMap<Uuid, VecDeque<PerformanceSample>>>>,
    model_metadata: Arc<RwLock<HashMap<Uuid, ModelMetadata>>>,
    
    // Drift detection
    drift_detectors: Arc<RwLock<HashMap<Uuid, DriftDetector>>>,
    
    // Alerting
    active_alerts: Arc<RwLock<HashMap<Uuid, PerformanceAlert>>>,
    alert_history: Arc<RwLock<VecDeque<PerformanceAlert>>>,
    last_alert_time: Arc<RwLock<HashMap<(Uuid, AlertType), DateTime<Utc>>>>,
}

impl AIPerformanceMonitor {
    /// Create a new AI performance monitor
    pub fn new(config: MonitoringConfig, database: Database) -> Self {
        Self {
            config,
            database,
            performance_samples: Arc::new(RwLock::new(HashMap::new())),
            model_metadata: Arc::new(RwLock::new(HashMap::new())),
            drift_detectors: Arc::new(RwLock::new(HashMap::new())),
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            alert_history: Arc::new(RwLock::new(VecDeque::new())),
            last_alert_time: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a model for monitoring
    pub async fn register_model(&self, metadata: ModelMetadata) -> Result<()> {
        let model_id = metadata.model_id;
        
        // Store model metadata
        {
            let mut models = self.model_metadata.write().await;
            models.insert(model_id, metadata);
        }
        
        // Initialize performance tracking
        {
            let mut samples = self.performance_samples.write().await;
            samples.insert(model_id, VecDeque::with_capacity(self.config.performance_window * 2));
        }
        
        // Initialize drift detector
        {
            let mut detectors = self.drift_detectors.write().await;
            detectors.insert(model_id, DriftDetector::new(self.config.drift_detection_window));
        }
        
        info!("Registered model for monitoring: {}", model_id);
        Ok(())
    }
    
    /// Record a performance sample
    pub async fn record_performance(&self, sample: PerformanceSample) -> Result<()> {
        let model_id = sample.model_id;
        
        // Store performance sample
        {
            let mut samples = self.performance_samples.write().await;
            if let Some(model_samples) = samples.get_mut(&model_id) {
                if model_samples.len() >= self.config.performance_window * 2 {
                    model_samples.pop_front();
                }
                model_samples.push_back(sample.clone());
            }
        }
        
        // Update drift detector
        {
            let mut detectors = self.drift_detectors.write().await;
            if let Some(detector) = detectors.get_mut(&model_id) {
                detector.add_sample(&sample);
            }
        }
        
        // Check for performance issues
        self.check_performance_thresholds(model_id, &sample).await?;
        
        debug!("Recorded performance sample for model: {}", model_id);
        Ok(())
    }
    
    /// Check performance against thresholds and generate alerts
    async fn check_performance_thresholds(&self, model_id: Uuid, sample: &PerformanceSample) -> Result<()> {
        let mut alerts = Vec::new();
        
        // Check accuracy threshold
        if sample.prediction_accuracy < self.config.accuracy_threshold {
            alerts.push(self.create_alert(
                model_id,
                AlertType::AccuracyDrop,
                AlertSeverity::High,
                format!("Model accuracy dropped to {:.2}%", sample.prediction_accuracy * 100.0),
                vec![("accuracy", sample.prediction_accuracy), ("threshold", self.config.accuracy_threshold)],
            ));
        }
        
        // Check latency threshold
        if sample.inference_latency_ms > self.config.latency_threshold_ms {
            alerts.push(self.create_alert(
                model_id,
                AlertType::LatencyIncrease,
                AlertSeverity::Medium,
                format!("Inference latency increased to {:.2}ms", sample.inference_latency_ms),
                vec![("latency_ms", sample.inference_latency_ms), ("threshold_ms", self.config.latency_threshold_ms)],
            ));
        }
        
        // Process alerts
        for alert in alerts {
            self.process_alert(alert).await?;
        }
        
        Ok(())
    }
    
    /// Detect model drift
    pub async fn detect_drift(&self, model_id: Uuid) -> Result<Option<DriftDetectionResult>> {
        let detectors = self.drift_detectors.read().await;
        
        if let Some(detector) = detectors.get(&model_id) {
            if let Some(drift_result) = detector.detect_drift(self.config.drift_threshold) {
                // Generate drift alert if significant
                if drift_result.is_drifting {
                    let alert = self.create_alert(
                        model_id,
                        AlertType::ModelDrift,
                        AlertSeverity::High,
                        format!("Model drift detected: {:?} (score: {:.3})", drift_result.drift_type, drift_result.drift_score),
                        vec![("drift_score", drift_result.drift_score), ("threshold", self.config.drift_threshold)],
                    );
                    
                    self.process_alert(alert).await?;
                }
                
                return Ok(Some(drift_result));
            }
        }
        
        Ok(None)
    }
    
    /// Get performance metrics for a model
    pub async fn get_model_performance(&self, model_id: Uuid) -> Result<Option<ModelPerformanceMetrics>> {
        let samples = self.performance_samples.read().await;
        
        if let Some(model_samples) = samples.get(&model_id) {
            if model_samples.is_empty() {
                return Ok(None);
            }
            
            let recent_samples: Vec<_> = model_samples.iter()
                .rev()
                .take(self.config.performance_window)
                .collect();
            
            let accuracy = recent_samples.iter()
                .map(|s| s.prediction_accuracy)
                .sum::<f64>() / recent_samples.len() as f64;
            
            let avg_latency = recent_samples.iter()
                .map(|s| s.inference_latency_ms)
                .sum::<f64>() / recent_samples.len() as f64;
            
            let total_predictions = recent_samples.len() as u64;
            let successful_predictions = recent_samples.iter()
                .filter(|s| s.prediction_accuracy > 0.5)
                .count() as u64;
            
            let metrics = ModelPerformanceMetrics {
                accuracy,
                precision: accuracy, // Simplified - would calculate properly in production
                recall: accuracy,    // Simplified - would calculate properly in production
                f1_score: accuracy,  // Simplified - would calculate properly in production
                inference_latency_ms: avg_latency,
                training_loss: 0.0,  // Not applicable for inference monitoring
                validation_loss: 0.0, // Not applicable for inference monitoring
                total_predictions,
                successful_predictions,
                last_evaluation: Some(Utc::now()),
            };
            
            return Ok(Some(metrics));
        }
        
        Ok(None)
    }
    
    /// Create a performance alert
    fn create_alert(
        &self,
        model_id: Uuid,
        alert_type: AlertType,
        severity: AlertSeverity,
        message: String,
        metrics: Vec<(&str, f64)>,
    ) -> PerformanceAlert {
        let metrics_map: HashMap<String, f64> = metrics.into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        
        PerformanceAlert {
            id: Uuid::new_v4(),
            model_id,
            alert_type,
            severity,
            message,
            timestamp: Utc::now(),
            metrics: metrics_map,
        }
    }
    
    /// Process and store an alert
    async fn process_alert(&self, alert: PerformanceAlert) -> Result<()> {
        // Check alert cooldown
        let alert_key = (alert.model_id, alert.alert_type.clone());
        let should_alert = {
            let last_alerts = self.last_alert_time.read().await;
            if let Some(last_time) = last_alerts.get(&alert_key) {
                let cooldown = Duration::minutes(self.config.alert_cooldown_minutes as i64);
                Utc::now().signed_duration_since(*last_time) > cooldown
            } else {
                true
            }
        };
        
        if should_alert {
            // Update last alert time
            {
                let mut last_alerts = self.last_alert_time.write().await;
                last_alerts.insert(alert_key, alert.timestamp);
            }
            
            // Store active alert
            {
                let mut active = self.active_alerts.write().await;
                active.insert(alert.id, alert.clone());
            }
            
            // Add to history
            {
                let mut history = self.alert_history.write().await;
                if history.len() >= 1000 {
                    history.pop_front();
                }
                history.push_back(alert.clone());
            }
            
            // Log alert
            match alert.severity {
                AlertSeverity::Critical => error!("CRITICAL ALERT: {}", alert.message),
                AlertSeverity::High => warn!("HIGH ALERT: {}", alert.message),
                AlertSeverity::Medium => warn!("MEDIUM ALERT: {}", alert.message),
                AlertSeverity::Low => info!("LOW ALERT: {}", alert.message),
            }
        }
        
        Ok(())
    }
    
    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Vec<PerformanceAlert> {
        let alerts = self.active_alerts.read().await;
        alerts.values().cloned().collect()
    }
    
    /// Clear an alert
    pub async fn clear_alert(&self, alert_id: Uuid) -> Result<()> {
        let mut alerts = self.active_alerts.write().await;
        alerts.remove(&alert_id);
        info!("Cleared alert: {}", alert_id);
        Ok(())
    }
    
    /// Start background monitoring tasks
    pub async fn start_monitoring(&self) -> Result<()> {
        let monitor = self.clone();
        let interval_duration = std::time::Duration::from_secs(self.config.monitoring_interval_seconds);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval_duration);
            loop {
                interval.tick().await;
                if let Err(e) = monitor.run_monitoring_cycle().await {
                    error!("Monitoring cycle failed: {}", e);
                }
            }
        });
        
        info!("AI performance monitoring started");
        Ok(())
    }
    
    /// Run a monitoring cycle
    async fn run_monitoring_cycle(&self) -> Result<()> {
        let model_ids: Vec<Uuid> = {
            let models = self.model_metadata.read().await;
            models.keys().cloned().collect()
        };
        
        for model_id in model_ids {
            // Detect drift
            if let Err(e) = self.detect_drift(model_id).await {
                warn!("Drift detection failed for model {}: {}", model_id, e);
            }
            
            // Update performance metrics
            if let Ok(Some(metrics)) = self.get_model_performance(model_id).await {
                debug!("Model {} performance: accuracy={:.3}, latency={:.1}ms", 
                       model_id, metrics.accuracy, metrics.inference_latency_ms);
            }
        }
        
        Ok(())
    }
}

// Clone implementation for background tasks
impl Clone for AIPerformanceMonitor {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            database: self.database.clone(),
            performance_samples: self.performance_samples.clone(),
            model_metadata: self.model_metadata.clone(),
            drift_detectors: self.drift_detectors.clone(),
            active_alerts: self.active_alerts.clone(),
            alert_history: self.alert_history.clone(),
            last_alert_time: self.last_alert_time.clone(),
        }
    }
}

/// Drift detector for individual models
pub struct DriftDetector {
    window_size: usize,
    samples: VecDeque<PerformanceSample>,
    baseline_stats: Option<BaselineStats>,
}

#[derive(Debug, Clone)]
struct BaselineStats {
    mean_accuracy: f64,
    std_accuracy: f64,
    mean_latency: f64,
    std_latency: f64,
    mean_error: f64,
    std_error: f64,
}

impl DriftDetector {
    pub fn new(window_size: usize) -> Self {
        Self {
            window_size,
            samples: VecDeque::with_capacity(window_size * 2),
            baseline_stats: None,
        }
    }
    
    pub fn add_sample(&mut self, sample: &PerformanceSample) {
        if self.samples.len() >= self.window_size * 2 {
            self.samples.pop_front();
        }
        self.samples.push_back(sample.clone());
        
        // Update baseline if we have enough samples
        if self.samples.len() >= self.window_size && self.baseline_stats.is_none() {
            self.update_baseline();
        }
    }
    
    fn update_baseline(&mut self) {
        let baseline_samples: Vec<_> = self.samples.iter().take(self.window_size).collect();
        
        let accuracies: Vec<f64> = baseline_samples.iter().map(|s| s.prediction_accuracy).collect();
        let latencies: Vec<f64> = baseline_samples.iter().map(|s| s.inference_latency_ms).collect();
        let errors: Vec<f64> = baseline_samples.iter().map(|s| s.error_magnitude).collect();
        
        self.baseline_stats = Some(BaselineStats {
            mean_accuracy: mean(&accuracies),
            std_accuracy: std_dev(&accuracies),
            mean_latency: mean(&latencies),
            std_latency: std_dev(&latencies),
            mean_error: mean(&errors),
            std_error: std_dev(&errors),
        });
    }
    
    pub fn detect_drift(&self, threshold: f64) -> Option<DriftDetectionResult> {
        if let Some(baseline) = &self.baseline_stats {
            if self.samples.len() >= self.window_size * 2 {
                let recent_samples: Vec<_> = self.samples.iter().rev().take(self.window_size).collect();
                
                let recent_accuracies: Vec<f64> = recent_samples.iter().map(|s| s.prediction_accuracy).collect();
                let recent_latencies: Vec<f64> = recent_samples.iter().map(|s| s.inference_latency_ms).collect();
                let recent_errors: Vec<f64> = recent_samples.iter().map(|s| s.error_magnitude).collect();
                
                // Calculate drift scores
                let accuracy_drift = (mean(&recent_accuracies) - baseline.mean_accuracy).abs() / baseline.std_accuracy.max(1e-8);
                let latency_drift = (mean(&recent_latencies) - baseline.mean_latency).abs() / baseline.std_latency.max(1e-8);
                let error_drift = (mean(&recent_errors) - baseline.mean_error).abs() / baseline.std_error.max(1e-8);
                
                // Determine dominant drift type and overall score
                let (drift_type, drift_score) = if accuracy_drift > latency_drift && accuracy_drift > error_drift {
                    (DriftType::PerformanceDrift, accuracy_drift)
                } else if latency_drift > error_drift {
                    (DriftType::LatencyDrift, latency_drift)
                } else {
                    (DriftType::ConceptDrift, error_drift)
                };
                
                let is_drifting = drift_score > threshold;
                let recommendation = if is_drifting {
                    match drift_type {
                        DriftType::PerformanceDrift => DriftRecommendation::Retrain,
                        DriftType::LatencyDrift => DriftRecommendation::IncreaseMonitoring,
                        _ => DriftRecommendation::RecalibrateThresholds,
                    }
                } else {
                    DriftRecommendation::NoAction
                };
                
                return Some(DriftDetectionResult {
                    model_id: recent_samples[0].model_id,
                    drift_score,
                    is_drifting,
                    drift_type,
                    detection_timestamp: Utc::now(),
                    recommendation,
                });
            }
        }
        
        None
    }
}

// Utility functions
fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

fn std_dev(values: &[f64]) -> f64 {
    if values.len() < 2 {
        0.0
    } else {
        let mean_val = mean(values);
        let variance = values.iter()
            .map(|x| (x - mean_val).powi(2))
            .sum::<f64>() / (values.len() - 1) as f64;
        variance.sqrt()
    }
}

/// Factory function to create an AI performance monitor
pub fn create_ai_performance_monitor(database: Database) -> AIPerformanceMonitor {
    let config = MonitoringConfig::default();
    AIPerformanceMonitor::new(config, database)
}
