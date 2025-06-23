use axum::{
    extract::{Path, Query, Extension},
    Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::{info, error, warn};

use crate::api::{AppState, responses::*};
use crate::api::middleware::auth::{AuthenticatedUser, Permission, has_permission};
use crate::database::types::Instrument;

/// Query parameters for listing instruments
#[derive(Debug, Deserialize)]
pub struct InstrumentsQuery {
    pub instrument_type: Option<String>,
    pub is_active: Option<bool>,
    pub base_currency: Option<String>,
    pub quote_currency: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Request body for creating instruments
#[derive(Debug, Deserialize)]
pub struct CreateInstrumentRequest {
    pub symbol: String,
    pub name: String,
    pub instrument_type: String,
    pub base_currency: String,
    pub quote_currency: String,
    pub tick_size: f64,
    pub lot_size: f64,
}

/// Request body for updating instruments
#[derive(Debug, Deserialize)]
pub struct UpdateInstrumentRequest {
    pub name: Option<String>,
    pub tick_size: Option<f64>,
    pub lot_size: Option<f64>,
    pub is_active: Option<bool>,
}

/// Response for listing instruments
#[derive(Debug, Serialize)]
pub struct InstrumentsResponse {
    pub instruments: Vec<InstrumentResponse>,
    pub count: usize,
    pub page_info: PageInfo,
}

/// List all instruments with filtering
pub async fn list_instruments(
    Query(params): Query<InstrumentsQuery>,
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<ApiResponse<InstrumentsResponse>>, StatusCode> {
    // Check permissions
    if !has_permission(&user, &Permission::ReadMarketData) {
        warn!(user_id = %user.id, "Permission denied for instruments access");
        return Err(StatusCode::FORBIDDEN);
    }

    info!(
        user_id = %user.id,
        instrument_type = ?params.instrument_type,
        "Instruments list requested"
    );

    let limit = params.limit.unwrap_or(50).min(1000);
    let offset = params.offset.unwrap_or(0);

    let query_manager = state.database.query_manager();

    // Get instruments based on filters
    let instruments = match query_manager.get_instruments_with_filters(
        params.instrument_type.as_deref(),
        params.is_active,
        params.base_currency.as_deref(),
        params.quote_currency.as_deref(),
        Some(limit),
        Some(offset),
    ).await {
        Ok(instruments) => instruments,
        Err(e) => {
            error!("Failed to fetch instruments: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Convert to response format
    let response_instruments: Vec<InstrumentResponse> = instruments.iter()
        .map(|instrument| InstrumentResponse {
            id: instrument.id,
            symbol: instrument.symbol.clone(),
            name: instrument.name.clone(),
            instrument_type: instrument.instrument_type.clone(),
            base_currency: instrument.base_currency.clone(),
            quote_currency: instrument.quote_currency.clone(),
            tick_size: instrument.tick_size,
            lot_size: instrument.lot_size,
            is_active: instrument.is_active,
            created_at: instrument.created_at,
            updated_at: instrument.updated_at,
        })
        .collect();

    let page_info = PageInfo {
        has_next_page: response_instruments.len() == limit as usize,
        has_previous_page: offset > 0,
        total_count: None, // Would need a separate count query
        page: Some(offset / limit + 1),
        per_page: Some(limit),
    };

    let response = InstrumentsResponse {
        count: response_instruments.len(),
        instruments: response_instruments,
        page_info,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Get a specific instrument by ID
pub async fn get_instrument(
    Path(instrument_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<ApiResponse<InstrumentResponse>>, StatusCode> {
    // Check permissions
    if !has_permission(&user, &Permission::ReadMarketData) {
        return Err(StatusCode::FORBIDDEN);
    }

    info!(
        user_id = %user.id,
        instrument_id = %instrument_id,
        "Instrument details requested"
    );

    let query_manager = state.database.query_manager();

    let instrument = match query_manager.get_instrument_by_id(instrument_id).await {
        Ok(Some(instrument)) => instrument,
        Ok(None) => {
            warn!("Instrument not found: {}", instrument_id);
            return Err(StatusCode::NOT_FOUND);
        }
        Err(e) => {
            error!("Failed to fetch instrument: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let response = InstrumentResponse {
        id: instrument.id,
        symbol: instrument.symbol,
        name: instrument.name,
        instrument_type: instrument.instrument_type,
        base_currency: instrument.base_currency,
        quote_currency: instrument.quote_currency,
        tick_size: instrument.tick_size,
        lot_size: instrument.lot_size,
        is_active: instrument.is_active,
        created_at: instrument.created_at,
        updated_at: instrument.updated_at,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Create a new instrument
pub async fn create_instrument(
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(request): Json<CreateInstrumentRequest>,
) -> Result<Json<ApiResponse<InstrumentResponse>>, StatusCode> {
    // Check permissions
    if !has_permission(&user, &Permission::WriteMarketData) {
        warn!(user_id = %user.id, "Permission denied for instrument creation");
        return Err(StatusCode::FORBIDDEN);
    }

    info!(
        user_id = %user.id,
        symbol = %request.symbol,
        "Instrument creation requested"
    );

    // Validate request
    if let Err(status) = validate_create_instrument_request(&request) {
        return Err(status);
    }

    let query_manager = state.database.query_manager();

    // Check if instrument with same symbol already exists
    match query_manager.get_instrument_by_symbol(&request.symbol).await {
        Ok(Some(_)) => {
            warn!("Instrument with symbol {} already exists", request.symbol);
            return Err(StatusCode::CONFLICT);
        }
        Ok(None) => {
            // Good, symbol is available
        }
        Err(e) => {
            error!("Failed to check existing instrument: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    // Create new instrument
    let new_instrument = Instrument {
        id: Uuid::new_v4(), // Will be overridden by database
        symbol: request.symbol,
        name: request.name,
        instrument_type: request.instrument_type,
        base_currency: request.base_currency,
        quote_currency: request.quote_currency,
        tick_size: request.tick_size,
        lot_size: request.lot_size,
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let instrument_id = match query_manager.insert_instrument(&new_instrument).await {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to create instrument: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Fetch the created instrument
    let created_instrument = match query_manager.get_instrument_by_id(instrument_id).await {
        Ok(Some(instrument)) => instrument,
        Ok(None) => {
            error!("Created instrument not found: {}", instrument_id);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
        Err(e) => {
            error!("Failed to fetch created instrument: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let response = InstrumentResponse {
        id: created_instrument.id,
        symbol: created_instrument.symbol,
        name: created_instrument.name,
        instrument_type: created_instrument.instrument_type,
        base_currency: created_instrument.base_currency,
        quote_currency: created_instrument.quote_currency,
        tick_size: created_instrument.tick_size,
        lot_size: created_instrument.lot_size,
        is_active: created_instrument.is_active,
        created_at: created_instrument.created_at,
        updated_at: created_instrument.updated_at,
    };

    info!(
        user_id = %user.id,
        instrument_id = %instrument_id,
        "Instrument created successfully"
    );

    Ok(Json(ApiResponse::success(response)))
}

/// Update an existing instrument
pub async fn update_instrument(
    Path(instrument_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(request): Json<UpdateInstrumentRequest>,
) -> Result<Json<ApiResponse<InstrumentResponse>>, StatusCode> {
    // Check permissions
    if !has_permission(&user, &Permission::WriteMarketData) {
        return Err(StatusCode::FORBIDDEN);
    }

    info!(
        user_id = %user.id,
        instrument_id = %instrument_id,
        "Instrument update requested"
    );

    let query_manager = state.database.query_manager();

    // Check if instrument exists
    let mut instrument = match query_manager.get_instrument_by_id(instrument_id).await {
        Ok(Some(instrument)) => instrument,
        Ok(None) => {
            warn!("Instrument not found for update: {}", instrument_id);
            return Err(StatusCode::NOT_FOUND);
        }
        Err(e) => {
            error!("Failed to fetch instrument for update: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Apply updates
    if let Some(name) = request.name {
        instrument.name = name;
    }
    if let Some(tick_size) = request.tick_size {
        if tick_size <= 0.0 {
            warn!("Invalid tick size: {}", tick_size);
            return Err(StatusCode::BAD_REQUEST);
        }
        instrument.tick_size = tick_size;
    }
    if let Some(lot_size) = request.lot_size {
        if lot_size <= 0.0 {
            warn!("Invalid lot size: {}", lot_size);
            return Err(StatusCode::BAD_REQUEST);
        }
        instrument.lot_size = lot_size;
    }
    if let Some(is_active) = request.is_active {
        instrument.is_active = is_active;
    }
    instrument.updated_at = Utc::now();

    // Update in database
    match query_manager.update_instrument(&instrument).await {
        Ok(_) => {}
        Err(e) => {
            error!("Failed to update instrument: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    let response = InstrumentResponse {
        id: instrument.id,
        symbol: instrument.symbol,
        name: instrument.name,
        instrument_type: instrument.instrument_type,
        base_currency: instrument.base_currency,
        quote_currency: instrument.quote_currency,
        tick_size: instrument.tick_size,
        lot_size: instrument.lot_size,
        is_active: instrument.is_active,
        created_at: instrument.created_at,
        updated_at: instrument.updated_at,
    };

    info!(
        user_id = %user.id,
        instrument_id = %instrument_id,
        "Instrument updated successfully"
    );

    Ok(Json(ApiResponse::success(response)))
}

/// Validate create instrument request
fn validate_create_instrument_request(request: &CreateInstrumentRequest) -> Result<(), StatusCode> {
    // Validate symbol
    if request.symbol.is_empty() || request.symbol.len() > 20 {
        warn!("Invalid symbol length: {}", request.symbol.len());
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validate name
    if request.name.is_empty() || request.name.len() > 255 {
        warn!("Invalid name length: {}", request.name.len());
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validate instrument type
    let valid_types = ["forex", "crypto", "stock", "commodity", "index"];
    if !valid_types.contains(&request.instrument_type.as_str()) {
        warn!("Invalid instrument type: {}", request.instrument_type);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validate currencies
    if request.base_currency.len() != 3 || request.quote_currency.len() != 3 {
        warn!("Invalid currency codes");
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validate tick size and lot size
    if request.tick_size <= 0.0 || request.lot_size <= 0.0 {
        warn!("Invalid tick size or lot size");
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(())
}
