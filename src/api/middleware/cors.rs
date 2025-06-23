use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
    http::{StatusCode, HeaderMap, HeaderValue, Method},
};
use tracing::{debug, warn};

/// CORS configuration
#[derive(Debug, Clone)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<Method>,
    pub allowed_headers: Vec<String>,
    pub exposed_headers: Vec<String>,
    pub allow_credentials: bool,
    pub max_age: Option<u64>,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec![
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::PATCH,
                Method::OPTIONS,
            ],
            allowed_headers: vec![
                "accept".to_string(),
                "authorization".to_string(),
                "content-type".to_string(),
                "user-agent".to_string(),
                "x-api-key".to_string(),
                "x-request-id".to_string(),
            ],
            exposed_headers: vec![
                "x-request-id".to_string(),
                "x-rate-limit-remaining".to_string(),
                "x-rate-limit-reset".to_string(),
            ],
            allow_credentials: false,
            max_age: Some(86400), // 24 hours
        }
    }
}

impl CorsConfig {
    /// Create a production-ready CORS configuration
    pub fn production() -> Self {
        Self {
            allowed_origins: vec![
                "https://app.pantherswap.com".to_string(),
                "https://dashboard.pantherswap.com".to_string(),
                "https://api.pantherswap.com".to_string(),
            ],
            allowed_methods: vec![
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::PATCH,
                Method::OPTIONS,
            ],
            allowed_headers: vec![
                "accept".to_string(),
                "authorization".to_string(),
                "content-type".to_string(),
                "user-agent".to_string(),
                "x-api-key".to_string(),
                "x-request-id".to_string(),
            ],
            exposed_headers: vec![
                "x-request-id".to_string(),
                "x-rate-limit-remaining".to_string(),
                "x-rate-limit-reset".to_string(),
            ],
            allow_credentials: true,
            max_age: Some(3600), // 1 hour
        }
    }

    /// Create a development CORS configuration (more permissive)
    pub fn development() -> Self {
        Self {
            allowed_origins: vec![
                "*".to_string(),
                "http://localhost:3000".to_string(),
                "http://localhost:8080".to_string(),
                "http://127.0.0.1:3000".to_string(),
                "http://127.0.0.1:8080".to_string(),
            ],
            ..Default::default()
        }
    }
}

/// CORS middleware
pub async fn cors_middleware(
    request: Request,
    next: Next,
) -> Response {
    let config = get_cors_config();
    
    // Handle preflight requests
    if request.method() == Method::OPTIONS {
        return handle_preflight_request(&request, &config);
    }

    // Process the request
    let mut response = next.run(request).await;

    // Add CORS headers to the response
    add_cors_headers(&mut response, &config);

    response
}

/// Handle preflight OPTIONS requests
fn handle_preflight_request(request: &Request, config: &CorsConfig) -> Response {
    let mut response = Response::builder()
        .status(StatusCode::NO_CONTENT)
        .body(axum::body::Body::empty())
        .unwrap();

    let headers = response.headers_mut();

    // Add CORS headers
    add_cors_headers_to_map(headers, config, Some(request.headers()));

    debug!("Handled CORS preflight request");
    response
}

/// Add CORS headers to response
fn add_cors_headers(response: &mut Response, config: &CorsConfig) {
    add_cors_headers_to_map(response.headers_mut(), config, None);
}

/// Add CORS headers to header map
fn add_cors_headers_to_map(
    headers: &mut HeaderMap,
    config: &CorsConfig,
    request_headers: Option<&HeaderMap>,
) {
    // Access-Control-Allow-Origin
    let origin = if config.allowed_origins.contains(&"*".to_string()) {
        "*"
    } else if let Some(req_headers) = request_headers {
        if let Some(origin_header) = req_headers.get("origin") {
            if let Ok(origin_str) = origin_header.to_str() {
                if config.allowed_origins.contains(&origin_str.to_string()) {
                    origin_str
                } else {
                    // Origin not allowed
                    warn!("Origin not allowed: {}", origin_str);
                    return;
                }
            } else {
                "*"
            }
        } else {
            "*"
        }
    } else {
        "*"
    };

    if let Ok(origin_value) = HeaderValue::from_str(origin) {
        headers.insert("access-control-allow-origin", origin_value);
    }

    // Access-Control-Allow-Methods
    let methods: Vec<String> = config.allowed_methods
        .iter()
        .map(|m| m.to_string())
        .collect();
    let methods_str = methods.join(", ");
    if let Ok(methods_value) = HeaderValue::from_str(&methods_str) {
        headers.insert("access-control-allow-methods", methods_value);
    }

    // Access-Control-Allow-Headers
    let headers_str = config.allowed_headers.join(", ");
    if let Ok(headers_value) = HeaderValue::from_str(&headers_str) {
        headers.insert("access-control-allow-headers", headers_value);
    }

    // Access-Control-Expose-Headers
    if !config.exposed_headers.is_empty() {
        let exposed_str = config.exposed_headers.join(", ");
        if let Ok(exposed_value) = HeaderValue::from_str(&exposed_str) {
            headers.insert("access-control-expose-headers", exposed_value);
        }
    }

    // Access-Control-Allow-Credentials
    if config.allow_credentials {
        headers.insert("access-control-allow-credentials", HeaderValue::from_static("true"));
    }

    // Access-Control-Max-Age
    if let Some(max_age) = config.max_age {
        if let Ok(max_age_value) = HeaderValue::from_str(&max_age.to_string()) {
            headers.insert("access-control-max-age", max_age_value);
        }
    }

    // Additional security headers
    headers.insert("vary", HeaderValue::from_static("Origin, Access-Control-Request-Method, Access-Control-Request-Headers"));
}

/// Get CORS configuration based on environment
fn get_cors_config() -> CorsConfig {
    // In a real application, this would be configurable via environment variables
    if cfg!(debug_assertions) {
        CorsConfig::development()
    } else {
        CorsConfig::production()
    }
}

/// Validate origin against allowed origins
pub fn is_origin_allowed(origin: &str, config: &CorsConfig) -> bool {
    if config.allowed_origins.contains(&"*".to_string()) {
        return true;
    }

    config.allowed_origins.contains(&origin.to_string())
}

/// Validate method against allowed methods
pub fn is_method_allowed(method: &Method, config: &CorsConfig) -> bool {
    config.allowed_methods.contains(method)
}

/// Validate header against allowed headers
pub fn is_header_allowed(header: &str, config: &CorsConfig) -> bool {
    let header_lower = header.to_lowercase();
    config.allowed_headers.iter().any(|h| h.to_lowercase() == header_lower)
}

/// Security headers middleware (can be combined with CORS)
pub async fn security_headers_middleware(
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    
    let headers = response.headers_mut();
    
    // Security headers
    headers.insert("x-content-type-options", HeaderValue::from_static("nosniff"));
    headers.insert("x-frame-options", HeaderValue::from_static("DENY"));
    headers.insert("x-xss-protection", HeaderValue::from_static("1; mode=block"));
    headers.insert("referrer-policy", HeaderValue::from_static("strict-origin-when-cross-origin"));
    headers.insert("content-security-policy", HeaderValue::from_static("default-src 'self'"));
    
    // Remove server information
    headers.remove("server");
    
    response
}
