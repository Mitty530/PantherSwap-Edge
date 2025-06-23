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
use crate::trading::execution::{Order, OrderStatus, Fill, LiquidityFlag};
use crate::trading::signals::OrderRequest;
use crate::database::types::{SignalType, OrderType, TimeInForce};

/// Query parameters for listing orders
#[derive(Debug, Deserialize)]
pub struct OrdersQuery {
    pub instrument_id: Option<Uuid>,
    pub status: Option<String>,
    pub strategy_name: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Request to submit a new order
#[derive(Debug, Deserialize)]
pub struct SubmitOrderRequest {
    pub instrument_id: Uuid,
    pub side: SignalType,
    pub quantity: f64,
    pub order_type: OrderType,
    pub price: Option<f64>,
    pub time_in_force: TimeInForce,
    pub strategy_name: Option<String>,
}

/// Response for order submission
#[derive(Debug, Serialize)]
pub struct OrderResponse {
    pub id: Uuid,
    pub instrument_id: Uuid,
    pub side: SignalType,
    pub quantity: f64,
    pub filled_quantity: f64,
    pub remaining_quantity: f64,
    pub order_type: OrderType,
    pub price: Option<f64>,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub fills: Vec<FillResponse>,
    pub strategy_name: Option<String>,
}

/// Response for order fill information
#[derive(Debug, Serialize)]
pub struct FillResponse {
    pub id: Uuid,
    pub order_id: Uuid,
    pub quantity: f64,
    pub price: f64,
    pub timestamp: DateTime<Utc>,
    pub fees: f64,
    pub liquidity_flag: LiquidityFlag,
}

/// Response for orders list
#[derive(Debug, Serialize)]
pub struct OrdersResponse {
    pub orders: Vec<OrderResponse>,
    pub total_count: u64,
    pub page_info: PageInfo,
}

/// Response for order statistics
#[derive(Debug, Serialize)]
pub struct OrderStatsResponse {
    pub total_orders: u32,
    pub active_orders: u32,
    pub filled_orders: u32,
    pub cancelled_orders: u32,
    pub rejected_orders: u32,
    pub average_fill_time_ms: f64,
    pub fill_rate: f64,
    pub total_volume: f64,
    pub total_fees: f64,
}

/// Submit a new trading order
pub async fn submit_order(
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(request): Json<SubmitOrderRequest>,
) -> Result<Json<ApiResponse<OrderResponse>>, StatusCode> {
    // Check permissions
    if !has_permission(&user, &Permission::Trade) {
        warn!(user_id = %user.id, "Permission denied for order submission");
        return Err(StatusCode::FORBIDDEN);
    }

    info!(
        user_id = %user.id,
        instrument_id = %request.instrument_id,
        side = ?request.side,
        quantity = %request.quantity,
        "Order submission requested"
    );

    // Validate order request
    if let Err(status) = validate_order_request(&request) {
        return Err(status);
    }

    // Convert to internal order request
    let order_request = OrderRequest {
        instrument_id: request.instrument_id,
        side: request.side,
        quantity: request.quantity,
        order_type: request.order_type,
        price: request.price,
        time_in_force: request.time_in_force,
    };

    // Submit order through trading engine
    let trading_engine = state.trading_engine.lock().await;
    let order_id = match trading_engine.submit_order(order_request).await {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to submit order: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Get the created order details
    let order = match trading_engine.get_order_details(order_id).await {
        Some(order) => order,
        None => {
            error!("Order not found after submission: {}", order_id);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let response = convert_order_to_response(order);
    Ok(Json(ApiResponse::success(response)))
}

/// Cancel an existing order
pub async fn cancel_order(
    Path(order_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<ApiResponse<OrderResponse>>, StatusCode> {
    // Check permissions
    if !has_permission(&user, &Permission::Trade) {
        return Err(StatusCode::FORBIDDEN);
    }

    info!(
        user_id = %user.id,
        order_id = %order_id,
        "Order cancellation requested"
    );

    let trading_engine = state.trading_engine.write().await;
    
    // Cancel the order
    if let Err(e) = trading_engine.cancel_order(order_id).await {
        error!("Failed to cancel order {}: {}", order_id, e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Get updated order details
    let order = match trading_engine.get_order_details(order_id).await {
        Some(order) => order,
        None => {
            warn!("Order not found: {}", order_id);
            return Err(StatusCode::NOT_FOUND);
        }
    };

    let response = convert_order_to_response(order);
    Ok(Json(ApiResponse::success(response)))
}

/// Get order details by ID
pub async fn get_order(
    Path(order_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<ApiResponse<OrderResponse>>, StatusCode> {
    // Check permissions
    if !has_permission(&user, &Permission::ReadTradingData) {
        return Err(StatusCode::FORBIDDEN);
    }

    info!(
        user_id = %user.id,
        order_id = %order_id,
        "Order details requested"
    );

    let trading_engine = state.trading_engine.lock().await;
    let order = match trading_engine.get_order_details(order_id).await {
        Some(order) => order,
        None => {
            warn!("Order not found: {}", order_id);
            return Err(StatusCode::NOT_FOUND);
        }
    };

    let response = convert_order_to_response(order);
    Ok(Json(ApiResponse::success(response)))
}

/// List orders with filtering and pagination
pub async fn list_orders(
    Query(params): Query<OrdersQuery>,
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<ApiResponse<OrdersResponse>>, StatusCode> {
    // Check permissions
    if !has_permission(&user, &Permission::ReadTradingData) {
        return Err(StatusCode::FORBIDDEN);
    }

    info!(
        user_id = %user.id,
        instrument_id = ?params.instrument_id,
        status = ?params.status,
        "Orders list requested"
    );

    let limit = params.limit.unwrap_or(50).min(1000);
    let offset = params.offset.unwrap_or(0);

    let trading_engine = state.trading_engine.lock().await;
    
    // Get orders based on filters
    let orders = match trading_engine.get_orders_with_filters(
        params.instrument_id,
        params.status.as_deref(),
        params.strategy_name.as_deref(),
        params.start_time,
        params.end_time,
        Some(limit),
        Some(offset),
    ).await {
        Ok(orders) => orders,
        Err(e) => {
            error!("Failed to fetch orders: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let total_count = orders.len() as u64;
    let order_responses: Vec<OrderResponse> = orders.into_iter()
        .map(convert_order_to_response)
        .collect();

    let response = OrdersResponse {
        orders: order_responses,
        total_count,
        page_info: PageInfo {
            has_next_page: total_count > (offset + limit) as u64,
            has_previous_page: offset > 0,
            total_count: Some(total_count as i64),
            page: Some((offset / limit) as i64 + 1),
            per_page: Some(limit),
        },
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Get order statistics
pub async fn get_order_stats(
    Extension(state): Extension<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<ApiResponse<OrderStatsResponse>>, StatusCode> {
    // Check permissions
    if !has_permission(&user, &Permission::ReadTradingData) {
        return Err(StatusCode::FORBIDDEN);
    }

    info!(user_id = %user.id, "Order statistics requested");

    let trading_engine = state.trading_engine.read().await;
    let stats = match trading_engine.get_order_statistics().await {
        Ok(stats) => stats,
        Err(e) => {
            error!("Failed to get order statistics: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let response = OrderStatsResponse {
        total_orders: stats.total_orders,
        active_orders: stats.active_orders,
        filled_orders: stats.filled_orders,
        cancelled_orders: stats.cancelled_orders,
        rejected_orders: stats.rejected_orders,
        average_fill_time_ms: stats.average_fill_time_ms,
        fill_rate: stats.fill_rate,
        total_volume: stats.total_volume,
        total_fees: stats.total_fees,
    };

    Ok(Json(ApiResponse::success(response)))
}

// Helper functions

fn validate_order_request(request: &SubmitOrderRequest) -> Result<(), StatusCode> {
    if request.quantity <= 0.0 {
        warn!("Invalid order quantity: {}", request.quantity);
        return Err(StatusCode::BAD_REQUEST);
    }

    if let Some(price) = request.price {
        if price <= 0.0 {
            warn!("Invalid order price: {}", price);
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    // Validate order type and price combination
    match request.order_type {
        OrderType::Limit | OrderType::StopLimit => {
            if request.price.is_none() {
                warn!("Limit orders require a price");
                return Err(StatusCode::BAD_REQUEST);
            }
        },
        _ => {}
    }

    Ok(())
}

fn convert_order_to_response(order: Order) -> OrderResponse {
    OrderResponse {
        id: order.id,
        instrument_id: order.instrument_id,
        side: order.side,
        quantity: order.quantity,
        filled_quantity: order.filled_quantity,
        remaining_quantity: order.remaining_quantity,
        order_type: order.order_type,
        price: order.price,
        status: order.status,
        created_at: order.created_at,
        updated_at: order.updated_at,
        fills: order.fills.into_iter().map(convert_fill_to_response).collect(),
        strategy_name: order.strategy_name,
    }
}

fn convert_fill_to_response(fill: Fill) -> FillResponse {
    FillResponse {
        id: fill.id,
        order_id: fill.order_id,
        quantity: fill.quantity,
        price: fill.price,
        timestamp: fill.timestamp,
        fees: fill.fees,
        liquidity_flag: fill.liquidity_flag,
    }
}
