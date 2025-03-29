use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use serde::{Serialize, Deserialize};
use toml;

use crate::error::ArxivError;

/// Configuration settings for the application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Directory where downloaded papers are stored
    pub download_dir: PathBuf,
    
    /// ArXiv API settings
    pub api: ApiConfig,
    
    /// Download settings
    pub download: DownloadConfig,
    
    /// PDF processing settings
    pub pdf: PdfConfig,
    
    /// Citation settings
    pub citation: CitationConfig,
    
    /// User agent for HTTP requests
    pub user_agent: String,
}

/// ArXiv API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Base URL for the arXiv API
    pub base_url: String,
    
    /// Maximum number of results to fetch
    pub max_results: u32,
    
    /// Request timeout in seconds
    pub timeout: u64,
    
    /// Number of retries for failed requests
    pub retries: u32,
    
    /// Delay between retries in milliseconds
    pub retry_delay: u64,
}

/// Download configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadConfig {
    /// Concurrent download limit
    pub concurrency: u32,
    
    /// Download timeout in seconds
    pub timeout: u64,
    
    /// Whether to follow redirects
    pub follow_redirects: bool,
    
    /// Maximum number of redirects to follow
    pub max_redirects: u32,
    
    /// File naming pattern
    pub filename_pattern: String,
}

/// PDF processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfConfig {
    /// Default output format
    pub default_format: String,
    
    /// Whether to include figures in extraction
    pub include_figures: bool,
    
    /// Whether to include tables in extraction
    pub include_tables: bool,
    
    /// Whether to include math formulas in extraction
    pub include_math: bool,
    
    /// Path to PDFium library (if used)
    pub pdfium_path: Option<String>,
}

/// Citation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationConfig {
    /// Whether to attempt to extract citations from PDF text
    pub extract_from_pdf: bool,
    
    /// Whether to generate citations from metadata if extraction fails
    pub generate_from_metadata: bool,
    
    /// Default citation style
    pub style: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            download_dir: dirs::download_dir()
                .unwrap_or_else(|| PathBuf::from("downloads"))
                .join("llama-arxiv"),
            api: ApiConfig::default(),
            download: DownloadConfig::default(),
            pdf: PdfConfig::default(),
            citation: CitationConfig::default(),
            user_agent: format!(
                "llama-arxiv/{} (https://github.com/llamamoonlight/llama-arxiv)",
                env!("CARGO_PKG_VERSION")
            ),
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            base_url: "http://export.arxiv.org/api/query".to_string(),
            max_results: 10,
            timeout: 30,
            retries: 3,
            retry_delay: 1000,
        }
    }
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            concurrency: 2,
            timeout: 120,
            follow_redirects: true,
            max_redirects: 5,
            filename_pattern: "{id}_{first_author}_{year}_{title}.pdf".to_string(),
        }
    }
}

impl Default for PdfConfig {
    fn default() -> Self {
        Self {
            default_format: "text".to_string(),
            include_figures: true,
            include_tables: true,
            include_math: true,
            pdfium_path: None,
        }
    }
}

impl Default for CitationConfig {
    fn default() -> Self {
        Self {
            extract_from_pdf: true,
            generate_from_metadata: true,
            style: "bibtex".to_string(),
        }
    }
}

/// Load configuration from a file
pub fn load(path: &Path) -> Result<Config, ArxivError> {
    if !path.exists() {
        return Err(ArxivError::ConfigError(format!(
            "Configuration file not found: {}",
            path.display()
        )));
    }
    
    let content = fs::read_to_string(path)
        .map_err(|e| ArxivError::ConfigError(format!(
            "Failed to read configuration file: {}",
            e
        )))?;
    
    let config: Config = toml::from_str(&content)
        .map_err(|e| ArxivError::ConfigError(format!(
            "Failed to parse configuration file: {}",
            e
        )))?;
    
    Ok(config)
}

/// Save configuration to a file
pub fn save(config: &Config, path: &Path) -> Result<(), ArxivError> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| ArxivError::IoError(e))?;
    }
    
    let content = toml::to_string_pretty(config)
        .map_err(|e| ArxivError::ConfigError(format!(
            "Failed to serialize configuration: {}",
            e
        )))?;
    
    fs::write(path, content)
        .map_err(|e| ArxivError::IoError(e))?;
    
    Ok(())
}

/// Create default configuration file if it doesn't exist
pub fn ensure_default_config(path: &Path) -> Result<(), ArxivError> {
    if !path.exists() {
        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Create default config
        let config = Config::default();
        
        // Save to file
        save(&config, path)?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        
        assert_eq!(config.api.max_results, 10);
        assert_eq!(config.download.concurrency, 2);
        assert_eq!(config.citation.style, "bibtex");
    }
    
    #[test]
    fn test_save_and_load() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        
        let config = Config::default();
        save(&config, &config_path).unwrap();
        
        assert!(config_path.exists());
        
        let loaded_config = load(&config_path).unwrap();
        assert_eq!(loaded_config.api.max_results, config.api.max_results);
    }
} 