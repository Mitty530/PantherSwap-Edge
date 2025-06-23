// Live System Test Runner for PantherSwap Edge
// Runs comprehensive integration tests with live IG API and TimescaleDB

use std::process;
use tracing::{info, error};

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info,sqlx=warn,hyper=warn")
        .init();

    info!("🚀 Starting PantherSwap Edge Live System Integration Test");

    // Import and run the test
    match pantherswap_edge::testing::live_system_integration_test::run_live_system_test().await {
        Ok(results) => {
            println!("\n🎯 LIVE SYSTEM INTEGRATION TEST RESULTS");
            println!("==========================================");
            println!("Overall Status: {}", results.overall_status);
            println!("Test Duration: {:.2}s", results.test_duration_seconds);
            
            // Print summary
            let success_rate = calculate_success_rate(&results);
            println!("Success Rate: {:.1}%", success_rate * 100.0);
            
            if success_rate >= 0.8 {
                println!("✅ System is ready for production deployment");
                process::exit(0);
            } else if success_rate >= 0.6 {
                println!("⚠️ System has minor issues but is functional");
                process::exit(1);
            } else {
                println!("❌ System has critical issues requiring attention");
                process::exit(2);
            }
        }
        Err(e) => {
            error!("❌ Live system test failed: {}", e);
            process::exit(3);
        }
    }
}

fn calculate_success_rate(results: &pantherswap_edge::testing::live_system_integration_test::SystemTestResults) -> f64 {
    let mut passed = 0;
    let total = 6;

    if results.database_health.connection_successful && results.database_health.pool_health {
        passed += 1;
    }

    if results.ig_api_connectivity.authentication_successful {
        passed += 1;
    }

    if results.market_data_pipeline.data_collection_active {
        passed += 1;
    }

    if results.ai_engine_performance.inference_successful {
        passed += 1;
    }

    if results.trading_engine_performance.signal_generation {
        passed += 1;
    }

    if results.end_to_end_performance.complete_cycle_successful {
        passed += 1;
    }

    passed as f64 / total as f64
}
