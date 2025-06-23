use serde::Serialize;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use axum::http::StatusCode;

/// Standard API response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
    pub timestamp: DateTime<Utc>,
    pub request_id: String,
}

/// API error structure
#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// Pagination information
#[derive(Debug, Serialize)]
pub struct PageInfo {
    pub has_next_page: bool,
    pub has_previous_page: bool,
    pub total_count: Option<i64>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub timestamp: DateTime<Utc>,
}

/// System status response
#[derive(Debug, Serialize)]
pub struct SystemStatusResponse {
    pub database: DatabaseStatus,
    pub market_data: MarketDataStatus,
    pub api: ApiStatus,
    pub overall_status: String,
}

#[derive(Debug, Serialize)]
pub struct DatabaseStatus {
    pub connected: bool,
    pub pool_size: u32,
    pub active_connections: u32,
    pub last_query_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct MarketDataStatus {
    pub providers_active: u32,
    pub last_update: Option<DateTime<Utc>>,
    pub total_instruments: u32,
    pub data_quality_avg: f64,
}

#[derive(Debug, Serialize)]
pub struct ApiStatus {
    pub requests_per_minute: u64,
    pub active_connections: u32,
    pub error_rate: f64,
}

/// Market tick response
#[derive(Debug, Serialize)]
pub struct MarketTickResponse {
    pub timestamp: DateTime<Utc>,
    pub instrument_id: Uuid,
    pub provider: String,
    pub bid_price: f64,
    pub ask_price: f64,
    pub spread: f64,
    pub bid_size: Option<f64>,
    pub ask_size: Option<f64>,
    pub volume: Option<f64>,
    pub data_quality_score: f64,
}

/// Instrument response
#[derive(Debug, Serialize)]
pub struct InstrumentResponse {
    pub id: Uuid,
    pub symbol: String,
    pub name: String,
    pub instrument_type: String,
    pub base_currency: String,
    pub quote_currency: String,
    pub tick_size: f64,
    pub lot_size: f64,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// OHLC candle response
#[derive(Debug, Serialize)]
pub struct CandleResponse {
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub vwap: Option<f64>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
            request_id: Uuid::new_v4().to_string(),
        }
    }

    pub fn error(code: String, message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ApiError {
                code,
                message,
                details: None,
            }),
            timestamp: Utc::now(),
            request_id: Uuid::new_v4().to_string(),
        }
    }

    pub fn error_with_details(code: String, message: String, details: serde_json::Value) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ApiError {
                code,
                message,
                details: Some(details),
            }),
            timestamp: Utc::now(),
            request_id: Uuid::new_v4().to_string(),
        }
    }
}

impl<T> ApiResponse<T> {
    /// Convert to HTTP status code based on error
    pub fn status_code(&self) -> StatusCode {
        if self.success {
            return StatusCode::OK;
        }

        if let Some(error) = &self.error {
            match error.code.as_str() {
                "VALIDATION_ERROR" => StatusCode::BAD_REQUEST,
                "UNAUTHORIZED" => StatusCode::UNAUTHORIZED,
                "FORBIDDEN" => StatusCode::FORBIDDEN,
                "NOT_FOUND" => StatusCode::NOT_FOUND,
                "RATE_LIMITED" => StatusCode::TOO_MANY_REQUESTS,
                "DATABASE_ERROR" => StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR" => StatusCode::INTERNAL_SERVER_ERROR,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

/// Common error codes
pub mod error_codes {
    pub const VALIDATION_ERROR: &str = "VALIDATION_ERROR";
    pub const UNAUTHORIZED: &str = "UNAUTHORIZED";
    pub const FORBIDDEN: &str = "FORBIDDEN";
    pub const NOT_FOUND: &str = "NOT_FOUND";
    pub const RATE_LIMITED: &str = "RATE_LIMITED";
    pub const DATABASE_ERROR: &str = "DATABASE_ERROR";
    pub const INTERNAL_ERROR: &str = "INTERNAL_ERROR";
    pub const INVALID_PARAMETERS: &str = "INVALID_PARAMETERS";
    pub const MARKET_DATA_ERROR: &str = "MARKET_DATA_ERROR";
}
