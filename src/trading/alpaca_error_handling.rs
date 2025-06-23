use crate::utils::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::{sleep, Instant};
use tracing::{error, warn, info, debug};

/// Enhanced error handling and retry mechanism for Alpaca API
#[derive(Debug, Clone)]
pub struct AlpacaErrorHandler {
    retry_config: RetryConfig,
    circuit_breaker: Arc<Mutex<CircuitBreaker>>,
    error_tracker: Arc<Mutex<ErrorTracker>>,
    rate_limiter: Arc<Mutex<EnhancedRateLimiter>>,
}

/// Retry configuration with exponential backoff
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
    pub jitter_factor: f64,
    pub retry_on_rate_limit: bool,
    pub retry_on_network_error: bool,
    pub retry_on_server_error: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 5,
            initial_delay_ms: 100,
            max_delay_ms: 30000, // 30 seconds max
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
            retry_on_rate_limit: true,
            retry_on_network_error: true,
            retry_on_server_error: true,
        }
    }
}

/// Circuit breaker pattern for API protection
#[derive(Debug)]
pub struct CircuitBreaker {
    state: CircuitBreakerState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<DateTime<Utc>>,
    failure_threshold: u32,
    recovery_timeout_ms: u64,
    half_open_max_calls: u32,
    half_open_calls: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    Closed,   // Normal operation
    Open,     // Failing, reject calls
    HalfOpen, // Testing recovery
}

/// Error tracking and analytics
#[derive(Debug)]
pub struct ErrorTracker {
    recent_errors: VecDeque<AlpacaError>,
    error_counts: std::collections::HashMap<AlpacaErrorType, u32>,
    window_size: usize,
    last_reset: DateTime<Utc>,
    reset_interval_hours: u32,
}

/// Enhanced rate limiter with burst handling
#[derive(Debug)]
pub struct EnhancedRateLimiter {
    requests_made: u32,
    window_start: DateTime<Utc>,
    max_requests_per_minute: u32,
    burst_allowance: u32,
    burst_used: u32,
    burst_reset_time: DateTime<Utc>,
    adaptive_delay: bool,
    current_delay_ms: u64,
}

/// Alpaca-specific error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlpacaError {
    pub error_type: AlpacaErrorType,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub retry_after: Option<u64>,
    pub request_id: Option<String>,
    pub status_code: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlpacaErrorType {
    RateLimit,
    NetworkError,
    ServerError,
    AuthenticationError,
    ValidationError,
    InsufficientFunds,
    MarketClosed,
    OrderRejected,
    PositionNotFound,
    UnknownError,
}

impl AlpacaErrorHandler {
    pub fn new(retry_config: RetryConfig) -> Self {
        Self {
            retry_config,
            circuit_breaker: Arc::new(Mutex::new(CircuitBreaker::new())),
            error_tracker: Arc::new(Mutex::new(ErrorTracker::new())),
            rate_limiter: Arc::new(Mutex::new(EnhancedRateLimiter::new(200))), // 200 requests per minute default
        }
    }

    /// Execute a request with comprehensive error handling and retries
    pub async fn execute_with_retry<F, T, E>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<T, E>> + Send>> + Send + Sync,
        E: Into<AlpacaError> + Send + Sync,
        T: Send,
    {
        // Check circuit breaker
        {
            let mut circuit_breaker = self.circuit_breaker.lock().await;
            if !circuit_breaker.can_execute() {
                return Err(crate::utils::PantherSwapError::trading(
                    "Circuit breaker is open - API calls are temporarily blocked".to_string()
                ));
            }
        }

        // Apply rate limiting
        {
            let mut rate_limiter = self.rate_limiter.lock().await;
            rate_limiter.wait_if_needed().await?;
        }

        let mut attempt = 0;
        let mut last_error: Option<AlpacaError> = None;

        while attempt <= self.retry_config.max_retries {
            let start_time = Instant::now();

            match operation().await {
                Ok(result) => {
                    // Success - update circuit breaker
                    {
                        let mut circuit_breaker = self.circuit_breaker.lock().await;
                        circuit_breaker.record_success();
                    }

                    // Log successful retry if this wasn't the first attempt
                    if attempt > 0 {
                        info!("Operation succeeded after {} retries", attempt);
                    }

                    return Ok(result);
                }
                Err(error) => {
                    let alpaca_error = error.into();
                    last_error = Some(alpaca_error.clone());

                    // Track the error
                    {
                        let mut error_tracker = self.error_tracker.lock().await;
                        error_tracker.record_error(alpaca_error.clone());
                    }

                    // Update circuit breaker
                    {
                        let mut circuit_breaker = self.circuit_breaker.lock().await;
                        circuit_breaker.record_failure();
                    }

                    // Check if we should retry
                    if attempt >= self.retry_config.max_retries || !self.should_retry(&alpaca_error) {
                        break;
                    }

                    // Calculate delay with exponential backoff and jitter
                    let delay = self.calculate_retry_delay(attempt, &alpaca_error);
                    
                    warn!("Operation failed (attempt {}/{}): {}. Retrying in {}ms", 
                        attempt + 1, self.retry_config.max_retries + 1, 
                        alpaca_error.message, delay);

                    sleep(Duration::from_millis(delay)).await;
                    attempt += 1;
                }
            }
        }

        // All retries exhausted
        let final_error = last_error.unwrap_or_else(|| AlpacaError {
            error_type: AlpacaErrorType::UnknownError,
            message: "Unknown error occurred".to_string(),
            timestamp: Utc::now(),
            retry_after: None,
            request_id: None,
            status_code: None,
        });

        error!("Operation failed after {} attempts: {}", attempt, final_error.message);

        Err(crate::utils::PantherSwapError::trading(
            format!("Alpaca API operation failed after {} retries: {}", attempt, final_error.message)
        ))
    }

    /// Check if an error should trigger a retry
    fn should_retry(&self, error: &AlpacaError) -> bool {
        match error.error_type {
            AlpacaErrorType::RateLimit => self.retry_config.retry_on_rate_limit,
            AlpacaErrorType::NetworkError => self.retry_config.retry_on_network_error,
            AlpacaErrorType::ServerError => self.retry_config.retry_on_server_error,
            AlpacaErrorType::AuthenticationError => false, // Don't retry auth errors
            AlpacaErrorType::ValidationError => false, // Don't retry validation errors
            AlpacaErrorType::InsufficientFunds => false, // Don't retry insufficient funds
            AlpacaErrorType::MarketClosed => false, // Don't retry when market is closed
            AlpacaErrorType::OrderRejected => false, // Don't retry rejected orders
            AlpacaErrorType::PositionNotFound => false, // Don't retry position not found
            AlpacaErrorType::UnknownError => true, // Retry unknown errors
        }
    }

    /// Calculate retry delay with exponential backoff and jitter
    fn calculate_retry_delay(&self, attempt: u32, error: &AlpacaError) -> u64 {
        // Use retry_after header if available (for rate limiting)
        if let Some(retry_after) = error.retry_after {
            return retry_after * 1000; // Convert to milliseconds
        }

        // Calculate exponential backoff
        let base_delay = self.retry_config.initial_delay_ms as f64;
        let exponential_delay = base_delay * self.retry_config.backoff_multiplier.powi(attempt as i32);

        // Apply jitter to avoid thundering herd
        let jitter = 1.0 + (rand::random::<f64>() - 0.5) * 2.0 * self.retry_config.jitter_factor;
        let delay_with_jitter = exponential_delay * jitter;

        // Cap at maximum delay
        let final_delay = delay_with_jitter.min(self.retry_config.max_delay_ms as f64) as u64;

        final_delay
    }

    /// Get error statistics
    pub async fn get_error_stats(&self) -> serde_json::Value {
        let error_tracker = self.error_tracker.lock().await;
        let circuit_breaker = self.circuit_breaker.lock().await;
        let rate_limiter = self.rate_limiter.lock().await;

        serde_json::json!({
            "error_tracking": {
                "recent_errors_count": error_tracker.recent_errors.len(),
                "error_counts_by_type": error_tracker.error_counts,
                "last_reset": error_tracker.last_reset,
            },
            "circuit_breaker": {
                "state": format!("{:?}", circuit_breaker.state),
                "failure_count": circuit_breaker.failure_count,
                "success_count": circuit_breaker.success_count,
                "last_failure_time": circuit_breaker.last_failure_time,
            },
            "rate_limiter": {
                "requests_made": rate_limiter.requests_made,
                "burst_used": rate_limiter.burst_used,
                "current_delay_ms": rate_limiter.current_delay_ms,
                "window_start": rate_limiter.window_start,
            }
        })
    }

    /// Reset error tracking and circuit breaker
    pub async fn reset(&self) {
        let mut error_tracker = self.error_tracker.lock().await;
        let mut circuit_breaker = self.circuit_breaker.lock().await;
        
        error_tracker.reset();
        circuit_breaker.reset();
        
        info!("Alpaca error handler reset");
    }
}

impl CircuitBreaker {
    pub fn new() -> Self {
        Self {
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
            failure_threshold: 5, // Open after 5 consecutive failures
            recovery_timeout_ms: 60000, // 1 minute recovery timeout
            half_open_max_calls: 3, // Allow 3 calls in half-open state
            half_open_calls: 0,
        }
    }

    pub fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                // Check if we should transition to half-open
                if let Some(last_failure) = self.last_failure_time {
                    let now = Utc::now();
                    let elapsed = (now - last_failure).num_milliseconds() as u64;
                    if elapsed >= self.recovery_timeout_ms {
                        self.state = CircuitBreakerState::HalfOpen;
                        self.half_open_calls = 0;
                        info!("Circuit breaker transitioning to half-open state");
                        return true;
                    }
                }
                false
            }
            CircuitBreakerState::HalfOpen => {
                if self.half_open_calls < self.half_open_max_calls {
                    self.half_open_calls += 1;
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn record_success(&mut self) {
        self.success_count += 1;

        match self.state {
            CircuitBreakerState::HalfOpen => {
                // Transition back to closed after successful calls
                self.state = CircuitBreakerState::Closed;
                self.failure_count = 0;
                self.half_open_calls = 0;
                info!("Circuit breaker closed - service recovered");
            }
            CircuitBreakerState::Closed => {
                // Reset failure count on success
                self.failure_count = 0;
            }
            _ => {}
        }
    }

    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(Utc::now());

        match self.state {
            CircuitBreakerState::Closed => {
                if self.failure_count >= self.failure_threshold {
                    self.state = CircuitBreakerState::Open;
                    warn!("Circuit breaker opened after {} failures", self.failure_count);
                }
            }
            CircuitBreakerState::HalfOpen => {
                // Go back to open state
                self.state = CircuitBreakerState::Open;
                self.half_open_calls = 0;
                warn!("Circuit breaker reopened due to failure in half-open state");
            }
            _ => {}
        }
    }

    pub fn reset(&mut self) {
        self.state = CircuitBreakerState::Closed;
        self.failure_count = 0;
        self.success_count = 0;
        self.last_failure_time = None;
        self.half_open_calls = 0;
    }
}

impl ErrorTracker {
    pub fn new() -> Self {
        Self {
            recent_errors: VecDeque::new(),
            error_counts: std::collections::HashMap::new(),
            window_size: 100, // Keep last 100 errors
            last_reset: Utc::now(),
            reset_interval_hours: 24, // Reset daily
        }
    }

    pub fn record_error(&mut self, error: AlpacaError) {
        // Add to recent errors
        self.recent_errors.push_back(error.clone());

        // Maintain window size
        if self.recent_errors.len() > self.window_size {
            self.recent_errors.pop_front();
        }

        // Update error counts
        *self.error_counts.entry(error.error_type).or_insert(0) += 1;

        // Check if we need to reset (daily reset)
        let now = Utc::now();
        if (now - self.last_reset).num_hours() >= self.reset_interval_hours as i64 {
            self.reset();
        }
    }

    pub fn reset(&mut self) {
        self.recent_errors.clear();
        self.error_counts.clear();
        self.last_reset = Utc::now();
    }
}

impl EnhancedRateLimiter {
    pub fn new(max_requests_per_minute: u32) -> Self {
        Self {
            requests_made: 0,
            window_start: Utc::now(),
            max_requests_per_minute,
            burst_allowance: max_requests_per_minute / 4, // 25% burst allowance
            burst_used: 0,
            burst_reset_time: Utc::now(),
            adaptive_delay: true,
            current_delay_ms: 0,
        }
    }

    pub async fn wait_if_needed(&mut self) -> Result<()> {
        let now = Utc::now();
        let window_elapsed = (now - self.window_start).num_seconds();

        // Reset window if more than 60 seconds have passed
        if window_elapsed >= 60 {
            self.requests_made = 0;
            self.window_start = now;
            self.current_delay_ms = 0;
        }

        // Reset burst allowance every 15 minutes
        if (now - self.burst_reset_time).num_minutes() >= 15 {
            self.burst_used = 0;
            self.burst_reset_time = now;
        }

        // Check if we need to wait
        if self.requests_made >= self.max_requests_per_minute {
            // Check if we can use burst allowance
            if self.burst_used < self.burst_allowance {
                self.burst_used += 1;
                warn!("Using burst allowance ({}/{})", self.burst_used, self.burst_allowance);
            } else {
                // Calculate wait time
                let wait_time = 60 - window_elapsed;
                if wait_time > 0 {
                    warn!("Rate limit reached, waiting {} seconds", wait_time);
                    sleep(Duration::from_secs(wait_time as u64)).await;
                    self.requests_made = 0;
                    self.window_start = Utc::now();
                }
            }
        }

        // Apply adaptive delay if enabled
        if self.adaptive_delay && self.current_delay_ms > 0 {
            sleep(Duration::from_millis(self.current_delay_ms)).await;
            // Gradually reduce delay
            self.current_delay_ms = (self.current_delay_ms as f64 * 0.9) as u64;
        }

        self.requests_made += 1;
        Ok(())
    }

    pub fn increase_delay(&mut self, additional_ms: u64) {
        self.current_delay_ms = (self.current_delay_ms + additional_ms).min(5000); // Max 5 second delay
    }
}

// Helper function to convert common errors to AlpacaError
impl From<reqwest::Error> for AlpacaError {
    fn from(error: reqwest::Error) -> Self {
        let error_type = if error.is_timeout() || error.is_connect() {
            AlpacaErrorType::NetworkError
        } else if error.is_status() {
            match error.status() {
                Some(status) if status.as_u16() == 429 => AlpacaErrorType::RateLimit,
                Some(status) if status.is_server_error() => AlpacaErrorType::ServerError,
                Some(status) if status.as_u16() == 401 || status.as_u16() == 403 => AlpacaErrorType::AuthenticationError,
                Some(status) if status.is_client_error() => AlpacaErrorType::ValidationError,
                _ => AlpacaErrorType::UnknownError,
            }
        } else {
            AlpacaErrorType::NetworkError
        };

        AlpacaError {
            error_type,
            message: error.to_string(),
            timestamp: Utc::now(),
            retry_after: None,
            request_id: None,
            status_code: error.status().map(|s| s.as_u16()),
        }
    }
}
