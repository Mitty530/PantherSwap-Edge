// Data integrity checker for PantherSwap Edge trading platform
// Ensures referential integrity, business rule compliance, and data consistency

use crate::utils::Result;
use crate::database::types::*;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};
use tracing::{info, warn, error};

/// Data integrity checker and enforcer
pub struct DataIntegrityChecker {
    pool: PgPool,
    config: IntegrityConfig,
    violation_cache: HashMap<String, Vec<IntegrityViolation>>,
}

#[derive(Debug, Clone)]
pub struct IntegrityConfig {
    pub check_referential_integrity: bool,
    pub check_business_rules: bool,
    pub check_temporal_consistency: bool,
    pub max_price_deviation_percent: f64,
    pub max_volume_spike_multiplier: f64,
    pub required_data_freshness_minutes: i64,
    pub enable_cross_validation: bool,
}

impl Default for IntegrityConfig {
    fn default() -> Self {
        Self {
            check_referential_integrity: true,
            check_business_rules: true,
            check_temporal_consistency: true,
            max_price_deviation_percent: 20.0, // 20%
            max_volume_spike_multiplier: 10.0, // 10x normal
            required_data_freshness_minutes: 5,
            enable_cross_validation: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityViolation {
    pub violation_type: String,
    pub severity: ViolationSeverity,
    pub description: String,
    pub affected_table: String,
    pub affected_record_id: Option<String>,
    pub detected_at: DateTime<Utc>,
    pub resolution_suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntegrityReport {
    pub check_timestamp: DateTime<Utc>,
    pub total_violations: usize,
    pub violations_by_severity: HashMap<String, usize>,
    pub violations_by_type: HashMap<String, usize>,
    pub affected_tables: HashSet<String>,
    pub violations: Vec<IntegrityViolation>,
    pub recommendations: Vec<String>,
}

impl DataIntegrityChecker {
    pub fn new(pool: PgPool, config: IntegrityConfig) -> Self {
        Self {
            pool,
            config,
            violation_cache: HashMap::new(),
        }
    }

    pub fn with_default_config(pool: PgPool) -> Self {
        Self::new(pool, IntegrityConfig::default())
    }

    /// Perform comprehensive integrity check
    pub async fn check_integrity(&mut self) -> Result<IntegrityReport> {
        info!("Starting comprehensive data integrity check");
        
        let mut violations = Vec::new();

        // Check referential integrity
        if self.config.check_referential_integrity {
            violations.extend(self.check_referential_integrity().await?);
        }

        // Check business rules
        if self.config.check_business_rules {
            violations.extend(self.check_business_rules().await?);
        }

        // Check temporal consistency
        if self.config.check_temporal_consistency {
            violations.extend(self.check_temporal_consistency().await?);
        }

        // Cross-validation checks
        if self.config.enable_cross_validation {
            violations.extend(self.check_cross_validation().await?);
        }

        // Generate report
        let report = self.generate_integrity_report(violations);
        
        info!("Integrity check completed: {} violations found", report.total_violations);
        Ok(report)
    }

    /// Check referential integrity constraints
    async fn check_referential_integrity(&self) -> Result<Vec<IntegrityViolation>> {
        let mut violations = Vec::new();

        // Check orphaned market ticks (instrument_id not in instruments)
        let orphaned_ticks = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM market_ticks mt
            LEFT JOIN instruments i ON mt.instrument_id = i.id
            WHERE i.id IS NULL
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        if orphaned_ticks.count.unwrap_or(0) > 0 {
            violations.push(IntegrityViolation {
                violation_type: "orphaned_records".to_string(),
                severity: ViolationSeverity::High,
                description: format!("Found {} market ticks with invalid instrument references", orphaned_ticks.count.unwrap_or(0)),
                affected_table: "market_ticks".to_string(),
                affected_record_id: None,
                detected_at: Utc::now(),
                resolution_suggestion: "Remove orphaned records or fix instrument references".to_string(),
            });
        }

        // Check orphaned trading signals
        let orphaned_signals = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM trading_signals ts
            LEFT JOIN instruments i ON ts.instrument_id = i.id
            WHERE i.id IS NULL
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        if orphaned_signals.count.unwrap_or(0) > 0 {
            violations.push(IntegrityViolation {
                violation_type: "orphaned_records".to_string(),
                severity: ViolationSeverity::High,
                description: format!("Found {} trading signals with invalid instrument references", orphaned_signals.count.unwrap_or(0)),
                affected_table: "trading_signals".to_string(),
                affected_record_id: None,
                detected_at: Utc::now(),
                resolution_suggestion: "Remove orphaned records or fix instrument references".to_string(),
            });
        }

        // Check orphaned AI predictions
        let orphaned_predictions = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM ai_predictions ap
            LEFT JOIN instruments i ON ap.instrument_id = i.id
            WHERE i.id IS NULL
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        if orphaned_predictions.count.unwrap_or(0) > 0 {
            violations.push(IntegrityViolation {
                violation_type: "orphaned_records".to_string(),
                severity: ViolationSeverity::Medium,
                description: format!("Found {} AI predictions with invalid instrument references", orphaned_predictions.count.unwrap_or(0)),
                affected_table: "ai_predictions".to_string(),
                affected_record_id: None,
                detected_at: Utc::now(),
                resolution_suggestion: "Remove orphaned records or fix instrument references".to_string(),
            });
        }

        Ok(violations)
    }

    /// Check business rule compliance
    async fn check_business_rules(&self) -> Result<Vec<IntegrityViolation>> {
        let mut violations = Vec::new();

        // Check for invalid price relationships (ask <= bid)
        let invalid_spreads = sqlx::query!(
            r#"
            SELECT instrument_id, provider, timestamp, bid_price, ask_price
            FROM market_ticks
            WHERE ask_price <= bid_price
            AND timestamp >= NOW() - INTERVAL '1 hour'
            LIMIT 100
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        for tick in invalid_spreads {
            violations.push(IntegrityViolation {
                violation_type: "invalid_spread".to_string(),
                severity: ViolationSeverity::High,
                description: format!("Invalid spread: ask ({}) <= bid ({})", tick.ask_price, tick.bid_price),
                affected_table: "market_ticks".to_string(),
                affected_record_id: Some(format!("{}_{}", tick.instrument_id, tick.timestamp)),
                detected_at: Utc::now(),
                resolution_suggestion: "Review data source and fix price feeds".to_string(),
            });
        }

        // Check for negative prices
        let negative_prices = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM market_ticks
            WHERE (bid_price <= 0 OR ask_price <= 0)
            AND timestamp >= NOW() - INTERVAL '1 hour'
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        if negative_prices.count.unwrap_or(0) > 0 {
            violations.push(IntegrityViolation {
                violation_type: "invalid_prices".to_string(),
                severity: ViolationSeverity::Critical,
                description: format!("Found {} records with negative or zero prices", negative_prices.count.unwrap_or(0)),
                affected_table: "market_ticks".to_string(),
                affected_record_id: None,
                detected_at: Utc::now(),
                resolution_suggestion: "Investigate data source and implement price validation".to_string(),
            });
        }

        // Check for invalid confidence scores
        let invalid_confidence = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM trading_signals
            WHERE confidence_score < 0 OR confidence_score > 1
            AND timestamp >= NOW() - INTERVAL '24 hours'
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        if invalid_confidence.count.unwrap_or(0) > 0 {
            violations.push(IntegrityViolation {
                violation_type: "invalid_confidence".to_string(),
                severity: ViolationSeverity::Medium,
                description: format!("Found {} trading signals with invalid confidence scores", invalid_confidence.count.unwrap_or(0)),
                affected_table: "trading_signals".to_string(),
                affected_record_id: None,
                detected_at: Utc::now(),
                resolution_suggestion: "Fix confidence score calculation in trading engine".to_string(),
            });
        }

        // Check for invalid risk scores
        let invalid_risk = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM trading_signals
            WHERE risk_score < 0 OR risk_score > 1
            AND timestamp >= NOW() - INTERVAL '24 hours'
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        if invalid_risk.count.unwrap_or(0) > 0 {
            violations.push(IntegrityViolation {
                violation_type: "invalid_risk_score".to_string(),
                severity: ViolationSeverity::Medium,
                description: format!("Found {} trading signals with invalid risk scores", invalid_risk.count.unwrap_or(0)),
                affected_table: "trading_signals".to_string(),
                affected_record_id: None,
                detected_at: Utc::now(),
                resolution_suggestion: "Fix risk score calculation in trading engine".to_string(),
            });
        }

        // Check for inconsistent signal price levels
        let inconsistent_signals = sqlx::query!(
            r#"
            SELECT instrument_id, timestamp, signal_type, target_price, stop_loss
            FROM trading_signals
            WHERE signal_type = 'BUY' 
            AND target_price IS NOT NULL 
            AND stop_loss IS NOT NULL
            AND target_price <= stop_loss
            AND timestamp >= NOW() - INTERVAL '24 hours'
            LIMIT 50
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        for signal in inconsistent_signals {
            violations.push(IntegrityViolation {
                violation_type: "inconsistent_price_levels".to_string(),
                severity: ViolationSeverity::High,
                description: format!("BUY signal with target ({:?}) <= stop loss ({:?})", signal.target_price, signal.stop_loss),
                affected_table: "trading_signals".to_string(),
                affected_record_id: Some(format!("{}_{}", signal.instrument_id, signal.timestamp)),
                detected_at: Utc::now(),
                resolution_suggestion: "Review signal generation logic for price level consistency".to_string(),
            });
        }

        Ok(violations)
    }

    /// Check temporal consistency
    async fn check_temporal_consistency(&self) -> Result<Vec<IntegrityViolation>> {
        let mut violations = Vec::new();

        // Check for future timestamps
        let future_timestamps = sqlx::query!(
            r#"
            SELECT 'market_ticks' as table_name, COUNT(*) as count
            FROM market_ticks
            WHERE timestamp > NOW() + INTERVAL '5 minutes'
            
            UNION ALL
            
            SELECT 'trading_signals' as table_name, COUNT(*) as count
            FROM trading_signals
            WHERE timestamp > NOW() + INTERVAL '5 minutes'
            
            UNION ALL
            
            SELECT 'ai_predictions' as table_name, COUNT(*) as count
            FROM ai_predictions
            WHERE timestamp > NOW() + INTERVAL '5 minutes'
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        for row in future_timestamps {
            if row.count.unwrap_or(0) > 0 {
                violations.push(IntegrityViolation {
                    violation_type: "future_timestamps".to_string(),
                    severity: ViolationSeverity::Medium,
                    description: format!("Found {} records with future timestamps in {}", row.count.unwrap_or(0), row.table_name.unwrap_or_default()),
                    affected_table: row.table_name.unwrap_or_default(),
                    affected_record_id: None,
                    detected_at: Utc::now(),
                    resolution_suggestion: "Check system clock synchronization and timestamp generation".to_string(),
                });
            }
        }

        // Check for stale data
        let stale_data_threshold = Utc::now() - Duration::minutes(self.config.required_data_freshness_minutes);
        
        let stale_instruments = sqlx::query!(
            r#"
            SELECT i.id, i.symbol, MAX(mt.timestamp) as last_update
            FROM instruments i
            LEFT JOIN market_ticks mt ON i.id = mt.instrument_id
            WHERE i.is_active = true
            GROUP BY i.id, i.symbol
            HAVING MAX(mt.timestamp) < $1 OR MAX(mt.timestamp) IS NULL
            "#,
            stale_data_threshold
        )
        .fetch_all(&self.pool)
        .await?;

        for instrument in stale_instruments {
            violations.push(IntegrityViolation {
                violation_type: "stale_data".to_string(),
                severity: ViolationSeverity::Medium,
                description: format!("No recent market data for active instrument: {}", instrument.symbol),
                affected_table: "market_ticks".to_string(),
                affected_record_id: Some(instrument.id.to_string()),
                detected_at: Utc::now(),
                resolution_suggestion: "Check market data feeds and provider connectivity".to_string(),
            });
        }

        Ok(violations)
    }

    /// Check cross-validation between different data sources
    async fn check_cross_validation(&self) -> Result<Vec<IntegrityViolation>> {
        let mut violations = Vec::new();

        // Check for significant price deviations between providers
        let price_deviations = sqlx::query!(
            r#"
            WITH provider_prices AS (
                SELECT 
                    instrument_id,
                    provider,
                    AVG((bid_price + ask_price) / 2) as avg_price,
                    COUNT(*) as tick_count
                FROM market_ticks
                WHERE timestamp >= NOW() - INTERVAL '1 hour'
                GROUP BY instrument_id, provider
                HAVING COUNT(*) >= 10
            ),
            price_comparison AS (
                SELECT 
                    p1.instrument_id,
                    p1.provider as provider1,
                    p2.provider as provider2,
                    p1.avg_price as price1,
                    p2.avg_price as price2,
                    ABS(p1.avg_price - p2.avg_price) / p1.avg_price * 100 as deviation_pct
                FROM provider_prices p1
                JOIN provider_prices p2 ON p1.instrument_id = p2.instrument_id
                WHERE p1.provider < p2.provider
            )
            SELECT *
            FROM price_comparison
            WHERE deviation_pct > $1
            "#,
            self.config.max_price_deviation_percent
        )
        .fetch_all(&self.pool)
        .await?;

        for deviation in price_deviations {
            violations.push(IntegrityViolation {
                violation_type: "price_deviation".to_string(),
                severity: ViolationSeverity::High,
                description: format!(
                    "Significant price deviation ({:.2}%) between {} and {} for instrument {}",
                    deviation.deviation_pct.unwrap_or(0.0),
                    deviation.provider1.unwrap_or_default(),
                    deviation.provider2.unwrap_or_default(),
                    deviation.instrument_id.unwrap_or_default()
                ),
                affected_table: "market_ticks".to_string(),
                affected_record_id: Some(deviation.instrument_id.unwrap_or_default().to_string()),
                detected_at: Utc::now(),
                resolution_suggestion: "Investigate data source discrepancies and provider reliability".to_string(),
            });
        }

        Ok(violations)
    }

    /// Generate comprehensive integrity report
    fn generate_integrity_report(&self, violations: Vec<IntegrityViolation>) -> IntegrityReport {
        let mut violations_by_severity = HashMap::new();
        let mut violations_by_type = HashMap::new();
        let mut affected_tables = HashSet::new();

        for violation in &violations {
            // Count by severity
            let severity_key = format!("{:?}", violation.severity);
            *violations_by_severity.entry(severity_key).or_insert(0) += 1;

            // Count by type
            *violations_by_type.entry(violation.violation_type.clone()).or_insert(0) += 1;

            // Track affected tables
            affected_tables.insert(violation.affected_table.clone());
        }

        // Generate recommendations
        let recommendations = self.generate_recommendations(&violations);

        IntegrityReport {
            check_timestamp: Utc::now(),
            total_violations: violations.len(),
            violations_by_severity,
            violations_by_type,
            affected_tables,
            violations,
            recommendations,
        }
    }

    /// Generate recommendations based on violations
    fn generate_recommendations(&self, violations: &[IntegrityViolation]) -> Vec<String> {
        let mut recommendations = Vec::new();

        let critical_count = violations.iter().filter(|v| matches!(v.severity, ViolationSeverity::Critical)).count();
        let high_count = violations.iter().filter(|v| matches!(v.severity, ViolationSeverity::High)).count();

        if critical_count > 0 {
            recommendations.push(format!("URGENT: {} critical violations require immediate attention", critical_count));
        }

        if high_count > 0 {
            recommendations.push(format!("HIGH PRIORITY: {} high-severity violations need resolution", high_count));
        }

        // Type-specific recommendations
        let orphaned_count = violations.iter().filter(|v| v.violation_type == "orphaned_records").count();
        if orphaned_count > 0 {
            recommendations.push("Review and clean up orphaned records to maintain referential integrity".to_string());
        }

        let price_issues = violations.iter().filter(|v| v.violation_type.contains("price")).count();
        if price_issues > 0 {
            recommendations.push("Investigate price data sources and validation logic".to_string());
        }

        let temporal_issues = violations.iter().filter(|v| v.violation_type.contains("timestamp") || v.violation_type == "stale_data").count();
        if temporal_issues > 0 {
            recommendations.push("Check system time synchronization and data feed connectivity".to_string());
        }

        if violations.is_empty() {
            recommendations.push("Data integrity is good - no violations detected".to_string());
        }

        recommendations
    }

    /// Fix specific types of violations automatically
    pub async fn auto_fix_violations(&self, violation_types: &[String]) -> Result<usize> {
        let mut fixed_count = 0;

        for violation_type in violation_types {
            match violation_type.as_str() {
                "orphaned_records" => {
                    fixed_count += self.fix_orphaned_records().await?;
                }
                "future_timestamps" => {
                    fixed_count += self.fix_future_timestamps().await?;
                }
                _ => {
                    warn!("Auto-fix not implemented for violation type: {}", violation_type);
                }
            }
        }

        info!("Auto-fixed {} violations", fixed_count);
        Ok(fixed_count)
    }

    /// Fix orphaned records by removing them
    async fn fix_orphaned_records(&self) -> Result<usize> {
        let mut fixed = 0;

        // Remove orphaned market ticks
        let result = sqlx::query!(
            r#"
            DELETE FROM market_ticks
            WHERE instrument_id NOT IN (SELECT id FROM instruments)
            "#
        )
        .execute(&self.pool)
        .await?;
        fixed += result.rows_affected() as usize;

        // Remove orphaned trading signals
        let result = sqlx::query!(
            r#"
            DELETE FROM trading_signals
            WHERE instrument_id NOT IN (SELECT id FROM instruments)
            "#
        )
        .execute(&self.pool)
        .await?;
        fixed += result.rows_affected() as usize;

        // Remove orphaned AI predictions
        let result = sqlx::query!(
            r#"
            DELETE FROM ai_predictions
            WHERE instrument_id NOT IN (SELECT id FROM instruments)
            "#
        )
        .execute(&self.pool)
        .await?;
        fixed += result.rows_affected() as usize;

        Ok(fixed)
    }

    /// Fix future timestamps by updating them to current time
    async fn fix_future_timestamps(&self) -> Result<usize> {
        let mut fixed = 0;
        let now = Utc::now();

        // Fix market ticks
        let result = sqlx::query!(
            r#"
            UPDATE market_ticks
            SET timestamp = $1
            WHERE timestamp > NOW() + INTERVAL '5 minutes'
            "#,
            now
        )
        .execute(&self.pool)
        .await?;
        fixed += result.rows_affected() as usize;

        // Fix trading signals
        let result = sqlx::query!(
            r#"
            UPDATE trading_signals
            SET timestamp = $1
            WHERE timestamp > NOW() + INTERVAL '5 minutes'
            "#,
            now
        )
        .execute(&self.pool)
        .await?;
        fixed += result.rows_affected() as usize;

        // Fix AI predictions
        let result = sqlx::query!(
            r#"
            UPDATE ai_predictions
            SET timestamp = $1
            WHERE timestamp > NOW() + INTERVAL '5 minutes'
            "#,
            now
        )
        .execute(&self.pool)
        .await?;
        fixed += result.rows_affected() as usize;

        Ok(fixed)
    }
}
