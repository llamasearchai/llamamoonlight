//! # llama-moonlight-finance
//!
//! A powerful, comprehensive financial data and trading integration library for the Llama Moonlight ecosystem.
//! This crate provides access to market data, trading capabilities, technical analysis, and portfolio management.
//!
//! ## Features
//!
//! - **Market Data**: Access financial data from multiple providers (Yahoo Finance, Alpha Vantage, etc.)
//! - **Trading**: Integration with various trading platforms (Binance, FTX, TradingView, etc.)
//! - **Analysis**: Technical analysis indicators, charting, and statistical tools
//! - **Portfolio**: Portfolio tracking, performance analysis, and risk assessment
//! - **Stealth**: Optional integration with llama-moonlight-stealth for avoiding rate limits and detection
//! - **Anonymity**: Optional integration with llama-moonlight-tor for enhanced privacy
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use llama_moonlight_finance::{FinanceClient, YahooProvider, Result};
//! use llama_moonlight_finance::data::{TimeInterval, TimeRange};
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Create a client with the Yahoo Finance provider
//!     let client = FinanceClient::new()
//!         .with_provider(YahooProvider::default())
//!         .build();
//!
//!     // Fetch historical prices for Apple stock
//!     let prices = client.historical_prices("AAPL")
//!         .interval(TimeInterval::Daily)
//!         .range(TimeRange::Year(1))
//!         .fetch()
//!         .await?;
//!
//!     // Calculate some basic statistics
//!     let stats = prices.statistics();
//!     println!("AAPL Average Price: ${:.2}", stats.mean);
//!     println!("AAPL Volatility: {:.2}%", stats.volatility * 100.0);
//!
//!     Ok(())
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use thiserror::Error;
use std::fmt;
use std::str::FromStr;

// Core modules
pub mod client;
pub mod config;
pub mod data;
pub mod market;
pub mod provider;
pub mod portfolio;
pub mod trading;
pub mod analysis;
pub mod screener;
pub mod alert;
pub mod utils;

// Feature-gated modules

// Browser automation
#[cfg(feature = "browser")]
pub mod browser;

// Stealth capabilities
#[cfg(feature = "stealth")]
pub mod stealth;

// Tor integration
#[cfg(feature = "tor")]
pub mod tor;

// Provider-specific modules
pub mod providers {
    //! Financial data provider implementations
    
    /// Yahoo Finance API provider
    #[cfg(feature = "yahoo")]
    pub mod yahoo;
    
    /// Alpha Vantage API provider
    #[cfg(feature = "alphavantage")]
    pub mod alpha_vantage;
    
    /// CoinMarketCap API provider
    #[cfg(feature = "coinmarketcap")]
    pub mod coinmarketcap;
    
    /// Binance API provider
    #[cfg(feature = "binance")]
    pub mod binance;
    
    /// FTX API provider
    #[cfg(feature = "ftx")]
    pub mod ftx;
    
    /// TradingView API provider
    #[cfg(feature = "tradingview")]
    pub mod tradingview;
}

// Re-exports for convenience
pub use crate::client::FinanceClient;
pub use crate::config::ClientConfig;
pub use crate::data::{Price, TimeSeries, MarketData, Quote};
pub use crate::provider::{Provider, DataProvider, TradingProvider};

#[cfg(feature = "yahoo")]
pub use crate::providers::yahoo::YahooProvider;

#[cfg(feature = "alphavantage")]
pub use crate::providers::alpha_vantage::AlphaVantageProvider;

/// Result type used throughout the crate
pub type Result<T> = std::result::Result<T, Error>;

/// Error enum for the finance library
#[derive(Error, Debug)]
pub enum Error {
    /// Data provider errors
    #[error("Provider error: {0}")]
    ProviderError(String),
    
    /// Market data errors
    #[error("Market data error: {0}")]
    MarketDataError(String),
    
    /// Trading errors
    #[error("Trading error: {0}")]
    TradingError(String),
    
    /// Authentication errors
    #[error("Authentication error: {0}")]
    AuthError(String),
    
    /// Rate limiting errors
    #[error("Rate limit error: {0}")]
    RateLimitError(String),
    
    /// Data parsing errors
    #[error("Parse error: {0}")]
    ParseError(String),
    
    /// Network errors
    #[error("Network error: {0}")]
    NetworkError(String),
    
    /// HTTP errors from reqwest
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    
    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    /// URL parsing errors
    #[error("URL error: {0}")]
    UrlError(#[from] url::ParseError),
    
    /// IO errors
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    /// Decimal errors
    #[error("Decimal error: {0}")]
    DecimalError(String),
    
    /// Validation errors
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    /// Browser automation errors
    #[cfg(feature = "browser")]
    #[error("Browser error: {0}")]
    BrowserError(String),
    
    /// Stealth mode errors
    #[cfg(feature = "stealth")]
    #[error("Stealth error: {0}")]
    StealthError(String),
    
    /// Tor-related errors
    #[cfg(feature = "tor")]
    #[error("Tor error: {0}")]
    TorError(String),
    
    /// Analysis errors
    #[error("Analysis error: {0}")]
    AnalysisError(String),
    
    /// Portfolio errors
    #[error("Portfolio error: {0}")]
    PortfolioError(String),
    
    /// Other errors
    #[error("Other error: {0}")]
    Other(String),
}

/// Asset class types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetClass {
    /// Stocks/Equities
    Stock,
    /// Bonds/Fixed Income
    Bond,
    /// Cryptocurrencies
    Crypto,
    /// Exchange-Traded Funds
    ETF,
    /// Options contracts
    Option,
    /// Futures contracts
    Future,
    /// Forex currencies
    Forex,
    /// Commodities
    Commodity,
    /// Real Estate Investment Trusts
    REIT,
    /// Other asset types
    Other,
}

impl fmt::Display for AssetClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssetClass::Stock => write!(f, "Stock"),
            AssetClass::Bond => write!(f, "Bond"),
            AssetClass::Crypto => write!(f, "Cryptocurrency"),
            AssetClass::ETF => write!(f, "ETF"),
            AssetClass::Option => write!(f, "Option"),
            AssetClass::Future => write!(f, "Future"),
            AssetClass::Forex => write!(f, "Forex"),
            AssetClass::Commodity => write!(f, "Commodity"),
            AssetClass::REIT => write!(f, "REIT"),
            AssetClass::Other => write!(f, "Other"),
        }
    }
}

impl FromStr for AssetClass {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "stock" | "equity" | "stocks" | "equities" => Ok(AssetClass::Stock),
            "bond" | "bonds" | "fixed income" | "fixed-income" => Ok(AssetClass::Bond),
            "crypto" | "cryptocurrency" | "cryptocurrencies" => Ok(AssetClass::Crypto),
            "etf" | "etfs" | "exchange traded fund" | "exchange-traded fund" => Ok(AssetClass::ETF),
            "option" | "options" => Ok(AssetClass::Option),
            "future" | "futures" => Ok(AssetClass::Future),
            "forex" | "fx" | "currency" | "currencies" => Ok(AssetClass::Forex),
            "commodity" | "commodities" => Ok(AssetClass::Commodity),
            "reit" | "reits" | "real estate" => Ok(AssetClass::REIT),
            "other" => Ok(AssetClass::Other),
            _ => Err(Error::ParseError(format!("Unknown asset class: {}", s))),
        }
    }
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Returns the current version of the library
pub fn version() -> &'static str {
    VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_class_display() {
        assert_eq!(AssetClass::Stock.to_string(), "Stock");
        assert_eq!(AssetClass::Crypto.to_string(), "Cryptocurrency");
        assert_eq!(AssetClass::ETF.to_string(), "ETF");
    }

    #[test]
    fn test_asset_class_from_str() {
        assert_eq!(AssetClass::from_str("stock").unwrap(), AssetClass::Stock);
        assert_eq!(AssetClass::from_str("equity").unwrap(), AssetClass::Stock);
        assert_eq!(AssetClass::from_str("Crypto").unwrap(), AssetClass::Crypto);
        assert_eq!(AssetClass::from_str("ETF").unwrap(), AssetClass::ETF);
        assert_eq!(AssetClass::from_str("fixed income").unwrap(), AssetClass::Bond);
        
        assert!(AssetClass::from_str("unknown").is_err());
    }

    #[test]
    fn test_version() {
        assert_eq!(version(), VERSION);
    }
} 