//! # Llama Moonlight Stealth
//!
//! Advanced stealth capabilities for web automation using the Llama Moonlight framework.
//! This crate integrates stealth techniques to bypass anti-bot systems and provide
//! realistic browser behavior.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use thiserror::Error;

pub mod evasion;
pub mod client;
pub mod fingerprint;
pub mod injection;
pub mod intercept;
pub mod proxy;
pub mod detection;
pub mod humanize;
pub mod timing;

/// Result type used throughout the library
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in the Llama Moonlight Stealth library
#[derive(Error, Debug)]
pub enum Error {
    /// Error during initialization
    #[error("Failed to initialize: {0}")]
    InitError(String),
    
    /// Error with injecting scripts
    #[error("Script injection error: {0}")]
    InjectionError(String),
    
    /// Error with fingerprinting
    #[error("Fingerprinting error: {0}")]
    FingerprintError(String),
    
    /// Error with proxy
    #[error("Proxy error: {0}")]
    ProxyError(String),
    
    /// Error with detection avoidance
    #[error("Detection error: {0}")]
    DetectionError(String),
    
    /// Error with request interception
    #[error("Intercept error: {0}")]
    InterceptError(String),
    
    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),
    
    /// Error from the headers crate
    #[error("Headers error: {0}")]
    HeadersError(#[from] llama_moonlight_headers::HeaderError),
    
    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),
    
    /// Other errors
    #[error("Other error: {0}")]
    Other(String),
}

/// Library version information
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Configuration for stealth operations
#[derive(Clone, Debug)]
pub struct StealthConfig {
    /// Whether to enable stealth mode
    pub stealth_enabled: bool,
    
    /// Whether to use random fingerprints
    pub random_fingerprints: bool,
    
    /// Whether to randomize user agents
    pub random_user_agents: bool,
    
    /// Whether to emulate human behavior
    pub emulate_human: bool,
    
    /// Whether to use proxy servers
    pub use_proxies: bool,
    
    /// Additional custom headers
    pub custom_headers: std::collections::HashMap<String, String>,
    
    /// Whether to intercept WebGL fingerprinting
    pub intercept_webgl: bool,
    
    /// Whether to intercept canvas fingerprinting
    pub intercept_canvas: bool,
    
    /// Whether to intercept font enumeration
    pub intercept_fonts: bool,
    
    /// Whether to hide automation markers
    pub hide_automation: bool,
}

impl Default for StealthConfig {
    fn default() -> Self {
        Self {
            stealth_enabled: true,
            random_fingerprints: true,
            random_user_agents: false,
            emulate_human: true,
            use_proxies: false,
            custom_headers: std::collections::HashMap::new(),
            intercept_webgl: true,
            intercept_canvas: true,
            intercept_fonts: true,
            hide_automation: true,
        }
    }
}

/// Stealth capabilities for a browser session
pub trait StealthCapabilities {
    /// Apply stealth techniques to the browser
    fn apply_stealth(&mut self) -> Result<()>;
    
    /// Set a custom fingerprint
    fn set_fingerprint(&mut self, fingerprint: &fingerprint::BrowserFingerprint) -> Result<()>;
    
    /// Set custom headers
    fn set_headers(&mut self, headers: std::collections::HashMap<String, String>) -> Result<()>;
    
    /// Set a proxy server
    fn set_proxy(&mut self, proxy: &proxy::ProxyConfig) -> Result<()>;
    
    /// Emulate human-like behavior
    fn emulate_human(&mut self) -> Result<()>;
    
    /// Hide automation markers
    fn hide_automation_markers(&mut self) -> Result<()>;
}

// Re-export key types for convenience
pub use evasion::{EvasionManager, EvasionTechnique};
pub use client::StealthClient;
pub use fingerprint::BrowserFingerprint;
pub use detection::DetectionTest;
pub use proxy::ProxyConfig; 