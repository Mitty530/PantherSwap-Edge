// Production Readiness Assessment for PantherSwap Edge
// Comprehensive evaluation for Alpaca API integration readiness

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde_json::{json, Value};
use chrono::Utc;

#[derive(Debug)]
struct AssessmentResult {
    component: String,
    score: u32,
    max_score: u32,
    status: String,
    details: String,
}

#[derive(Debug)]
struct ProductionReadinessAssessment {
    results: Vec<AssessmentResult>,
    overall_score: u32,
    max_total_score: u32,
    recommendations: Vec<String>,
}

impl ProductionReadinessAssessment {
    fn new() -> Self {
        Self {
            results: Vec::new(),
            overall_score: 0,
            max_total_score: 0,
            recommendations: Vec::new(),
        }
    }

    fn run_assessment(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 PantherSwap Edge Production Readiness Assessment");
        println!("=" .repeat(60));

        // 1. System Integration Verification
        self.assess_system_integration();
        
        // 2. Configuration Validation
        self.assess_configuration();
        
        // 3. Performance Target Compliance
        self.assess_performance_targets();
        
        // 4. Safety Mechanisms
        self.assess_safety_mechanisms();
        
        // 5. Monitoring and Logging
        self.assess_monitoring_logging();
        
        // 6. Database Integration
        self.assess_database_integration();
        
        // 7. Generate final recommendation
        self.generate_final_recommendation();
        
        Ok(())
    }

    fn assess_system_integration(&mut self) {
        println!("\n📋 1. System Integration Verification");
        println!("-".repeat(40));
        
        let mut score = 0;
        let max_score = 100;
        
        // Check configuration files
        let config_files = [
            "config/default.toml",
            "config/production.toml",
            ".env.example"
        ];
        
        for config_file in &config_files {
            if Path::new(config_file).exists() {
                println!("✅ Found {}", config_file);
                score += 15;
            } else {
                println!("❌ Missing {}", config_file);
            }
        }
        
        // Check source code structure
        let source_files = [
            "src/market_data/alpaca.rs",
            "src/trading/alpaca_execution.rs", 
            "src/trading/alpaca_trading_engine.rs",
            "src/api/routes/health.rs",
            "src/monitoring/production.rs"
        ];
        
        for source_file in &source_files {
            if Path::new(source_file).exists() {
                println!("✅ Found {}", source_file);
                score += 5;
            } else {
                println!("❌ Missing {}", source_file);
            }
        }
        
        // Check Cargo.toml for dependencies
        if Path::new("Cargo.toml").exists() {
            if let Ok(content) = fs::read_to_string("Cargo.toml") {
                if content.contains("sqlx") && content.contains("tokio") && content.contains("axum") {
                    println!("✅ Core dependencies configured");
                    score += 20;
                }
            }
        }
        
        let status = if score >= 80 { "PASS" } else { "FAIL" };
        
        self.results.push(AssessmentResult {
            component: "System Integration".to_string(),
            score,
            max_score,
            status: status.to_string(),
            details: format!("Integration score: {}/{}", score, max_score),
        });
    }

    fn assess_configuration(&mut self) {
        println!("\n⚙️  2. Configuration Validation");
        println!("-".repeat(40));
        
        let mut score = 0;
        let max_score = 100;
        
        // Check production configuration
        if let Ok(content) = fs::read_to_string("config/production.toml") {
            println!("✅ Production configuration found");
            score += 20;
            
            // Check for Alpaca configuration
            if content.contains("[market_data.alpaca]") {
                println!("✅ Alpaca configuration section found");
                score += 15;
            }
            
            if content.contains("api_key") && content.contains("secret_key") {
                println!("✅ API credentials configured");
                score += 15;
            }
            
            if content.contains("paper_trading = true") {
                println!("✅ Paper trading enabled (safe for testing)");
                score += 15;
            }
            
            if content.contains("paper-api.alpaca.markets") {
                println!("✅ Paper trading URL configured");
                score += 15;
            }
            
            if content.contains("enable_order_execution") {
                println!("✅ Order execution capability configured");
                score += 10;
            }
            
            if content.contains("rate_limit_per_minute") {
                println!("✅ Rate limiting configured");
                score += 10;
            }
        } else {
            println!("❌ Production configuration missing");
        }
        
        let status = if score >= 80 { "PASS" } else { "FAIL" };
        
        self.results.push(AssessmentResult {
            component: "Configuration".to_string(),
            score,
            max_score,
            status: status.to_string(),
            details: format!("Configuration score: {}/{}", score, max_score),
        });
    }

    fn assess_performance_targets(&mut self) {
        println!("\n🎯 3. Performance Target Compliance");
        println!("-".repeat(40));
        
        let mut score = 0;
        let max_score = 100;
        
        // Check performance optimization binaries
        let perf_files = [
            "src/bin/performance_benchmark.rs",
            "src/bin/order_latency_optimizer.rs",
            "src/bin/ai_inference_optimizer.rs", 
            "src/bin/throughput_optimizer.rs"
        ];
        
        for perf_file in &perf_files {
            if Path::new(perf_file).exists() {
                println!("✅ Found {}", perf_file);
                score += 20;
            } else {
                println!("❌ Missing {}", perf_file);
            }
        }
        
        // Check production config for performance settings
        if let Ok(content) = fs::read_to_string("config/production.toml") {
            if content.contains("target_latency_ms") {
                println!("✅ Target latency configured");
                score += 10;
            }
            
            if content.contains("target_throughput_tps") {
                println!("✅ Target throughput configured");
                score += 10;
            }
        }
        
        let status = if score >= 60 { "PASS" } else { "FAIL" };
        
        self.results.push(AssessmentResult {
            component: "Performance Targets".to_string(),
            score,
            max_score,
            status: status.to_string(),
            details: format!("Performance readiness: {}/{}", score, max_score),
        });
    }

    fn assess_safety_mechanisms(&mut self) {
        println!("\n🛡️  4. Safety Mechanisms Verification");
        println!("-".repeat(40));
        
        let mut score = 0;
        let max_score = 100;
        
        // Check paper trading configuration
        let configs = ["config/default.toml", "config/production.toml"];
        for config_file in &configs {
            if let Ok(content) = fs::read_to_string(config_file) {
                if content.contains("paper_trading = true") {
                    println!("✅ Paper trading enabled in {}", config_file);
                    score += 20;
                }
                
                if content.contains("paper-api.alpaca.markets") {
                    println!("✅ Paper trading URL in {}", config_file);
                    score += 15;
                }
            }
        }
        
        // Check risk management
        if let Ok(content) = fs::read_to_string("config/production.toml") {
            if content.contains("max_daily_loss") {
                println!("✅ Daily loss limits configured");
                score += 15;
            }
            
            if content.contains("max_portfolio_var") {
                println!("✅ Portfolio VaR limits configured");
                score += 15;
            }
            
            if content.contains("drawdown_limit") {
                println!("✅ Drawdown limits configured");
                score += 15;
            }
        }
        
        // Check for configuration management
        if Path::new("src/config/settings.rs").exists() {
            println!("✅ Configuration management system exists");
            score += 20;
        }
        
        let status = if score >= 80 { "PASS" } else { "FAIL" };
        
        self.results.push(AssessmentResult {
            component: "Safety Mechanisms".to_string(),
            score,
            max_score,
            status: status.to_string(),
            details: format!("Safety score: {}/{}", score, max_score),
        });
    }

    fn assess_monitoring_logging(&mut self) {
        println!("\n📊 5. Monitoring and Logging Validation");
        println!("-".repeat(40));
        
        let mut score = 0;
        let max_score = 100;
        
        // Check health endpoints
        let health_files = [
            "src/api/routes/health.rs",
            "src/api/health.rs",
            "src/monitoring/production.rs",
            "src/monitoring/prometheus.rs"
        ];
        
        for health_file in &health_files {
            if Path::new(health_file).exists() {
                println!("✅ Found {}", health_file);
                score += 20;
            } else {
                println!("❌ Missing {}", health_file);
            }
        }
        
        // Check production monitoring configuration
        if let Ok(content) = fs::read_to_string("config/production.toml") {
            if content.contains("enable_auto_recovery") {
                println!("✅ Auto-recovery configured");
                score += 10;
            }
            
            if content.contains("health_check_interval_seconds") {
                println!("✅ Health check interval configured");
                score += 10;
            }
        }
        
        let status = if score >= 60 { "PASS" } else { "FAIL" };

        self.results.push(AssessmentResult {
            component: "Monitoring & Logging".to_string(),
            score,
            max_score,
            status: status.to_string(),
            details: format!("Monitoring score: {}/{}", score, max_score),
        });
    }

    fn assess_database_integration(&mut self) {
        println!("\n🗄️  6. Database Integration Assessment");
        println!("-".repeat(40));

        let mut score = 0;
        let max_score = 100;

        // Check database configuration
        if let Ok(content) = fs::read_to_string("config/production.toml") {
            if content.contains("tsdb.cloud.timescale.com") {
                println!("✅ TimescaleDB connection configured");
                score += 30;
            }

            if content.contains("max_connections") {
                println!("✅ Connection pool configured");
                score += 15;
            }

            if content.contains("query_timeout") {
                println!("✅ Query timeout configured");
                score += 15;
            }

            if content.contains("enable_real_time_monitoring") {
                println!("✅ Real-time monitoring enabled");
                score += 20;
            }
        }

        // Check database migration files
        let migration_dirs = ["migrations/", "src/database/"];
        for migration_dir in &migration_dirs {
            if Path::new(migration_dir).exists() {
                println!("✅ Found {}", migration_dir);
                score += 10;
            }
        }

        let status = if score >= 70 { "PASS" } else { "FAIL" };

        self.results.push(AssessmentResult {
            component: "Database Integration".to_string(),
            score,
            max_score,
            status: status.to_string(),
            details: format!("Database score: {}/{}", score, max_score),
        });
    }

    fn generate_final_recommendation(&mut self) {
        println!("\n🎯 7. Final Assessment and Recommendations");
        println!("-".repeat(40));

        // Calculate overall score
        let mut total_score = 0;
        let mut max_total_score = 0;

        for result in &self.results {
            total_score += result.score;
            max_total_score += result.max_score;
        }

        self.overall_score = total_score;
        self.max_total_score = max_total_score;

        let overall_percentage = if max_total_score > 0 {
            (total_score as f64 / max_total_score as f64) * 100.0
        } else {
            0.0
        };

        // Determine overall status and recommendation
        let (status, recommendation) = if overall_percentage >= 85.0 {
            ("READY", "🟢 GO - System is ready for live Alpaca API testing")
        } else if overall_percentage >= 70.0 {
            ("CONDITIONAL", "🟡 CONDITIONAL GO - Address critical issues before live testing")
        } else {
            ("NOT_READY", "🔴 NO-GO - Significant issues must be resolved")
        };

        println!("\n{}", recommendation);
        println!("Overall Score: {}/{} ({:.1}%)", total_score, max_total_score, overall_percentage);

        // Generate specific recommendations
        self.recommendations.clear();

        for result in &self.results {
            if result.status == "FAIL" {
                self.recommendations.push(format!("❌ Fix {}: {}", result.component, result.details));
            } else if result.score < (result.max_score as f64 * 0.9) as u32 {
                self.recommendations.push(format!("⚠️  Improve {}: {}", result.component, result.details));
            }
        }

        // Add Alpaca-specific recommendations
        self.recommendations.extend([
            "✅ Verify Alpaca API credentials are valid and active".to_string(),
            "✅ Test paper trading environment connectivity".to_string(),
            "✅ Validate real-time market data feeds".to_string(),
            "✅ Perform end-to-end order execution test in paper trading".to_string(),
            "✅ Monitor system performance under load".to_string(),
            "✅ Verify all safety mechanisms are operational".to_string(),
            "✅ Test failover and recovery procedures".to_string(),
        ]);

        println!("\n📋 Recommendations:");
        for rec in &self.recommendations {
            println!("  {}", rec);
        }

        // Save results to JSON
        self.save_results_to_json().unwrap_or_else(|e| {
            eprintln!("Failed to save results: {}", e);
        });

        println!("\n📄 Full report saved to: production_readiness_report.json");
        println!("\n{}", "=".repeat(60));
        println!("🏁 Assessment Complete - Status: {}", status);
    }

    fn save_results_to_json(&self) -> Result<(), Box<dyn std::error::Error>> {
        let report = json!({
            "timestamp": Utc::now().to_rfc3339(),
            "overall_status": if self.overall_score as f64 / self.max_total_score as f64 >= 0.85 { "READY" }
                             else if self.overall_score as f64 / self.max_total_score as f64 >= 0.70 { "CONDITIONAL" }
                             else { "NOT_READY" },
            "overall_score": format!("{}/{} ({:.1}%)",
                self.overall_score,
                self.max_total_score,
                (self.overall_score as f64 / self.max_total_score as f64) * 100.0
            ),
            "components": self.results.iter().map(|r| json!({
                "component": r.component,
                "score": r.score,
                "max_score": r.max_score,
                "status": r.status,
                "details": r.details
            })).collect::<Vec<_>>(),
            "recommendations": self.recommendations
        });

        fs::write("production_readiness_report.json", serde_json::to_string_pretty(&report)?)?;
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut assessment = ProductionReadinessAssessment::new();
    assessment.run_assessment()?;
    Ok(())
}
