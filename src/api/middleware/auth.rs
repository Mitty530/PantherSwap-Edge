use axum::{
    extract::{Request, Extension},
    middleware::Next,
    response::Response,
    http::{StatusCode, HeaderMap},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tracing::{info, warn, error};

use crate::api::responses::{ApiResponse, error_codes};
use crate::database::Database;

/// Authenticated user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub api_key: String,
    pub name: String,
    pub role: UserRole,
    pub permissions: Vec<Permission>,
    pub rate_limit: RateLimit,
    pub is_active: bool,
    pub last_used: Option<DateTime<Utc>>,
}

/// User roles with different access levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    Admin,
    Trader,
    Analyst,
    ReadOnly,
}

/// Specific permissions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Permission {
    ReadMarketData,
    WriteMarketData,
    ReadTradingSignals,
    WriteTradingSignals,
    ReadPortfolio,
    WritePortfolio,
    ReadStrategies,
    WriteStrategies,
    ReadTradingData,
    Trade,
    SystemAdmin,
    UserManagement,
}

/// Rate limiting configuration per user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub burst_limit: u32,
}

impl Default for RateLimit {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            requests_per_hour: 1000,
            burst_limit: 10,
        }
    }
}

impl UserRole {
    /// Get default permissions for a role
    pub fn default_permissions(&self) -> Vec<Permission> {
        match self {
            UserRole::Admin => vec![
                Permission::ReadMarketData,
                Permission::WriteMarketData,
                Permission::ReadTradingSignals,
                Permission::WriteTradingSignals,
                Permission::ReadPortfolio,
                Permission::WritePortfolio,
                Permission::ReadStrategies,
                Permission::WriteStrategies,
                Permission::ReadTradingData,
                Permission::Trade,
                Permission::SystemAdmin,
                Permission::UserManagement,
            ],
            UserRole::Trader => vec![
                Permission::ReadMarketData,
                Permission::ReadTradingSignals,
                Permission::WriteTradingSignals,
                Permission::ReadPortfolio,
                Permission::WritePortfolio,
                Permission::ReadStrategies,
                Permission::WriteStrategies,
                Permission::ReadTradingData,
                Permission::Trade,
            ],
            UserRole::Analyst => vec![
                Permission::ReadMarketData,
                Permission::ReadTradingSignals,
                Permission::ReadPortfolio,
                Permission::ReadStrategies,
                Permission::ReadTradingData,
            ],
            UserRole::ReadOnly => vec![
                Permission::ReadMarketData,
                Permission::ReadTradingSignals,
                Permission::ReadPortfolio,
                Permission::ReadStrategies,
                Permission::ReadTradingData,
            ],
        }
    }

    /// Get default rate limit for a role
    pub fn default_rate_limit(&self) -> RateLimit {
        match self {
            UserRole::Admin => RateLimit {
                requests_per_minute: 300,
                requests_per_hour: 10000,
                burst_limit: 50,
            },
            UserRole::Trader => RateLimit {
                requests_per_minute: 120,
                requests_per_hour: 5000,
                burst_limit: 20,
            },
            UserRole::Analyst => RateLimit {
                requests_per_minute: 60,
                requests_per_hour: 2000,
                burst_limit: 10,
            },
            UserRole::ReadOnly => RateLimit {
                requests_per_minute: 30,
                requests_per_hour: 1000,
                burst_limit: 5,
            },
        }
    }
}

/// Authentication middleware
pub async fn auth_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip authentication for health endpoints
    if crate::api::middleware::is_health_endpoint(request.uri().path()) {
        return Ok(next.run(request).await);
    }

    // Extract API key from headers
    let api_key = match extract_api_key(request.headers()) {
        Ok(key) => key,
        Err(status) => {
            let error_response = ApiResponse::<()>::error(
                error_codes::UNAUTHORIZED.to_string(),
                "Missing or invalid API key".to_string(),
            );
            return Err(status);
        }
    };

    // Get database from extensions
    let database = request.extensions()
        .get::<Database>()
        .cloned()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // Authenticate user
    let authenticated_user = match authenticate_api_key(&api_key, &database).await {
        Ok(user) => user,
        Err(_) => {
            warn!("Authentication failed for API key: {}", mask_api_key(&api_key));
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Check if user is active
    if !authenticated_user.is_active {
        warn!("Inactive user attempted access: {}", authenticated_user.name);
        return Err(StatusCode::FORBIDDEN);
    }

    // Add authenticated user to request extensions
    request.extensions_mut().insert(authenticated_user.clone());

    info!(
        user_id = %authenticated_user.id,
        user_name = %authenticated_user.name,
        role = ?authenticated_user.role,
        "User authenticated successfully"
    );

    Ok(next.run(request).await)
}

/// Extract API key from request headers
fn extract_api_key(headers: &HeaderMap) -> Result<String, StatusCode> {
    // Try Authorization header first (Bearer token)
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                return Ok(auth_str[7..].to_string());
            }
        }
    }

    // Try X-API-Key header
    if let Some(api_key_header) = headers.get("x-api-key") {
        if let Ok(api_key) = api_key_header.to_str() {
            return Ok(api_key.to_string());
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

/// Authenticate API key and return user information
async fn authenticate_api_key(api_key: &str, database: &Database) -> Result<AuthenticatedUser, Box<dyn std::error::Error>> {
    // For now, create a mock user since we don't have user management in database yet
    // In production, this would query the database for the API key
    
    // Mock authentication - in production this would be a database lookup
    if api_key == "demo-admin-key" {
        Ok(AuthenticatedUser {
            id: Uuid::new_v4(),
            api_key: api_key.to_string(),
            name: "Demo Admin".to_string(),
            role: UserRole::Admin,
            permissions: UserRole::Admin.default_permissions(),
            rate_limit: UserRole::Admin.default_rate_limit(),
            is_active: true,
            last_used: Some(Utc::now()),
        })
    } else if api_key == "demo-trader-key" {
        Ok(AuthenticatedUser {
            id: Uuid::new_v4(),
            api_key: api_key.to_string(),
            name: "Demo Trader".to_string(),
            role: UserRole::Trader,
            permissions: UserRole::Trader.default_permissions(),
            rate_limit: UserRole::Trader.default_rate_limit(),
            is_active: true,
            last_used: Some(Utc::now()),
        })
    } else if api_key == "demo-readonly-key" {
        Ok(AuthenticatedUser {
            id: Uuid::new_v4(),
            api_key: api_key.to_string(),
            name: "Demo ReadOnly".to_string(),
            role: UserRole::ReadOnly,
            permissions: UserRole::ReadOnly.default_permissions(),
            rate_limit: UserRole::ReadOnly.default_rate_limit(),
            is_active: true,
            last_used: Some(Utc::now()),
        })
    } else {
        Err("Invalid API key".into())
    }
}

/// Mask API key for logging (show only first 4 and last 4 characters)
fn mask_api_key(api_key: &str) -> String {
    if api_key.len() <= 8 {
        "*".repeat(api_key.len())
    } else {
        format!("{}****{}", &api_key[..4], &api_key[api_key.len()-4..])
    }
}

/// Check if user has specific permission
pub fn has_permission(user: &AuthenticatedUser, permission: &Permission) -> bool {
    user.permissions.contains(permission)
}

/// Permission checking middleware
pub fn require_permission(
    permission: Permission,
) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>> + Clone {
    move |request: Request, next: Next| {
        let perm = permission.clone();
        Box::pin(async move {
            let user = request.extensions()
                .get::<AuthenticatedUser>()
                .ok_or(StatusCode::UNAUTHORIZED)?;

            if !has_permission(user, &perm) {
                warn!(
                    user_id = %user.id,
                    required_permission = ?perm,
                    "Permission denied"
                );
                return Err(StatusCode::FORBIDDEN);
            }

            Ok(next.run(request).await)
        })
    }
}
