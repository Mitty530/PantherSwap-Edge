pub mod strategy;
pub mod scheduler;
pub mod recovery;
pub mod retention;

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use uuid::Uuid;
use std::path::PathBuf;

/// Backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    pub enabled: bool,
    pub backup_strategies: HashMap<String, BackupStrategy>,
    pub default_retention_policy: RetentionPolicy,
    pub backup_location: BackupLocation,
    pub encryption: EncryptionConfig,
    pub compression: CompressionConfig,
    pub verification: VerificationConfig,
    pub notification: NotificationConfig,
}

/// Backup strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupStrategy {
    pub name: String,
    pub backup_type: BackupType,
    pub schedule: BackupSchedule,
    pub retention_policy: RetentionPolicy,
    pub priority: BackupPriority,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub enable_compression: bool,
    pub enable_encryption: bool,
    pub enable_verification: bool,
}

/// Types of backups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupType {
    Full,
    Incremental,
    Differential,
    Snapshot,
    LogShipping,
    PointInTime,
}

/// Backup schedule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSchedule {
    pub frequency: BackupFrequency,
    pub time_of_day: Option<String>, // HH:MM format
    pub day_of_week: Option<u8>,     // 0-6, Sunday = 0
    pub day_of_month: Option<u8>,    // 1-31
    pub timezone: String,
    pub max_concurrent_backups: u8,
}

/// Backup frequency options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupFrequency {
    Continuous,
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Custom(String), // Cron expression
}

/// Backup priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupPriority {
    Critical,
    High,
    Medium,
    Low,
}

/// Retention policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub keep_daily: u32,
    pub keep_weekly: u32,
    pub keep_monthly: u32,
    pub keep_yearly: u32,
    pub max_age_days: u32,
    pub max_total_size_gb: Option<u64>,
    pub auto_cleanup: bool,
}

/// Backup location configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupLocation {
    pub primary: StorageLocation,
    pub secondary: Option<StorageLocation>,
    pub offsite: Option<StorageLocation>,
}

/// Storage location types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageLocation {
    Local { path: PathBuf },
    S3 { bucket: String, region: String, access_key: String, secret_key: String },
    GCS { bucket: String, credentials_path: String },
    Azure { container: String, account_name: String, account_key: String },
    SFTP { host: String, username: String, password: String, path: String },
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub enabled: bool,
    pub algorithm: String,
    pub key_derivation: String,
    pub key_rotation_days: u32,
}

/// Compression configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    pub enabled: bool,
    pub algorithm: CompressionAlgorithm,
    pub level: u8, // 1-9
}

/// Compression algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    Gzip,
    Bzip2,
    Lz4,
    Zstd,
}

/// Verification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationConfig {
    pub enabled: bool,
    pub checksum_algorithm: ChecksumAlgorithm,
    pub verify_after_backup: bool,
    pub periodic_verification: bool,
    pub verification_schedule: Option<BackupSchedule>,
}

/// Checksum algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChecksumAlgorithm {
    MD5,
    SHA1,
    SHA256,
    SHA512,
}

/// Notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub enabled: bool,
    pub notify_on_success: bool,
    pub notify_on_failure: bool,
    pub notify_on_verification_failure: bool,
    pub email_recipients: Vec<String>,
    pub slack_webhook: Option<String>,
    pub webhook_url: Option<String>,
}

/// Backup metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub id: Uuid,
    pub strategy_name: String,
    pub backup_type: BackupType,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: BackupStatus,
    pub size_bytes: u64,
    pub compressed_size_bytes: Option<u64>,
    pub checksum: Option<String>,
    pub location: StorageLocation,
    pub retention_until: DateTime<Utc>,
    pub verification_status: VerificationStatus,
    pub error_message: Option<String>,
}

/// Backup status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
    Expired,
}

/// Verification status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationStatus {
    NotVerified,
    Verified,
    VerificationFailed,
    VerificationPending,
}

/// Disaster recovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisasterRecoveryConfig {
    pub enabled: bool,
    pub rpo_minutes: u32,  // Recovery Point Objective
    pub rto_minutes: u32,  // Recovery Time Objective
    pub failover_strategy: FailoverStrategy,
    pub backup_sites: Vec<BackupSite>,
    pub automated_failover: bool,
    pub health_check_interval_seconds: u32,
    pub failover_triggers: Vec<FailoverTrigger>,
}

/// Failover strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailoverStrategy {
    Manual,
    Automatic,
    SemiAutomatic,
}

/// Backup site configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSite {
    pub name: String,
    pub location: String,
    pub priority: u8,
    pub capacity_gb: u64,
    pub network_bandwidth_mbps: u32,
    pub replication_lag_seconds: u32,
    pub status: SiteStatus,
}

/// Site status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SiteStatus {
    Active,
    Standby,
    Offline,
    Maintenance,
}

/// Failover triggers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailoverTrigger {
    DatabaseUnavailable,
    HighLatency(u32),
    LowThroughput(u32),
    HighErrorRate(f64),
    ManualTrigger,
    HealthCheckFailure,
}

impl Default for BackupConfig {
    fn default() -> Self {
        let mut strategies = HashMap::new();
        
        // Critical data - continuous backup
        strategies.insert("critical_data".to_string(), BackupStrategy {
            name: "Critical Trading Data".to_string(),
            backup_type: BackupType::PointInTime,
            schedule: BackupSchedule {
                frequency: BackupFrequency::Continuous,
                time_of_day: None,
                day_of_week: None,
                day_of_month: None,
                timezone: "UTC".to_string(),
                max_concurrent_backups: 2,
            },
            retention_policy: RetentionPolicy {
                keep_daily: 30,
                keep_weekly: 12,
                keep_monthly: 12,
                keep_yearly: 7,
                max_age_days: 2555, // 7 years
                max_total_size_gb: Some(1000),
                auto_cleanup: true,
            },
            priority: BackupPriority::Critical,
            include_patterns: vec![
                "market_ticks".to_string(),
                "orders".to_string(),
                "trades".to_string(),
                "positions".to_string(),
            ],
            exclude_patterns: vec![
                "temp_*".to_string(),
                "cache_*".to_string(),
            ],
            enable_compression: true,
            enable_encryption: true,
            enable_verification: true,
        });
        
        // Application data - daily backup
        strategies.insert("application_data".to_string(), BackupStrategy {
            name: "Application Data".to_string(),
            backup_type: BackupType::Full,
            schedule: BackupSchedule {
                frequency: BackupFrequency::Daily,
                time_of_day: Some("02:00".to_string()),
                day_of_week: None,
                day_of_month: None,
                timezone: "UTC".to_string(),
                max_concurrent_backups: 1,
            },
            retention_policy: RetentionPolicy {
                keep_daily: 7,
                keep_weekly: 4,
                keep_monthly: 6,
                keep_yearly: 2,
                max_age_days: 730, // 2 years
                max_total_size_gb: Some(500),
                auto_cleanup: true,
            },
            priority: BackupPriority::High,
            include_patterns: vec![
                "users".to_string(),
                "strategies".to_string(),
                "configurations".to_string(),
            ],
            exclude_patterns: vec![],
            enable_compression: true,
            enable_encryption: true,
            enable_verification: true,
        });

        Self {
            enabled: true,
            backup_strategies: strategies,
            default_retention_policy: RetentionPolicy {
                keep_daily: 7,
                keep_weekly: 4,
                keep_monthly: 6,
                keep_yearly: 2,
                max_age_days: 365,
                max_total_size_gb: Some(100),
                auto_cleanup: true,
            },
            backup_location: BackupLocation {
                primary: StorageLocation::Local {
                    path: PathBuf::from("/var/backups/pantherswap-edge"),
                },
                secondary: Some(StorageLocation::S3 {
                    bucket: "pantherswap-edge-backups".to_string(),
                    region: "us-east-1".to_string(),
                    access_key: "".to_string(),
                    secret_key: "".to_string(),
                }),
                offsite: None,
            },
            encryption: EncryptionConfig {
                enabled: true,
                algorithm: "AES-256-GCM".to_string(),
                key_derivation: "PBKDF2".to_string(),
                key_rotation_days: 90,
            },
            compression: CompressionConfig {
                enabled: true,
                algorithm: CompressionAlgorithm::Zstd,
                level: 6,
            },
            verification: VerificationConfig {
                enabled: true,
                checksum_algorithm: ChecksumAlgorithm::SHA256,
                verify_after_backup: true,
                periodic_verification: true,
                verification_schedule: Some(BackupSchedule {
                    frequency: BackupFrequency::Weekly,
                    time_of_day: Some("03:00".to_string()),
                    day_of_week: Some(0), // Sunday
                    day_of_month: None,
                    timezone: "UTC".to_string(),
                    max_concurrent_backups: 1,
                }),
            },
            notification: NotificationConfig {
                enabled: true,
                notify_on_success: false,
                notify_on_failure: true,
                notify_on_verification_failure: true,
                email_recipients: vec!["admin@pantherswap.com".to_string()],
                slack_webhook: None,
                webhook_url: None,
            },
        }
    }
}

/// Backup manager for coordinating all backup operations
pub struct BackupManager {
    config: BackupConfig,
    scheduler: scheduler::BackupScheduler,
    recovery: recovery::RecoveryManager,
    retention: retention::RetentionManager,
}

impl BackupManager {
    /// Create new backup manager
    pub fn new(config: BackupConfig) -> Self {
        Self {
            scheduler: scheduler::BackupScheduler::new(&config),
            recovery: recovery::RecoveryManager::new(&config),
            retention: retention::RetentionManager::new(&config),
            config,
        }
    }

    /// Start backup services
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.config.enabled {
            return Ok(());
        }

        // Start backup scheduler
        self.scheduler.start().await?;

        // Start retention manager
        self.retention.start().await?;

        Ok(())
    }

    /// Perform immediate backup
    pub async fn backup_now(&self, strategy_name: &str) -> Result<BackupMetadata, Box<dyn std::error::Error>> {
        self.scheduler.execute_backup(strategy_name).await
    }

    /// List available backups
    pub async fn list_backups(&self, strategy_name: Option<&str>) -> Result<Vec<BackupMetadata>, Box<dyn std::error::Error>> {
        self.scheduler.list_backups(strategy_name).await
    }

    /// Restore from backup
    pub async fn restore(&self, backup_id: Uuid, target_location: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        self.recovery.restore_from_backup(backup_id, target_location).await
    }

    /// Test disaster recovery procedures
    pub async fn test_disaster_recovery(&self) -> Result<DisasterRecoveryTestResult, Box<dyn std::error::Error>> {
        self.recovery.test_disaster_recovery().await
    }

    /// Get backup status
    pub async fn get_status(&self) -> Result<BackupSystemStatus, Box<dyn std::error::Error>> {
        Ok(BackupSystemStatus {
            enabled: self.config.enabled,
            active_backups: self.scheduler.get_active_backup_count().await,
            last_successful_backup: self.scheduler.get_last_successful_backup().await,
            total_backup_size_gb: self.scheduler.get_total_backup_size().await,
            available_space_gb: self.scheduler.get_available_space().await,
            health_status: self.check_health().await,
        })
    }

    /// Check backup system health
    async fn check_health(&self) -> BackupHealthStatus {
        // This would check various health indicators
        BackupHealthStatus::Healthy
    }
}

/// Backup system status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSystemStatus {
    pub enabled: bool,
    pub active_backups: u32,
    pub last_successful_backup: Option<DateTime<Utc>>,
    pub total_backup_size_gb: f64,
    pub available_space_gb: f64,
    pub health_status: BackupHealthStatus,
}

/// Backup health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupHealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

/// Disaster recovery test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisasterRecoveryTestResult {
    pub test_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub success: bool,
    pub rpo_achieved_minutes: u32,
    pub rto_achieved_minutes: u32,
    pub issues_found: Vec<String>,
    pub recommendations: Vec<String>,
}
