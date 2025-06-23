use crate::database::{Database, types::MarketTick};
use crate::trading::execution::MarketData;
use crate::trading::order_manager::{OrderBook, OrderBookEntry};
use crate::utils::Result;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::{HashMap, VecDeque};
use tokio::sync::{RwLock, mpsc};
use std::sync::Arc;
use tracing::{info, warn, error, debug};
use reqwest::Client;

// Market Interface Configuration
#[derive(Debug, Clone)]
pub struct MarketInterfaceConfig {
    pub enable_real_time_data: bool,
    pub data_update_interval_ms: u64,
    pub max_data_age_seconds: u64,
    pub enable_order_routing: bool,
    pub enable_smart_routing: bool,
    pub max_routing_latency_ms: u64,
    pub data_quality_threshold: f64,
    pub enable_data_validation: bool,
    pub backup_data_sources: Vec<String>,
    pub primary_data_source: String,
}

impl Default for MarketInterfaceConfig {
    fn default() -> Self {
        Self {
            enable_real_time_data: true,
            data_update_interval_ms: 100,  // 100ms updates
            max_data_age_seconds: 5,       // 5 seconds max age
            enable_order_routing: true,
            enable_smart_routing: true,
            max_routing_latency_ms: 50,    // 50ms max routing latency
            data_quality_threshold: 0.95,  // 95% quality threshold
            enable_data_validation: true,
            backup_data_sources: vec![
                "backup_feed_1".to_string(),
                "backup_feed_2".to_string(),
            ],
            primary_data_source: "alpha_vantage".to_string(),
        }
    }
}

// Market Data Source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataSource {
    pub name: String,
    pub url: String,
    pub api_key: Option<String>,
    pub is_active: bool,
    pub priority: u32,
    pub latency_ms: f64,
    pub reliability_score: f64,
    pub last_update: DateTime<Utc>,
}

// Market Data Feed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataFeed {
    pub source: String,
    pub instrument_id: Uuid,
    pub symbol: String,
    pub data: MarketData,
    pub quality_score: f64,
    pub latency_ms: f64,
    pub sequence_number: u64,
}

// Order Routing Destination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDestination {
    pub name: String,
    pub venue_type: VenueType,
    pub is_active: bool,
    pub latency_ms: f64,
    pub fill_rate: f64,
    pub cost_per_trade: f64,
    pub supported_instruments: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VenueType {
    Exchange,
    DarkPool,
    ECN,
    MarketMaker,
    InternalCrossing,
}

// Market Interface Events
#[derive(Debug, Clone)]
pub enum MarketInterfaceEvent {
    DataReceived(MarketDataFeed),
    DataQualityAlert(String),
    SourceFailover(String, String),
    OrderRouted(Uuid, String),
    RoutingFailure(Uuid, String),
    LatencyAlert(String, f64),
}

// Market Interface Implementation
pub struct MarketInterface {
    config: MarketInterfaceConfig,
    database: Database,
    http_client: Client,
    
    // Data Sources
    data_sources: Arc<RwLock<HashMap<String, MarketDataSource>>>,
    active_feeds: Arc<RwLock<HashMap<Uuid, MarketDataFeed>>>,
    data_history: Arc<RwLock<HashMap<Uuid, VecDeque<MarketData>>>>,
    
    // Order Routing
    routing_destinations: Arc<RwLock<HashMap<String, RoutingDestination>>>,
    routing_statistics: Arc<RwLock<RoutingStatistics>>,
    
    // Event System
    event_sender: mpsc::UnboundedSender<MarketInterfaceEvent>,
    event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<MarketInterfaceEvent>>>>,
    
    // Data Quality
    data_quality_metrics: Arc<RwLock<DataQualityMetrics>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingStatistics {
    pub total_orders_routed: u32,
    pub successful_routes: u32,
    pub failed_routes: u32,
    pub average_routing_latency_ms: f64,
    pub venue_performance: HashMap<String, VenuePerformance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VenuePerformance {
    pub orders_sent: u32,
    pub orders_filled: u32,
    pub average_fill_time_ms: f64,
    pub average_slippage_bps: f64,
    pub rejection_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQualityMetrics {
    pub total_updates: u32,
    pub valid_updates: u32,
    pub invalid_updates: u32,
    pub average_latency_ms: f64,
    pub data_gaps: u32,
    pub quality_score: f64,
    pub last_quality_check: DateTime<Utc>,
}

impl Default for RoutingStatistics {
    fn default() -> Self {
        Self {
            total_orders_routed: 0,
            successful_routes: 0,
            failed_routes: 0,
            average_routing_latency_ms: 0.0,
            venue_performance: HashMap::new(),
        }
    }
}

impl Default for DataQualityMetrics {
    fn default() -> Self {
        Self {
            total_updates: 0,
            valid_updates: 0,
            invalid_updates: 0,
            average_latency_ms: 0.0,
            data_gaps: 0,
            quality_score: 1.0,
            last_quality_check: Utc::now(),
        }
    }
}

impl MarketInterface {
    pub async fn new(config: MarketInterfaceConfig, database: Database) -> Result<Self> {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        let interface = Self {
            config,
            database,
            http_client: Client::new(),
            data_sources: Arc::new(RwLock::new(HashMap::new())),
            active_feeds: Arc::new(RwLock::new(HashMap::new())),
            data_history: Arc::new(RwLock::new(HashMap::new())),
            routing_destinations: Arc::new(RwLock::new(HashMap::new())),
            routing_statistics: Arc::new(RwLock::new(RoutingStatistics::default())),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
            data_quality_metrics: Arc::new(RwLock::new(DataQualityMetrics::default())),
        };

        // Initialize data sources
        interface.initialize_data_sources().await?;
        
        // Initialize routing destinations
        interface.initialize_routing_destinations().await?;
        
        // Start background processing
        interface.start_background_processing().await?;

        info!("Market Interface initialized successfully");
        Ok(interface)
    }

    /// Add market data source
    pub async fn add_data_source(&self, source: MarketDataSource) -> Result<()> {
        let mut sources_guard = self.data_sources.write().await;
        sources_guard.insert(source.name.clone(), source.clone());
        
        info!("Added market data source: {}", source.name);
        Ok(())
    }

    /// Get real-time market data for instrument
    pub async fn get_market_data(&self, instrument_id: Uuid) -> Option<MarketData> {
        let feeds_guard = self.active_feeds.read().await;
        feeds_guard.get(&instrument_id).map(|feed| feed.data.clone())
    }

    /// Get order book for instrument
    pub async fn get_order_book(&self, instrument_id: Uuid) -> Option<OrderBook> {
        // This would typically fetch from the order book data source
        // For now, we'll create a mock order book
        Some(OrderBook {
            instrument_id,
            bids: vec![
                OrderBookEntry {
                    price: 100.0,
                    size: 1000.0,
                    order_count: 5,
                    timestamp: Utc::now(),
                },
            ],
            asks: vec![
                OrderBookEntry {
                    price: 100.1,
                    size: 1500.0,
                    order_count: 3,
                    timestamp: Utc::now(),
                },
            ],
            last_updated: Utc::now(),
            sequence_number: 1,
        })
    }

    /// Route order to best venue
    pub async fn route_order(&self, order_id: Uuid, instrument_id: Uuid, quantity: f64) -> Result<String> {
        let destinations_guard = self.routing_destinations.read().await;
        
        // Find best venue for this order
        let best_venue = self.find_best_venue(&destinations_guard, instrument_id, quantity).await?;
        
        // Update routing statistics
        {
            let mut stats_guard = self.routing_statistics.write().await;
            stats_guard.total_orders_routed += 1;
            stats_guard.successful_routes += 1;
        }

        // Send event
        self.send_event(MarketInterfaceEvent::OrderRouted(order_id, best_venue.clone())).await;

        info!("Order {} routed to venue: {}", order_id, best_venue);
        Ok(best_venue)
    }

    /// Update market data from external source
    pub async fn update_market_data(&self, ticks: Vec<MarketTick>) -> Result<()> {
        for tick in ticks {
            let market_data = MarketData {
                instrument_id: tick.instrument_id,
                bid_price: tick.bid_price,
                ask_price: tick.ask_price,
                bid_size: tick.bid_size,
                ask_size: tick.ask_size,
                last_price: tick.last_price,
                timestamp: tick.timestamp,
            };

            // Validate data quality
            let quality_score = self.calculate_data_quality(&market_data).await;
            
            if quality_score >= self.config.data_quality_threshold {
                // Update active feeds
                {
                    let mut feeds_guard = self.active_feeds.write().await;
                    let feed = MarketDataFeed {
                        source: self.config.primary_data_source.clone(),
                        instrument_id: tick.instrument_id,
                        symbol: format!("SYMBOL_{}", tick.instrument_id), // Mock symbol
                        data: market_data.clone(),
                        quality_score,
                        latency_ms: 0.0, // Would be calculated from timestamp
                        sequence_number: 1, // Would be incremented
                    };
                    feeds_guard.insert(tick.instrument_id, feed.clone());
                    
                    // Send event
                    self.send_event(MarketInterfaceEvent::DataReceived(feed)).await;
                }

                // Update data history
                {
                    let mut history_guard = self.data_history.write().await;
                    let history = history_guard.entry(tick.instrument_id).or_insert_with(VecDeque::new);
                    history.push_back(market_data);
                    
                    // Keep only recent data
                    while history.len() > 1000 {
                        history.pop_front();
                    }
                }
            } else {
                warn!("Low quality data received for instrument {}: score {}", 
                      tick.instrument_id, quality_score);
                
                self.send_event(MarketInterfaceEvent::DataQualityAlert(
                    format!("Low quality data for instrument {}", tick.instrument_id)
                )).await;
            }
        }

        Ok(())
    }

    /// Get data quality metrics
    pub async fn get_data_quality_metrics(&self) -> DataQualityMetrics {
        self.data_quality_metrics.read().await.clone()
    }

    /// Get routing statistics
    pub async fn get_routing_statistics(&self) -> RoutingStatistics {
        self.routing_statistics.read().await.clone()
    }

    /// Initialize data sources
    async fn initialize_data_sources(&self) -> Result<()> {
        // Add Alpha Vantage as primary source
        let alpha_vantage = MarketDataSource {
            name: "alpha_vantage".to_string(),
            url: "https://www.alphavantage.co/query".to_string(),
            api_key: Some("EZDZ4VOFQ2GRA7VU".to_string()), // From user's API key
            is_active: true,
            priority: 1,
            latency_ms: 100.0,
            reliability_score: 0.95,
            last_update: Utc::now(),
        };

        self.add_data_source(alpha_vantage).await?;

        // Add backup sources
        for (i, backup_name) in self.config.backup_data_sources.iter().enumerate() {
            let backup_source = MarketDataSource {
                name: backup_name.clone(),
                url: format!("https://backup-{}.example.com", i + 1),
                api_key: None,
                is_active: false, // Backup sources start inactive
                priority: (i + 2) as u32,
                latency_ms: 200.0,
                reliability_score: 0.85,
                last_update: Utc::now(),
            };
            self.add_data_source(backup_source).await?;
        }

        info!("Initialized {} data sources", self.config.backup_data_sources.len() + 1);
        Ok(())
    }

    /// Initialize routing destinations
    async fn initialize_routing_destinations(&self) -> Result<()> {
        let destinations = vec![
            RoutingDestination {
                name: "primary_exchange".to_string(),
                venue_type: VenueType::Exchange,
                is_active: true,
                latency_ms: 10.0,
                fill_rate: 0.95,
                cost_per_trade: 0.001,
                supported_instruments: vec![], // Would be populated with actual instruments
            },
            RoutingDestination {
                name: "dark_pool_1".to_string(),
                venue_type: VenueType::DarkPool,
                is_active: true,
                latency_ms: 15.0,
                fill_rate: 0.85,
                cost_per_trade: 0.0005,
                supported_instruments: vec![],
            },
            RoutingDestination {
                name: "ecn_venue".to_string(),
                venue_type: VenueType::ECN,
                is_active: true,
                latency_ms: 8.0,
                fill_rate: 0.90,
                cost_per_trade: 0.0008,
                supported_instruments: vec![],
            },
        ];

        {
            let mut destinations_guard = self.routing_destinations.write().await;
            for dest in destinations {
                destinations_guard.insert(dest.name.clone(), dest);
            }
        }

        info!("Initialized routing destinations");
        Ok(())
    }

    /// Find best venue for order
    async fn find_best_venue(
        &self,
        destinations: &HashMap<String, RoutingDestination>,
        _instrument_id: Uuid,
        quantity: f64,
    ) -> Result<String> {
        // Simple venue selection logic - in production this would be much more sophisticated
        let mut best_venue = None;
        let mut best_score = 0.0;

        for (name, dest) in destinations {
            if !dest.is_active {
                continue;
            }

            // Calculate venue score based on multiple factors
            let latency_score = 1.0 / (1.0 + dest.latency_ms / 100.0);
            let fill_rate_score = dest.fill_rate;
            let cost_score = 1.0 / (1.0 + dest.cost_per_trade * quantity);

            let total_score = (latency_score + fill_rate_score + cost_score) / 3.0;

            if total_score > best_score {
                best_score = total_score;
                best_venue = Some(name.clone());
            }
        }

        best_venue.ok_or_else(|| {
            crate::utils::PantherSwapError::trading("No active venues available".to_string())
        })
    }

    /// Calculate data quality score
    async fn calculate_data_quality(&self, data: &MarketData) -> f64 {
        let mut quality_score = 1.0;

        // Check data freshness
        let age = Utc::now() - data.timestamp;
        if age.num_seconds() > self.config.max_data_age_seconds as i64 {
            quality_score *= 0.5;
        }

        // Check spread reasonableness
        if data.ask_price > 0.0 && data.bid_price > 0.0 {
            let spread_pct = (data.ask_price - data.bid_price) / data.bid_price;
            if spread_pct > 0.1 { // 10% spread seems unreasonable
                quality_score *= 0.7;
            }
        } else {
            quality_score *= 0.3; // Missing bid/ask is poor quality
        }

        // Check size reasonableness
        if data.bid_size <= 0.0 || data.ask_size <= 0.0 {
            quality_score *= 0.8;
        }

        // Update quality metrics
        {
            let mut metrics_guard = self.data_quality_metrics.write().await;
            metrics_guard.total_updates += 1;
            if quality_score >= self.config.data_quality_threshold {
                metrics_guard.valid_updates += 1;
            } else {
                metrics_guard.invalid_updates += 1;
            }
            metrics_guard.quality_score = metrics_guard.valid_updates as f64 / metrics_guard.total_updates as f64;
            metrics_guard.last_quality_check = Utc::now();
        }

        quality_score
    }

    /// Start background processing
    async fn start_background_processing(&self) -> Result<()> {
        // Start data source monitoring
        self.start_data_source_monitoring().await?;

        // Start event processing
        self.start_event_processing().await?;

        Ok(())
    }

    /// Start data source monitoring
    async fn start_data_source_monitoring(&self) -> Result<()> {
        let data_sources = self.data_sources.clone();
        let event_sender = self.event_sender.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30)); // Check every 30 seconds

            loop {
                interval.tick().await;

                // Monitor data source health
                let sources_guard = data_sources.read().await;
                for (name, source) in sources_guard.iter() {
                    let age = Utc::now() - source.last_update;

                    if age.num_seconds() > 60 && source.is_active {
                        // Data source hasn't updated in over a minute
                        warn!("Data source {} appears stale: {} seconds old", name, age.num_seconds());

                        if let Err(e) = event_sender.send(MarketInterfaceEvent::DataQualityAlert(
                            format!("Stale data from source: {}", name)
                        )) {
                            error!("Failed to send data quality alert: {}", e);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Start event processing
    async fn start_event_processing(&self) -> Result<()> {
        let mut receiver_guard = self.event_receiver.write().await;
        if let Some(mut receiver) = receiver_guard.take() {
            drop(receiver_guard);

            tokio::spawn(async move {
                while let Some(event) = receiver.recv().await {
                    // Process events (logging, notifications, etc.)
                    match event {
                        MarketInterfaceEvent::DataReceived(feed) => {
                            debug!("Data received for instrument: {} from source: {}",
                                   feed.instrument_id, feed.source);
                        },
                        MarketInterfaceEvent::DataQualityAlert(msg) => {
                            warn!("Data quality alert: {}", msg);
                        },
                        MarketInterfaceEvent::SourceFailover(from, to) => {
                            warn!("Data source failover: {} -> {}", from, to);
                        },
                        MarketInterfaceEvent::OrderRouted(order_id, venue) => {
                            info!("Order {} routed to venue: {}", order_id, venue);
                        },
                        MarketInterfaceEvent::RoutingFailure(order_id, reason) => {
                            error!("Order routing failed for {}: {}", order_id, reason);
                        },
                        MarketInterfaceEvent::LatencyAlert(source, latency) => {
                            warn!("High latency alert from {}: {}ms", source, latency);
                        },
                    }
                }
            });
        }

        Ok(())
    }

    /// Send an event
    async fn send_event(&self, event: MarketInterfaceEvent) {
        if let Err(e) = self.event_sender.send(event) {
            error!("Failed to send market interface event: {}", e);
        }
    }

    /// Fetch data from Alpha Vantage (example implementation)
    pub async fn fetch_alpha_vantage_data(&self, symbol: &str) -> Result<MarketData> {
        let url = format!(
            "https://www.alphavantage.co/query?function=GLOBAL_QUOTE&symbol={}&apikey={}",
            symbol,
            "EZDZ4VOFQ2GRA7VU" // User's API key
        );

        let response = self.http_client.get(&url).send().await
            .map_err(|e| crate::utils::PantherSwapError::trading(format!("HTTP request failed: {}", e)))?;

        let data: serde_json::Value = response.json().await
            .map_err(|e| crate::utils::PantherSwapError::trading(format!("JSON parsing failed: {}", e)))?;

        // Parse Alpha Vantage response (simplified)
        let quote = data.get("Global Quote")
            .ok_or_else(|| crate::utils::PantherSwapError::trading("Invalid Alpha Vantage response".to_string()))?;

        let price = quote.get("05. price")
            .and_then(|p| p.as_str())
            .and_then(|p| p.parse::<f64>().ok())
            .unwrap_or(0.0);

        Ok(MarketData {
            instrument_id: Uuid::new_v4(), // Would map symbol to instrument ID
            bid_price: price * 0.999, // Mock bid slightly below price
            ask_price: price * 1.001, // Mock ask slightly above price
            bid_size: 1000.0,
            ask_size: 1000.0,
            last_price: Some(price),
            timestamp: Utc::now(),
        })
    }
}
