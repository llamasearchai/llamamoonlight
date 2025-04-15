use crate::CloudflareError;
use rand::prelude::*;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

/// A proxy with its details and metadata
#[derive(Debug, Clone)]
pub struct Proxy {
    /// Full proxy URL (e.g., http://user:pass@host:port)
    pub url: String,
    /// Proxy host
    pub host: String,
    /// Proxy port
    pub port: u16,
    /// Proxy username, if any
    pub username: Option<String>,
    /// Proxy password, if any
    pub password: Option<String>,
    /// Proxy protocol (http, https, socks5)
    pub protocol: String,
    /// Number of successful requests
    pub success_count: u32,
    /// Number of failed requests
    pub fail_count: u32,
    /// Last time the proxy was used
    pub last_used: Instant,
    /// Last known status (true if working, false if not)
    pub is_working: bool,
    /// Average response time in milliseconds
    pub avg_response_time: u32,
}

impl Proxy {
    /// Create a new proxy from a URL
    pub fn new(url: &str) -> Result<Self, CloudflareError> {
        let mut protocol = "http".to_string();
        let mut host = "".to_string();
        let mut port = 80;
        let mut username = None;
        let mut password = None;
        
        // Parse the URL
        if url.contains("://") {
            let parts: Vec<&str> = url.splitn(2, "://").collect();
            protocol = parts[0].to_string();
            
            let url_parts = parts[1];
            
            // Check for authentication
            if url_parts.contains('@') {
                let auth_parts: Vec<&str> = url_parts.splitn(2, '@').collect();
                let auth = auth_parts[0];
                let host_port = auth_parts[1];
                
                if auth.contains(':') {
                    let auth_split: Vec<&str> = auth.splitn(2, ':').collect();
                    username = Some(auth_split[0].to_string());
                    password = Some(auth_split[1].to_string());
                } else {
                    username = Some(auth.to_string());
                }
                
                // Parse host and port
                if host_port.contains(':') {
                    let host_port_split: Vec<&str> = host_port.splitn(2, ':').collect();
                    host = host_port_split[0].to_string();
                    port = host_port_split[1].parse::<u16>().unwrap_or(80);
                } else {
                    host = host_port.to_string();
                }
            } else {
                // No authentication
                if url_parts.contains(':') {
                    let host_port_split: Vec<&str> = url_parts.splitn(2, ':').collect();
                    host = host_port_split[0].to_string();
                    port = host_port_split[1].parse::<u16>().unwrap_or(80);
                } else {
                    host = url_parts.to_string();
                }
            }
        } else {
            // Simple host:port format
            if url.contains(':') {
                let host_port_split: Vec<&str> = url.splitn(2, ':').collect();
                host = host_port_split[0].to_string();
                port = host_port_split[1].parse::<u16>().unwrap_or(80);
            } else {
                host = url.to_string();
            }
        }
        
        Ok(Self {
            url: url.to_string(),
            host,
            port,
            username,
            password,
            protocol,
            success_count: 0,
            fail_count: 0,
            last_used: Instant::now(),
            is_working: true,
            avg_response_time: 0,
        })
    }
    
    /// Format the proxy URL
    pub fn format_url(&self) -> String {
        match (&self.username, &self.password) {
            (Some(username), Some(password)) => {
                format!("{}://{}:{}@{}:{}", self.protocol, username, password, self.host, self.port)
            }
            (Some(username), None) => {
                format!("{}://{}@{}:{}", self.protocol, username, self.host, self.port)
            }
            _ => {
                format!("{}://{}:{}", self.protocol, self.host, self.port)
            }
        }
    }
    
    /// Record a successful request
    pub fn record_success(&mut self, response_time_ms: u32) {
        self.success_count += 1;
        self.last_used = Instant::now();
        self.is_working = true;
        
        // Update average response time
        if self.avg_response_time == 0 {
            self.avg_response_time = response_time_ms;
        } else {
            self.avg_response_time = (self.avg_response_time + response_time_ms) / 2;
        }
    }
    
    /// Record a failed request
    pub fn record_failure(&mut self) {
        self.fail_count += 1;
        self.last_used = Instant::now();
        
        // Mark as not working if too many failures
        if self.fail_count > 3 && self.success_count < self.fail_count {
            self.is_working = false;
        }
    }
    
    /// Check if the proxy is valid
    pub fn is_valid(&self) -> bool {
        !self.host.is_empty() && self.port > 0
    }
    
    /// Proxy score for ranking (higher is better)
    pub fn score(&self) -> f32 {
        if !self.is_working {
            return 0.0;
        }
        
        let success_rate = if self.success_count + self.fail_count > 0 {
            self.success_count as f32 / (self.success_count + self.fail_count) as f32
        } else {
            0.5 // Default for new proxies
        };
        
        let response_time_factor = if self.avg_response_time > 0 {
            1000.0 / self.avg_response_time as f32
        } else {
            1.0 // Default for new proxies
        };
        
        success_rate * response_time_factor
    }
}

/// Strategies for proxy rotation
#[derive(Debug, Clone, Copy)]
pub enum RotationStrategy {
    /// Round-robin rotation
    RoundRobin,
    /// Random selection
    Random,
    /// Weighted random based on performance
    WeightedRandom,
    /// Best performing proxy
    BestPerforming,
    /// Least recently used
    LeastRecentlyUsed,
}

/// A proxy rotator that manages multiple proxies
pub struct ProxyManager {
    /// List of proxies
    proxies: Vec<Proxy>,
    /// Current proxy index for round-robin rotation
    current_index: usize,
    /// Rotation strategy
    strategy: RotationStrategy,
    /// Last proxy check time
    last_check: Instant,
    /// Check interval in seconds
    check_interval: u64,
}

impl ProxyManager {
    /// Create a new ProxyManager from a list of proxy URLs
    pub fn new(proxy_urls: Vec<String>) -> Self {
        let proxies = proxy_urls
            .iter()
            .filter_map(|url| Proxy::new(url).ok())
            .collect();
        
        Self {
            proxies,
            current_index: 0,
            strategy: RotationStrategy::RoundRobin,
            last_check: Instant::now(),
            check_interval: 60, // Check proxies every 60 seconds
        }
    }
    
    /// Create a ProxyManager from a file containing one proxy per line
    pub fn from_file(file_path: &PathBuf) -> Result<Self, CloudflareError> {
        let file = File::open(file_path)
            .map_err(|e| CloudflareError::ProxyError(format!("Failed to open proxy file: {}", e)))?;
        
        let reader = BufReader::new(file);
        let proxy_urls: Vec<String> = reader
            .lines()
            .filter_map(|line| line.ok())
            .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
            .collect();
        
        Ok(Self::new(proxy_urls))
    }
    
    /// Set the rotation strategy
    pub fn set_strategy(&mut self, strategy: RotationStrategy) {
        self.strategy = strategy;
    }
    
    /// Get the current rotation strategy
    pub fn strategy(&self) -> RotationStrategy {
        self.strategy
    }
    
    /// Get the number of proxies
    pub fn count(&self) -> usize {
        self.proxies.len()
    }
    
    /// Get the number of working proxies
    pub fn working_count(&self) -> usize {
        self.proxies.iter().filter(|p| p.is_working).count()
    }
    
    /// Get the next proxy URL based on the current strategy
    pub fn get_proxy(&mut self) -> Option<String> {
        if self.proxies.is_empty() {
            return None;
        }
        
        // Check if we need to re-check proxies
        if self.last_check.elapsed() > Duration::from_secs(self.check_interval) {
            self.check_proxies();
            self.last_check = Instant::now();
        }
        
        let working_proxies: Vec<&Proxy> = self.proxies.iter().filter(|p| p.is_working).collect();
        
        if working_proxies.is_empty() {
            return None;
        }
        
        let proxy = match self.strategy {
            RotationStrategy::RoundRobin => {
                self.current_index = (self.current_index + 1) % working_proxies.len();
                working_proxies[self.current_index].clone()
            }
            RotationStrategy::Random => {
                let index = thread_rng().gen_range(0..working_proxies.len());
                working_proxies[index].clone()
            }
            RotationStrategy::WeightedRandom => {
                // Create a weighted list based on proxy scores
                let total_score: f32 = working_proxies.iter().map(|p| p.score()).sum();
                
                if total_score <= 0.0 {
                    // Fall back to random if all scores are 0
                    let index = thread_rng().gen_range(0..working_proxies.len());
                    working_proxies[index].clone()
                } else {
                    let mut rng = thread_rng();
                    let target = rng.gen::<f32>() * total_score;
                    
                    let mut cumulative_score = 0.0;
                    for proxy in working_proxies {
                        cumulative_score += proxy.score();
                        if cumulative_score >= target {
                            return Some(proxy.format_url());
                        }
                    }
                    
                    // Fallback to the last proxy if we somehow didn't find one
                    working_proxies.last().unwrap().clone()
                }
            }
            RotationStrategy::BestPerforming => {
                working_proxies.iter()
                    .max_by(|a, b| a.score().partial_cmp(&b.score()).unwrap())
                    .unwrap()
                    .clone()
            }
            RotationStrategy::LeastRecentlyUsed => {
                working_proxies.iter()
                    .min_by_key(|p| p.last_used)
                    .unwrap()
                    .clone()
            }
        };
        
        Some(proxy.format_url())
    }
    
    /// Add a new proxy
    pub fn add_proxy(&mut self, proxy_url: &str) -> Result<(), CloudflareError> {
        let proxy = Proxy::new(proxy_url)?;
        self.proxies.push(proxy);
        Ok(())
    }
    
    /// Remove a proxy
    pub fn remove_proxy(&mut self, proxy_url: &str) {
        self.proxies.retain(|p| p.url != proxy_url);
    }
    
    /// Record a successful request with a proxy
    pub fn record_success(&mut self, proxy_url: &str, response_time_ms: u32) {
        for proxy in &mut self.proxies {
            if proxy.url == proxy_url {
                proxy.record_success(response_time_ms);
                break;
            }
        }
    }
    
    /// Record a failed request with a proxy
    pub fn record_failure(&mut self, proxy_url: &str) {
        for proxy in &mut self.proxies {
            if proxy.url == proxy_url {
                proxy.record_failure();
                break;
            }
        }
    }
    
    /// Check all proxies for availability (no actual check, just reset failed proxies)
    fn check_proxies(&mut self) {
        // In a real implementation, we would check each proxy by making a test request
        // For this placeholder, we'll just reset proxies that have been marked as not working
        for proxy in &mut self.proxies {
            if !proxy.is_working && proxy.last_used.elapsed() > Duration::from_secs(300) {
                // Reset proxies after 5 minutes of not being used
                proxy.is_working = true;
                proxy.fail_count = 0;
            }
        }
    }
    
    /// Export the proxy list to a file
    pub fn export_to_file(&self, file_path: &PathBuf) -> Result<(), CloudflareError> {
        let mut file = File::create(file_path)
            .map_err(|e| CloudflareError::ProxyError(format!("Failed to create proxy file: {}", e)))?;
        
        for proxy in &self.proxies {
            std::io::Write::write_all(&mut file, format!("{}\n", proxy.url).as_bytes())
                .map_err(|e| CloudflareError::ProxyError(format!("Failed to write to proxy file: {}", e)))?;
        }
        
        Ok(())
    }
} 