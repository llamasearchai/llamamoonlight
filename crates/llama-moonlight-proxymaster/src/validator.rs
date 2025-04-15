//! Validator module.
//! Provides functionality for checking if proxies are working.

use crate::models::Proxy;
use chrono::Utc;
use log::{debug, error, info};
use reqwest::{Client, Proxy as ReqwestProxy};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::timeout;

/// Configuration for the validator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorConfig {
    /// URL to test HTTP proxies.
    pub http_test_url: String,
    
    /// URL to test HTTPS proxies.
    pub https_test_url: String,
    
    /// Connect timeout in seconds.
    pub connect_timeout: u64,
    
    /// Request timeout in seconds.
    pub request_timeout: u64,
    
    /// Whether to check for anonymity.
    pub check_anonymity: bool,
    
    /// Whether to check for country.
    pub check_country: bool,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            http_test_url: "http://httpbin.org/ip".to_string(),
            https_test_url: "https://httpbin.org/ip".to_string(),
            connect_timeout: 5,
            request_timeout: 10,
            check_anonymity: true,
            check_country: true,
        }
    }
}

/// Result of validating a proxy.
#[derive(Debug)]
pub struct ValidationResult {
    /// Whether the proxy is working.
    pub is_working: bool,
    
    /// Response time in milliseconds (if working).
    pub response_time: Option<i64>,
    
    /// True if the proxy is anonymous (if checked).
    pub is_anonymous: Option<bool>,
    
    /// Country code (if detected).
    pub country: Option<String>,
    
    /// Error message (if not working).
    pub error: Option<String>,
}

/// Validates a proxy.
pub async fn validate_proxy(proxy: &mut Proxy, config: &ValidatorConfig) -> ValidationResult {
    let start_time = std::time::Instant::now();
    
    // Build a reqwest client with the proxy
    let client_builder = Client::builder()
        .connect_timeout(Duration::from_secs(config.connect_timeout))
        .timeout(Duration::from_secs(config.request_timeout));
    
    // HTTP proxy
    let reqwest_proxy = if proxy.https {
        ReqwestProxy::https(&proxy.as_str())
    } else {
        ReqwestProxy::http(&proxy.as_str())
    };
    
    let client = match reqwest_proxy.and_then(|p| client_builder.proxy(p).build()) {
        Ok(client) => client,
        Err(e) => {
            return ValidationResult {
                is_working: false,
                response_time: None,
                is_anonymous: None,
                country: None,
                error: Some(format!("Failed to build client: {}", e)),
            };
        }
    };
    
    // Test the proxy
    let test_url = if proxy.https {
        &config.https_test_url
    } else {
        &config.http_test_url
    };
    
    // Set a timeout for the request
    let request_timeout = Duration::from_secs(config.request_timeout);
    let response = match timeout(request_timeout, client.get(test_url).send()).await {
        Ok(result) => result,
        Err(_) => {
            return ValidationResult {
                is_working: false,
                response_time: None,
                is_anonymous: None,
                country: None,
                error: Some("Request timed out".to_string()),
            };
        }
    };
    
    // Check if the request was successful
    match response {
        Ok(resp) => {
            // Calculate response time
            let elapsed = start_time.elapsed();
            let response_time = elapsed.as_millis() as i64;
            
            // Check status code
            if !resp.status().is_success() {
                return ValidationResult {
                    is_working: false,
                    response_time: Some(response_time),
                    is_anonymous: None,
                    country: None,
                    error: Some(format!("HTTP error: {}", resp.status())),
                };
            }
            
            // Try to parse the response
            match resp.text().await {
                Ok(_) => {
                    // Update proxy fields
                    proxy.last_checked = Some(Utc::now());
                    proxy.response_time = Some(response_time);
                    
                    // Increment success rate
                    proxy.success_rate = proxy.success_rate * 0.8 + 0.2;
                    
                    // Adjust weight based on response time
                    // Lower response time = higher weight (max 10)
                    if response_time < 100 {
                        proxy.weight = 10.0;
                    } else if response_time < 200 {
                        proxy.weight = 8.0;
                    } else if response_time < 500 {
                        proxy.weight = 5.0;
                    } else if response_time < 1000 {
                        proxy.weight = 3.0;
                    } else {
                        proxy.weight = 1.0;
                    }
                    
                    ValidationResult {
                        is_working: true,
                        response_time: Some(response_time),
                        is_anonymous: None,
                        country: None,
                        error: None,
                    }
                },
                Err(e) => {
                    // Decrement success rate
                    proxy.success_rate = proxy.success_rate * 0.8;
                    
                    ValidationResult {
                        is_working: false,
                        response_time: Some(response_time),
                        is_anonymous: None,
                        country: None,
                        error: Some(format!("Failed to parse response: {}", e)),
                    }
                }
            }
        },
        Err(e) => {
            // Decrement success rate
            proxy.success_rate = proxy.success_rate * 0.8;
            
            ValidationResult {
                is_working: false,
                response_time: None,
                is_anonymous: None,
                country: None,
                error: Some(format!("Request failed: {}", e)),
            }
        }
    }
}

/// Validates multiple proxies concurrently.
pub async fn validate_proxies(
    proxies: &mut [Proxy],
    config: &ValidatorConfig,
    concurrency: usize,
) -> Vec<ValidationResult> {
    use futures::stream::{self, StreamExt};
    
    info!("Validating {} proxies with concurrency {}", proxies.len(), concurrency);
    
    let results = stream::iter(proxies.iter_mut())
        .map(|proxy| async move {
            let result = validate_proxy(proxy, config).await;
            (proxy.id, result)
        })
        .buffer_unordered(concurrency)
        .collect::<Vec<_>>()
        .await;
    
    // Create a map to lookup results by proxy ID
    let mut result_map = std::collections::HashMap::new();
    for (id, result) in results {
        result_map.insert(id, result);
    }
    
    // Return results in the same order as input proxies
    proxies
        .iter()
        .map(|proxy| {
            result_map
                .remove(&proxy.id)
                .unwrap_or_else(|| ValidationResult {
                    is_working: false,
                    response_time: None,
                    is_anonymous: None,
                    country: None,
                    error: Some("Result not found".to_string()),
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_validation_invalid_proxy() {
        let mut proxy = Proxy::new("0.0.0.0".to_string(), 1, false); // Invalid proxy
        let config = ValidatorConfig::default();
        
        let result = validate_proxy(&mut proxy, &config).await;
        assert!(!result.is_working);
        assert!(result.error.is_some());
    }
} 