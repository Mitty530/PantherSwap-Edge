pub mod scanner;
pub mod encryption;
pub mod audit;
pub mod compliance;

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;

/// Security assessment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAssessment {
    pub assessment_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub overall_score: f64,
    pub risk_level: RiskLevel,
    pub vulnerabilities: Vec<Vulnerability>,
    pub compliance_status: ComplianceStatus,
    pub recommendations: Vec<SecurityRecommendation>,
}

/// Risk levels for security assessment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Vulnerability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: VulnerabilitySeverity,
    pub category: VulnerabilityCategory,
    pub affected_component: String,
    pub cve_id: Option<String>,
    pub cvss_score: Option<f64>,
    pub discovered_at: DateTime<Utc>,
    pub status: VulnerabilityStatus,
    pub remediation: Option<String>,
}

/// Vulnerability severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VulnerabilitySeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

/// Vulnerability categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VulnerabilityCategory {
    Authentication,
    Authorization,
    DataExposure,
    Injection,
    Cryptography,
    Configuration,
    Dependencies,
    Network,
    Application,
}

/// Vulnerability status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VulnerabilityStatus {
    Open,
    InProgress,
    Fixed,
    Accepted,
    FalsePositive,
}

/// Compliance status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStatus {
    pub frameworks: HashMap<String, ComplianceFrameworkStatus>,
    pub overall_compliance_score: f64,
    pub last_assessment: DateTime<Utc>,
}

/// Compliance framework status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFrameworkStatus {
    pub framework_name: String,
    pub version: String,
    pub compliance_percentage: f64,
    pub passed_controls: u32,
    pub total_controls: u32,
    pub failed_controls: Vec<String>,
}

/// Security recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRecommendation {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub priority: RecommendationPriority,
    pub category: String,
    pub estimated_effort: String,
    pub impact: String,
}

/// Recommendation priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enable_vulnerability_scanning: bool,
    pub scan_interval_hours: u32,
    pub enable_compliance_monitoring: bool,
    pub compliance_frameworks: Vec<String>,
    pub enable_security_headers: bool,
    pub enable_rate_limiting: bool,
    pub enable_audit_logging: bool,
    pub max_login_attempts: u32,
    pub session_timeout_minutes: u32,
    pub password_policy: PasswordPolicy,
    pub encryption_settings: EncryptionSettings,
}

/// Password policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    pub min_length: u32,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_special_chars: bool,
    pub max_age_days: u32,
    pub prevent_reuse_count: u32,
}

/// Encryption settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionSettings {
    pub algorithm: String,
    pub key_size: u32,
    pub enable_at_rest_encryption: bool,
    pub enable_in_transit_encryption: bool,
    pub key_rotation_days: u32,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_vulnerability_scanning: true,
            scan_interval_hours: 24,
            enable_compliance_monitoring: true,
            compliance_frameworks: vec![
                "SOC2".to_string(),
                "ISO27001".to_string(),
                "PCI-DSS".to_string(),
            ],
            enable_security_headers: true,
            enable_rate_limiting: true,
            enable_audit_logging: true,
            max_login_attempts: 5,
            session_timeout_minutes: 30,
            password_policy: PasswordPolicy {
                min_length: 12,
                require_uppercase: true,
                require_lowercase: true,
                require_numbers: true,
                require_special_chars: true,
                max_age_days: 90,
                prevent_reuse_count: 5,
            },
            encryption_settings: EncryptionSettings {
                algorithm: "AES-256-GCM".to_string(),
                key_size: 256,
                enable_at_rest_encryption: true,
                enable_in_transit_encryption: true,
                key_rotation_days: 30,
            },
        }
    }
}

/// Security manager for coordinating security operations
pub struct SecurityManager {
    config: SecurityConfig,
    scanner: scanner::VulnerabilityScanner,
    audit_logger: audit::AuditLogger,
    compliance_monitor: compliance::ComplianceMonitor,
}

impl SecurityManager {
    /// Create new security manager
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            scanner: scanner::VulnerabilityScanner::new(&config),
            audit_logger: audit::AuditLogger::new(&config),
            compliance_monitor: compliance::ComplianceMonitor::new(&config),
            config,
        }
    }

    /// Perform comprehensive security assessment
    pub async fn perform_security_assessment(&self) -> Result<SecurityAssessment, Box<dyn std::error::Error>> {
        let assessment_id = Uuid::new_v4();
        let timestamp = Utc::now();

        // Run vulnerability scan
        let vulnerabilities = self.scanner.scan_system().await?;

        // Check compliance status
        let compliance_status = self.compliance_monitor.check_compliance().await?;

        // Calculate overall security score
        let overall_score = self.calculate_security_score(&vulnerabilities, &compliance_status);

        // Determine risk level
        let risk_level = self.determine_risk_level(overall_score, &vulnerabilities);

        // Generate recommendations
        let recommendations = self.generate_recommendations(&vulnerabilities, &compliance_status);

        Ok(SecurityAssessment {
            assessment_id,
            timestamp,
            overall_score,
            risk_level,
            vulnerabilities,
            compliance_status,
            recommendations,
        })
    }

    /// Calculate overall security score
    fn calculate_security_score(&self, vulnerabilities: &[Vulnerability], compliance: &ComplianceStatus) -> f64 {
        let mut score = 100.0;

        // Deduct points for vulnerabilities
        for vuln in vulnerabilities {
            let deduction = match vuln.severity {
                VulnerabilitySeverity::Critical => 20.0,
                VulnerabilitySeverity::High => 10.0,
                VulnerabilitySeverity::Medium => 5.0,
                VulnerabilitySeverity::Low => 2.0,
                VulnerabilitySeverity::Info => 0.5,
            };
            score -= deduction;
        }

        // Factor in compliance score
        score = score * (compliance.overall_compliance_score / 100.0);

        score.max(0.0).min(100.0)
    }

    /// Determine risk level based on score and vulnerabilities
    fn determine_risk_level(&self, score: f64, vulnerabilities: &[Vulnerability]) -> RiskLevel {
        let has_critical = vulnerabilities.iter().any(|v| v.severity == VulnerabilitySeverity::Critical);
        let has_high = vulnerabilities.iter().any(|v| v.severity == VulnerabilitySeverity::High);

        if has_critical || score < 50.0 {
            RiskLevel::Critical
        } else if has_high || score < 70.0 {
            RiskLevel::High
        } else if score < 85.0 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        }
    }

    /// Generate security recommendations
    fn generate_recommendations(&self, vulnerabilities: &[Vulnerability], compliance: &ComplianceStatus) -> Vec<SecurityRecommendation> {
        let mut recommendations = Vec::new();

        // Add vulnerability-based recommendations
        for vuln in vulnerabilities {
            if let Some(remediation) = &vuln.remediation {
                recommendations.push(SecurityRecommendation {
                    id: Uuid::new_v4(),
                    title: format!("Fix {}", vuln.title),
                    description: remediation.clone(),
                    priority: match vuln.severity {
                        VulnerabilitySeverity::Critical => RecommendationPriority::Critical,
                        VulnerabilitySeverity::High => RecommendationPriority::High,
                        VulnerabilitySeverity::Medium => RecommendationPriority::Medium,
                        _ => RecommendationPriority::Low,
                    },
                    category: format!("{:?}", vuln.category),
                    estimated_effort: "TBD".to_string(),
                    impact: "Security improvement".to_string(),
                });
            }
        }

        // Add compliance-based recommendations
        for (framework, status) in &compliance.frameworks {
            if status.compliance_percentage < 100.0 {
                recommendations.push(SecurityRecommendation {
                    id: Uuid::new_v4(),
                    title: format!("Improve {} compliance", framework),
                    description: format!("Address failed controls: {:?}", status.failed_controls),
                    priority: if status.compliance_percentage < 80.0 {
                        RecommendationPriority::High
                    } else {
                        RecommendationPriority::Medium
                    },
                    category: "Compliance".to_string(),
                    estimated_effort: "Medium".to_string(),
                    impact: "Regulatory compliance".to_string(),
                });
            }
        }

        recommendations
    }

    /// Start continuous security monitoring
    pub async fn start_monitoring(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Start vulnerability scanning schedule
        if self.config.enable_vulnerability_scanning {
            self.scanner.start_scheduled_scanning(self.config.scan_interval_hours).await?;
        }

        // Start compliance monitoring
        if self.config.enable_compliance_monitoring {
            self.compliance_monitor.start_monitoring().await?;
        }

        // Start audit logging
        if self.config.enable_audit_logging {
            self.audit_logger.start_logging().await?;
        }

        Ok(())
    }
}
