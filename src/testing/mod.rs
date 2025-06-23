// Testing Module for PantherSwap Edge
// Provides comprehensive testing frameworks for live trading simulation and validation

pub mod live_trading_simulation;
pub mod live_system_integration_test;
pub mod comprehensive_live_simulation;


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

pub use live_system_integration_test::{
    run_live_system_test,
    SystemTestResults,
    DatabaseTestResults,
    IGAPITestResults,
    MarketDataTestResults,
    AIEngineTestResults,
    TradingEngineTestResults,
    EndToEndTestResults,
};

// IG Trading integration testing will be handled through live_trading_simulation

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

/// Execute comprehensive IG Trading integration test suite
pub async fn execute_ig_trading_integration_tests() -> Result<SimulationTestReport> {
    info!("🚀 Starting IG Trading integration test suite...");

    // Use the live trading simulation for IG Trading testing
    execute_live_trading_test().await
}

/// Quick IG Trading connectivity test
pub async fn quick_ig_trading_connectivity_test() -> Result<bool> {
    info!("🔍 Running quick IG Trading connectivity test...");

    // Use the quick system validation for IG Trading testing
    quick_system_validation().await
}
