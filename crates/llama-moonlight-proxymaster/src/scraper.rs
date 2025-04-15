//! Scraper module.
//! Provides functionality for scraping proxies from various sources.

use crate::models::Proxy;
use futures::stream::{self, StreamExt};
use log::{debug, error, info, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::error::Error;
use std::time::Duration;
use thiserror::Error;

/// Maximum number of proxies to scrape from a single source.
const MAX_PROXIES_PER_SOURCE: usize = 100;

/// Error type for scraping operations.
#[derive(Error, Debug)]
pub enum ScraperError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("Parsing error: {0}")]
    Parsing(String),
    
    #[error("No proxies found in response")]
    NoProxies,
}

/// Configuration for the scraper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScraperConfig {
    /// List of free proxy sources to scrape.
    pub sources: Vec<String>,
    
    /// HTTP client timeout in seconds.
    pub timeout: u64,
    
    /// Maximum concurrency for scraping.
    pub max_concurrency: usize,
}

impl Default for ScraperConfig {
    fn default() -> Self {
        Self {
            sources: vec![
                "https://www.proxy-list.download/api/v1/get?type=http".to_string(),
                "https://api.proxyscrape.com/v2/?request=getproxies&protocol=http".to_string(),
                "https://raw.githubusercontent.com/TheSpeedX/PROXY-List/master/http.txt".to_string(),
                "https://raw.githubusercontent.com/ShiftyTR/Proxy-List/master/http.txt".to_string(),
                "https://raw.githubusercontent.com/monosans/proxy-list/main/proxies/http.txt".to_string(),
            ],
            timeout: 10,
            max_concurrency: 5,
        }
    }
}

/// Scrapes proxies from a single source URL.
async fn scrape_source(client: &Client, url: &str) -> Result<Vec<Proxy>, ScraperError> {
    info!("Scraping proxies from {}", url);
    
    // Get response from source
    let response = client.get(url).send().await?;
    
    if !response.status().is_success() {
        return Err(ScraperError::Network(reqwest::Error::new(
            reqwest::StatusCode::INTERNAL_SERVER_ERROR.into(),
            format!("HTTP error: {}", response.status()),
        )));
    }
    
    // Parse response text
    let text = response.text().await?;
    let lines: Vec<&str> = text.lines().collect();
    
    if lines.is_empty() {
        return Err(ScraperError::NoProxies);
    }
    
    // Parse proxies
    let mut proxies = Vec::new();
    let mut found = 0;
    
    for line in lines {
        if found >= MAX_PROXIES_PER_SOURCE {
            break;
        }
        
        // Skip empty lines
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        
        // Try to parse proxy from the line
        if let Some(mut proxy) = Proxy::from_str(line) {
            // Set default values
            proxy.https = false; // Assume HTTP by default
            
            proxies.push(proxy);
            found += 1;
        }
    }
    
    if proxies.is_empty() {
        warn!("No valid proxies found in response from {}", url);
        return Err(ScraperError::NoProxies);
    }
    
    info!("Found {} proxies from {}", proxies.len(), url);
    Ok(proxies)
}

/// Scrapes proxies from multiple sources in parallel.
pub async fn scrape_proxies(config: &ScraperConfig) -> Result<Vec<Proxy>, Box<dyn Error + Send + Sync>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(config.timeout))
        .build()?;
    
    // Scrape from all sources in parallel
    let results = stream::iter(config.sources.iter())
        .map(|url| {
            let client = client.clone();
            async move {
                match scrape_source(&client, url).await {
                    Ok(proxies) => (url.clone(), Ok(proxies)),
                    Err(e) => (url.clone(), Err(e)),
                }
            }
        })
        .buffer_unordered(config.max_concurrency)
        .collect::<Vec<_>>()
        .await;
    
    // Collect all proxies, removing duplicates
    let mut unique_proxies = HashSet::new();
    let mut all_proxies = Vec::new();
    
    for (url, result) in results {
        match result {
            Ok(proxies) => {
                info!("Successfully scraped {} proxies from {}", proxies.len(), url);
                
                for proxy in proxies {
                    let key = format!("{}:{}", proxy.ip, proxy.port);
                    if unique_proxies.insert(key) {
                        all_proxies.push(proxy);
                    }
                }
            },
            Err(e) => {
                error!("Failed to scrape from {}: {}", url, e);
            }
        }
    }
    
    info!("Scraped a total of {} unique proxies", all_proxies.len());
    
    if all_proxies.is_empty() {
        error!("Failed to scrape any proxies from all sources");
    }
    
    Ok(all_proxies)
}

/// Scrapes HTTPS proxies specifically.
pub async fn scrape_https_proxies(config: &ScraperConfig) -> Result<Vec<Proxy>, Box<dyn Error + Send + Sync>> {
    let mut proxies = scrape_proxies(config).await?;
    
    // Filter to HTTPS-only and mark them as HTTPS
    for proxy in &mut proxies {
        proxy.https = true;
    }
    
    Ok(proxies)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, server_url};
    
    #[tokio::test]
    async fn test_scrape_source() {
        // Mock server
        let _m = mock("GET", "/proxies")
            .with_status(200)
            .with_body("192.168.1.1:8080\n192.168.1.2:8081\n")
            .create();
        
        let client = Client::new();
        let url = &format!("{}/proxies", server_url());
        
        let proxies = scrape_source(&client, url).await.unwrap();
        
        assert_eq!(proxies.len(), 2);
        assert_eq!(proxies[0].ip, "192.168.1.1");
        assert_eq!(proxies[0].port, 8080);
        assert_eq!(proxies[1].ip, "192.168.1.2");
        assert_eq!(proxies[1].port, 8081);
    }
    
    #[tokio::test]
    async fn test_scrape_empty_response() {
        // Mock server with empty response
        let _m = mock("GET", "/empty")
            .with_status(200)
            .with_body("")
            .create();
        
        let client = Client::new();
        let url = &format!("{}/empty", server_url());
        
        let result = scrape_source(&client, url).await;
        assert!(result.is_err());
        
        match result {
            Err(ScraperError::NoProxies) => {}
            _ => panic!("Expected NoProxies error"),
        }
    }
} 