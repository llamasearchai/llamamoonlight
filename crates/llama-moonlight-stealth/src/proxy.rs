//! Proxy management for stealth browser automation
//!
//! This module provides capabilities to use and rotate proxies for web requests,
//! helping to avoid IP-based blocking and tracking.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::{Duration, Instant};
use url::Url;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rand::Rng;

use crate::Result;
use crate::Error;

/// Proxy protocol
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum ProxyProtocol {
    /// HTTP proxy
    Http,
    
    /// HTTPS proxy
    Https,
    
    /// SOCKS4 proxy
    Socks4,
    
    /// SOCKS5 proxy
    Socks5,
}

impl std::fmt::Display for ProxyProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProxyProtocol::Http => write!(f, "http"),
            ProxyProtocol::Https => write!(f, "https"),
            ProxyProtocol::Socks4 => write!(f, "socks4"),
            ProxyProtocol::Socks5 => write!(f, "socks5"),
        }
    }
}

impl ProxyProtocol {
    /// Parse a proxy protocol from a string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "http" => Some(ProxyProtocol::Http),
            "https" => Some(ProxyProtocol::Https),
            "socks4" => Some(ProxyProtocol::Socks4),
            "socks4a" => Some(ProxyProtocol::Socks4),
            "socks5" => Some(ProxyProtocol::Socks5),
            "socks5h" => Some(ProxyProtocol::Socks5),
            _ => None,
        }
    }
}

/// Authentication for a proxy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProxyAuth {
    /// Username for authentication
    pub username: String,
    
    /// Password for authentication
    pub password: String,
}

impl ProxyAuth {
    /// Create a new proxy authentication
    pub fn new(username: &str, password: &str) -> Self {
        Self {
            username: username.to_string(),
            password: password.to_string(),
        }
    }
    
    /// Get the authentication as a string
    pub fn as_string(&self) -> String {
        format!("{}:{}", self.username, self.password)
    }
}

/// Proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Protocol of the proxy
    pub protocol: ProxyProtocol,
    
    /// Host of the proxy
    pub host: String,
    
    /// Port of the proxy
    pub port: u16,
    
    /// Authentication for the proxy
    pub auth: Option<ProxyAuth>,
    
    /// Country of the proxy
    pub country: Option<String>,
    
    /// Whether the proxy is active
    pub active: bool,
    
    /// Last time the proxy was used
    #[serde(skip)]
    pub last_used: Option<Instant>,
    
    /// Success count for this proxy
    pub success_count: u32,
    
    /// Failure count for this proxy
    pub failure_count: u32,
    
    /// Response time in milliseconds
    pub response_time_ms: Option<u64>,
    
    /// Custom metadata
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl ProxyConfig {
    /// Create a new proxy configuration
    pub fn new(protocol: ProxyProtocol, host: &str, port: u16) -> Self {
        Self {
            protocol,
            host: host.to_string(),
            port,
            auth: None,
            country: None,
            active: true,
            last_used: None,
            success_count: 0,
            failure_count: 0,
            response_time_ms: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Set authentication for the proxy
    pub fn with_auth(mut self, username: &str, password: &str) -> Self {
        self.auth = Some(ProxyAuth::new(username, password));
        self
    }
    
    /// Set the country of the proxy
    pub fn with_country(mut self, country: &str) -> Self {
        self.country = Some(country.to_string());
        self
    }
    
    /// Get the URL of the proxy
    pub fn to_url(&self) -> String {
        let auth_part = match &self.auth {
            Some(auth) => format!("{}:{}@", auth.username, auth.password),
            None => String::new(),
        };
        
        format!("{}://{}{}:{}", self.protocol, auth_part, self.host, self.port)
    }
    
    /// Parse a proxy from a URL string
    pub fn from_url(url_str: &str) -> Result<Self> {
        let url = Url::parse(url_str).map_err(|e| Error::ProxyError(format!("Invalid proxy URL: {}", e)))?;
        
        let protocol = match url.scheme() {
            "http" => ProxyProtocol::Http,
            "https" => ProxyProtocol::Https,
            "socks4" => ProxyProtocol::Socks4,
            "socks5" => ProxyProtocol::Socks5,
            _ => return Err(Error::ProxyError(format!("Unsupported proxy protocol: {}", url.scheme()))),
        };
        
        let host = url.host_str()
            .ok_or_else(|| Error::ProxyError("Missing proxy host".to_string()))?
            .to_string();
        
        let port = url.port().unwrap_or_else(|| match protocol {
            ProxyProtocol::Http => 80,
            ProxyProtocol::Https => 443,
            ProxyProtocol::Socks4 | ProxyProtocol::Socks5 => 1080,
        });
        
        let auth = if let Some(username) = url.username() {
            if username.is_empty() {
                None
            } else {
                let password = url.password().unwrap_or("");
                Some(ProxyAuth::new(username, password))
            }
        } else {
            None
        };
        
        Ok(Self {
            protocol,
            host,
            port,
            auth,
            country: None,
            active: true,
            last_used: None,
            success_count: 0,
            failure_count: 0,
            response_time_ms: None,
            metadata: HashMap::new(),
        })
    }
    
    /// Record a successful use of the proxy
    pub fn record_success(&mut self, response_time_ms: Option<u64>) {
        self.success_count += 1;
        self.last_used = Some(Instant::now());
        self.response_time_ms = response_time_ms;
    }
    
    /// Record a failed use of the proxy
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_used = Some(Instant::now());
    }
    
    /// Calculate the success rate of the proxy
    pub fn success_rate(&self) -> f64 {
        let total = self.success_count + self.failure_count;
        if total == 0 {
            return 0.0;
        }
        self.success_count as f64 / total as f64
    }
    
    /// Check if the proxy is healthy
    pub fn is_healthy(&self) -> bool {
        // Require at least 3 attempts to make a judgment
        if self.success_count + self.failure_count < 3 {
            return true;
        }
        
        self.success_rate() >= 0.7
    }
}

impl std::fmt::Display for ProxyConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let auth_part = match &self.auth {
            Some(_) => "<auth>@",
            None => "",
        };
        
        let country_part = match &self.country {
            Some(country) => format!(" [{}]", country),
            None => String::new(),
        };
        
        write!(f, "{}://{}{}:{}{}", self.protocol, auth_part, self.host, self.port, country_part)
    }
}

/// Manager for proxy rotation
#[derive(Debug)]
pub struct ProxyManager {
    /// List of available proxies
    proxies: Vec<ProxyConfig>,
    
    /// Currently active proxy
    active_proxy: Option<usize>,
    
    /// Rotation strategy
    rotation_strategy: RotationStrategy,
    
    /// Maximum failures before removing a proxy
    max_failures: u32,
    
    /// Minimum success rate to keep a proxy
    min_success_rate: f64,
}

/// Strategy for rotating proxies
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RotationStrategy {
    /// Rotate to the next proxy in the list
    RoundRobin,
    
    /// Choose a random proxy
    Random,
    
    /// Choose the least recently used proxy
    LeastRecentlyUsed,
    
    /// Choose the most successful proxy
    MostSuccessful,
    
    /// Fixed proxy (no rotation)
    Fixed,
}

impl ProxyManager {
    /// Create a new proxy manager
    pub fn new() -> Self {
        Self {
            proxies: Vec::new(),
            active_proxy: None,
            rotation_strategy: RotationStrategy::RoundRobin,
            max_failures: 5,
            min_success_rate: 0.7,
        }
    }
    
    /// Create a proxy manager with a specific rotation strategy
    pub fn with_strategy(strategy: RotationStrategy) -> Self {
        Self {
            proxies: Vec::new(),
            active_proxy: None,
            rotation_strategy: strategy,
            max_failures: 5,
            min_success_rate: 0.7,
        }
    }
    
    /// Add a proxy to the manager
    pub fn add_proxy(&mut self, proxy: ProxyConfig) -> &mut Self {
        self.proxies.push(proxy);
        
        // Set the first added proxy as active if none is active
        if self.active_proxy.is_none() && !self.proxies.is_empty() {
            self.active_proxy = Some(0);
        }
        
        self
    }
    
    /// Add multiple proxies to the manager
    pub fn add_proxies(&mut self, proxies: Vec<ProxyConfig>) -> &mut Self {
        for proxy in proxies {
            self.add_proxy(proxy);
        }
        self
    }
    
    /// Add proxies from URLs
    pub fn add_proxies_from_urls(&mut self, urls: &[&str]) -> Result<&mut Self> {
        for url in urls {
            let proxy = ProxyConfig::from_url(url)?;
            self.add_proxy(proxy);
        }
        Ok(self)
    }
    
    /// Set the maximum number of failures before removing a proxy
    pub fn with_max_failures(mut self, max_failures: u32) -> Self {
        self.max_failures = max_failures;
        self
    }
    
    /// Set the minimum success rate to keep a proxy
    pub fn with_min_success_rate(mut self, min_success_rate: f64) -> Self {
        self.min_success_rate = min_success_rate;
        self
    }
    
    /// Get the current active proxy
    pub fn active_proxy(&self) -> Option<&ProxyConfig> {
        self.active_proxy.and_then(|idx| self.proxies.get(idx))
    }
    
    /// Get the current active proxy URL
    pub fn active_proxy_url(&self) -> Option<String> {
        self.active_proxy().map(|proxy| proxy.to_url())
    }
    
    /// Rotate to the next proxy based on the strategy
    pub fn rotate(&mut self) -> Option<&ProxyConfig> {
        if self.proxies.is_empty() {
            self.active_proxy = None;
            return None;
        }
        
        // Clean up unhealthy proxies
        self.cleanup_unhealthy_proxies();
        
        if self.proxies.is_empty() {
            self.active_proxy = None;
            return None;
        }
        
        let next_idx = match self.rotation_strategy {
            RotationStrategy::RoundRobin => {
                // Get the next proxy in the list
                let current = self.active_proxy.unwrap_or(0);
                (current + 1) % self.proxies.len()
            },
            RotationStrategy::Random => {
                // Choose a random proxy
                let mut rng = thread_rng();
                rng.gen_range(0..self.proxies.len())
            },
            RotationStrategy::LeastRecentlyUsed => {
                // Find the least recently used proxy
                let now = Instant::now();
                self.proxies
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, p)| {
                        p.last_used.map(|t| now.duration_since(t)).unwrap_or(Duration::from_secs(0))
                    })
                    .map(|(idx, _)| idx)
                    .unwrap_or(0)
            },
            RotationStrategy::MostSuccessful => {
                // Find the most successful proxy
                self.proxies
                    .iter()
                    .enumerate()
                    .max_by(|(_, a), (_, b)| {
                        a.success_rate().partial_cmp(&b.success_rate()).unwrap()
                    })
                    .map(|(idx, _)| idx)
                    .unwrap_or(0)
            },
            RotationStrategy::Fixed => {
                // Keep using the same proxy
                self.active_proxy.unwrap_or(0)
            },
        };
        
        self.active_proxy = Some(next_idx);
        self.active_proxy()
    }
    
    /// Record a successful use of the current proxy
    pub fn record_success(&mut self, response_time_ms: Option<u64>) {
        if let Some(idx) = self.active_proxy {
            if let Some(proxy) = self.proxies.get_mut(idx) {
                proxy.record_success(response_time_ms);
            }
        }
    }
    
    /// Record a failed use of the current proxy
    pub fn record_failure(&mut self) {
        if let Some(idx) = self.active_proxy {
            if let Some(proxy) = self.proxies.get_mut(idx) {
                proxy.record_failure();
            }
        }
    }
    
    /// Remove unhealthy proxies
    fn cleanup_unhealthy_proxies(&mut self) {
        self.proxies.retain(|proxy| {
            proxy.failure_count < self.max_failures && 
            (proxy.success_count + proxy.failure_count < 3 || proxy.success_rate() >= self.min_success_rate)
        });
    }
    
    /// Generate a random proxy (for testing or prototyping)
    pub fn generate_random_proxy() -> ProxyConfig {
        let mut rng = thread_rng();
        let protocols = [
            ProxyProtocol::Http,
            ProxyProtocol::Https,
            ProxyProtocol::Socks4,
            ProxyProtocol::Socks5,
        ];
        
        let protocol = protocols.choose(&mut rng).unwrap().clone();
        
        // Generate a random IP address
        let ip = Ipv4Addr::new(
            rng.gen_range(1..255),
            rng.gen_range(1..255),
            rng.gen_range(1..255),
            rng.gen_range(1..255),
        );
        
        // Generate a random port
        let port = rng.gen_range(1025..65535);
        
        ProxyConfig::new(protocol, &ip.to_string(), port)
    }
}

impl Default for ProxyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_proxy_config() {
        let proxy = ProxyConfig::new(ProxyProtocol::Http, "proxy.example.com", 8080)
            .with_auth("user", "pass")
            .with_country("US");
        
        assert_eq!(proxy.protocol, ProxyProtocol::Http);
        assert_eq!(proxy.host, "proxy.example.com");
        assert_eq!(proxy.port, 8080);
        assert_eq!(proxy.auth, Some(ProxyAuth::new("user", "pass")));
        assert_eq!(proxy.country, Some("US".to_string()));
        
        let url = proxy.to_url();
        assert_eq!(url, "http://user:pass@proxy.example.com:8080");
    }
    
    #[test]
    fn test_proxy_from_url() {
        let url = "http://user:pass@proxy.example.com:8080";
        let proxy = ProxyConfig::from_url(url).unwrap();
        
        assert_eq!(proxy.protocol, ProxyProtocol::Http);
        assert_eq!(proxy.host, "proxy.example.com");
        assert_eq!(proxy.port, 8080);
        assert_eq!(proxy.auth, Some(ProxyAuth::new("user", "pass")));
    }
    
    #[test]
    fn test_proxy_manager_round_robin() {
        let mut manager = ProxyManager::with_strategy(RotationStrategy::RoundRobin);
        
        // Add some test proxies
        manager.add_proxy(ProxyConfig::new(ProxyProtocol::Http, "proxy1.example.com", 8080));
        manager.add_proxy(ProxyConfig::new(ProxyProtocol::Http, "proxy2.example.com", 8080));
        manager.add_proxy(ProxyConfig::new(ProxyProtocol::Http, "proxy3.example.com", 8080));
        
        // First active proxy should be the first one
        let active = manager.active_proxy().unwrap();
        assert_eq!(active.host, "proxy1.example.com");
        
        // Rotate to next proxy
        let active = manager.rotate().unwrap();
        assert_eq!(active.host, "proxy2.example.com");
        
        // Rotate again
        let active = manager.rotate().unwrap();
        assert_eq!(active.host, "proxy3.example.com");
        
        // Rotate again, should wrap around
        let active = manager.rotate().unwrap();
        assert_eq!(active.host, "proxy1.example.com");
    }
    
    #[test]
    fn test_proxy_success_rate() {
        let mut proxy = ProxyConfig::new(ProxyProtocol::Http, "proxy.example.com", 8080);
        
        // No use yet
        assert_eq!(proxy.success_rate(), 0.0);
        
        // Record some successes and failures
        proxy.record_success(Some(100));
        proxy.record_success(Some(150));
        proxy.record_failure();
        
        // 2 successes, 1 failure = 2/3 = 0.67
        assert!((proxy.success_rate() - 0.67).abs() < 0.01);
        
        // More successes
        proxy.record_success(Some(120));
        proxy.record_success(Some(130));
        
        // 4 successes, 1 failure = 4/5 = 0.8
        assert!((proxy.success_rate() - 0.8).abs() < 0.01);
    }
    
    #[test]
    fn test_proxy_manager_cleanup() {
        let mut manager = ProxyManager::with_strategy(RotationStrategy::RoundRobin)
            .with_max_failures(3)
            .with_min_success_rate(0.6);
        
        // Add some test proxies
        manager.add_proxy(ProxyConfig::new(ProxyProtocol::Http, "proxy1.example.com", 8080));
        manager.add_proxy(ProxyConfig::new(ProxyProtocol::Http, "proxy2.example.com", 8080));
        manager.add_proxy(ProxyConfig::new(ProxyProtocol::Http, "proxy3.example.com", 8080));
        
        // Make proxy2 unhealthy with too many failures
        manager.active_proxy = Some(1); // Select proxy2
        manager.record_failure();
        manager.record_failure();
        manager.record_failure(); // 3 failures, should be removed
        
        // Rotate, which should also cleanup
        manager.rotate();
        
        // Should have removed proxy2
        assert_eq!(manager.proxies.len(), 2);
        assert_eq!(manager.proxies[0].host, "proxy1.example.com");
        assert_eq!(manager.proxies[1].host, "proxy3.example.com");
    }
} 