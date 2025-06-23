// Test Runner for PantherSwap Edge Integration Tests
// Run with: cargo test --test test_runner

use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;

/// Run all integration tests in sequence
#[tokio::test]
async fn run_all_integration_tests() {
    println!("🧪 Starting PantherSwap Edge Integration Test Suite");
    println!("=" .repeat(60));

    // Test categories to run
    let test_categories = vec![
        ("Basic Integration Tests", "integration_tests"),
        ("Authentication Tests", "auth_tests"),
        ("Rate Limiting Tests", "rate_limit_tests"),
        ("API Endpoint Tests", "api_endpoint_tests"),
        ("Database Tests", "database_tests"),
    ];

    let mut total_passed = 0;
    let mut total_failed = 0;

    for (category_name, test_file) in test_categories {
        println!("\n📋 Running: {}", category_name);
        println!("-".repeat(40));

        let output = Command::new("cargo")
            .args(&["test", "--test", test_file, "--", "--nocapture"])
            .output();

        match output {
            Ok(result) => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                let stderr = String::from_utf8_lossy(&result.stderr);

                if result.status.success() {
                    println!("✅ {} - PASSED", category_name);
                    total_passed += 1;
                } else {
                    println!("❌ {} - FAILED", category_name);
                    total_failed += 1;
                }

                // Print test output
                if !stdout.is_empty() {
                    println!("📄 Output:");
                    println!("{}", stdout);
                }

                if !stderr.is_empty() {
                    println!("⚠️  Errors:");
                    println!("{}", stderr);
                }
            }
            Err(e) => {
                println!("❌ Failed to run {}: {}", category_name, e);
                total_failed += 1;
            }
        }

        // Small delay between test categories
        sleep(Duration::from_millis(500)).await;
    }

    // Summary
    println!("\n" + &"=".repeat(60));
    println!("🎯 Integration Test Summary");
    println!("=" .repeat(60));
    println!("✅ Passed: {}", total_passed);
    println!("❌ Failed: {}", total_failed);
    println!("📊 Total:  {}", total_passed + total_failed);

    if total_failed == 0 {
        println!("\n🎉 All integration tests passed!");
    } else {
        println!("\n⚠️  Some tests failed. Check output above for details.");
    }

    println!("\n📝 Test Categories:");
    println!("  • Basic Integration Tests - Core API functionality");
    println!("  • Authentication Tests - API key validation and RBAC");
    println!("  • Rate Limiting Tests - Request throttling and limits");
    println!("  • API Endpoint Tests - All REST endpoints");
    println!("  • Database Tests - CRUD operations and performance");

    println!("\n🔧 To run individual test categories:");
    println!("  cargo test --test integration_tests");
    println!("  cargo test --test auth_tests");
    println!("  cargo test --test rate_limit_tests");
    println!("  cargo test --test api_endpoint_tests");
    println!("  cargo test --test database_tests");
}

/// Quick smoke test for critical functionality
#[tokio::test]
async fn smoke_test() {
    println!("💨 Running smoke test for critical functionality");

    // This is a minimal test to verify the system is basically working
    let output = Command::new("cargo")
        .args(&["test", "--test", "integration_tests", "test_health_check", "--", "--nocapture"])
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                println!("✅ Smoke test passed - basic functionality working");
            } else {
                println!("❌ Smoke test failed - check system setup");
                let stderr = String::from_utf8_lossy(&result.stderr);
                println!("Error: {}", stderr);
            }
        }
        Err(e) => {
            println!("❌ Failed to run smoke test: {}", e);
        }
    }
}

/// Performance benchmark test
#[tokio::test]
async fn performance_benchmark() {
    println!("⚡ Running performance benchmark");

    let start = std::time::Instant::now();

    let output = Command::new("cargo")
        .args(&["test", "--test", "database_tests", "test_database_performance", "--", "--nocapture"])
        .output();

    let duration = start.elapsed();

    match output {
        Ok(result) => {
            if result.status.success() {
                println!("✅ Performance test completed in {:?}", duration);
            } else {
                println!("❌ Performance test failed");
            }
        }
        Err(e) => {
            println!("❌ Failed to run performance test: {}", e);
        }
    }
}

/// Test environment validation
#[tokio::test]
async fn validate_test_environment() {
    println!("🔍 Validating test environment");

    // Check if required environment variables are set
    let env_vars = vec![
        "DATABASE_URL",
        "ALPHA_VANTAGE_API_KEY",
    ];

    for var in env_vars {
        match std::env::var(var) {
            Ok(value) => {
                if value.is_empty() {
                    println!("⚠️  {} is set but empty", var);
                } else {
                    println!("✅ {} is configured", var);
                }
            }
            Err(_) => {
                println!("ℹ️  {} not set (using defaults)", var);
            }
        }
    }

    // Check if cargo is available
    let cargo_check = Command::new("cargo")
        .args(&["--version"])
        .output();

    match cargo_check {
        Ok(result) => {
            if result.status.success() {
                let version = String::from_utf8_lossy(&result.stdout);
                println!("✅ Cargo available: {}", version.trim());
            }
        }
        Err(e) => {
            println!("❌ Cargo not available: {}", e);
        }
    }

    println!("🎯 Environment validation complete");
}
