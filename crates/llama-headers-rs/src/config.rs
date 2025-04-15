//! Configuration settings for the llama-headers-rs crate
use crate::user_agent::UserAgent;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::errors::LlamaHeadersError;

/// Configuration for header generation
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Preferred language for Accept-Language header
    pub language: Option<String>,
    
    /// Custom User-Agent
    pub user_agent: Option<UserAgent>,
    
    /// Whether to generate mobile headers
    pub mobile: Option<bool>,
    
    /// Custom referer URL
    pub referer: Option<String>,
    
    /// Custom Accept header
    pub accept: Option<String>,
    
    /// Custom Accept-Encoding header
    pub accept_encoding: Option<String>,
    
    /// Custom Connection header
    pub connection: Option<String>,
    
    /// Additional custom headers
    pub custom_headers: Option<Vec<(String, String)>>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            language: None,
            user_agent: None,
            mobile: Some(false),
            referer: None,
            accept: None,
            accept_encoding: None,
            connection: None,
            custom_headers: None,
        }
    }
}

impl Config {
    /// Creates a new default configuration
    pub fn new() -> Self {
        Config::default()
    }
    
    /// Load configuration from a TOML file
    pub fn load(config_path: &str) -> Result<Self, LlamaHeadersError> {
        let path = Path::new(config_path);

        if path.exists() {
            let contents = fs::read_to_string(path)
                .map_err(|e| LlamaHeadersError::Io(e))?;
            
            let config: Config = toml::from_str(&contents)
                .map_err(|e| LlamaHeadersError::ConfigError(e.to_string()))?;
            
            Ok(config)
        } else {
            Err(LlamaHeadersError::ConfigError(format!("Configuration file '{}' not found", config_path)))
        }
    }
    
    /// Save configuration to a TOML file
    pub fn save(&self, config_path: &str) -> Result<(), LlamaHeadersError> {
        let toml_string = toml::to_string(self)
            .map_err(|e| LlamaHeadersError::ConfigError(e.to_string()))?;
        
        fs::write(config_path, toml_string)
            .map_err(|e| LlamaHeadersError::Io(e))?;
        
        Ok(())
    }
    
    /// Set the language
    pub fn with_language(mut self, language: &str) -> Self {
        self.language = Some(language.to_string());
        self
    }
    
    /// Set the user agent
    pub fn with_user_agent(mut self, user_agent: UserAgent) -> Self {
        self.user_agent = Some(user_agent);
        self
    }
    
    /// Set whether to use mobile headers
    pub fn with_mobile(mut self, mobile: bool) -> Self {
        self.mobile = Some(mobile);
        self
    }
    
    /// Set the referer
    pub fn with_referer(mut self, referer: &str) -> Self {
        self.referer = Some(referer.to_string());
        self
    }
    
    /// Add a custom header
    pub fn with_custom_header(mut self, key: &str, value: &str) -> Self {
        let mut headers = self.custom_headers.unwrap_or_default();
        headers.push((key.to_string(), value.to_string()));
        self.custom_headers = Some(headers);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.mobile, Some(false));
        assert!(config.language.is_none());
        assert!(config.user_agent.is_none());
    }
    
    #[test]
    fn test_config_builder() {
        let config = Config::new()
            .with_language("fr-FR")
            .with_mobile(true)
            .with_referer("https://example.com")
            .with_custom_header("X-Custom", "Value");
        
        assert_eq!(config.language, Some("fr-FR".to_string()));
        assert_eq!(config.mobile, Some(true));
        assert_eq!(config.referer, Some("https://example.com".to_string()));
        assert!(config.custom_headers.is_some());
        
        let headers = config.custom_headers.unwrap();
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0], ("X-Custom".to_string(), "Value".to_string()));
    }
    
    #[test]
    fn test_save_load_config() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();
        
        let config = Config::new()
            .with_language("de-DE")
            .with_mobile(true);
        
        // Save the config
        config.save(path).unwrap();
        
        // Load the config
        let loaded_config = Config::load(path).unwrap();
        
        assert_eq!(loaded_config.language, Some("de-DE".to_string()));
        assert_eq!(loaded_config.mobile, Some(true));
    }
    
    #[test]
    fn test_load_nonexistent_config() {
        let result = Config::load("/path/to/nonexistent/config.toml");
        assert!(result.is_err());
    }
} 