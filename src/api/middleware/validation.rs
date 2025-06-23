use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
    http::{StatusCode, HeaderMap, Method},
};
use serde_json::Value;
use tracing::{warn, debug};

use crate::api::responses::{ApiResponse, error_codes};

/// Request validation middleware
pub async fn validation_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip validation for health endpoints
    if crate::api::middleware::is_health_endpoint(request.uri().path()) {
        return Ok(next.run(request).await);
    }

    // Validate request size
    if let Err(status) = validate_request_size(request.headers()) {
        return Err(status);
    }

    // Validate content type for POST/PUT requests
    if let Err(status) = validate_content_type(request.method(), request.headers()) {
        return Err(status);
    }

    // Validate headers
    if let Err(status) = validate_headers(request.headers()) {
        return Err(status);
    }

    // Validate URL parameters
    if let Err(status) = validate_url_parameters(request.uri().path(), request.uri().query()) {
        return Err(status);
    }

    Ok(next.run(request).await)
}

/// Validate request size
fn validate_request_size(headers: &HeaderMap) -> Result<(), StatusCode> {
    const MAX_REQUEST_SIZE: usize = 10 * 1024 * 1024; // 10MB

    if let Some(content_length) = headers.get("content-length") {
        if let Ok(length_str) = content_length.to_str() {
            if let Ok(length) = length_str.parse::<usize>() {
                if length > MAX_REQUEST_SIZE {
                    warn!("Request size too large: {} bytes", length);
                    return Err(StatusCode::PAYLOAD_TOO_LARGE);
                }
            } else {
                warn!("Invalid content-length header: {}", length_str);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    }

    Ok(())
}

/// Validate content type for POST/PUT requests
fn validate_content_type(method: &Method, headers: &HeaderMap) -> Result<(), StatusCode> {
    if method == Method::POST || method == Method::PUT || method == Method::PATCH {
        if let Some(content_type) = headers.get("content-type") {
            if let Ok(content_type_str) = content_type.to_str() {
                if !content_type_str.starts_with("application/json") {
                    warn!("Unsupported content type: {}", content_type_str);
                    return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
                }
            } else {
                warn!("Invalid content-type header");
                return Err(StatusCode::BAD_REQUEST);
            }
        } else {
            warn!("Missing content-type header for {} request", method);
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    Ok(())
}

/// Validate common headers
fn validate_headers(headers: &HeaderMap) -> Result<(), StatusCode> {
    // Validate User-Agent (should be present)
    if headers.get("user-agent").is_none() {
        debug!("Missing user-agent header");
        // Don't fail for missing user-agent, just log it
    }

    // Validate Accept header if present
    if let Some(accept) = headers.get("accept") {
        if let Ok(accept_str) = accept.to_str() {
            if !accept_str.contains("application/json") && 
               !accept_str.contains("*/*") && 
               !accept_str.contains("application/*") {
                warn!("Unsupported accept header: {}", accept_str);
                return Err(StatusCode::NOT_ACCEPTABLE);
            }
        }
    }

    // Check for suspicious headers
    let suspicious_headers = [
        "x-forwarded-host",
        "x-original-url",
        "x-rewrite-url",
    ];

    for header_name in &suspicious_headers {
        if headers.get(*header_name).is_some() {
            warn!("Suspicious header detected: {}", header_name);
            // Log but don't block - these might be legitimate in some proxy setups
        }
    }

    Ok(())
}

/// Validate URL parameters
fn validate_url_parameters(path: &str, query: Option<&str>) -> Result<(), StatusCode> {
    // Check path length
    if path.len() > 2048 {
        warn!("URL path too long: {} characters", path.len());
        return Err(StatusCode::URI_TOO_LONG);
    }

    // Check for path traversal attempts
    if path.contains("..") || path.contains("//") {
        warn!("Potential path traversal attempt: {}", path);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validate query parameters if present
    if let Some(query_str) = query {
        if query_str.len() > 4096 {
            warn!("Query string too long: {} characters", query_str.len());
            return Err(StatusCode::URI_TOO_LONG);
        }

        // Check for SQL injection patterns
        let suspicious_patterns = [
            "union", "select", "insert", "update", "delete", "drop",
            "exec", "execute", "script", "javascript:", "vbscript:",
            "<script", "</script", "onload", "onerror",
        ];

        let query_lower = query_str.to_lowercase();
        for pattern in &suspicious_patterns {
            if query_lower.contains(pattern) {
                warn!("Suspicious pattern in query string: {}", pattern);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    }

    Ok(())
}

/// Validate JSON payload
pub fn validate_json_payload(payload: &Value) -> Result<(), ValidationError> {
    // Check JSON depth to prevent stack overflow
    if get_json_depth(payload) > 20 {
        return Err(ValidationError::TooDeep);
    }

    // Check for excessively large arrays
    if let Some(array) = payload.as_array() {
        if array.len() > 10000 {
            return Err(ValidationError::ArrayTooLarge);
        }
    }

    // Check for excessively large objects
    if let Some(object) = payload.as_object() {
        if object.len() > 1000 {
            return Err(ValidationError::ObjectTooLarge);
        }

        // Check for suspicious keys
        for key in object.keys() {
            if key.len() > 256 {
                return Err(ValidationError::KeyTooLong);
            }

            // Check for potential injection patterns in keys
            let key_lower = key.to_lowercase();
            if key_lower.contains("__proto__") || 
               key_lower.contains("constructor") || 
               key_lower.contains("prototype") {
                return Err(ValidationError::SuspiciousKey);
            }
        }
    }

    // Check string values
    if let Some(string_val) = payload.as_str() {
        if string_val.len() > 1_000_000 { // 1MB string limit
            return Err(ValidationError::StringTooLong);
        }

        // Check for potential XSS patterns
        let string_lower = string_val.to_lowercase();
        if string_lower.contains("<script") || 
           string_lower.contains("javascript:") || 
           string_lower.contains("vbscript:") {
            return Err(ValidationError::SuspiciousContent);
        }
    }

    Ok(())
}

/// Get JSON nesting depth
fn get_json_depth(value: &Value) -> usize {
    match value {
        Value::Object(obj) => {
            1 + obj.values().map(get_json_depth).max().unwrap_or(0)
        }
        Value::Array(arr) => {
            1 + arr.iter().map(get_json_depth).max().unwrap_or(0)
        }
        _ => 0,
    }
}

/// Validation errors
#[derive(Debug)]
pub enum ValidationError {
    TooDeep,
    ArrayTooLarge,
    ObjectTooLarge,
    KeyTooLong,
    StringTooLong,
    SuspiciousKey,
    SuspiciousContent,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::TooDeep => write!(f, "JSON nesting too deep"),
            ValidationError::ArrayTooLarge => write!(f, "Array too large"),
            ValidationError::ObjectTooLarge => write!(f, "Object too large"),
            ValidationError::KeyTooLong => write!(f, "Object key too long"),
            ValidationError::StringTooLong => write!(f, "String value too long"),
            ValidationError::SuspiciousKey => write!(f, "Suspicious object key detected"),
            ValidationError::SuspiciousContent => write!(f, "Suspicious content detected"),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Sanitize string input
pub fn sanitize_string(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric() || " -_.,()[]{}:;".contains(*c))
        .take(1000) // Limit length
        .collect()
}

/// Validate UUID string
pub fn validate_uuid(uuid_str: &str) -> Result<uuid::Uuid, ValidationError> {
    uuid::Uuid::parse_str(uuid_str)
        .map_err(|_| ValidationError::SuspiciousContent)
}

/// Validate timestamp string
pub fn validate_timestamp(timestamp_str: &str) -> Result<chrono::DateTime<chrono::Utc>, ValidationError> {
    chrono::DateTime::parse_from_rfc3339(timestamp_str)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|_| ValidationError::SuspiciousContent)
}

/// Validate numeric range
pub fn validate_numeric_range<T>(value: T, min: T, max: T) -> Result<T, ValidationError>
where
    T: PartialOrd + Copy,
{
    if value < min || value > max {
        Err(ValidationError::SuspiciousContent)
    } else {
        Ok(value)
    }
}
