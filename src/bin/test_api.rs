// Test REST API Endpoints for PantherSwap Edge
// Run with: cargo run --bin test_api

use pantherswap_edge::config::Settings;
use pantherswap_edge::database::Database;
use pantherswap_edge::trading::TradingEngine;
use pantherswap_edge::ai::AIEngine;
use pantherswap_edge::api::{AppState, create_app, start_server};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("🧪 Starting REST API Test Server");

    // Load configuration
    let settings = Settings::load()?;
    info!("✅ Configuration loaded successfully");

    // Initialize database
    info!("🔗 Connecting to database...");
    let database = match Database::new_testing(&settings.database.url).await {
        Ok(db) => {
            info!("✅ Database connection established");
            db
        }
        Err(e) => {
            info!("❌ Database connection failed: {}", e);
            info!("💡 API will start but database-dependent endpoints will fail");
            // Create a mock database for testing
            Database::new_testing("postgresql://localhost/mock").await
                .unwrap_or_else(|_| panic!("Failed to create mock database"))
        }
    };

    // Initialize trading engine
    info!("🔧 Initializing trading engine...");
    let trading_config = pantherswap_edge::trading::TradingEngineConfig::default();
    let trading_engine = Arc::new(Mutex::new(
        TradingEngine::new(trading_config, database.clone()).await?
    ));
    info!("✅ Trading engine initialized");

    // Initialize AI engine
    info!("🤖 Initializing AI engine...");
    let ai_engine = Arc::new(Mutex::new(
        AIEngine::new(database.clone()).await?
    ));
    info!("✅ AI engine initialized");

    // Create application state
    let app_state = AppState {
        database,
        trading_engine,
        ai_engine,
    };

    // Create the application with all routes and middleware
    info!("🚀 Creating API application...");
    let app = create_app(app_state).await;
    info!("✅ API application created with all routes and middleware");

    // Use default port since it's not in config
    let port = 8080;

    // Print API endpoints information
    print_api_info(port);

    // Start the server
    info!("🌐 Starting API server on port {}...", port);
    start_server(app, port).await?;

    Ok(())
}

fn print_api_info(port: u16) {
    let base_url = format!("http://localhost:{}", port);
    
    info!("🎯 PantherSwap Edge REST API Endpoints:");
    info!("");
    info!("📊 Health & Monitoring:");
    info!("  GET  {}/health                    - Basic health check", base_url);
    info!("  GET  {}/health/liveness           - Kubernetes liveness probe", base_url);
    info!("  GET  {}/health/readiness          - Kubernetes readiness probe", base_url);
    info!("  GET  {}/status                    - Detailed system status", base_url);
    info!("  GET  {}/metrics                   - System metrics", base_url);
    info!("");
    info!("🔧 Instruments API:");
    info!("  GET  {}/api/v1/instruments        - List all instruments", base_url);
    info!("  GET  {}/api/v1/instruments/{{id}}   - Get specific instrument", base_url);
    info!("  POST {}/api/v1/instruments        - Create new instrument", base_url);
    info!("  PUT  {}/api/v1/instruments/{{id}}   - Update instrument", base_url);
    info!("");
    info!("📈 Market Data API:");
    info!("  GET  {}/api/v1/market-data/ticks  - Get market ticks", base_url);
    info!("  GET  {}/api/v1/market-data/latest - Get latest ticks", base_url);
    info!("  GET  {}/api/v1/market-data/ohlc   - Get OHLC data", base_url);
    info!("  GET  {}/api/v1/market-data/stats  - Get market statistics", base_url);
    info!("");
    info!("🔐 Authentication:");
    info!("  Use one of these API keys in the 'Authorization: Bearer <key>' header:");
    info!("  - demo-admin-key     (Admin access - full permissions)");
    info!("  - demo-trader-key    (Trader access - read/write trading data)");
    info!("  - demo-readonly-key  (Read-only access)");
    info!("");
    info!("📝 Example API Calls:");
    info!("");
    info!("  # Health check (no auth required)");
    info!("  curl {}/health", base_url);
    info!("");
    info!("  # Get instruments (auth required)");
    info!("  curl -H 'Authorization: Bearer demo-admin-key' \\");
    info!("       {}/api/v1/instruments", base_url);
    info!("");
    info!("  # Get latest market data (auth required)");
    info!("  curl -H 'Authorization: Bearer demo-readonly-key' \\");
    info!("       {}/api/v1/market-data/latest", base_url);
    info!("");
    info!("  # Create new instrument (admin auth required)");
    info!("  curl -X POST \\");
    info!("       -H 'Authorization: Bearer demo-admin-key' \\");
    info!("       -H 'Content-Type: application/json' \\");
    info!("       -d '{{");
    info!("         \"symbol\": \"GBPJPY\",");
    info!("         \"name\": \"British Pound / Japanese Yen\",");
    info!("         \"instrument_type\": \"forex\",");
    info!("         \"base_currency\": \"GBP\",");
    info!("         \"quote_currency\": \"JPY\",");
    info!("         \"tick_size\": 0.001,");
    info!("         \"lot_size\": 100000.0");
    info!("       }}' \\");
    info!("       {}/api/v1/instruments", base_url);
    info!("");
    info!("🔒 Security Features:");
    info!("  ✅ API Key Authentication");
    info!("  ✅ Role-based Access Control (Admin, Trader, ReadOnly)");
    info!("  ✅ Rate Limiting (per user and per IP)");
    info!("  ✅ Request Validation & Sanitization");
    info!("  ✅ CORS Protection");
    info!("  ✅ Security Headers");
    info!("  ✅ Request/Response Logging");
    info!("");
    info!("📊 Rate Limits:");
    info!("  - Admin:    300 req/min, 10,000 req/hour, 50 burst");
    info!("  - Trader:   120 req/min,  5,000 req/hour, 20 burst");
    info!("  - ReadOnly:  60 req/min,  2,000 req/hour, 10 burst");
    info!("  - IP-based:  10 req/min,    100 req/hour,  3 burst (unauthenticated)");
    info!("");
    info!("🎉 API server is ready for testing!");
    info!("   Press Ctrl+C to stop the server");
}
