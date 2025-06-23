use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use uuid::Uuid;
use sqlx::PgPool;
use tracing::{info, warn, error, debug};
use crate::utils::Result;

/// Data retention configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRetentionConfig {
    pub enabled: bool,
    pub retention_policies: HashMap<String, RetentionPolicy>,
    pub archival_policies: HashMap<String, ArchivalPolicy>,
    pub cleanup_schedule: CleanupSchedule,
    pub compliance_requirements: ComplianceRequirements,
    pub enable_audit_logging: bool,
}

/// Retention policy for specific data types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub data_type: String,
    pub retention_period_days: u32,
    pub archival_period_days: Option<u32>,
    pub deletion_method: DeletionMethod,
    pub compliance_category: ComplianceCategory,
    pub enable_soft_delete: bool,
    pub soft_delete_period_days: u32,
    pub batch_size: u32,
    pub max_deletion_rate_per_hour: u32,
}

/// Archival policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivalPolicy {
    pub data_type: String,
    pub archive_after_days: u32,
    pub archive_location: ArchiveLocation,
    pub compression_enabled: bool,
    pub encryption_enabled: bool,
    pub verification_enabled: bool,
    pub restore_sla_hours: u32,
}

/// Archive storage locations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArchiveLocation {
    ColdStorage { path: String },
    S3Glacier { bucket: String, region: String },
    AzureArchive { container: String, account: String },
    GCSColdline { bucket: String },
    Tape { library: String },
}

/// Data deletion methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeletionMethod {
    SoftDelete,
    HardDelete,
    Anonymize,
    Archive,
    Truncate,
}

/// Compliance categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceCategory {
    Financial,     // 7 years retention
    Personal,      // GDPR compliance
    Operational,   // Business requirements
    Audit,         // Audit trail requirements
    Temporary,     // Short-term data
}

/// Cleanup schedule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupSchedule {
    pub frequency: CleanupFrequency,
    pub time_of_day: String, // HH:MM format
    pub timezone: String,
    pub max_concurrent_operations: u8,
    pub enable_maintenance_window: bool,
    pub maintenance_window_start: String,
    pub maintenance_window_end: String,
}

/// Cleanup frequency options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CleanupFrequency {
    Daily,
    Weekly,
    Monthly,
    Custom(String), // Cron expression
}

/// Compliance requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRequirements {
    pub gdpr_enabled: bool,
    pub ccpa_enabled: bool,
    pub sox_enabled: bool,
    pub pci_dss_enabled: bool,
    pub right_to_be_forgotten: bool,
    pub data_portability: bool,
    pub audit_trail_retention_years: u32,
}

/// Data retention manager
pub struct DataRetentionManager {
    config: DataRetentionConfig,
    pool: PgPool,
}

impl DataRetentionManager {
    /// Create new data retention manager
    pub fn new(config: DataRetentionConfig, pool: PgPool) -> Self {
        Self { config, pool }
    }

    /// Start retention management services
    pub async fn start(&self) -> Result<()> {
        if !self.config.enabled {
            info!("Data retention management is disabled");
            return Ok(());
        }

        info!("Starting data retention management services");

        // Start cleanup scheduler
        self.start_cleanup_scheduler().await?;

        // Start archival scheduler
        self.start_archival_scheduler().await?;

        // Start compliance monitoring
        self.start_compliance_monitoring().await?;

        Ok(())
    }

    /// Execute retention policies for all data types
    pub async fn execute_retention_policies(&self) -> Result<RetentionExecutionReport> {
        let mut report = RetentionExecutionReport {
            execution_id: Uuid::new_v4(),
            started_at: Utc::now(),
            completed_at: None,
            policies_executed: 0,
            records_processed: 0,
            records_deleted: 0,
            records_archived: 0,
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        for (policy_name, policy) in &self.config.retention_policies {
            info!("Executing retention policy: {}", policy_name);
            
            match self.execute_single_policy(policy).await {
                Ok(result) => {
                    report.policies_executed += 1;
                    report.records_processed += result.records_processed;
                    report.records_deleted += result.records_deleted;
                    report.records_archived += result.records_archived;
                    
                    if !result.warnings.is_empty() {
                        report.warnings.extend(result.warnings);
                    }
                }
                Err(e) => {
                    error!("Failed to execute retention policy {}: {}", policy_name, e);
                    report.errors.push(format!("Policy {}: {}", policy_name, e));
                }
            }
        }

        report.completed_at = Some(Utc::now());
        Ok(report)
    }

    /// Execute a single retention policy
    async fn execute_single_policy(&self, policy: &RetentionPolicy) -> Result<PolicyExecutionResult> {
        let cutoff_date = Utc::now() - Duration::days(policy.retention_period_days as i64);
        
        let mut result = PolicyExecutionResult {
            records_processed: 0,
            records_deleted: 0,
            records_archived: 0,
            warnings: Vec::new(),
        };

        match policy.data_type.as_str() {
            "market_ticks" => {
                result = self.cleanup_market_ticks(policy, cutoff_date).await?;
            }
            "trading_signals" => {
                result = self.cleanup_trading_signals(policy, cutoff_date).await?;
            }
            "audit_logs" => {
                result = self.cleanup_audit_logs(policy, cutoff_date).await?;
            }
            "user_sessions" => {
                result = self.cleanup_user_sessions(policy, cutoff_date).await?;
            }
            "temporary_data" => {
                result = self.cleanup_temporary_data(policy, cutoff_date).await?;
            }
            _ => {
                warn!("Unknown data type for retention policy: {}", policy.data_type);
                result.warnings.push(format!("Unknown data type: {}", policy.data_type));
            }
        }

        Ok(result)
    }

    /// Clean up market ticks data
    async fn cleanup_market_ticks(&self, policy: &RetentionPolicy, cutoff_date: DateTime<Utc>) -> Result<PolicyExecutionResult> {
        let mut result = PolicyExecutionResult::default();

        // Check if archival is needed first
        if let Some(archival_days) = policy.archival_period_days {
            let archive_cutoff = Utc::now() - Duration::days(archival_days as i64);
            
            // Archive old data before deletion
            let archive_query = r#"
                SELECT COUNT(*) as count
                FROM market_ticks 
                WHERE timestamp < $1 AND timestamp >= $2
                AND archived = false
            "#;
            
            let archive_count: i64 = sqlx::query_scalar(archive_query)
                .bind(archive_cutoff)
                .bind(cutoff_date)
                .fetch_one(&self.pool)
                .await?;

            if archive_count > 0 {
                info!("Archiving {} market tick records", archive_count);
                
                // Mark records for archival
                let archive_update = r#"
                    UPDATE market_ticks 
                    SET archived = true, archived_at = NOW()
                    WHERE timestamp < $1 AND timestamp >= $2
                    AND archived = false
                "#;
                
                let archived_rows = sqlx::query(archive_update)
                    .bind(archive_cutoff)
                    .bind(cutoff_date)
                    .execute(&self.pool)
                    .await?
                    .rows_affected();

                result.records_archived = archived_rows;
            }
        }

        // Delete old records
        match policy.deletion_method {
            DeletionMethod::SoftDelete => {
                let soft_delete_query = r#"
                    UPDATE market_ticks 
                    SET deleted = true, deleted_at = NOW()
                    WHERE timestamp < $1 AND deleted = false
                "#;
                
                let deleted_rows = sqlx::query(soft_delete_query)
                    .bind(cutoff_date)
                    .execute(&self.pool)
                    .await?
                    .rows_affected();

                result.records_deleted = deleted_rows;
            }
            DeletionMethod::HardDelete => {
                // Delete in batches to avoid long-running transactions
                let mut total_deleted = 0;
                loop {
                    let delete_query = r#"
                        DELETE FROM market_ticks 
                        WHERE id IN (
                            SELECT id FROM market_ticks 
                            WHERE timestamp < $1 
                            LIMIT $2
                        )
                    "#;
                    
                    let deleted_rows = sqlx::query(delete_query)
                        .bind(cutoff_date)
                        .bind(policy.batch_size as i64)
                        .execute(&self.pool)
                        .await?
                        .rows_affected();

                    total_deleted += deleted_rows;
                    
                    if deleted_rows == 0 {
                        break;
                    }

                    // Rate limiting
                    if total_deleted >= policy.max_deletion_rate_per_hour as u64 {
                        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
                        total_deleted = 0;
                    }
                }
                
                result.records_deleted = total_deleted;
            }
            DeletionMethod::Truncate => {
                // Use TimescaleDB chunk dropping for efficient deletion
                let drop_chunks_query = r#"
                    SELECT drop_chunks('market_ticks', $1)
                "#;
                
                let dropped_chunks = sqlx::query_scalar::<_, i64>(drop_chunks_query)
                    .bind(cutoff_date)
                    .fetch_one(&self.pool)
                    .await?;

                result.records_deleted = dropped_chunks as u64;
            }
            _ => {
                result.warnings.push("Unsupported deletion method for market_ticks".to_string());
            }
        }

        // Count processed records
        let count_query = "SELECT COUNT(*) FROM market_ticks WHERE timestamp < $1";
        result.records_processed = sqlx::query_scalar(count_query)
            .bind(cutoff_date)
            .fetch_one(&self.pool)
            .await?;

        Ok(result)
    }

    /// Clean up trading signals
    async fn cleanup_trading_signals(&self, policy: &RetentionPolicy, cutoff_date: DateTime<Utc>) -> Result<PolicyExecutionResult> {
        let mut result = PolicyExecutionResult::default();

        let delete_query = match policy.deletion_method {
            DeletionMethod::SoftDelete => {
                r#"
                UPDATE trading_signals 
                SET deleted = true, deleted_at = NOW()
                WHERE timestamp < $1 AND deleted = false
                "#
            }
            DeletionMethod::HardDelete => {
                r#"
                DELETE FROM trading_signals 
                WHERE timestamp < $1
                "#
            }
            _ => {
                result.warnings.push("Unsupported deletion method for trading_signals".to_string());
                return Ok(result);
            }
        };

        let deleted_rows = sqlx::query(delete_query)
            .bind(cutoff_date)
            .execute(&self.pool)
            .await?
            .rows_affected();

        result.records_deleted = deleted_rows;
        result.records_processed = deleted_rows;

        Ok(result)
    }

    /// Clean up audit logs
    async fn cleanup_audit_logs(&self, policy: &RetentionPolicy, cutoff_date: DateTime<Utc>) -> Result<PolicyExecutionResult> {
        let mut result = PolicyExecutionResult::default();

        // Audit logs should never be hard deleted for compliance
        if matches!(policy.deletion_method, DeletionMethod::HardDelete) {
            result.warnings.push("Hard deletion not allowed for audit logs - using archival instead".to_string());
        }

        // Archive audit logs
        let archive_query = r#"
            UPDATE audit_logs 
            SET archived = true, archived_at = NOW()
            WHERE created_at < $1 AND archived = false
        "#;

        let archived_rows = sqlx::query(archive_query)
            .bind(cutoff_date)
            .execute(&self.pool)
            .await?
            .rows_affected();

        result.records_archived = archived_rows;
        result.records_processed = archived_rows;

        Ok(result)
    }

    /// Clean up user sessions
    async fn cleanup_user_sessions(&self, policy: &RetentionPolicy, cutoff_date: DateTime<Utc>) -> Result<PolicyExecutionResult> {
        let mut result = PolicyExecutionResult::default();

        let delete_query = r#"
            DELETE FROM user_sessions 
            WHERE last_activity < $1 OR expires_at < NOW()
        "#;

        let deleted_rows = sqlx::query(delete_query)
            .bind(cutoff_date)
            .execute(&self.pool)
            .await?
            .rows_affected();

        result.records_deleted = deleted_rows;
        result.records_processed = deleted_rows;

        Ok(result)
    }

    /// Clean up temporary data
    async fn cleanup_temporary_data(&self, policy: &RetentionPolicy, cutoff_date: DateTime<Utc>) -> Result<PolicyExecutionResult> {
        let mut result = PolicyExecutionResult::default();

        // Clean up various temporary tables
        let temp_tables = vec![
            "temp_calculations",
            "temp_market_data",
            "temp_analysis_results",
            "cache_entries",
        ];

        for table in temp_tables {
            let delete_query = format!("DELETE FROM {} WHERE created_at < $1", table);
            
            match sqlx::query(&delete_query)
                .bind(cutoff_date)
                .execute(&self.pool)
                .await
            {
                Ok(query_result) => {
                    result.records_deleted += query_result.rows_affected();
                }
                Err(e) => {
                    // Table might not exist, log warning but continue
                    result.warnings.push(format!("Failed to clean table {}: {}", table, e));
                }
            }
        }

        result.records_processed = result.records_deleted;
        Ok(result)
    }

    /// Start cleanup scheduler
    async fn start_cleanup_scheduler(&self) -> Result<()> {
        // Implementation would start a background task that runs cleanup based on schedule
        info!("Cleanup scheduler started");
        Ok(())
    }

    /// Start archival scheduler
    async fn start_archival_scheduler(&self) -> Result<()> {
        // Implementation would start a background task for archival operations
        info!("Archival scheduler started");
        Ok(())
    }

    /// Start compliance monitoring
    async fn start_compliance_monitoring(&self) -> Result<()> {
        // Implementation would monitor compliance requirements
        info!("Compliance monitoring started");
        Ok(())
    }

    /// Handle right to be forgotten requests (GDPR)
    pub async fn handle_right_to_be_forgotten(&self, user_id: Uuid) -> Result<()> {
        if !self.config.compliance_requirements.right_to_be_forgotten {
            return Err("Right to be forgotten not enabled".into());
        }

        info!("Processing right to be forgotten request for user: {}", user_id);

        // Anonymize or delete user data across all tables
        let tables_to_clean = vec![
            ("users", "id"),
            ("user_sessions", "user_id"),
            ("audit_logs", "user_id"),
            ("trading_signals", "user_id"),
            ("orders", "user_id"),
        ];

        for (table, column) in tables_to_clean {
            let anonymize_query = format!(
                "UPDATE {} SET deleted = true, anonymized = true, anonymized_at = NOW() WHERE {} = $1",
                table, column
            );

            sqlx::query(&anonymize_query)
                .bind(user_id)
                .execute(&self.pool)
                .await?;
        }

        info!("Right to be forgotten request completed for user: {}", user_id);
        Ok(())
    }
}

impl Default for DataRetentionConfig {
    fn default() -> Self {
        let mut retention_policies = HashMap::new();
        
        // Market data - 2 years retention, 7 years archive
        retention_policies.insert("market_ticks".to_string(), RetentionPolicy {
            data_type: "market_ticks".to_string(),
            retention_period_days: 730, // 2 years
            archival_period_days: Some(2555), // 7 years
            deletion_method: DeletionMethod::Truncate,
            compliance_category: ComplianceCategory::Financial,
            enable_soft_delete: false,
            soft_delete_period_days: 30,
            batch_size: 10000,
            max_deletion_rate_per_hour: 100000,
        });

        // Trading signals - 1 year retention
        retention_policies.insert("trading_signals".to_string(), RetentionPolicy {
            data_type: "trading_signals".to_string(),
            retention_period_days: 365,
            archival_period_days: Some(1095), // 3 years
            deletion_method: DeletionMethod::SoftDelete,
            compliance_category: ComplianceCategory::Financial,
            enable_soft_delete: true,
            soft_delete_period_days: 90,
            batch_size: 5000,
            max_deletion_rate_per_hour: 50000,
        });

        // Audit logs - 7 years retention (compliance)
        retention_policies.insert("audit_logs".to_string(), RetentionPolicy {
            data_type: "audit_logs".to_string(),
            retention_period_days: 2555, // 7 years
            archival_period_days: None,
            deletion_method: DeletionMethod::Archive,
            compliance_category: ComplianceCategory::Audit,
            enable_soft_delete: false,
            soft_delete_period_days: 0,
            batch_size: 1000,
            max_deletion_rate_per_hour: 10000,
        });

        // User sessions - 30 days
        retention_policies.insert("user_sessions".to_string(), RetentionPolicy {
            data_type: "user_sessions".to_string(),
            retention_period_days: 30,
            archival_period_days: None,
            deletion_method: DeletionMethod::HardDelete,
            compliance_category: ComplianceCategory::Personal,
            enable_soft_delete: false,
            soft_delete_period_days: 0,
            batch_size: 1000,
            max_deletion_rate_per_hour: 10000,
        });

        // Temporary data - 7 days
        retention_policies.insert("temporary_data".to_string(), RetentionPolicy {
            data_type: "temporary_data".to_string(),
            retention_period_days: 7,
            archival_period_days: None,
            deletion_method: DeletionMethod::HardDelete,
            compliance_category: ComplianceCategory::Temporary,
            enable_soft_delete: false,
            soft_delete_period_days: 0,
            batch_size: 5000,
            max_deletion_rate_per_hour: 100000,
        });

        Self {
            enabled: true,
            retention_policies,
            archival_policies: HashMap::new(),
            cleanup_schedule: CleanupSchedule {
                frequency: CleanupFrequency::Daily,
                time_of_day: "02:00".to_string(),
                timezone: "UTC".to_string(),
                max_concurrent_operations: 2,
                enable_maintenance_window: true,
                maintenance_window_start: "01:00".to_string(),
                maintenance_window_end: "05:00".to_string(),
            },
            compliance_requirements: ComplianceRequirements {
                gdpr_enabled: true,
                ccpa_enabled: true,
                sox_enabled: true,
                pci_dss_enabled: false,
                right_to_be_forgotten: true,
                data_portability: true,
                audit_trail_retention_years: 7,
            },
            enable_audit_logging: true,
        }
    }
}

/// Retention execution report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionExecutionReport {
    pub execution_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub policies_executed: u32,
    pub records_processed: u64,
    pub records_deleted: u64,
    pub records_archived: u64,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Policy execution result
#[derive(Debug, Clone, Default)]
struct PolicyExecutionResult {
    pub records_processed: u64,
    pub records_deleted: u64,
    pub records_archived: u64,
    pub warnings: Vec<String>,
}
