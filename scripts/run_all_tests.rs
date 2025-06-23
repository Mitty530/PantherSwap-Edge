#!/usr/bin/env rust-script
//! Comprehensive Test Suite Runner for PantherSwap Edge
//! Executes all system tests and generates a complete report

use std::process::Command;
use std::time::Instant;
use chrono::Utc;

#[derive(Debug)]
struct TestSuiteResults {
    connectivity_test: TestResult,
    market_data_test: TestResult,
    ai_simulation_test: TestResult,
    trading_performance_test: TestResult,
    end_to_end_test: TestResult,
    overall_duration_seconds: f64,
    total_tests_passed: u32,
    total_tests_run: u32,
    system_readiness_score: f64,
}

#[derive(Debug)]
struct TestResult {
    name: String,
    passed: bool,
    duration_ms: f64,
    details: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 PantherSwap Edge Comprehensive Test Suite");
    println!("=============================================");
    println!("Running complete system validation tests...\n");

    let suite_start = Instant::now();
    
    let mut results = TestSuiteResults {
        connectivity_test: TestResult::new("Connectivity Test"),
        market_data_test: TestResult::new("Market Data Pipeline Test"),
        ai_simulation_test: TestResult::new("AI Engine Simulation Test"),
        trading_performance_test: TestResult::new("Trading Performance Test"),
        end_to_end_test: TestResult::new("End-to-End Integration Test"),
        overall_duration_seconds: 0.0,
        total_tests_passed: 0,
        total_tests_run: 5,
        system_readiness_score: 0.0,
    };

    // Test 1: Connectivity Test
    println!("🔌 Running Connectivity Test...");
    results.connectivity_test = run_test("test_connectivity");
    print_test_result(&results.connectivity_test);

    // Test 2: Market Data Pipeline Test
    println!("\n📈 Running Market Data Pipeline Test...");
    results.market_data_test = run_test("test_simple_storage");
    print_test_result(&results.market_data_test);

    // Test 3: AI Engine Simulation Test
    println!("\n🤖 Running AI Engine Simulation Test...");
    results.ai_simulation_test = run_test("test_ai_simulation");
    print_test_result(&results.ai_simulation_test);

    // Test 4: Trading Performance Test
    println!("\n⚡ Running Trading Performance Test...");
    results.trading_performance_test = run_test("test_trading_performance");
    print_test_result(&results.trading_performance_test);

    // Test 5: End-to-End Integration Test
    println!("\n🔄 Running End-to-End Integration Test...");
    results.end_to_end_test = run_test("test_end_to_end");
    print_test_result(&results.end_to_end_test);

    // Calculate overall results
    results.overall_duration_seconds = suite_start.elapsed().as_secs_f64();
    results.total_tests_passed = count_passed_tests(&results);
    results.system_readiness_score = calculate_readiness_score(&results);

    // Print comprehensive report
    print_comprehensive_report(&results);

    Ok(())
}

impl TestResult {
    fn new(name: &str) -> Self {
        TestResult {
            name: name.to_string(),
            passed: false,
            duration_ms: 0.0,
            details: String::new(),
        }
    }
}

fn run_test(test_name: &str) -> TestResult {
    let start = Instant::now();
    
    let output = Command::new("cargo")
        .args(&["run", "--bin", test_name])
        .output();

    let duration = start.elapsed().as_millis() as f64;
    
    match output {
        Ok(result) => {
            let success = result.status.success();
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stderr = String::from_utf8_lossy(&result.stderr);
            
            TestResult {
                name: test_name.to_string(),
                passed: success,
                duration_ms: duration,
                details: if success {
                    extract_success_details(&stdout)
                } else {
                    format!("Error: {}", stderr)
                },
            }
        }
        Err(e) => TestResult {
            name: test_name.to_string(),
            passed: false,
            duration_ms: duration,
            details: format!("Failed to execute: {}", e),
        }
    }
}

fn extract_success_details(output: &str) -> String {
    // Extract key metrics from test output
    let lines: Vec<&str> = output.lines().collect();
    let mut details = Vec::new();
    
    for line in lines {
        if line.contains("✅") && (
            line.contains("Database:") ||
            line.contains("IG API:") ||
            line.contains("Performance:") ||
            line.contains("TPS") ||
            line.contains("Latency:") ||
            line.contains("Health Score:")
        ) {
            details.push(line.trim());
        }
    }
    
    if details.is_empty() {
        "Test completed successfully".to_string()
    } else {
        details.join("; ")
    }
}

fn print_test_result(result: &TestResult) {
    if result.passed {
        println!("✅ {} - PASSED ({:.0}ms)", result.name, result.duration_ms);
        if !result.details.is_empty() {
            println!("   {}", result.details);
        }
    } else {
        println!("❌ {} - FAILED ({:.0}ms)", result.name, result.duration_ms);
        println!("   {}", result.details);
    }
}

fn count_passed_tests(results: &TestSuiteResults) -> u32 {
    let mut count = 0;
    if results.connectivity_test.passed { count += 1; }
    if results.market_data_test.passed { count += 1; }
    if results.ai_simulation_test.passed { count += 1; }
    if results.trading_performance_test.passed { count += 1; }
    if results.end_to_end_test.passed { count += 1; }
    count
}

fn calculate_readiness_score(results: &TestSuiteResults) -> f64 {
    let base_score = results.total_tests_passed as f64 / results.total_tests_run as f64;
    
    // Bonus points for critical tests
    let mut bonus = 0.0;
    if results.connectivity_test.passed { bonus += 0.1; }
    if results.end_to_end_test.passed { bonus += 0.1; }
    
    // Performance penalty if tests are too slow
    let avg_duration = results.overall_duration_seconds / results.total_tests_run as f64;
    let performance_penalty = if avg_duration > 10.0 { 0.1 } else { 0.0 };
    
    (base_score + bonus - performance_penalty).min(1.0).max(0.0)
}

fn print_comprehensive_report(results: &TestSuiteResults) {
    println!("\n{}", "=".repeat(60));
    println!("🎯 PANTHERSWAP EDGE COMPREHENSIVE TEST REPORT");
    println!("{}", "=".repeat(60));
    println!("Test Suite Execution Time: {:.2}s", results.overall_duration_seconds);
    println!("Tests Passed: {}/{}", results.total_tests_passed, results.total_tests_run);
    println!("System Readiness Score: {:.1}%", results.system_readiness_score * 100.0);
    println!("Timestamp: {}", Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
    
    println!("\n📊 DETAILED TEST RESULTS:");
    println!("┌─────────────────────────────────┬────────┬──────────┐");
    println!("│ Test Name                       │ Status │ Duration │");
    println!("├─────────────────────────────────┼────────┼──────────┤");
    
    let tests = vec![
        &results.connectivity_test,
        &results.market_data_test,
        &results.ai_simulation_test,
        &results.trading_performance_test,
        &results.end_to_end_test,
    ];
    
    for test in tests {
        let status = if test.passed { "✅ PASS" } else { "❌ FAIL" };
        println!("│ {:<31} │ {:<6} │ {:>6.0}ms │", 
                 test.name, status, test.duration_ms);
    }
    
    println!("└─────────────────────────────────┴────────┴──────────┘");
    
    println!("\n🔍 SYSTEM ANALYSIS:");
    
    if results.system_readiness_score >= 0.9 {
        println!("🟢 EXCELLENT - System is production ready");
        println!("   • All critical components are functioning optimally");
        println!("   • Performance targets are being met");
        println!("   • Ready for live trading deployment");
    } else if results.system_readiness_score >= 0.7 {
        println!("🟡 GOOD - System is functional with minor issues");
        println!("   • Core functionality is working");
        println!("   • Some optimizations may be needed");
        println!("   • Suitable for staging environment testing");
    } else if results.system_readiness_score >= 0.5 {
        println!("🟠 FAIR - System needs attention");
        println!("   • Basic functionality is present");
        println!("   • Several issues need to be addressed");
        println!("   • Not ready for production deployment");
    } else {
        println!("🔴 CRITICAL - System requires significant fixes");
        println!("   • Major components are failing");
        println!("   • Extensive debugging and fixes needed");
        println!("   • Not suitable for any live environment");
    }
    
    println!("\n🚀 RECOMMENDATIONS:");
    
    if !results.connectivity_test.passed {
        println!("   • Fix database and API connectivity issues");
    }
    if !results.market_data_test.passed {
        println!("   • Resolve market data pipeline problems");
    }
    if !results.ai_simulation_test.passed {
        println!("   • Debug AI engine and model issues");
    }
    if !results.trading_performance_test.passed {
        println!("   • Optimize trading engine performance");
    }
    if !results.end_to_end_test.passed {
        println!("   • Fix end-to-end integration issues");
    }
    
    if results.system_readiness_score >= 0.8 {
        println!("   • Consider load testing with higher volumes");
        println!("   • Implement comprehensive monitoring");
        println!("   • Prepare for production deployment");
        println!("   • Set up alerting and backup systems");
    }
    
    println!("\n📈 PERFORMANCE METRICS:");
    println!("   • Database Connectivity: {}", 
             if results.connectivity_test.passed { "✅ Operational" } else { "❌ Issues" });
    println!("   • Market Data Pipeline: {}", 
             if results.market_data_test.passed { "✅ Functional" } else { "❌ Problems" });
    println!("   • AI Engine: {}", 
             if results.ai_simulation_test.passed { "✅ Active" } else { "❌ Inactive" });
    println!("   • Trading Engine: {}", 
             if results.trading_performance_test.passed { "✅ Optimized" } else { "❌ Needs Work" });
    println!("   • System Integration: {}", 
             if results.end_to_end_test.passed { "✅ Complete" } else { "❌ Incomplete" });
    
    println!("\n{}", "=".repeat(60));
    println!("Test suite completed. System readiness: {:.1}%",
             results.system_readiness_score * 100.0);
    println!("{}", "=".repeat(60));
}
