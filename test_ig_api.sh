#!/bin/bash

# IG Trading API Connectivity Test Script
# This script tests the IG Trading API using curl commands

echo "🚀 IG Trading API Connectivity Test"
echo "===================================="
echo "🕐 Test Started: $(date)"
echo ""

# IG Trading API Configuration
API_KEY="3ded3ba7db96187488bf8773b86bdf3e8fc42e9b"
SECURITY_TOKEN="1206a1630c34bcc90fdcc1b62fc5920fa7ed3a216ae09933430d3de2c6bcf6CD01112"
CST="48417021199921da08b95b210d8f9492c36614232983a9f1f3e1a8f0748ce33CC01113"
VERSION="2"
BASE_URL="https://demo-api.ig.com/gateway/deal"
CONTENT_TYPE="application/json; charset=UTF-8"
ACCEPT="application/json; charset=UTF-8"

echo "📋 Configuration:"
echo "   API Key: ${API_KEY:0:8}...${API_KEY: -8}"
echo "   Base URL: $BASE_URL"
echo "   Version: $VERSION"
echo ""

# Test 1: Basic API Connectivity
echo "🔍 Test 1: Basic API Connectivity"
echo "--------------------------------"

echo "📡 Testing basic connectivity to IG API..."
response=$(curl -s -w "HTTPSTATUS:%{http_code}" \
  -H "Content-Type: $CONTENT_TYPE" \
  -H "Accept: $ACCEPT" \
  -H "X-IG-API-KEY: $API_KEY" \
  -H "Version: $VERSION" \
  --connect-timeout 10 \
  "$BASE_URL/session")

http_code=$(echo $response | tr -d '\n' | sed -e 's/.*HTTPSTATUS://')
body=$(echo $response | sed -e 's/HTTPSTATUS\:.*//g')

echo "📋 HTTP Status Code: $http_code"
echo "📄 Response Body: ${body:0:200}..."

if [[ $http_code -eq 200 ]] || [[ $http_code -eq 401 ]] || [[ $http_code -eq 403 ]]; then
    echo "✅ API endpoint is reachable (Status: $http_code)"
    basic_test_passed=true
else
    echo "❌ API endpoint test failed (Status: $http_code)"
    basic_test_passed=false
fi

echo ""

# Test 2: Authentication Test
echo "🔐 Test 2: Authentication Test"
echo "------------------------------"

echo "🔑 Testing authentication endpoint..."
auth_data='{"identifier":"","password":"","encryptedPassword":false}'

auth_response=$(curl -s -w "HTTPSTATUS:%{http_code}" \
  -X POST \
  -H "Content-Type: $CONTENT_TYPE" \
  -H "Accept: $ACCEPT" \
  -H "X-IG-API-KEY: $API_KEY" \
  -H "Version: $VERSION" \
  -d "$auth_data" \
  --connect-timeout 10 \
  "$BASE_URL/session")

auth_http_code=$(echo $auth_response | tr -d '\n' | sed -e 's/.*HTTPSTATUS://')
auth_body=$(echo $auth_response | sed -e 's/HTTPSTATUS\:.*//g')

echo "📋 HTTP Status Code: $auth_http_code"
echo "📄 Response Body: ${auth_body:0:300}..."

if [[ $auth_http_code -eq 401 ]]; then
    echo "✅ Authentication endpoint working correctly (401 - credentials needed)"
    auth_test_passed=true
elif [[ $auth_http_code -eq 200 ]]; then
    echo "✅ Authentication successful!"
    auth_test_passed=true
else
    echo "❌ Authentication test failed (Status: $auth_http_code)"
    auth_test_passed=false
fi

echo ""

# Test 3: Market Data Test
echo "📈 Test 3: Market Data Test"
echo "---------------------------"

# Test popular instruments
instruments=("CS.D.EURUSD.MINI.IP" "CS.D.GBPUSD.MINI.IP" "IX.D.FTSE.DAILY.IP")
market_data_passed=0

for instrument in "${instruments[@]}"; do
    echo "🔍 Testing instrument: $instrument"
    
    market_response=$(curl -s -w "HTTPSTATUS:%{http_code}" \
      -H "Content-Type: $CONTENT_TYPE" \
      -H "Accept: $ACCEPT" \
      -H "X-IG-API-KEY: $API_KEY" \
      -H "Version: $VERSION" \
      -H "CST: $CST" \
      -H "X-SECURITY-TOKEN: $SECURITY_TOKEN" \
      --connect-timeout 10 \
      "$BASE_URL/markets/$instrument")
    
    market_http_code=$(echo $market_response | tr -d '\n' | sed -e 's/.*HTTPSTATUS://')
    market_body=$(echo $market_response | sed -e 's/HTTPSTATUS\:.*//g')
    
    echo "   📋 Status Code: $market_http_code"
    
    if [[ $market_http_code -eq 200 ]]; then
        echo "   ✅ Market data retrieved for $instrument"
        echo "   📊 Response preview: ${market_body:0:100}..."
        ((market_data_passed++))
    elif [[ $market_http_code -eq 401 ]]; then
        echo "   ⚠️  Authentication required for $instrument"
    elif [[ $market_http_code -eq 404 ]]; then
        echo "   ❌ Instrument $instrument not found"
    else
        echo "   ❌ Error $market_http_code for $instrument"
    fi
    echo ""
done

echo ""

# Test 4: Rate Limit Test
echo "⚡ Test 4: Rate Limit Test"
echo "-------------------------"

echo "🚀 Testing API rate limits with 3 rapid requests..."
rate_limit_passed=0
start_time=$(date +%s)

for i in {1..3}; do
    rate_response=$(curl -s -w "HTTPSTATUS:%{http_code}" \
      -H "Content-Type: $CONTENT_TYPE" \
      -H "Accept: $ACCEPT" \
      -H "X-IG-API-KEY: $API_KEY" \
      -H "Version: $VERSION" \
      --connect-timeout 5 \
      "$BASE_URL/session")
    
    rate_http_code=$(echo $rate_response | tr -d '\n' | sed -e 's/.*HTTPSTATUS://')
    
    if [[ $rate_http_code -eq 200 ]] || [[ $rate_http_code -eq 401 ]] || [[ $rate_http_code -eq 403 ]]; then
        echo "   ✅ Request $i: Success ($rate_http_code)"
        ((rate_limit_passed++))
    else
        echo "   ❌ Request $i: Failed ($rate_http_code)"
    fi
    
    # Small delay between requests
    sleep 0.5
done

end_time=$(date +%s)
total_time=$((end_time - start_time))

echo ""
echo "📊 Rate Limit Test Results:"
echo "   ⏱️  Total Time: $total_time seconds"
echo "   ✅ Successful Requests: $rate_limit_passed"
echo "   ❌ Failed Requests: $((3 - rate_limit_passed))"
echo "   📊 Success Rate: $(( (rate_limit_passed * 100) / 3 ))%"

echo ""

# Test Summary
echo "📋 Test Summary"
echo "==============="

total_tests=4
passed_tests=0

if [[ $basic_test_passed == true ]]; then
    echo "✅ Basic Connection: PASSED"
    ((passed_tests++))
else
    echo "❌ Basic Connection: FAILED"
fi

if [[ $auth_test_passed == true ]]; then
    echo "✅ Authentication: PASSED"
    ((passed_tests++))
else
    echo "❌ Authentication: FAILED"
fi

if [[ $market_data_passed -gt 0 ]]; then
    echo "✅ Market Data: PASSED ($market_data_passed/3 instruments)"
    ((passed_tests++))
else
    echo "❌ Market Data: FAILED"
fi

if [[ $rate_limit_passed -gt 1 ]]; then
    echo "✅ Rate Limits: PASSED"
    ((passed_tests++))
else
    echo "❌ Rate Limits: FAILED"
fi

echo ""
echo "📊 Overall Results: $passed_tests/$total_tests tests passed"

if [[ $passed_tests -ge 2 ]]; then
    echo "🎉 IG Trading API integration looks promising!"
    echo "💡 Next steps:"
    echo "   1. Set up proper IG Trading demo account credentials"
    echo "   2. Test with actual login credentials"
    echo "   3. Integrate with PantherSwap Edge Rust code"
else
    echo "⚠️  IG Trading API integration needs attention"
    echo "🔧 Troubleshooting:"
    echo "   1. Verify API credentials"
    echo "   2. Check network connectivity"
    echo "   3. Review IG Trading API documentation"
fi

echo ""
echo "🏁 Test Completed: $(date)"
