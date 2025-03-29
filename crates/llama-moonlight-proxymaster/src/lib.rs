//! # Llama Moonlight ProxyMaster
//! 
//! A high-performance, asynchronous proxy management system with scraping,
//! validation, rotation, and a REST API. Part of the Llama Moonlight ecosystem.
//! 
//! ## Features
//! 
//! - Proxy Discovery: Scrapes free proxies from multiple online sources
//! - Validation: Tests proxies for functionality, speed, and anonymity
//! - Rotation: Smart proxy rotation with multiple selection strategies
//! - Persistence: Stores proxies in SQLite database
//! - REST API: Complete API for proxy management
//! - Integration: Seamless integration with the Llama Moonlight ecosystem
//! 
//! ## Example
//! 
//! ```rust,no_run
//! use llama_moonlight_proxymaster::{
//!     database::init_db,
//!     pool::ProxyPool,
//!     scraper::scrape_proxies,
//! };
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize database
//!     let db_pool = init_db("sqlite:proxymaster.db").await?;
//!     
//!     // Create proxy pool
//!     let proxy_pool = ProxyPool::new(db_pool);
//!     proxy_pool.initialize().await?;
//!     
//!     // Get a proxy from the pool
//!     if let Some(proxy) = proxy_pool.get_proxy().await {
//!         println!("Got proxy: {}", proxy.as_str());
//!     }
//!     
//!     Ok(())
//! }
//! ```

// Re-export all modules
pub mod api;
pub mod database;
pub mod models;
pub mod pool;
pub mod scraper;
pub mod utils;
pub mod validator;

// Re-export commonly used types
pub use crate::models::{Proxy, SelectionStrategy};
pub use crate::pool::{PoolConfig, ProxyPool};
pub use crate::scraper::{ScraperConfig, scrape_proxies};
pub use crate::validator::ValidatorConfig;

/// Version of the crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Module containing public types for use in library consumers
pub mod types {
    pub use crate::models::{Proxy, SelectionStrategy};
    pub use crate::pool::PoolConfig;
    pub use crate::scraper::ScraperConfig;
    pub use crate::validator::ValidatorConfig;
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_version() {
        assert!(!super::VERSION.is_empty());
    }
} 