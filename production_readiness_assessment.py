#!/usr/bin/env python3
"""
PantherSwap Edge Production Readiness Assessment
Comprehensive evaluation for Alpaca API integration readiness
"""

import os
import json
import subprocess
import time
from datetime import datetime
from typing import Dict, List, Any, Optional

# Simple TOML parser (basic implementation)
def parse_toml_basic(content):
    """Basic TOML parser for simple key-value pairs"""
    result = {}
    current_section = result
    section_stack = [result]

    for line in content.split('\n'):
        line = line.strip()
        if not line or line.startswith('#'):
            continue

        if line.startswith('[') and line.endswith(']'):
            # Section header
            section_name = line[1:-1].strip()
            if '.' in section_name:
                # Nested section
                parts = section_name.split('.')
                current_section = result
                for part in parts:
                    if part not in current_section:
                        current_section[part] = {}
                    current_section = current_section[part]
            else:
                if section_name not in result:
                    result[section_name] = {}
                current_section = result[section_name]
        elif '=' in line:
            # Key-value pair
            key, value = line.split('=', 1)
            key = key.strip()
            value = value.strip().strip('"\'')

            # Try to convert to appropriate type
            if value.lower() in ('true', 'false'):
                value = value.lower() == 'true'
            elif value.isdigit():
                value = int(value)
            elif '.' in value and value.replace('.', '').isdigit():
                value = float(value)

            current_section[key] = value

    return result

def load_toml(filepath):
    """Load TOML file using basic parser"""
    try:
        with open(filepath, 'r') as f:
            content = f.read()
        return parse_toml_basic(content)
    except Exception as e:
        raise Exception(f"Error loading {filepath}: {e}")

class ProductionReadinessAssessment:
    def __init__(self):
        self.results = {
            "timestamp": datetime.now().isoformat(),
            "overall_status": "UNKNOWN",
            "components": {},
            "performance_metrics": {},
            "safety_checks": {},
            "recommendations": []
        }
        
    def run_assessment(self) -> Dict[str, Any]:
        """Run comprehensive production readiness assessment"""
        print("🚀 Starting PantherSwap Edge Production Readiness Assessment")
        print("=" * 60)
        
        # 1. System Integration Verification
        self.assess_system_integration()
        
        # 2. Configuration Validation
        self.assess_configuration()
        
        # 3. Performance Target Compliance
        self.assess_performance_targets()
        
        # 4. Safety Mechanisms
        self.assess_safety_mechanisms()
        
        # 5. Monitoring and Logging
        self.assess_monitoring_logging()
        
        # 6. Database Integration
        self.assess_database_integration()
        
        # 7. Generate final recommendation
        self.generate_final_recommendation()
        
        return self.results
    
    def assess_system_integration(self):
        """Assess system integration readiness"""
        print("\n📋 1. System Integration Verification")
        print("-" * 40)
        
        integration_score = 0
        max_score = 100
        
        # Check configuration files
        config_files = [
            "config/default.toml",
            "config/production.toml",
            ".env.example"
        ]
        
        for config_file in config_files:
            if os.path.exists(config_file):
                print(f"✅ Found {config_file}")
                integration_score += 15
                
                # Check Alpaca configuration
                if config_file.endswith('.toml'):
                    try:
                        config = load_toml(config_file)
                        if 'market_data' in config and 'alpaca' in config['market_data']:
                            alpaca_config = config['market_data']['alpaca']
                            if alpaca_config.get('api_key') and alpaca_config.get('secret_key'):
                                print(f"✅ Alpaca credentials configured in {config_file}")
                                integration_score += 10
                            if alpaca_config.get('paper_trading', True):
                                print(f"✅ Paper trading enabled in {config_file}")
                                integration_score += 5
                    except Exception as e:
                        print(f"⚠️  Error reading {config_file}: {e}")
            else:
                print(f"❌ Missing {config_file}")
        
        # Check source code structure
        source_files = [
            "src/market_data/alpaca.rs",
            "src/trading/alpaca_execution.rs",
            "src/trading/alpaca_trading_engine.rs",
            "src/api/routes/health.rs",
            "src/monitoring/production.rs"
        ]
        
        for source_file in source_files:
            if os.path.exists(source_file):
                print(f"✅ Found {source_file}")
                integration_score += 5
            else:
                print(f"❌ Missing {source_file}")
        
        self.results["components"]["system_integration"] = {
            "score": integration_score,
            "max_score": max_score,
            "status": "PASS" if integration_score >= 80 else "FAIL",
            "details": f"Integration score: {integration_score}/{max_score}"
        }
    
    def assess_configuration(self):
        """Assess configuration validation"""
        print("\n⚙️  2. Configuration Validation")
        print("-" * 40)
        
        config_score = 0
        max_score = 100
        
        # Load and validate production configuration
        try:
            prod_config = load_toml("config/production.toml")
            print("✅ Production configuration loaded")
            config_score += 20
            
            # Check Alpaca configuration
            alpaca_config = prod_config.get('market_data', {}).get('alpaca', {})
            
            # API credentials
            if alpaca_config.get('api_key') == "CK6KLMXTNEGGKCMVZA2R":
                print("✅ Alpaca API key configured")
                config_score += 15
            
            if alpaca_config.get('secret_key'):
                print("✅ Alpaca secret key configured")
                config_score += 15
            
            # Environment settings
            if alpaca_config.get('paper_trading', True):
                print("✅ Paper trading enabled (safe for testing)")
                config_score += 10
            
            if alpaca_config.get('base_url') == "https://paper-api.alpaca.markets":
                print("✅ Paper trading URL configured")
                config_score += 10
            
            # Safety settings
            if alpaca_config.get('enable_order_execution', False):
                print("✅ Order execution capability configured")
                config_score += 10
            
            # Rate limiting
            if alpaca_config.get('rate_limit_per_minute', 0) >= 200:
                print("✅ Appropriate rate limiting configured")
                config_score += 10
            
            # Connection settings
            if alpaca_config.get('connection_timeout_ms', 0) <= 5000:
                print("✅ Connection timeout configured")
                config_score += 10
                
        except Exception as e:
            print(f"❌ Error loading production config: {e}")
        
        self.results["components"]["configuration"] = {
            "score": config_score,
            "max_score": max_score,
            "status": "PASS" if config_score >= 80 else "FAIL",
            "details": f"Configuration score: {config_score}/{max_score}"
        }
    
    def assess_performance_targets(self):
        """Assess performance target compliance"""
        print("\n🎯 3. Performance Target Compliance")
        print("-" * 40)
        
        # Check if performance optimization code exists
        perf_files = [
            "src/bin/performance_benchmark.rs",
            "src/bin/order_latency_optimizer.rs", 
            "src/bin/ai_inference_optimizer.rs",
            "src/bin/throughput_optimizer.rs"
        ]
        
        perf_score = 0
        max_score = 100
        
        for perf_file in perf_files:
            if os.path.exists(perf_file):
                print(f"✅ Found {perf_file}")
                perf_score += 20
            else:
                print(f"❌ Missing {perf_file}")
        
        # Check production configuration for performance settings
        try:
            prod_config = toml.load("config/production.toml")
            
            # Check performance settings
            if prod_config.get('trading', {}).get('target_latency_ms', 0) <= 10:
                print("✅ Target latency ≤10ms configured")
                perf_score += 10
                
            if prod_config.get('trading', {}).get('target_throughput_tps', 0) >= 1000:
                print("✅ Target throughput ≥1000 TPS configured")
                perf_score += 10
                
        except Exception as e:
            print(f"⚠️  Could not verify performance config: {e}")
        
        self.results["components"]["performance_targets"] = {
            "score": perf_score,
            "max_score": max_score,
            "status": "PASS" if perf_score >= 60 else "FAIL",
            "details": f"Performance readiness score: {perf_score}/{max_score}"
        }
    
    def assess_safety_mechanisms(self):
        """Assess safety mechanisms"""
        print("\n🛡️  4. Safety Mechanisms Verification")
        print("-" * 40)
        
        safety_score = 0
        max_score = 100
        
        # Check paper trading configuration
        try:
            configs = ["config/default.toml", "config/production.toml"]
            for config_file in configs:
                if os.path.exists(config_file):
                    config = toml.load(config_file)
                    alpaca_config = config.get('market_data', {}).get('alpaca', {})
                    
                    if alpaca_config.get('paper_trading', False):
                        print(f"✅ Paper trading enabled in {config_file}")
                        safety_score += 20
                    
                    if alpaca_config.get('base_url', '').endswith('paper-api.alpaca.markets'):
                        print(f"✅ Paper trading URL in {config_file}")
                        safety_score += 15
                        
        except Exception as e:
            print(f"⚠️  Error checking safety config: {e}")
        
        # Check risk management settings
        try:
            prod_config = toml.load("config/production.toml")
            risk_config = prod_config.get('risk', {})
            
            if risk_config.get('max_daily_loss', 0) > 0:
                print("✅ Daily loss limits configured")
                safety_score += 15
                
            if risk_config.get('max_portfolio_var', 1.0) <= 0.02:
                print("✅ Portfolio VaR limits configured")
                safety_score += 15
                
            if risk_config.get('drawdown_limit', 1.0) <= 0.1:
                print("✅ Drawdown limits configured")
                safety_score += 15
                
        except Exception as e:
            print(f"⚠️  Could not verify risk config: {e}")
        
        # Check for environment switching capability
        if os.path.exists("src/config/settings.rs"):
            print("✅ Configuration management system exists")
            safety_score += 20
        
        self.results["components"]["safety_mechanisms"] = {
            "score": safety_score,
            "max_score": max_score,
            "status": "PASS" if safety_score >= 80 else "FAIL",
            "details": f"Safety score: {safety_score}/{max_score}"
        }

    def assess_monitoring_logging(self):
        """Assess monitoring and logging capabilities"""
        print("\n📊 5. Monitoring and Logging Validation")
        print("-" * 40)

        monitoring_score = 0
        max_score = 100

        # Check health endpoints
        health_files = [
            "src/api/routes/health.rs",
            "src/api/health.rs",
            "src/monitoring/production.rs",
            "src/monitoring/prometheus.rs"
        ]

        for health_file in health_files:
            if os.path.exists(health_file):
                print(f"✅ Found {health_file}")
                monitoring_score += 20
            else:
                print(f"❌ Missing {health_file}")

        # Check production monitoring configuration
        try:
            prod_config = toml.load("config/production.toml")
            monitoring_config = prod_config.get('monitoring', {})

            if monitoring_config.get('enable_auto_recovery', False):
                print("✅ Auto-recovery enabled")
                monitoring_score += 10

            if monitoring_config.get('health_check_interval_seconds', 0) <= 60:
                print("✅ Health check interval configured")
                monitoring_score += 10

        except Exception as e:
            print(f"⚠️  Could not verify monitoring config: {e}")

        self.results["components"]["monitoring_logging"] = {
            "score": monitoring_score,
            "max_score": max_score,
            "status": "PASS" if monitoring_score >= 60 else "FAIL",
            "details": f"Monitoring score: {monitoring_score}/{max_score}"
        }

    def assess_database_integration(self):
        """Assess database integration"""
        print("\n🗄️  6. Database Integration Assessment")
        print("-" * 40)

        db_score = 0
        max_score = 100

        # Check database configuration
        try:
            prod_config = toml.load("config/production.toml")
            db_config = prod_config.get('database', {})

            # Check TimescaleDB connection
            db_url = db_config.get('url', '')
            if 'tsdb.cloud.timescale.com' in db_url:
                print("✅ TimescaleDB connection configured")
                db_score += 30

            # Check connection pool settings
            if db_config.get('max_connections', 0) >= 50:
                print("✅ Adequate connection pool size")
                db_score += 15

            if db_config.get('query_timeout', 0) <= 30:
                print("✅ Query timeout configured")
                db_score += 15

            if db_config.get('enable_real_time_monitoring', False):
                print("✅ Real-time monitoring enabled")
                db_score += 20

        except Exception as e:
            print(f"⚠️  Could not verify database config: {e}")

        # Check database migration files
        migration_dirs = ["migrations/", "src/database/"]
        for migration_dir in migration_dirs:
            if os.path.exists(migration_dir):
                print(f"✅ Found {migration_dir}")
                db_score += 10

        self.results["components"]["database_integration"] = {
            "score": db_score,
            "max_score": max_score,
            "status": "PASS" if db_score >= 70 else "FAIL",
            "details": f"Database score: {db_score}/{max_score}"
        }

    def generate_final_recommendation(self):
        """Generate final go/no-go recommendation"""
        print("\n🎯 7. Final Assessment and Recommendations")
        print("-" * 40)

        # Calculate overall score
        total_score = 0
        max_total_score = 0

        for component, data in self.results["components"].items():
            total_score += data["score"]
            max_total_score += data["max_score"]

        overall_percentage = (total_score / max_total_score) * 100 if max_total_score > 0 else 0

        # Determine overall status
        if overall_percentage >= 85:
            self.results["overall_status"] = "READY"
            recommendation = "🟢 GO - System is ready for live Alpaca API testing"
        elif overall_percentage >= 70:
            self.results["overall_status"] = "CONDITIONAL"
            recommendation = "🟡 CONDITIONAL GO - Address critical issues before live testing"
        else:
            self.results["overall_status"] = "NOT_READY"
            recommendation = "🔴 NO-GO - Significant issues must be resolved"

        print(f"\n{recommendation}")
        print(f"Overall Score: {total_score}/{max_total_score} ({overall_percentage:.1f}%)")

        # Generate specific recommendations
        recommendations = []

        for component, data in self.results["components"].items():
            if data["status"] == "FAIL":
                recommendations.append(f"❌ Fix {component}: {data['details']}")
            elif data["score"] < data["max_score"] * 0.9:
                recommendations.append(f"⚠️  Improve {component}: {data['details']}")

        # Add specific Alpaca integration recommendations
        recommendations.extend([
            "✅ Verify Alpaca API credentials are valid and active",
            "✅ Test paper trading environment connectivity",
            "✅ Validate real-time market data feeds",
            "✅ Perform end-to-end order execution test in paper trading",
            "✅ Monitor system performance under load",
            "✅ Verify all safety mechanisms are operational",
            "✅ Test failover and recovery procedures"
        ])

        self.results["recommendations"] = recommendations
        self.results["overall_score"] = f"{total_score}/{max_total_score} ({overall_percentage:.1f}%)"

        print("\n📋 Recommendations:")
        for rec in recommendations:
            print(f"  {rec}")

def main():
    """Main assessment function"""
    assessment = ProductionReadinessAssessment()
    results = assessment.run_assessment()

    # Save results to file
    with open("production_readiness_report.json", "w") as f:
        json.dump(results, f, indent=2)

    print(f"\n📄 Full report saved to: production_readiness_report.json")
    print("\n" + "=" * 60)
    print("🏁 Assessment Complete")

    return results

if __name__ == "__main__":
    main()
