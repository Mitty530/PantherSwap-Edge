pub mod engine;
pub mod strategies;
pub mod signals;
pub mod execution;
pub mod risk;
pub mod portfolio;
pub mod order_manager;
pub mod market_interface;
pub mod adaptive_batching;
pub mod lock_free_structures;
pub mod performance_validation;
pub mod strategy_optimization;
pub mod strategy_analytics_db;
pub mod risk_reward_enhancement;
pub mod execution_optimization;
pub mod performance_monitoring;
pub mod alpaca_execution;
pub mod alpaca_trading_engine;
pub mod alpaca_error_handling;

pub use engine::{TradingEngine, TradingEngineConfig};
pub use execution::{ExecutionEngine, ExecutionConfig, Order, OrderStatus, Fill, MarketData};
pub use risk::{RiskManager, RiskManagerConfig};
pub use portfolio::{PortfolioManager, PortfolioConfig, PortfolioState, PerformanceMetrics, PortfolioSummary};
pub use order_manager::{OrderManager, OrderManagerConfig, OrderBook, OrderStatistics};
pub use market_interface::{MarketInterface, MarketInterfaceConfig, MarketDataSource, RoutingDestination};
pub use signals::{SignalGenerator, TradingSignal, SignalWeights};
pub use strategies::{TradingStrategy, StrategyPerformance, create_strategy};
pub use crate::database::types::SignalType;
pub use crate::trading::signals::StrategyType;
pub use adaptive_batching::{AdaptiveBatchProcessor, AdaptiveBatchingConfig, BatchMetrics};
pub use lock_free_structures::{LockFreeOrderQueue, LockFreeMemoryPool};
pub use strategy_optimization::{
    StrategyWeightOptimizer, StrategyWeights, StrategyWeightConfig,
    StrategyAnalytics, OptimizationConfig, PortfolioOptimizationMetrics
};
pub use strategy_analytics_db::StrategyAnalyticsDB;
pub use risk_reward_enhancement::{
    RiskRewardEngine, RiskRewardConfig, PositionSizingResult,
    RiskManagementDecision, PortfolioRiskMetrics, PerformanceFeedback,
    VolatilityRegime
};
pub use execution_optimization::{
    ExecutionOptimizer, ExecutionConfig as ExecutionOptimizationConfig, ExecutionAlgorithm, ExecutionSlice,
    PreTradeAnalysis, ExecutionMetrics, MarketImpactModel, OrderSide
};
pub use performance_monitoring::{
    PerformanceMonitor, PerformanceMonitoringConfig, RealTimePnL, StrategyAttribution,
    PerformanceBenchmark, PerformanceAlert, OptimizationRecommendation, ComprehensiveMetrics,
    TradeRecord, PositionInfo, AlertType, AlertSeverity, OptimizationType
};
pub use alpaca_execution::{AlpacaExecutionEngine, AlpacaOrderInfo, ExecutionStats};
pub use alpaca_trading_engine::{AlpacaTradingEngine, AlpacaTradingConfig, TradingPerformanceMetrics};
pub use alpaca_error_handling::{
    AlpacaErrorHandler, RetryConfig, AlpacaError, AlpacaErrorType, CircuitBreakerState
};
