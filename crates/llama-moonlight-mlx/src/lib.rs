//! MLX integration for AI-powered browser automation in Llama-Moonlight
//!
//! This crate provides integration with MLX (a machine learning framework) for
//! browser automation. It enables AI-powered features like text generation,
//! image recognition, and decision making for automation scenarios.
//!
//! Note: Currently, this uses candle-core as a placeholder for MLX integration,
//! as MLX is primarily available on macOS/Apple Silicon.

use anyhow::Result;
use log::{debug, error, info, warn};
use llama_moonlight_core::{
    Browser, BrowserContext, Page,
    options::{BrowserOptions, ContextOptions, PageOptions},
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use thiserror::Error;

pub mod text;
pub mod vision;
pub mod agent;
pub mod config;
pub mod utils;

#[cfg(feature = "text")]
pub use text::{TextModel, TextModelConfig, TextGeneration, ChatMessage};

#[cfg(feature = "vision")]
pub use vision::{VisionModel, VisionModelConfig, ImageClassification, ObjectDetection};

pub use agent::{Agent, AgentConfig, AgentAction, AgentObservation};
pub use config::ModelConfig;

/// MLX-related errors
#[derive(Error, Debug)]
pub enum MlxError {
    #[error("Model loading error: {0}")]
    ModelLoading(String),
    
    #[error("Model inference error: {0}")]
    ModelInference(String),
    
    #[error("Text generation error: {0}")]
    TextGeneration(String),
    
    #[error("Vision processing error: {0}")]
    VisionProcessing(String),
    
    #[error("Model configuration error: {0}")]
    ModelConfiguration(String),
    
    #[error("Agent error: {0}")]
    Agent(String),
    
    #[error("Tokenization error: {0}")]
    Tokenization(String),
    
    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Image processing error: {0}")]
    ImageProcessing(String),
    
    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),
    
    #[error("Other error: {0}")]
    Other(String),
}

/// The main MLX interface for Llama-Moonlight
pub struct Mlx {
    /// Path to the models directory
    models_dir: PathBuf,
    /// Models cache
    models: std::collections::HashMap<String, Arc<dyn ModelTrait>>,
}

impl Mlx {
    /// Create a new Mlx instance
    pub fn new(models_dir: impl AsRef<Path>) -> Self {
        Self {
            models_dir: models_dir.as_ref().to_path_buf(),
            models: std::collections::HashMap::new(),
        }
    }
    
    /// Get the models directory
    pub fn models_dir(&self) -> &Path {
        &self.models_dir
    }
    
    /// Load a text model for text generation and LLM tasks
    #[cfg(feature = "text")]
    pub async fn load_text_model(&mut self, model_name: &str, config: TextModelConfig) -> Result<Arc<TextModel>, MlxError> {
        if let Some(model) = self.models.get(model_name) {
            // Check if it's a TextModel
            if let Some(text_model) = model.as_any().downcast_ref::<TextModel>() {
                return Ok(Arc::new(text_model.clone()));
            } else {
                return Err(MlxError::ModelLoading(format!(
                    "Model '{}' exists but is not a text model",
                    model_name
                )));
            }
        }
        
        let model_path = self.models_dir.join(model_name);
        let model = TextModel::load_from_path(&model_path, config).await?;
        let model_arc = Arc::new(model);
        self.models.insert(model_name.to_string(), model_arc.clone() as Arc<dyn ModelTrait>);
        
        Ok(model_arc)
    }
    
    /// Load a vision model for image recognition tasks
    #[cfg(feature = "vision")]
    pub async fn load_vision_model(&mut self, model_name: &str, config: VisionModelConfig) -> Result<Arc<VisionModel>, MlxError> {
        if let Some(model) = self.models.get(model_name) {
            // Check if it's a VisionModel
            if let Some(vision_model) = model.as_any().downcast_ref::<VisionModel>() {
                return Ok(Arc::new(vision_model.clone()));
            } else {
                return Err(MlxError::ModelLoading(format!(
                    "Model '{}' exists but is not a vision model",
                    model_name
                )));
            }
        }
        
        let model_path = self.models_dir.join(model_name);
        let model = VisionModel::load_from_path(&model_path, config).await?;
        let model_arc = Arc::new(model);
        self.models.insert(model_name.to_string(), model_arc.clone() as Arc<dyn ModelTrait>);
        
        Ok(model_arc)
    }
    
    /// Create an agent for autonomous browser automation
    pub async fn create_agent(&self, config: AgentConfig, page: Arc<Page>) -> Result<Agent, MlxError> {
        let agent = Agent::new(config, page);
        Ok(agent)
    }
}

/// Trait for models
pub trait ModelTrait: Send + Sync {
    /// Convert to Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any;
    
    /// Get the model name
    fn name(&self) -> &str;
    
    /// Get the model type
    fn model_type(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mlx_new() {
        let mlx = Mlx::new("/tmp/models");
        assert_eq!(mlx.models_dir(), Path::new("/tmp/models"));
        assert_eq!(mlx.models.len(), 0);
    }
} 