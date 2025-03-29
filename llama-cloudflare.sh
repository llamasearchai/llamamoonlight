#!/bin/bash

# Complete Full-Length Shell Script with llamaflare Program Code
# Self-installs, Self-Tests, and Runs llamaflare on macOS

set -e # Exit immediately if a command exits with a non-zero status

# --- 1. Initial Setup and Checks ---

echo "--- Starting llamaflare Self-Installation & Test Script ---"

# Check if Rust and Cargo are installed
if ! command -v rustc &> /dev/null || ! command -v cargo &> /dev/null; then
    echo "Error: Rust and Cargo are required to build and run llamaflare."
    echo "Please install Rust using rustup: https://rustup.rs/"
    exit 1
fi

echo "Rust and Cargo are installed. Proceeding..."

# Get the directory where the script is located
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
echo "Script directory: ${SCRIPT_DIR}"
cd "${SCRIPT_DIR}" || exit 1 # Change to script directory, exit if fails

# Create llamaflare project directory
PROJECT_NAME="llamaflare_temp_project"
rm -rf "${PROJECT_NAME}" # Remove if exists from previous runs
mkdir "${PROJECT_NAME}"
cd "${PROJECT_NAME}" || exit 1

# --- 2. Create Directory Structure and Write Rust Code Files ---

echo "--- Creating Directory Structure and Writing Rust Code ---"

# Create directories
mkdir -p src
mkdir -p src/caching
mkdir -p src/cli
mkdir -p src/cloudflare_bypass
mkdir -p src/generation
mkdir -p src/image_proc
mkdir -p src/models
mkdir -p src/proxy_manager
mkdir -p src/rate_limiting
mkdir -p src/tokenizer
mkdir -p src/cli

# Write Cargo.toml
cat > Cargo.toml << EOF
[package]
name = "llamaflare"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your_email@example.com>"] # Replace with your info
license = "MIT"
description = "Rust crate for advanced web scraping, including Cloudflare bypass."
repository = "https://github.com/YourUsername/llamaflare" # Replace with your repo
readme = "README.md"

[dependencies]
candle-core = "0.3"
candle-nn = "0.3"
tokenizers = "0.14"
image = "0.24"
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
thiserror = "1.0"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["gzip", "brotli", "rustls-tls"] } # HTTP Client
lru-cache = "0.8" # Caching
indicatif = "0.18" # Progress bars
log = "0.4"
tracing = "0.1"
tracing-subscriber = "0.3"
regex = "1.10" # For HTML parsing and challenge extraction
select = "0.7" # CSS selector for HTML parsing
rand = "0.8"
futures-util = "0.3" # Added for file_download.rs
EOF

# Write README.md
cat > README.md << EOF
# llamaflare

## Overview

\`llamaflare\` is a Rust crate designed to simplify web scraping tasks, with a focus on bypassing Cloudflare's anti-bot protections. It provides features such as proxy management, request rate limiting, response caching, and more. This crate is intended for research, development, and other appropriate web scraping applications.

## Features

*   **Cloudflare Bypassing:** Automatically handles Cloudflare's "I'm Under Attack Mode" (IUAM) challenges and Captchas using a combination of Javascript interpretation, and optional integration with 3rd party captcha solvers.
*   **Proxy Management:** Supports proxy rotation, health checks, and management.
*   **Rate Limiting:** Intelligent request throttling to prevent overloading target servers.
*   **Caching:** Response caching for efficiency and reduced server load.
*   **Asynchronous Requests:** Leverages \`tokio\` for parallel HTTP requests.
*   **File Downloads:** Download files with progress tracking.
*   **Custom Headers:** Supports custom headers for requests.
*   **Flexible Configuration:** Allows customization through configuration files or programmatic options.

## Installation

\`\`\`bash
cargo add llamaflare
\`\`\`

or, to add the crate to your project using the path:

\`\`\`bash
cargo add --path /path/to/llamaflare
\`\`\`

### Basic Usage

\`\`\`rust
use llamaflare::LlamaflareClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = llamaflare::Config::default(); // Or load from file
    let scraper = LlamaflareClient::new(config)?;
    let response = scraper.get("[https://example.com](https://example.com)").await?;
    println!("Response status: {}", response.status());
    let body = response.text().await?;
    println!("Response body: {}", body);
    Ok(())
}
\`\`\`

### Configuration

Configuration options are provided through the \`Config\` struct. Example configuration:

\`\`\`json
{
  "cache_enabled": true,
  "cache_max_age_seconds": 7200,
  "cache_max_size": 1000,
  "proxy": "http://user:pass@host:port",
  "proxy_rotation_enabled": true,
  "proxy_list_file": "proxies.txt",
  "rate_limiting_enabled": true,
  "min_delay_seconds": 1,
  "max_delay_seconds": 3,
  "browser_profile": "chrome_desktop"
}
\`\`\`

See the \`config.rs\` module for detailed information on configuration options and how to load them.

### Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### License

MIT License - See \`LICENSE\` file for details.

### Support

GitHub Issues: [Report a bug](https://github.com/yourusername/llamaflare/issues)
EOF

# Write LICENSE
cat > LICENSE << EOF
MIT License

Copyright (c) [Year] [Your Name]

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
EOF

# Write src/lib.rs
cat > src/lib.rs << EOF
pub mod cloudflare_bypass;
pub mod proxy_manager;
pub mod caching;
pub mod rate_limiting;
pub mod async_request;
pub mod error;
pub mod config;
pub mod logging_stats;
pub mod file_download;
pub mod request;
pub mod utils;
pub mod tokenizer; // Tokenizer module
pub mod image_proc; // Image processing
pub mod models;      // Models
pub mod generation;  // Generation

pub use error::Error;
pub use request::LlamaflareClient;
pub use config::Config; // Re-export config

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
EOF

# Write src/error.rs
cat > src/error.rs << EOF
use thiserror::Error;
use reqwest::Error as ReqwestError; // Import reqwest::Error
use std::io::Error as IOError; // Import std::io::Error
use serde_json::Error as SerdeJsonError; // Import Serde JSON Error
// use crate::tokenizer::Tokenizer; // Import Tokenizer trait - not needed here

#[derive(Error, Debug)]
pub enum Error {
    #[error("Cloudflare bypass failed: {0}")]
    CloudflareError(String),
    #[error("Captcha solving failed: {0}")]
    CaptchaError(String),
    #[error("Proxy error: {0}")]
    ProxyError(String),
    #[error("Rate limiting error: {0}")]
    RateLimitError(String),
    #[error("Cache error: {0}")]
    CacheError(String),
    #[error("Request error: {0}")]
    RequestError(String),
    #[error("IO error: {0}")]
    IOError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("JSON error: {0}")]
    JsonError(#[from] SerdeJsonError),
    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] ReqwestError),
    #[error("Image error: {0}")] // For image processing errors
    ImageError(String),
    #[error("Model error: {0}")] // For model loading or inference errors
    ModelError(String),
    #[error("Tokenization error: {0}")] // For tokenizer errors
    TokenizerError(String),
    #[error("Other error: {0}")]
    Other(String),
}

// Implement a conversion from reqwest::Error to our custom Error enum
impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::ReqwestError(err)
    }
}

impl From<IOError> for Error {
    fn from(err: IOError) -> Self {
        Error::IOError(err.to_string())
    }
}
EOF

# Write src/config.rs
cat > src/config.rs << EOF
use serde::Deserialize;
use std::path::PathBuf;
use crate::error::Result;
use std::fs;

/// Configuration struct for llamaflare.
#[derive(Deserialize, Debug, Default, Clone)]
pub struct Config {
    /// Enables response caching.
    pub cache_enabled: Option<bool>,
    /// Maximum age for cached responses in seconds.
    pub cache_max_age_seconds: Option<u64>,
    /// Maximum size of the cache in entries.
    pub cache_max_size: Option<usize>,
    /// Proxy URL to use for requests (e.g., "http://user:pass@host:port").
    pub proxy: Option<String>,
    /// Enables proxy rotation from a list.
    pub proxy_rotation_enabled: Option<bool>,
    /// Path to a file containing a list of proxies (one proxy per line).
    pub proxy_list_file: Option<PathBuf>,
    /// Enables request rate limiting.
    pub rate_limiting_enabled: Option<bool>,
    /// Minimum delay between requests in seconds.
    pub min_delay_seconds: Option<u64>,
    /// Maximum delay between requests in seconds (used for random delay).
    pub max_delay_seconds: Option<u64>,
    /// Captcha solver provider (e.g., "2captcha", "capsolver").
    pub captcha_solver_provider: Option<String>, // "2captcha", "capsolver", etc.
    /// API key for the captcha solver service.
    pub captcha_api_key: Option<String>,
    /// Browser profile to mimic for User-Agent and headers (e.g., "chrome_desktop", "firefox_mobile").
    pub browser_profile: Option<String>, // Chrome, Firefox or a custom path to json

    /// Vocab size for the model (example config - adjust as needed)
    pub vocab_size: Option<usize>,
    /// Hidden size for the model
    pub hidden_size: Option<usize>,
    /// Number of hidden layers in the model
    pub num_hidden_layers: Option<usize>,
    /// Number of attention heads in the model
    pub num_attention_heads: Option<usize>,
    /// Intermediate size for the model's MLP layers
    pub intermediate_size: Option<usize>,
    /// Hidden size for the vision encoder part of the model
    pub vision_hidden_size: Option<usize>,
    /// Size of the image input expected by the vision encoder
    pub image_size: Option<usize>,
    /// Type of model to load (e.g., "qwen2_vl")
    pub model_type: Option<String>,
}

impl Config {
    /// Loads configuration from a JSON file.
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to the configuration file.
    ///
    /// # Returns
    ///
    /// A `Result` containing the loaded `Config` or an `Error` if loading fails.
    pub fn load_from_file(config_path: &PathBuf) -> Result<Self> {
        let config_str = fs::read_to_string(config_path)
            .map_err(|e| crate::error::Error::IOError(format!("Failed to read config file: {}", e)))?;
        let config: Config = serde_json::from_str(&config_str)
            .map_err(|e| crate::error::Error::ConfigError(format!("Failed to parse config JSON: {}", e)))?;
        Ok(config)
    }
}
EOF

# Write src/models/mod.rs
cat > src/models/mod.rs << EOF
pub mod qwen2_vl;
pub mod model_config;

pub use qwen2_vl::Qwen2VLModel;
pub use model_config::ModelConfig;
EOF

# Write src/models/model_config.rs
cat > src/models/model_config.rs << EOF
// src/models/model_config.rs

pub trait ModelConfig {
    // Define common configuration methods or associated types if needed
}
EOF

# Write src/models/qwen2_vl.rs
cat > src/models/qwen2_vl.rs << EOF
use candle_core::{ Module, Tensor, Result, D, Device };
use candle_nn::{ Embedding, Linear, LayerNorm, VarBuilder, Module, linear, layer_norm, embedding, conv2d };
use crate::config::Config;
use crate::tokenizer::Tokenizer;

// Simplified configuration for the text encoder
#[derive(Debug, Clone)]
pub struct Qwen2VLTextConfig {
    pub vocab_size: usize,
    pub hidden_size: usize,
    pub num_hidden_layers: usize,
    pub num_attention_heads: usize,
    pub intermediate_size: usize,
    pub num_key_value_heads: usize,
    pub rms_norm_eps: f64,
}

// Simplified configuration for the vision encoder
#[derive(Debug, Clone)]
pub struct Qwen2VLVisionConfig {
    pub image_size: usize,
    pub patch_size: usize,
    pub hidden_size: usize,
    pub num_channels: usize,
    // ... Add other relevant vision config parameters (e.g., number of layers)
}

#[derive(Debug, Clone)]
pub struct Qwen2VLModelConfig {
    pub text_config: Qwen2VLTextConfig,
    pub vision_config: Qwen2VLVisionConfig,
    pub model_type: String,
    pub image_token_id: u32,
    pub video_token_id: u32,
}

impl Qwen2VLModelConfig {
    pub fn from_config(config: &Config) -> Result<Self> {
        Ok(Self {
            text_config: Qwen2VLTextConfig {
                vocab_size: config.vocab_size.unwrap_or(32000),
                hidden_size: config.hidden_size.unwrap_or(2048),
                num_hidden_layers: config.num_hidden_layers.unwrap_or(24),
                num_attention_heads: config.num_attention_heads.unwrap_or(16),
                intermediate_size: config.intermediate_size.unwrap_or(8192),
                num_key_value_heads: config.num_attention_heads.unwrap_or(16),
                rms_norm_eps: 1e-6,
            },
            vision_config: Qwen2VLVisionConfig {
                image_size: config.image_size.unwrap_or(384),
                patch_size: 14,
                hidden_size: config.vision_hidden_size.unwrap_or(1024),
                num_channels: 3,
                // ... Initialize vision config from config
            },
            model_type: config.model_type.clone().unwrap_or_else(|| "qwen2_vl".to_string()),
            image_token_id: 151655, // Replace with the actual image token ID
            video_token_id: 151656, // Replace with the actual video token ID (if applicable)
        })
    }
}

impl crate::models::model_config::ModelConfig for Qwen2VLModelConfig {}

// Vision Patch Embedding (very basic example)
#[derive(Debug, Clone)]
struct VisionPatchEmbed {
    proj: candle_nn::Conv2d,
}

impl VisionPatchEmbed {
    fn new(cfg: &Qwen2VLVisionConfig, vb: VarBuilder) -> Result<Self> {
        let proj = conv2d(
            cfg.num_channels,
            cfg.hidden_size,
            cfg.patch_size,
            vb.pp("proj"),
        )?;
        Ok(Self { proj })
    }

    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let x = self.proj.forward(x)?;
        let (b, c, h, w) = x.dims4()?;
        let x = x.reshape((b, c, h * w))?;
        Ok(x)
    }
}

impl Module for VisionPatchEmbed {
    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        self.proj.forward(x)
    }
}

// Simplified Attention (no RoPE, no multi-head mixing)
#[derive(Debug, Clone)]
struct Attention {
    q_proj: Linear,
    k_proj: Linear,
    v_proj: Linear,
    out_proj: Linear,
    num_heads: usize,
    head_dim: usize,
    scale: f64,
}

impl Attention {
    fn new(cfg: &Qwen2VLTextConfig, vb: VarBuilder) -> Result<Self> {
        let head_dim = cfg.hidden_size / cfg.num_attention_heads;
        let scale = 1.0 / (head_dim as f64).sqrt();
        let q_proj = linear(cfg.hidden_size, cfg.hidden_size, vb.pp("q_proj"))?;
        let k_proj = linear(cfg.hidden_size, cfg.hidden_size, vb.pp("k_proj"))?;
        let v_proj = linear(cfg.hidden_size, cfg.hidden_size, vb.pp("v_proj"))?;
        let out_proj = linear(cfg.hidden_size, cfg.hidden_size, vb.pp("out_proj"))?;

        Ok(Self {
            q_proj,
            k_proj,
            v_proj,
            out_proj,
            num_heads: cfg.num_attention_heads,
            head_dim,
            scale,
        })
    }

    fn forward(&self, x: &Tensor, mask: Option<&Tensor>) -> Result<Tensor> {
        let (b, seq_len, _) = x.dims3()?;
        let q = self.q_proj.forward(x)?.reshape((b, seq_len, self.num_heads, self.head_dim))?.transpose(1, 2)?;
        let k = self.k_proj.forward(x)?.reshape((b, seq_len, self.num_heads, self.head_dim))?.transpose(1, 2)?;
        let v = self.v_proj.forward(x)?.reshape((b, seq_len, self.num_heads, self.head_dim))?.transpose(1, 2)?;

        let attn_weights = (q @ k.transpose(2, 3)?).transpose(0, 2, 1, 3)? * self.scale; // (B, H, L, L)

        let attn_weights = match mask {
            Some(mask) => {
                // Handle attention mask
                let mask = mask.expand((b,self.num_heads, seq_len, seq_len))?;
                attn_weights.masked_fill(&mask, f32::NEG_INFINITY)?
            },
            None => attn_weights,
        };

        let attn_weights = candle_nn::ops::softmax(&attn_weights, D::Minus1)?;
        let output = (attn_weights @ v).transpose(1, 2)?.reshape((b, seq_len, self.head_dim * self.num_heads))?;
        let output = self.out_proj.forward(&output)?;

        Ok(output)
    }
}

// Simplified MLP (no activation)
#[derive(Debug, Clone)]
struct MLP {
    fc1: Linear,
    fc2: Linear,
}

impl MLP {
    fn new(cfg: &Qwen2VLTextConfig, vb: VarBuilder) -> Result<Self> {
        let fc1 = linear(
            cfg.hidden_size,
            cfg.intermediate_size,
            vb.pp("fc1"),
        )?;
        let fc2 = linear(
            cfg.intermediate_size,
            cfg.hidden_size,
            vb.pp("fc2"),
        )?;

        Ok(Self { fc1, fc2 })
    }

    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let x = self.fc1.forward(x)?;
        let x = self.fc2.forward(x)?;
        Ok(x)
    }
}

// Simplified Transformer Block
#[derive(Debug, Clone)]
struct TransformerBlock {
    attn: Attention,
    mlp: MLP,
    layer_norm1: LayerNorm,
    layer_norm2: LayerNorm,
}

impl TransformerBlock {
    fn new(cfg: &Qwen2VLTextConfig, vb: VarBuilder) -> Result<Self> {
        let attn = Attention::new(cfg, vb.pp("attn"))?;
        let mlp = MLP::new(cfg, vb.pp("mlp"))?;
        let layer_norm1 = layer_norm(cfg.hidden_size, vb.pp("layer_norm1"))?;
        let layer_norm2 = layer_norm(cfg.hidden_size, vb.pp("layer_norm2"))?;

        Ok(Self {
            attn,
            mlp,
            layer_norm1,
            layer_norm2,
        })
    }

    fn forward(&self, x: &Tensor, mask: Option<&Tensor>) -> Result<Tensor> {
        let residual = x;
        let x = self.layer_norm1.forward(x)?;
        let x = self.attn.forward(&x, mask)?;
        let x = x + residual;

        let residual = x;
        let x = self.layer_norm2.forward(x)?;
        let x = self.mlp.forward(x)?;
        let x = residual + x;
        Ok(x)
    }
}

// Text Embedding
#[derive(Debug, Clone)]
struct LanguageModel {
    embed_tokens: Embedding,
    layers: Vec<TransformerBlock>,
    norm: LayerNorm,
}

impl LanguageModel {
    fn new(cfg: &Qwen2VLTextConfig, vb: VarBuilder) -> Result<Self> {
        let embed_tokens = embedding(
            cfg.vocab_size,
            cfg.hidden_size,
            vb.pp("embed_tokens"),
        )?;

        let layers: Vec<TransformerBlock> = (0..cfg.num_hidden_layers)
            .map(|i| TransformerBlock::new(cfg, vb.pp(&format!("layers.{i}"))))
            .collect::<Result<_>>()?;

        let norm = layer_norm(cfg.hidden_size, vb.pp("norm"))?;

        Ok(Self {
            embed_tokens,
            layers,
            norm,
        })
    }

    fn forward(&self, x: &Tensor, mask: Option<&Tensor>) -> Result<Tensor> {
        let mut x = self.embed_tokens.forward(x)?;
        for layer in self.layers.iter() {
            x = layer.forward(&x, mask)?;
        }
        let x = self.norm.forward(&x)?;
        Ok(x)
    }
}

// Full Model
#[derive(Debug, Clone)]
pub struct Qwen2VLModel {
    vision_patch_embed: VisionPatchEmbed,
    language_model: LanguageModel,
    config: Qwen2VLModelConfig,
    device: Device,
}

impl Qwen2VLModel {
    pub fn new(config: Qwen2VLModelConfig, vb: VarBuilder) -> Result<Self> {
        let device = vb.device();
        let vision_patch_embed = VisionPatchEmbed::new(&config.vision_config, vb.pp("vision_patch_embed"))?;
        let language_model = LanguageModel::new(&config.text_config, vb.pp("language_model"))?;

        Ok(Self {
            vision_patch_embed,
            language_model,
            config,
            device,
        })
    }

    pub fn forward(
        &self,
        input_ids: &Tensor,
        pixel_values: Option<&Tensor>, // Image feature as input is now optional
    ) -> Result<Tensor> {
        // 1. Process vision input (pixel_values) - only if provided
        let vision_output = if let Some(pixel_values) = pixel_values {
            Some(self.vision_patch_embed.forward(pixel_values)?)
        } else {
            None
        };

        // 2. Process text input (input_ids)
        let language_output = self.language_model.forward(input_ids, None)?; // No mask for now

        Ok(language_output) // For now, just return language output, adjust as needed for VL fusion
    }
}
EOF

# Write src/tokenizer/mod.rs
cat > src/tokenizer/mod.rs << EOF
pub mod tokenizer;
pub mod detokenizer;

pub use tokenizer::Tokenizer;
pub use detokenizer::Detokenizer;
EOF

# Write src/tokenizer/tokenizer.rs
cat > src/tokenizer/tokenizer.rs << EOF
use crate::error::{Result, Error};
use std::collections::HashMap;

/// Trait defining the interface for a tokenizer.
pub trait Tokenizer {
    /// Encodes text into a sequence of token IDs.
    fn encode(&self, text: &str) -> Result<Vec<u32>>;
    /// Decodes a sequence of token IDs back into text.
    fn decode(&self, tokens: &[u32]) -> Result<String>;
    /// Returns the end-of-sentence token ID.
    fn eos_token_id(&self) -> u32;
    /// Returns the padding token ID.
    fn pad_token_id(&self) -> u32;
    // Add any other required methods (e.g., special token handling)
}

/// A simple whitespace tokenizer for demonstration purposes.
pub struct SimpleTokenizer {
    /// Vocabulary mapping words to token IDs.
    pub vocab: HashMap<String, u32>,
    /// End-of-sentence token ID.
    pub eos_token_id: u32,
    /// Padding token ID.
    pub pad_token_id: u32,
}

impl SimpleTokenizer {
    /// Creates a new \`SimpleTokenizer\`.
    ///
    /// # Arguments
    ///
    /// *   \`vocab\` - Vocabulary map.
    /// *   \`eos_token_id\` - End-of-sentence token ID.
    /// *   \`pad_token_id\` - Padding token ID.
    pub fn new(vocab: HashMap<String, u32>, eos_token_id: u32, pad_token_id: u32) -> Self {
        SimpleTokenizer { vocab, eos_token_id, pad_token_id }
    }
}

impl Tokenizer for SimpleTokenizer {
    /// Encodes text by whitespace splitting and vocabulary lookup.
    fn encode(&self, text: &str) -> Result<Vec<u32>> {
        let mut tokens = vec![];
        for word in text.split_whitespace() {
            if let Some(&token_id) = self.vocab.get(word) {
                tokens.push(token_id);
            } else {
                // For simplicity, treat unknown words as an error in this example
                return Err(Error::TokenizerError(format!("Unknown word: {}", word)));
            }
        }
        Ok(tokens)
    }

    /// Decodes tokens by reverse vocabulary lookup and joining with spaces.
    fn decode(&self, tokens: &[u32]) -> Result<String> {
        let mut text = String::new();
        let reverse_vocab: HashMap<u32, String> = self.vocab.iter().map(|(k, v)| (*v, k.clone())).collect();
        for token_id in tokens {
            if let Some(word) = reverse_vocab.get(token_id) {
                text.push_str(word);
                text.push(' ');
            } else {
                return Err(Error::TokenizerError(format!("Unknown token ID: {}", token_id)));
            }
        }
        Ok(text.trim().to_string())
    }

    /// Returns the EOS token ID.
    fn eos_token_id(&self) -> u32 {
        self.eos_token_id
    }

    /// Returns the PAD token ID.
    fn pad_token_id(&self) -> u32 {
        self.pad_token_id
    }
}
EOF

# Write src/tokenizer/detokenizer.rs
cat > src/tokenizer/detokenizer.rs << EOF
use crate::error::Result;

/// Trait defining the interface for a streaming detokenizer.
pub trait Detokenizer {
    /// Resets the detokenizer state.
    fn reset(&mut self);
    /// Adds a token to the detokenizer stream.
    fn add_token(&mut self, token: u32);
    /// Finalizes the detokenization process.
    fn finalize(&mut self);
    /// Returns the detokenized text so far.
    fn text(&self) -> String;
    /// Returns the last detokenized segment since the last \`reset\` or \`finalize\`.
    fn last_segment(&self) -> String;
    /// Returns the sequence of tokens processed so far.
    fn tokens(&self) -> &[u32];
}

/// A naive streaming detokenizer that simply adds spaces between tokens.
pub struct NaiveStreamingDetokenizer {
    text_so_far: String,
    tokens_so_far: Vec<u32>,
}

impl NaiveStreamingDetokenizer {
    /// Creates a new \`NaiveStreamingDetokenizer\`.
    pub fn new() -> Self {
        NaiveStreamingDetokenizer {
            text_so_far: String::new(),
            tokens_so_far: Vec::new(),
        }
    }
}

impl Detokenizer for NaiveStreamingDetokenizer {
    /// Resets the text and token history.
    fn reset(&mut self) {
        self.text_so_far.clear();
        self.tokens_so_far.clear();
    }

    /// Adds a token and appends a space to the current text.
    fn add_token(&mut self, token: u32) {
        self.tokens_so_far.push(token);
        self.text_so_far.push_str(" "); // Simple space-based detokenization
    }

    /// Placeholder: No finalization needed for this naive implementation.
    fn finalize(&mut self) {
        // No finalization needed for this naive implementation
    }

    /// Returns the accumulated text.
    fn text(&self) -> String {
        self.text_so_far.clone()
    }

    /// Returns the entire text as the last segment (for simplicity).
    fn last_segment(&self) -> String {
        self.text_so_far.clone()
    }

    /// Returns the accumulated tokens.
    fn tokens(&self) -> &[u32] {
        &self.tokens_so_far
    }
}
EOF

# Write src/image_proc/mod.rs
cat > src/image_proc/mod.rs << EOF
pub mod image_processor;

pub use image_processor::ImageProcessor;
pub use image_processor::BasicImageProcessor;
EOF

# Write src/image_proc/image_processor.rs
cat > src/image_proc/image_processor.rs << EOF
use crate::error::{Error, Result};
use image::{DynamicImage, ImageBuffer, Rgb, GenericImageView, imageops::FilterType, open};
use std::path::Path;
use candle_core::{Tensor, Device, DType};

/// Trait for image processing operations.
pub trait ImageProcessor {
    /// Loads an image from a given path.
    fn load_image(&self, path: &Path) -> Result<DynamicImage>;
    /// Resizes an image to a square of the given size.
    fn resize_image(&self, image: &DynamicImage, size: u32) -> Result<DynamicImage>;
    /// Normalizes an image (placeholder - grayscale normalization).
    fn normalize_image(&self, image: &DynamicImage) -> Result<DynamicImage>;
    /// Preprocesses an image: load, resize, normalize, and convert to Tensor.
    fn preprocess_image(&self, path: &Path, device: &Device) -> Result<Tensor>; // New: Return Tensor
}

/// Basic placeholder image processor.
pub struct BasicImageProcessor;

impl BasicImageProcessor {
    /// Creates a new \`BasicImageProcessor\`.
    pub fn new() -> Self {
        BasicImageProcessor
    }
}

impl ImageProcessor for BasicImageProcessor {
    /// Loads an image using the \`image\` crate.
    fn load_image(&self, path: &Path) -> Result<DynamicImage> {
        open(path).map_err(|e| Error::ImageError(e.to_string()))
    }

    /// Resizes an image using Lanczos3 resampling.
    fn resize_image(&self, image: &DynamicImage, size: u32) -> Result<DynamicImage> {
        Ok(image.resize_exact(size, size, FilterType::Lanczos3))
    }

    /// Placeholder: Basic grayscale normalization (converts to grayscale then RGB).
    fn normalize_image(&self, image: &DynamicImage) -> Result<DynamicImage> {
        let grayscale_image: ImageBuffer<Rgb<u8>, Vec<u8>> = image.grayscale().to_rgb8();
        Ok(DynamicImage::ImageRgb8(grayscale_image))
    }

    /// Preprocesses the image by loading, resizing, normalizing, and converting it to a \`Tensor\`.
    fn preprocess_image(&self, path: &Path, device: &Device) -> Result<Tensor> {
        let image = self.load_image(path)?;
        let resized_image = self.resize_image(&image, 384)?; // Example size
        let normalized_image = self.normalize_image(&resized_image)?;

        let image_array = normalized_image.to_rgb8();
        let (width, height) = image_array.dimensions();

        let mut data = Vec::new();
        for i in 0..height {
            for j in 0..width {
                let pixel = image_array.get_pixel(j, i);
                data.push(pixel[0] as f32 / 255.0); // R
                data.push(pixel[1] as f32 / 255.0); // G
                data.push(pixel[2] as f32 / 255.0); // B
            }
        }

        // Convert to MLX Tensor (channels last: H, W, C) and then reshape to (1, H, W, C)
        let tensor = Tensor::from_vec(data, (height as usize, width as usize, 3), device)?;
        let tensor = tensor.permute(&[2, 0, 1])?.unsqueeze(0)?; // Change to (1, C, H, W)

        Ok(tensor.to_dtype(DType::F32)?) // Ensure it's float32
    }
}
EOF

# Write src/generation/mod.rs
cat > src/generation/mod.rs << EOF
pub mod generate;
pub mod prompt_utils;

pub use generate::generate;
pub use generate::sample_token; // Export sample_token for potential external use
pub use prompt_utils::apply_chat_template; // Export prompt utilities
pub use prompt_utils::get_message_json;
EOF

# Write src/generation/generate.rs
cat > src/generation/generate.rs << EOF
use crate::{
    error::{Result, Error},
    models::Qwen2VLModel,
    tokenizer::Tokenizer,
    image_proc::ImageProcessor,
};
use candle_core::{ Tensor, Device, DType, ops::softmax };
use rand::{distributions::Distribution, distributions::WeightedIndex, prelude::StdRng, SeedableRng};

/// Generates text based on a model, tokenizer, and optional image.
pub fn generate(
    model: &Qwen2VLModel,
    tokenizer: &dyn Tokenizer,
    detokenizer: &mut dyn crate::tokenizer::detokenizer::Detokenizer,
    image_processor: &dyn ImageProcessor,
    image_path: Option<&str>,
    prompt: &str,
    max_tokens: usize,
    temperature: f64,
    device: &Device,
) -> Result<String> {
    // 1. Load and preprocess image (if image_path is provided)
    let pixel_values = if let Some(path) = image_path {
        Some(image_processor.preprocess_image(std::path::Path::new(path), device)?)
    } else {
        None
    };

    // 2. Tokenize prompt
    let input_ids_u32 = tokenizer.encode(prompt)?;
    let mut input_ids = input_ids_u32.iter().map(|&id| id as i64).collect::<Vec<_>>(); // Convert to i64
    let mut input_tensor = Tensor::from_vec(input_ids.clone(), (1, input_ids.len()), device)?;

    // 3. Model inference loop
    detokenizer.reset(); // Reset detokenizer state
    for _ in 0..max_tokens {
        let logits = model.forward(
            &input_tensor,
            pixel_values.as_ref(), // Pass the Option<&Tensor>
        )?;

        let next_token_id = sample_token(&logits.squeeze(0)?, temperature)?; // Squeeze to remove batch dim

        detokenizer.add_token(next_token_id as u32);

        if next_token_id == tokenizer.eos_token_id() as u32 {
            break; // Stop generation if EOS token is reached
        }

        input_ids.push(next_token_id as i64); // Append to input_ids (for next iteration, as i64)
        input_tensor = Tensor::from_vec(input_ids.clone(), (1, input_ids.len()), device)?; // Create tensor from updated input_ids
    }
    detokenizer.finalize(); // Finalize detokenization

    Ok(detokenizer.text())
}


/// Samples a token from the logits distribution using temperature scaling.
pub fn sample_token(logits: &Tensor, temperature: f64) -> Result<u32> {
    if temperature < 1e-5 {
        let next_token = logits.argmax(candle_core::D::Minus1)?;
        Ok(next_token.to_scalar::<u32>()?)
    } else {
        let logits = logits.to_dtype(DType::F32)?;
        let scaled_logits = (logits / temperature)?;
        let weights = softmax(&scaled_logits, candle_core::D::Minus1)?;
        let weights_v = weights.to_vec1::<f32>()?;

        let mut rng = StdRng::seed_from_u64(42); // Seed for reproducibility, remove for random
        let distribution = WeightedIndex::new(&weights_v).map_err(|e| Error::Other(format!("WeightedIndexError: {:?}", e)))?;
        let next_token_index = distribution.sample(&mut rng) as u32;
        Ok(next_token_index)
    }
}
EOF

# Write src/generation/prompt_utils.rs
cat > src/generation/prompt_utils.rs << EOF
use crate::error::{Result, Error};
use crate::tokenizer::Tokenizer;

/// Applies a chat template to the prompt based on the model type (placeholder).
pub fn apply_chat_template(
    tokenizer: &dyn Tokenizer,
    prompt: &str,
    _num_images: usize, // Example parameter - adjust as needed
) -> Result<String> {
    // Placeholder: Implement chat template logic based on model type
    // For example, you might prepend image tokens and special tokens.
    // Handle different roles (user, assistant, system) based on model.
    // This is where the specifics of the VLM's prompting style go.
    let formatted_prompt = format!("<image>User: {prompt}\\nAssistant:"); // Placeholder template
    Ok(formatted_prompt)
}

/// Creates a simplified JSON-like message representation (placeholder).
pub fn get_message_json(
    model_name: &str,
    prompt: &str,
    role: &str,
    skip_image_token: bool,
    num_images: usize,
    video: Option<&str>, // Example video parameter
) -> Result<String> {
    // Placeholder: Create a simplified JSON-like message representation (e.g., for logging or debugging).
    let message_json = format!(
        r#"{{"role": "{}", "content": "{}", "model": "{}", "skip_image_token": {}, "num_images": {}, "video": "{:?}"}}"#,
        role, prompt, model_name, skip_image_token, num_images, video
    );
    Ok(message_json)
}
EOF

# Write src/cli/main.rs
cat > src/cli/main.rs << EOF
use clap::{Parser, Subcommand};
use llamaflare::cli::generate_cmd::GenerateCommand;
use llamaflare::cli::chat_ui_cmd::ChatUiCommand; // Placeholder import
use anyhow::Result;

/// llamaflare: Your ultimate web scraping and vision-language assistant
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate text from a vision-language model
    Generate(GenerateCommand),
    /// Launch a chat UI (Placeholder - Future Feature)
    ChatUi(ChatUiCommand), // Placeholder command
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init(); // Initialize tracing for logging

    let cli = Cli::parse();

    match &cli.command {
        Commands::Generate(cmd) => cmd.run().await?,
        Commands::ChatUi(cmd) => cmd.run()?, // Placeholder command run - fix return type to Result<()>
    }
    Ok(())
}
EOF

# Write src/cli/generate_cmd.rs
cat > src/cli/generate_cmd.rs << EOF
use clap::Parser;
use llamaflare::{generation, tokenizer::SimpleTokenizer, image_proc::BasicImageProcessor, error::Result, config::Config};
use std::path::PathBuf;
use candle_core::Device;
use std::collections::HashMap;

/// Generate text using a vision-language model.
#[derive(Parser, Debug)]
pub struct GenerateCommand {
    /// Path to the model directory.
    #[arg(long, default_value = "mlx-community/qwen2-vl-2b-instruct-bf16")] // Replace with your model path
    model: String,

    /// Prompt for text generation.
    #[arg(long)]
    prompt: String,

    /// Path to an image file (optional for vision-language models).
    #[arg(long)]
    image: Option<String>,

    /// Maximum number of tokens to generate.
    #[arg(long, default_value_t = 100)]
    max_tokens: usize,

    /// Temperature for sampling (higher values more random).
    #[arg(long, default_value_t = 0.7)]
    temperature: f64,

    /// Enable verbose output (for debugging).
    #[arg(long, default_value_t = false)]
    verbose: bool,
}

impl GenerateCommand {
    /// Executes the text generation command.
    pub async fn run(&self) -> anyhow::Result<()> {
        let model_path = PathBuf::from(&self.model);
        let tokenizer = SimpleTokenizer::new(
            // Placeholder: Provide a sample vocab, eos, pad -  **IMPORTANT: Replace with actual vocab loading**
            HashMap::from([("hello".to_string(), 1), ("world".to_string(), 2), ("<|endoftext|>".to_string(), 2), ("<|pad|>".to_string(), 0)]),
            2, // eos_token_id (replace with actual)
            0, // pad_token_id (replace with actual)
        );

        let mut detokenizer = crate::tokenizer::detokenizer::NaiveStreamingDetokenizer::new();
        let image_processor = BasicImageProcessor::new();
        let device = Device::cuda_if_available()?; // Or Cpu::new_cpu() if no CUDA

        // Load model config
        let config_path = model_path.join("config.json");
        let config = Config::load_from_file(&config_path)?;

        let model_config = llamaflare::models::Qwen2VLModelConfig::from_config(&config)?;

        // Load model (Placeholder - Implement proper weight loading)
        let vb = candle_core::VarBuilder::zeros(candle_core::DType::F32, &device); // Dummy VarBuilder for now
        let model = llamaflare::models::qwen2_vl::Qwen2VLModel::new(model_config, vb)?; // Dummy model for now

        if self.verbose {
            println!("Model loaded.");
            println!("Configuration: {:?}", config);
        }

        let generated_text = generation::generate(
            &model,
            &tokenizer,
            &mut detokenizer,
            &image_processor,
            self.image.as_deref(),
            &self.prompt,
            self.max_tokens,
            self.temperature,
            &device,
        )?;

        println!("\\nGenerated Text:\\n{}", generated_text);

        Ok(())
    }
}
EOF

# Write src/cli/chat_ui_cmd.rs
cat > src/cli/chat_ui_cmd.rs << EOF
use clap::Parser;
use crate::error::Result;

/// Launch an interactive chat UI (placeholder).
#[derive(Parser, Debug)]
pub struct ChatUiCommand {
    /// Path to the model directory.
    #[arg(long, default_value = "mlx-community/qwen2-vl-2b-instruct-bf16")] // Replace with your model path
    model: String,
}

impl ChatUiCommand {
    /// Executes the chat UI command (placeholder implementation).
    pub fn run(&self) -> Result<()> {
        println!("Chat UI command is a placeholder in this version.");
        println!("Interactive chat UI is a planned future feature.");
        println!("Model path (specified, but not used in placeholder): {}", self.model);
        Ok(())
    }
}
EOF

# Write src/logging_stats.rs
cat > src/logging_stats.rs << EOF
use std::sync::atomic::{AtomicUsize, Ordering};

/// Simple struct to track request and bypass statistics.
pub struct LoggingStats {
    requests_attempted: AtomicUsize,
    requests_successful: AtomicUsize,
    cloudflare_challenges: AtomicUsize,
    cloudflare_bypasses: AtomicUsize,
    cache_hits: AtomicUsize,
}

impl LoggingStats {
    /// Creates a new \`LoggingStats\` instance with all counters initialized to zero.
    pub fn new() -> Self {
        LoggingStats {
            requests_attempted: AtomicUsize::new(0),
            requests_successful: AtomicUsize::new(0),
            cloudflare_challenges: AtomicUsize::new(0),
            cloudflare_bypasses: AtomicUsize::new(0),
            cache_hits: AtomicUsize::new(0),
        }
    }

    /// Increments the count of attempted requests.
    pub fn increment_requests_attempted(&self) {
        self.requests_attempted.fetch_add(1, Ordering::SeqCst);
    }

    /// Increments the count of successful requests.
    pub fn increment_requests_successful(&self) {
        self.requests_successful.fetch_add(1, Ordering::SeqCst);
    }

    /// Increments the count of Cloudflare challenges detected.
    pub fn increment_cloudflare_challenges(&self) {
        self.cloudflare_challenges.fetch_add(1, Ordering::SeqCst);
    }

    /// Increments the count of successful Cloudflare bypasses.
    pub fn increment_cloudflare_bypasses(&self) {
        self.cloudflare_bypasses.fetch_add(1, Ordering::SeqCst);
    }

    /// Increments the count of cache hits.
    pub fn increment_cache_hits(&self) {
        self.cache_hits.fetch_add(1, Ordering::SeqCst);
    }

    /// Prints the current statistics to the console.
    pub fn print_stats(&self) {
        println!("--- Logging Statistics ---");
        println!("Requests Attempted: {}", self.requests_attempted.load(Ordering::SeqCst));
        println!("Requests Successful: {}", self.requests_successful.load(Ordering::SeqCst));
        println!("Cloudflare Challenges: {}", self.cloudflare_challenges.load(Ordering::SeqCst));
        println!("Cloudflare Bypasses: {}", self.cloudflare_bypasses.load(Ordering::SeqCst));
        println!("Cache Hits: {}", self.cache_hits.load(Ordering::SeqCst));
        println!("------------------------");
    }
}
EOF

# Write src/file_download.rs
cat > src/file_download.rs << EOF
use crate::error::{Result, Error};
use reqwest::{Client, Response};
use indicatif::{ProgressBar, ProgressStyle};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::{info, debug, error};
use futures_util::StreamExt;

/// Manages file downloads with progress tracking.
pub struct FileDownloader {
    client: Client,
}

impl FileDownloader {
    /// Creates a new \`FileDownloader\` instance.
    pub fn new(client: Client) -> Self {
        FileDownloader { client }
    }

    /// Downloads a file from a given URL and saves it to the specified path.
    pub async fn download(&self, initial_response: Response, url: &str, destination_path: &str) -> Result<()> {
        let total_size = initial_response.content_length().unwrap_or(0);

        let pb = ProgressBar::new(total_size);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
            .progress_chars("#>-"));

        let mut downloaded: u64 = 0;
        let mut stream = initial_response.bytes_stream();

        let mut dest_file = File::create(destination_path).await?;
        info!("Downloading file from {} to {}", url, destination_path);

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    dest_file.write_all(&chunk).await?;
                    let new_downloaded = downloaded + chunk.len() as u64;
                    downloaded = new_downloaded;
                    pb.set_position(downloaded);
                }
                Err(e) => {
                    error!("Download stream error: {}", e);
                    pb.finish_with_message("Download failed");
                    return Err(Error::IOError(format!(" أثناء تنزيل الدفق: {}", e))); // Error message in Arabic? - Check and correct if needed
                }
            }
        }

        pb.finish_with_message("Download complete");
        dest_file.flush().await?;
        Ok(())
    }
}
EOF

# Write src/request.rs
cat > src/request.rs << EOF
use crate::error::{Error, Result};
use reqwest::{Client, RequestBuilder, Response, Url};
use reqwest::header::{HeaderMap, HeaderValue, COOKIE, USER_AGENT, ACCEPT, ACCEPT_LANGUAGE, CACHE_CONTROL, UPGRADE_INSECURE_REQUESTS};
use std::collections::HashMap;

use crate::cloudflare_bypass::{CookieManager, UserAgentManager, NativeJavascriptSolver, ChallengeParams, extract_challenge_params};
use crate::proxy_manager::ProxyRotator;
use crate::caching::ResponseCache;
use crate::rate_limiting::RateLimiter;
use crate::logging_stats::LoggingStats;
use crate::config::Config;
use tracing::{info, debug, warn, error};
use crate::utils::extract_base_url;
use crate::file_download::FileDownloader;

/// Core HTTP client for making requests with Cloudflare bypass and other features.
pub struct LlamaflareClient {
    client: Client,
    cookie_manager: CookieManager,
    user_agent_manager: UserAgentManager,
    js_solver: NativeJavascriptSolver,
    proxy_rotator: Option<ProxyRotator>,
    response_cache: Option<ResponseCache>,
    rate_limiter: Option<RateLimiter>,
    logging_stats: LoggingStats,
    config: Config,
    file_downloader: FileDownloader,
}

impl LlamaflareClient {
    /// Creates a new \`LlamaflareClient\` instance.
    ///
    /// Initializes the HTTP client, cookie manager, user agent manager,
    /// JavaScript solver, and optionally proxy rotator, response cache, and rate limiter
    /// based on the provided configuration.
    ///
    /// # Arguments
    ///
    /// *   \`config\` - A reference to the \`Config\` struct containing client configurations.
    ///
    /// # Returns
    ///
    /// A \`Result\` containing the \`LlamaflareClient\` instance or an \`Error\` if initialization fails.
    pub fn new(config: Config) -> Result<Self> {
        let mut client_builder = Client::builder();

        // Proxy setup
        if let Some(proxy_url) = &config.proxy {
            let proxy = reqwest::Proxy::http(proxy_url.clone())
                .map_err(|e| Error::ConfigError(format!("Invalid proxy URL: {}", e)))?;
            client_builder = client_builder.proxy(proxy);
            info!("Proxy configured: {}", proxy_url);
        }

        let client = client_builder
            .cookie_store(true) // Enable cookie store
            .build()?;

        let cookie_manager = CookieManager::new();
        let user_agent_manager = UserAgentManager::new(config.browser_profile.clone());
        let js_solver = NativeJavascriptSolver::new();
        let proxy_rotator = if config.proxy_rotation_enabled.unwrap_or(false) {
            Some(ProxyRotator::new(config.proxy_list_file.clone())?)
        } else {
            None
        };
        let response_cache = if config.cache_enabled.unwrap_or(false) {
            Some(ResponseCache::new(config.cache_max_age_seconds.unwrap_or(3600), config.cache_max_size.unwrap_or(1000)))
        } else {
            None
        };
        let rate_limiter = if config.rate_limiting_enabled.unwrap_or(false) {
            Some(RateLimiter::new(config.min_delay_seconds.unwrap_or(1), config.max_delay_seconds.unwrap_or(3)))
        } else {
            None
        };
        let logging_stats = LoggingStats::new();
        let file_downloader = FileDownloader::new(client.clone());


        info!("LlamaflareClient initialized.");

        Ok(Self {
            client,
            cookie_manager,
            user_agent_manager,
            js_solver,
            proxy_rotator,
            response_cache,
            rate_limiter,
            logging_stats,
            config,
            file_downloader,
        })
    }

    /// Creates a \`RequestBuilder\` with common headers and settings pre-configured.
    fn create_request_builder(&self, url: &str) -> Result<RequestBuilder> {
        let mut headers = HeaderMap::new();

        // User-Agent
        let user_agent = self.user_agent_manager.get_user_agent();
        headers.insert(USER_AGENT, HeaderValue::from_str(&user_agent)?);

        // Accept Headers (Mimic Browser)
        headers.insert(ACCEPT, HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8"));
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));
        headers.insert(CACHE_CONTROL, HeaderValue::from_static("max-age=0"));
        headers.insert(UPGRADE_INSECURE_REQUESTS, HeaderValue::from_static("1"));

        let mut request_builder = self.client.get(url).headers(headers);

        // Proxy rotation
        if let Some(proxy_rotator) = &self.proxy_rotator {
            if let Some(proxy_url) = proxy_rotator.get_proxy() {
                let proxy = reqwest::Proxy::http(proxy_url.clone())
                    .map_err(|e| Error::ProxyError(format!("Invalid proxy URL from rotator: {}", e)))?;
                request_builder = request_builder.proxy(proxy);
                debug!("Using proxy from rotator: {}", proxy_url);
            } else {
                warn!("No proxies available from rotator, proceeding without proxy.");
            }
        }

        Ok(request_builder)
    }


    /// Performs a GET request to the specified URL, handling Cloudflare bypass, caching, etc.
    ///
    /// # Arguments
    ///
    /// *   \`url\` - The URL to request.
    ///
    /// # Returns
    ///
    /// A \`Result\` containing the \`reqwest::Response\` on success, or an \`Error\` on failure.
    pub async fn get(&self, url: &str) -> Result<Response> {
        self.execute_request(url, |client, url| client.get(url)).await
    }

    /// Performs a POST request to the specified URL, handling Cloudflare bypass, caching, etc.
    ///
    /// # Arguments
    ///
    /// *   \`url\` - The URL to request.
    /// *   \`body\` - The body of the POST request (e.g., \`reqwest::Body\`).
    ///
    /// # Returns
    ///
    /// A \`Result\` containing the \`reqwest::Response\` on success, or an \`Error\` on failure.
    pub async fn post(&self, url: &str, body: reqwest::Body) -> Result<Response> {
        self.execute_request(url, |client, url| client.post(url).body(body)).await
    }


    /// Downloads a file from the specified URL with progress tracking, handling Cloudflare bypass, etc.
    ///
    /// # Arguments
    ///
    /// *   \`url\` - The URL of the file to download.
    /// *   \`destination_path\` - The local path where the file should be saved.
    ///
    /// # Returns
    ///
    /// A \`Result\` indicating success or an \`Error\` on failure.
    pub async fn download_file(&self, url: &str, destination_path: &str) -> Result<()> {
        info!("Starting file download from {} to {}", url, destination_path);

        // Rate limiting (apply before cache check)
        if let Some(rate_limiter) = &self.rate_limiter {
            rate_limiter.wait().await;
        }

        // Cloudflare and request setup via execute_request for initial response headers and body
        let initial_response = self.execute_request(url, |client, url| {
            let request_builder = self.create_request_builder(url)?; // Use create_request_builder for initial request setup
            Ok(request_builder)
        }).await?;


        self.file_downloader.download(initial_response, url, destination_path).await?;
        info!("File downloaded successfully to {}", destination_path);
        Ok(())
    }


    async fn execute_request<F>(&self, url_str: &str, request_fn: F) -> Result<Response>
        where
            F: Fn(&Client, &str) -> Result<RequestBuilder> + Sync + Send,
    {
        let url = Url::parse(url_str)?;
        let base_url = extract_base_url(&url)?;
        let mut current_url_str = url_str.to_string(); // Mutable copy for retries

        // Rate limiting (apply before cache check)
        if let Some(rate_limiter) = &self.rate_limiter {
            rate_limiter.wait().await;
        }

        // Check cache
        if let Some(cache) = &self.response_cache {
            if let Some(cached_response) = cache.get(&current_url_str) {
                self.logging_stats.increment_cache_hits();
                debug!("Cache hit for URL: {}", current_url_str);
                return Ok(cached_response);
            }
        }

        self.logging_stats.increment_requests_attempted();

        let mut response = self.make_initial_request(&current_url_str, &request_fn).await?;

        // Cloudflare IUAM handling with retry logic
        for retry_attempt in 0..3 { // Limit retries to prevent infinite loops
            if Self::is_cloudflare_iuam_challenge(&response) {
                warn!("Cloudflare IUAM challenge detected for URL: {}", current_url_str);
                self.logging_stats.increment_cloudflare_challenges();

                let challenge_params_result = extract_challenge_params(&current_url_str, response.text().await?)