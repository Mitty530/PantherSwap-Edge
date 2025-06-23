use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
    http::{StatusCode, HeaderMap},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use tracing::{info, warn, error, debug};
use jsonwebtoken::{decode, encode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use bcrypt::{hash, verify, DEFAULT_COST};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::api::middleware::auth::{AuthenticatedUser, UserRole, Permission, RateLimit};
use crate::api::responses::{ApiResponse, error_codes};

/// JWT Claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // Subject (user ID)
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub permissions: Vec<Permission>,
    pub exp: usize,   // Expiration time
    pub iat: usize,   // Issued at
    pub jti: String,  // JWT ID (for token revocation)
    pub session_id: String,
}

/// JWT Authentication Manager
#[derive(Clone)]
pub struct JwtAuthManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
    revoked_tokens: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
    active_sessions: Arc<RwLock<HashMap<String, SessionInfo>>>,
}

/// Session information for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub user_id: Uuid,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub expires_at: DateTime<Utc>,
}

/// Login request structure
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub remember_me: Option<bool>,
    pub mfa_token: Option<String>,
}

/// Login response structure
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserInfo,
}

/// User information for response
#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub permissions: Vec<Permission>,
    pub last_login: Option<DateTime<Utc>>,
}

/// Refresh token request
#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// Password change request
#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

impl JwtAuthManager {
    /// Create new JWT authentication manager
    pub fn new(secret: &str) -> Self {
        let encoding_key = EncodingKey::from_secret(secret.as_ref());
        let decoding_key = DecodingKey::from_secret(secret.as_ref());
        
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_audience(&["pantherswap-edge"]);
        validation.set_issuer(&["pantherswap-edge"]);
        
        Self {
            encoding_key,
            decoding_key,
            validation,
            revoked_tokens: Arc::new(RwLock::new(HashMap::new())),
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate JWT token for user
    pub async fn generate_token(
        &self,
        user: &AuthenticatedUser,
        session_duration: Duration,
    ) -> Result<(String, String), Box<dyn std::error::Error>> {
        let now = Utc::now();
        let exp = (now + session_duration).timestamp() as usize;
        let iat = now.timestamp() as usize;
        let jti = Uuid::new_v4().to_string();
        let session_id = Uuid::new_v4().to_string();

        let claims = Claims {
            sub: user.id.to_string(),
            username: user.name.clone(),
            email: format!("{}@pantherswap.com", user.name.to_lowercase().replace(" ", ".")),
            role: user.role.clone(),
            permissions: user.permissions.clone(),
            exp,
            iat,
            jti: jti.clone(),
            session_id: session_id.clone(),
        };

        let mut header = Header::new(Algorithm::HS256);
        header.kid = Some("pantherswap-edge-key".to_string());

        let token = encode(&header, &claims, &self.encoding_key)?;

        // Store session information
        let session_info = SessionInfo {
            user_id: user.id,
            username: user.name.clone(),
            created_at: now,
            last_activity: now,
            ip_address: None,
            user_agent: None,
            expires_at: now + session_duration,
        };

        self.active_sessions.write().await.insert(session_id.clone(), session_info);

        // Generate refresh token (longer duration)
        let refresh_exp = (now + Duration::days(30)).timestamp() as usize;
        let refresh_claims = Claims {
            sub: user.id.to_string(),
            username: user.name.clone(),
            email: format!("{}@pantherswap.com", user.name.to_lowercase().replace(" ", ".")),
            role: user.role.clone(),
            permissions: user.permissions.clone(),
            exp: refresh_exp,
            iat,
            jti: Uuid::new_v4().to_string(),
            session_id: session_id.clone(),
        };

        let refresh_token = encode(&header, &refresh_claims, &self.encoding_key)?;

        Ok((token, refresh_token))
    }

    /// Validate JWT token
    pub async fn validate_token(&self, token: &str) -> Result<Claims, Box<dyn std::error::Error>> {
        let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)?;
        let claims = token_data.claims;

        // Check if token is revoked
        let revoked_tokens = self.revoked_tokens.read().await;
        if revoked_tokens.contains_key(&claims.jti) {
            return Err("Token has been revoked".into());
        }

        // Check if session is still active
        let mut sessions = self.active_sessions.write().await;
        if let Some(session) = sessions.get_mut(&claims.session_id) {
            session.last_activity = Utc::now();
            
            // Check if session has expired
            if session.expires_at < Utc::now() {
                sessions.remove(&claims.session_id);
                return Err("Session has expired".into());
            }
        } else {
            return Err("Session not found".into());
        }

        Ok(claims)
    }

    /// Revoke token
    pub async fn revoke_token(&self, jti: &str) {
        let mut revoked_tokens = self.revoked_tokens.write().await;
        revoked_tokens.insert(jti.to_string(), Utc::now());
    }

    /// Revoke all tokens for a user
    pub async fn revoke_user_tokens(&self, user_id: Uuid) {
        let mut sessions = self.active_sessions.write().await;
        sessions.retain(|_, session| session.user_id != user_id);
    }

    /// Clean up expired tokens and sessions
    pub async fn cleanup_expired(&self) {
        let now = Utc::now();
        
        // Clean up expired revoked tokens (keep for 24 hours)
        let mut revoked_tokens = self.revoked_tokens.write().await;
        revoked_tokens.retain(|_, revoked_at| now.signed_duration_since(*revoked_at) < Duration::hours(24));

        // Clean up expired sessions
        let mut sessions = self.active_sessions.write().await;
        sessions.retain(|_, session| session.expires_at > now);
    }

    /// Get active sessions for a user
    pub async fn get_user_sessions(&self, user_id: Uuid) -> Vec<SessionInfo> {
        let sessions = self.active_sessions.read().await;
        sessions.values()
            .filter(|session| session.user_id == user_id)
            .cloned()
            .collect()
    }
}

/// JWT authentication middleware
pub async fn jwt_auth_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip authentication for health endpoints and login
    let path = request.uri().path();
    if crate::api::middleware::is_health_endpoint(path) || 
       path.starts_with("/auth/") {
        return Ok(next.run(request).await);
    }

    // Extract JWT token from headers
    let token = match extract_jwt_token(request.headers()) {
        Ok(token) => token,
        Err(status) => {
            debug!("JWT token extraction failed");
            return Err(status);
        }
    };

    // Get JWT manager from extensions (would be injected by app state)
    // For now, create a temporary one with a default secret
    let jwt_manager = JwtAuthManager::new("your-secret-key-here");

    // Validate token
    let claims = match jwt_manager.validate_token(&token).await {
        Ok(claims) => claims,
        Err(e) => {
            warn!("JWT validation failed: {}", e);
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Convert claims to AuthenticatedUser
    let authenticated_user = AuthenticatedUser {
        id: Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        api_key: "".to_string(), // JWT doesn't use API keys
        name: claims.username,
        role: claims.role,
        permissions: claims.permissions,
        rate_limit: RateLimit::default(),
        is_active: true,
        last_used: Some(Utc::now()),
    };

    // Add authenticated user to request extensions
    request.extensions_mut().insert(authenticated_user.clone());

    info!(
        user_id = %authenticated_user.id,
        user_name = %authenticated_user.name,
        role = ?authenticated_user.role,
        "User authenticated via JWT"
    );

    Ok(next.run(request).await)
}

/// Extract JWT token from request headers
fn extract_jwt_token(headers: &HeaderMap) -> Result<String, StatusCode> {
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                return Ok(auth_str[7..].to_string());
            }
        }
    }
    
    Err(StatusCode::UNAUTHORIZED)
}

/// Hash password using bcrypt
pub fn hash_password(password: &str) -> Result<String, Box<dyn std::error::Error>> {
    Ok(hash(password, DEFAULT_COST)?)
}

/// Verify password against hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, Box<dyn std::error::Error>> {
    Ok(verify(password, hash)?)
}
