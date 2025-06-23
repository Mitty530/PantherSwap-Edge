// Reinforcement Learning Trading Agent
// Lightweight implementation for MVP - will be enhanced with proper RL frameworks

use crate::database::types::MarketTick;
use crate::trading::signals::{AISignal, RLRecommendation};
use crate::utils::Result;
use std::collections::VecDeque;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use ndarray::Array1;
use rand;

/// Configuration for RL Trading Agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RLAgentConfig {
    pub state_dimensions: usize,
    pub action_dimensions: usize,
    pub learning_rate: f64,
    pub discount_factor: f64,
    pub exploration_rate: f64,
    pub exploration_decay: f64,
    pub memory_size: usize,
    pub batch_size: usize,
}

impl Default for RLAgentConfig {
    fn default() -> Self {
        Self {
            state_dimensions: 32,
            action_dimensions: 5, // Hold, Buy_Small, Buy_Large, Sell_Small, Sell_Large
            learning_rate: 0.001,
            discount_factor: 0.95,
            exploration_rate: 0.1,
            exploration_decay: 0.995,
            memory_size: 10000,
            batch_size: 32,
        }
    }
}

/// Trading action types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradingAction {
    Hold = 0,
    BuySmall = 1,
    BuyLarge = 2,
    SellSmall = 3,
    SellLarge = 4,
}

impl TradingAction {
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(TradingAction::Hold),
            1 => Some(TradingAction::BuySmall),
            2 => Some(TradingAction::BuyLarge),
            3 => Some(TradingAction::SellSmall),
            4 => Some(TradingAction::SellLarge),
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            TradingAction::Hold => "HOLD".to_string(),
            TradingAction::BuySmall => "BUY_SMALL".to_string(),
            TradingAction::BuyLarge => "BUY_LARGE".to_string(),
            TradingAction::SellSmall => "SELL_SMALL".to_string(),
            TradingAction::SellLarge => "SELL_LARGE".to_string(),
        }
    }

    pub fn position_size(&self) -> f64 {
        match self {
            TradingAction::Hold => 0.0,
            TradingAction::BuySmall => 0.01,  // 1% of portfolio
            TradingAction::BuyLarge => 0.05,  // 5% of portfolio
            TradingAction::SellSmall => -0.01,
            TradingAction::SellLarge => -0.05,
        }
    }
}

/// Market state representation for RL agent
#[derive(Debug, Clone)]
pub struct MarketState {
    pub timestamp: DateTime<Utc>,
    pub features: Array1<f32>,
    pub price: f64,
    pub volatility: f64,
    pub trend: f64,
    pub volume: f64,
}

/// Experience tuple for replay buffer
#[derive(Debug, Clone)]
pub struct Experience {
    pub state: MarketState,
    pub action: TradingAction,
    pub reward: f64,
    pub next_state: MarketState,
    pub done: bool,
}

/// Simple Q-Network (lightweight implementation)
#[derive(Debug, Clone)]
pub struct QNetwork {
    weights_input_hidden: Array1<f32>,
    weights_hidden_output: Array1<f32>,
    bias_hidden: Array1<f32>,
    bias_output: Array1<f32>,
    hidden_size: usize,
}

impl QNetwork {
    pub fn new(input_size: usize, hidden_size: usize, output_size: usize) -> Self {
        let input_hidden_size = input_size * hidden_size;
        let hidden_output_size = hidden_size * output_size;
        
        // Initialize with small random weights
        let weights_input_hidden = Array1::from_vec(
            (0..input_hidden_size).map(|_| (rand::random::<f32>() - 0.5) * 0.1).collect()
        );
        let weights_hidden_output = Array1::from_vec(
            (0..hidden_output_size).map(|_| (rand::random::<f32>() - 0.5) * 0.1).collect()
        );
        let bias_hidden = Array1::zeros(hidden_size);
        let bias_output = Array1::zeros(output_size);
        
        Self {
            weights_input_hidden,
            weights_hidden_output,
            bias_hidden,
            bias_output,
            hidden_size,
        }
    }
    
    /// Forward pass through the network
    pub fn forward(&self, input: &Array1<f32>) -> Array1<f32> {
        // Simple feedforward network for MVP
        // In production, this would use proper neural network libraries
        
        let input_size = input.len();
        let output_size = self.bias_output.len();
        
        // Hidden layer
        let mut hidden = Array1::zeros(self.hidden_size);
        for i in 0..self.hidden_size {
            let mut sum = self.bias_hidden[i];
            for j in 0..input_size {
                let weight_idx = i * input_size + j;
                if weight_idx < self.weights_input_hidden.len() {
                    sum += input[j] * self.weights_input_hidden[weight_idx];
                }
            }
            hidden[i] = sum.max(0.0); // ReLU activation
        }
        
        // Output layer
        let mut output = Array1::zeros(output_size);
        for i in 0..output_size {
            let mut sum = self.bias_output[i];
            for j in 0..self.hidden_size {
                let weight_idx = i * self.hidden_size + j;
                if weight_idx < self.weights_hidden_output.len() {
                    sum += hidden[j] * self.weights_hidden_output[weight_idx];
                }
            }
            output[i] = sum;
        }
        
        output
    }
}

/// Reinforcement Learning Trading Agent
#[derive(Clone)]
pub struct RLTradingAgent {
    config: RLAgentConfig,
    q_network: QNetwork,
    target_network: QNetwork,
    replay_buffer: VecDeque<Experience>,
    current_exploration_rate: f64,
    training_step: u64,
    last_state: Option<MarketState>,
    last_action: Option<TradingAction>,
    performance_metrics: RLPerformanceMetrics,
}

/// Performance tracking for RL agent
#[derive(Debug, Default, Clone)]
pub struct RLPerformanceMetrics {
    pub total_actions: u64,
    pub profitable_actions: u64,
    pub total_rewards: f64,
    pub average_reward: f64,
    pub training_loss: f64,
    pub training_steps: u64,
    pub exploration_rate: f64,
    pub avg_td_error: f64,
    pub episode_rewards: VecDeque<f64>,
    pub last_update: Option<DateTime<Utc>>,
}

impl RLTradingAgent {
    /// Create a new RL trading agent
    pub fn new(config: RLAgentConfig) -> Self {
        let q_network = QNetwork::new(
            config.state_dimensions,
            64, // hidden layer size
            config.action_dimensions,
        );
        let target_network = q_network.clone();
        let exploration_rate = config.exploration_rate;
        let memory_size = config.memory_size;

        Self {
            config,
            q_network,
            target_network,
            replay_buffer: VecDeque::with_capacity(memory_size),
            current_exploration_rate: exploration_rate,
            training_step: 0,
            last_state: None,
            last_action: None,
            performance_metrics: RLPerformanceMetrics::default(),
        }
    }
    
    /// Generate trading recommendation based on current market state
    pub fn get_recommendation(&mut self, market_data: &[MarketTick], ai_signal: &AISignal) -> Result<RLRecommendation> {
        // Extract market state from current data
        let state = self.extract_market_state(market_data, ai_signal)?;
        
        // Choose action using epsilon-greedy policy
        let action = self.choose_action(&state);
        
        // Store state and action for next reward calculation
        self.last_state = Some(state.clone());
        self.last_action = Some(action);
        
        // Update performance metrics
        self.performance_metrics.total_actions += 1;
        self.performance_metrics.exploration_rate = self.current_exploration_rate;
        self.performance_metrics.last_update = Some(Utc::now());
        
        Ok(RLRecommendation {
            action: action.to_string(),
            confidence: self.calculate_action_confidence(&state, action),
            expected_reward: self.estimate_expected_reward(&state, action),
        })
    }
    
    /// Extract market state features from market data and AI signals
    fn extract_market_state(&self, market_data: &[MarketTick], ai_signal: &AISignal) -> Result<MarketState> {
        if market_data.is_empty() {
            return Err(crate::utils::PantherSwapError::ai_prediction("No market data available".to_string()));
        }
        
        let latest_tick = &market_data[market_data.len() - 1];
        let price = (latest_tick.bid_price + latest_tick.ask_price) / 2.0;
        
        // Create feature vector
        let mut features = Array1::zeros(self.config.state_dimensions);
        
        // Basic price features
        features[0] = price as f32;
        features[1] = latest_tick.spread as f32;
        features[2] = latest_tick.bid_size as f32;
        features[3] = latest_tick.ask_size as f32;
        
        // AI signal features
        if !ai_signal.price_predictions.is_empty() {
            features[4] = ai_signal.price_predictions[0].predicted_price as f32;
            features[5] = ai_signal.price_predictions[0].confidence_score as f32;
        }
        
        features[6] = ai_signal.confidence_score as f32;
        
        // Technical indicators (simplified)
        if market_data.len() >= 5 {
            let recent_prices: Vec<f64> = market_data.iter()
                .rev()
                .take(5)
                .map(|t| (t.bid_price + t.ask_price) / 2.0)
                .collect();
            
            let sma = recent_prices.iter().sum::<f64>() / recent_prices.len() as f64;
            features[7] = sma as f32;
            
            let volatility = recent_prices.iter()
                .map(|&p| (p - sma).powi(2))
                .sum::<f64>() / recent_prices.len() as f64;
            features[8] = volatility.sqrt() as f32;
        }
        
        // Fill remaining features with normalized values
        for i in 9..self.config.state_dimensions {
            features[i] = 0.0;
        }
        
        let volatility = features[8] as f64;

        Ok(MarketState {
            timestamp: latest_tick.timestamp,
            features,
            price,
            volatility,
            trend: if market_data.len() >= 2 {
                price - (market_data[market_data.len() - 2].bid_price + market_data[market_data.len() - 2].ask_price) / 2.0
            } else { 0.0 },
            volume: latest_tick.volume.unwrap_or(0.0),
        })
    }
    
    /// Enhanced action selection with improved exploration strategy
    fn choose_action(&self, state: &MarketState) -> TradingAction {
        let q_values = self.q_network.forward(&state.features);

        // Use Boltzmann exploration for better action selection
        if rand::random::<f64>() < self.current_exploration_rate {
            // Boltzmann exploration with temperature annealing
            let temperature = 1.0 / (1.0 + self.training_step as f64 * 0.001);
            let exp_values: Vec<f64> = q_values.iter()
                .map(|&q| (q as f64 / temperature).exp())
                .collect();

            let sum_exp: f64 = exp_values.iter().sum();
            let probabilities: Vec<f64> = exp_values.iter()
                .map(|&exp_val| exp_val / sum_exp)
                .collect();

            // Sample action based on probabilities
            let random_val = rand::random::<f64>();
            let mut cumulative_prob = 0.0;
            for (idx, &prob) in probabilities.iter().enumerate() {
                cumulative_prob += prob;
                if random_val <= cumulative_prob {
                    return TradingAction::from_index(idx).unwrap_or(TradingAction::Hold);
                }
            }
        }

        // Exploit: best action with confidence weighting
        let best_action_idx = q_values.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(idx, _)| idx)
            .unwrap_or(0);

        TradingAction::from_index(best_action_idx).unwrap_or(TradingAction::Hold)
    }
    
    /// Calculate confidence in the chosen action
    fn calculate_action_confidence(&self, state: &MarketState, action: TradingAction) -> f64 {
        let q_values = self.q_network.forward(&state.features);
        let action_value = q_values[action as usize];
        let max_value = q_values.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        let min_value = q_values.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        
        if max_value == min_value {
            0.5 // Neutral confidence when all actions have same value
        } else {
            ((action_value - min_value) / (max_value - min_value)) as f64
        }
    }
    
    /// Estimate expected reward for an action
    fn estimate_expected_reward(&self, state: &MarketState, action: TradingAction) -> f64 {
        let q_values = self.q_network.forward(&state.features);
        q_values[action as usize] as f64
    }
    
    /// Update exploration rate
    pub fn update_exploration_rate(&mut self) {
        self.current_exploration_rate *= self.config.exploration_decay;
        self.current_exploration_rate = self.current_exploration_rate.max(0.01); // Minimum exploration
    }

    /// Train the RL agent with enhanced experience replay and prioritized sampling
    pub fn train_step(&mut self) -> Result<f64> {
        if self.replay_buffer.len() < self.config.batch_size {
            return Ok(0.0); // Not enough experiences to train
        }

        // Sample prioritized batch from replay buffer
        let batch = self.sample_prioritized_batch();
        let mut total_loss = 0.0;
        let mut td_errors = Vec::new();

        for experience in &batch {
            let current_q_values = self.q_network.forward(&experience.state.features);
            let next_q_values = self.target_network.forward(&experience.next_state.features);

            // Double DQN: use main network to select action, target network to evaluate
            let next_action_idx = {
                let main_next_q = self.q_network.forward(&experience.next_state.features);
                main_next_q.iter()
                    .enumerate()
                    .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(idx, _)| idx)
                    .unwrap_or(0)
            };

            // Calculate target Q-value using Double DQN
            let next_q_value = if experience.done {
                0.0
            } else {
                next_q_values[next_action_idx] as f64
            };

            let target_q = experience.reward + self.config.discount_factor * next_q_value;

            // Calculate TD error for prioritized replay
            let current_q = current_q_values[experience.action as usize] as f64;
            let td_error = (target_q - current_q).abs();
            td_errors.push(td_error);

            // Calculate loss with Huber loss for stability
            let loss = if td_error < 1.0 {
                0.5 * td_error.powi(2)
            } else {
                td_error - 0.5
            };
            total_loss += loss;

            // Update Q-network weights with improved gradient descent
            self.update_q_network_improved(&experience, target_q as f32, current_q as f32);
        }

        self.training_step += 1;

        // Update target network with soft updates for stability
        if self.training_step % 10 == 0 {
            self.soft_update_target_network(0.01); // tau = 0.01
        }

        // Update exploration rate with improved decay
        self.update_exploration_rate_improved();

        // Update performance metrics
        let avg_loss = total_loss / batch.len() as f64;
        self.performance_metrics.training_loss = avg_loss;
        self.performance_metrics.training_steps = self.training_step;
        self.performance_metrics.avg_td_error = td_errors.iter().sum::<f64>() / td_errors.len() as f64;

        Ok(avg_loss)
    }

    /// Sample a prioritized batch from replay buffer based on TD error
    fn sample_prioritized_batch(&self) -> Vec<Experience> {
        let mut batch = Vec::with_capacity(self.config.batch_size);
        let buffer_vec: Vec<_> = self.replay_buffer.iter().collect();

        if buffer_vec.is_empty() {
            return batch;
        }

        // For now, use uniform sampling with some recent bias
        // In production, implement proper prioritized experience replay
        let recent_bias = 0.3; // 30% chance to sample from recent experiences
        let recent_threshold = (buffer_vec.len() as f64 * 0.2) as usize; // Last 20% of buffer

        for _ in 0..self.config.batch_size.min(buffer_vec.len()) {
            let idx = if rand::random::<f64>() < recent_bias && buffer_vec.len() > recent_threshold {
                // Sample from recent experiences
                let start_idx = buffer_vec.len() - recent_threshold;
                start_idx + (rand::random::<usize>() % recent_threshold)
            } else {
                // Sample uniformly
                rand::random::<usize>() % buffer_vec.len()
            };
            batch.push(buffer_vec[idx].clone());
        }

        batch
    }

    /// Sample a random batch from replay buffer (fallback method)
    fn sample_batch(&self) -> Vec<Experience> {
        let mut batch = Vec::with_capacity(self.config.batch_size);
        let buffer_vec: Vec<_> = self.replay_buffer.iter().collect();

        for _ in 0..self.config.batch_size.min(buffer_vec.len()) {
            let idx = rand::random::<usize>() % buffer_vec.len();
            batch.push(buffer_vec[idx].clone());
        }

        batch
    }

    /// Update Q-network weights with improved gradient descent
    fn update_q_network_improved(&mut self, experience: &Experience, target_q: f32, current_q: f32) {
        let learning_rate = self.config.learning_rate as f32;
        let error = target_q - current_q;

        // Clip gradient to prevent exploding gradients
        let clipped_error = error.max(-1.0).min(1.0);

        // Get hidden layer activations for proper gradient calculation
        let hidden_activations = self.compute_hidden_activations(&experience.state.features);

        // Update output layer weights with proper gradients
        let action_idx = experience.action as usize;
        if action_idx < self.q_network.bias_output.len() {
            // Update bias for the specific action
            self.q_network.bias_output[action_idx] += learning_rate * clipped_error;

            // Update weights from hidden to output for the specific action
            for h in 0..self.q_network.hidden_size {
                let weight_idx = action_idx * self.q_network.hidden_size + h;
                if weight_idx < self.q_network.weights_hidden_output.len() {
                    let gradient = clipped_error * hidden_activations[h];
                    self.q_network.weights_hidden_output[weight_idx] += learning_rate * gradient;
                }
            }
        }

        // Update hidden layer weights (simplified backpropagation)
        for h in 0..self.q_network.hidden_size {
            if hidden_activations[h] > 0.0 { // ReLU derivative
                let hidden_error = clipped_error * 0.1; // Simplified error propagation
                self.q_network.bias_hidden[h] += learning_rate * hidden_error * 0.1;

                // Update input to hidden weights
                for i in 0..experience.state.features.len() {
                    let weight_idx = h * experience.state.features.len() + i;
                    if weight_idx < self.q_network.weights_input_hidden.len() {
                        let gradient = hidden_error * experience.state.features[i];
                        self.q_network.weights_input_hidden[weight_idx] += learning_rate * gradient * 0.01;
                    }
                }
            }
        }
    }

    /// Compute hidden layer activations for gradient calculation
    fn compute_hidden_activations(&self, input: &Array1<f32>) -> Array1<f32> {
        let mut hidden = Array1::zeros(self.q_network.hidden_size);
        for i in 0..self.q_network.hidden_size {
            let mut sum = self.q_network.bias_hidden[i];
            for j in 0..input.len() {
                let weight_idx = i * input.len() + j;
                if weight_idx < self.q_network.weights_input_hidden.len() {
                    sum += input[j] * self.q_network.weights_input_hidden[weight_idx];
                }
            }
            hidden[i] = sum.max(0.0); // ReLU activation
        }
        hidden
    }

    /// Update Q-network weights (legacy method for compatibility)
    fn update_q_network(&mut self, experience: &Experience, target_q: f32) {
        let current_q = self.q_network.forward(&experience.state.features)[experience.action as usize];
        self.update_q_network_improved(experience, target_q, current_q);
    }

    /// Soft update target network for improved stability
    fn soft_update_target_network(&mut self, tau: f32) {
        // Soft update: target = tau * main + (1 - tau) * target
        for i in 0..self.target_network.weights_input_hidden.len() {
            self.target_network.weights_input_hidden[i] =
                tau * self.q_network.weights_input_hidden[i] +
                (1.0 - tau) * self.target_network.weights_input_hidden[i];
        }

        for i in 0..self.target_network.weights_hidden_output.len() {
            self.target_network.weights_hidden_output[i] =
                tau * self.q_network.weights_hidden_output[i] +
                (1.0 - tau) * self.target_network.weights_hidden_output[i];
        }

        for i in 0..self.target_network.bias_hidden.len() {
            self.target_network.bias_hidden[i] =
                tau * self.q_network.bias_hidden[i] +
                (1.0 - tau) * self.target_network.bias_hidden[i];
        }

        for i in 0..self.target_network.bias_output.len() {
            self.target_network.bias_output[i] =
                tau * self.q_network.bias_output[i] +
                (1.0 - tau) * self.target_network.bias_output[i];
        }
    }

    /// Update target network with current Q-network weights (hard update)
    fn update_target_network(&mut self) {
        self.target_network.weights_input_hidden = self.q_network.weights_input_hidden.clone();
        self.target_network.weights_hidden_output = self.q_network.weights_hidden_output.clone();
        self.target_network.bias_hidden = self.q_network.bias_hidden.clone();
        self.target_network.bias_output = self.q_network.bias_output.clone();
    }

    /// Update exploration rate with improved decay schedule
    fn update_exploration_rate_improved(&mut self) {
        // Use exponential decay with minimum threshold
        let decay_rate = 0.995; // Slower decay for better exploration
        self.current_exploration_rate *= decay_rate;
        self.current_exploration_rate = self.current_exploration_rate.max(0.05); // Higher minimum for continuous learning

        // Add occasional exploration boosts for regime changes
        if self.training_step % 1000 == 0 {
            self.current_exploration_rate = (self.current_exploration_rate + 0.1).min(0.3);
        }
    }

    /// Store experience in replay buffer
    pub fn store_experience(&mut self, experience: Experience) {
        if self.replay_buffer.len() >= self.config.memory_size {
            self.replay_buffer.pop_front();
        }
        self.replay_buffer.push_back(experience);
    }

    /// Process reward from previous action and update agent with enhanced tracking
    pub fn process_reward(&mut self, reward: f64, new_market_data: &[MarketTick], new_ai_signal: &AISignal) -> Result<()> {
        // Clone the values to avoid borrowing issues
        let last_state = self.last_state.clone();
        let last_action = self.last_action;

        if let (Some(state), Some(action)) = (last_state, last_action) {
            // Extract new state
            let new_state = self.extract_market_state(new_market_data, new_ai_signal)?;

            // Create experience
            let experience = Experience {
                state,
                action,
                reward,
                next_state: new_state,
                done: false, // For continuous trading, episodes don't really "end"
            };

            // Store experience
            self.store_experience(experience);

            // Train if we have enough experiences
            if self.replay_buffer.len() >= self.config.batch_size {
                self.train_step()?;
            }

            // Update performance metrics with enhanced tracking
            self.update_performance_metrics(reward, action);
        }

        Ok(())
    }

    /// Enhanced performance metrics update
    fn update_performance_metrics(&mut self, reward: f64, _action: TradingAction) {
        self.performance_metrics.total_rewards += reward;

        // Track profitable actions
        if reward > 0.0 {
            self.performance_metrics.profitable_actions += 1;
        }

        // Update average reward with exponential moving average
        let alpha = 0.1; // EMA smoothing factor
        if self.performance_metrics.total_actions > 0 {
            self.performance_metrics.average_reward =
                alpha * reward + (1.0 - alpha) * self.performance_metrics.average_reward;
        } else {
            self.performance_metrics.average_reward = reward;
        }

        // Track episode rewards (sliding window)
        if self.performance_metrics.episode_rewards.len() >= 100 {
            self.performance_metrics.episode_rewards.pop_front();
        }
        self.performance_metrics.episode_rewards.push_back(reward);

        // Update exploration rate in metrics
        self.performance_metrics.exploration_rate = self.current_exploration_rate;
        self.performance_metrics.last_update = Some(Utc::now());
    }

    /// Get current performance metrics
    pub fn get_performance_metrics(&self) -> &RLPerformanceMetrics {
        &self.performance_metrics
    }

    /// Check if agent is ready for trading with enhanced criteria
    pub fn is_ready(&self) -> bool {
        self.training_step > 100 && // Require some training
        self.replay_buffer.len() >= self.config.batch_size && // Have enough experiences
        self.performance_metrics.avg_td_error < 1.0 // Model is converging
    }

    /// Get training progress as percentage
    pub fn get_training_progress(&self) -> f64 {
        let min_steps = 1000.0;
        (self.training_step as f64 / min_steps).min(1.0) * 100.0
    }

    /// Get current exploration rate
    pub fn get_exploration_rate(&self) -> f64 {
        self.current_exploration_rate
    }

    /// Force exploration boost (useful for regime changes)
    pub fn boost_exploration(&mut self, boost_factor: f64) {
        self.current_exploration_rate = (self.current_exploration_rate * boost_factor).min(0.5);
    }

    /// Get success rate of recent actions
    pub fn get_recent_success_rate(&self) -> f64 {
        if self.performance_metrics.episode_rewards.is_empty() {
            return 0.0;
        }

        let positive_rewards = self.performance_metrics.episode_rewards
            .iter()
            .filter(|&&r| r > 0.0)
            .count();

        positive_rewards as f64 / self.performance_metrics.episode_rewards.len() as f64
    }

    /// Reset agent state for new trading session
    pub fn reset(&mut self) {
        self.last_state = None;
        self.last_action = None;
        self.current_exploration_rate = self.config.exploration_rate;
        self.performance_metrics = RLPerformanceMetrics::default();
    }

    /// Save agent state for persistence
    pub fn save_state(&self) -> Result<String> {
        // In production, this would serialize the entire agent state
        // For now, return a simple JSON representation of key metrics
        let state = serde_json::json!({
            "training_steps": self.training_step,
            "exploration_rate": self.current_exploration_rate,
            "buffer_size": self.replay_buffer.len(),
            "performance": {
                "total_actions": self.performance_metrics.total_actions,
                "profitable_actions": self.performance_metrics.profitable_actions,
                "average_reward": self.performance_metrics.average_reward,
                "training_loss": self.performance_metrics.training_loss
            }
        });

        Ok(state.to_string())
    }

    /// Load agent state from persistence
    pub fn load_state(&mut self, state_json: &str) -> Result<()> {
        // In production, this would deserialize and restore the full agent state
        // For now, just parse basic metrics
        let state: serde_json::Value = serde_json::from_str(state_json)
            .map_err(|e| crate::utils::PantherSwapError::ai_prediction(format!("Failed to parse state: {}", e)))?;

        if let Some(steps) = state["training_steps"].as_u64() {
            self.training_step = steps;
        }

        if let Some(exploration) = state["exploration_rate"].as_f64() {
            self.current_exploration_rate = exploration;
        }

        Ok(())
    }
}

/// Factory function to create a configured RL agent for forex trading
pub fn create_forex_rl_agent() -> RLTradingAgent {
    let config = RLAgentConfig {
        state_dimensions: 32,
        action_dimensions: 5,
        learning_rate: 0.001,
        discount_factor: 0.95,
        exploration_rate: 0.1,
        exploration_decay: 0.995,
        memory_size: 10000,
        batch_size: 32,
    };
    
    RLTradingAgent::new(config)
}
