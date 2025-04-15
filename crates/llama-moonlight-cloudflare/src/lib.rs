//! # Llama Moonlight Cloudflare
//!
//! A Rust crate for bypassing Cloudflare protection in web scraping applications.
//! This crate provides tools to handle Cloudflare challenges, including IUAM
//! (I'm Under Attack Mode) challenges and CAPTCHAs.

use anyhow::Result;
use lazy_static::lazy_static;
use log::{debug, error, info, warn};
use reqwest::{Client, Response, StatusCode};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use thiserror::Error;
use url::Url;

pub mod challenge;
pub mod cookie;
pub mod fingerprint;
pub mod headers;
pub mod javascript;
pub mod proxy;
pub mod tls;
pub mod useragent;
pub mod captcha;
pub mod util;
pub mod solvers;
pub mod client;
pub mod sessions;

pub use client::CloudflareClient;
pub use challenge::{Challenge, ChallengeType, ChallengeSolution};
pub use sessions::Session;

/// Cloudflare bypass errors
#[derive(Error, Debug)]
pub enum CloudflareError {
    /// Error when a Cloudflare challenge is encountered
    #[error("Cloudflare challenge detected: {0}")]
    ChallengeDetected(String),
    
    /// Error when solving a Cloudflare challenge
    #[error("Failed to solve Cloudflare challenge: {0}")]
    ChallengeSolvingFailed(String),
    
    /// Error when handling cookies
    #[error("Cookie error: {0}")]
    CookieError(String),
    
    /// Error with HTTP client
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    
    /// Error when parsing HTML
    #[error("HTML parsing error: {0}")]
    HtmlParsingError(String),
    
    /// Error when evaluating JavaScript
    #[error("JavaScript error: {0}")]
    JavaScriptError(String),
    
    /// Error when CAPTCHA is required
    #[error("CAPTCHA required: {0}")]
    CaptchaRequired(String),
    
    /// Error when solving CAPTCHA
    #[error("CAPTCHA solving failed: {0}")]
    CaptchaSolvingFailed(String),
    
    /// Error with proxy
    #[error("Proxy error: {0}")]
    ProxyError(String),
    
    /// Error when rate limited
    #[error("Rate limited: {0}")]
    RateLimited(String),
    
    /// Error when IP is banned
    #[error("IP banned: {0}")]
    IpBanned(String),
    
    /// Other errors
    #[error("Other error: {0}")]
    Other(String),
}

/// Cloudflare bypass configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareConfig {
    /// User agent to use
    pub user_agent: Option<String>,
    
    /// Whether to use stealth mode
    pub stealth_mode: bool,
    
    /// Whether to follow redirects
    pub follow_redirects: bool,
    
    /// Maximum number of retries for challenges
    pub max_retries: usize,
    
    /// Timeout for requests in seconds
    pub timeout_seconds: u64,
    
    /// Whether to solve CAPTCHAs
    pub solve_captchas: bool,
    
    /// CAPTCHA API key
    pub captcha_api_key: Option<String>,
    
    /// CAPTCHA provider
    pub captcha_provider: Option<String>,
    
    /// Proxy URL
    pub proxy: Option<String>,
    
    /// List of proxies to rotate through
    pub proxy_list: Option<Vec<String>>,
    
    /// Whether to rotate proxies
    pub rotate_proxies: bool,
    
    /// Custom headers
    pub custom_headers: Option<HashMap<String, String>>,
    
    /// Cookie jar to use
    pub cookies: Option<HashMap<String, String>>,
    
    /// Whether to use Cloudflare clearance cookies
    pub use_clearance: bool,
}

impl Default for CloudflareConfig {
    fn default() -> Self {
        Self {
            user_agent: None,
            stealth_mode: true,
            follow_redirects: true,
            max_retries: 3,
            timeout_seconds: 30,
            solve_captchas: false,
            captcha_api_key: None,
            captcha_provider: None,
            proxy: None,
            proxy_list: None,
            rotate_proxies: false,
            custom_headers: None,
            cookies: None,
            use_clearance: true,
        }
    }
}

/// Check if a response is a Cloudflare challenge
pub fn is_cloudflare_challenge(response: &Response) -> bool {
    // Check for common Cloudflare challenge signatures
    let status = response.status();
    let cf_ray = response.headers().get("cf-ray").is_some();
    
    if status == StatusCode::FORBIDDEN || status == StatusCode::UNAUTHORIZED {
        if cf_ray {
            return true;
        }
    }
    
    if status == StatusCode::TOO_MANY_REQUESTS && cf_ray {
        return true;
    }
    
    if status == StatusCode::SERVICE_UNAVAILABLE && cf_ray {
        let body = match response.text() {
            Ok(body) => body,
            Err(_) => return false,
        };
        
        if body.contains("Checking your browser") || body.contains("security challenge") {
            return true;
        }
    }
    
    false
}

/// Check if a response is a Cloudflare CAPTCHA
pub fn is_cloudflare_captcha(response: &Response) -> bool {
    let status = response.status();
    let cf_ray = response.headers().get("cf-ray").is_some();
    
    if (status == StatusCode::FORBIDDEN || status == StatusCode::UNAUTHORIZED) && cf_ray {
        let body = match response.text() {
            Ok(body) => body,
            Err(_) => return false,
        };
        
        if body.contains("captcha") || body.contains("CAPTCHA") {
            return true;
        }
    }
    
    false
}

/// Extract a Cloudflare challenge from a response
pub fn extract_challenge(response: &Response) -> Result<Challenge, CloudflareError> {
    // Extract the challenge parameters from the response
    let body = match response.text() {
        Ok(body) => body,
        Err(e) => return Err(CloudflareError::HtmlParsingError(e.to_string())),
    };
    
    // Check for different types of challenges
    if body.contains("jschl_vc") && body.contains("jschl_answer") {
        // IUAM challenge
        challenge::extract_iuam_challenge(&body)
    } else if body.contains("captcha") || body.contains("CAPTCHA") {
        // CAPTCHA challenge
        challenge::extract_captcha_challenge(&body)
    } else if body.contains("turnstile") || body.contains("Turnstile") {
        // Turnstile challenge
        challenge::extract_turnstile_challenge(&body)
    } else {
        Err(CloudflareError::ChallengeDetected("Unknown challenge type".to_string()))
    }
}

/// Common Cloudflare bypass functionality
pub fn get_default_bypass_headers(url: &str) -> Result<HashMap<String, String>, CloudflareError> {
    let headers = llama_headers_rs::get_header(url, None);
    
    match headers {
        Ok(header_str) => {
            let mut header_map = HashMap::new();
            
            // Parse header string into map
            for line in header_str.lines() {
                if line.is_empty() || !line.contains(":") {
                    continue;
                }
                
                let parts: Vec<&str> = line.splitn(2, ":").collect();
                if parts.len() != 2 {
                    continue;
                }
                
                let key = parts[0].trim();
                let value = parts[1].trim();
                
                header_map.insert(key.to_string(), value.to_string());
            }
            
            Ok(header_map)
        }
        Err(e) => Err(CloudflareError::Other(format!("Failed to get headers: {}", e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_is_cloudflare_challenge() {
        // This is a placeholder test
        // In a real implementation, we would mock a response and test it
    }
} 