use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod database;
mod market_data;
mod microstructure;
mod ai;
mod trading;
mod api;
mod monitoring;
mod utils;

use config::Settings;
use database::Database;
use market_data::MarketDataManager;
use microstructure::MicrostructureEngine;
use ai::AIEngine;
use trading::engine::{TradingEngine, TradingEngineConfig};
use api::{create_app, start_server, AppState};
use monitoring::{ProductionMonitor, ProductionMonitoringConfig};
use ai::monitoring::{create_ai_performance_monitor};
use database::health_monitor::{DatabaseHealthMonitor, HealthMonitorConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Initialize logging
    setup_logging()?;

    info!("🚀 Starting PantherSwap Edge - Market Microstructure Intelligence Platform");

    // Load configuration
    let settings = Settings::load()?;
    info!("✅ Configuration loaded successfully");

    // Initialize database
    let database = Database::new(&settings.database.url).await?;
    database.run_migrations().await?;
    info!("✅ Database connected and migrations applied");

    // Initialize core engines
    let app_components = initialize_components(settings.clone(), database.clone()).await?;
    info!("✅ All components initialized successfully");

    // Start production monitoring
    info!("📊 Starting Production Monitoring System...");
    app_components.production_monitor.start_monitoring().await?;
    info!("✅ Production Monitoring System started");

    info!("🌟 PantherSwap Edge is now running!");
    info!("📊 Market Microstructure Intelligence: ACTIVE");
    info!("🤖 AI Prediction Engine: ACTIVE");
    info!("⚡ Trading Signal Generation: ACTIVE");
    info!("🔒 Risk Management: ACTIVE");
    info!("📊 Production Monitoring: ACTIVE");
    info!("🌐 API Server: http://{}:{}", settings.server.host, settings.server.port);

    // Start API server with monitoring integration
    start_api_server(app_components, settings.server.port).await?;

    info!("🛑 PantherSwap Edge shutdown complete");
    Ok(())
}

fn setup_logging() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx=warn,hyper=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(())
}

#[derive(Clone)]
struct ApplicationComponents {
    pub database: Database,
    pub market_data_manager: Arc<Mutex<MarketDataManager>>,
    pub microstructure_engine: Arc<Mutex<MicrostructureEngine>>,
    pub ai_engine: Arc<Mutex<AIEngine>>,
    pub trading_engine: Arc<Mutex<TradingEngine>>,
    pub production_monitor: Arc<ProductionMonitor>,
    pub settings: Settings,
}

async fn initialize_components(
    settings: Settings,
    database: Database
) -> Result<ApplicationComponents> {
    info!("🔧 Initializing system components...");

    // Initialize market data manager
    info!("📈 Initializing Market Data Manager...");
    let market_data_manager = MarketDataManager::new(&settings, database.clone()).await?;
    info!("✅ Market Data Manager initialized");

    // Initialize microstructure engine
    info!("🧬 Initializing Microstructure Analysis Engine...");
    let microstructure_engine = MicrostructureEngine::new().await?;
    info!("✅ Microstructure Engine initialized");

    // Initialize AI engine
    info!("🤖 Initializing AI Prediction Engine...");
    let ai_engine = AIEngine::new(database.clone()).await?;
    info!("✅ AI Engine initialized");

    // Initialize trading engine
    info!("⚡ Initializing Trading Engine...");
    let trading_config = TradingEngineConfig {
        enable_live_trading: settings.trading.enable_live_trading,
        max_position_size: settings.trading.max_position_size,
        confidence_threshold: settings.trading.confidence_threshold,
        signal_generation_interval_ms: settings.trading.signal_generation_interval_ms,
        risk_check_interval_ms: settings.trading.risk_check_interval_ms,
        ..TradingEngineConfig::default()
    };
    let trading_engine = TradingEngine::new(trading_config, database.clone()).await?;
    info!("✅ Trading Engine initialized");

    // Initialize production monitoring
    info!("📊 Initializing Production Monitoring...");
    let ai_monitor = Arc::new(create_ai_performance_monitor(database.clone()));
    let db_monitor = Arc::new(DatabaseHealthMonitor::with_defaults(database.pool.clone()));
    let trading_engine_arc = Arc::new(tokio::sync::RwLock::new(trading_engine));

    let monitoring_config = ProductionMonitoringConfig::default();
    let production_monitor = Arc::new(ProductionMonitor::new(
        monitoring_config,
        ai_monitor,
        db_monitor,
        trading_engine_arc.clone(),
    ));
    info!("✅ Production Monitoring initialized");

    Ok(ApplicationComponents {
        database,
        market_data_manager: Arc::new(Mutex::new(market_data_manager)),
        microstructure_engine: Arc::new(Mutex::new(microstructure_engine)),
        ai_engine: Arc::new(Mutex::new(ai_engine)),
        trading_engine: trading_engine_arc,
        production_monitor,
        settings,
    })
}

async fn start_api_server(components: ApplicationComponents, port: u16) -> Result<()> {
    info!("🌐 Starting API server on port {}...", port);

    let app_state = AppState {
        database: components.database,
        trading_engine: components.trading_engine,
        ai_engine: components.ai_engine,
        production_monitor: components.production_monitor,
    };

    let app = create_app(app_state).await;
    start_server(app, port).await?;

    Ok(())
}
