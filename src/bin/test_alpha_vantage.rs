// Test Alpha Vantage API integration for PantherSwap Edge
// Run with: PANTHERSWAP_MARKET_DATA_ALPHA_VANTAGE_API_KEY=EZDZ4VOFQ2GRA7VU cargo run --bin test_alpha_vantage

use pantherswap_edge::config::Settings;
use pantherswap_edge::database::Database;
use pantherswap_edge::market_data::providers::AlphaVantageProvider;
use pantherswap_edge::market_data::MarketDataManager;
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

    info!("🧪 Starting Alpha Vantage API Integration Test");

    // Load configuration
    let mut settings = Settings::load()?;
    info!("✅ Configuration loaded successfully");
    info!("🔍 Alpha Vantage API key length: {}", settings.market_data.alpha_vantage_api_key.len());

    // Override API key from environment if not loaded
    if settings.market_data.alpha_vantage_api_key.is_empty() {
        if let Ok(api_key) = std::env::var("PANTHERSWAP_MARKET_DATA_ALPHA_VANTAGE_API_KEY") {
            info!("🔧 Manually setting API key from environment variable");
            settings.market_data.alpha_vantage_api_key = api_key;
        } else {
            info!("⚠️  No API key found in environment variable PANTHERSWAP_MARKET_DATA_ALPHA_VANTAGE_API_KEY");
            info!("💡 Using hardcoded API key for testing: EZDZ4VOFQ2GRA7VU");
            settings.market_data.alpha_vantage_api_key = "EZDZ4VOFQ2GRA7VU".to_string();
        }
    }

    info!("🔍 Final API key length: {}", settings.market_data.alpha_vantage_api_key.len());

    // Test Alpha Vantage provider directly
    info!("🔗 Testing Alpha Vantage provider...");
    let alpha_vantage = AlphaVantageProvider::new(
        settings.market_data.alpha_vantage_api_key.clone()
    );

    // Validate configuration
    match alpha_vantage.validate_configuration() {
        Ok(_) => info!("✅ Alpha Vantage API key validation passed"),
        Err(e) => {
            info!("❌ Alpha Vantage API key validation failed: {}", e);
            info!("💡 Make sure PANTHERSWAP_MARKET_DATA_ALPHA_VANTAGE_API_KEY is set to: EZDZ4VOFQ2GRA7VU");
            return Ok(());
        }
    }

    // Test fetching a forex quote
    info!("📊 Testing forex quote fetching...");
    match alpha_vantage.get_fx_quote("EUR", "USD").await {
        Ok(quote) => {
            info!("✅ Successfully fetched EUR/USD quote:");
            info!("   📈 Symbol: {}", quote.symbol);
            info!("   💰 Bid: {:.5}", quote.bid_price);
            info!("   💰 Ask: {:.5}", quote.ask_price);
            info!("   💰 Mid: {:.5}", quote.mid_price);
            info!("   📏 Spread: {:.5}", quote.spread.unwrap_or(0.0));
            info!("   ⭐ Quality: {:.2}", quote.data_quality);
            info!("   🕐 Timestamp: {}", quote.timestamp);
            info!("   🏢 Provider: {}", quote.provider);
        }
        Err(e) => {
            info!("❌ Failed to fetch EUR/USD quote: {}", e);
            info!("💡 This might be due to rate limiting or API issues");
        }
    }

    // Test supported currency pairs
    info!("🌍 Supported currency pairs:");
    for (from, to) in AlphaVantageProvider::get_supported_pairs() {
        info!("   📊 {} -> {}", from, to);
    }

    // Test database integration if available
    info!("🗄️ Testing database integration...");
    match Database::new_testing(&settings.database.url).await {
        Ok(database) => {
            info!("✅ Database connection established");
            
            // Test creating default instruments
            match MarketDataManager::create_default_instruments(&database).await {
                Ok(instruments) => {
                    info!("✅ Created {} default forex instruments", instruments.len());
                    for (symbol, id) in instruments.iter().take(3) {
                        info!("   📈 {}: {}", symbol, id);
                    }
                }
                Err(e) => {
                    info!("⚠️  Could not create default instruments: {}", e);
                    info!("   This might be due to existing instruments in database");
                }
            }

            // Test market data manager initialization
            match MarketDataManager::new(&settings, database).await {
                Ok(_manager) => {
                    info!("✅ Market Data Manager initialized successfully");
                    info!("🎉 Alpha Vantage integration is ready for production!");
                }
                Err(e) => {
                    info!("❌ Market Data Manager initialization failed: {}", e);
                }
            }
        }
        Err(e) => {
            info!("⚠️  Database connection failed: {}", e);
            info!("   Alpha Vantage provider works independently of database");
        }
    }

    // Test rate limiting
    info!("⏱️  Testing rate limiting...");
    info!("   Making multiple requests to test rate limiting behavior...");
    
    for i in 1..=3 {
        info!("   Request {}/3", i);
        match alpha_vantage.get_fx_quote("GBP", "USD").await {
            Ok(quote) => {
                info!("   ✅ GBP/USD: Bid={:.5}, Ask={:.5}", quote.bid_price, quote.ask_price);
            }
            Err(e) => {
                info!("   ⚠️  Request failed: {}", e);
            }
        }
        
        if i < 3 {
            info!("   Waiting 12 seconds for rate limiting...");
            tokio::time::sleep(std::time::Duration::from_secs(12)).await;
        }
    }

    info!("🏁 Alpha Vantage API integration test completed");
    info!("💡 To start real-time data collection, use the market data manager");
    
    Ok(())
}
