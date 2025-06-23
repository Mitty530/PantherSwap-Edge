use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Market quote from external data providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketQuote {
    pub symbol: String,
    pub provider: String,
    pub timestamp: DateTime<Utc>,
    pub bid_price: f64,
    pub ask_price: f64,
    pub mid_price: f64,
    pub bid_size: Option<f64>,
    pub ask_size: Option<f64>,
    pub volume: Option<f64>,
    pub spread: Option<f64>,
    pub data_quality: f64,
}



/// Data quality assessment result
#[derive(Debug, Clone)]
pub struct DataQualityResult {
    pub score: f64,
    pub issues: Vec<String>,
    pub is_valid: bool,
}

/// Rate limiting state
#[derive(Debug, Clone)]
pub struct RateLimitState {
    pub requests_made: u32,
    pub window_start: DateTime<Utc>,
    pub max_requests_per_minute: u32,
}

impl RateLimitState {
    pub fn new(max_requests_per_minute: u32) -> Self {
        Self {
            requests_made: 0,
            window_start: Utc::now(),
            max_requests_per_minute,
        }
    }

    pub fn can_make_request(&self) -> bool {
        let now = Utc::now();
        let window_elapsed = (now - self.window_start).num_seconds();

        // Reset window if more than 60 seconds have passed
        if window_elapsed >= 60 {
            return true;
        }

        self.requests_made < self.max_requests_per_minute
    }

    pub fn record_request(&mut self) {
        let now = Utc::now();
        let window_elapsed = (now - self.window_start).num_seconds();

        // Reset window if more than 60 seconds have passed
        if window_elapsed >= 60 {
            self.requests_made = 1;
            self.window_start = now;
        } else {
            self.requests_made += 1;
        }
    }

    pub fn time_until_next_request(&self) -> Option<std::time::Duration> {
        if self.can_make_request() {
            return None;
        }

        let now = Utc::now();
        let window_elapsed = (now - self.window_start).num_seconds();
        let remaining_seconds = 60 - window_elapsed;

        if remaining_seconds > 0 {
            Some(std::time::Duration::from_secs(remaining_seconds as u64))
        } else {
            None
        }
    }
}
