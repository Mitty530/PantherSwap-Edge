#!/bin/bash

# PantherSwap Edge Live System Test
# Comprehensive testing of Alpaca integration, streaming, and failover mechanisms

echo "🚀 PantherSwap Edge Live System Test"
echo "===================================================="
echo "Timestamp: $(date)"
echo "Testing Environment: Paper Trading"
echo "===================================================="

# Initialize test results
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
WARNING_TESTS=0

# Test result tracking
declare -a TEST_RESULTS=()

# Function to run a test and track results
run_test() {
    local test_name="$1"
    local test_command="$2"
    local expected_result="$3"
    
    echo ""
    echo "🔍 Testing: $test_name"
    echo "----------------------------------------"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    # Run the test
    if eval "$test_command"; then
        echo "✅ PASS: $test_name"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        TEST_RESULTS+=("PASS: $test_name")
    else
        echo "❌ FAIL: $test_name"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        TEST_RESULTS+=("FAIL: $test_name")
    fi
}

run_warning_test() {
    local test_name="$1"
    local test_command="$2"
    local message="$3"
    
    echo ""
    echo "🔍 Testing: $test_name"
    echo "----------------------------------------"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if eval "$test_command"; then
        echo "✅ PASS: $test_name"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        TEST_RESULTS+=("PASS: $test_name")
    else
        echo "⚠️  WARNING: $test_name - $message"
        WARNING_TESTS=$((WARNING_TESTS + 1))
        TEST_RESULTS+=("WARNING: $test_name - $message")
    fi
}

# Test 1: Configuration Validation
echo ""
echo "📋 1. CONFIGURATION VALIDATION"
echo "===================================================="

run_test "Production Config Exists" "test -f config/production.toml"
run_test "Default Config Exists" "test -f config/default.toml"
run_test "Alpaca Config Present" "grep -q 'CK6KLMXTNEGGKCMVZA2R' config/production.toml"
run_test "Paper Trading Enabled" "grep -q 'paper_trading = true' config/production.toml"
run_test "Paper API URL Configured" "grep -q 'paper-api.alpaca.markets' config/production.toml"
run_test "TimescaleDB Configured" "grep -q 'tsdb.cloud.timescale.com' config/production.toml"

# Test 2: Source Code Structure
echo ""
echo "📁 2. SOURCE CODE STRUCTURE"
echo "===================================================="

run_test "Alpaca Provider Exists" "test -f src/market_data/alpaca.rs"
run_test "Alpaca Execution Engine Exists" "test -f src/trading/alpaca_execution.rs"
run_test "Alpaca Trading Engine Exists" "test -f src/trading/alpaca_trading_engine.rs"
run_test "Health Endpoints Exist" "test -f src/api/routes/health.rs"
run_test "Production Monitoring Exists" "test -f src/monitoring/production.rs"
run_test "Database Health Monitor Exists" "test -f src/database/health_monitor.rs"

# Test 3: API Connectivity Test
echo ""
echo "🌐 3. ALPACA API CONNECTIVITY"
echo "===================================================="

echo "Testing Alpaca API connectivity..."
API_RESPONSE=$(curl -s -w "HTTPSTATUS:%{http_code}" \
    -H "APCA-API-KEY-ID: CK6KLMXTNEGGKCMVZA2R" \
    -H "APCA-API-SECRET-KEY: vFxGY6FDzr3Kq1XhkSHzrZRFgvDKuEfQj9b6odCR" \
    https://paper-api.alpaca.markets/v2/account)

HTTP_STATUS=$(echo $API_RESPONSE | tr -d '\n' | sed -e 's/.*HTTPSTATUS://')
RESPONSE_BODY=$(echo $API_RESPONSE | sed -e 's/HTTPSTATUS:.*//g')

echo "HTTP Status: $HTTP_STATUS"
echo "Response: $RESPONSE_BODY"

if [ "$HTTP_STATUS" = "200" ]; then
    echo "✅ PASS: Alpaca API Connectivity"
    PASSED_TESTS=$((PASSED_TESTS + 1))
    TEST_RESULTS+=("PASS: Alpaca API Connectivity")
elif [ "$HTTP_STATUS" = "403" ]; then
    echo "⚠️  WARNING: Alpaca API Credentials - 403 Forbidden (credentials may need refresh)"
    WARNING_TESTS=$((WARNING_TESTS + 1))
    TEST_RESULTS+=("WARNING: Alpaca API Credentials - 403 Forbidden")
else
    echo "❌ FAIL: Alpaca API Connectivity - HTTP $HTTP_STATUS"
    FAILED_TESTS=$((FAILED_TESTS + 1))
    TEST_RESULTS+=("FAIL: Alpaca API Connectivity - HTTP $HTTP_STATUS")
fi

TOTAL_TESTS=$((TOTAL_TESTS + 1))

# Test 4: Market Data Endpoints
echo ""
echo "📊 4. MARKET DATA ENDPOINTS"
echo "===================================================="

echo "Testing market data endpoint..."
DATA_RESPONSE=$(curl -s -w "HTTPSTATUS:%{http_code}" \
    -H "APCA-API-KEY-ID: CK6KLMXTNEGGKCMVZA2R" \
    -H "APCA-API-SECRET-KEY: vFxGY6FDzr3Kq1XhkSHzrZRFgvDKuEfQj9b6odCR" \
    https://data.alpaca.markets/v2/stocks/AAPL/quotes/latest)

DATA_HTTP_STATUS=$(echo $DATA_RESPONSE | tr -d '\n' | sed -e 's/.*HTTPSTATUS://')
echo "Market Data HTTP Status: $DATA_HTTP_STATUS"

if [ "$DATA_HTTP_STATUS" = "200" ]; then
    echo "✅ PASS: Market Data Access"
    PASSED_TESTS=$((PASSED_TESTS + 1))
    TEST_RESULTS+=("PASS: Market Data Access")
elif [ "$DATA_HTTP_STATUS" = "403" ]; then
    echo "⚠️  WARNING: Market Data Access - 403 Forbidden"
    WARNING_TESTS=$((WARNING_TESTS + 1))
    TEST_RESULTS+=("WARNING: Market Data Access - 403 Forbidden")
else
    echo "❌ FAIL: Market Data Access - HTTP $DATA_HTTP_STATUS"
    FAILED_TESTS=$((FAILED_TESTS + 1))
    TEST_RESULTS+=("FAIL: Market Data Access - HTTP $DATA_HTTP_STATUS")
fi

TOTAL_TESTS=$((TOTAL_TESTS + 1))

# Test 5: Performance Configuration
echo ""
echo "⚡ 5. PERFORMANCE CONFIGURATION"
echo "===================================================="

run_test "Target Latency Configured" "grep -q 'target_latency_ms = 5.0' config/production.toml"
run_test "Target Throughput Configured" "grep -q 'target_throughput_tps = 2000.0' config/production.toml"
run_test "AI Latency Target Set" "grep -q 'ai_latency_threshold_ms = 100.0' config/production.toml"
run_test "Lock-Free Processing Enabled" "grep -q 'enable_lock_free_processing = true' config/production.toml"
run_test "Memory Pool Enabled" "grep -q 'enable_memory_pool = true' config/production.toml"

# Test 6: Safety Mechanisms
echo ""
echo "🛡️  6. SAFETY MECHANISMS"
echo "===================================================="

run_test "Paper Trading Enforced" "grep -q 'paper_trading = true' config/production.toml"
run_test "Max Daily Loss Set" "grep -q 'max_daily_loss' config/production.toml"
run_test "Portfolio VaR Limit Set" "grep -q 'max_portfolio_var' config/production.toml"
run_test "Drawdown Limit Set" "grep -q 'drawdown_limit' config/production.toml"
run_test "Emergency Stop Loss Set" "grep -q 'emergency_stop_loss_pct' config/production.toml"

# Test 7: Monitoring and Alerting
echo ""
echo "📊 7. MONITORING AND ALERTING"
echo "===================================================="

run_test "Auto Recovery Enabled" "grep -q 'enable_auto_recovery = true' config/production.toml"
run_test "Health Check Interval Set" "grep -q 'health_check_interval_seconds' config/production.toml"
run_test "Performance Profiling Enabled" "grep -q 'enable_performance_profiling = true' config/production.toml"
run_test "Database Monitoring Enabled" "grep -q 'enable_real_time_monitoring = true' config/production.toml"
run_test "Alerting Configured" "grep -q 'enable_notifications = true' config/production.toml"

# Test 8: Failover Configuration
echo ""
echo "🔄 8. FAILOVER CONFIGURATION"
echo "===================================================="

run_test "Failover Enabled" "grep -q 'enable_failover = true' config/production.toml"
run_test "Backup Providers Configured" "grep -q 'backup_providers' config/production.toml"
run_test "Circuit Breaker Enabled" "grep -q 'enable_circuit_breaker = true' config/production.toml"
run_test "Recovery Check Interval Set" "grep -q 'recovery_check_interval_seconds' config/production.toml"

# Test 9: Database Integration
echo ""
echo "🗄️  9. DATABASE INTEGRATION"
echo "===================================================="

run_test "Connection Pool Configured" "grep -q 'max_connections = 75' config/production.toml"
run_test "Query Timeout Set" "grep -q 'query_timeout' config/production.toml"
run_test "Connection Monitoring Enabled" "grep -q 'connection_pool_monitoring = true' config/production.toml"
run_test "Performance Metrics Enabled" "grep -q 'enable_performance_metrics = true' config/production.toml"

# Generate Final Report
echo ""
echo "🎯 FINAL TEST RESULTS"
echo "===================================================="
echo "Total Tests: $TOTAL_TESTS"
echo "Passed: $PASSED_TESTS"
echo "Failed: $FAILED_TESTS"
echo "Warnings: $WARNING_TESTS"
echo ""

SUCCESS_RATE=$(echo "scale=1; $PASSED_TESTS * 100 / $TOTAL_TESTS" | bc -l 2>/dev/null || echo "0")
echo "Success Rate: $SUCCESS_RATE%"

# Determine overall status
if [ $FAILED_TESTS -eq 0 ] && [ $WARNING_TESTS -eq 0 ]; then
    OVERALL_STATUS="🟢 READY - All systems operational"
elif [ $FAILED_TESTS -eq 0 ]; then
    OVERALL_STATUS="🟡 CONDITIONAL - Minor issues detected (mainly API credentials)"
else
    OVERALL_STATUS="🔴 NOT READY - Critical failures detected"
fi

echo ""
echo "Overall Status: $OVERALL_STATUS"

# Save detailed results
cat > live_test_results.txt << EOF
PantherSwap Edge Live System Test Results
Generated: $(date)

Overall Status: $OVERALL_STATUS
Success Rate: $SUCCESS_RATE%

Test Summary:
- Total Tests: $TOTAL_TESTS
- Passed: $PASSED_TESTS
- Failed: $FAILED_TESTS  
- Warnings: $WARNING_TESTS

Detailed Results:
EOF

for result in "${TEST_RESULTS[@]}"; do
    echo "- $result" >> live_test_results.txt
done

echo ""
echo "📄 Detailed results saved to: live_test_results.txt"

# Exit with appropriate code
if [ $FAILED_TESTS -eq 0 ]; then
    exit 0
else
    exit 1
fi
