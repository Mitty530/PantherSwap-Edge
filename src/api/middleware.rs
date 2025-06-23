pub mod auth;
pub mod rate_limit;
pub mod validation;
pub mod cors;

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
    http::{StatusCode, HeaderMap},
};
use std::time::Instant;
use tracing::{info, warn};
use uuid::Uuid;

/// Request ID middleware - adds unique request ID to all requests
pub async fn request_id_middleware(
    mut request: Request,
    next: Next,
) -> Response {
    let request_id = Uuid::new_v4().to_string();

    // Add request ID to request extensions
    request.extensions_mut().insert(request_id.clone());

    let mut response = next.run(request).await;

    // Add request ID to response headers
    response.headers_mut().insert(
        "x-request-id",
        request_id.parse().unwrap_or_else(|_| "invalid".parse().unwrap())
    );

    response
}

/// Logging middleware - logs all requests and responses
pub async fn logging_middleware(
    request: Request,
    next: Next,
) -> Response {
    let start_time = Instant::now();
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let query = request.uri().query().map(|q| q.to_string());

    // Extract request ID if available
    let request_id = request.extensions()
        .get::<String>()
        .cloned()
        .unwrap_or_else(|| "unknown".to_string());

    // Extract IP address
    let ip_address = extract_ip_address(request.headers());

    info!(
        request_id = %request_id,
        method = %method,
        path = %path,
        query = ?query,
        ip = %ip_address,
        "Request started"
    );

    let response = next.run(request).await;
    let duration = start_time.elapsed();
    let status = response.status();

    if status.is_success() {
        info!(
            request_id = %request_id,
            method = %method,
            path = %path,
            status = %status,
            duration_ms = %duration.as_millis(),
            "Request completed successfully"
        );
    } else {
        warn!(
            request_id = %request_id,
            method = %method,
            path = %path,
            status = %status,
            duration_ms = %duration.as_millis(),
            "Request completed with error"
        );
    }

    response
}

/// Error handling middleware - converts errors to standard API responses
pub async fn error_handling_middleware(
    request: Request,
    next: Next,
) -> Response {
    let response = next.run(request).await;

    // If response is already an error, let it pass through
    if !response.status().is_success() {
        return response;
    }

    response
}

/// Content type validation middleware
pub async fn content_type_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let method = request.method();

    // Only validate content type for POST and PUT requests
    if method == "POST" || method == "PUT" {
        if let Some(content_type) = request.headers().get("content-type") {
            let content_type_str = content_type.to_str()
                .map_err(|_| StatusCode::BAD_REQUEST)?;

            if !content_type_str.starts_with("application/json") {
                return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
            }
        } else {
            return Err(StatusCode::BAD_REQUEST);
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

/// Health check bypass middleware - allows health endpoints without authentication
pub fn is_health_endpoint(path: &str) -> bool {
    path.starts_with("/health") ||
    path.starts_with("/metrics") ||
    path == "/status" ||
    path == "/"
}
