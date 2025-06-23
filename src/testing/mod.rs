// Testing Module for PantherSwap Edge
// Provides comprehensive testing frameworks for live trading simulation and validation

pub mod live_trading_simulation;
pub mod alpaca_integration_tests;

pub use live_trading_simulation::{
    LiveTradingSimulator,
    LiveTradingSimulationConfig,
    SimulationTestReport,
    SimulationMetrics,
    DatabasePerformanceReport,
    TradingPerformanceReport,
    SystemPerformanceReport,
    ValidationResults,
};

pub use alpaca_integration_tests::{
    AlpacaIntegrationTestSuite,
    TestConfiguration,
    TestResults,
    PerformanceTargets,
    ConnectivityTestResults,
    MarketDataTestResults,
    OrderExecutionTestResults,
    DatabaseTestResults,
    PerformanceTestResults,
    ErrorHandlingTestResults,
};

use crate::utils::Result;
use tracing::info;

/// Execute comprehensive live trading simulation test
pub async fn execute_live_trading_test() -> Result<SimulationTestReport> {
    info!("🚀 Starting comprehensive live trading simulation test...");
    
    // Create simulation configuration
    let config = LiveTradingSimulationConfig::default();
    
    // Initialize simulator
    let mut simulator = LiveTradingSimulator::new(config).await?;
    
    // Validate system components
    let validation_passed = simulator.validate_system_components().await?;
    if !validation_passed {
        info!("⚠️  Some validation checks failed, but proceeding with simulation");
    }
    
    // Initialize trading components
    simulator.initialize_trading_components().await?;
    
    // Execute simulation
    let report = simulator.execute_simulation().await?;
    
    info!("✅ Live trading simulation test completed");
    Ok(report)
}

/// Quick system validation test
pub async fn quick_system_validation() -> Result<bool> {
    info!("🔍 Running quick system validation...");
    
    let config = LiveTradingSimulationConfig::default();
    let simulator = LiveTradingSimulator::new(config).await?;
    
    let validation_result = simulator.validate_system_components().await?;
    
    if validation_result {
        info!("✅ Quick system validation passed");
    } else {
        info!("⚠️  Quick system validation found issues");
    }
    
    Ok(validation_result)
}

/// Execute comprehensive Alpaca integration test suite
pub async fn execute_alpaca_integration_tests() -> Result<TestResults> {
    info!("🚀 Starting Alpaca integration test suite...");

    // Load configuration
    let settings = crate::config::Settings::load()?;

    // Initialize database
    let database = crate::database::Database::new(&settings.database).await?;

    // Create test suite
    let mut test_suite = AlpacaIntegrationTestSuite::new(settings, database).await?;

    // Initialize components
    test_suite.initialize_components().await?;

    // Run complete test suite
    let results = test_suite.run_complete_test_suite().await?;

    // Generate and log report
    let report = test_suite.generate_test_report(&results);
    info!("📊 Test Report:\n{}", report);

    info!("✅ Alpaca integration test suite completed");
    Ok(results)
}

/// Quick Alpaca connectivity test
pub async fn quick_alpaca_connectivity_test() -> Result<bool> {
    info!("🔍 Running quick Alpaca connectivity test...");

    let settings = crate::config::Settings::load()?;
    let database = crate::database::Database::new(&settings.database).await?;

    let mut test_suite = AlpacaIntegrationTestSuite::new(settings, database).await?;
    test_suite.initialize_components().await?;

    // Run only connectivity tests
    let connectivity_results = test_suite.run_connectivity_tests().await?;

    let success = connectivity_results.alpaca_api_connection &&
                  connectivity_results.account_access &&
                  connectivity_results.authentication_valid;

    if success {
        info!("✅ Alpaca connectivity test passed");
    } else {
        info!("⚠️  Alpaca connectivity test found issues");
    }

    Ok(success)
}
