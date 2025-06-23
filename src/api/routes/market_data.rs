use axum::{
    extract::{Path, Query, Extension},
    Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tracing::{info, error, warn};

use crate::api::{AppState, responses::*};
use crate::api::middleware::auth::{AuthenticatedUser, Permission, has_permission};
use crate::database::types::MarketTick;

/// Query parameters for market ticks
#[derive(Debug, Deserialize)]
pub struct TicksQuery {
    pub instrument_id: Option<Uuid>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub provider: Option<String>,
    pub min_quality: Option<f64>,
}

/// Query parameters for OHLC data
#[derive(Debug, Deserialize)]
pub struct OhlcQuery {
    pub instrument_id: Uuid,
    pub interval: String, // "1m", "5m", "15m", "1h", "4h", "1d"
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
}

/// Response for market ticks
#[derive(Debug, Serialize)]
pub struct TicksResponse {
    pub ticks: Vec<MarketTickResponse>,
    pub count: usize,
    pub page_info: PageInfo,
}

/// Response for OHLC data
#[derive(Debug, Serialize)]
pub struct OhlcResponse {
    pub interval: String,
    pub count: usize,
    pub candles: Vec<CandleResponse>,
}

/// Response for latest ticks by instrument
#[derive(Debug, Serialize)]
pub struct LatestTicksResponse {
    pub ticks: HashMap<Uuid, MarketTickResponse>,
    pub timestamp: DateTime<Utc>,
}

/// Get market ticks with filtering and pagination
pub async fn get_ticks(
    Query(params): Query<TicksQuery>,
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<ApiResponse<TicksResponse>>, StatusCode> {
    // Check permissions
    if !has_permission(&user, &Permission::ReadMarketData) {
        warn!(user_id = %user.id, "Permission denied for market data access");
        return Err(StatusCode::FORBIDDEN);
    }

    info!(
        user_id = %user.id,
        instrument_id = ?params.instrument_id,
        "Market ticks requested"
    );

    let limit = params.limit.unwrap_or(100).min(1000); // Max 1000 records
    let min_quality = params.min_quality.unwrap_or(0.0);

    let query_manager = state.database.query_manager();

    // Build query based on parameters
    let ticks = if let Some(instrument_id) = params.instrument_id {
        // Get ticks for specific instrument
        match query_manager.get_market_ticks_for_instrument(
            instrument_id,
            params.start_time,
            params.end_time,
            Some(limit),
        ).await {
            Ok(ticks) => ticks,
            Err(e) => {
                error!("Failed to fetch market ticks: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    } else {
        // Get latest ticks across all instruments
        match query_manager.get_latest_market_ticks(None, Some(limit)).await {
            Ok(ticks) => ticks,
            Err(e) => {
                error!("Failed to fetch latest market ticks: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    };

    // Filter by quality and provider if specified
    let filtered_ticks: Vec<MarketTick> = ticks.into_iter()
        .filter(|tick| {
            // Quality filter
            if tick.data_quality_score < min_quality {
                return false;
            }
            
            // Provider filter
            if let Some(ref provider) = params.provider {
                if tick.provider != *provider {
                    return false;
                }
            }
            
            true
        })
        .collect();

    // Convert to response format
    let response_ticks: Vec<MarketTickResponse> = filtered_ticks.iter()
        .map(|tick| MarketTickResponse {
            timestamp: tick.timestamp,
            instrument_id: tick.instrument_id,
            provider: tick.provider.clone(),
            bid_price: tick.bid_price,
            ask_price: tick.ask_price,
            spread: tick.spread,
            bid_size: Some(tick.bid_size),
            ask_size: Some(tick.ask_size),
            volume: tick.volume,
            data_quality_score: tick.data_quality_score,
        })
        .collect();

    let page_info = PageInfo {
        has_next_page: response_ticks.len() == limit as usize,
        has_previous_page: false, // Would need offset-based pagination for this
        total_count: Some(response_ticks.len() as i64),
        page: None,
        per_page: Some(limit),
    };

    let response = TicksResponse {
        count: response_ticks.len(),
        ticks: response_ticks,
        page_info,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Get latest tick for each instrument
pub async fn get_latest_ticks(
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<ApiResponse<LatestTicksResponse>>, StatusCode> {
    // Check permissions
    if !has_permission(&user, &Permission::ReadMarketData) {
        return Err(StatusCode::FORBIDDEN);
    }

    info!(user_id = %user.id, "Latest ticks requested");

    let query_manager = state.database.query_manager();
    
    let latest_ticks = match query_manager.get_latest_market_ticks(None, Some(100)).await {
        Ok(ticks) => ticks,
        Err(e) => {
            error!("Failed to fetch latest market ticks: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Group by instrument_id and keep only the latest for each
    let mut response_map = HashMap::new();
    
    for tick in latest_ticks {
        let response_tick = MarketTickResponse {
            timestamp: tick.timestamp,
            instrument_id: tick.instrument_id,
            provider: tick.provider,
            bid_price: tick.bid_price,
            ask_price: tick.ask_price,
            spread: tick.spread,
            bid_size: Some(tick.bid_size),
            ask_size: Some(tick.ask_size),
            volume: tick.volume,
            data_quality_score: tick.data_quality_score,
        };
        
        // Only keep if this is newer than existing entry
        if let Some(existing) = response_map.get(&tick.instrument_id) {
            let existing_tick: &MarketTickResponse = existing;
            if tick.timestamp > existing_tick.timestamp {
                response_map.insert(tick.instrument_id, response_tick);
            }
        } else {
            response_map.insert(tick.instrument_id, response_tick);
        }
    }
    
    let response = LatestTicksResponse {
        ticks: response_map,
        timestamp: Utc::now(),
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Get OHLC (candlestick) data
pub async fn get_ohlc(
    Query(params): Query<OhlcQuery>,
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<ApiResponse<OhlcResponse>>, StatusCode> {
    // Check permissions
    if !has_permission(&user, &Permission::ReadMarketData) {
        return Err(StatusCode::FORBIDDEN);
    }

    info!(
        user_id = %user.id,
        instrument_id = %params.instrument_id,
        interval = %params.interval,
        "OHLC data requested"
    );

    // Validate interval
    let interval_seconds = match params.interval.as_str() {
        "1m" => 60,
        "5m" => 300,
        "15m" => 900,
        "1h" => 3600,
        "4h" => 14400,
        "1d" => 86400,
        _ => {
            warn!("Invalid interval requested: {}", params.interval);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    let limit = params.limit.unwrap_or(100).min(1000);
    let end_time = params.end_time.unwrap_or_else(Utc::now);
    let start_time = params.start_time.unwrap_or_else(|| {
        end_time - chrono::Duration::seconds(interval_seconds * limit)
    });

    // For now, return mock OHLC data since we don't have aggregation implemented yet
    // In a real implementation, this would use TimescaleDB's time_bucket function
    let mock_candles = generate_mock_ohlc_data(
        params.instrument_id,
        start_time,
        end_time,
        interval_seconds,
        limit as usize,
    );

    let response = OhlcResponse {
        interval: params.interval,
        count: mock_candles.len(),
        candles: mock_candles,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Generate mock OHLC data (temporary implementation)
fn generate_mock_ohlc_data(
    _instrument_id: Uuid,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    interval_seconds: i64,
    limit: usize,
) -> Vec<CandleResponse> {
    let mut candles = Vec::new();
    let mut current_time = start_time;
    let interval_duration = chrono::Duration::seconds(interval_seconds);
    
    let mut base_price = 1.0850; // Starting price for EUR/USD
    
    for _ in 0..limit {
        if current_time > end_time {
            break;
        }
        
        // Generate realistic OHLC data
        let open = base_price;
        let volatility = 0.001; // 0.1% volatility
        let change = (rand::random::<f64>() - 0.5) * volatility;
        let close = open + change;
        
        let high = open.max(close) + (rand::random::<f64>() * volatility * 0.5);
        let low = open.min(close) - (rand::random::<f64>() * volatility * 0.5);
        
        let volume = 1000000.0 + (rand::random::<f64>() * 5000000.0);
        
        candles.push(CandleResponse {
            timestamp: current_time,
            open,
            high,
            low,
            close,
            volume,
            vwap: Some((high + low + close) / 3.0),
        });
        
        base_price = close;
        current_time += interval_duration;
    }
    
    candles
}

/// Get market data statistics
pub async fn get_market_stats(
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<ApiResponse<MarketStatsResponse>>, StatusCode> {
    // Check permissions
    if !has_permission(&user, &Permission::ReadMarketData) {
        return Err(StatusCode::FORBIDDEN);
    }

    info!(user_id = %user.id, "Market statistics requested");

    // Get basic statistics from database
    let query_manager = state.database.query_manager();
    
    // This would be implemented with proper aggregation queries
    let stats = MarketStatsResponse {
        total_instruments: 5, // Our default forex pairs
        total_ticks_today: 0, // Would need to be calculated
        average_quality_score: 0.95,
        active_providers: vec!["alpha_vantage".to_string()],
        last_update: Utc::now(),
    };

    Ok(Json(ApiResponse::success(stats)))
}

#[derive(Debug, Serialize)]
pub struct MarketStatsResponse {
    pub total_instruments: u32,
    pub total_ticks_today: u64,
    pub average_quality_score: f64,
    pub active_providers: Vec<String>,
    pub last_update: DateTime<Utc>,
}
