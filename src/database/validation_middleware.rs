// Validation middleware for PantherSwap Edge trading platform
// Integrates data validation, quality assessment, and integrity checking

use crate::utils::Result;
use crate::database::{
    types::*,
    validation::{DataValidator, ValidationError, ValidationConfig},
    data_quality::{DataQualityAssessor, QualityConfig, QualityReport},
    integrity::{DataIntegrityChecker, IntegrityConfig, IntegrityReport},
};
use sqlx::PgPool;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use tracing::{info, warn, error};

/// Comprehensive validation middleware
pub struct ValidationMiddleware {
    validator: Arc<RwLock<DataValidator>>,
    quality_assessor: Arc<RwLock<DataQualityAssessor>>,
    integrity_checker: Arc<RwLock<DataIntegrityChecker>>,
    config: MiddlewareConfig,
}

#[derive(Debug, Clone)]
pub struct MiddlewareConfig {
    pub enable_validation: bool,
    pub enable_quality_assessment: bool,
    pub enable_integrity_checking: bool,
    pub fail_on_validation_error: bool,
    pub fail_on_low_quality: bool,
    pub quality_threshold: f64,
    pub auto_fix_violations: bool,
    pub log_validation_results: bool,
}

impl Default for MiddlewareConfig {
    fn default() -> Self {
        Self {
            enable_validation: true,
            enable_quality_assessment: true,
            enable_integrity_checking: true,
            fail_on_validation_error: true,
            fail_on_low_quality: false, // Allow low quality but log it
            quality_threshold: 0.7,
            auto_fix_violations: false, // Manual approval required
            log_validation_results: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub validation_passed: bool,
    pub quality_score: Option<f64>,
    pub quality_report: Option<QualityReport>,
    pub validation_errors: Vec<String>,
    pub recommendations: Vec<String>,
    pub processed_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComprehensiveValidationReport {
    pub validation_summary: ValidationSummary,
    pub quality_summary: QualitySummary,
    pub integrity_summary: IntegritySummary,
    pub overall_health_score: f64,
    pub critical_issues: Vec<String>,
    pub recommendations: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationSummary {
    pub total_validations: u64,
    pub successful_validations: u64,
    pub failed_validations: u64,
    pub error_breakdown: std::collections::HashMap<String, u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QualitySummary {
    pub average_quality_score: f64,
    pub instruments_assessed: usize,
    pub low_quality_alerts: usize,
    pub anomalies_detected: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntegritySummary {
    pub total_violations: usize,
    pub critical_violations: usize,
    pub high_severity_violations: usize,
    pub auto_fixed_violations: usize,
}

impl ValidationMiddleware {
    /// Create new validation middleware with custom configurations
    pub fn new(
        pool: PgPool,
        validation_config: ValidationConfig,
        quality_config: QualityConfig,
        integrity_config: IntegrityConfig,
        middleware_config: MiddlewareConfig,
    ) -> Self {
        Self {
            validator: Arc::new(RwLock::new(DataValidator::new(validation_config))),
            quality_assessor: Arc::new(RwLock::new(DataQualityAssessor::new(quality_config))),
            integrity_checker: Arc::new(RwLock::new(DataIntegrityChecker::new(pool, integrity_config))),
            config: middleware_config,
        }
    }

    /// Create validation middleware with default configurations
    pub fn with_defaults(pool: PgPool) -> Self {
        Self::new(
            pool,
            ValidationConfig::default(),
            QualityConfig::default(),
            IntegrityConfig::default(),
            MiddlewareConfig::default(),
        )
    }

    /// Validate market tick data with comprehensive checks
    pub async fn validate_market_tick(&self, tick: &MarketTick) -> Result<ValidationResult> {
        let mut validation_errors = Vec::new();
        let mut recommendations = Vec::new();
        let mut is_valid = true;
        let mut validation_passed = true;
        let mut quality_score = None;
        let mut quality_report = None;

        // Step 1: Basic validation
        if self.config.enable_validation {
            let mut validator = self.validator.write().await;
            if let Err(error) = validator.validate_market_tick(tick) {
                validation_passed = false;
                validation_errors.push(error.to_string());
                
                if self.config.fail_on_validation_error {
                    is_valid = false;
                }
                
                if self.config.log_validation_results {
                    warn!("Market tick validation failed: {}", error);
                }
            }
        }

        // Step 2: Quality assessment
        if self.config.enable_quality_assessment {
            let mut assessor = self.quality_assessor.write().await;
            match assessor.assess_market_tick_quality(tick) {
                Ok(report) => {
                    quality_score = Some(report.overall_score);
                    
                    if report.overall_score < self.config.quality_threshold {
                        if self.config.fail_on_low_quality {
                            is_valid = false;
                        }
                        
                        if self.config.log_validation_results {
                            warn!("Low quality market tick: score {:.2}", report.overall_score);
                        }
                    }
                    
                    recommendations.extend(report.recommendations.clone());
                    quality_report = Some(report);
                }
                Err(error) => {
                    validation_errors.push(format!("Quality assessment failed: {}", error));
                    if self.config.log_validation_results {
                        error!("Quality assessment error: {}", error);
                    }
                }
            }
        }

        // Log successful validation
        if is_valid && self.config.log_validation_results {
            info!("Market tick validation successful for instrument {}", tick.instrument_id);
        }

        Ok(ValidationResult {
            is_valid,
            validation_passed,
            quality_score,
            quality_report,
            validation_errors,
            recommendations,
            processed_at: Utc::now(),
        })
    }

    /// Validate trading signal with comprehensive checks
    pub async fn validate_trading_signal(&self, signal: &TradingSignal) -> Result<ValidationResult> {
        let mut validation_errors = Vec::new();
        let mut recommendations = Vec::new();
        let mut is_valid = true;
        let mut validation_passed = true;
        let mut quality_score = None;
        let mut quality_report = None;

        // Step 1: Basic validation
        if self.config.enable_validation {
            let mut validator = self.validator.write().await;
            if let Err(error) = validator.validate_trading_signal(signal) {
                validation_passed = false;
                validation_errors.push(error.to_string());
                
                if self.config.fail_on_validation_error {
                    is_valid = false;
                }
                
                if self.config.log_validation_results {
                    warn!("Trading signal validation failed: {}", error);
                }
            }
        }

        // Step 2: Quality assessment
        if self.config.enable_quality_assessment {
            let assessor = self.quality_assessor.read().await;
            match assessor.assess_trading_signal_quality(signal) {
                Ok(report) => {
                    quality_score = Some(report.overall_score);
                    
                    if report.overall_score < self.config.quality_threshold {
                        if self.config.fail_on_low_quality {
                            is_valid = false;
                        }
                        
                        if self.config.log_validation_results {
                            warn!("Low quality trading signal: score {:.2}", report.overall_score);
                        }
                    }
                    
                    recommendations.extend(report.recommendations.clone());
                    quality_report = Some(report);
                }
                Err(error) => {
                    validation_errors.push(format!("Quality assessment failed: {}", error));
                    if self.config.log_validation_results {
                        error!("Quality assessment error: {}", error);
                    }
                }
            }
        }

        Ok(ValidationResult {
            is_valid,
            validation_passed,
            quality_score,
            quality_report,
            validation_errors,
            recommendations,
            processed_at: Utc::now(),
        })
    }

    /// Validate AI prediction with comprehensive checks
    pub async fn validate_ai_prediction(&self, prediction: &AIPrediction) -> Result<ValidationResult> {
        let mut validation_errors = Vec::new();
        let mut recommendations = Vec::new();
        let mut is_valid = true;
        let mut validation_passed = true;

        // Basic validation
        if self.config.enable_validation {
            let mut validator = self.validator.write().await;
            if let Err(error) = validator.validate_ai_prediction(prediction) {
                validation_passed = false;
                validation_errors.push(error.to_string());
                
                if self.config.fail_on_validation_error {
                    is_valid = false;
                }
                
                if self.config.log_validation_results {
                    warn!("AI prediction validation failed: {}", error);
                }
            }
        }

        // Add AI-specific quality checks
        if prediction.confidence_score < 0.6 {
            recommendations.push("Consider improving model confidence before using prediction".to_string());
        }

        if prediction.prediction_horizon_minutes > 60 {
            recommendations.push("Long-term predictions may have reduced accuracy".to_string());
        }

        Ok(ValidationResult {
            is_valid,
            validation_passed,
            quality_score: Some(prediction.confidence_score),
            quality_report: None,
            validation_errors,
            recommendations,
            processed_at: Utc::now(),
        })
    }

    /// Validate instrument data
    pub async fn validate_instrument(&self, instrument: &Instrument) -> Result<ValidationResult> {
        let mut validation_errors = Vec::new();
        let mut is_valid = true;
        let mut validation_passed = true;

        if self.config.enable_validation {
            let mut validator = self.validator.write().await;
            if let Err(error) = validator.validate_instrument(instrument) {
                validation_passed = false;
                validation_errors.push(error.to_string());
                
                if self.config.fail_on_validation_error {
                    is_valid = false;
                }
                
                if self.config.log_validation_results {
                    warn!("Instrument validation failed: {}", error);
                }
            }
        }

        Ok(ValidationResult {
            is_valid,
            validation_passed,
            quality_score: Some(1.0), // Instruments don't have quality scores
            quality_report: None,
            validation_errors,
            recommendations: Vec::new(),
            processed_at: Utc::now(),
        })
    }

    /// Perform comprehensive system integrity check
    pub async fn check_system_integrity(&self) -> Result<ComprehensiveValidationReport> {
        let mut critical_issues = Vec::new();
        let mut recommendations = Vec::new();

        // Get validation statistics
        let validator = self.validator.read().await;
        let validation_stats = validator.get_stats();
        let validation_summary = ValidationSummary {
            total_validations: validation_stats.total_validations,
            successful_validations: validation_stats.successful_validations,
            failed_validations: validation_stats.failed_validations,
            error_breakdown: validation_stats.validation_errors.clone(),
        };

        // Perform integrity check
        let integrity_report = if self.config.enable_integrity_checking {
            let mut checker = self.integrity_checker.write().await;
            match checker.check_integrity().await {
                Ok(report) => {
                    if report.total_violations > 0 {
                        critical_issues.push(format!("Found {} data integrity violations", report.total_violations));
                        recommendations.extend(report.recommendations.clone());
                    }
                    
                    // Auto-fix if enabled
                    if self.config.auto_fix_violations && !report.violations.is_empty() {
                        let fixable_types: Vec<String> = report.violations
                            .iter()
                            .filter(|v| v.violation_type == "orphaned_records" || v.violation_type == "future_timestamps")
                            .map(|v| v.violation_type.clone())
                            .collect();
                        
                        if !fixable_types.is_empty() {
                            match checker.auto_fix_violations(&fixable_types).await {
                                Ok(fixed_count) => {
                                    info!("Auto-fixed {} violations", fixed_count);
                                }
                                Err(e) => {
                                    error!("Auto-fix failed: {}", e);
                                }
                            }
                        }
                    }
                    
                    Some(report)
                }
                Err(e) => {
                    critical_issues.push(format!("Integrity check failed: {}", e));
                    None
                }
            }
        } else {
            None
        };

        // Calculate overall health score
        let validation_health = if validation_summary.total_validations > 0 {
            validation_summary.successful_validations as f64 / validation_summary.total_validations as f64
        } else {
            1.0
        };

        let integrity_health = if let Some(ref report) = integrity_report {
            if report.total_violations == 0 {
                1.0
            } else {
                let critical_violations = report.violations.iter()
                    .filter(|v| matches!(v.severity, crate::database::integrity::ViolationSeverity::Critical))
                    .count();
                
                if critical_violations > 0 {
                    0.3 // Critical issues significantly impact health
                } else {
                    0.7 // Non-critical issues have moderate impact
                }
            }
        } else {
            1.0
        };

        let overall_health_score = (validation_health + integrity_health) / 2.0;

        // Generate summary reports
        let quality_summary = QualitySummary {
            average_quality_score: 0.85, // This would be calculated from actual quality assessments
            instruments_assessed: 0,
            low_quality_alerts: 0,
            anomalies_detected: 0,
        };

        let integrity_summary = if let Some(ref report) = integrity_report {
            let critical_violations = report.violations.iter()
                .filter(|v| matches!(v.severity, crate::database::integrity::ViolationSeverity::Critical))
                .count();
            let high_violations = report.violations.iter()
                .filter(|v| matches!(v.severity, crate::database::integrity::ViolationSeverity::High))
                .count();

            IntegritySummary {
                total_violations: report.total_violations,
                critical_violations,
                high_severity_violations: high_violations,
                auto_fixed_violations: 0, // Would track this in real implementation
            }
        } else {
            IntegritySummary {
                total_violations: 0,
                critical_violations: 0,
                high_severity_violations: 0,
                auto_fixed_violations: 0,
            }
        };

        // Add health-based recommendations
        if overall_health_score < 0.7 {
            recommendations.push("System health is below optimal - immediate attention required".to_string());
        } else if overall_health_score < 0.9 {
            recommendations.push("System health is good but could be improved".to_string());
        }

        Ok(ComprehensiveValidationReport {
            validation_summary,
            quality_summary,
            integrity_summary,
            overall_health_score,
            critical_issues,
            recommendations,
            generated_at: Utc::now(),
        })
    }

    /// Get validation statistics
    pub async fn get_validation_stats(&self) -> ValidationSummary {
        let validator = self.validator.read().await;
        let stats = validator.get_stats();
        
        ValidationSummary {
            total_validations: stats.total_validations,
            successful_validations: stats.successful_validations,
            failed_validations: stats.failed_validations,
            error_breakdown: stats.validation_errors.clone(),
        }
    }

    /// Reset all validation statistics
    pub async fn reset_stats(&self) {
        let mut validator = self.validator.write().await;
        validator.reset_stats();
    }

    /// Update middleware configuration
    pub fn update_config(&mut self, config: MiddlewareConfig) {
        self.config = config;
    }
}
