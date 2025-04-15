use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::MlxError;

/// Common trait for all model configurations
pub trait ModelConfig {
    /// Get the model type
    fn model_type(&self) -> &str;
    
    /// Validate the configuration
    fn validate(&self) -> Result<(), MlxError>;
}

/// Device configuration for model inference
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceConfig {
    /// CPU device
    Cpu,
    /// GPU/Metal device (applicable on macOS)
    Metal,
    /// Automatically select the best available device
    Auto,
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self::Auto
    }
}

/// Format configuration for model input/output
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelFormat {
    /// GGUF format (commonly used for LLaMA models)
    Gguf,
    /// ONNX format
    Onnx,
    /// MLX native format
    Mlx,
    /// PyTorch format
    PyTorch,
    /// TensorFlow format
    TensorFlow,
    /// Other format
    Other(String),
}

impl Default for ModelFormat {
    fn default() -> Self {
        Self::Gguf
    }
}

/// Quantization level for models
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QuantizationLevel {
    /// No quantization (FP32)
    None,
    /// BF16 precision
    BF16,
    /// FP16 precision
    F16,
    /// 8-bit quantization
    Q8_0,
    /// 4-bit quantization
    Q4_0,
    /// 4-bit quantization with better quality
    Q4_1,
    /// 5-bit quantization
    Q5_0,
    /// 5-bit quantization with better quality
    Q5_1,
    /// 2-bit quantization
    Q2_K,
    /// 3-bit quantization
    Q3_K,
    /// 4-bit quantization with K-means
    Q4_K,
    /// 5-bit quantization with K-means
    Q5_K,
    /// 6-bit quantization with K-means
    Q6_K,
    /// 8-bit quantization with K-means
    Q8_K,
}

impl Default for QuantizationLevel {
    fn default() -> Self {
        Self::None
    }
}

/// Base configuration shared by all model types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseModelConfig {
    /// Model name
    pub name: String,
    /// Path to the model file or directory
    pub path: Option<PathBuf>,
    /// Model format
    pub format: ModelFormat,
    /// Device to run the model on
    pub device: DeviceConfig,
    /// Quantization level
    pub quantization: QuantizationLevel,
    /// Number of threads to use for CPU inference
    pub num_threads: Option<usize>,
    /// Whether to use memory mapping for loading
    pub memory_map: Option<bool>,
    /// Custom parameters
    #[serde(flatten)]
    pub custom_params: std::collections::HashMap<String, serde_json::Value>,
}

impl Default for BaseModelConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            path: None,
            format: ModelFormat::default(),
            device: DeviceConfig::default(),
            quantization: QuantizationLevel::default(),
            num_threads: None,
            memory_map: Some(true),
            custom_params: std::collections::HashMap::new(),
        }
    }
}

impl BaseModelConfig {
    /// Validate the base configuration
    pub fn validate(&self) -> Result<(), MlxError> {
        if self.name.is_empty() {
            return Err(MlxError::ModelConfiguration("Model name cannot be empty".to_string()));
        }
        
        if let Some(ref path) = self.path {
            if !path.exists() {
                return Err(MlxError::ModelConfiguration(format!(
                    "Model path does not exist: {}",
                    path.display()
                )));
            }
        }
        
        Ok(())
    }
}

/// Loads model configuration from a JSON file
pub fn load_model_config_from_file<T: for<'de> Deserialize<'de>>(path: &std::path::Path) -> Result<T, MlxError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| MlxError::ModelConfiguration(format!("Failed to read config file: {}", e)))?;
    
    serde_json::from_str(&content)
        .map_err(|e| MlxError::ModelConfiguration(format!("Failed to parse config JSON: {}", e)))
}

/// Saves model configuration to a JSON file
pub fn save_model_config_to_file<T: Serialize>(config: &T, path: &std::path::Path) -> Result<(), MlxError> {
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| MlxError::ModelConfiguration(format!("Failed to serialize config: {}", e)))?;
    
    std::fs::write(path, content)
        .map_err(|e| MlxError::ModelConfiguration(format!("Failed to write config file: {}", e)))
} 