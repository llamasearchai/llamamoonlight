//! # Llama Moonlight Headers
//!
//! A library for generating realistic browser headers for web automation and scraping.
//! This library provides various functions for generating headers that mimic real browsers,
//! which can help with avoiding detection when scraping websites.
//!
//! ## Features
//!
//! - Generate headers for various browsers (Chrome, Firefox, Safari, Edge)
//! - Customizable user agents
//! - Stealth mode for avoiding bot detection
//! - Fingerprinting protection
//! - Session persistence
//!
//! ## Example
//!
//! ```rust
//! use llama_moonlight_headers::{HeaderGenerator, BrowserType};
//!
//! let generator = HeaderGenerator::new(BrowserType::Chrome);
//! let headers = generator.generate("https://example.com");
//!
//! println!("{:#?}", headers);
//! ```

use std::collections::HashMap;
use rand::Rng;
use lazy_static::lazy_static;
use thiserror::Error;
use log::{debug, warn};
use chrono::Utc;

pub mod browser;
pub mod device;
pub mod fingerprint;
pub mod stealth;
pub mod useragent;
pub mod platform;
pub mod language;
pub mod utils;

pub use browser::BrowserType;
pub use device::DeviceType;
pub use platform::PlatformType;

/// Errors that can occur when generating headers
#[derive(Error, Debug)]
pub enum HeaderError {
    /// Error when the browser type is invalid
    #[error("Invalid browser type: {0}")]
    InvalidBrowserType(String),
    
    /// Error when the device type is invalid
    #[error("Invalid device type: {0}")]
    InvalidDeviceType(String),
    
    /// Error when the platform type is invalid
    #[error("Invalid platform type: {0}")]
    InvalidPlatformType(String),
    
    /// Error when a header value is invalid
    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(String),
    
    /// Error when JSON serialization/deserialization fails
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    /// Other errors
    #[error("Other error: {0}")]
    Other(String),
}

/// Result type for header operations
pub type Result<T> = std::result::Result<T, HeaderError>;

/// A generator for HTTP headers that mimic real browsers
#[derive(Debug, Clone)]
pub struct HeaderGenerator {
    /// The type of browser to generate headers for
    browser_type: BrowserType,
    
    /// The type of device to generate headers for
    device_type: DeviceType,
    
    /// The type of platform to generate headers for
    platform_type: PlatformType,
    
    /// Custom user agent, if any
    custom_user_agent: Option<String>,
    
    /// Whether to use stealth mode
    stealth_mode: bool,
    
    /// Custom headers to include
    custom_headers: HashMap<String, String>,
    
    /// Language code (e.g., "en-US")
    language: String,
    
    /// Whether to use random Accept-Language values
    randomize_language: bool,
    
    /// Whether to include Sec-* headers
    include_sec_headers: bool,
}

impl Default for HeaderGenerator {
    fn default() -> Self {
        Self {
            browser_type: BrowserType::Chrome,
            device_type: DeviceType::Desktop,
            platform_type: PlatformType::Windows,
            custom_user_agent: None,
            stealth_mode: false,
            custom_headers: HashMap::new(),
            language: "en-US".to_string(),
            randomize_language: false,
            include_sec_headers: true,
        }
    }
}

impl HeaderGenerator {
    /// Create a new HeaderGenerator with the specified browser type
    pub fn new(browser_type: BrowserType) -> Self {
        Self {
            browser_type,
            ..Default::default()
        }
    }
    
    /// Set the device type
    pub fn with_device(mut self, device_type: DeviceType) -> Self {
        self.device_type = device_type;
        self
    }
    
    /// Set the platform type
    pub fn with_platform(mut self, platform_type: PlatformType) -> Self {
        self.platform_type = platform_type;
        self
    }
    
    /// Set a custom user agent
    pub fn with_user_agent(mut self, user_agent: &str) -> Self {
        self.custom_user_agent = Some(user_agent.to_string());
        self
    }
    
    /// Enable stealth mode
    pub fn with_stealth(mut self, stealth: bool) -> Self {
        self.stealth_mode = stealth;
        self
    }
    
    /// Add a custom header
    pub fn with_custom_header(mut self, name: &str, value: &str) -> Self {
        self.custom_headers.insert(name.to_string(), value.to_string());
        self
    }
    
    /// Set the language
    pub fn with_language(mut self, language: &str) -> Self {
        self.language = language.to_string();
        self
    }
    
    /// Set whether to randomize the Accept-Language header
    pub fn with_randomize_language(mut self, randomize: bool) -> Self {
        self.randomize_language = randomize;
        self
    }
    
    /// Set whether to include Sec-* headers
    pub fn with_sec_headers(mut self, include: bool) -> Self {
        self.include_sec_headers = include;
        self
    }
    
    /// Get the user agent string
    pub fn get_user_agent(&self) -> String {
        if let Some(ref ua) = self.custom_user_agent {
            return ua.clone();
        }
        
        useragent::generate_user_agent(&self.browser_type, &self.device_type, &self.platform_type)
    }
    
    /// Generate headers for a specific URL
    pub fn generate(&self, url: &str) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        let user_agent = self.get_user_agent();
        
        // Basic headers
        headers.insert("User-Agent".to_string(), user_agent);
        headers.insert("Accept".to_string(), self.get_accept_header());
        headers.insert("Accept-Language".to_string(), self.get_accept_language());
        headers.insert("Accept-Encoding".to_string(), "gzip, deflate, br".to_string());
        headers.insert("Connection".to_string(), "keep-alive".to_string());
        
        // Include Referer if stealth mode is enabled and URL is not empty
        if self.stealth_mode && !url.is_empty() {
            if let Some(referer) = self.generate_referer(url) {
                headers.insert("Referer".to_string(), referer);
            }
        }
        
        // Add browser-specific headers
        match self.browser_type {
            BrowserType::Chrome => {
                headers.insert("Upgrade-Insecure-Requests".to_string(), "1".to_string());
                
                if self.include_sec_headers {
                    headers.insert("Sec-Ch-Ua".to_string(), self.get_sec_ch_ua());
                    headers.insert("Sec-Ch-Ua-Mobile".to_string(), match self.device_type {
                        DeviceType::Mobile => "?1".to_string(),
                        _ => "?0".to_string(),
                    });
                    headers.insert("Sec-Ch-Ua-Platform".to_string(), self.get_sec_ch_ua_platform());
                    headers.insert("Sec-Fetch-Dest".to_string(), "document".to_string());
                    headers.insert("Sec-Fetch-Mode".to_string(), "navigate".to_string());
                    headers.insert("Sec-Fetch-Site".to_string(), "none".to_string());
                    headers.insert("Sec-Fetch-User".to_string(), "?1".to_string());
                }
            },
            BrowserType::Firefox => {
                headers.insert("Upgrade-Insecure-Requests".to_string(), "1".to_string());
                headers.insert("Pragma".to_string(), "no-cache".to_string());
                headers.insert("Cache-Control".to_string(), "no-cache".to_string());
                headers.insert("TE".to_string(), "trailers".to_string());
            },
            BrowserType::Safari => {
                headers.insert("Accept-Language".to_string(), 
                               if self.randomize_language {
                                   language::random_safari_language()
                               } else {
                                   format!("{},en;q=0.9", self.language)
                               });
            },
            BrowserType::Edge => {
                headers.insert("Upgrade-Insecure-Requests".to_string(), "1".to_string());
                
                if self.include_sec_headers {
                    headers.insert("Sec-Ch-Ua".to_string(), self.get_sec_ch_ua());
                    headers.insert("Sec-Ch-Ua-Mobile".to_string(), match self.device_type {
                        DeviceType::Mobile => "?1".to_string(),
                        _ => "?0".to_string(),
                    });
                    headers.insert("Sec-Ch-Ua-Platform".to_string(), self.get_sec_ch_ua_platform());
                    headers.insert("Sec-Fetch-Dest".to_string(), "document".to_string());
                    headers.insert("Sec-Fetch-Mode".to_string(), "navigate".to_string());
                    headers.insert("Sec-Fetch-Site".to_string(), "none".to_string());
                }
            },
            BrowserType::Opera => {
                headers.insert("Upgrade-Insecure-Requests".to_string(), "1".to_string());
                
                if self.include_sec_headers {
                    headers.insert("Sec-Ch-Ua".to_string(), self.get_sec_ch_ua());
                    headers.insert("Sec-Ch-Ua-Mobile".to_string(), match self.device_type {
                        DeviceType::Mobile => "?1".to_string(),
                        _ => "?0".to_string(),
                    });
                    headers.insert("Sec-Ch-Ua-Platform".to_string(), self.get_sec_ch_ua_platform());
                }
            },
            BrowserType::Custom(_) => {
                // No additional headers for custom browser types
            },
        }
        
        // Add stealth mode headers
        if self.stealth_mode {
            stealth::add_stealth_headers(&mut headers, url, &self.browser_type, &self.device_type);
        }
        
        // Add custom headers
        for (name, value) in &self.custom_headers {
            headers.insert(name.clone(), value.clone());
        }
        
        headers
    }
    
    /// Generate headers as a string
    pub fn generate_as_string(&self, url: &str) -> String {
        let headers = self.generate(url);
        headers.iter()
            .map(|(name, value)| format!("{}: {}", name, value))
            .collect::<Vec<String>>()
            .join("\r\n")
    }
    
    /// Generate a referer header for the given URL
    fn generate_referer(&self, url: &str) -> Option<String> {
        if url.is_empty() {
            return None;
        }
        
        if !self.stealth_mode {
            return None;
        }
        
        // Parse the URL to get the domain
        if let Ok(parsed_url) = url::Url::parse(url) {
            let host = parsed_url.host_str()?;
            let scheme = parsed_url.scheme();
            
            // Generate a referer from a common site or the same domain
            let mut rng = rand::thread_rng();
            if rng.gen_bool(0.7) {
                // 70% chance to use a search engine as referer
                let search_engines = [
                    "https://www.google.com/search?q=",
                    "https://www.bing.com/search?q=",
                    "https://search.yahoo.com/search?p=",
                    "https://duckduckgo.com/?q=",
                ];
                
                let search_engine = search_engines[rng.gen_range(0..search_engines.len())];
                let query = if host.contains('.') {
                    let parts: Vec<&str> = host.split('.').collect();
                    if parts.len() >= 2 {
                        parts[parts.len() - 2].to_string()
                    } else {
                        host.to_string()
                    }
                } else {
                    host.to_string()
                };
                
                return Some(format!("{}{}", search_engine, query));
            } else {
                // 30% chance to use the same domain with a different path
                let paths = [
                    "/",
                    "/index.html",
                    "/home",
                    "/search",
                    "/about",
                    "/contact",
                ];
                
                let path = paths[rng.gen_range(0..paths.len())];
                return Some(format!("{}://{}{}", scheme, host, path));
            }
        }
        
        None
    }
    
    /// Get the Accept header based on the browser type
    fn get_accept_header(&self) -> String {
        match self.browser_type {
            BrowserType::Chrome | BrowserType::Edge | BrowserType::Opera => {
                "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7".to_string()
            },
            BrowserType::Firefox => {
                "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8".to_string()
            },
            BrowserType::Safari => {
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".to_string()
            },
            BrowserType::Custom(_) => {
                "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8".to_string()
            },
        }
    }
    
    /// Get the Accept-Language header
    fn get_accept_language(&self) -> String {
        if self.randomize_language {
            language::random_language()
        } else {
            format!("{},en;q=0.9", self.language)
        }
    }
    
    /// Get the Sec-Ch-Ua header
    fn get_sec_ch_ua(&self) -> String {
        match self.browser_type {
            BrowserType::Chrome => {
                "\"Google Chrome\";v=\"117\", \"Not;A=Brand\";v=\"8\", \"Chromium\";v=\"117\"".to_string()
            },
            BrowserType::Edge => {
                "\"Microsoft Edge\";v=\"117\", \"Not;A=Brand\";v=\"8\", \"Chromium\";v=\"117\"".to_string()
            },
            BrowserType::Opera => {
                "\"Opera\";v=\"101\", \"Not;A=Brand\";v=\"8\", \"Chromium\";v=\"117\"".to_string()
            },
            _ => "".to_string(),
        }
    }
    
    /// Get the Sec-Ch-Ua-Platform header
    fn get_sec_ch_ua_platform(&self) -> String {
        match self.platform_type {
            PlatformType::Windows => "\"Windows\"".to_string(),
            PlatformType::MacOS => "\"macOS\"".to_string(),
            PlatformType::Linux => "\"Linux\"".to_string(),
            PlatformType::Android => "\"Android\"".to_string(),
            PlatformType::IOS => "\"iOS\"".to_string(),
            PlatformType::ChromeOS => "\"Chrome OS\"".to_string(),
            PlatformType::Custom(_) => "\"Unknown\"".to_string(),
        }
    }
}

/// A simplified function to generate headers for a URL
pub fn get_header(url: &str, browser_type: Option<BrowserType>) -> Result<String> {
    let browser = browser_type.unwrap_or(BrowserType::Chrome);
    let generator = HeaderGenerator::new(browser);
    Ok(generator.generate_as_string(url))
}

/// A simplified function to generate headers as a HashMap
pub fn get_header_map(url: &str, browser_type: Option<BrowserType>) -> Result<HashMap<String, String>> {
    let browser = browser_type.unwrap_or(BrowserType::Chrome);
    let generator = HeaderGenerator::new(browser);
    Ok(generator.generate(url))
}

/// A simplified function to generate stealth headers
pub fn get_stealth_header(url: &str, browser_type: Option<BrowserType>) -> Result<String> {
    let browser = browser_type.unwrap_or(BrowserType::Chrome);
    let generator = HeaderGenerator::new(browser).with_stealth(true);
    Ok(generator.generate_as_string(url))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_header_generator_default() {
        let generator = HeaderGenerator::default();
        let headers = generator.generate("https://example.com");
        
        assert!(headers.contains_key("User-Agent"));
        assert!(headers.contains_key("Accept"));
        assert!(headers.contains_key("Accept-Language"));
        assert!(headers.contains_key("Accept-Encoding"));
        assert!(headers.contains_key("Connection"));
    }
    
    #[test]
    fn test_chrome_headers() {
        let generator = HeaderGenerator::new(BrowserType::Chrome);
        let headers = generator.generate("https://example.com");
        
        assert!(headers.contains_key("User-Agent"));
        assert!(headers["User-Agent"].contains("Chrome"));
        assert!(headers.contains_key("Sec-Ch-Ua"));
    }
    
    #[test]
    fn test_firefox_headers() {
        let generator = HeaderGenerator::new(BrowserType::Firefox);
        let headers = generator.generate("https://example.com");
        
        assert!(headers.contains_key("User-Agent"));
        assert!(headers["User-Agent"].contains("Firefox"));
        assert!(headers.contains_key("TE"));
    }
    
    #[test]
    fn test_custom_user_agent() {
        let generator = HeaderGenerator::default()
            .with_user_agent("Custom User Agent");
        let headers = generator.generate("https://example.com");
        
        assert_eq!(headers["User-Agent"], "Custom User Agent");
    }
    
    #[test]
    fn test_custom_header() {
        let generator = HeaderGenerator::default()
            .with_custom_header("X-Custom", "Value");
        let headers = generator.generate("https://example.com");
        
        assert_eq!(headers["X-Custom"], "Value");
    }
    
    #[test]
    fn test_stealth_mode() {
        let generator = HeaderGenerator::default()
            .with_stealth(true);
        let headers = generator.generate("https://example.com");
        
        assert!(headers.contains_key("Referer"));
    }
} 