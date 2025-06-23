#!/bin/bash

# Alpaca API Diagnostic Test
# Comprehensive testing to identify API credential issues

echo "🔍 Alpaca API Diagnostic Test"
echo "=============================================="
echo "Testing both sets of credentials and endpoints"
echo ""

# Credentials to test
API_KEY_1="CK6KLMXTNEGGKCMVZA2R"
SECRET_KEY_1="vFxGY6FDzr3Kq1XhkSHzrZRFgvDKuEfQj9b6odCR"

API_KEY_2="CKG0KGXSOGQ9JG8MJTVY"
SECRET_KEY_2="cuqp56NLVarz0Lgo5bCn1vrxVI9i7bFazb4Dn7bl"

# Endpoints to test
PAPER_API="https://paper-api.alpaca.markets"
LIVE_API="https://api.alpaca.markets"
DATA_API="https://data.alpaca.markets"

echo "📋 Testing Credential Set 1 (Original)"
echo "API Key: $API_KEY_1"
echo "Secret: ${SECRET_KEY_1:0:10}..."
echo ""

echo "🔍 Paper Trading Account Endpoint:"
RESPONSE_1=$(curl -s -w "HTTPSTATUS:%{http_code}" \
    -H "APCA-API-KEY-ID: $API_KEY_1" \
    -H "APCA-API-SECRET-KEY: $SECRET_KEY_1" \
    "$PAPER_API/v2/account")

HTTP_STATUS_1=$(echo $RESPONSE_1 | tr -d '\n' | sed -e 's/.*HTTPSTATUS://')
RESPONSE_BODY_1=$(echo $RESPONSE_1 | sed -e 's/HTTPSTATUS:.*//g')

echo "   Status: $HTTP_STATUS_1"
echo "   Response: $RESPONSE_BODY_1"

echo ""
echo "🔍 Live Trading Account Endpoint:"
RESPONSE_1_LIVE=$(curl -s -w "HTTPSTATUS:%{http_code}" \
    -H "APCA-API-KEY-ID: $API_KEY_1" \
    -H "APCA-API-SECRET-KEY: $SECRET_KEY_1" \
    "$LIVE_API/v2/account")

HTTP_STATUS_1_LIVE=$(echo $RESPONSE_1_LIVE | tr -d '\n' | sed -e 's/.*HTTPSTATUS://')
RESPONSE_BODY_1_LIVE=$(echo $RESPONSE_1_LIVE | sed -e 's/HTTPSTATUS:.*//g')

echo "   Status: $HTTP_STATUS_1_LIVE"
echo "   Response: $RESPONSE_BODY_1_LIVE"

echo ""
echo "=" .repeat(50)
echo ""

echo "📋 Testing Credential Set 2 (New)"
echo "API Key: $API_KEY_2"
echo "Secret: ${SECRET_KEY_2:0:10}..."
echo ""

echo "🔍 Paper Trading Account Endpoint:"
RESPONSE_2=$(curl -s -w "HTTPSTATUS:%{http_code}" \
    -H "APCA-API-KEY-ID: $API_KEY_2" \
    -H "APCA-API-SECRET-KEY: $SECRET_KEY_2" \
    "$PAPER_API/v2/account")

HTTP_STATUS_2=$(echo $RESPONSE_2 | tr -d '\n' | sed -e 's/.*HTTPSTATUS://')
RESPONSE_BODY_2=$(echo $RESPONSE_2 | sed -e 's/HTTPSTATUS:.*//g')

echo "   Status: $HTTP_STATUS_2"
echo "   Response: $RESPONSE_BODY_2"

echo ""
echo "🔍 Live Trading Account Endpoint:"
RESPONSE_2_LIVE=$(curl -s -w "HTTPSTATUS:%{http_code}" \
    -H "APCA-API-KEY-ID: $API_KEY_2" \
    -H "APCA-API-SECRET-KEY: $SECRET_KEY_2" \
    "$LIVE_API/v2/account")

HTTP_STATUS_2_LIVE=$(echo $RESPONSE_2_LIVE | tr -d '\n' | sed -e 's/.*HTTPSTATUS://')
RESPONSE_BODY_2_LIVE=$(echo $RESPONSE_2_LIVE | sed -e 's/HTTPSTATUS:.*//g')

echo "   Status: $HTTP_STATUS_2_LIVE"
echo "   Response: $RESPONSE_BODY_2_LIVE"

echo ""
echo "🔍 Market Data Endpoint (Set 2):"
RESPONSE_2_DATA=$(curl -s -w "HTTPSTATUS:%{http_code}" \
    -H "APCA-API-KEY-ID: $API_KEY_2" \
    -H "APCA-API-SECRET-KEY: $SECRET_KEY_2" \
    "$DATA_API/v2/stocks/AAPL/quotes/latest")

HTTP_STATUS_2_DATA=$(echo $RESPONSE_2_DATA | tr -d '\n' | sed -e 's/.*HTTPSTATUS://')
RESPONSE_BODY_2_DATA=$(echo $RESPONSE_2_DATA | sed -e 's/HTTPSTATUS:.*//g')

echo "   Status: $HTTP_STATUS_2_DATA"
echo "   Response: ${RESPONSE_BODY_2_DATA:0:200}..."

echo ""
echo "🎯 DIAGNOSTIC RESULTS"
echo "=============================================="

# Analyze results
if [ "$HTTP_STATUS_1" = "200" ] || [ "$HTTP_STATUS_1_LIVE" = "200" ]; then
    echo "✅ Credential Set 1: WORKING"
    WORKING_CREDS="Set 1"
elif [ "$HTTP_STATUS_2" = "200" ] || [ "$HTTP_STATUS_2_LIVE" = "200" ]; then
    echo "✅ Credential Set 2: WORKING"
    WORKING_CREDS="Set 2"
else
    echo "❌ Both credential sets: FAILED"
    WORKING_CREDS="None"
fi

echo ""
echo "📊 Status Summary:"
echo "   Set 1 Paper: $HTTP_STATUS_1"
echo "   Set 1 Live:  $HTTP_STATUS_1_LIVE"
echo "   Set 2 Paper: $HTTP_STATUS_2"
echo "   Set 2 Live:  $HTTP_STATUS_2_LIVE"
echo "   Set 2 Data:  $HTTP_STATUS_2_DATA"

echo ""
echo "🔍 Possible Issues:"

if [ "$HTTP_STATUS_1" = "403" ] && [ "$HTTP_STATUS_2" = "403" ]; then
    echo "   ❌ 403 Forbidden - Possible causes:"
    echo "      • API keys are invalid or expired"
    echo "      • Account is suspended or restricted"
    echo "      • Paper trading not enabled on account"
    echo "      • Account needs to be funded (even for paper trading)"
    echo "      • API access not enabled in account settings"
    echo "      • Wrong environment (live keys used for paper endpoint)"
fi

if [ "$HTTP_STATUS_1" = "401" ] || [ "$HTTP_STATUS_2" = "401" ]; then
    echo "   ❌ 401 Unauthorized - Authentication failed"
    echo "      • Check API key format"
    echo "      • Verify secret key is correct"
fi

echo ""
echo "💡 Recommendations:"
echo "   1. Log into Alpaca dashboard and verify:"
echo "      • Account status is active"
echo "      • Paper trading is enabled"
echo "      • API access is enabled"
echo "      • Generate fresh API keys"
echo ""
echo "   2. Check account requirements:"
echo "      • Some accounts need minimum funding"
echo "      • Verify account type supports API access"
echo "      • Ensure account is not restricted"
echo ""
echo "   3. Test with Alpaca's official tools:"
echo "      • Use Alpaca's API documentation examples"
echo "      • Test with their official SDKs"
echo "      • Contact Alpaca support if issues persist"

echo ""
echo "🚀 Next Steps for PantherSwap Edge:"
if [ "$WORKING_CREDS" != "None" ]; then
    echo "   ✅ Use working credentials for live testing"
    echo "   ✅ Update production configuration"
    echo "   ✅ Proceed with streaming and failover tests"
else
    echo "   ❌ Resolve API credential issues first"
    echo "   ❌ Cannot proceed with live testing until resolved"
    echo "   ✅ System architecture is ready once credentials work"
fi

echo ""
echo "📄 Diagnostic complete - $(date)"
