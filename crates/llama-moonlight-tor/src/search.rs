//! Dark web search functionality
//!
//! This module provides capabilities for searching the dark web (onion services)
//! through various search engines and aggregating the results.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use reqwest::Response;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};
use url::Url;
use futures::future::join_all;

use crate::Result;
use crate::Error;
use crate::client::TorClient;
use crate::onion::{OnionMetadata, OnionService};

/// A search result from a dark web search engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// The title of the result
    pub title: String,
    
    /// The URL of the result
    pub url: String,
    
    /// A snippet or description of the result
    pub snippet: Option<String>,
    
    /// The search engine that provided this result
    pub source: String,
    
    /// When this result was found
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// Relevance score (0.0 to 1.0)
    pub relevance: f32,
    
    /// Category or type of result
    pub category: Option<String>,
    
    /// Whether this result has been verified as online
    pub verified: bool,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl SearchResult {
    /// Create a new search result
    pub fn new(title: &str, url: &str, source: &str) -> Self {
        Self {
            title: title.to_string(),
            url: url.to_string(),
            snippet: None,
            source: source.to_string(),
            timestamp: chrono::Utc::now(),
            relevance: 0.5, // Default relevance
            category: None,
            verified: false,
            metadata: HashMap::new(),
        }
    }
    
    /// Set the snippet for this result
    pub fn with_snippet(mut self, snippet: &str) -> Self {
        self.snippet = Some(snippet.to_string());
        self
    }
    
    /// Set the relevance score
    pub fn with_relevance(mut self, relevance: f32) -> Self {
        self.relevance = relevance.clamp(0.0, 1.0);
        self
    }
    
    /// Set the category
    pub fn with_category(mut self, category: &str) -> Self {
        self.category = Some(category.to_string());
        self
    }
    
    /// Mark as verified
    pub fn with_verified(mut self, verified: bool) -> Self {
        self.verified = verified;
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Check if this result is for an onion service
    pub fn is_onion(&self) -> bool {
        self.url.contains(".onion")
    }
}

/// A dark web search engine
#[derive(Debug, Clone)]
pub struct SearchEngine {
    /// Name of the search engine
    pub name: String,
    
    /// Base URL of the search engine
    pub base_url: String,
    
    /// Search URL template, with {query} placeholder
    pub search_url_template: String,
    
    /// Whether this search engine is enabled
    pub enabled: bool,
    
    /// Parser function for this search engine
    parser: SearchEngineParserType,
    
    /// Weight of this engine in aggregated results (0.0 to 1.0)
    pub weight: f32,
}

/// Type for search engine parser functions
type SearchEngineParserType = fn(&str) -> Vec<SearchResult>;

impl SearchEngine {
    /// Create a new search engine
    pub fn new(name: &str, base_url: &str, search_url_template: &str, parser: SearchEngineParserType) -> Self {
        Self {
            name: name.to_string(),
            base_url: base_url.to_string(),
            search_url_template: search_url_template.to_string(),
            enabled: true,
            parser,
            weight: 1.0,
        }
    }
    
    /// Get the search URL for a query
    pub fn get_search_url(&self, query: &str) -> String {
        self.search_url_template.replace("{query}", &url_encode(query))
    }
    
    /// Parse search results from HTML
    pub fn parse_results(&self, html: &str) -> Vec<SearchResult> {
        (self.parser)(html)
    }
}

/// Dark web search aggregator
#[derive(Debug)]
pub struct DarkWebSearch {
    /// Tor client for making requests
    client: Arc<TorClient>,
    
    /// Onion service handler
    onion_service: Arc<OnionService>,
    
    /// Search engines
    engines: Vec<SearchEngine>,
    
    /// Cache of search results
    cache: Arc<RwLock<HashMap<String, Vec<SearchResult>>>>,
    
    /// Max cache age
    max_cache_age: Duration,
    
    /// Request timeout
    timeout: Duration,
    
    /// Maximum number of concurrent requests
    max_concurrent_requests: usize,
}

impl DarkWebSearch {
    /// Create a new dark web search aggregator
    pub fn new(client: Arc<TorClient>) -> Self {
        let onion_service = Arc::new(OnionService::new(client.clone()));
        
        let engines = vec![
            ahia_search_engine(),
            torch_search_engine(),
            not_evil_search_engine(),
            phobos_search_engine(),
        ];
        
        Self {
            client,
            onion_service,
            engines,
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_cache_age: Duration::from_secs(3600), // 1 hour
            timeout: Duration::from_secs(30),
            max_concurrent_requests: 3,
        }
    }
    
    /// Search the dark web with the given query
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        // Check the cache first
        let normalized_query = normalize_query(query);
        
        // Check if we have a cached result
        let cached = self.get_cached_results(&normalized_query).await;
        if let Some(results) = cached {
            return Ok(results);
        }
        
        // Gather search engines to use
        let engines: Vec<_> = self.engines.iter()
            .filter(|e| e.enabled)
            .collect();
        
        // Create a vector to hold all the futures
        let mut search_futures = Vec::new();
        
        // Create a future for each search engine
        for engine in engines {
            let search_url = engine.get_search_url(&normalized_query);
            let engine_clone = engine.clone();
            let client_clone = self.client.clone();
            
            // Create a future for this search
            let future = async move {
                let start_time = Instant::now();
                
                // Make the request
                let result = client_clone.get(&search_url).await;
                
                // Return engine name and results
                match result {
                    Ok(response) => {
                        if response.status().is_success() {
                            let html = response.text().await.unwrap_or_default();
                            let results = engine_clone.parse_results(&html);
                            (engine_clone.name, results, true)
                        } else {
                            (engine_clone.name, Vec::new(), false)
                        }
                    },
                    Err(_) => (engine_clone.name, Vec::new(), false),
                }
            };
            
            search_futures.push(future);
        }
        
        // Execute the futures in batches to avoid too many concurrent requests
        let mut all_results = Vec::new();
        
        // Process futures in batches
        for chunk in search_futures.chunks(self.max_concurrent_requests) {
            let results = join_all(chunk.to_vec()).await;
            
            for (engine_name, engine_results, success) in results {
                // Add the results
                all_results.extend(engine_results);
                
                // Log failure if needed
                if !success {
                    log::warn!("Failed to search with engine: {}", engine_name);
                }
            }
        }
        
        // Deduplicate and rank results
        let ranked_results = self.rank_and_deduplicate(all_results);
        
        // Cache the results
        self.cache_results(&normalized_query, &ranked_results).await;
        
        Ok(ranked_results)
    }
    
    /// Add a custom search engine
    pub fn add_search_engine(&mut self, engine: SearchEngine) {
        self.engines.push(engine);
    }
    
    /// Enable a search engine by name
    pub fn enable_engine(&mut self, name: &str) {
        if let Some(engine) = self.engines.iter_mut().find(|e| e.name == name) {
            engine.enabled = true;
        }
    }
    
    /// Disable a search engine by name
    pub fn disable_engine(&mut self, name: &str) {
        if let Some(engine) = self.engines.iter_mut().find(|e| e.name == name) {
            engine.enabled = false;
        }
    }
    
    /// Get available search engines
    pub fn get_engines(&self) -> Vec<&SearchEngine> {
        self.engines.iter().collect()
    }
    
    /// Set the maximum cache age
    pub fn set_max_cache_age(&mut self, age: Duration) {
        self.max_cache_age = age;
    }
    
    /// Set the request timeout
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }
    
    /// Set the maximum number of concurrent requests
    pub fn set_max_concurrent_requests(&mut self, max: usize) {
        self.max_concurrent_requests = max;
    }
    
    /// Clear the search cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
    
    /// Get cached search results if available and not expired
    async fn get_cached_results(&self, query: &str) -> Option<Vec<SearchResult>> {
        let cache = self.cache.read().await;
        
        if let Some(results) = cache.get(query) {
            // Check if the first result is expired
            if !results.is_empty() {
                let first = &results[0];
                let age = chrono::Utc::now().signed_duration_since(first.timestamp);
                
                if age < chrono::Duration::from_std(self.max_cache_age).unwrap() {
                    return Some(results.clone());
                }
            }
        }
        
        None
    }
    
    /// Cache search results
    async fn cache_results(&self, query: &str, results: &[SearchResult]) {
        let mut cache = self.cache.write().await;
        cache.insert(query.to_string(), results.to_vec());
    }
    
    /// Rank and deduplicate search results
    fn rank_and_deduplicate(&self, results: Vec<SearchResult>) -> Vec<SearchResult> {
        // Create a map to deduplicate by URL
        let mut url_map: HashMap<String, SearchResult> = HashMap::new();
        
        // Process each result
        for result in results {
            let url = result.url.clone();
            
            if let Some(existing) = url_map.get_mut(&url) {
                // If we already have this URL, update the relevance if the new one is higher
                if result.relevance > existing.relevance {
                    existing.relevance = result.relevance;
                }
                
                // If this has a snippet and the existing one doesn't, use this snippet
                if result.snippet.is_some() && existing.snippet.is_none() {
                    existing.snippet = result.snippet;
                }
                
                // Add this source to the metadata
                existing.metadata.insert(
                    format!("also_found_in_{}", result.source),
                    "true".to_string()
                );
            } else {
                // New URL, add it to the map
                url_map.insert(url, result);
            }
        }
        
        // Get all values and sort by relevance
        let mut sorted_results: Vec<_> = url_map.into_values().collect();
        sorted_results.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap_or(std::cmp::Ordering::Equal));
        
        sorted_results
    }
    
    /// Verify that search results are online
    pub async fn verify_results(&self, results: &mut [SearchResult]) -> Result<()> {
        // Create a vector of futures
        let mut verify_futures = Vec::new();
        
        for result in results.iter() {
            // Only verify onion services
            if !result.is_onion() {
                continue;
            }
            
            let url = result.url.clone();
            let onion_service = self.onion_service.clone();
            
            // Create a future for this verification
            let future = async move {
                let is_online = onion_service.check_online(&url).await.unwrap_or(false);
                (url, is_online)
            };
            
            verify_futures.push(future);
        }
        
        // Execute the futures in batches
        for chunk in verify_futures.chunks(self.max_concurrent_requests) {
            let verifications = join_all(chunk.to_vec()).await;
            
            // Update the results
            for (url, is_online) in verifications {
                if let Some(result) = results.iter_mut().find(|r| r.url == url) {
                    result.verified = is_online;
                    
                    // Lower relevance for offline sites
                    if !is_online {
                        result.relevance *= 0.5;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Extract metadata for search results
    pub async fn extract_metadata(&self, results: &mut [SearchResult]) -> Result<()> {
        // Create a vector of futures
        let mut metadata_futures = Vec::new();
        
        for result in results.iter() {
            // Only process onion services
            if !result.is_onion() {
                continue;
            }
            
            let url = result.url.clone();
            let onion_service = self.onion_service.clone();
            
            // Create a future for this metadata extraction
            let future = async move {
                match onion_service.extract_metadata(&url).await {
                    Ok(metadata) => (url, Some(metadata)),
                    Err(_) => (url, None),
                }
            };
            
            metadata_futures.push(future);
        }
        
        // Execute the futures in batches
        for chunk in metadata_futures.chunks(self.max_concurrent_requests) {
            let metadata_results = join_all(chunk.to_vec()).await;
            
            // Update the results
            for (url, metadata_opt) in metadata_results {
                if let Some(metadata) = metadata_opt {
                    if let Some(result) = results.iter_mut().find(|r| r.url == url) {
                        // Update the result with metadata
                        if let Some(title) = &metadata.title {
                            if result.title.is_empty() {
                                result.title = title.clone();
                            }
                        }
                        
                        if let Some(desc) = &metadata.description {
                            if result.snippet.is_none() {
                                result.snippet = Some(desc.clone());
                            }
                        }
                        
                        // Add metadata
                        result.metadata.insert("online".to_string(), metadata.is_online.to_string());
                        if let Some(time) = metadata.avg_response_time_ms {
                            result.metadata.insert("response_time_ms".to_string(), time.to_string());
                        }
                        
                        // Verify based on actual check
                        result.verified = metadata.is_online;
                    }
                }
            }
        }
        
        Ok(())
    }
}

/// URL encode a string
fn url_encode(s: &str) -> String {
    // Simple URL encoding
    s.replace(' ', "+")
        .replace('&', "%26")
        .replace('?', "%3F")
        .replace('=', "%3D")
        .replace('/', "%2F")
        .replace('#', "%23")
}

/// Normalize a search query
fn normalize_query(query: &str) -> String {
    query.trim().to_lowercase()
}

/// Factory function for the Ahmia search engine
fn ahia_search_engine() -> SearchEngine {
    SearchEngine::new(
        "Ahmia",
        "http://juhanurmihxlp77nkq76byazcldy2hlmovfu2epvl5ankdibsot4csyd.onion",
        "http://juhanurmihxlp77nkq76byazcldy2hlmovfu2epvl5ankdibsot4csyd.onion/search/?q={query}",
        |html| {
            let mut results = Vec::new();
            
            // Extract search results
            let re = regex::Regex::new(r#"<h4><a href="([^"]+)"[^>]*>([^<]+)</a></h4>.*?<p>([^<]+)</p>"#).unwrap();
            
            for cap in re.captures_iter(html) {
                let url = cap.get(1).map_or("", |m| m.as_str()).to_string();
                let title = cap.get(2).map_or("", |m| m.as_str()).to_string();
                let snippet = cap.get(3).map_or("", |m| m.as_str()).to_string();
                
                results.push(
                    SearchResult::new(&title, &url, "Ahmia")
                        .with_snippet(&snippet)
                        .with_relevance(0.8)
                );
            }
            
            results
        }
    )
}

/// Factory function for the Torch search engine
fn torch_search_engine() -> SearchEngine {
    SearchEngine::new(
        "Torch",
        "http://xmh57jrzrnw6insl.onion",
        "http://xmh57jrzrnw6insl.onion/4a1f6b371c/search.cgi?q={query}",
        |html| {
            let mut results = Vec::new();
            
            // Extract search results
            let re = regex::Regex::new(r#"<dt><a href="([^"]+)"[^>]*>([^<]+)</a></dt>.*?<dd[^>]*>(.*?)</dd>"#).unwrap();
            
            for cap in re.captures_iter(html) {
                let url = cap.get(1).map_or("", |m| m.as_str()).to_string();
                let title = cap.get(2).map_or("", |m| m.as_str()).to_string();
                let snippet = cap.get(3).map_or("", |m| m.as_str()).to_string();
                
                // Clean up snippet (remove HTML tags)
                let clean_snippet = snippet.replace(regex::Regex::new(r"<[^>]+>").unwrap().as_str(""), "");
                
                results.push(
                    SearchResult::new(&title, &url, "Torch")
                        .with_snippet(&clean_snippet)
                        .with_relevance(0.7)
                );
            }
            
            results
        }
    )
}

/// Factory function for the NotEvil search engine
fn not_evil_search_engine() -> SearchEngine {
    SearchEngine::new(
        "NotEvil",
        "http://hss3uro2hsxfogfq.onion",
        "http://hss3uro2hsxfogfq.onion/index.php?q={query}",
        |html| {
            let mut results = Vec::new();
            
            // Extract search results
            let re = regex::Regex::new(r#"<h5><a href="([^"]+)"[^>]*>([^<]+)</a></h5>.*?<p[^>]*>(.*?)</p>"#).unwrap();
            
            for cap in re.captures_iter(html) {
                let url = cap.get(1).map_or("", |m| m.as_str()).to_string();
                let title = cap.get(2).map_or("", |m| m.as_str()).to_string();
                let snippet = cap.get(3).map_or("", |m| m.as_str()).to_string();
                
                results.push(
                    SearchResult::new(&title, &url, "NotEvil")
                        .with_snippet(&snippet)
                        .with_relevance(0.75)
                );
            }
            
            results
        }
    )
}

/// Factory function for the Phobos search engine
fn phobos_search_engine() -> SearchEngine {
    SearchEngine::new(
        "Phobos",
        "http://phobosxilamwcg75xt22id7aywkzol6q6rfl2flipcqoc4e4ahima5id.onion",
        "http://phobosxilamwcg75xt22id7aywkzol6q6rfl2flipcqoc4e4ahima5id.onion/search?query={query}",
        |html| {
            let mut results = Vec::new();
            
            // Extract search results
            let re = regex::Regex::new(r#"<h5><a href="([^"]+)"[^>]*>([^<]+)</a></h5>.*?<p[^>]*>(.*?)</p>"#).unwrap();
            
            for cap in re.captures_iter(html) {
                let url = cap.get(1).map_or("", |m| m.as_str()).to_string();
                let title = cap.get(2).map_or("", |m| m.as_str()).to_string();
                let snippet = cap.get(3).map_or("", |m| m.as_str()).to_string();
                
                results.push(
                    SearchResult::new(&title, &url, "Phobos")
                        .with_snippet(&snippet)
                        .with_relevance(0.65)
                );
            }
            
            results
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_url_encode() {
        assert_eq!(url_encode("hello world"), "hello+world");
        assert_eq!(url_encode("foo & bar"), "foo+%26+bar");
        assert_eq!(url_encode("search?q=test"), "search%3Fq%3Dtest");
    }
    
    #[test]
    fn test_normalize_query() {
        assert_eq!(normalize_query("  Hello World  "), "hello world");
        assert_eq!(normalize_query("Tor Network"), "tor network");
    }
    
    #[test]
    fn test_search_result() {
        let result = SearchResult::new("Test Page", "http://example.onion", "TestEngine")
            .with_snippet("This is a test page")
            .with_relevance(0.85)
            .with_category("test")
            .with_verified(true)
            .with_metadata("key", "value");
            
        assert_eq!(result.title, "Test Page");
        assert_eq!(result.url, "http://example.onion");
        assert_eq!(result.snippet, Some("This is a test page".to_string()));
        assert_eq!(result.source, "TestEngine");
        assert_eq!(result.relevance, 0.85);
        assert_eq!(result.category, Some("test".to_string()));
        assert_eq!(result.verified, true);
        assert_eq!(result.metadata.get("key"), Some(&"value".to_string()));
        assert_eq!(result.is_onion(), true);
    }
    
    #[test]
    fn test_search_engine() {
        let engine = SearchEngine::new(
            "Test Engine",
            "http://example.onion",
            "http://example.onion/search?q={query}",
            |_| Vec::new(),
        );
        
        assert_eq!(engine.name, "Test Engine");
        assert_eq!(engine.base_url, "http://example.onion");
        assert_eq!(
            engine.get_search_url("test query"),
            "http://example.onion/search?q=test+query"
        );
    }
} 