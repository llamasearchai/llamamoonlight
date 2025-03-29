//! # Llama Moonlight Tor
//!
//! Tor network integration for the Llama Moonlight browser automation framework.
//! This crate provides capabilities for anonymous browsing, access to onion services,
//! and dark web search aggregation.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use thiserror::Error;

pub mod client;
pub mod circuit;
pub mod config;
pub mod controller;
pub mod engines;
pub mod guard;
pub mod onion;
pub mod proxy;
pub mod search;
pub mod socks;
pub mod utils;

/// Result type used throughout the library
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in the Llama Moonlight Tor library
#[derive(Error, Debug)]
pub enum Error {
    /// Error during Tor initialization
    #[error("Failed to initialize Tor: {0}")]
    TorInitError(String),
    
    /// Error with Tor circuit creation
    #[error("Circuit error: {0}")]
    CircuitError(String),
    
    /// Error with Tor controller
    #[error("Controller error: {0}")]
    ControllerError(String),
    
    /// Error with Tor SOCKS proxy
    #[error("SOCKS proxy error: {0}")]
    SocksError(String),
    
    /// Error with Tor configuration
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    /// Error with onion service
    #[error("Onion service error: {0}")]
    OnionError(String),
    
    /// Error with search engines
    #[error("Search engine error: {0}")]
    SearchError(String),
    
    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),
    
    /// Error from the Arti Tor client
    #[error("Arti client error: {0}")]
    ArtiClientError(#[from] arti_client::Error),
    
    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    /// URL parsing error
    #[error("URL parsing error: {0}")]
    UrlError(#[from] url::ParseError),
    
    /// Parsing error
    #[error("Parsing error: {0}")]
    ParseError(String),
    
    /// HTTP error
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),
    
    /// Error from the stealth crate
    #[error("Stealth error: {0}")]
    StealthError(#[from] llama_moonlight_stealth::Error),
    
    /// Other errors
    #[error("Other error: {0}")]
    Other(String),
}

/// Library version information
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Configuration for Tor operations
#[derive(Debug, Clone)]
pub struct TorConfig {
    /// Path to the Tor data directory
    pub data_dir: std::path::PathBuf,
    
    /// Whether to use the Tor browser bundle
    pub use_tor_browser: bool,
    
    /// Path to the Tor browser directory
    pub tor_browser_path: Option<std::path::PathBuf>,
    
    /// SOCKS proxy host
    pub socks_host: String,
    
    /// SOCKS proxy port
    pub socks_port: u16,
    
    /// Control port
    pub control_port: u16,
    
    /// Authentication password for the control port
    pub control_password: Option<String>,
    
    /// Country codes to use for exit nodes (e.g. "US,DE,FR")
    pub exit_nodes: Option<String>,
    
    /// Whether to use bridges
    pub use_bridges: bool,
    
    /// Bridge lines to use
    pub bridges: Vec<String>,
    
    /// Additional Tor options
    pub options: std::collections::HashMap<String, String>,
    
    /// Timeout for Tor operations in seconds
    pub timeout_secs: u64,
}

impl Default for TorConfig {
    fn default() -> Self {
        Self {
            data_dir: std::path::PathBuf::from("./.tor_data"),
            use_tor_browser: false,
            tor_browser_path: None,
            socks_host: "127.0.0.1".to_string(),
            socks_port: 9050,
            control_port: 9051,
            control_password: None,
            exit_nodes: None,
            use_bridges: false,
            bridges: Vec::new(),
            options: std::collections::HashMap::new(),
            timeout_secs: 120,
        }
    }
}

/// Capabilities for a browser or client that can use Tor
pub trait TorCapable {
    /// Configure the browser or client to use Tor
    fn configure_tor(&mut self, config: &TorConfig) -> Result<()>;
    
    /// Check if the connection is using Tor
    fn is_using_tor(&self) -> Result<bool>;
    
    /// Get the current Tor circuit information
    fn get_circuit_info(&self) -> Result<circuit::CircuitInfo>;
    
    /// Request a new Tor circuit
    fn new_circuit(&mut self) -> Result<()>;
    
    /// Access an onion service
    fn access_onion(&mut self, onion_url: &str) -> Result<()>;
}

// Re-export key types for convenience
pub use client::TorClient;
pub use circuit::TorCircuit;
pub use controller::TorController;
pub use onion::OnionService;
pub use proxy::TorProxy;
pub use search::TorSearchEngine; 