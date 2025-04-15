//! # Llama Headers RS
//! 
//! `llama-headers-rs` is a sophisticated HTTP header generation library for realistic browser emulation.
//! This crate helps in generating headers that mimic real browsers to avoid detection by anti-bot systems.
//!
//! ## Features
//!
//! - User-agent generation with realistic browser fingerprints
//! - Language and locale-aware header generation
//! - Referer generation based on domain context
//! - Sec-CH-UA headers for modern browser emulation
//! - Mobile/desktop device simulation
//! - Configurable via TOML files
//!
//! ## Example
//!
//! ```
//! use llama_headers_rs::{get_header, Config};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Simple usage with defaults
//!     let header = get_header("https://example.com", None)?;
//!     println!("{}", header);
//!     
//!     // Advanced usage with configuration
//!     let config = Config {
//!         language: Some("de-DE".to_string()),
//!         mobile: Some(true),
//!         ..Default::default()
//!     };
//!     
//!     let header = get_header("https://example.de", Some(config))?;
//!     println!("{}", header);
//!     
//!     Ok(())
//! }
//! ```

pub mod header;
pub mod user_agent;
pub mod utils;
pub mod errors;
pub mod config;

use crate::header::Header;
use crate::user_agent::UserAgent;
use crate::utils::{get_domain, get_random_referer, get_sec_ch_ua, get_accept_encoding, get_accept_language, get_sec_fetch_dest, get_sec_fetch_mode, get_sec_fetch_site, get_sec_fetch_user, get_connection};
use crate::errors::LlamaHeadersError;
use crate::config::Config;
use std::collections::HashMap;

/// Generates a single `Header` instance based on the provided parameters.
///
/// # Arguments
///
/// * `url` - The URL for which to generate headers.
/// * `config` - Optional configuration settings.
///
/// # Returns
///
/// A `Result` containing a `Header` instance or a `LlamaHeadersError`.
pub fn get_header(url: &str, config: Option<Config>) -> Result<Header, LlamaHeadersError> {
    let config = config.unwrap_or_default();  // Use default config if none provided
    let domain = get_domain(url)?;
    let language = config.language.unwrap_or_else(|| utils::get_language_from_domain(&domain));
    let user_agent = match config.user_agent {
        Some(ua) => ua,
        None => UserAgent::get_random_user_agent(config.mobile.unwrap_or(false))?,
    };

    let referer = config.referer.unwrap_or_else(||get_random_referer(&language, &domain).unwrap_or(domain.clone()));
    let sec_ch_ua = get_sec_ch_ua(&user_agent);
    let accept_encoding = get_accept_encoding();
    let accept_language = get_accept_language(&language);

    let mut headers = HashMap::new();
    headers.insert("Host".to_string(), domain.clone());
    headers.insert("Connection".to_string(), get_connection());
    headers.insert("Upgrade-Insecure-Requests".to_string(), "1".to_string()); //Always 1 for simplicity
    headers.insert("User-Agent".to_string(), user_agent.to_string());
    headers.insert("Accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7".to_string()); // A common accept header
    headers.insert("Sec-Fetch-Site".to_string(), get_sec_fetch_site(&referer, &domain));
    headers.insert("Sec-Fetch-Mode".to_string(), get_sec_fetch_mode());
    headers.insert("Sec-Fetch-User".to_string(), get_sec_fetch_user());
    headers.insert("Sec-Fetch-Dest".to_string(), get_sec_fetch_dest());
    headers.insert("Referer".to_string(), referer);
    
    if let Some(sec_ch_ua_value) = sec_ch_ua {
        headers.insert("Sec-Ch-Ua".to_string(), sec_ch_ua_value.clone());
        if user_agent.is_mobile() {
            headers.insert("Sec-Ch-Ua-Mobile".to_string(), "?1".to_string());
        } else {
            headers.insert("Sec-Ch-Ua-Mobile".to_string(), "?0".to_string());
        }
        headers.insert("Sec-Ch-Ua-Platform".to_string(), format!("\"{}\"",user_agent.get_platform_for_sec_ch_ua()));
    }

    headers.insert("Accept-Encoding".to_string(), accept_encoding);
    headers.insert("Accept-Language".to_string(), accept_language);

    Ok(Header::new(user_agent, headers))
}

/// Generates multiple `Header` instances.
///
/// # Arguments
///
/// * `url` - The URL for which to generate headers.
/// * `num` - The number of `Header` instances to generate.
/// * `config` - Optional configuration settings.
///
/// # Returns
///
/// A `Result` containing a `Vec<Header>` or a `LlamaHeadersError`.
pub fn get_headers(url: &str, num: usize, config: Option<Config>) -> Result<Vec<Header>, LlamaHeadersError> {
    let mut headers_list = Vec::with_capacity(num);
    for _ in 0..num {
        headers_list.push(get_header(url, config.clone())?);
    }
    Ok(headers_list)
} 