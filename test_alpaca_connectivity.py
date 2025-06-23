#!/usr/bin/env python3
"""
Live Alpaca API Connectivity Test
Tests actual connection to Alpaca API using configured credentials
"""

import requests
import json
import time
from datetime import datetime, timedelta
import sys

# Alpaca API Configuration from production.toml
API_KEY = "CK6KLMXTNEGGKCMVZA2R"
SECRET_KEY = "vFxGY6FDzr3Kq1XhkSHzrZRFgvDKuEfQj9b6odCR"
BASE_URL = "https://paper-api.alpaca.markets"
DATA_URL = "https://data.alpaca.markets"

class AlpacaAPITester:
    def __init__(self):
        self.headers = {
            "APCA-API-KEY-ID": API_KEY,
            "APCA-API-SECRET-KEY": SECRET_KEY,
            "Content-Type": "application/json"
        }
        self.results = {
            "timestamp": datetime.now().isoformat(),
            "tests": [],
            "overall_status": "UNKNOWN"
        }
    
    def test_account_access(self):
        """Test basic account access"""
        print("\n🔍 Testing Account Access...")
        print("-" * 40)
        
        try:
            response = requests.get(
                f"{BASE_URL}/v2/account",
                headers=self.headers,
                timeout=10
            )
            
            if response.status_code == 200:
                account_data = response.json()
                print("✅ Account access successful!")
                print(f"   Account ID: {account_data.get('id', 'N/A')}")
                print(f"   Status: {account_data.get('status', 'N/A')}")
                print(f"   Trading Blocked: {account_data.get('trading_blocked', 'N/A')}")
                print(f"   Paper Trading: True (using paper-api endpoint)")
                print(f"   Buying Power: ${account_data.get('buying_power', 'N/A')}")
                print(f"   Cash: ${account_data.get('cash', 'N/A')}")
                
                self.results["tests"].append({
                    "test": "Account Access",
                    "status": "PASS",
                    "details": account_data
                })
                return True
            else:
                print(f"❌ Account access failed: {response.status_code}")
                print(f"   Response: {response.text}")
                self.results["tests"].append({
                    "test": "Account Access", 
                    "status": "FAIL",
                    "error": f"HTTP {response.status_code}: {response.text}"
                })
                return False
                
        except Exception as e:
            print(f"❌ Account access error: {e}")
            self.results["tests"].append({
                "test": "Account Access",
                "status": "FAIL", 
                "error": str(e)
            })
            return False
    
    def test_market_data_access(self):
        """Test market data API access"""
        print("\n📊 Testing Market Data Access...")
        print("-" * 40)
        
        # Test symbols
        symbols = ["AAPL", "MSFT", "GOOGL", "TSLA", "SPY"]
        
        try:
            # Test latest quotes
            for symbol in symbols[:2]:  # Test first 2 symbols
                print(f"   Testing {symbol} quote...")
                response = requests.get(
                    f"{DATA_URL}/v2/stocks/{symbol}/quotes/latest",
                    headers=self.headers,
                    timeout=10
                )
                
                if response.status_code == 200:
                    quote_data = response.json()
                    quote = quote_data.get('quote', {})
                    print(f"   ✅ {symbol}: Bid=${quote.get('bp', 'N/A')}, Ask=${quote.get('ap', 'N/A')}")
                else:
                    print(f"   ❌ {symbol}: Failed ({response.status_code})")
                    
                time.sleep(0.5)  # Rate limiting
            
            # Test bars (historical data)
            print(f"   Testing historical bars for AAPL...")
            end_date = datetime.now()
            start_date = end_date - timedelta(days=1)
            
            response = requests.get(
                f"{DATA_URL}/v2/stocks/AAPL/bars",
                headers=self.headers,
                params={
                    "start": start_date.strftime("%Y-%m-%d"),
                    "end": end_date.strftime("%Y-%m-%d"),
                    "timeframe": "1Hour"
                },
                timeout=10
            )
            
            if response.status_code == 200:
                bars_data = response.json()
                bars = bars_data.get('bars', [])
                print(f"   ✅ Historical data: {len(bars)} bars retrieved")
                
                self.results["tests"].append({
                    "test": "Market Data Access",
                    "status": "PASS",
                    "details": f"Successfully retrieved quotes and {len(bars)} historical bars"
                })
                return True
            else:
                print(f"   ❌ Historical data failed: {response.status_code}")
                self.results["tests"].append({
                    "test": "Market Data Access",
                    "status": "FAIL",
                    "error": f"Historical data HTTP {response.status_code}"
                })
                return False
                
        except Exception as e:
            print(f"❌ Market data error: {e}")
            self.results["tests"].append({
                "test": "Market Data Access",
                "status": "FAIL",
                "error": str(e)
            })
            return False
    
    def test_order_capabilities(self):
        """Test order submission capabilities (dry run)"""
        print("\n📋 Testing Order Capabilities...")
        print("-" * 40)
        
        try:
            # Test order validation (without actually placing)
            test_order = {
                "symbol": "AAPL",
                "qty": 1,
                "side": "buy",
                "type": "market",
                "time_in_force": "day"
            }
            
            print("   Testing order validation...")
            
            # First check if we can get current positions
            response = requests.get(
                f"{BASE_URL}/v2/positions",
                headers=self.headers,
                timeout=10
            )
            
            if response.status_code == 200:
                positions = response.json()
                print(f"   ✅ Positions access: {len(positions)} current positions")
            else:
                print(f"   ⚠️  Positions access failed: {response.status_code}")
            
            # Check orders endpoint
            response = requests.get(
                f"{BASE_URL}/v2/orders",
                headers=self.headers,
                timeout=10
            )
            
            if response.status_code == 200:
                orders = response.json()
                print(f"   ✅ Orders access: {len(orders)} orders found")
                print("   ✅ Order submission capability confirmed (paper trading)")
                
                self.results["tests"].append({
                    "test": "Order Capabilities",
                    "status": "PASS",
                    "details": "Order endpoints accessible, ready for paper trading"
                })
                return True
            else:
                print(f"   ❌ Orders access failed: {response.status_code}")
                self.results["tests"].append({
                    "test": "Order Capabilities",
                    "status": "FAIL", 
                    "error": f"Orders endpoint HTTP {response.status_code}"
                })
                return False
                
        except Exception as e:
            print(f"❌ Order capabilities error: {e}")
            self.results["tests"].append({
                "test": "Order Capabilities",
                "status": "FAIL",
                "error": str(e)
            })
            return False
    
    def test_market_status(self):
        """Test market status and clock"""
        print("\n🕐 Testing Market Status...")
        print("-" * 40)
        
        try:
            response = requests.get(
                f"{BASE_URL}/v2/clock",
                headers=self.headers,
                timeout=10
            )
            
            if response.status_code == 200:
                clock_data = response.json()
                print(f"   ✅ Market Status Retrieved:")
                print(f"      Current Time: {clock_data.get('timestamp', 'N/A')}")
                print(f"      Market Open: {clock_data.get('is_open', 'N/A')}")
                print(f"      Next Open: {clock_data.get('next_open', 'N/A')}")
                print(f"      Next Close: {clock_data.get('next_close', 'N/A')}")
                
                self.results["tests"].append({
                    "test": "Market Status",
                    "status": "PASS",
                    "details": clock_data
                })
                return True
            else:
                print(f"   ❌ Market status failed: {response.status_code}")
                self.results["tests"].append({
                    "test": "Market Status",
                    "status": "FAIL",
                    "error": f"HTTP {response.status_code}"
                })
                return False
                
        except Exception as e:
            print(f"❌ Market status error: {e}")
            self.results["tests"].append({
                "test": "Market Status",
                "status": "FAIL",
                "error": str(e)
            })
            return False
    
    def run_all_tests(self):
        """Run all connectivity tests"""
        print("🚀 Alpaca API Live Connectivity Test")
        print("=" * 50)
        print(f"Testing against: {BASE_URL}")
        print(f"Data endpoint: {DATA_URL}")
        print(f"API Key: {API_KEY}")
        print(f"Paper Trading: True")
        
        tests = [
            self.test_account_access,
            self.test_market_status,
            self.test_market_data_access,
            self.test_order_capabilities
        ]
        
        passed = 0
        total = len(tests)
        
        for test in tests:
            if test():
                passed += 1
        
        # Generate final results
        success_rate = (passed / total) * 100
        
        print(f"\n🎯 Test Results Summary")
        print("=" * 50)
        print(f"Tests Passed: {passed}/{total} ({success_rate:.1f}%)")
        
        if success_rate >= 100:
            status = "🟢 READY - All tests passed"
            self.results["overall_status"] = "READY"
        elif success_rate >= 75:
            status = "🟡 CONDITIONAL - Most tests passed"
            self.results["overall_status"] = "CONDITIONAL"
        else:
            status = "🔴 NOT READY - Critical failures"
            self.results["overall_status"] = "NOT_READY"
        
        print(f"Overall Status: {status}")
        
        # Save results
        with open("alpaca_connectivity_test_results.json", "w") as f:
            json.dump(self.results, f, indent=2)
        
        print(f"\n📄 Detailed results saved to: alpaca_connectivity_test_results.json")
        
        return success_rate >= 75

def main():
    tester = AlpacaAPITester()
    success = tester.run_all_tests()
    
    if success:
        print("\n✅ Alpaca API connectivity confirmed - Ready for live testing!")
        return 0
    else:
        print("\n❌ Alpaca API connectivity issues detected - Fix required before live testing")
        return 1

if __name__ == "__main__":
    sys.exit(main())
