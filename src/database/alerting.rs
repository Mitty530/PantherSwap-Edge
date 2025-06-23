// Database alerting system for PantherSwap Edge
// Provides alert management, notification routing, and escalation policies

use crate::utils::Result;
use super::health_monitor::{HealthAlert, AlertType, AlertSeverity};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use tracing::{info, warn, error};
use tokio::sync::mpsc;

/// Alert manager for database health monitoring
pub struct AlertManager {
    config: AlertConfig,
    alert_history: Vec<ProcessedAlert>,
    notification_channels: Vec<Box<dyn NotificationChannel>>,
    alert_sender: mpsc::UnboundedSender<HealthAlert>,
    alert_receiver: Option<mpsc::UnboundedReceiver<HealthAlert>>,
}

#[derive(Debug, Clone)]
pub struct AlertConfig {
    pub enable_notifications: bool,
    pub alert_cooldown_minutes: u64,
    pub max_alerts_per_hour: usize,
    pub escalation_enabled: bool,
    pub escalation_threshold_minutes: u64,
    pub alert_retention_hours: i64,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            enable_notifications: true,
            alert_cooldown_minutes: 5,
            max_alerts_per_hour: 20,
            escalation_enabled: true,
            escalation_threshold_minutes: 15,
            alert_retention_hours: 168, // 7 days
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedAlert {
    pub original_alert: HealthAlert,
    pub processed_at: DateTime<Utc>,
    pub notification_sent: bool,
    pub escalated: bool,
    pub resolved_at: Option<DateTime<Utc>>,
    pub acknowledgment: Option<AlertAcknowledgment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertAcknowledgment {
    pub acknowledged_by: String,
    pub acknowledged_at: DateTime<Utc>,
    pub notes: Option<String>,
}

/// Notification channel trait for different alert delivery methods
pub trait NotificationChannel: Send + Sync {
    fn send_alert(&self, alert: &ProcessedAlert) -> Result<()>;
    fn channel_type(&self) -> &str;
    fn is_available(&self) -> bool;
}

/// Console notification channel for development
pub struct ConsoleNotificationChannel;

impl NotificationChannel for ConsoleNotificationChannel {
    fn send_alert(&self, alert: &ProcessedAlert) -> Result<()> {
        let severity_emoji = match alert.original_alert.severity {
            AlertSeverity::Emergency => "🚨",
            AlertSeverity::Critical => "🔴",
            AlertSeverity::Warning => "⚠️",
            AlertSeverity::Info => "ℹ️",
        };

        println!(
            "{} [{}] {:?} Alert: {}",
            severity_emoji,
            alert.processed_at.format("%Y-%m-%d %H:%M:%S UTC"),
            alert.original_alert.severity,
            alert.original_alert.message
        );

        Ok(())
    }

    fn channel_type(&self) -> &str {
        "console"
    }

    fn is_available(&self) -> bool {
        true
    }
}

/// Log-based notification channel
pub struct LogNotificationChannel;

impl NotificationChannel for LogNotificationChannel {
    fn send_alert(&self, alert: &ProcessedAlert) -> Result<()> {
        match alert.original_alert.severity {
            AlertSeverity::Emergency | AlertSeverity::Critical => {
                error!(
                    alert_type = ?alert.original_alert.alert_type,
                    severity = ?alert.original_alert.severity,
                    metric_value = alert.original_alert.metric_value,
                    threshold = alert.original_alert.threshold,
                    "Database health alert: {}",
                    alert.original_alert.message
                );
            }
            AlertSeverity::Warning => {
                warn!(
                    alert_type = ?alert.original_alert.alert_type,
                    metric_value = alert.original_alert.metric_value,
                    threshold = alert.original_alert.threshold,
                    "Database health warning: {}",
                    alert.original_alert.message
                );
            }
            AlertSeverity::Info => {
                info!(
                    alert_type = ?alert.original_alert.alert_type,
                    "Database health info: {}",
                    alert.original_alert.message
                );
            }
        }

        Ok(())
    }

    fn channel_type(&self) -> &str {
        "log"
    }

    fn is_available(&self) -> bool {
        true
    }
}

/// Email notification channel (placeholder implementation)
pub struct EmailNotificationChannel {
    pub smtp_server: String,
    pub recipients: Vec<String>,
    pub enabled: bool,
}

impl EmailNotificationChannel {
    pub fn new(smtp_server: String, recipients: Vec<String>) -> Self {
        Self {
            smtp_server,
            recipients,
            enabled: false, // Disabled by default for MVP
        }
    }
}

impl NotificationChannel for EmailNotificationChannel {
    fn send_alert(&self, alert: &ProcessedAlert) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        // Placeholder for email implementation
        info!(
            "Would send email alert to {:?}: {}",
            self.recipients,
            alert.original_alert.message
        );

        Ok(())
    }

    fn channel_type(&self) -> &str {
        "email"
    }

    fn is_available(&self) -> bool {
        self.enabled && !self.smtp_server.is_empty() && !self.recipients.is_empty()
    }
}

/// Webhook notification channel
pub struct WebhookNotificationChannel {
    pub webhook_url: String,
    pub enabled: bool,
}

impl WebhookNotificationChannel {
    pub fn new(webhook_url: String) -> Self {
        Self {
            webhook_url,
            enabled: false, // Disabled by default for MVP
        }
    }
}

impl NotificationChannel for WebhookNotificationChannel {
    fn send_alert(&self, alert: &ProcessedAlert) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        // Placeholder for webhook implementation
        info!(
            "Would send webhook alert to {}: {}",
            self.webhook_url,
            alert.original_alert.message
        );

        Ok(())
    }

    fn channel_type(&self) -> &str {
        "webhook"
    }

    fn is_available(&self) -> bool {
        self.enabled && !self.webhook_url.is_empty()
    }
}

impl AlertManager {
    /// Create a new alert manager
    pub fn new(config: AlertConfig) -> Self {
        let (alert_sender, alert_receiver) = mpsc::unbounded_channel();

        Self {
            config,
            alert_history: Vec::new(),
            notification_channels: Vec::new(),
            alert_sender,
            alert_receiver: Some(alert_receiver),
        }
    }

    /// Create with default configuration and basic channels
    pub fn with_defaults() -> Self {
        let mut manager = Self::new(AlertConfig::default());
        
        // Add default notification channels
        manager.add_notification_channel(Box::new(ConsoleNotificationChannel));
        manager.add_notification_channel(Box::new(LogNotificationChannel));
        
        manager
    }

    /// Add a notification channel
    pub fn add_notification_channel(&mut self, channel: Box<dyn NotificationChannel>) {
        info!("Added notification channel: {}", channel.channel_type());
        self.notification_channels.push(channel);
    }

    /// Get alert sender for external use
    pub fn get_alert_sender(&self) -> mpsc::UnboundedSender<HealthAlert> {
        self.alert_sender.clone()
    }

    /// Start alert processing
    pub async fn start_processing(&mut self) -> Result<()> {
        let mut receiver = self.alert_receiver.take()
            .ok_or_else(|| crate::utils::PantherSwapError::Internal("Alert receiver already taken".to_string()))?;

        let config = self.config.clone();
        let channels = std::mem::take(&mut self.notification_channels);

        tokio::spawn(async move {
            let mut alert_history: Vec<ProcessedAlert> = Vec::new();
            let mut alert_counts: HashMap<String, usize> = HashMap::new();

            while let Some(alert) = receiver.recv().await {
                if let Err(e) = Self::process_alert(
                    alert,
                    &config,
                    &channels,
                    &mut alert_history,
                    &mut alert_counts,
                ).await {
                    error!("Error processing alert: {}", e);
                }
            }
        });

        info!("Started alert processing with {} notification channels", self.notification_channels.len());
        Ok(())
    }

    /// Process a single alert
    async fn process_alert(
        alert: HealthAlert,
        config: &AlertConfig,
        channels: &[Box<dyn NotificationChannel>],
        alert_history: &mut Vec<ProcessedAlert>,
        alert_counts: &mut HashMap<String, usize>,
    ) -> Result<()> {
        // Check if we should suppress this alert due to cooldown
        if Self::should_suppress_alert(&alert, config, alert_history) {
            return Ok(());
        }

        // Check rate limiting
        let alert_key = format!("{:?}_{:?}", alert.alert_type, alert.severity);
        let current_count = alert_counts.entry(alert_key.clone()).or_insert(0);
        
        if *current_count >= config.max_alerts_per_hour {
            warn!("Rate limiting alert: {} ({})", alert.message, alert_key);
            return Ok(());
        }

        *current_count += 1;

        // Create processed alert
        let processed_alert = ProcessedAlert {
            original_alert: alert,
            processed_at: Utc::now(),
            notification_sent: false,
            escalated: false,
            resolved_at: None,
            acknowledgment: None,
        };

        // Send notifications
        let mut notification_sent = false;
        for channel in channels {
            if channel.is_available() {
                match channel.send_alert(&processed_alert) {
                    Ok(()) => {
                        notification_sent = true;
                        info!("Alert sent via {}: {}", channel.channel_type(), processed_alert.original_alert.message);
                    }
                    Err(e) => {
                        error!("Failed to send alert via {}: {}", channel.channel_type(), e);
                    }
                }
            }
        }

        // Update processed alert
        let mut final_alert = processed_alert;
        final_alert.notification_sent = notification_sent;

        // Store in history
        alert_history.push(final_alert);

        // Cleanup old alerts
        let cutoff_time = Utc::now() - chrono::Duration::hours(config.alert_retention_hours);
        alert_history.retain(|a| a.processed_at > cutoff_time);

        // Reset hourly counters (simplified - in production use proper time windows)
        if alert_history.len() % 100 == 0 {
            alert_counts.clear();
        }

        Ok(())
    }

    /// Check if alert should be suppressed due to cooldown
    fn should_suppress_alert(
        alert: &HealthAlert,
        config: &AlertConfig,
        alert_history: &[ProcessedAlert],
    ) -> bool {
        if !config.enable_notifications {
            return true;
        }

        let cooldown_threshold = Utc::now() - chrono::Duration::minutes(config.alert_cooldown_minutes as i64);

        // Check for similar recent alerts
        alert_history.iter().any(|processed| {
            processed.processed_at > cooldown_threshold
                && processed.original_alert.alert_type == alert.alert_type
                && processed.original_alert.severity == alert.severity
        })
    }

    /// Get alert statistics
    pub fn get_alert_statistics(&self) -> AlertStatistics {
        let now = Utc::now();
        let last_hour = now - chrono::Duration::hours(1);
        let last_day = now - chrono::Duration::days(1);

        let alerts_last_hour = self.alert_history.iter()
            .filter(|a| a.processed_at > last_hour)
            .count();

        let alerts_last_day = self.alert_history.iter()
            .filter(|a| a.processed_at > last_day)
            .count();

        let critical_alerts_last_day = self.alert_history.iter()
            .filter(|a| {
                a.processed_at > last_day
                    && matches!(
                        a.original_alert.severity,
                        AlertSeverity::Critical | AlertSeverity::Emergency
                    )
            })
            .count();

        let unresolved_alerts = self.alert_history.iter()
            .filter(|a| a.resolved_at.is_none())
            .count();

        AlertStatistics {
            total_alerts: self.alert_history.len(),
            alerts_last_hour,
            alerts_last_day,
            critical_alerts_last_day,
            unresolved_alerts,
            notification_channels_count: self.notification_channels.len(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlertStatistics {
    pub total_alerts: usize,
    pub alerts_last_hour: usize,
    pub alerts_last_day: usize,
    pub critical_alerts_last_day: usize,
    pub unresolved_alerts: usize,
    pub notification_channels_count: usize,
}
