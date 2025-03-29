#![cfg(feature = "text")]

use crate::{MlxError, ModelTrait, config::{BaseModelConfig, ModelConfig}};
use serde::{Deserialize, Serialize};
use std::{path::{Path, PathBuf}, sync::Arc};

/// Configuration for text generation models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextModelConfig {
    /// Base model configuration
    #[serde(flatten)]
    pub base: BaseModelConfig,
    
    /// Tokenizer path (if different from model path)
    pub tokenizer_path: Option<PathBuf>,
    
    /// Context size (max tokens)
    pub context_size: Option<usize>,
    
    /// Vocabulary size
    pub vocab_size: Option<usize>,
    
    /// Whether the model uses chat format
    pub is_chat_model: Option<bool>,
    
    /// Temperature for sampling
    pub temperature: Option<f32>,
    
    /// Top-p for nucleus sampling
    pub top_p: Option<f32>,
    
    /// Top-k for top-k sampling
    pub top_k: Option<u32>,
    
    /// Repetition penalty
    pub repetition_penalty: Option<f32>,
    
    /// System prompt for chat models
    pub system_prompt: Option<String>,
    
    /// Special tokens
    pub special_tokens: Option<SpecialTokens>,
}

impl Default for TextModelConfig {
    fn default() -> Self {
        Self {
            base: BaseModelConfig::default(),
            tokenizer_path: None,
            context_size: Some(4096),
            vocab_size: None,
            is_chat_model: Some(false),
            temperature: Some(0.7),
            top_p: Some(0.9),
            top_k: Some(40),
            repetition_penalty: Some(1.1),
            system_prompt: None,
            special_tokens: None,
        }
    }
}

impl ModelConfig for TextModelConfig {
    fn model_type(&self) -> &str {
        "text"
    }
    
    fn validate(&self) -> Result<(), MlxError> {
        self.base.validate()?;
        
        if let Some(ref tokenizer_path) = self.tokenizer_path {
            if !tokenizer_path.exists() {
                return Err(MlxError::ModelConfiguration(format!(
                    "Tokenizer path does not exist: {}",
                    tokenizer_path.display()
                )));
            }
        }
        
        Ok(())
    }
}

/// Special tokens for text models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialTokens {
    /// Beginning of sentence token
    pub bos_token: Option<String>,
    /// End of sentence token
    pub eos_token: Option<String>,
    /// Padding token
    pub pad_token: Option<String>,
    /// Unknown token
    pub unk_token: Option<String>,
    /// Separator token
    pub sep_token: Option<String>,
    /// Mask token
    pub mask_token: Option<String>,
    /// User message token
    pub user_token: Option<String>,
    /// Assistant message token
    pub assistant_token: Option<String>,
    /// System message token
    pub system_token: Option<String>,
}

impl Default for SpecialTokens {
    fn default() -> Self {
        Self {
            bos_token: Some("<s>".to_string()),
            eos_token: Some("</s>".to_string()),
            pad_token: Some("<pad>".to_string()),
            unk_token: Some("<unk>".to_string()),
            sep_token: None,
            mask_token: None,
            user_token: None,
            assistant_token: None,
            system_token: None,
        }
    }
}

/// Text generation parameters
#[derive(Debug, Clone)]
pub struct TextGenerationParams {
    /// Maximum number of tokens to generate
    pub max_tokens: usize,
    /// Temperature for sampling
    pub temperature: f32,
    /// Top-p for nucleus sampling
    pub top_p: f32,
    /// Top-k for top-k sampling
    pub top_k: u32,
    /// Repetition penalty
    pub repetition_penalty: f32,
    /// Whether to echo the prompt in the response
    pub echo: bool,
    /// Stop sequences (stop generating when encountered)
    pub stop: Vec<String>,
}

impl Default for TextGenerationParams {
    fn default() -> Self {
        Self {
            max_tokens: 256,
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            repetition_penalty: 1.1,
            echo: false,
            stop: Vec::new(),
        }
    }
}

/// Chat message role
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// System message
    System,
    /// User message
    User,
    /// Assistant message
    Assistant,
    /// Function call message
    Function,
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Message role
    pub role: MessageRole,
    /// Message content
    pub content: String,
}

/// Text generation struct with streaming support
#[derive(Debug, Clone)]
pub struct TextGeneration {
    /// Generated text
    pub text: String,
    /// Finished generating
    pub finished: bool,
    /// Usage statistics
    pub usage: Option<UsageInfo>,
}

/// Usage information for text generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageInfo {
    /// Number of prompt tokens
    pub prompt_tokens: usize,
    /// Number of completion tokens
    pub completion_tokens: usize,
    /// Total tokens
    pub total_tokens: usize,
}

/// Text model for text generation and chat
#[derive(Debug, Clone)]
pub struct TextModel {
    /// Model configuration
    pub config: TextModelConfig,
    /// Model path
    pub model_path: PathBuf,
    /// Model name
    pub name: String,
    /// Tokenizer path
    pub tokenizer_path: PathBuf,
    // Note: In a real implementation, this would contain the actual model
    // but for this placeholder, we'll just store the config
}

impl TextModel {
    /// Load a text model from the specified path
    pub async fn load_from_path(path: &Path, config: TextModelConfig) -> Result<Self, MlxError> {
        // Validate configuration
        config.validate()?;
        
        let model_path = path.to_path_buf();
        let tokenizer_path = config.tokenizer_path.clone().unwrap_or_else(|| model_path.clone());
        
        // In a real implementation, this would load the model weights
        // For now, just check that the paths exist
        if !model_path.exists() {
            return Err(MlxError::ModelLoading(format!(
                "Model path does not exist: {}",
                model_path.display()
            )));
        }
        
        if !tokenizer_path.exists() {
            return Err(MlxError::ModelLoading(format!(
                "Tokenizer path does not exist: {}",
                tokenizer_path.display()
            )));
        }
        
        Ok(Self {
            config: config.clone(),
            model_path,
            name: config.base.name.clone(),
            tokenizer_path,
        })
    }
    
    /// Generate text from a prompt
    pub async fn generate(&self, prompt: &str, params: TextGenerationParams) -> Result<TextGeneration, MlxError> {
        #[cfg(not(feature = "text"))]
        return Err(MlxError::UnsupportedFeature(
            "Text generation feature is not enabled".to_string()
        ));
        
        // In a real implementation, this would run inference with the model
        // For now, return a placeholder response
        let text = format!("This is a placeholder response for prompt: {}", prompt);
        
        Ok(TextGeneration {
            text,
            finished: true,
            usage: Some(UsageInfo {
                prompt_tokens: prompt.split_whitespace().count(),
                completion_tokens: text.split_whitespace().count(),
                total_tokens: prompt.split_whitespace().count() + text.split_whitespace().count(),
            }),
        })
    }
    
    /// Generate text from a chat conversation
    pub async fn chat_completion(
        &self,
        messages: &[ChatMessage],
        params: TextGenerationParams,
    ) -> Result<TextGeneration, MlxError> {
        // Convert chat messages to a prompt string
        let system_prompt = self.config.system_prompt.clone().unwrap_or_default();
        let mut prompt = String::new();
        
        if !system_prompt.is_empty() {
            prompt.push_str(&format!("System: {}\n\n", system_prompt));
        }
        
        for message in messages {
            match message.role {
                MessageRole::System => {
                    prompt.push_str(&format!("System: {}\n\n", message.content));
                }
                MessageRole::User => {
                    prompt.push_str(&format!("User: {}\n\n", message.content));
                }
                MessageRole::Assistant => {
                    prompt.push_str(&format!("Assistant: {}\n\n", message.content));
                }
                MessageRole::Function => {
                    prompt.push_str(&format!("Function: {}\n\n", message.content));
                }
            }
        }
        
        prompt.push_str("Assistant: ");
        
        // Generate text using the constructed prompt
        self.generate(&prompt, params).await
    }
}

impl ModelTrait for TextModel {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn model_type(&self) -> &str {
        "text"
    }
} 