// Advanced Execution Optimization and Slippage Reduction System
use crate::utils::{Result, PantherSwapError};
use crate::trading::signals::TradingSignal;
use crate::database::types::MarketTick;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, Duration, Timelike};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Order side enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Execution algorithm types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ExecutionAlgorithm {
    Market,           // Immediate market execution
    TWAP,            // Time-Weighted Average Price
    VWAP,            // Volume-Weighted Average Price
    Implementation,   // Implementation Shortfall
    POV,             // Participation of Volume
    Iceberg,         // Iceberg orders
    SmartRouting,    // Smart order routing
}

/// Execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    // Algorithm selection
    pub default_algorithm: ExecutionAlgorithm,
    pub algorithm_selection_enabled: bool,
    
    // TWAP parameters
    pub twap_duration_minutes: u32,
    pub twap_slice_count: u32,
    pub twap_randomization_factor: f64,
    
    // VWAP parameters
    pub vwap_lookback_periods: u32,
    pub vwap_participation_rate: f64,
    pub vwap_max_slice_size: f64,
    
    // Implementation Shortfall parameters
    pub is_risk_aversion: f64,
    pub is_temporary_impact_factor: f64,
    pub is_permanent_impact_factor: f64,
    
    // POV parameters
    pub pov_target_rate: f64,
    pub pov_max_rate: f64,
    pub pov_min_rate: f64,
    
    // Iceberg parameters
    pub iceberg_visible_size: f64,
    pub iceberg_randomization: f64,
    
    // Smart routing parameters
    pub enable_dark_pools: bool,
    pub max_venues: usize,
    pub venue_selection_criteria: VenueSelectionCriteria,
    
    // Slippage control
    pub max_slippage_bps: f64,
    pub slippage_monitoring_enabled: bool,
    pub adaptive_sizing_enabled: bool,
    
    // Latency optimization
    pub enable_latency_optimization: bool,
    pub max_execution_latency_ms: u64,
    pub pre_trade_analysis_enabled: bool,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            default_algorithm: ExecutionAlgorithm::SmartRouting,
            algorithm_selection_enabled: true,
            
            twap_duration_minutes: 30,
            twap_slice_count: 10,
            twap_randomization_factor: 0.1,
            
            vwap_lookback_periods: 20,
            vwap_participation_rate: 0.1, // 10% of volume
            vwap_max_slice_size: 0.05, // 5% of order
            
            is_risk_aversion: 0.5,
            is_temporary_impact_factor: 0.1,
            is_permanent_impact_factor: 0.01,
            
            pov_target_rate: 0.15, // 15% participation
            pov_max_rate: 0.25,
            pov_min_rate: 0.05,
            
            iceberg_visible_size: 0.1, // 10% visible
            iceberg_randomization: 0.2,
            
            enable_dark_pools: true,
            max_venues: 5,
            venue_selection_criteria: VenueSelectionCriteria::default(),
            
            max_slippage_bps: 10.0, // 10 basis points
            slippage_monitoring_enabled: true,
            adaptive_sizing_enabled: true,
            
            enable_latency_optimization: true,
            max_execution_latency_ms: 50,
            pre_trade_analysis_enabled: true,
        }
    }
}

/// Venue selection criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VenueSelectionCriteria {
    pub prioritize_liquidity: bool,
    pub prioritize_speed: bool,
    pub prioritize_cost: bool,
    pub min_fill_rate: f64,
    pub max_latency_ms: u64,
    pub max_spread_bps: f64,
}

impl Default for VenueSelectionCriteria {
    fn default() -> Self {
        Self {
            prioritize_liquidity: true,
            prioritize_speed: true,
            prioritize_cost: false,
            min_fill_rate: 0.8,
            max_latency_ms: 100,
            max_spread_bps: 5.0,
        }
    }
}

/// Execution slice for algorithmic trading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSlice {
    pub slice_id: Uuid,
    pub parent_order_id: Uuid,
    pub quantity: f64,
    pub target_price: Option<f64>,
    pub execution_time: DateTime<Utc>,
    pub algorithm: ExecutionAlgorithm,
    pub venue: Option<String>,
    pub priority: u8,
    pub status: SliceStatus,
}

/// Execution slice status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SliceStatus {
    Pending,
    Executing,
    Completed,
    Cancelled,
    Failed,
}

/// Market impact model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketImpactModel {
    pub temporary_impact_coefficient: f64,
    pub permanent_impact_coefficient: f64,
    pub volatility_adjustment: f64,
    pub liquidity_adjustment: f64,
    pub size_adjustment: f64,
}

impl Default for MarketImpactModel {
    fn default() -> Self {
        Self {
            temporary_impact_coefficient: 0.1,
            permanent_impact_coefficient: 0.01,
            volatility_adjustment: 1.0,
            liquidity_adjustment: 1.0,
            size_adjustment: 1.0,
        }
    }
}

/// Execution performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    pub total_orders: u64,
    pub completed_orders: u64,
    pub average_slippage_bps: f64,
    pub average_execution_time_ms: f64,
    pub fill_rate: f64,
    pub implementation_shortfall: f64,
    pub market_impact_bps: f64,
    pub venue_performance: HashMap<String, VenueMetrics>,
    pub algorithm_performance: HashMap<ExecutionAlgorithm, AlgorithmMetrics>,
    pub last_updated: DateTime<Utc>,
}

/// Venue-specific performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VenueMetrics {
    pub fill_rate: f64,
    pub average_latency_ms: f64,
    pub average_slippage_bps: f64,
    pub total_volume: f64,
    pub rejection_rate: f64,
}

/// Algorithm-specific performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmMetrics {
    pub usage_count: u64,
    pub average_slippage_bps: f64,
    pub average_completion_time_ms: f64,
    pub success_rate: f64,
    pub market_impact_bps: f64,
}

/// Pre-trade analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreTradeAnalysis {
    pub recommended_algorithm: ExecutionAlgorithm,
    pub estimated_slippage_bps: f64,
    pub estimated_market_impact_bps: f64,
    pub recommended_venues: Vec<String>,
    pub optimal_slice_count: u32,
    pub estimated_completion_time_minutes: f64,
    pub risk_score: f64,
    pub confidence: f64,
}

/// Advanced Execution Optimization Engine
pub struct ExecutionOptimizer {
    config: ExecutionConfig,
    market_impact_model: MarketImpactModel,
    execution_metrics: Arc<RwLock<ExecutionMetrics>>,
    active_slices: Arc<RwLock<HashMap<Uuid, ExecutionSlice>>>,
    market_data_cache: Arc<RwLock<VecDeque<MarketTick>>>,
    volume_profile: Arc<RwLock<HashMap<String, VolumeProfile>>>, // symbol -> profile
    venue_connectivity: Arc<RwLock<HashMap<String, VenueConnection>>>,
    slippage_tracker: Arc<RwLock<SlippageTracker>>,
}

/// Volume profile for VWAP calculations
#[derive(Debug, Clone)]
pub struct VolumeProfile {
    pub historical_volumes: VecDeque<f64>,
    pub intraday_pattern: Vec<f64>, // 24-hour volume pattern
    pub average_daily_volume: f64,
    pub last_updated: DateTime<Utc>,
}

/// Venue connection status
#[derive(Debug, Clone)]
pub struct VenueConnection {
    pub venue_name: String,
    pub is_connected: bool,
    pub latency_ms: f64,
    pub last_heartbeat: DateTime<Utc>,
    pub fill_rate: f64,
    pub available_liquidity: f64,
}

/// Slippage tracking and analysis
#[derive(Debug, Clone)]
pub struct SlippageTracker {
    pub recent_slippages: VecDeque<SlippageEvent>,
    pub algorithm_slippage: HashMap<ExecutionAlgorithm, f64>,
    pub venue_slippage: HashMap<String, f64>,
    pub time_of_day_slippage: Vec<f64>, // 24-hour slippage pattern
    pub volatility_slippage_correlation: f64,
}

/// Individual slippage event
#[derive(Debug, Clone)]
pub struct SlippageEvent {
    pub timestamp: DateTime<Utc>,
    pub order_id: Uuid,
    pub algorithm: ExecutionAlgorithm,
    pub venue: String,
    pub slippage_bps: f64,
    pub market_volatility: f64,
    pub order_size: f64,
}

impl ExecutionOptimizer {
    /// Create new execution optimizer
    pub fn new(config: ExecutionConfig) -> Self {
        Self {
            config,
            market_impact_model: MarketImpactModel::default(),
            execution_metrics: Arc::new(RwLock::new(ExecutionMetrics {
                total_orders: 0,
                completed_orders: 0,
                average_slippage_bps: 0.0,
                average_execution_time_ms: 0.0,
                fill_rate: 0.0,
                implementation_shortfall: 0.0,
                market_impact_bps: 0.0,
                venue_performance: HashMap::new(),
                algorithm_performance: HashMap::new(),
                last_updated: Utc::now(),
            })),
            active_slices: Arc::new(RwLock::new(HashMap::new())),
            market_data_cache: Arc::new(RwLock::new(VecDeque::with_capacity(1000))),
            volume_profile: Arc::new(RwLock::new(HashMap::new())),
            venue_connectivity: Arc::new(RwLock::new(HashMap::new())),
            slippage_tracker: Arc::new(RwLock::new(SlippageTracker {
                recent_slippages: VecDeque::with_capacity(1000),
                algorithm_slippage: HashMap::new(),
                venue_slippage: HashMap::new(),
                time_of_day_slippage: vec![0.0; 24],
                volatility_slippage_correlation: 0.0,
            })),
        }
    }

    /// Perform pre-trade analysis to optimize execution
    pub async fn analyze_execution_strategy(
        &self,
        order_size: f64,
        symbol: &str,
        side: OrderSide,
        urgency: f64, // 0.0 = patient, 1.0 = urgent
    ) -> Result<PreTradeAnalysis> {
        // Get current market data
        let market_data = self.market_data_cache.read().await;
        let latest_tick = market_data.back().ok_or_else(|| {
            PantherSwapError::execution("No market data available for analysis".to_string())
        })?;

        // Calculate market impact
        let estimated_market_impact = self.estimate_market_impact(order_size, &latest_tick).await?;

        // Calculate expected slippage for different algorithms
        let algorithm_slippages = self.estimate_algorithm_slippages(order_size, symbol, urgency).await?;

        // Select optimal algorithm
        let recommended_algorithm = self.select_optimal_algorithm(&algorithm_slippages, urgency).await?;

        // Get venue recommendations
        let recommended_venues = self.recommend_venues(symbol, order_size).await?;

        // Calculate optimal slice count
        let optimal_slice_count = self.calculate_optimal_slice_count(
            order_size,
            &recommended_algorithm,
            urgency,
        ).await?;

        // Estimate completion time
        let estimated_completion_time = self.estimate_completion_time(
            &recommended_algorithm,
            optimal_slice_count,
            urgency,
        ).await?;

        // Calculate risk score
        let risk_score = self.calculate_execution_risk_score(
            order_size,
            &latest_tick,
            &recommended_algorithm,
        ).await?;

        // Calculate confidence based on historical performance
        let confidence = self.calculate_analysis_confidence(&recommended_algorithm).await?;

        Ok(PreTradeAnalysis {
            recommended_algorithm,
            estimated_slippage_bps: algorithm_slippages.get(&recommended_algorithm).unwrap_or(&5.0).clone(),
            estimated_market_impact_bps: estimated_market_impact,
            recommended_venues,
            optimal_slice_count,
            estimated_completion_time_minutes: estimated_completion_time,
            risk_score,
            confidence,
        })
    }

    /// Create execution plan for a trading signal
    pub async fn create_execution_plan(
        &self,
        signal: &TradingSignal,
        order_size: f64,
        urgency: f64,
    ) -> Result<Vec<ExecutionSlice>> {
        // Perform pre-trade analysis
        let analysis = self.analyze_execution_strategy(
            order_size,
            &signal.instrument_id.to_string(),
            if signal.direction > 0 { OrderSide::Buy } else { OrderSide::Sell },
            urgency,
        ).await?;

        // Create execution slices based on recommended algorithm
        let slices = match analysis.recommended_algorithm {
            ExecutionAlgorithm::TWAP => {
                self.create_twap_slices(signal, order_size, analysis.optimal_slice_count).await?
            },
            ExecutionAlgorithm::VWAP => {
                self.create_vwap_slices(signal, order_size, &signal.instrument_id.to_string()).await?
            },
            ExecutionAlgorithm::Implementation => {
                self.create_implementation_shortfall_slices(signal, order_size, urgency).await?
            },
            ExecutionAlgorithm::POV => {
                self.create_pov_slices(signal, order_size).await?
            },
            ExecutionAlgorithm::Iceberg => {
                self.create_iceberg_slices(signal, order_size).await?
            },
            ExecutionAlgorithm::SmartRouting => {
                self.create_smart_routing_slices(signal, order_size, &analysis.recommended_venues).await?
            },
            ExecutionAlgorithm::Market => {
                self.create_market_slices(signal, order_size).await?
            },
        };

        // Store active slices
        {
            let mut active_slices = self.active_slices.write().await;
            for slice in &slices {
                active_slices.insert(slice.slice_id, slice.clone());
            }
        }

        Ok(slices)
    }

    /// Estimate market impact for given order size
    async fn estimate_market_impact(&self, order_size: f64, tick: &MarketTick) -> Result<f64> {
        let mid_price = (tick.bid_price + tick.ask_price) / 2.0;
        let spread = tick.ask_price - tick.bid_price;
        let spread_bps = (spread / mid_price) * 10000.0;

        // Simple market impact model: impact = sqrt(order_size) * volatility * liquidity_factor
        let volume = tick.volume.unwrap_or(1000.0);
        let size_factor = (order_size / volume).sqrt();
        let liquidity_factor = spread_bps / 10.0; // Normalize spread

        let temporary_impact = size_factor * liquidity_factor * self.market_impact_model.temporary_impact_coefficient;
        let permanent_impact = size_factor * self.market_impact_model.permanent_impact_coefficient;

        Ok(temporary_impact + permanent_impact)
    }

    /// Estimate slippage for different algorithms
    async fn estimate_algorithm_slippages(
        &self,
        order_size: f64,
        symbol: &str,
        urgency: f64,
    ) -> Result<HashMap<ExecutionAlgorithm, f64>> {
        let mut slippages = HashMap::new();
        let slippage_tracker = self.slippage_tracker.read().await;

        // Base slippage from historical data
        let base_slippage = 2.0; // 2 bps base

        // Algorithm-specific adjustments
        slippages.insert(ExecutionAlgorithm::Market, base_slippage * (1.0 + urgency * 2.0));
        slippages.insert(ExecutionAlgorithm::TWAP, base_slippage * 0.7);
        slippages.insert(ExecutionAlgorithm::VWAP, base_slippage * 0.6);
        slippages.insert(ExecutionAlgorithm::Implementation, base_slippage * 0.8);
        slippages.insert(ExecutionAlgorithm::POV, base_slippage * 0.9);
        slippages.insert(ExecutionAlgorithm::Iceberg, base_slippage * 0.75);
        slippages.insert(ExecutionAlgorithm::SmartRouting, base_slippage * 0.5);

        // Adjust based on historical performance
        for (algorithm, slippage) in slippages.iter_mut() {
            if let Some(historical_slippage) = slippage_tracker.algorithm_slippage.get(algorithm) {
                *slippage = (*slippage + historical_slippage) / 2.0; // Average with historical
            }
        }

        Ok(slippages)
    }

    /// Select optimal algorithm based on slippage estimates and urgency
    async fn select_optimal_algorithm(
        &self,
        algorithm_slippages: &HashMap<ExecutionAlgorithm, f64>,
        urgency: f64,
    ) -> Result<ExecutionAlgorithm> {
        if urgency > 0.8 {
            // High urgency - prioritize speed
            return Ok(ExecutionAlgorithm::Market);
        }

        // Find algorithm with lowest slippage
        let optimal_algorithm = algorithm_slippages.iter()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(algo, _)| algo.clone())
            .unwrap_or(ExecutionAlgorithm::SmartRouting);

        Ok(optimal_algorithm)
    }

    /// Recommend venues based on performance and liquidity
    async fn recommend_venues(&self, symbol: &str, order_size: f64) -> Result<Vec<String>> {
        let venue_connectivity = self.venue_connectivity.read().await;
        let execution_metrics = self.execution_metrics.read().await;

        let mut venue_scores: Vec<(String, f64)> = Vec::new();

        for (venue_name, connection) in venue_connectivity.iter() {
            if !connection.is_connected {
                continue;
            }

            let mut score = 0.0;

            // Latency score (lower is better)
            score += (100.0 - connection.latency_ms.min(100.0)) / 100.0 * 0.3;

            // Fill rate score
            score += connection.fill_rate * 0.4;

            // Liquidity score
            let liquidity_score = (connection.available_liquidity / order_size).min(1.0);
            score += liquidity_score * 0.3;

            // Historical performance
            if let Some(venue_metrics) = execution_metrics.venue_performance.get(venue_name) {
                score += (1.0 - venue_metrics.rejection_rate) * 0.2;
                score -= venue_metrics.average_slippage_bps / 100.0 * 0.1; // Penalize high slippage
            }

            venue_scores.push((venue_name.clone(), score));
        }

        // Sort by score and return top venues
        venue_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        Ok(venue_scores.into_iter()
            .take(self.config.max_venues)
            .map(|(venue, _)| venue)
            .collect())
    }

    /// Calculate optimal slice count
    async fn calculate_optimal_slice_count(
        &self,
        order_size: f64,
        algorithm: &ExecutionAlgorithm,
        urgency: f64,
    ) -> Result<u32> {
        let base_slices = match algorithm {
            ExecutionAlgorithm::Market => 1,
            ExecutionAlgorithm::TWAP => self.config.twap_slice_count,
            ExecutionAlgorithm::VWAP => (order_size / 1000.0).ceil() as u32, // 1 slice per 1000 units
            ExecutionAlgorithm::Implementation => ((1.0 - urgency) * 20.0) as u32 + 5, // 5-25 slices
            ExecutionAlgorithm::POV => 10,
            ExecutionAlgorithm::Iceberg => (order_size / (order_size * self.config.iceberg_visible_size)).ceil() as u32,
            ExecutionAlgorithm::SmartRouting => 5,
        };

        Ok(base_slices.max(1).min(50)) // Limit between 1 and 50 slices
    }

    /// Estimate completion time
    async fn estimate_completion_time(
        &self,
        algorithm: &ExecutionAlgorithm,
        slice_count: u32,
        urgency: f64,
    ) -> Result<f64> {
        let base_time_minutes = match algorithm {
            ExecutionAlgorithm::Market => 0.1, // 6 seconds
            ExecutionAlgorithm::TWAP => self.config.twap_duration_minutes as f64,
            ExecutionAlgorithm::VWAP => 15.0, // 15 minutes typical
            ExecutionAlgorithm::Implementation => 30.0 * (1.0 - urgency), // 0-30 minutes
            ExecutionAlgorithm::POV => 20.0,
            ExecutionAlgorithm::Iceberg => slice_count as f64 * 2.0, // 2 minutes per slice
            ExecutionAlgorithm::SmartRouting => 10.0,
        };

        Ok(base_time_minutes)
    }

    /// Calculate execution risk score
    async fn calculate_execution_risk_score(
        &self,
        order_size: f64,
        tick: &MarketTick,
        algorithm: &ExecutionAlgorithm,
    ) -> Result<f64> {
        let mut risk_score = 0.0;

        // Size risk
        let volume = tick.volume.unwrap_or(1000.0);
        let size_ratio = order_size / volume;
        risk_score += size_ratio * 0.3;

        // Spread risk
        let mid_price = (tick.bid_price + tick.ask_price) / 2.0;
        let spread_bps = ((tick.ask_price - tick.bid_price) / mid_price) * 10000.0;
        risk_score += (spread_bps / 50.0).min(1.0) * 0.3; // Normalize to 50 bps

        // Algorithm risk
        let algo_risk = match algorithm {
            ExecutionAlgorithm::Market => 0.8,
            ExecutionAlgorithm::TWAP => 0.3,
            ExecutionAlgorithm::VWAP => 0.2,
            ExecutionAlgorithm::Implementation => 0.4,
            ExecutionAlgorithm::POV => 0.5,
            ExecutionAlgorithm::Iceberg => 0.3,
            ExecutionAlgorithm::SmartRouting => 0.2,
        };
        risk_score += algo_risk * 0.4;

        Ok(risk_score.min(1.0))
    }

    /// Calculate analysis confidence
    async fn calculate_analysis_confidence(&self, algorithm: &ExecutionAlgorithm) -> Result<f64> {
        let execution_metrics = self.execution_metrics.read().await;

        if let Some(algo_metrics) = execution_metrics.algorithm_performance.get(algorithm) {
            // Base confidence on historical usage and success rate
            let usage_factor = (algo_metrics.usage_count as f64 / 100.0).min(1.0);
            let success_factor = algo_metrics.success_rate;

            Ok((usage_factor * 0.5 + success_factor * 0.5).max(0.5))
        } else {
            Ok(0.7) // Default confidence for new algorithms
        }
    }

    /// Create TWAP execution slices
    async fn create_twap_slices(
        &self,
        signal: &TradingSignal,
        order_size: f64,
        slice_count: u32,
    ) -> Result<Vec<ExecutionSlice>> {
        let mut slices = Vec::new();
        let slice_size = order_size / slice_count as f64;
        let duration_per_slice = Duration::minutes(self.config.twap_duration_minutes as i64 / slice_count as i64);

        for i in 0..slice_count {
            let execution_time = Utc::now() + duration_per_slice * i as i32;

            // Add randomization to avoid predictable patterns
            let randomization = (rand::random::<f64>() - 0.5) * self.config.twap_randomization_factor;
            let adjusted_size = slice_size * (1.0 + randomization);

            slices.push(ExecutionSlice {
                slice_id: Uuid::new_v4(),
                parent_order_id: signal.signal_id,
                quantity: adjusted_size,
                target_price: None, // Market price
                execution_time,
                algorithm: ExecutionAlgorithm::TWAP,
                venue: None, // Will be determined at execution
                priority: (slice_count - i) as u8, // Earlier slices have higher priority
                status: SliceStatus::Pending,
            });
        }

        Ok(slices)
    }

    /// Create VWAP execution slices
    async fn create_vwap_slices(
        &self,
        signal: &TradingSignal,
        order_size: f64,
        symbol: &str,
    ) -> Result<Vec<ExecutionSlice>> {
        let volume_profile = self.volume_profile.read().await;
        let profile = volume_profile.get(symbol);

        let mut slices = Vec::new();
        let participation_rate = self.config.vwap_participation_rate;

        // If no volume profile, fall back to equal slices
        if profile.is_none() {
            return self.create_twap_slices(signal, order_size, 10).await;
        }

        let profile = profile.unwrap();
        let current_hour = Utc::now().hour() as usize;

        // Calculate slices based on expected volume pattern
        let mut remaining_size = order_size;
        let mut slice_time = Utc::now();

        for hour_offset in 0..6 { // Next 6 hours
            let hour_index = (current_hour + hour_offset) % 24;
            let expected_volume = profile.intraday_pattern[hour_index] * profile.average_daily_volume / 24.0;
            let max_slice_size = expected_volume * participation_rate;

            if remaining_size <= 0.0 {
                break;
            }

            let slice_size = remaining_size.min(max_slice_size).min(order_size * self.config.vwap_max_slice_size);

            if slice_size > 0.0 {
                slices.push(ExecutionSlice {
                    slice_id: Uuid::new_v4(),
                    parent_order_id: signal.signal_id,
                    quantity: slice_size,
                    target_price: None,
                    execution_time: slice_time,
                    algorithm: ExecutionAlgorithm::VWAP,
                    venue: None,
                    priority: (6 - hour_offset) as u8,
                    status: SliceStatus::Pending,
                });

                remaining_size -= slice_size;
            }

            slice_time += Duration::hours(1);
        }

        // If there's remaining size, add it to the last slice or create a new one
        if remaining_size > 0.0 {
            if let Some(last_slice) = slices.last_mut() {
                last_slice.quantity += remaining_size;
            } else {
                slices.push(ExecutionSlice {
                    slice_id: Uuid::new_v4(),
                    parent_order_id: signal.signal_id,
                    quantity: remaining_size,
                    target_price: None,
                    execution_time: Utc::now(),
                    algorithm: ExecutionAlgorithm::VWAP,
                    venue: None,
                    priority: 1,
                    status: SliceStatus::Pending,
                });
            }
        }

        Ok(slices)
    }

    /// Create Implementation Shortfall slices
    async fn create_implementation_shortfall_slices(
        &self,
        signal: &TradingSignal,
        order_size: f64,
        urgency: f64,
    ) -> Result<Vec<ExecutionSlice>> {
        let mut slices = Vec::new();

        // Implementation Shortfall balances market impact vs. timing risk
        let immediate_portion = urgency * 0.5; // Execute 0-50% immediately based on urgency
        let remaining_portion = 1.0 - immediate_portion;

        // Immediate execution slice
        if immediate_portion > 0.0 {
            slices.push(ExecutionSlice {
                slice_id: Uuid::new_v4(),
                parent_order_id: signal.signal_id,
                quantity: order_size * immediate_portion,
                target_price: None,
                execution_time: Utc::now(),
                algorithm: ExecutionAlgorithm::Implementation,
                venue: None,
                priority: 10,
                status: SliceStatus::Pending,
            });
        }

        // Spread remaining over time
        if remaining_portion > 0.0 {
            let remaining_slices = ((1.0 - urgency) * 10.0) as u32 + 2; // 2-12 slices
            let slice_size = order_size * remaining_portion / remaining_slices as f64;
            let time_interval = Duration::minutes(30 / remaining_slices as i64); // Spread over 30 minutes

            for i in 0..remaining_slices {
                slices.push(ExecutionSlice {
                    slice_id: Uuid::new_v4(),
                    parent_order_id: signal.signal_id,
                    quantity: slice_size,
                    target_price: None,
                    execution_time: Utc::now() + time_interval * (i + 1) as i32,
                    algorithm: ExecutionAlgorithm::Implementation,
                    venue: None,
                    priority: (remaining_slices - i) as u8,
                    status: SliceStatus::Pending,
                });
            }
        }

        Ok(slices)
    }

    /// Create POV (Participation of Volume) slices
    async fn create_pov_slices(
        &self,
        signal: &TradingSignal,
        order_size: f64,
    ) -> Result<Vec<ExecutionSlice>> {
        // POV slices are created dynamically based on real-time volume
        // For now, create initial slices that will be adjusted during execution
        let mut slices = Vec::new();
        let estimated_slices = 10;
        let slice_size = order_size / estimated_slices as f64;

        for i in 0..estimated_slices {
            slices.push(ExecutionSlice {
                slice_id: Uuid::new_v4(),
                parent_order_id: signal.signal_id,
                quantity: slice_size,
                target_price: None,
                execution_time: Utc::now() + Duration::minutes(i * 2), // Every 2 minutes
                algorithm: ExecutionAlgorithm::POV,
                venue: None,
                priority: (estimated_slices - i) as u8,
                status: SliceStatus::Pending,
            });
        }

        Ok(slices)
    }

    /// Create Iceberg slices
    async fn create_iceberg_slices(
        &self,
        signal: &TradingSignal,
        order_size: f64,
    ) -> Result<Vec<ExecutionSlice>> {
        let mut slices = Vec::new();
        let visible_size = order_size * self.config.iceberg_visible_size;
        let slice_count = (order_size / visible_size).ceil() as u32;

        for i in 0..slice_count {
            let is_last_slice = i == slice_count - 1;
            let slice_size = if is_last_slice {
                order_size - (visible_size * i as f64) // Remaining amount
            } else {
                visible_size * (1.0 + (rand::random::<f64>() - 0.5) * self.config.iceberg_randomization)
            };

            slices.push(ExecutionSlice {
                slice_id: Uuid::new_v4(),
                parent_order_id: signal.signal_id,
                quantity: slice_size,
                target_price: None,
                execution_time: Utc::now() + Duration::seconds(i as i64 * 30), // 30 seconds apart
                algorithm: ExecutionAlgorithm::Iceberg,
                venue: None,
                priority: (slice_count - i) as u8,
                status: SliceStatus::Pending,
            });
        }

        Ok(slices)
    }

    /// Create Smart Routing slices
    async fn create_smart_routing_slices(
        &self,
        signal: &TradingSignal,
        order_size: f64,
        recommended_venues: &[String],
    ) -> Result<Vec<ExecutionSlice>> {
        let mut slices = Vec::new();

        if recommended_venues.is_empty() {
            // Fallback to single market slice
            return self.create_market_slices(signal, order_size).await;
        }

        // Distribute order across multiple venues
        let slice_size = order_size / recommended_venues.len() as f64;

        for (i, venue) in recommended_venues.iter().enumerate() {
            slices.push(ExecutionSlice {
                slice_id: Uuid::new_v4(),
                parent_order_id: signal.signal_id,
                quantity: slice_size,
                target_price: None,
                execution_time: Utc::now() + Duration::milliseconds(i as i64 * 100), // Stagger by 100ms
                algorithm: ExecutionAlgorithm::SmartRouting,
                venue: Some(venue.clone()),
                priority: (recommended_venues.len() - i) as u8,
                status: SliceStatus::Pending,
            });
        }

        Ok(slices)
    }

    /// Create Market execution slices
    async fn create_market_slices(
        &self,
        signal: &TradingSignal,
        order_size: f64,
    ) -> Result<Vec<ExecutionSlice>> {
        // Single immediate execution
        Ok(vec![ExecutionSlice {
            slice_id: Uuid::new_v4(),
            parent_order_id: signal.signal_id,
            quantity: order_size,
            target_price: None,
            execution_time: Utc::now(),
            algorithm: ExecutionAlgorithm::Market,
            venue: None,
            priority: 10,
            status: SliceStatus::Pending,
        }])
    }

    /// Update market data for execution optimization
    pub async fn update_market_data(&self, tick: MarketTick) -> Result<()> {
        let mut market_data = self.market_data_cache.write().await;

        if market_data.len() >= 1000 {
            market_data.pop_front();
        }
        market_data.push_back(tick);

        Ok(())
    }

    /// Get current execution metrics
    pub async fn get_execution_metrics(&self) -> ExecutionMetrics {
        self.execution_metrics.read().await.clone()
    }

    /// Record execution result for performance tracking
    pub async fn record_execution_result(
        &self,
        slice_id: Uuid,
        executed_price: f64,
        executed_quantity: f64,
        execution_time_ms: u64,
        venue: String,
    ) -> Result<()> {
        // Update slice status
        {
            let mut active_slices = self.active_slices.write().await;
            if let Some(slice) = active_slices.get_mut(&slice_id) {
                slice.status = SliceStatus::Completed;
            }
        }

        // Calculate and record slippage
        let market_data = self.market_data_cache.read().await;
        if let Some(latest_tick) = market_data.back() {
            let mid_price = (latest_tick.bid_price + latest_tick.ask_price) / 2.0;
            let slippage_bps = ((executed_price - mid_price).abs() / mid_price) * 10000.0;

            // Record slippage event
            let mut slippage_tracker = self.slippage_tracker.write().await;
            slippage_tracker.recent_slippages.push_back(SlippageEvent {
                timestamp: Utc::now(),
                order_id: slice_id,
                algorithm: ExecutionAlgorithm::Market, // Would need to get from slice
                venue: venue.clone(),
                slippage_bps,
                market_volatility: 0.0, // Would calculate from recent price moves
                order_size: executed_quantity,
            });

            // Update venue slippage tracking
            let venue_avg_slippage = slippage_tracker.venue_slippage.entry(venue.clone()).or_insert(0.0);
            *venue_avg_slippage = (*venue_avg_slippage + slippage_bps) / 2.0;
        }

        // Update execution metrics
        let mut metrics = self.execution_metrics.write().await;
        metrics.total_orders += 1;
        metrics.completed_orders += 1;
        metrics.last_updated = Utc::now();

        Ok(())
    }
}

// Add rand dependency for randomization
use rand;
