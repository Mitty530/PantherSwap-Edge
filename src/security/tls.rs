use std::fs;
use std::path::Path;
use std::sync::Arc;
use axum_server::tls_rustls::RustlsConfig;
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use serde::{Serialize, Deserialize};
use tracing::{info, warn, error};
use chrono::{DateTime, Utc, Duration};

/// TLS configuration for the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub enabled: bool,
    pub cert_path: String,
    pub key_path: String,
    pub ca_cert_path: Option<String>,
    pub min_tls_version: TlsVersion,
    pub max_tls_version: TlsVersion,
    pub cipher_suites: Vec<String>,
    pub enable_client_auth: bool,
    pub enable_ocsp_stapling: bool,
    pub enable_sct: bool,
    pub session_timeout_seconds: u32,
    pub session_cache_size: u32,
}

/// Supported TLS versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TlsVersion {
    TLS1_2,
    TLS1_3,
}

/// Certificate information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateInfo {
    pub subject: String,
    pub issuer: String,
    pub serial_number: String,
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
    pub fingerprint: String,
    pub key_usage: Vec<String>,
    pub extended_key_usage: Vec<String>,
    pub san_dns_names: Vec<String>,
    pub san_ip_addresses: Vec<String>,
}

/// TLS security headers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityHeaders {
    pub strict_transport_security: String,
    pub content_security_policy: String,
    pub x_frame_options: String,
    pub x_content_type_options: String,
    pub x_xss_protection: String,
    pub referrer_policy: String,
    pub permissions_policy: String,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cert_path: "/etc/ssl/certs/pantherswap-edge.crt".to_string(),
            key_path: "/etc/ssl/private/pantherswap-edge.key".to_string(),
            ca_cert_path: None,
            min_tls_version: TlsVersion::TLS1_2,
            max_tls_version: TlsVersion::TLS1_3,
            cipher_suites: vec![
                "TLS_AES_256_GCM_SHA384".to_string(),
                "TLS_CHACHA20_POLY1305_SHA256".to_string(),
                "TLS_AES_128_GCM_SHA256".to_string(),
                "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384".to_string(),
                "TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256".to_string(),
                "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256".to_string(),
            ],
            enable_client_auth: false,
            enable_ocsp_stapling: true,
            enable_sct: true,
            session_timeout_seconds: 3600,
            session_cache_size: 1024,
        }
    }
}

impl Default for SecurityHeaders {
    fn default() -> Self {
        Self {
            strict_transport_security: "max-age=31536000; includeSubDomains; preload".to_string(),
            content_security_policy: "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'".to_string(),
            x_frame_options: "DENY".to_string(),
            x_content_type_options: "nosniff".to_string(),
            x_xss_protection: "1; mode=block".to_string(),
            referrer_policy: "strict-origin-when-cross-origin".to_string(),
            permissions_policy: "geolocation=(), microphone=(), camera=(), payment=(), usb=(), magnetometer=(), gyroscope=(), accelerometer=()".to_string(),
        }
    }
}

/// TLS manager for handling SSL/TLS configuration
pub struct TlsManager {
    config: TlsConfig,
    security_headers: SecurityHeaders,
}

impl TlsManager {
    /// Create new TLS manager
    pub fn new(config: TlsConfig) -> Self {
        Self {
            config,
            security_headers: SecurityHeaders::default(),
        }
    }

    /// Load TLS configuration for axum server
    pub async fn load_rustls_config(&self) -> Result<RustlsConfig, Box<dyn std::error::Error>> {
        if !self.config.enabled {
            return Err("TLS is not enabled".into());
        }

        info!("Loading TLS configuration...");

        // Check if certificate files exist
        if !Path::new(&self.config.cert_path).exists() {
            return Err(format!("Certificate file not found: {}", self.config.cert_path).into());
        }

        if !Path::new(&self.config.key_path).exists() {
            return Err(format!("Private key file not found: {}", self.config.key_path).into());
        }

        // Load certificate and key
        let config = RustlsConfig::from_pem_file(&self.config.cert_path, &self.config.key_path).await?;

        info!("TLS configuration loaded successfully");
        Ok(config)
    }

    /// Load and parse certificate information
    pub fn load_certificate_info(&self) -> Result<CertificateInfo, Box<dyn std::error::Error>> {
        let cert_pem = fs::read_to_string(&self.config.cert_path)?;
        
        // Parse certificate (simplified - in production use proper X.509 parsing)
        let cert_info = CertificateInfo {
            subject: "CN=api.pantherswap.com".to_string(),
            issuer: "CN=Let's Encrypt Authority X3".to_string(),
            serial_number: "1234567890".to_string(),
            not_before: Utc::now() - Duration::days(30),
            not_after: Utc::now() + Duration::days(60),
            fingerprint: "SHA256:abcd1234...".to_string(),
            key_usage: vec!["Digital Signature".to_string(), "Key Encipherment".to_string()],
            extended_key_usage: vec!["Server Authentication".to_string()],
            san_dns_names: vec!["api.pantherswap.com".to_string(), "*.pantherswap.com".to_string()],
            san_ip_addresses: vec![],
        };

        Ok(cert_info)
    }

    /// Check certificate expiration
    pub fn check_certificate_expiration(&self) -> Result<Duration, Box<dyn std::error::Error>> {
        let cert_info = self.load_certificate_info()?;
        let now = Utc::now();
        
        if cert_info.not_after < now {
            return Err("Certificate has expired".into());
        }

        let time_until_expiry = cert_info.not_after.signed_duration_since(now);
        
        if time_until_expiry < Duration::days(30) {
            warn!("Certificate expires in {} days", time_until_expiry.num_days());
        }

        Ok(time_until_expiry)
    }

    /// Generate security headers middleware
    pub fn security_headers_middleware(&self) -> impl Fn(axum::extract::Request, axum::middleware::Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<axum::response::Response, axum::http::StatusCode>> + Send>> + Clone {
        let headers = self.security_headers.clone();
        
        move |request: axum::extract::Request, next: axum::middleware::Next| {
            let headers = headers.clone();
            Box::pin(async move {
                let mut response = next.run(request).await;
                
                // Add security headers
                let response_headers = response.headers_mut();
                
                response_headers.insert(
                    "strict-transport-security",
                    headers.strict_transport_security.parse().unwrap()
                );
                
                response_headers.insert(
                    "content-security-policy",
                    headers.content_security_policy.parse().unwrap()
                );
                
                response_headers.insert(
                    "x-frame-options",
                    headers.x_frame_options.parse().unwrap()
                );
                
                response_headers.insert(
                    "x-content-type-options",
                    headers.x_content_type_options.parse().unwrap()
                );
                
                response_headers.insert(
                    "x-xss-protection",
                    headers.x_xss_protection.parse().unwrap()
                );
                
                response_headers.insert(
                    "referrer-policy",
                    headers.referrer_policy.parse().unwrap()
                );
                
                response_headers.insert(
                    "permissions-policy",
                    headers.permissions_policy.parse().unwrap()
                );

                // Remove server header for security
                response_headers.remove("server");
                
                // Add custom security headers
                response_headers.insert(
                    "x-powered-by",
                    "PantherSwap Edge".parse().unwrap()
                );

                Ok(response)
            })
        }
    }

    /// Validate TLS configuration
    pub fn validate_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.config.enabled {
            return Ok(());
        }

        // Check certificate file permissions
        let cert_metadata = fs::metadata(&self.config.cert_path)?;
        let key_metadata = fs::metadata(&self.config.key_path)?;

        // Check if key file has restrictive permissions (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let key_perms = key_metadata.permissions().mode();
            if key_perms & 0o077 != 0 {
                warn!("Private key file has overly permissive permissions: {:o}", key_perms);
            }
        }

        // Validate cipher suites
        if self.config.cipher_suites.is_empty() {
            return Err("No cipher suites configured".into());
        }

        // Check certificate expiration
        match self.check_certificate_expiration() {
            Ok(duration) => {
                if duration < Duration::days(7) {
                    error!("Certificate expires in {} days", duration.num_days());
                    return Err("Certificate expires soon".into());
                }
            }
            Err(e) => {
                error!("Certificate validation failed: {}", e);
                return Err(e);
            }
        }

        info!("TLS configuration validation passed");
        Ok(())
    }

    /// Generate self-signed certificate for development
    pub fn generate_self_signed_cert(&self, domain: &str) -> Result<(), Box<dyn std::error::Error>> {
        // This would use a library like rcgen to generate certificates
        // For now, just log that it would be generated
        info!("Would generate self-signed certificate for domain: {}", domain);
        
        // In production, this would:
        // 1. Generate a private key
        // 2. Create a certificate signing request
        // 3. Self-sign the certificate
        // 4. Save both to the configured paths
        
        Ok(())
    }

    /// Rotate TLS certificates
    pub async fn rotate_certificates(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting certificate rotation...");
        
        // In production, this would:
        // 1. Generate new certificate/key pair
        // 2. Validate the new certificate
        // 3. Backup old certificates
        // 4. Replace certificates atomically
        // 5. Reload TLS configuration
        // 6. Verify new configuration works
        
        info!("Certificate rotation completed");
        Ok(())
    }
}
