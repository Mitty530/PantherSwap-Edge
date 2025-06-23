// AI Configuration Management and Hyperparameter Optimization
use crate::utils::Result;
use crate::ai::models::{ModelType, ModelMetadata, ModelPerformanceMetrics};
use crate::ai::time_series::LSTMConfig;
use crate::ai::rl_agent::RLAgentConfig;
use crate::ai::hmm_regime::HMMConfig;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use uuid::Uuid;
use tracing::{info, warn, debug};
use rand::Rng;

/// Configuration for hyperparameter optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    pub max_iterations: usize,
    pub population_size: usize,
    pub mutation_rate: f64,
    pub crossover_rate: f64,
    pub convergence_threshold: f64,
    pub early_stopping_patience: usize,
    pub optimization_timeout_minutes: u64,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            population_size: 20,
            mutation_rate: 0.1,
            crossover_rate: 0.8,
            convergence_threshold: 1e-6,
            early_stopping_patience: 10,
            optimization_timeout_minutes: 60,
        }
    }
}

/// Hyperparameter search space definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSpace {
    pub parameters: HashMap<String, ParameterRange>,
}

/// Parameter range for optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterRange {
    Float { min: f64, max: f64 },
    Integer { min: i32, max: i32 },
    Categorical { values: Vec<String> },
    Boolean,
}

/// Optimization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub best_parameters: HashMap<String, ParameterValue>,
    pub best_score: f64,
    pub iterations_completed: usize,
    pub convergence_history: Vec<f64>,
    pub optimization_time_seconds: f64,
    pub final_config: ModelConfig,
}

/// Parameter value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterValue {
    Float(f64),
    Integer(i32),
    String(String),
    Boolean(bool),
}

/// Unified model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelConfig {
    LSTM(LSTMConfig),
    RLAgent(RLAgentConfig),
    HMM(HMMConfig),
}

/// Individual in the optimization population
#[derive(Debug, Clone)]
struct Individual {
    parameters: HashMap<String, ParameterValue>,
    fitness: Option<f64>,
    config: Option<ModelConfig>,
}

/// Hyperparameter optimizer using genetic algorithm
pub struct HyperparameterOptimizer {
    config: OptimizationConfig,
    search_space: SearchSpace,
    model_type: ModelType,
    population: Vec<Individual>,
    best_individual: Option<Individual>,
    generation: usize,
    convergence_history: Vec<f64>,
}

impl HyperparameterOptimizer {
    /// Create a new hyperparameter optimizer
    pub fn new(
        config: OptimizationConfig,
        search_space: SearchSpace,
        model_type: ModelType,
    ) -> Self {
        Self {
            config,
            search_space,
            model_type,
            population: Vec::new(),
            best_individual: None,
            generation: 0,
            convergence_history: Vec::new(),
        }
    }
    
    /// Run hyperparameter optimization
    pub async fn optimize<F>(&mut self, fitness_function: F) -> Result<OptimizationResult>
    where
        F: Fn(&ModelConfig) -> f64 + Send + Sync,
    {
        let start_time = std::time::Instant::now();
        
        // Initialize population
        self.initialize_population();
        info!("Initialized population of {} individuals", self.config.population_size);
        
        let mut no_improvement_count = 0;
        let mut last_best_score = f64::NEG_INFINITY;
        
        for generation in 0..self.config.max_iterations {
            self.generation = generation;
            
            // Evaluate fitness for all individuals
            self.evaluate_population(&fitness_function).await?;
            
            // Track best individual
            let current_best = self.get_best_individual();
            let current_best_score = current_best.fitness.unwrap_or(f64::NEG_INFINITY);
            
            self.convergence_history.push(current_best_score);
            
            // Check for improvement
            if current_best_score > last_best_score + self.config.convergence_threshold {
                last_best_score = current_best_score;
                no_improvement_count = 0;
                self.best_individual = Some(current_best.clone());
                info!("Generation {}: New best score = {:.6}", generation, current_best_score);
            } else {
                no_improvement_count += 1;
            }
            
            // Early stopping
            if no_improvement_count >= self.config.early_stopping_patience {
                info!("Early stopping at generation {} (no improvement for {} generations)", 
                      generation, no_improvement_count);
                break;
            }
            
            // Check timeout
            if start_time.elapsed().as_secs() > self.config.optimization_timeout_minutes * 60 {
                warn!("Optimization timeout reached");
                break;
            }
            
            // Create next generation
            if generation < self.config.max_iterations - 1 {
                self.evolve_population();
            }
            
            debug!("Generation {} completed, best score: {:.6}", generation, current_best_score);
        }
        
        let optimization_time = start_time.elapsed().as_secs_f64();
        
        // Build result
        let best = self.best_individual.as_ref()
            .ok_or_else(|| crate::utils::PantherSwapError::ai_prediction("No best individual found".to_string()))?;
        
        let result = OptimizationResult {
            best_parameters: best.parameters.clone(),
            best_score: best.fitness.unwrap_or(0.0),
            iterations_completed: self.generation + 1,
            convergence_history: self.convergence_history.clone(),
            optimization_time_seconds: optimization_time,
            final_config: best.config.as_ref().unwrap().clone(),
        };
        
        info!("Optimization completed in {:.2}s, best score: {:.6}", 
              optimization_time, result.best_score);
        
        Ok(result)
    }
    
    /// Initialize random population
    fn initialize_population(&mut self) {
        self.population.clear();
        
        for _ in 0..self.config.population_size {
            let parameters = self.generate_random_parameters();
            let individual = Individual {
                parameters,
                fitness: None,
                config: None,
            };
            self.population.push(individual);
        }
    }
    
    /// Generate random parameters within search space
    fn generate_random_parameters(&self) -> HashMap<String, ParameterValue> {
        let mut rng = rand::thread_rng();
        let mut parameters = HashMap::new();
        
        for (param_name, param_range) in &self.search_space.parameters {
            let value = match param_range {
                ParameterRange::Float { min, max } => {
                    ParameterValue::Float(rng.gen_range(*min..=*max))
                }
                ParameterRange::Integer { min, max } => {
                    ParameterValue::Integer(rng.gen_range(*min..=*max))
                }
                ParameterRange::Categorical { values } => {
                    let idx = rng.gen_range(0..values.len());
                    ParameterValue::String(values[idx].clone())
                }
                ParameterRange::Boolean => {
                    ParameterValue::Boolean(rng.gen_bool(0.5))
                }
            };
            parameters.insert(param_name.clone(), value);
        }
        
        parameters
    }
    
    /// Evaluate fitness for all individuals in population
    async fn evaluate_population<F>(&mut self, fitness_function: &F) -> Result<()>
    where
        F: Fn(&ModelConfig) -> f64,
    {
        let mut configs_to_evaluate = Vec::new();

        // First pass: collect parameters that need evaluation
        for (i, individual) in self.population.iter().enumerate() {
            if individual.fitness.is_none() {
                let config = self.parameters_to_config(&individual.parameters)?;
                configs_to_evaluate.push((i, config));
            }
        }

        // Second pass: evaluate and update
        for (i, config) in configs_to_evaluate {
            let fitness = fitness_function(&config);
            self.population[i].fitness = Some(fitness);
            self.population[i].config = Some(config);
        }

        Ok(())
    }
    
    /// Convert parameters to model configuration
    fn parameters_to_config(&self, parameters: &HashMap<String, ParameterValue>) -> Result<ModelConfig> {
        match self.model_type {
            ModelType::LSTM => {
                let config = LSTMConfig {
                    sequence_length: self.get_int_param(parameters, "sequence_length").unwrap_or(128),
                    feature_dimensions: self.get_int_param(parameters, "feature_dimensions").unwrap_or(16),
                    hidden_size: self.get_int_param(parameters, "hidden_size").unwrap_or(256),
                    num_layers: self.get_int_param(parameters, "num_layers").unwrap_or(3),
                    dropout_rate: self.get_float_param(parameters, "dropout_rate").unwrap_or(0.1),
                    prediction_horizons: vec![60, 300, 900, 3600], // 1min, 5min, 15min, 1hour
                    learning_rate: self.get_float_param(parameters, "learning_rate").unwrap_or(0.001),
                };
                Ok(ModelConfig::LSTM(config))
            }
            ModelType::RLAgent => {
                let config = RLAgentConfig {
                    state_dimensions: self.get_int_param(parameters, "state_dimensions").unwrap_or(10),
                    action_dimensions: self.get_int_param(parameters, "action_dimensions").unwrap_or(3),
                    learning_rate: self.get_float_param(parameters, "learning_rate").unwrap_or(0.001),
                    discount_factor: self.get_float_param(parameters, "discount_factor").unwrap_or(0.99),
                    exploration_rate: self.get_float_param(parameters, "exploration_rate").unwrap_or(0.1),
                    exploration_decay: self.get_float_param(parameters, "exploration_decay").unwrap_or(0.995),
                    memory_size: self.get_int_param(parameters, "memory_size").unwrap_or(10000),
                    batch_size: self.get_int_param(parameters, "batch_size").unwrap_or(32),
                };
                Ok(ModelConfig::RLAgent(config))
            }
            ModelType::HMMRegime => {
                let mut config = HMMConfig::default();

                // Override with provided parameters
                if let Some(num_states) = self.get_int_param(parameters, "num_states") {
                    config.num_states = num_states;
                }
                if let Some(observation_window) = self.get_int_param(parameters, "observation_window") {
                    config.observation_window = observation_window;
                }
                if let Some(feature_dimensions) = self.get_int_param(parameters, "feature_dimensions") {
                    config.feature_dimensions = feature_dimensions;
                }
                if let Some(convergence_threshold) = self.get_float_param(parameters, "convergence_threshold") {
                    config.convergence_threshold = convergence_threshold;
                }
                if let Some(max_iterations) = self.get_int_param(parameters, "max_iterations") {
                    config.max_iterations = max_iterations;
                }
                if let Some(min_confidence) = self.get_float_param(parameters, "min_confidence") {
                    config.min_confidence = min_confidence;
                }

                Ok(ModelConfig::HMM(config))
            }
        }
    }
    
    /// Helper to get float parameter
    fn get_float_param(&self, parameters: &HashMap<String, ParameterValue>, name: &str) -> Option<f64> {
        parameters.get(name).and_then(|v| match v {
            ParameterValue::Float(f) => Some(*f),
            _ => None,
        })
    }
    
    /// Helper to get integer parameter
    fn get_int_param(&self, parameters: &HashMap<String, ParameterValue>, name: &str) -> Option<usize> {
        parameters.get(name).and_then(|v| match v {
            ParameterValue::Integer(i) => Some(*i as usize),
            _ => None,
        })
    }
    
    /// Get best individual from current population
    fn get_best_individual(&self) -> Individual {
        self.population.iter()
            .filter(|ind| ind.fitness.is_some())
            .max_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap_or(std::cmp::Ordering::Equal))
            .cloned()
            .unwrap_or_else(|| self.population[0].clone())
    }
    
    /// Evolve population using genetic algorithm
    fn evolve_population(&mut self) {
        let mut new_population = Vec::new();
        let mut rng = rand::thread_rng();
        
        // Sort population by fitness
        self.population.sort_by(|a, b| {
            b.fitness.partial_cmp(&a.fitness).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Elitism: keep best individuals
        let elite_count = (self.config.population_size as f64 * 0.1) as usize;
        for i in 0..elite_count {
            new_population.push(self.population[i].clone());
        }
        
        // Generate offspring
        while new_population.len() < self.config.population_size {
            // Tournament selection
            let parent1 = self.tournament_selection();
            let parent2 = self.tournament_selection();
            
            // Crossover
            let mut offspring = if rng.gen::<f64>() < self.config.crossover_rate {
                self.crossover(&parent1, &parent2)
            } else {
                parent1.clone()
            };
            
            // Mutation
            if rng.gen::<f64>() < self.config.mutation_rate {
                self.mutate(&mut offspring);
            }
            
            // Reset fitness for new individual
            offspring.fitness = None;
            offspring.config = None;
            
            new_population.push(offspring);
        }
        
        self.population = new_population;
    }
    
    /// Tournament selection
    fn tournament_selection(&self) -> Individual {
        let mut rng = rand::thread_rng();
        let tournament_size = 3;
        
        let mut best = &self.population[rng.gen_range(0..self.population.len())];
        
        for _ in 1..tournament_size {
            let candidate = &self.population[rng.gen_range(0..self.population.len())];
            if candidate.fitness.unwrap_or(f64::NEG_INFINITY) > best.fitness.unwrap_or(f64::NEG_INFINITY) {
                best = candidate;
            }
        }
        
        best.clone()
    }
    
    /// Crossover two individuals
    fn crossover(&self, parent1: &Individual, parent2: &Individual) -> Individual {
        let mut rng = rand::thread_rng();
        let mut offspring_params = HashMap::new();
        
        for (param_name, _) in &self.search_space.parameters {
            let value = if rng.gen_bool(0.5) {
                parent1.parameters.get(param_name).cloned()
            } else {
                parent2.parameters.get(param_name).cloned()
            };
            
            if let Some(v) = value {
                offspring_params.insert(param_name.clone(), v);
            }
        }
        
        Individual {
            parameters: offspring_params,
            fitness: None,
            config: None,
        }
    }
    
    /// Mutate an individual
    fn mutate(&self, individual: &mut Individual) {
        let mut rng = rand::thread_rng();
        
        for (param_name, param_range) in &self.search_space.parameters {
            if rng.gen::<f64>() < 0.1 { // 10% chance to mutate each parameter
                let new_value = match param_range {
                    ParameterRange::Float { min, max } => {
                        ParameterValue::Float(rng.gen_range(*min..=*max))
                    }
                    ParameterRange::Integer { min, max } => {
                        ParameterValue::Integer(rng.gen_range(*min..=*max))
                    }
                    ParameterRange::Categorical { values } => {
                        let idx = rng.gen_range(0..values.len());
                        ParameterValue::String(values[idx].clone())
                    }
                    ParameterRange::Boolean => {
                        ParameterValue::Boolean(rng.gen_bool(0.5))
                    }
                };
                individual.parameters.insert(param_name.clone(), new_value);
            }
        }
    }
}

/// Create default search spaces for different model types
pub fn create_default_search_space(model_type: ModelType) -> SearchSpace {
    let mut parameters = HashMap::new();
    
    match model_type {
        ModelType::LSTM => {
            parameters.insert("hidden_size".to_string(), ParameterRange::Integer { min: 32, max: 256 });
            parameters.insert("num_layers".to_string(), ParameterRange::Integer { min: 1, max: 4 });
            parameters.insert("dropout".to_string(), ParameterRange::Float { min: 0.0, max: 0.5 });
            parameters.insert("learning_rate".to_string(), ParameterRange::Float { min: 0.0001, max: 0.01 });
            parameters.insert("batch_size".to_string(), ParameterRange::Integer { min: 16, max: 128 });
            parameters.insert("sequence_length".to_string(), ParameterRange::Integer { min: 20, max: 100 });
        }
        ModelType::RLAgent => {
            parameters.insert("learning_rate".to_string(), ParameterRange::Float { min: 0.0001, max: 0.01 });
            parameters.insert("discount_factor".to_string(), ParameterRange::Float { min: 0.9, max: 0.999 });
            parameters.insert("exploration_rate".to_string(), ParameterRange::Float { min: 0.01, max: 0.3 });
            parameters.insert("exploration_decay".to_string(), ParameterRange::Float { min: 0.99, max: 0.999 });
            parameters.insert("memory_size".to_string(), ParameterRange::Integer { min: 1000, max: 50000 });
            parameters.insert("batch_size".to_string(), ParameterRange::Integer { min: 16, max: 128 });
        }
        ModelType::HMMRegime => {
            parameters.insert("num_states".to_string(), ParameterRange::Integer { min: 2, max: 8 });
            parameters.insert("observation_window".to_string(), ParameterRange::Integer { min: 20, max: 200 });
            parameters.insert("convergence_threshold".to_string(), ParameterRange::Float { min: 1e-8, max: 1e-4 });
            parameters.insert("min_confidence".to_string(), ParameterRange::Float { min: 0.5, max: 0.9 });
        }
    }
    
    SearchSpace { parameters }
}

/// Factory function to create a hyperparameter optimizer
pub fn create_hyperparameter_optimizer(model_type: ModelType) -> HyperparameterOptimizer {
    let config = OptimizationConfig::default();
    let search_space = create_default_search_space(model_type.clone());
    HyperparameterOptimizer::new(config, search_space, model_type)
}
