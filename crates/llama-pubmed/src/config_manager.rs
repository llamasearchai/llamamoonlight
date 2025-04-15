use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_yaml::Error),
    
    #[error("Could not determine config directory")]
    NoConfigDir,
    
    #[error("Configuration file not found at {0}")]
    ConfigNotFound(PathBuf),
}

// Define the configuration struct.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub download_dir: PathBuf,
    pub max_retries: u32,
    pub user_agent: Option<String>,
    pub finders: Vec<String>,
}

// Define the default configuration values.
impl Default for Config {
    fn default() -> Self {
        Config {
            download_dir: PathBuf::from("./downloads"),
            max_retries: 3,
            user_agent: Some(
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/56.0.2924.87 Safari/537.36".to_string()
            ),
            finders: vec![
                "genericCitationLabelled".to_string(),
                "pubmed_central_v2".to_string(),
                "acsPublications".to_string(),
                "uchicagoPress".to_string(),
                "nejm".to_string(),
                "futureMedicine".to_string(),
                "science_direct".to_string(),
                "direct_pdf_link".to_string(),
            ],
        }
    }
}

const CONFIG_FILE_NAME: &str = "llamapubmed_config.yaml";

// Function to get the config file path.
pub fn get_config_file_path() -> Result<PathBuf, ConfigError> {
    let config_dir = dirs::config_dir().ok_or(ConfigError::NoConfigDir)?;
    let config_file_path = config_dir.join(CONFIG_FILE_NAME);
    Ok(config_file_path)
}

// Function to load the configuration from a file.
pub fn load_config_from_file(file_path: &Path) -> Result<Config, ConfigError> {
    if !file_path.exists() {
        return Err(ConfigError::ConfigNotFound(file_path.to_path_buf()));
    }
    
    let file = File::open(file_path)?;
    let config: Config = serde_yaml::from_reader(file)?;
    info!("Loaded configuration from: {:?}", file_path);
    Ok(config)
}

// Function to load the default configuration.
pub fn load_default_config() -> Result<Config, ConfigError> {
    let config_file_path = get_config_file_path()?;
    if config_file_path.exists() {
        load_config_from_file(&config_file_path)
    } else {
        // If the config file doesn't exist, create it with default settings.
        let default_config = Config::default();
        save_config(&default_config)?;
        info!("Created default config file at: {:?}", config_file_path);
        Ok(default_config)
    }
}

// Function to save the config to disk
pub fn save_config(config: &Config) -> Result<(), ConfigError> {
    let config_file_path = get_config_file_path()?;

    // Ensure the directory exists
    if let Some(dir) = config_file_path.parent() {
        fs::create_dir_all(dir)?;
    }

    let mut file = File::create(&config_file_path)?;
    serde_yaml::to_writer(&mut file, config)?;
    info!("Configuration saved to: {:?}", config_file_path);
    Ok(())
}

// Function to reset the configuration to default values.
pub fn reset_config() -> Result<(), ConfigError> {
    let default_config = Config::default();
    save_config(&default_config)?;
    Ok(())
} 