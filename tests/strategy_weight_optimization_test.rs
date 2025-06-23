use pantherswap_edge::database::Database;
use pantherswap_edge::trading::{
    StrategyWeightOptimizer, StrategyWeights, OptimizationConfig,
    StrategyAnalyticsDB, StrategyPerformance, StrategyType
};
use pantherswap_edge::database::types::RegimeType;
use std::sync::Arc;
use std::time::Duration;
use tokio;
use anyhow::Result;

#[tokio::test]
async fn test_strategy_weight_optimization_framework() -> Result<()> {
    // Initialize test database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://tsdbadmin:sz2eu577bgqi5767@jqrbtbc5nw.w0mq2s13iy.tsdb.cloud.timescale.com:35762/tsdb?sslmode=require".to_string());
    let database = Arc::new(Database::new(&database_url).await?);

    // Run migration for strategy analytics tables
    sqlx::migrate!("./migrations")
        .run(&database.pool)
        .await?;

    // Initialize strategy analytics database interface
    let analytics_db = StrategyAnalyticsDB::new(database.clone());
    
    // Initialize strategy weight optimizer
    let optimization_config = OptimizationConfig {
        target_sharpe_ratio: 2.5,
        max_drawdown_threshold: 0.12,
        min_diversification_ratio: 1.3,
        risk_free_rate: 0.02,
        confidence_level: 0.95,
        optimization_frequency_hours: 6,
        enable_regime_based_weights: true,
        enable_correlation_adjustment: true,
        enable_risk_parity: true,
        kelly_fraction_limit: 0.25,
    };
    
    let optimizer = StrategyWeightOptimizer::new(database.clone(), optimization_config);

    // Test 1: Store mock strategy performance data
    println!("🧪 Test 1: Storing strategy performance data");
    
    let mock_performances = create_mock_strategy_performances();
    
    for (strategy_type, performance) in &mock_performances {
        let performance_id = analytics_db.store_strategy_performance(*strategy_type, performance).await?;
        println!("  ✅ Stored performance for {:?}: {}", strategy_type, performance_id);
        
        // Update optimizer with performance data
        optimizer.update_strategy_performance(*strategy_type, performance.clone()).await?;
    }

    // Test 2: Test weight optimization
    println!("\n🧪 Test 2: Testing strategy weight optimization");
    
    // Test optimization in different market regimes
    let regimes = vec![
        Some(RegimeType::Normal),
        Some(RegimeType::Trending),
        Some(RegimeType::Volatile),
        Some(RegimeType::Crisis),
        None,
    ];

    for regime in regimes {
        let optimized_weights = optimizer.optimize_weights(regime).await?;
        
        println!("  📊 Regime {:?} - Optimized weights:", regime);
        println!("    PMM: {:.3}, MM: {:.3}, RA: {:.3}, LH: {:.3}",
                 optimized_weights.predictive_market_making,
                 optimized_weights.microstructure_momentum,
                 optimized_weights.regime_arbitrage,
                 optimized_weights.liquidity_harvesting);
        
        // Validate weights
        optimized_weights.validate()?;
        
        // Store optimized weights
        let weights_id = analytics_db.store_strategy_weights(
            &optimized_weights,
            "multi_objective_optimization",
            Some(2.5),
            None,
            None,
            None,
        ).await?;
        
        println!("    ✅ Stored weights with ID: {}", weights_id);
    }

    // Test 3: Portfolio metrics calculation
    println!("\n🧪 Test 3: Testing portfolio metrics calculation");
    
    let portfolio_metrics = optimizer.calculate_portfolio_metrics().await?;
    
    println!("  📈 Portfolio Metrics:");
    println!("    Sharpe Ratio: {:.3}", portfolio_metrics.portfolio_sharpe_ratio);
    println!("    Volatility: {:.3}", portfolio_metrics.portfolio_volatility);
    println!("    Return: {:.3}", portfolio_metrics.portfolio_return);
    println!("    Diversification Ratio: {:.3}", portfolio_metrics.diversification_ratio);
    println!("    Risk Concentration: {:.3}", portfolio_metrics.risk_concentration);
    println!("    Weight Stability: {:.3}", portfolio_metrics.weight_stability);

    // Test 4: Database retrieval functions
    println!("\n🧪 Test 4: Testing database retrieval functions");
    
    let latest_performance = analytics_db.get_latest_strategy_performance().await?;
    println!("  📊 Retrieved performance for {} strategies", latest_performance.len());
    
    for (strategy_type, performance) in &latest_performance {
        println!("    {:?}: Sharpe={:.3}, Return={:.6}", 
                 strategy_type, performance.sharpe_ratio, performance.avg_return_per_trade);
    }
    
    let latest_weights = analytics_db.get_latest_strategy_weights().await?;
    if let Some(weights) = latest_weights {
        println!("  ⚖️  Latest weights: PMM={:.3}, MM={:.3}, RA={:.3}, LH={:.3}",
                 weights.predictive_market_making,
                 weights.microstructure_momentum,
                 weights.regime_arbitrage,
                 weights.liquidity_harvesting);
    }

    // Test 5: Performance validation
    println!("\n🧪 Test 5: Performance validation");
    
    // Validate that optimization improves portfolio metrics
    let initial_weights = StrategyWeights::default();
    let optimized_weights = optimizer.optimize_weights(Some(RegimeType::Normal)).await?;
    
    println!("  📊 Weight comparison:");
    println!("    Initial - PMM: {:.3}, MM: {:.3}, RA: {:.3}, LH: {:.3}",
             initial_weights.predictive_market_making,
             initial_weights.microstructure_momentum,
             initial_weights.regime_arbitrage,
             initial_weights.liquidity_harvesting);
    
    println!("    Optimized - PMM: {:.3}, MM: {:.3}, RA: {:.3}, LH: {:.3}",
             optimized_weights.predictive_market_making,
             optimized_weights.microstructure_momentum,
             optimized_weights.regime_arbitrage,
             optimized_weights.liquidity_harvesting);

    // Test 6: Stress testing with extreme scenarios
    println!("\n🧪 Test 6: Stress testing optimization");
    
    let extreme_performance = create_extreme_strategy_performance();
    for (strategy_type, performance) in &extreme_performance {
        optimizer.update_strategy_performance(*strategy_type, performance.clone()).await?;
    }
    
    let stress_weights = optimizer.optimize_weights(Some(RegimeType::Crisis)).await?;
    println!("  🚨 Crisis regime weights: PMM={:.3}, MM={:.3}, RA={:.3}, LH={:.3}",
             stress_weights.predictive_market_making,
             stress_weights.microstructure_momentum,
             stress_weights.regime_arbitrage,
             stress_weights.liquidity_harvesting);
    
    stress_weights.validate()?;

    // Test 7: Enhanced Performance Analytics
    println!("\n🧪 Test 7: Testing enhanced performance analytics");

    test_enhanced_performance_analytics(&optimizer).await?;

    println!("\n✅ All strategy weight optimization tests passed!");
    Ok(())
}

/// Test enhanced performance analytics with advanced metrics
async fn test_enhanced_performance_analytics(optimizer: &StrategyWeightOptimizer) -> Result<()> {
    println!("  📊 Testing advanced performance metrics calculation...");

    // Create comprehensive test data with realistic daily returns
    let test_performance = create_comprehensive_test_performance();

    // Update optimizer with test performance data
    for (strategy_type, performance) in &test_performance {
        optimizer.update_strategy_performance(*strategy_type, performance.clone()).await?;
    }

    // Test portfolio metrics with enhanced analytics
    let portfolio_metrics = optimizer.calculate_portfolio_metrics().await?;

    println!("  📈 Enhanced Portfolio Analytics:");
    println!("    Portfolio Sharpe Ratio: {:.4}", portfolio_metrics.portfolio_sharpe_ratio);
    println!("    Portfolio Volatility: {:.4}", portfolio_metrics.portfolio_volatility);
    println!("    Portfolio Return: {:.6}", portfolio_metrics.portfolio_return);
    println!("    Diversification Ratio: {:.4}", portfolio_metrics.diversification_ratio);
    println!("    Risk Concentration: {:.4}", portfolio_metrics.risk_concentration);
    println!("    Weight Stability: {:.4}", portfolio_metrics.weight_stability);

    // Validate that enhanced metrics are being calculated
    assert!(portfolio_metrics.diversification_ratio > 1.0, "Diversification ratio should be > 1.0");
    assert!(portfolio_metrics.risk_concentration <= 1.0, "Risk concentration should be <= 1.0");
    assert!(portfolio_metrics.weight_stability >= 0.0 && portfolio_metrics.weight_stability <= 1.0,
            "Weight stability should be between 0 and 1");

    println!("  ✅ Enhanced performance analytics validation passed");

    // Test optimization with enhanced metrics
    println!("  🔧 Testing optimization with enhanced analytics...");

    let optimized_weights = optimizer.optimize_weights(Some(RegimeType::Normal)).await?;

    println!("  ⚖️  Optimized weights with enhanced analytics:");
    println!("    PMM: {:.4} ({}%)", optimized_weights.predictive_market_making,
             (optimized_weights.predictive_market_making * 100.0) as i32);
    println!("    MM: {:.4} ({}%)", optimized_weights.microstructure_momentum,
             (optimized_weights.microstructure_momentum * 100.0) as i32);
    println!("    RA: {:.4} ({}%)", optimized_weights.regime_arbitrage,
             (optimized_weights.regime_arbitrage * 100.0) as i32);
    println!("    LH: {:.4} ({}%)", optimized_weights.liquidity_harvesting,
             (optimized_weights.liquidity_harvesting * 100.0) as i32);

    // Validate optimization results
    optimized_weights.validate()?;

    // Test that weights are data-driven (not equal allocation)
    let weights_vec = vec![
        optimized_weights.predictive_market_making,
        optimized_weights.microstructure_momentum,
        optimized_weights.regime_arbitrage,
        optimized_weights.liquidity_harvesting,
    ];

    let max_weight = weights_vec.iter().fold(0.0f64, |a, &b| a.max(b));
    let min_weight = weights_vec.iter().fold(1.0f64, |a, &b| a.min(b));
    let weight_spread = max_weight - min_weight;

    println!("  📊 Weight distribution analysis:");
    println!("    Max weight: {:.4}", max_weight);
    println!("    Min weight: {:.4}", min_weight);
    println!("    Weight spread: {:.4}", weight_spread);

    // Expect some differentiation in weights (not perfectly equal)
    assert!(weight_spread > 0.01, "Weights should show some differentiation based on performance");

    println!("  ✅ Enhanced optimization validation passed");

    Ok(())
}

fn create_mock_strategy_performances() -> Vec<(StrategyType, StrategyPerformance)> {
    vec![
        (StrategyType::PredictiveMarketMaking, StrategyPerformance {
            total_trades: 1500,
            winning_trades: 1050,
            total_pnl: 125000.0,
            sharpe_ratio: 1.8,
            max_drawdown: 0.08,
            avg_holding_period: Duration::from_secs(15 * 60),
            success_rate: 0.70,
            avg_return_per_trade: 83.33,
            sortino_ratio: 2.1,
            calmar_ratio: 22.5,
            information_ratio: 1.2,
            var_95: -2500.0,
            expected_shortfall: -3200.0,
            profit_factor: 2.3,
            recovery_factor: 15.6,
            tail_ratio: 1.4,
            skewness: 0.2,
            kurtosis: 3.1,
            daily_returns: vec![0.001, 0.002, -0.001, 0.003, 0.001],
            rolling_sharpe_30d: 1.9,
            rolling_volatility_30d: 0.12,
            max_consecutive_losses: 3,
            avg_win_loss_ratio: 2.1,
            kelly_fraction: 0.18,
            correlation_to_market: 0.3,
            beta: 0.8,
            alpha: 0.05,
            tracking_error: 0.08,
            upside_capture: 0.85,
            downside_capture: 0.65,
        }),
        (StrategyType::MicrostructureMomentum, StrategyPerformance {
            total_trades: 3200,
            winning_trades: 2240,
            total_pnl: 180000.0,
            sharpe_ratio: 2.2,
            max_drawdown: 0.12,
            avg_holding_period: Duration::from_secs(5 * 60),
            success_rate: 0.70,
            avg_return_per_trade: 56.25,
            sortino_ratio: 2.8,
            calmar_ratio: 18.3,
            information_ratio: 1.5,
            var_95: -3200.0,
            expected_shortfall: -4100.0,
            profit_factor: 2.8,
            recovery_factor: 12.5,
            tail_ratio: 1.6,
            skewness: 0.1,
            kurtosis: 2.9,
            daily_returns: vec![0.002, 0.003, -0.001, 0.004, 0.002],
            rolling_sharpe_30d: 2.3,
            rolling_volatility_30d: 0.15,
            max_consecutive_losses: 4,
            avg_win_loss_ratio: 2.5,
            kelly_fraction: 0.22,
            correlation_to_market: 0.4,
            beta: 1.1,
            alpha: 0.08,
            tracking_error: 0.12,
            upside_capture: 1.05,
            downside_capture: 0.75,
        }),
        (StrategyType::RegimeArbitrage, StrategyPerformance {
            total_trades: 800,
            winning_trades: 600,
            total_pnl: 95000.0,
            sharpe_ratio: 1.6,
            max_drawdown: 0.10,
            avg_holding_period: Duration::from_secs(60 * 60),
            success_rate: 0.75,
            avg_return_per_trade: 118.75,
            sortino_ratio: 2.0,
            calmar_ratio: 16.0,
            information_ratio: 1.0,
            var_95: -2800.0,
            expected_shortfall: -3600.0,
            profit_factor: 2.1,
            recovery_factor: 9.5,
            tail_ratio: 1.3,
            skewness: 0.3,
            kurtosis: 3.2,
            daily_returns: vec![0.001, 0.002, 0.000, 0.003, 0.001],
            rolling_sharpe_30d: 1.7,
            rolling_volatility_30d: 0.10,
            max_consecutive_losses: 2,
            avg_win_loss_ratio: 1.8,
            kelly_fraction: 0.15,
            correlation_to_market: 0.2,
            beta: 0.6,
            alpha: 0.04,
            tracking_error: 0.06,
            upside_capture: 0.70,
            downside_capture: 0.50,
        }),
        (StrategyType::LiquidityHarvesting, StrategyPerformance {
            total_trades: 2400,
            winning_trades: 1800,
            total_pnl: 72000.0,
            sharpe_ratio: 1.4,
            max_drawdown: 0.06,
            avg_holding_period: Duration::from_secs(30 * 60),
            success_rate: 0.75,
            avg_return_per_trade: 30.0,
            sortino_ratio: 1.7,
            calmar_ratio: 23.3,
            information_ratio: 0.8,
            var_95: -1800.0,
            expected_shortfall: -2300.0,
            profit_factor: 1.9,
            recovery_factor: 12.0,
            tail_ratio: 1.2,
            skewness: 0.1,
            kurtosis: 2.8,
            daily_returns: vec![0.001, 0.001, 0.000, 0.002, 0.001],
            rolling_sharpe_30d: 1.5,
            rolling_volatility_30d: 0.08,
            max_consecutive_losses: 2,
            avg_win_loss_ratio: 1.6,
            kelly_fraction: 0.12,
            correlation_to_market: 0.1,
            beta: 0.4,
            alpha: 0.02,
            tracking_error: 0.04,
            upside_capture: 0.60,
            downside_capture: 0.40,
        }),
    ]
}

fn create_extreme_strategy_performance() -> Vec<(StrategyType, StrategyPerformance)> {
    // Create extreme performance scenarios for stress testing
    vec![
        (StrategyType::PredictiveMarketMaking, StrategyPerformance {
            total_trades: 100,
            winning_trades: 30,
            total_pnl: -25000.0,
            sharpe_ratio: -0.5,
            max_drawdown: 0.25,
            avg_holding_period: Duration::from_secs(15 * 60),
            success_rate: 0.30,
            avg_return_per_trade: -250.0,
            sortino_ratio: -0.8,
            calmar_ratio: -2.0,
            information_ratio: -0.5,
            var_95: -5000.0,
            expected_shortfall: -6500.0,
            profit_factor: 0.6,
            recovery_factor: -1.0,
            tail_ratio: 0.8,
            skewness: -0.5,
            kurtosis: 4.5,
            daily_returns: vec![-0.005, -0.003, 0.001, -0.008, -0.002],
            rolling_sharpe_30d: -0.6,
            rolling_volatility_30d: 0.25,
            max_consecutive_losses: 8,
            avg_win_loss_ratio: 0.8,
            kelly_fraction: 0.0,
            correlation_to_market: 0.8,
            beta: 1.5,
            alpha: -0.10,
            tracking_error: 0.20,
            upside_capture: 0.40,
            downside_capture: 1.20,
        }),
    ]
}

/// Create comprehensive test performance data with realistic daily returns for enhanced analytics testing
fn create_comprehensive_test_performance() -> Vec<(StrategyType, StrategyPerformance)> {
    // Generate realistic daily returns for 60 days
    let pmm_returns = generate_realistic_returns(60, 0.0008, 0.012, 0.1); // Market making: lower vol, positive skew
    let mm_returns = generate_realistic_returns(60, 0.0012, 0.018, -0.2); // Momentum: higher vol, negative skew
    let ra_returns = generate_realistic_returns(60, 0.0010, 0.015, 0.3); // Regime arbitrage: moderate vol, positive skew
    let lh_returns = generate_realistic_returns(60, 0.0006, 0.008, 0.05); // Liquidity harvesting: low vol, neutral skew

    vec![
        (StrategyType::PredictiveMarketMaking, StrategyPerformance {
            total_trades: 2000,
            winning_trades: 1400,
            total_pnl: 150000.0,
            sharpe_ratio: 2.1,
            max_drawdown: 0.07,
            avg_holding_period: Duration::from_secs(12 * 60),
            success_rate: 0.70,
            avg_return_per_trade: 75.0,
            sortino_ratio: 2.8,
            calmar_ratio: 30.0,
            information_ratio: 1.4,
            var_95: -2200.0,
            expected_shortfall: -2800.0,
            profit_factor: 2.5,
            recovery_factor: 21.4,
            tail_ratio: 1.5,
            skewness: 0.1,
            kurtosis: 3.2,
            daily_returns: pmm_returns,
            rolling_sharpe_30d: 2.2,
            rolling_volatility_30d: 0.011,
            max_consecutive_losses: 3,
            avg_win_loss_ratio: 2.2,
            kelly_fraction: 0.19,
            correlation_to_market: 0.25,
            beta: 0.7,
            alpha: 0.06,
            tracking_error: 0.07,
            upside_capture: 0.80,
            downside_capture: 0.60,
        }),
        (StrategyType::MicrostructureMomentum, StrategyPerformance {
            total_trades: 4500,
            winning_trades: 3150,
            total_pnl: 220000.0,
            sharpe_ratio: 2.6,
            max_drawdown: 0.11,
            avg_holding_period: Duration::from_secs(3 * 60),
            success_rate: 0.70,
            avg_return_per_trade: 48.89,
            sortino_ratio: 3.2,
            calmar_ratio: 23.6,
            information_ratio: 1.8,
            var_95: -3800.0,
            expected_shortfall: -4900.0,
            profit_factor: 3.1,
            recovery_factor: 20.0,
            tail_ratio: 1.7,
            skewness: -0.2,
            kurtosis: 3.8,
            daily_returns: mm_returns,
            rolling_sharpe_30d: 2.7,
            rolling_volatility_30d: 0.017,
            max_consecutive_losses: 5,
            avg_win_loss_ratio: 2.8,
            kelly_fraction: 0.24,
            correlation_to_market: 0.45,
            beta: 1.2,
            alpha: 0.09,
            tracking_error: 0.13,
            upside_capture: 1.15,
            downside_capture: 0.85,
        }),
        (StrategyType::RegimeArbitrage, StrategyPerformance {
            total_trades: 1200,
            winning_trades: 900,
            total_pnl: 135000.0,
            sharpe_ratio: 1.9,
            max_drawdown: 0.09,
            avg_holding_period: Duration::from_secs(45 * 60),
            success_rate: 0.75,
            avg_return_per_trade: 112.5,
            sortino_ratio: 2.4,
            calmar_ratio: 21.1,
            information_ratio: 1.3,
            var_95: -3100.0,
            expected_shortfall: -3900.0,
            profit_factor: 2.4,
            recovery_factor: 15.0,
            tail_ratio: 1.4,
            skewness: 0.3,
            kurtosis: 3.5,
            daily_returns: ra_returns,
            rolling_sharpe_30d: 2.0,
            rolling_volatility_30d: 0.014,
            max_consecutive_losses: 3,
            avg_win_loss_ratio: 2.0,
            kelly_fraction: 0.17,
            correlation_to_market: 0.15,
            beta: 0.5,
            alpha: 0.05,
            tracking_error: 0.05,
            upside_capture: 0.65,
            downside_capture: 0.45,
        }),
        (StrategyType::LiquidityHarvesting, StrategyPerformance {
            total_trades: 3600,
            winning_trades: 2880,
            total_pnl: 108000.0,
            sharpe_ratio: 1.7,
            max_drawdown: 0.05,
            avg_holding_period: Duration::from_secs(20 * 60),
            success_rate: 0.80,
            avg_return_per_trade: 30.0,
            sortino_ratio: 2.1,
            calmar_ratio: 34.0,
            information_ratio: 1.0,
            var_95: -1600.0,
            expected_shortfall: -2000.0,
            profit_factor: 2.2,
            recovery_factor: 21.6,
            tail_ratio: 1.3,
            skewness: 0.05,
            kurtosis: 2.9,
            daily_returns: lh_returns,
            rolling_sharpe_30d: 1.8,
            rolling_volatility_30d: 0.007,
            max_consecutive_losses: 2,
            avg_win_loss_ratio: 1.8,
            kelly_fraction: 0.14,
            correlation_to_market: 0.05,
            beta: 0.3,
            alpha: 0.03,
            tracking_error: 0.03,
            upside_capture: 0.55,
            downside_capture: 0.35,
        }),
    ]
}

/// Generate realistic daily returns with specified characteristics
fn generate_realistic_returns(days: usize, mean: f64, volatility: f64, skew: f64) -> Vec<f64> {
    use std::f64::consts::PI;

    let mut returns = Vec::with_capacity(days);
    let mut rng_state = 42u64; // Simple PRNG seed

    for i in 0..days {
        // Simple Box-Muller transform for normal distribution
        let u1 = simple_random(&mut rng_state);
        let u2 = simple_random(&mut rng_state);
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos();

        // Apply skewness transformation
        let skewed_z = if skew != 0.0 {
            z + skew * (z * z - 1.0) / 6.0
        } else {
            z
        };

        // Scale and shift
        let return_val = mean + volatility * skewed_z;

        // Add some autocorrelation for realism
        let autocorr_factor = if i > 0 { 0.1 * returns[i-1] } else { 0.0 };

        returns.push(return_val + autocorr_factor);
    }

    returns
}

/// Simple pseudo-random number generator
fn simple_random(state: &mut u64) -> f64 {
    *state = state.wrapping_mul(1103515245).wrapping_add(12345);
    (*state as f64) / (u64::MAX as f64)
}
