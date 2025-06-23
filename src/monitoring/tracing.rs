use opentelemetry::{
    global, sdk::trace as sdktrace, trace::{TraceError, Tracer, TracerProvider},
    KeyValue, Context, trace::{Span, SpanKind, Status},
};
use opentelemetry_jaeger::JaegerTraceRuntime;
use opentelemetry_semantic_conventions as semcov;
use tracing::{info, warn, error, Instrument};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Registry};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Tracing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    pub enabled: bool,
    pub service_name: String,
    pub service_version: String,
    pub environment: String,
    pub jaeger_endpoint: String,
    pub sampling_ratio: f64,
    pub max_events_per_span: u32,
    pub max_attributes_per_span: u32,
    pub max_links_per_span: u32,
    pub enable_logging: bool,
    pub log_level: String,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            service_name: "pantherswap-edge".to_string(),
            service_version: "1.0.0".to_string(),
            environment: "production".to_string(),
            jaeger_endpoint: "http://localhost:14268/api/traces".to_string(),
            sampling_ratio: 1.0,
            max_events_per_span: 128,
            max_attributes_per_span: 128,
            max_links_per_span: 128,
            enable_logging: true,
            log_level: "info".to_string(),
        }
    }
}

/// Custom span attributes for trading operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingSpanAttributes {
    pub operation_type: String,
    pub instrument_id: Option<String>,
    pub order_id: Option<String>,
    pub user_id: Option<String>,
    pub strategy_name: Option<String>,
    pub execution_venue: Option<String>,
    pub order_side: Option<String>,
    pub order_type: Option<String>,
    pub quantity: Option<f64>,
    pub price: Option<f64>,
}

/// Custom span attributes for AI operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AISpanAttributes {
    pub model_type: String,
    pub model_version: Option<String>,
    pub input_features: Option<u32>,
    pub prediction_horizon: Option<String>,
    pub confidence_score: Option<f64>,
    pub cache_hit: Option<bool>,
    pub batch_size: Option<u32>,
}

/// Tracing manager for distributed tracing
pub struct TracingManager {
    config: TracingConfig,
    tracer: Box<dyn Tracer + Send + Sync>,
}

impl TracingManager {
    /// Initialize distributed tracing
    pub fn new(config: TracingConfig) -> Result<Self, TraceError> {
        if !config.enabled {
            // Return a no-op tracer if tracing is disabled
            let tracer = global::tracer("noop");
            return Ok(Self {
                config,
                tracer: Box::new(tracer),
            });
        }

        // Configure Jaeger exporter
        let tracer = opentelemetry_jaeger::new_agent_pipeline()
            .with_service_name(&config.service_name)
            .with_endpoint(&config.jaeger_endpoint)
            .with_trace_config(
                sdktrace::config()
                    .with_sampler(sdktrace::Sampler::TraceIdRatioBased(config.sampling_ratio))
                    .with_max_events_per_span(config.max_events_per_span)
                    .with_max_attributes_per_span(config.max_attributes_per_span)
                    .with_max_links_per_span(config.max_links_per_span)
                    .with_resource(opentelemetry::sdk::Resource::new(vec![
                        KeyValue::new(semcov::resource::SERVICE_NAME, config.service_name.clone()),
                        KeyValue::new(semcov::resource::SERVICE_VERSION, config.service_version.clone()),
                        KeyValue::new(semcov::resource::DEPLOYMENT_ENVIRONMENT, config.environment.clone()),
                    ])),
            )
            .install_batch(JaegerTraceRuntime::Tokio)?;

        Ok(Self {
            config,
            tracer: Box::new(tracer),
        })
    }

    /// Initialize tracing subscriber with OpenTelemetry layer
    pub fn init_subscriber(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.config.enabled {
            // Initialize basic subscriber without OpenTelemetry
            tracing_subscriber::registry()
                .with(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| format!("{}={}", self.config.service_name, self.config.log_level).into()),
                )
                .with(tracing_subscriber::fmt::layer())
                .init();
            return Ok(());
        }

        // Create OpenTelemetry layer
        let telemetry_layer = OpenTelemetryLayer::new(self.tracer.clone());

        // Initialize subscriber with OpenTelemetry
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| format!("{}={}", self.config.service_name, self.config.log_level).into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .with(telemetry_layer)
            .init();

        info!("Distributed tracing initialized with Jaeger endpoint: {}", self.config.jaeger_endpoint);
        Ok(())
    }

    /// Create a new span for trading operations
    pub fn create_trading_span(&self, operation_name: &str, attributes: TradingSpanAttributes) -> impl Span {
        let mut span = self.tracer
            .span_builder(operation_name)
            .with_kind(SpanKind::Internal)
            .start(&self.tracer);

        // Add trading-specific attributes
        span.set_attribute(KeyValue::new("trading.operation_type", attributes.operation_type));
        
        if let Some(instrument_id) = attributes.instrument_id {
            span.set_attribute(KeyValue::new("trading.instrument_id", instrument_id));
        }
        
        if let Some(order_id) = attributes.order_id {
            span.set_attribute(KeyValue::new("trading.order_id", order_id));
        }
        
        if let Some(user_id) = attributes.user_id {
            span.set_attribute(KeyValue::new("trading.user_id", user_id));
        }
        
        if let Some(strategy_name) = attributes.strategy_name {
            span.set_attribute(KeyValue::new("trading.strategy_name", strategy_name));
        }
        
        if let Some(execution_venue) = attributes.execution_venue {
            span.set_attribute(KeyValue::new("trading.execution_venue", execution_venue));
        }
        
        if let Some(order_side) = attributes.order_side {
            span.set_attribute(KeyValue::new("trading.order_side", order_side));
        }
        
        if let Some(order_type) = attributes.order_type {
            span.set_attribute(KeyValue::new("trading.order_type", order_type));
        }
        
        if let Some(quantity) = attributes.quantity {
            span.set_attribute(KeyValue::new("trading.quantity", quantity));
        }
        
        if let Some(price) = attributes.price {
            span.set_attribute(KeyValue::new("trading.price", price));
        }

        span
    }

    /// Create a new span for AI operations
    pub fn create_ai_span(&self, operation_name: &str, attributes: AISpanAttributes) -> impl Span {
        let mut span = self.tracer
            .span_builder(operation_name)
            .with_kind(SpanKind::Internal)
            .start(&self.tracer);

        // Add AI-specific attributes
        span.set_attribute(KeyValue::new("ai.model_type", attributes.model_type));
        
        if let Some(model_version) = attributes.model_version {
            span.set_attribute(KeyValue::new("ai.model_version", model_version));
        }
        
        if let Some(input_features) = attributes.input_features {
            span.set_attribute(KeyValue::new("ai.input_features", input_features as i64));
        }
        
        if let Some(prediction_horizon) = attributes.prediction_horizon {
            span.set_attribute(KeyValue::new("ai.prediction_horizon", prediction_horizon));
        }
        
        if let Some(confidence_score) = attributes.confidence_score {
            span.set_attribute(KeyValue::new("ai.confidence_score", confidence_score));
        }
        
        if let Some(cache_hit) = attributes.cache_hit {
            span.set_attribute(KeyValue::new("ai.cache_hit", cache_hit));
        }
        
        if let Some(batch_size) = attributes.batch_size {
            span.set_attribute(KeyValue::new("ai.batch_size", batch_size as i64));
        }

        span
    }

    /// Create a new span for database operations
    pub fn create_db_span(&self, operation_name: &str, table: &str, query_type: &str) -> impl Span {
        let mut span = self.tracer
            .span_builder(operation_name)
            .with_kind(SpanKind::Client)
            .start(&self.tracer);

        span.set_attribute(KeyValue::new(semcov::trace::DB_SYSTEM, "postgresql"));
        span.set_attribute(KeyValue::new(semcov::trace::DB_NAME, "pantherswap_edge"));
        span.set_attribute(KeyValue::new(semcov::trace::DB_OPERATION, query_type));
        span.set_attribute(KeyValue::new("db.table", table));

        span
    }

    /// Create a new span for HTTP operations
    pub fn create_http_span(&self, method: &str, url: &str, user_agent: Option<&str>) -> impl Span {
        let mut span = self.tracer
            .span_builder(format!("{} {}", method, url))
            .with_kind(SpanKind::Server)
            .start(&self.tracer);

        span.set_attribute(KeyValue::new(semcov::trace::HTTP_METHOD, method.to_string()));
        span.set_attribute(KeyValue::new(semcov::trace::HTTP_URL, url.to_string()));
        
        if let Some(ua) = user_agent {
            span.set_attribute(KeyValue::new(semcov::trace::HTTP_USER_AGENT, ua.to_string()));
        }

        span
    }

    /// Add error information to span
    pub fn record_error(&self, span: &mut dyn Span, error: &dyn std::error::Error) {
        span.set_status(Status::Error {
            description: error.to_string().into(),
        });
        span.set_attribute(KeyValue::new("error", true));
        span.set_attribute(KeyValue::new("error.message", error.to_string()));
        span.add_event("exception", vec![
            KeyValue::new("exception.type", std::any::type_name_of_val(error)),
            KeyValue::new("exception.message", error.to_string()),
        ]);
    }

    /// Add custom event to span
    pub fn add_event(&self, span: &mut dyn Span, name: &str, attributes: Vec<KeyValue>) {
        span.add_event(name, attributes);
    }

    /// Get current trace ID for correlation
    pub fn current_trace_id(&self) -> Option<String> {
        let context = Context::current();
        let span_context = context.span().span_context();
        if span_context.is_valid() {
            Some(span_context.trace_id().to_string())
        } else {
            None
        }
    }

    /// Shutdown tracing and flush remaining spans
    pub async fn shutdown(&self) -> Result<(), TraceError> {
        if self.config.enabled {
            global::shutdown_tracer_provider();
        }
        Ok(())
    }
}

/// Middleware for automatic HTTP request tracing
pub async fn tracing_middleware(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let method = request.method().to_string();
    let uri = request.uri().to_string();
    let user_agent = request.headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok());

    // Create span for this request
    let span = tracing::info_span!(
        "http_request",
        http.method = %method,
        http.url = %uri,
        http.user_agent = user_agent.unwrap_or("unknown"),
        otel.kind = "server"
    );

    // Execute request within span
    async move {
        let start_time = std::time::Instant::now();
        let response = next.run(request).await;
        let duration = start_time.elapsed();

        // Add response attributes to span
        tracing::Span::current().record("http.status_code", response.status().as_u16());
        tracing::Span::current().record("http.response_time_ms", duration.as_millis() as u64);

        if response.status().is_client_error() || response.status().is_server_error() {
            tracing::Span::current().record("error", true);
        }

        response
    }
    .instrument(span)
    .await
}

/// Utility macro for creating traced functions
#[macro_export]
macro_rules! traced_function {
    ($name:expr, $($key:expr => $value:expr),*) => {
        tracing::info_span!($name, $($key = $value),*)
    };
}

/// Utility function to extract trace context from headers
pub fn extract_trace_context(headers: &axum::http::HeaderMap) -> Context {
    use opentelemetry::propagation::Extractor;
    
    struct HeaderExtractor<'a>(&'a axum::http::HeaderMap);
    
    impl<'a> Extractor for HeaderExtractor<'a> {
        fn get(&self, key: &str) -> Option<&str> {
            self.0.get(key).and_then(|v| v.to_str().ok())
        }
        
        fn keys(&self) -> Vec<&str> {
            self.0.keys().map(|k| k.as_str()).collect()
        }
    }
    
    let extractor = HeaderExtractor(headers);
    global::get_text_map_propagator(|propagator| {
        propagator.extract(&extractor)
    })
}
