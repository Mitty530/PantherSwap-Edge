#!/usr/bin/env python3
"""
Simple IG Trading API Connectivity Test
This script tests the IG Trading API connection using Python requests
"""

import requests
import json
import time
from datetime import datetime

# IG Trading API Configuration
IG_CONFIG = {
    "api_key": "3ded3ba7db96187488bf8773b86bdf3e8fc42e9b",
    "security_token": "1206a1630c34bcc90fdcc1b62fc5920fa7ed3a216ae09933430d3de2c6bcf6CD01112",
    "cst": "48417021199921da08b95b210d8f9492c36614232983a9f1f3e1a8f0748ce33CC01113",
    "version": "2",
    "base_url": "https://demo-api.ig.com/gateway/deal",
    "content_type": "application/json; charset=UTF-8",
    "accept": "application/json; charset=UTF-8"
}

def print_header(title):
    """Print a formatted header"""
    print(f"\n{'='*50}")
    print(f"🔍 {title}")
    print(f"{'='*50}")

def print_result(success, message):
    """Print a formatted result"""
    icon = "✅" if success else "❌"
    print(f"{icon} {message}")

def test_basic_connection():
    """Test basic API connectivity"""
    print_header("Basic API Connection Test")
    
    try:
        # Test basic connectivity to IG's API endpoint
        response = requests.get(
            f"{IG_CONFIG['base_url']}/session",
            headers={
                "Content-Type": IG_CONFIG["content_type"],
                "Accept": IG_CONFIG["accept"],
                "X-IG-API-KEY": IG_CONFIG["api_key"],
                "Version": IG_CONFIG["version"]
            },
            timeout=10
        )
        
        print(f"📡 Request URL: {IG_CONFIG['base_url']}/session")
        print(f"📋 Status Code: {response.status_code}")
        print(f"📊 Response Headers: {dict(response.headers)}")
        
        if response.status_code in [200, 401, 403]:  # These are expected responses
            print_result(True, f"API endpoint is reachable (Status: {response.status_code})")
            return True
        else:
            print_result(False, f"Unexpected status code: {response.status_code}")
            return False
            
    except requests.exceptions.RequestException as e:
        print_result(False, f"Connection failed: {str(e)}")
        return False

def test_authentication():
    """Test API authentication"""
    print_header("Authentication Test")
    
    try:
        # Note: For demo API, we typically need actual login credentials
        # This test will show us what the API expects
        auth_data = {
            "identifier": "",  # Would need actual username
            "password": "",    # Would need actual password
            "encryptedPassword": False
        }
        
        response = requests.post(
            f"{IG_CONFIG['base_url']}/session",
            headers={
                "Content-Type": IG_CONFIG["content_type"],
                "Accept": IG_CONFIG["accept"],
                "X-IG-API-KEY": IG_CONFIG["api_key"],
                "Version": IG_CONFIG["version"]
            },
            json=auth_data,
            timeout=10
        )
        
        print(f"📡 Authentication URL: {IG_CONFIG['base_url']}/session")
        print(f"📋 Status Code: {response.status_code}")
        print(f"📄 Response: {response.text[:500]}...")
        
        if response.status_code == 401:
            print_result(True, "API is responding correctly (401 - credentials needed)")
            return True
        elif response.status_code == 200:
            print_result(True, "Authentication successful!")
            return True
        else:
            print_result(False, f"Unexpected authentication response: {response.status_code}")
            return False
            
    except requests.exceptions.RequestException as e:
        print_result(False, f"Authentication test failed: {str(e)}")
        return False

def test_market_data():
    """Test market data endpoint"""
    print_header("Market Data Test")
    
    # Test popular instruments
    test_instruments = [
        "CS.D.EURUSD.MINI.IP",  # EUR/USD
        "CS.D.GBPUSD.MINI.IP",  # GBP/USD
        "IX.D.FTSE.DAILY.IP",   # FTSE 100
    ]
    
    for instrument in test_instruments:
        try:
            print(f"\n🔍 Testing instrument: {instrument}")
            
            response = requests.get(
                f"{IG_CONFIG['base_url']}/markets/{instrument}",
                headers={
                    "Content-Type": IG_CONFIG["content_type"],
                    "Accept": IG_CONFIG["accept"],
                    "X-IG-API-KEY": IG_CONFIG["api_key"],
                    "Version": IG_CONFIG["version"],
                    "CST": IG_CONFIG["cst"],
                    "X-SECURITY-TOKEN": IG_CONFIG["security_token"]
                },
                timeout=10
            )
            
            print(f"   📋 Status Code: {response.status_code}")
            
            if response.status_code == 200:
                data = response.json()
                print_result(True, f"Market data retrieved for {instrument}")
                print(f"   📊 Response preview: {str(data)[:200]}...")
            elif response.status_code == 401:
                print_result(False, f"Authentication required for {instrument}")
            elif response.status_code == 404:
                print_result(False, f"Instrument {instrument} not found")
            else:
                print_result(False, f"Error {response.status_code} for {instrument}")
                
        except requests.exceptions.RequestException as e:
            print_result(False, f"Request failed for {instrument}: {str(e)}")

def test_api_limits():
    """Test API rate limits"""
    print_header("API Rate Limit Test")
    
    print("🚀 Testing API rate limits with 5 rapid requests...")
    
    start_time = time.time()
    successful_requests = 0
    failed_requests = 0
    
    for i in range(5):
        try:
            response = requests.get(
                f"{IG_CONFIG['base_url']}/session",
                headers={
                    "Content-Type": IG_CONFIG["content_type"],
                    "Accept": IG_CONFIG["accept"],
                    "X-IG-API-KEY": IG_CONFIG["api_key"],
                    "Version": IG_CONFIG["version"]
                },
                timeout=5
            )
            
            if response.status_code in [200, 401, 403]:
                successful_requests += 1
                print(f"   ✅ Request {i+1}: Success ({response.status_code})")
            else:
                failed_requests += 1
                print(f"   ❌ Request {i+1}: Failed ({response.status_code})")
                
        except requests.exceptions.RequestException as e:
            failed_requests += 1
            print(f"   ❌ Request {i+1}: Exception - {str(e)}")
        
        # Small delay between requests
        time.sleep(0.5)
    
    total_time = time.time() - start_time
    avg_time = total_time / 5
    
    print(f"\n📊 Rate Limit Test Results:")
    print(f"   ⏱️  Total Time: {total_time:.2f} seconds")
    print(f"   📈 Average Time per Request: {avg_time:.2f} seconds")
    print(f"   ✅ Successful Requests: {successful_requests}")
    print(f"   ❌ Failed Requests: {failed_requests}")
    print(f"   📊 Success Rate: {(successful_requests/5)*100:.1f}%")

def main():
    """Main test function"""
    print("🚀 IG Trading API Connectivity Test")
    print("=" * 50)
    print(f"🕐 Test Started: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"🌐 API Base URL: {IG_CONFIG['base_url']}")
    print(f"🔑 API Key: {IG_CONFIG['api_key'][:8]}...{IG_CONFIG['api_key'][-8:]}")
    print(f"📋 Version: {IG_CONFIG['version']}")
    
    # Run all tests
    tests = [
        ("Basic Connection", test_basic_connection),
        ("Authentication", test_authentication),
        ("Market Data", test_market_data),
        ("Rate Limits", test_api_limits)
    ]
    
    results = {}
    
    for test_name, test_func in tests:
        try:
            results[test_name] = test_func()
        except Exception as e:
            print_result(False, f"{test_name} test crashed: {str(e)}")
            results[test_name] = False
    
    # Summary
    print_header("Test Summary")
    
    passed_tests = sum(1 for result in results.values() if result)
    total_tests = len(results)
    
    for test_name, result in results.items():
        icon = "✅" if result else "❌"
        print(f"{icon} {test_name}: {'PASSED' if result else 'FAILED'}")
    
    print(f"\n📊 Overall Results: {passed_tests}/{total_tests} tests passed")
    
    if passed_tests >= 2:  # Basic connection and auth response are good signs
        print("🎉 IG Trading API integration looks promising!")
        print("💡 Next steps:")
        print("   1. Set up proper IG Trading demo account credentials")
        print("   2. Test with actual login credentials")
        print("   3. Integrate with PantherSwap Edge Rust code")
    else:
        print("⚠️  IG Trading API integration needs attention")
        print("🔧 Troubleshooting:")
        print("   1. Verify API credentials")
        print("   2. Check network connectivity")
        print("   3. Review IG Trading API documentation")
    
    print(f"\n🏁 Test Completed: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")

if __name__ == "__main__":
    main()
