// AI Models Management and Serialization
use crate::utils::{Result, PantherSwapError};
use crate::ai::time_series::{LSTMTimeSeriesModel, LSTMConfig};
use crate::ai::rl_agent::{RLTradingAgent, RLAgentConfig};
use crate::ai::hmm_regime::{HMMRegimeDetector, HMMConfig};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use uuid::Uuid;
use tracing::{info, warn, error};

/// Model metadata for tracking and management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub model_id: Uuid,
    pub model_type: ModelType,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub training_data_hash: Option<String>,
    pub performance_metrics: ModelPerformanceMetrics,
    pub config_hash: String,
    pub file_path: PathBuf,
}

/// Types of AI models supported
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModelType {
    LSTM,
    RLAgent,
    HMMRegime,
}

/// Performance metrics for model tracking
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelPerformanceMetrics {
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
    pub inference_latency_ms: f64,
    pub training_loss: f64,
    pub validation_loss: f64,
    pub total_predictions: u64,
    pub successful_predictions: u64,
    pub last_evaluation: Option<DateTime<Utc>>,
}

/// Model configuration for different types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelConfig {
    LSTM(LSTMConfig),
    RLAgent(RLAgentConfig),
    HMMRegime(HMMConfig),
}

/// Serializable model state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableModelState {
    pub metadata: ModelMetadata,
    pub config: ModelConfig,
    pub model_data: Vec<u8>, // Serialized model weights/parameters
}

/// Model registry for managing all AI models
pub struct ModelRegistry {
    models: HashMap<Uuid, ModelMetadata>,
    model_storage_path: PathBuf,
    active_models: HashMap<Uuid, ActiveModel>,
}

/// Active model wrapper
pub enum ActiveModel {
    LSTM(LSTMTimeSeriesModel),
    RLAgent(RLTradingAgent),
    HMMRegime(HMMRegimeDetector),
}

impl ModelRegistry {
    /// Create a new model registry
    pub fn new(storage_path: PathBuf) -> Result<Self> {
        // Ensure storage directory exists
        if !storage_path.exists() {
            fs::create_dir_all(&storage_path)
                .map_err(|e| PantherSwapError::ai_prediction(format!("Failed to create model storage directory: {}", e)))?;
        }

        let mut registry = Self {
            models: HashMap::new(),
            model_storage_path: storage_path,
            active_models: HashMap::new(),
        };

        // Load existing models
        registry.load_existing_models()?;

        Ok(registry)
    }

    /// Register a new LSTM model
    pub fn register_lstm_model(&mut self, model: LSTMTimeSeriesModel, config: LSTMConfig) -> Result<Uuid> {
        let model_id = Uuid::new_v4();
        let metadata = ModelMetadata {
            model_id,
            model_type: ModelType::LSTM,
            version: "1.0.0".to_string(),
            created_at: Utc::now(),
            last_updated: Utc::now(),
            training_data_hash: None,
            performance_metrics: ModelPerformanceMetrics::default(),
            config_hash: self.calculate_config_hash(&ModelConfig::LSTM(config.clone())),
            file_path: self.get_model_file_path(model_id),
        };

        // Save model to disk
        self.save_model_to_disk(&metadata, &ModelConfig::LSTM(config), &model)?;

        // Register in memory
        self.models.insert(model_id, metadata);
        self.active_models.insert(model_id, ActiveModel::LSTM(model));

        info!("Registered LSTM model with ID: {}", model_id);
        Ok(model_id)
    }

    /// Register a new RL agent
    pub fn register_rl_agent(&mut self, agent: RLTradingAgent, config: RLAgentConfig) -> Result<Uuid> {
        let model_id = Uuid::new_v4();
        let metadata = ModelMetadata {
            model_id,
            model_type: ModelType::RLAgent,
            version: "1.0.0".to_string(),
            created_at: Utc::now(),
            last_updated: Utc::now(),
            training_data_hash: None,
            performance_metrics: ModelPerformanceMetrics::default(),
            config_hash: self.calculate_config_hash(&ModelConfig::RLAgent(config.clone())),
            file_path: self.get_model_file_path(model_id),
        };

        // Save model to disk
        self.save_model_to_disk(&metadata, &ModelConfig::RLAgent(config), &agent)?;

        // Register in memory
        self.models.insert(model_id, metadata);
        self.active_models.insert(model_id, ActiveModel::RLAgent(agent));

        info!("Registered RL agent with ID: {}", model_id);
        Ok(model_id)
    }

    /// Register a new HMM regime detector
    pub fn register_hmm_detector(&mut self, detector: HMMRegimeDetector, config: HMMConfig) -> Result<Uuid> {
        let model_id = Uuid::new_v4();
        let metadata = ModelMetadata {
            model_id,
            model_type: ModelType::HMMRegime,
            version: "1.0.0".to_string(),
            created_at: Utc::now(),
            last_updated: Utc::now(),
            training_data_hash: None,
            performance_metrics: ModelPerformanceMetrics::default(),
            config_hash: self.calculate_config_hash(&ModelConfig::HMMRegime(config.clone())),
            file_path: self.get_model_file_path(model_id),
        };

        // Save model to disk
        self.save_model_to_disk(&metadata, &ModelConfig::HMMRegime(config), &detector)?;

        // Register in memory
        self.models.insert(model_id, metadata);
        self.active_models.insert(model_id, ActiveModel::HMMRegime(detector));

        info!("Registered HMM regime detector with ID: {}", model_id);
        Ok(model_id)
    }

    /// Get model metadata by ID
    pub fn get_model_metadata(&self, model_id: Uuid) -> Option<&ModelMetadata> {
        self.models.get(&model_id)
    }

    /// Get active model by ID
    pub fn get_active_model(&self, model_id: Uuid) -> Option<&ActiveModel> {
        self.active_models.get(&model_id)
    }

    /// Get mutable active model by ID
    pub fn get_active_model_mut(&mut self, model_id: Uuid) -> Option<&mut ActiveModel> {
        self.active_models.get_mut(&model_id)
    }

    /// Update model performance metrics
    pub fn update_model_performance(&mut self, model_id: Uuid, metrics: ModelPerformanceMetrics) -> Result<()> {
        if let Some(metadata) = self.models.get_mut(&model_id) {
            metadata.performance_metrics = metrics;
            metadata.last_updated = Utc::now();

            info!("Updated performance metrics for model: {}", model_id);
            Ok(())
        } else {
            Err(PantherSwapError::ai_prediction(format!("Model not found: {}", model_id)))
        }
    }

    /// List all registered models
    pub fn list_models(&self) -> Vec<&ModelMetadata> {
        self.models.values().collect()
    }

    /// List models by type
    pub fn list_models_by_type(&self, model_type: ModelType) -> Vec<&ModelMetadata> {
        self.models.values()
            .filter(|metadata| metadata.model_type == model_type)
            .collect()
    }

    /// Remove a model from registry
    pub fn remove_model(&mut self, model_id: Uuid) -> Result<()> {
        if let Some(metadata) = self.models.remove(&model_id) {
            // Remove from active models
            self.active_models.remove(&model_id);

            // Remove file from disk
            if metadata.file_path.exists() {
                fs::remove_file(&metadata.file_path)
                    .map_err(|e| PantherSwapError::ai_prediction(format!("Failed to remove model file: {}", e)))?;
            }

            info!("Removed model: {}", model_id);
            Ok(())
        } else {
            Err(PantherSwapError::ai_prediction(format!("Model not found: {}", model_id)))
        }
    }

    /// Load existing models from disk
    fn load_existing_models(&mut self) -> Result<()> {
        if !self.model_storage_path.exists() {
            return Ok(());
        }

        let entries = fs::read_dir(&self.model_storage_path)
            .map_err(|e| PantherSwapError::ai_prediction(format!("Failed to read model directory: {}", e)))?;

        for entry in entries {
            let entry = entry.map_err(|e| PantherSwapError::ai_prediction(format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.load_model_from_disk(&path) {
                    Ok((metadata, _config, _model_data)) => {
                        let model_id = metadata.model_id;
                        self.models.insert(model_id, metadata);
                        info!("Loaded model metadata: {}", model_id);
                    }
                    Err(e) => {
                        warn!("Failed to load model from {}: {}", path.display(), e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Save model metadata to disk (simplified approach)
    fn save_model_to_disk<T>(&self, metadata: &ModelMetadata, config: &ModelConfig, _model: &T) -> Result<()> {
        // For MVP, we'll only save metadata and config, not the actual model weights
        // In production, this would include proper model serialization
        let model_state = SerializableModelState {
            metadata: metadata.clone(),
            config: config.clone(),
            model_data: vec![], // Empty for now - would contain serialized model in production
        };

        let json_data = serde_json::to_string_pretty(&model_state)
            .map_err(|e| PantherSwapError::ai_prediction(format!("Failed to serialize model state: {}", e)))?;

        fs::write(&metadata.file_path, json_data)
            .map_err(|e| PantherSwapError::ai_prediction(format!("Failed to write model file: {}", e)))?;

        Ok(())
    }

    /// Load model from disk
    fn load_model_from_disk(&self, file_path: &Path) -> Result<(ModelMetadata, ModelConfig, Vec<u8>)> {
        let json_data = fs::read_to_string(file_path)
            .map_err(|e| PantherSwapError::ai_prediction(format!("Failed to read model file: {}", e)))?;

        let model_state: SerializableModelState = serde_json::from_str(&json_data)
            .map_err(|e| PantherSwapError::ai_prediction(format!("Failed to deserialize model state: {}", e)))?;

        Ok((model_state.metadata, model_state.config, model_state.model_data))
    }



    /// Calculate configuration hash for change detection
    fn calculate_config_hash(&self, config: &ModelConfig) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let serialized = serde_json::to_string(config).unwrap_or_default();
        let mut hasher = DefaultHasher::new();
        serialized.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Get file path for model storage
    fn get_model_file_path(&self, model_id: Uuid) -> PathBuf {
        self.model_storage_path.join(format!("{}.json", model_id))
    }

    /// Get model statistics
    pub fn get_model_statistics(&self) -> ModelRegistryStatistics {
        let total_models = self.models.len();
        let active_models = self.active_models.len();

        let mut by_type = HashMap::new();
        for metadata in self.models.values() {
            *by_type.entry(metadata.model_type.clone()).or_insert(0) += 1;
        }

        let avg_accuracy = self.models.values()
            .map(|m| m.performance_metrics.accuracy)
            .filter(|&acc| acc > 0.0)
            .collect::<Vec<_>>();

        let average_accuracy = if !avg_accuracy.is_empty() {
            avg_accuracy.iter().sum::<f64>() / avg_accuracy.len() as f64
        } else {
            0.0
        };

        ModelRegistryStatistics {
            total_models,
            active_models,
            models_by_type: by_type,
            average_accuracy,
            last_updated: Utc::now(),
        }
    }
}

/// Statistics about the model registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRegistryStatistics {
    pub total_models: usize,
    pub active_models: usize,
    pub models_by_type: HashMap<ModelType, usize>,
    pub average_accuracy: f64,
    pub last_updated: DateTime<Utc>,
}

/// Factory function to create a model registry with default storage path
pub fn create_model_registry() -> Result<ModelRegistry> {
    let storage_path = std::env::current_dir()
        .map_err(|e| PantherSwapError::ai_prediction(format!("Failed to get current directory: {}", e)))?
        .join("models");

    ModelRegistry::new(storage_path)
}
