//! Models module.
//! Defines data structures used throughout the application.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Proxy model representing a single proxy server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proxy {
    /// Unique identifier for the proxy.
    pub id: Uuid,
    
    /// IP address of the proxy.
    pub ip: String,
    
    /// Port number of the proxy.
    pub port: u16,
    
    /// Country code where the proxy is located (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    
    /// Anonymity level of the proxy (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anonymity: Option<String>,
    
    /// Whether the proxy supports HTTPS.
    pub https: bool,
    
    /// List of supported protocols (http, socks4, socks5).
    pub protocols: Vec<String>,
    
    /// Last time the proxy was checked for validity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_checked: Option<DateTime<Utc>>,
    
    /// Response time in milliseconds (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_time: Option<i64>,
    
    /// Weight used for weighted selection (higher is better).
    pub weight: f32,
    
    /// Success rate (0.0 to 1.0).
    pub success_rate: f32,
}

impl Proxy {
    /// Creates a new proxy with default values.
    pub fn new(ip: String, port: u16, https: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            ip,
            port,
            country: None,
            anonymity: None,
            https,
            protocols: vec!["http".to_string()],
            last_checked: None,
            response_time: None,
            weight: 1.0,
            success_rate: 0.0,
        }
    }
    
    /// Returns the proxy as a string in the format "ip:port".
    pub fn as_str(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
    
    /// Returns the proxy as a URL for use with reqwest.
    pub fn as_url(&self) -> String {
        let protocol = if self.https { "https" } else { "http" };
        format!("{}://{}:{}", protocol, self.ip, self.port)
    }
    
    /// Parses a proxy from a string in the format "ip:port".
    pub fn from_str(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return None;
        }
        
        let ip = parts[0].to_string();
        let port = parts[1].parse::<u16>().ok()?;
        
        Some(Self::new(ip, port, false))
    }
}

impl fmt::Display for Proxy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.ip, self.port)
    }
}

/// Selection strategy for proxy rotation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SelectionStrategy {
    /// Random selection (uniform).
    Random,
    
    /// Weighted random selection based on proxy weight.
    Weighted,
    
    /// Round-robin selection.
    RoundRobin,
    
    /// Select the fastest proxy.
    Fastest,
}

impl Default for SelectionStrategy {
    fn default() -> Self {
        Self::Weighted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_proxy_creation() {
        let proxy = Proxy::new("127.0.0.1".to_string(), 8080, true);
        
        assert_eq!(proxy.ip, "127.0.0.1");
        assert_eq!(proxy.port, 8080);
        assert!(proxy.https);
        assert_eq!(proxy.protocols, vec!["http".to_string()]);
        assert_eq!(proxy.weight, 1.0);
        assert_eq!(proxy.success_rate, 0.0);
    }
    
    #[test]
    fn test_proxy_as_str() {
        let proxy = Proxy::new("127.0.0.1".to_string(), 8080, true);
        assert_eq!(proxy.as_str(), "127.0.0.1:8080");
    }
    
    #[test]
    fn test_proxy_as_url() {
        let proxy = Proxy::new("127.0.0.1".to_string(), 8080, true);
        assert_eq!(proxy.as_url(), "https://127.0.0.1:8080");
        
        let proxy = Proxy::new("127.0.0.1".to_string(), 8080, false);
        assert_eq!(proxy.as_url(), "http://127.0.0.1:8080");
    }
    
    #[test]
    fn test_proxy_from_str() {
        let proxy_str = "192.168.1.1:8080";
        let proxy = Proxy::from_str(proxy_str).unwrap();
        
        assert_eq!(proxy.ip, "192.168.1.1");
        assert_eq!(proxy.port, 8080);
        assert!(!proxy.https);
        
        // Invalid format
        assert!(Proxy::from_str("invalid").is_none());
        assert!(Proxy::from_str("127.0.0.1:abc").is_none());
    }
} 