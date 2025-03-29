pub mod cli;
pub mod arxiv;
pub mod download;
pub mod parser;
pub mod metadata;
pub mod config;

// Context struct to hold application state
#[derive(Debug)]
pub struct Context {
    /// Command-line arguments/settings
    pub app_config: cli::AppConfig,
    
    /// Application configuration
    pub config: config::Config,
}

impl Context {
    /// Create a new runtime context
    pub fn new(app_config: cli::AppConfig, config: config::Config) -> Self {
        Self { app_config, config }
    }
} 