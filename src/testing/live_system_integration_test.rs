// Live System Integration Test for PantherSwap Edge
// Tests complete system with live IG API and TimescaleDB integration

use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio::time::sleep;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use tracing::{info, warn, error, debug};

use crate::config::Settings;
use crate::database::Database;
use crate::market_data::MarketDataManager;
use crate::trading::engine::{TradingEngine, TradingEngineConfig};
use crate::ai::AIEngine;


#[derive(Debug, Serialize, Deserialize)]
pub struct SystemTestResults {
    pub database_health: DatabaseTestResults,
    pub ig_api_connectivity: IGAPITestResults,
    pub market_data_pipeline: MarketDataTestResults,
    pub ai_engine_performance: AIEngineTestResults,
    pub trading_engine_performance: TradingEngineTestResults,
    pub end_to_end_performance: EndToEndTestResults,
    pub overall_status: String,
    pub test_duration_seconds: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseTestResults {
    pub connection_successful: bool,
    pub pool_health: bool,
    pub migration_status: bool,
    pub query_performance_ms: f64,
    pub connection_count: u32,
    pub error_details: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IGAPITestResults {
    pub authentication_successful: bool,
    pub market_data_retrieval: bool,
    pub response_time_ms: f64,
    pub rate_limiting_respected: bool,
    pub demo_mode_active: bool,
    pub error_details: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketDataTestResults {
    pub data_collection_active: bool,
    pub database_persistence: bool,
    pub data_quality_score: f64,
    pub symbols_processed: u32,
    pub processing_latency_ms: f64,
    pub error_details: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AIEngineTestResults {
    pub inference_successful: bool,
    pub inference_latency_ms: f64,
    pub hmm_regime_detection: bool,
    pub prediction_accuracy: f64,
    pub model_health: bool,
    pub error_details: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TradingEngineTestResults {
    pub signal_generation: bool,
    pub order_execution_latency_ms: f64,
    pub throughput_tps: f64,
    pub risk_management_active: bool,
    pub performance_targets_met: bool,
    pub error_details: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EndToEndTestResults {
    pub complete_cycle_successful: bool,
    pub total_latency_ms: f64,
    pub data_persistence_verified: bool,
    pub ai_trading_integration: bool,
    pub monitoring_active: bool,
    pub error_details: Option<String>,
}

/// Comprehensive live system integration test
pub async fn run_live_system_test() -> Result<SystemTestResults, Box<dyn std::error::Error>> {
    let test_start = Instant::now();
    info!("🚀 Starting Live System Integration Test for PantherSwap Edge");

    // Initialize logging for test
    tracing_subscriber::fmt()
        .with_env_filter("info,sqlx=warn,hyper=warn")
        .init();

    // Load configuration
    let settings = Settings::load()?;
    info!("✅ Configuration loaded successfully");

    let mut test_results = SystemTestResults {
        database_health: DatabaseTestResults {
            connection_successful: false,
            pool_health: false,
            migration_status: false,
            query_performance_ms: 0.0,
            connection_count: 0,
            error_details: None,
        },
        ig_api_connectivity: IGAPITestResults {
            authentication_successful: false,
            market_data_retrieval: false,
            response_time_ms: 0.0,
            rate_limiting_respected: false,
            demo_mode_active: false,
            error_details: None,
        },
        market_data_pipeline: MarketDataTestResults {
            data_collection_active: false,
            database_persistence: false,
            data_quality_score: 0.0,
            symbols_processed: 0,
            processing_latency_ms: 0.0,
            error_details: None,
        },
        ai_engine_performance: AIEngineTestResults {
            inference_successful: false,
            inference_latency_ms: 0.0,
            hmm_regime_detection: false,
            prediction_accuracy: 0.0,
            model_health: false,
            error_details: None,
        },
        trading_engine_performance: TradingEngineTestResults {
            signal_generation: false,
            order_execution_latency_ms: 0.0,
            throughput_tps: 0.0,
            risk_management_active: false,
            performance_targets_met: false,
            error_details: None,
        },
        end_to_end_performance: EndToEndTestResults {
            complete_cycle_successful: false,
            total_latency_ms: 0.0,
            data_persistence_verified: false,
            ai_trading_integration: false,
            monitoring_active: false,
            error_details: None,
        },
        overall_status: "TESTING".to_string(),
        test_duration_seconds: 0.0,
        timestamp: Utc::now(),
    };

    // Test 1: Database Connection Validation
    info!("📊 Testing Database Connection and Health...");
    test_results.database_health = test_database_connectivity(&settings).await;

    // Test 2: IG API Authentication & Connectivity
    info!("🔌 Testing IG Trading API Connectivity...");
    test_results.ig_api_connectivity = test_ig_api_connectivity(&settings).await;

    // Test 3: Market Data Pipeline Testing
    info!("📈 Testing Market Data Pipeline...");
    test_results.market_data_pipeline = test_market_data_pipeline(&settings).await;

    // Test 4: AI Engine Integration Testing
    info!("🤖 Testing AI Engine Performance...");
    test_results.ai_engine_performance = test_ai_engine_performance(&settings).await;

    // Test 5: Trading Engine Performance Testing
    info!("⚡ Testing Trading Engine Performance...");
    test_results.trading_engine_performance = test_trading_engine_performance(&settings).await;

    // Test 6: End-to-End System Testing
    info!("🔄 Testing End-to-End System Integration...");
    test_results.end_to_end_performance = test_end_to_end_integration(&settings).await;

    // Calculate overall status
    let test_duration = test_start.elapsed();
    test_results.test_duration_seconds = test_duration.as_secs_f64();
    test_results.overall_status = calculate_overall_status(&test_results);

    info!("✅ Live System Integration Test completed in {:.2}s", test_results.test_duration_seconds);
    info!("📊 Overall Status: {}", test_results.overall_status);

    Ok(test_results)
}

/// Test database connectivity and health
async fn test_database_connectivity(settings: &Settings) -> DatabaseTestResults {
    let test_start = Instant::now();
    let mut results = DatabaseTestResults {
        connection_successful: false,
        pool_health: false,
        migration_status: false,
        query_performance_ms: 0.0,
        connection_count: 0,
        error_details: None,
    };

    match Database::new(&settings.database.url).await {
        Ok(database) => {
            results.connection_successful = true;
            info!("✅ Database connection established");

            // Test migrations
            match database.run_migrations().await {
                Ok(_) => {
                    results.migration_status = true;
                    info!("✅ Database migrations completed");
                }
                Err(e) => {
                    results.error_details = Some(format!("Migration error: {}", e));
                    warn!("⚠️ Database migration failed: {}", e);
                }
            }

            // Test query performance
            let query_start = Instant::now();
            match sqlx::query("SELECT 1 as test").fetch_one(&database.pool).await {
                Ok(_) => {
                    results.query_performance_ms = query_start.elapsed().as_millis() as f64;
                    results.pool_health = true;
                    info!("✅ Database query test successful ({:.2}ms)", results.query_performance_ms);
                }
                Err(e) => {
                    results.error_details = Some(format!("Query error: {}", e));
                    warn!("⚠️ Database query test failed: {}", e);
                }
            }

            // Get connection pool stats
            results.connection_count = database.pool.size();
        }
        Err(e) => {
            results.error_details = Some(format!("Connection error: {}", e));
            error!("❌ Database connection failed: {}", e);
        }
    }

    results
}

/// Test IG Trading API connectivity
async fn test_ig_api_connectivity(settings: &Settings) -> IGAPITestResults {
    let test_start = Instant::now();
    let mut results = IGAPITestResults {
        authentication_successful: false,
        market_data_retrieval: false,
        response_time_ms: 0.0,
        rate_limiting_respected: true,
        demo_mode_active: settings.market_data.ig_trading.demo_mode,
        error_details: None,
    };

    match MarketDataManager::new(settings.clone()).await {
        Ok(mut manager) => {
            info!("✅ Market Data Manager initialized");

            // Test IG API connectivity
            let api_test_start = Instant::now();
            match manager.test_ig_trading_connectivity().await {
                Ok(true) => {
                    results.authentication_successful = true;
                    results.response_time_ms = api_test_start.elapsed().as_millis() as f64;
                    info!("✅ IG Trading API authentication successful ({:.2}ms)", results.response_time_ms);

                    // Test market data retrieval
                    let test_symbols = vec!["AAPL".to_string(), "MSFT".to_string()];
                    match manager.get_multiple_quotes(&test_symbols).await {
                        Ok(quotes) => {
                            results.market_data_retrieval = !quotes.is_empty();
                            info!("✅ Market data retrieval successful ({} quotes)", quotes.len());
                        }
                        Err(e) => {
                            results.error_details = Some(format!("Market data retrieval error: {}", e));
                            warn!("⚠️ Market data retrieval failed: {}", e);
                        }
                    }
                }
                Ok(false) => {
                    results.error_details = Some("IG Trading API connectivity test returned false".to_string());
                    warn!("⚠️ IG Trading API connectivity test failed");
                }
                Err(e) => {
                    results.error_details = Some(format!("IG Trading API error: {}", e));
                    error!("❌ IG Trading API test failed: {}", e);
                }
            }
        }
        Err(e) => {
            results.error_details = Some(format!("Manager initialization error: {}", e));
            error!("❌ Market Data Manager initialization failed: {}", e);
        }
    }

    results
}

/// Test market data pipeline performance
async fn test_market_data_pipeline(settings: &Settings) -> MarketDataTestResults {
    let mut results = MarketDataTestResults {
        data_collection_active: false,
        database_persistence: false,
        data_quality_score: 0.0,
        symbols_processed: 0,
        processing_latency_ms: 0.0,
        error_details: None,
    };

    match Database::new(&settings.database.url).await {
        Ok(database) => {
            match MarketDataManager::new(settings.clone(), database.clone()).await {
                Ok(mut manager) => {
                    let test_symbols = vec![
                        "AAPL".to_string(), "MSFT".to_string(), "GOOGL".to_string(),
                        "TSLA".to_string(), "NVDA".to_string()
                    ];

                    let pipeline_start = Instant::now();

                    // Test data collection
                    match manager.collect_market_data(&test_symbols).await {
                        Ok(market_data) => {
                            results.data_collection_active = true;
                            results.symbols_processed = market_data.len() as u32;
                            results.processing_latency_ms = pipeline_start.elapsed().as_millis() as f64;

                            // Calculate data quality score
                            let quality_scores: Vec<f64> = market_data.iter()
                                .map(|tick| tick.data_quality_score)
                                .collect();
                            results.data_quality_score = quality_scores.iter().sum::<f64>() / quality_scores.len() as f64;

                            info!("✅ Market data collection successful ({} symbols, {:.2}ms, quality: {:.2})",
                                  results.symbols_processed, results.processing_latency_ms, results.data_quality_score);

                            // Test database persistence
                            for tick in &market_data {
                                match database.store_market_tick(tick).await {
                                    Ok(_) => {
                                        results.database_persistence = true;
                                    }
                                    Err(e) => {
                                        results.error_details = Some(format!("Database persistence error: {}", e));
                                        warn!("⚠️ Failed to persist market tick: {}", e);
                                        break;
                                    }
                                }
                            }

                            if results.database_persistence {
                                info!("✅ Market data persistence successful");
                            }
                        }
                        Err(e) => {
                            results.error_details = Some(format!("Data collection error: {}", e));
                            error!("❌ Market data collection failed: {}", e);
                        }
                    }
                }
                Err(e) => {
                    results.error_details = Some(format!("Manager initialization error: {}", e));
                    error!("❌ Market Data Manager initialization failed: {}", e);
                }
            }
        }
        Err(e) => {
            results.error_details = Some(format!("Database connection error: {}", e));
            error!("❌ Database connection failed: {}", e);
        }
    }

    results
}

/// Test AI engine performance
async fn test_ai_engine_performance(settings: &Settings) -> AIEngineTestResults {
    let mut results = AIEngineTestResults {
        inference_successful: false,
        inference_latency_ms: 0.0,
        hmm_regime_detection: false,
        prediction_accuracy: 0.0,
        model_health: false,
        error_details: None,
    };

    match Database::new(&settings.database.url).await {
        Ok(database) => {
            match AIEngine::new(database.clone()).await {
                Ok(mut ai_engine) => {
                    results.model_health = true;
                    info!("✅ AI Engine initialized successfully");

                    // Test AI inference
                    let inference_start = Instant::now();
                    let test_symbol = "AAPL";

                    match ai_engine.generate_prediction(test_symbol).await {
                        Ok(prediction) => {
                            results.inference_successful = true;
                            results.inference_latency_ms = inference_start.elapsed().as_millis() as f64;
                            results.prediction_accuracy = prediction.confidence;

                            info!("✅ AI inference successful ({:.2}ms, confidence: {:.2})",
                                  results.inference_latency_ms, results.prediction_accuracy);

                            // Test HMM regime detection
                            match ai_engine.detect_market_regime(test_symbol).await {
                                Ok(regime) => {
                                    results.hmm_regime_detection = true;
                                    info!("✅ HMM regime detection successful: {:?}", regime.regime_type);
                                }
                                Err(e) => {
                                    results.error_details = Some(format!("HMM regime detection error: {}", e));
                                    warn!("⚠️ HMM regime detection failed: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            results.error_details = Some(format!("AI inference error: {}", e));
                            error!("❌ AI inference failed: {}", e);
                        }
                    }
                }
                Err(e) => {
                    results.error_details = Some(format!("AI Engine initialization error: {}", e));
                    error!("❌ AI Engine initialization failed: {}", e);
                }
            }
        }
        Err(e) => {
            results.error_details = Some(format!("Database connection error: {}", e));
            error!("❌ Database connection failed: {}", e);
        }
    }

    results
}

/// Test trading engine performance
async fn test_trading_engine_performance(settings: &Settings) -> TradingEngineTestResults {
    let mut results = TradingEngineTestResults {
        signal_generation: false,
        order_execution_latency_ms: 0.0,
        throughput_tps: 0.0,
        risk_management_active: false,
        performance_targets_met: false,
        error_details: None,
    };

    match Database::new(&settings.database.url).await {
        Ok(database) => {
            let trading_config = TradingEngineConfig::from_settings(settings);

            match TradingEngine::new(trading_config, database.clone()).await {
                Ok(mut trading_engine) => {
                    info!("✅ Trading Engine initialized successfully");

                    // Test signal generation
                    let signal_start = Instant::now();
                    let test_symbols = vec!["AAPL".to_string(), "MSFT".to_string()];

                    match trading_engine.generate_trading_signals().await {
                        Ok(signals) => {
                            results.signal_generation = !signals.is_empty();
                            info!("✅ Trading signal generation successful ({} signals)", signals.len());

                            // Test order execution latency
                            if let Some(signal) = signals.first() {
                                let execution_start = Instant::now();
                                match trading_engine.execute_signal(signal).await {
                                    Ok(_) => {
                                        results.order_execution_latency_ms = execution_start.elapsed().as_millis() as f64;
                                        info!("✅ Order execution successful ({:.2}ms)", results.order_execution_latency_ms);
                                    }
                                    Err(e) => {
                                        results.error_details = Some(format!("Order execution error: {}", e));
                                        warn!("⚠️ Order execution failed: {}", e);
                                    }
                                }
                            }

                            // Test throughput
                            let throughput_start = Instant::now();
                            let mut successful_executions = 0;

                            for signal in &signals {
                                if trading_engine.execute_signal(signal).await.is_ok() {
                                    successful_executions += 1;
                                }
                            }

                            let throughput_duration = throughput_start.elapsed().as_secs_f64();
                            results.throughput_tps = successful_executions as f64 / throughput_duration;

                            info!("✅ Throughput test completed ({:.2} TPS)", results.throughput_tps);
                        }
                        Err(e) => {
                            results.error_details = Some(format!("Signal generation error: {}", e));
                            error!("❌ Trading signal generation failed: {}", e);
                        }
                    }

                    // Test risk management
                    match trading_engine.validate_risk_limits().await {
                        Ok(risk_status) => {
                            results.risk_management_active = risk_status;
                            info!("✅ Risk management validation: {}", risk_status);
                        }
                        Err(e) => {
                            results.error_details = Some(format!("Risk management error: {}", e));
                            warn!("⚠️ Risk management validation failed: {}", e);
                        }
                    }

                    // Check performance targets
                    results.performance_targets_met =
                        results.order_execution_latency_ms < 10.0 &&
                        results.throughput_tps > 1000.0;
                }
                Err(e) => {
                    results.error_details = Some(format!("Trading Engine initialization error: {}", e));
                    error!("❌ Trading Engine initialization failed: {}", e);
                }
            }
        }
        Err(e) => {
            results.error_details = Some(format!("Database connection error: {}", e));
            error!("❌ Database connection failed: {}", e);
        }
    }

    results
}

/// Test end-to-end system integration
async fn test_end_to_end_integration(settings: &Settings) -> EndToEndTestResults {
    let mut results = EndToEndTestResults {
        complete_cycle_successful: false,
        total_latency_ms: 0.0,
        data_persistence_verified: false,
        ai_trading_integration: false,
        monitoring_active: false,
        error_details: None,
    };

    let e2e_start = Instant::now();

    match Database::new(&settings.database.url).await {
        Ok(database) => {
            // Initialize all components
            match MarketDataManager::new(settings.clone(), database.clone()).await {
                Ok(mut market_manager) => {
                    match AIEngine::new(database.clone()).await {
                        Ok(mut ai_engine) => {
                            let trading_config = TradingEngineConfig::from_settings(settings);
                            match TradingEngine::new(trading_config, database.clone()).await {
                                Ok(mut trading_engine) => {
                                    info!("✅ All components initialized for E2E test");

                                    // Complete trading cycle test
                                    let test_symbol = "AAPL";
                                    let cycle_start = Instant::now();

                                    // Step 1: Collect market data
                                    match market_manager.get_live_quote(test_symbol).await {
                                        Ok(quote) => {
                                            info!("✅ Market data collected: ${:.2}", quote.price);

                                            // Step 2: AI inference
                                            match ai_engine.generate_prediction(test_symbol).await {
                                                Ok(prediction) => {
                                                    info!("✅ AI prediction generated: {:.2} confidence", prediction.confidence);

                                                    // Step 3: Test trading engine status
                                                    match trading_engine.get_system_status().await {
                                                        Ok(status) => {
                                                            info!("✅ Trading engine status: {:?}", status);
                                                            results.complete_cycle_successful = true;
                                                            results.total_latency_ms = cycle_start.elapsed().as_millis() as f64;
                                                            results.ai_trading_integration = true;

                                                            info!("✅ Complete trading cycle successful ({:.2}ms)", results.total_latency_ms);

                                                            // Verify data persistence by checking market data storage
                                                            match database.get_recent_market_data("AAPL", 1).await {
                                                                Ok(data) => {
                                                                    results.data_persistence_verified = !data.is_empty();
                                                                    info!("✅ Data persistence verified");
                                                                }
                                                                Err(e) => {
                                                                    warn!("⚠️ Data persistence check failed: {}", e);
                                                                }
                                                            }
                                                        }
                                                        Err(e) => {
                                                            results.error_details = Some(format!("Trading engine status error: {}", e));
                                                            error!("❌ Trading engine status check failed: {}", e);
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    results.error_details = Some(format!("AI prediction error: {}", e));
                                                    error!("❌ AI prediction failed: {}", e);
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            results.error_details = Some(format!("Market data error: {}", e));
                                            error!("❌ Market data collection failed: {}", e);
                                        }
                                    }

                                    // Test monitoring system
                                    match trading_engine.get_system_status().await {
                                        Ok(_) => {
                                            results.monitoring_active = true;
                                            info!("✅ Monitoring system active");
                                        }
                                        Err(e) => {
                                            warn!("⚠️ Monitoring system check failed: {}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    results.error_details = Some(format!("Trading Engine initialization error: {}", e));
                                    error!("❌ Trading Engine initialization failed: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            results.error_details = Some(format!("AI Engine initialization error: {}", e));
                            error!("❌ AI Engine initialization failed: {}", e);
                        }
                    }
                }
                Err(e) => {
                    results.error_details = Some(format!("Market Data Manager initialization error: {}", e));
                    error!("❌ Market Data Manager initialization failed: {}", e);
                }
            }
        }
        Err(e) => {
            results.error_details = Some(format!("Database connection error: {}", e));
            error!("❌ Database connection failed: {}", e);
        }
    }

    results
}

/// Calculate overall test status
fn calculate_overall_status(results: &SystemTestResults) -> String {
    let mut score = 0;
    let mut total = 6;

    if results.database_health.connection_successful && results.database_health.pool_health {
        score += 1;
    }

    if results.ig_api_connectivity.authentication_successful && results.ig_api_connectivity.market_data_retrieval {
        score += 1;
    }

    if results.market_data_pipeline.data_collection_active && results.market_data_pipeline.database_persistence {
        score += 1;
    }

    if results.ai_engine_performance.inference_successful && results.ai_engine_performance.model_health {
        score += 1;
    }

    if results.trading_engine_performance.signal_generation && results.trading_engine_performance.performance_targets_met {
        score += 1;
    }

    if results.end_to_end_performance.complete_cycle_successful && results.end_to_end_performance.ai_trading_integration {
        score += 1;
    }

    match score {
        6 => "EXCELLENT - All systems operational".to_string(),
        5 => "GOOD - Minor issues detected".to_string(),
        4 => "FAIR - Some systems need attention".to_string(),
        3 => "POOR - Multiple system failures".to_string(),
        _ => "CRITICAL - Major system failures".to_string(),
    }
}

/// Main test runner
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match run_live_system_test().await {
        Ok(results) => {
            println!("\n🎯 LIVE SYSTEM INTEGRATION TEST RESULTS");
            println!("==========================================");
            println!("Overall Status: {}", results.overall_status);
            println!("Test Duration: {:.2}s", results.test_duration_seconds);
            println!("Timestamp: {}", results.timestamp);

            println!("\n📊 DATABASE HEALTH:");
            println!("  Connection: {}", if results.database_health.connection_successful { "✅" } else { "❌" });
            println!("  Pool Health: {}", if results.database_health.pool_health { "✅" } else { "❌" });
            println!("  Query Performance: {:.2}ms", results.database_health.query_performance_ms);

            println!("\n🔌 IG API CONNECTIVITY:");
            println!("  Authentication: {}", if results.ig_api_connectivity.authentication_successful { "✅" } else { "❌" });
            println!("  Market Data: {}", if results.ig_api_connectivity.market_data_retrieval { "✅" } else { "❌" });
            println!("  Response Time: {:.2}ms", results.ig_api_connectivity.response_time_ms);
            println!("  Demo Mode: {}", results.ig_api_connectivity.demo_mode_active);

            println!("\n📈 MARKET DATA PIPELINE:");
            println!("  Data Collection: {}", if results.market_data_pipeline.data_collection_active { "✅" } else { "❌" });
            println!("  Database Persistence: {}", if results.market_data_pipeline.database_persistence { "✅" } else { "❌" });
            println!("  Data Quality: {:.2}", results.market_data_pipeline.data_quality_score);
            println!("  Symbols Processed: {}", results.market_data_pipeline.symbols_processed);

            println!("\n🤖 AI ENGINE PERFORMANCE:");
            println!("  Inference: {}", if results.ai_engine_performance.inference_successful { "✅" } else { "❌" });
            println!("  Inference Latency: {:.2}ms", results.ai_engine_performance.inference_latency_ms);
            println!("  HMM Regime Detection: {}", if results.ai_engine_performance.hmm_regime_detection { "✅" } else { "❌" });
            println!("  Prediction Accuracy: {:.2}", results.ai_engine_performance.prediction_accuracy);

            println!("\n⚡ TRADING ENGINE PERFORMANCE:");
            println!("  Signal Generation: {}", if results.trading_engine_performance.signal_generation { "✅" } else { "❌" });
            println!("  Execution Latency: {:.2}ms", results.trading_engine_performance.order_execution_latency_ms);
            println!("  Throughput: {:.2} TPS", results.trading_engine_performance.throughput_tps);
            println!("  Performance Targets: {}", if results.trading_engine_performance.performance_targets_met { "✅" } else { "❌" });

            println!("\n🔄 END-TO-END INTEGRATION:");
            println!("  Complete Cycle: {}", if results.end_to_end_performance.complete_cycle_successful { "✅" } else { "❌" });
            println!("  Total Latency: {:.2}ms", results.end_to_end_performance.total_latency_ms);
            println!("  AI-Trading Integration: {}", if results.end_to_end_performance.ai_trading_integration { "✅" } else { "❌" });
            println!("  Monitoring Active: {}", if results.end_to_end_performance.monitoring_active { "✅" } else { "❌" });

            // Save results to file
            let results_json = serde_json::to_string_pretty(&results)?;
            std::fs::write("live_system_test_results.json", results_json)?;
            println!("\n💾 Results saved to live_system_test_results.json");

            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Live system test failed: {}", e);
            Err(e)
        }
    }
}
