use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
    http::{StatusCode, HeaderMap},
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use tracing::{warn, debug};

use crate::api::middleware::auth::AuthenticatedUser;
use crate::api::responses::{ApiResponse, error_codes};

/// Rate limiter state
#[derive(Debug, Clone)]
pub struct RateLimiterState {
    pub requests: Vec<DateTime<Utc>>,
    pub last_reset: DateTime<Utc>,
}

impl Default for RateLimiterState {
    fn default() -> Self {
        Self {
            requests: Vec::new(),
            last_reset: Utc::now(),
        }
    }
}

/// Global rate limiter
pub struct RateLimiter {
    // User-specific rate limiting
    user_states: Arc<RwLock<HashMap<String, RateLimiterState>>>,
    // IP-based rate limiting for unauthenticated requests
    ip_states: Arc<RwLock<HashMap<String, RateLimiterState>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            user_states: Arc::new(RwLock::new(HashMap::new())),
            ip_states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if request is allowed for authenticated user
    pub async fn check_user_rate_limit(
        &self,
        user: &AuthenticatedUser,
    ) -> Result<(), RateLimitError> {
        let mut states = self.user_states.write().await;
        let state = states.entry(user.api_key.clone()).or_default();
        
        let now = Utc::now();
        
        // Clean old requests (older than 1 hour)
        state.requests.retain(|&timestamp| {
            now.signed_duration_since(timestamp) < Duration::hours(1)
        });
        
        // Check hourly limit
        if state.requests.len() >= user.rate_limit.requests_per_hour as usize {
            return Err(RateLimitError::HourlyLimitExceeded);
        }
        
        // Check minute limit
        let minute_ago = now - Duration::minutes(1);
        let requests_last_minute = state.requests.iter()
            .filter(|&&timestamp| timestamp > minute_ago)
            .count();
            
        if requests_last_minute >= user.rate_limit.requests_per_minute as usize {
            return Err(RateLimitError::MinuteLimitExceeded);
        }
        
        // Check burst limit (last 10 seconds)
        let ten_seconds_ago = now - Duration::seconds(10);
        let requests_last_10_seconds = state.requests.iter()
            .filter(|&&timestamp| timestamp > ten_seconds_ago)
            .count();
            
        if requests_last_10_seconds >= user.rate_limit.burst_limit as usize {
            return Err(RateLimitError::BurstLimitExceeded);
        }
        
        // Add current request
        state.requests.push(now);
        
        debug!(
            user_id = %user.id,
            requests_last_minute = requests_last_minute,
            requests_last_hour = state.requests.len(),
            "Rate limit check passed"
        );
        
        Ok(())
    }

    /// Check rate limit for IP address (unauthenticated requests)
    pub async fn check_ip_rate_limit(&self, ip: &str) -> Result<(), RateLimitError> {
        let mut states = self.ip_states.write().await;
        let state = states.entry(ip.to_string()).or_default();
        
        let now = Utc::now();
        
        // Clean old requests (older than 1 hour)
        state.requests.retain(|&timestamp| {
            now.signed_duration_since(timestamp) < Duration::hours(1)
        });
        
        // More restrictive limits for unauthenticated requests
        const IP_REQUESTS_PER_MINUTE: usize = 10;
        const IP_REQUESTS_PER_HOUR: usize = 100;
        const IP_BURST_LIMIT: usize = 3;
        
        // Check hourly limit
        if state.requests.len() >= IP_REQUESTS_PER_HOUR {
            return Err(RateLimitError::HourlyLimitExceeded);
        }
        
        // Check minute limit
        let minute_ago = now - Duration::minutes(1);
        let requests_last_minute = state.requests.iter()
            .filter(|&&timestamp| timestamp > minute_ago)
            .count();
            
        if requests_last_minute >= IP_REQUESTS_PER_MINUTE {
            return Err(RateLimitError::MinuteLimitExceeded);
        }
        
        // Check burst limit
        let ten_seconds_ago = now - Duration::seconds(10);
        let requests_last_10_seconds = state.requests.iter()
            .filter(|&&timestamp| timestamp > ten_seconds_ago)
            .count();
            
        if requests_last_10_seconds >= IP_BURST_LIMIT {
            return Err(RateLimitError::BurstLimitExceeded);
        }
        
        // Add current request
        state.requests.push(now);
        
        debug!(
            ip = ip,
            requests_last_minute = requests_last_minute,
            requests_last_hour = state.requests.len(),
            "IP rate limit check passed"
        );
        
        Ok(())
    }

    /// Clean up old state entries periodically
    pub async fn cleanup_old_states(&self) {
        let cutoff_time = Utc::now() - Duration::hours(2);
        
        // Clean user states
        {
            let mut user_states = self.user_states.write().await;
            user_states.retain(|_, state| {
                state.last_reset > cutoff_time || !state.requests.is_empty()
            });
        }
        
        // Clean IP states
        {
            let mut ip_states = self.ip_states.write().await;
            ip_states.retain(|_, state| {
                state.last_reset > cutoff_time || !state.requests.is_empty()
            });
        }
    }
}

/// Rate limit errors
#[derive(Debug)]
pub enum RateLimitError {
    MinuteLimitExceeded,
    HourlyLimitExceeded,
    BurstLimitExceeded,
}

impl std::fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RateLimitError::MinuteLimitExceeded => write!(f, "Rate limit exceeded: too many requests per minute"),
            RateLimitError::HourlyLimitExceeded => write!(f, "Rate limit exceeded: too many requests per hour"),
            RateLimitError::BurstLimitExceeded => write!(f, "Rate limit exceeded: burst limit exceeded"),
        }
    }
}

impl std::error::Error for RateLimitError {}

/// Global rate limiter instance
static RATE_LIMITER: std::sync::OnceLock<RateLimiter> = std::sync::OnceLock::new();

/// Get global rate limiter instance
pub fn get_rate_limiter() -> &'static RateLimiter {
    RATE_LIMITER.get_or_init(|| RateLimiter::new())
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip rate limiting for health endpoints
    if crate::api::middleware::is_health_endpoint(request.uri().path()) {
        return Ok(next.run(request).await);
    }

    let rate_limiter = get_rate_limiter();

    // Check if user is authenticated
    if let Some(user) = request.extensions().get::<AuthenticatedUser>() {
        // Use user-specific rate limiting
        match rate_limiter.check_user_rate_limit(user).await {
            Ok(()) => {
                debug!(user_id = %user.id, "Rate limit check passed for user");
            }
            Err(error) => {
                warn!(
                    user_id = %user.id,
                    error = %error,
                    "Rate limit exceeded for user"
                );
                return Err(StatusCode::TOO_MANY_REQUESTS);
            }
        }
    } else {
        // Use IP-based rate limiting for unauthenticated requests
        let ip = extract_ip_address(request.headers());
        match rate_limiter.check_ip_rate_limit(&ip).await {
            Ok(()) => {
                debug!(ip = ip, "Rate limit check passed for IP");
            }
            Err(error) => {
                warn!(
                    ip = ip,
                    error = %error,
                    "Rate limit exceeded for IP"
                );
                return Err(StatusCode::TOO_MANY_REQUESTS);
            }
        }
    }

    Ok(next.run(request).await)
}

/// Extract IP address from request headers
fn extract_ip_address(headers: &HeaderMap) -> String {
    // Try various headers in order of preference
    let ip_headers = [
        "x-forwarded-for",
        "x-real-ip",
        "cf-connecting-ip",
        "x-client-ip",
    ];
    
    for header_name in &ip_headers {
        if let Some(header_value) = headers.get(*header_name) {
            if let Ok(ip_str) = header_value.to_str() {
                // Take the first IP if there are multiple (comma-separated)
                let ip = ip_str.split(',').next().unwrap_or("").trim();
                if !ip.is_empty() {
                    return ip.to_string();
                }
            }
        }
    }
    
    "unknown".to_string()
}

/// Start background task to clean up old rate limit states
pub fn start_cleanup_task() {
    tokio::spawn(async {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(300)); // 5 minutes
        
        loop {
            interval.tick().await;
            let rate_limiter = get_rate_limiter();
            rate_limiter.cleanup_old_states().await;
        }
    });
}
