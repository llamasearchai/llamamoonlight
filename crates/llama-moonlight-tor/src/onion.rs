//! Onion service handling
//!
//! This module provides functionality for accessing and interacting with
//! onion services (hidden services) on the Tor network.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use reqwest::{Client, Response, StatusCode};
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};
use url::Url;

use crate::Result;
use crate::Error;
use crate::TorConfig;
use crate::client::TorClient;

/// Metadata about an onion service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnionMetadata {
    /// The onion address
    pub address: String,
    
    /// The onion version (v2 or v3)
    pub version: OnionVersion,
    
    /// When this service was first discovered
    pub first_seen: chrono::DateTime<chrono::Utc>,
    
    /// When this service was last successfully accessed
    pub last_accessed: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Title of the site, if available
    pub title: Option<String>,
    
    /// Description of the site, if available
    pub description: Option<String>,
    
    /// Keywords associated with the site
    pub keywords: Vec<String>,
    
    /// Whether the service is currently available
    pub is_online: bool,
    
    /// When the status was last checked
    pub last_checked: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Categories associated with this service
    pub categories: Vec<String>,
    
    /// Custom tags for this service
    pub tags: Vec<String>,
    
    /// Number of successful connections
    pub connection_count: u32,
    
    /// Average response time in milliseconds
    pub avg_response_time_ms: Option<u64>,
    
    /// Additional metadata as key-value pairs
    pub additional_metadata: HashMap<String, String>,
}

impl OnionMetadata {
    /// Create new metadata for an onion service
    pub fn new(address: &str) -> Self {
        // Determine version from address length
        let version = if address.len() == 16 + 6 { // 16 chars + ".onion"
            OnionVersion::V2
        } else {
            OnionVersion::V3
        };
        
        Self {
            address: address.to_string(),
            version,
            first_seen: chrono::Utc::now(),
            last_accessed: None,
            title: None,
            description: None,
            keywords: Vec::new(),
            is_online: false,
            last_checked: None,
            categories: Vec::new(),
            tags: Vec::new(),
            connection_count: 0,
            avg_response_time_ms: None,
            additional_metadata: HashMap::new(),
        }
    }
    
    /// Mark this service as accessed successfully
    pub fn mark_accessed(&mut self, response_time_ms: Option<u64>) {
        self.last_accessed = Some(chrono::Utc::now());
        self.is_online = true;
        self.connection_count += 1;
        
        // Update the average response time
        if let Some(time) = response_time_ms {
            if let Some(avg) = self.avg_response_time_ms {
                // Compute a weighted average (90% old, 10% new)
                self.avg_response_time_ms = Some(
                    (avg as f64 * 0.9 + time as f64 * 0.1) as u64
                );
            } else {
                self.avg_response_time_ms = Some(time);
            }
        }
    }
    
    /// Mark this service as checked (whether successful or not)
    pub fn mark_checked(&mut self, is_online: bool) {
        self.last_checked = Some(chrono::Utc::now());
        self.is_online = is_online;
    }
    
    /// Add a category to this service
    pub fn add_category(&mut self, category: &str) {
        if !self.categories.contains(&category.to_string()) {
            self.categories.push(category.to_string());
        }
    }
    
    /// Add a tag to this service
    pub fn add_tag(&mut self, tag: &str) {
        if !self.tags.contains(&tag.to_string()) {
            self.tags.push(tag.to_string());
        }
    }
    
    /// Set the title of this service
    pub fn set_title(&mut self, title: &str) {
        self.title = Some(title.to_string());
    }
    
    /// Set the description of this service
    pub fn set_description(&mut self, description: &str) {
        self.description = Some(description.to_string());
    }
    
    /// Add keywords for this service
    pub fn add_keywords(&mut self, keywords: &[&str]) {
        for keyword in keywords {
            if !self.keywords.contains(&keyword.to_string()) {
                self.keywords.push(keyword.to_string());
            }
        }
    }
    
    /// Add additional metadata
    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.additional_metadata.insert(key.to_string(), value.to_string());
    }
    
    /// Get the full onion URL
    pub fn get_url(&self) -> String {
        format!("http://{}", self.address)
    }
    
    /// Check if this service has been seen recently
    pub fn is_recent(&self, max_age: Duration) -> bool {
        if let Some(last_checked) = self.last_checked {
            let age = chrono::Utc::now().signed_duration_since(last_checked);
            return age < chrono::Duration::from_std(max_age).unwrap();
        }
        false
    }
}

/// Onion service version
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OnionVersion {
    /// Version 2 onion service (16 characters)
    V2,
    /// Version 3 onion service (56 characters)
    V3,
}

/// Service for accessing and managing onion services
#[derive(Debug)]
pub struct OnionService {
    /// Tor client for accessing onion services
    client: Arc<TorClient>,
    
    /// Cache of onion service metadata
    metadata_cache: Arc<RwLock<HashMap<String, OnionMetadata>>>,
    
    /// Maximum cache age
    max_cache_age: Duration,
    
    /// Request timeout
    timeout: Duration,
}

impl OnionService {
    /// Create a new onion service handler
    pub fn new(client: Arc<TorClient>) -> Self {
        Self {
            client,
            metadata_cache: Arc::new(RwLock::new(HashMap::new())),
            max_cache_age: Duration::from_secs(3600), // 1 hour
            timeout: Duration::from_secs(30),
        }
    }
    
    /// Access an onion service and return the response
    pub async fn access(&self, onion_address: &str) -> Result<Response> {
        // Ensure it's an onion address
        if !onion_address.contains(".onion") {
            return Err(Error::OnionError(format!("Not an onion address: {}", onion_address)));
        }
        
        // Format the URL if needed
        let url = if onion_address.starts_with("http") {
            onion_address.to_string()
        } else {
            format!("http://{}", onion_address)
        };
        
        // Record the start time
        let start_time = Instant::now();
        
        // Access the service
        let result = self.client.get(&url).await;
        
        // Calculate response time
        let response_time = start_time.elapsed().as_millis() as u64;
        
        // Update metadata
        self.update_metadata(onion_address, result.is_ok(), Some(response_time)).await;
        
        // Return the result
        result
    }
    
    /// Check if an onion service is online
    pub async fn check_online(&self, onion_address: &str) -> Result<bool> {
        // Try to access the service
        match self.access(onion_address).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    
    /// Extract metadata from an onion service
    pub async fn extract_metadata(&self, onion_address: &str) -> Result<OnionMetadata> {
        // Try to access the service
        let response = self.access(onion_address).await?;
        
        // Get the existing metadata or create new
        let mut metadata = match self.get_metadata(onion_address).await {
            Some(meta) => meta,
            None => OnionMetadata::new(onion_address),
        };
        
        // Extract the HTML content
        let html = response.text().await.map_err(|e| 
            Error::OnionError(format!("Failed to read response: {}", e))
        )?;
        
        // Extract title
        if let Some(title) = extract_title(&html) {
            metadata.set_title(&title);
        }
        
        // Extract description
        if let Some(description) = extract_description(&html) {
            metadata.set_description(&description);
        }
        
        // Extract keywords
        if let Some(keywords) = extract_keywords(&html) {
            metadata.add_keywords(&keywords.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        }
        
        // Store the updated metadata
        self.store_metadata(metadata.clone()).await;
        
        Ok(metadata)
    }
    
    /// Get metadata for an onion service
    pub async fn get_metadata(&self, onion_address: &str) -> Option<OnionMetadata> {
        // Normalize the address
        let address = normalize_onion_address(onion_address);
        
        // Check the cache
        let cache = self.metadata_cache.read().await;
        cache.get(&address).cloned()
    }
    
    /// Store metadata for an onion service
    pub async fn store_metadata(&self, metadata: OnionMetadata) {
        // Normalize the address
        let address = normalize_onion_address(&metadata.address);
        
        // Update the cache
        let mut cache = self.metadata_cache.write().await;
        cache.insert(address, metadata);
    }
    
    /// Update metadata for an onion service
    async fn update_metadata(&self, onion_address: &str, is_online: bool, response_time_ms: Option<u64>) {
        // Normalize the address
        let address = normalize_onion_address(onion_address);
        
        // Get existing metadata or create new
        let mut metadata = match self.get_metadata(&address).await {
            Some(meta) => meta,
            None => OnionMetadata::new(&address),
        };
        
        // Update the metadata
        if is_online {
            metadata.mark_accessed(response_time_ms);
        } else {
            metadata.mark_checked(false);
        }
        
        // Store the updated metadata
        self.store_metadata(metadata).await;
    }
    
    /// Get all known onion services
    pub async fn get_all_services(&self) -> Vec<OnionMetadata> {
        let cache = self.metadata_cache.read().await;
        cache.values().cloned().collect()
    }
    
    /// Get online onion services
    pub async fn get_online_services(&self) -> Vec<OnionMetadata> {
        let cache = self.metadata_cache.read().await;
        cache.values()
            .filter(|meta| meta.is_online)
            .cloned()
            .collect()
    }
    
    /// Set the maximum cache age
    pub fn set_max_cache_age(&mut self, age: Duration) {
        self.max_cache_age = age;
    }
    
    /// Set the request timeout
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }
    
    /// Clear the metadata cache
    pub async fn clear_cache(&self) {
        let mut cache = self.metadata_cache.write().await;
        cache.clear();
    }
}

/// Normalize an onion address (remove http:// and trailing slashes)
fn normalize_onion_address(address: &str) -> String {
    let mut result = address.to_string();
    
    // Remove http:// or https://
    if result.starts_with("http://") {
        result = result[7..].to_string();
    } else if result.starts_with("https://") {
        result = result[8..].to_string();
    }
    
    // Remove trailing slashes
    while result.ends_with('/') {
        result.pop();
    }
    
    result
}

/// Extract the title from HTML
fn extract_title(html: &str) -> Option<String> {
    // Simple regex-based extraction
    let re = regex::Regex::new(r"<title[^>]*>(.*?)</title>").ok()?;
    re.captures(html).and_then(|cap| {
        cap.get(1).map(|m| m.as_str().to_string())
    })
}

/// Extract the description from HTML
fn extract_description(html: &str) -> Option<String> {
    // Look for meta description
    let re = regex::Regex::new(r#"<meta\s+name\s*=\s*["']description["']\s+content\s*=\s*["'](.*?)["']"#).ok()?;
    re.captures(html).and_then(|cap| {
        cap.get(1).map(|m| m.as_str().to_string())
    })
}

/// Extract keywords from HTML
fn extract_keywords(html: &str) -> Option<Vec<String>> {
    // Look for meta keywords
    let re = regex::Regex::new(r#"<meta\s+name\s*=\s*["']keywords["']\s+content\s*=\s*["'](.*?)["']"#).ok()?;
    re.captures(html).and_then(|cap| {
        cap.get(1).map(|m| {
            m.as_str()
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_normalize_onion_address() {
        assert_eq!(
            normalize_onion_address("http://abcdefghijklmnop.onion/"),
            "abcdefghijklmnop.onion"
        );
        
        assert_eq!(
            normalize_onion_address("abcdefghijklmnop.onion"),
            "abcdefghijklmnop.onion"
        );
        
        assert_eq!(
            normalize_onion_address("https://abcdefghijklmnop.onion///"),
            "abcdefghijklmnop.onion"
        );
    }
    
    #[test]
    fn test_extract_title() {
        let html = r#"<html><head><title>Test Page</title></head><body></body></html>"#;
        assert_eq!(extract_title(html), Some("Test Page".to_string()));
        
        let html = r#"<html><head><title></title></head><body></body></html>"#;
        assert_eq!(extract_title(html), Some("".to_string()));
        
        let html = r#"<html><head></head><body></body></html>"#;
        assert_eq!(extract_title(html), None);
    }
    
    #[test]
    fn test_extract_description() {
        let html = r#"<html><head><meta name="description" content="Test Description"></head><body></body></html>"#;
        assert_eq!(extract_description(html), Some("Test Description".to_string()));
        
        let html = r#"<html><head></head><body></body></html>"#;
        assert_eq!(extract_description(html), None);
    }
    
    #[test]
    fn test_extract_keywords() {
        let html = r#"<html><head><meta name="keywords" content="test, keywords, example"></head><body></body></html>"#;
        assert_eq!(
            extract_keywords(html), 
            Some(vec!["test".to_string(), "keywords".to_string(), "example".to_string()])
        );
        
        let html = r#"<html><head></head><body></body></html>"#;
        assert_eq!(extract_keywords(html), None);
    }
    
    #[test]
    fn test_onion_metadata() {
        let mut metadata = OnionMetadata::new("abcdefghijklmnop.onion");
        
        assert_eq!(metadata.address, "abcdefghijklmnop.onion");
        assert_eq!(metadata.version, OnionVersion::V2);
        assert_eq!(metadata.is_online, false);
        
        metadata.mark_accessed(Some(500));
        assert_eq!(metadata.is_online, true);
        assert_eq!(metadata.connection_count, 1);
        assert_eq!(metadata.avg_response_time_ms, Some(500));
        
        metadata.set_title("Test Page");
        assert_eq!(metadata.title, Some("Test Page".to_string()));
        
        metadata.add_category("marketplace");
        assert_eq!(metadata.categories, vec!["marketplace"]);
        
        metadata.add_tag("tested");
        assert_eq!(metadata.tags, vec!["tested"]);
        
        assert_eq!(metadata.get_url(), "http://abcdefghijklmnop.onion");
    }
} 