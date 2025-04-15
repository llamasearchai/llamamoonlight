//! # Llama-Moonlight Core
//! 
//! Core functionality for Llama-Moonlight, a Rust-based browser automation framework.
//! This crate provides the foundational functionality for browser automation with AI capabilities.
//!
//! ## Features
//!
//! - Browser automation (Chrome, Firefox, Safari support)
//! - Headless and headed modes
//! - Network interception and mocking
//! - Screenshot and video capture
//! - WebSocket protocol support
//! - Integration with llama-headers-rs for stealth browsing
//! - Support for MLX integration for AI-powered automation
//!
//! ## Example
//!
//! ```
//! use llama_moonlight_core::{Moonlight, BrowserType};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize the framework
//!     let moonlight = Moonlight::new().await?;
//!     
//!     // Launch a browser
//!     let browser_type = moonlight.browser_type("chromium").unwrap();
//!     let browser = browser_type.launch().await?;
//!     
//!     // Create a new page
//!     let context = browser.new_context().await?;
//!     let page = context.new_page().await?;
//!     
//!     // Navigate to a URL
//!     page.goto("https://example.com").await?;
//!     
//!     // Take a screenshot
//!     page.screenshot("example.png").await?;
//!     
//!     // Close the browser
//!     browser.close().await?;
//!     
//!     Ok(())
//! }
//! ```

mod browser;
mod context;
mod page;
mod frame;
mod element;
mod input;
mod network;
mod selectors;
mod dialog;
mod download;
mod video;
mod errors;
mod event;
mod har;
mod cdp;
mod accessibility;
mod worker;
mod protocol;
mod options;
mod utils;
mod chromium;
mod firefox;
mod webkit;
mod llama_integration;

// Re-exports
pub use browser::{Browser, BrowserType};
pub use context::BrowserContext;
pub use page::Page;
pub use frame::Frame;
pub use element::ElementHandle;
pub use input::{Keyboard, Mouse, Touchscreen};
pub use network::{Request, Response, Route, WebSocket};
pub use selectors::Selectors;
pub use dialog::Dialog;
pub use download::Download;
pub use video::VideoRecorder;
pub use errors::Error;
pub use event::EventEmitter;
pub use har::Har;
pub use cdp::CDPSession;
pub use accessibility::Accessibility;
pub use worker::Worker;
pub use options::{BrowserOptions, ContextOptions, PageOptions};
pub use llama_integration::LlamaModel;

use crate::protocol::Connection;
use std::sync::Arc;
use log::{debug, info};

/// The main entry point for the Llama-Moonlight API.
pub struct Moonlight {
    browser_types: Vec<BrowserType>,
}

impl Moonlight {
    /// Creates a new instance of Moonlight.
    pub async fn new() -> Result<Self, Error> {
        info!("Initializing Llama-Moonlight framework");
        
        let browser_types = vec![
            BrowserType::new("chromium"),
            BrowserType::new("firefox"),
            BrowserType::new("webkit"),
        ];
        
        debug!("Initialized {} browser types", browser_types.len());
        
        Ok(Self { browser_types })
    }
    
    /// Returns a browser type by name.
    pub fn browser_type(&self, name: &str) -> Option<&BrowserType> {
        self.browser_types.iter().find(|bt| bt.name() == name)
    }
    
    /// Returns all available browser types.
    pub fn browser_types(&self) -> &[BrowserType] {
        &self.browser_types
    }
    
    /// Sets up a browser with stealth mode using llama-headers-rs.
    #[cfg(feature = "stealth")]
    pub async fn stealth_browser(&self, name: &str) -> Result<Browser, Error> {
        let browser_type = self.browser_type(name)
            .ok_or_else(|| Error::BrowserTypeNotFound(name.to_string()))?;
            
        let mut options = options::BrowserOptions::default();
        options.stealth = Some(true);
        
        browser_type.launch_with_options(options).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_moonlight_new() {
        let moonlight = Moonlight::new().await.unwrap();
        assert_eq!(moonlight.browser_types().len(), 3);
    }
    
    #[tokio::test]
    async fn test_moonlight_browser_type() {
        let moonlight = Moonlight::new().await.unwrap();
        let browser_type = moonlight.browser_type("chromium");
        assert!(browser_type.is_some());
        assert_eq!(browser_type.unwrap().name(), "chromium");
        
        let non_existent = moonlight.browser_type("nonexistent");
        assert!(non_existent.is_none());
    }
} 