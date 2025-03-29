use std::collections::HashMap;
use url::Url;
use reqwest::{Client, Response};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use roxmltree::{Document, Node};
use thiserror::Error;
use log::{debug, info, warn, error};
use futures::stream::{self, StreamExt};

use crate::modules::metadata::{PaperMetadata, Link};
use crate::modules::config::ApiConfig;

/// Error types for ArXiv API operations
#[derive(Error, Debug)]
pub enum ArxivError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("XML parsing error: {0}")]
    XmlParse(#[from] roxmltree::Error),
    
    #[error("URL parsing error: {0}")]
    UrlParse(#[from] url::ParseError),
    
    #[error("ID format error: {0}")]
    IdFormat(String),
    
    #[error("API response error: {0}")]
    ApiResponse(String),
    
    #[error("No results found")]
    NoResults,
}

/// Client for interacting with the arXiv API
pub struct ArxivClient {
    /// HTTP client for making requests
    client: Client,
    
    /// API configuration
    config: ApiConfig,
}

impl ArxivClient {
    /// Create a new arXiv API client
    pub fn new(config: ApiConfig) -> Result<Self, ArxivError> {
        let mut headers = HeaderMap::new();
        
        // Set custom user agent if provided
        if let Some(user_agent) = &config.user_agent {
            headers.insert(USER_AGENT, HeaderValue::from_str(user_agent)
                .map_err(|_| ArxivError::ApiResponse("Invalid user agent".to_string()))?);
        } else {
            headers.insert(USER_AGENT, HeaderValue::from_static("llama-arxiv/0.1.0"));
        }
        
        let client = Client::builder()
            .default_headers(headers)
            .timeout(config.timeout)
            .build()?;
            
        Ok(Self { client, config })
    }
    
    /// Get metadata for a specific paper by ID
    pub async fn get_paper(&self, id: &str) -> Result<PaperMetadata, ArxivError> {
        let id = self.normalize_id(id)?;
        
        let url = Url::parse_with_params(
            &self.config.base_url,
            &[
                ("id_list", id.as_str()),
                ("max_results", "1"),
            ],
        )?;
        
        debug!("Fetching paper with ID: {}", id);
        let response = self.client.get(url).send().await?;
        
        if !response.status().is_success() {
            return Err(ArxivError::ApiResponse(format!(
                "API returned status code: {}", response.status()
            )));
        }
        
        let text = response.text().await?;
        let doc = Document::parse(&text)?;
        
        // Extract the first entry
        let entry = doc
            .descendants()
            .find(|n| n.has_tag_name("entry"))
            .ok_or(ArxivError::NoResults)?;
            
        self.parse_entry(entry)
    }
    
    /// Search for papers matching the given query
    pub async fn search(
        &self, 
        query: &str, 
        start: usize, 
        max_results: usize
    ) -> Result<Vec<PaperMetadata>, ArxivError> {
        // Ensure max_results is within reasonable bounds
        let max_results = max_results.min(self.config.max_results);
        
        let url = Url::parse_with_params(
            &self.config.base_url,
            &[
                ("search_query", query),
                ("start", &start.to_string()),
                ("max_results", &max_results.to_string()),
                ("sortBy", "relevance"),
                ("sortOrder", "descending"),
            ],
        )?;
        
        info!("Searching arXiv with query: {}", query);
        let response = self.client.get(url).send().await?;
        
        if !response.status().is_success() {
            return Err(ArxivError::ApiResponse(format!(
                "API returned status code: {}", response.status()
            )));
        }
        
        let text = response.text().await?;
        let doc = Document::parse(&text)?;
        
        let entries: Vec<_> = doc
            .descendants()
            .filter(|n| n.has_tag_name("entry"))
            .collect();
            
        if entries.is_empty() {
            return Err(ArxivError::NoResults);
        }
        
        let mut results = Vec::with_capacity(entries.len());
        
        for entry in entries {
            match self.parse_entry(entry) {
                Ok(metadata) => results.push(metadata),
                Err(e) => warn!("Failed to parse entry: {}", e),
            }
        }
        
        Ok(results)
    }
    
    /// Batch download metadata for multiple papers
    pub async fn batch_get_papers(
        &self, 
        ids: &[String], 
        concurrency: usize
    ) -> HashMap<String, Result<PaperMetadata, ArxivError>> {
        let concurrency = concurrency.min(5); // Cap concurrency
        
        let results = stream::iter(ids)
            .map(|id| async {
                let id_clone = id.clone();
                let result = self.get_paper(&id).await;
                (id_clone, result)
            })
            .buffer_unordered(concurrency)
            .collect::<HashMap<String, Result<PaperMetadata, ArxivError>>>()
            .await;
            
        results
    }
    
    /// Normalize arXiv ID to a consistent format
    fn normalize_id(&self, id: &str) -> Result<String, ArxivError> {
        // Simple case: already in the format "2101.12345" or "2101.12345v1"
        if id.len() >= 9 && id.contains('.') {
            return Ok(id.to_string());
        }
        
        // Remove any "arXiv:" prefix
        let id = id.trim().replace("arXiv:", "");
        
        // Check if it's a legacy ID (pre-2007)
        if id.contains('/') {
            return Ok(id);
        }
        
        Err(ArxivError::IdFormat(format!("Invalid arXiv ID format: {}", id)))
    }
    
    /// Parse an entry node into paper metadata
    fn parse_entry(&self, entry: Node) -> Result<PaperMetadata, ArxivError> {
        // Extract paper ID from the ID URL
        let id_node = entry
            .children()
            .find(|n| n.has_tag_name("id"))
            .ok_or_else(|| ArxivError::ApiResponse("Missing ID field".to_string()))?;
            
        let id_url = id_node.text().unwrap_or("");
        let id = id_url
            .rsplit('/')
            .next()
            .unwrap_or("")
            .to_string();
            
        let version = if id.contains('v') {
            id.split('v')
                .last()
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(1)
        } else {
            1
        };
        
        // Create metadata with extracted ID
        let mut metadata = PaperMetadata::new(&id);
        metadata.version = version;
        
        // Extract title
        if let Some(title_node) = entry.children().find(|n| n.has_tag_name("title")) {
            metadata.title = title_node
                .text()
                .unwrap_or("")
                .replace('\n', " ")
                .trim()
                .to_string();
        }
        
        // Extract authors
        for author_node in entry.children().filter(|n| n.has_tag_name("author")) {
            if let Some(name_node) = author_node.children().find(|n| n.has_tag_name("name")) {
                if let Some(name) = name_node.text() {
                    metadata.authors.push(name.trim().to_string());
                }
            }
        }
        
        // Extract summary
        if let Some(summary_node) = entry.children().find(|n| n.has_tag_name("summary")) {
            metadata.summary = summary_node
                .text()
                .unwrap_or("")
                .replace('\n', " ")
                .trim()
                .to_string();
        }
        
        // Extract published date
        if let Some(published_node) = entry.children().find(|n| n.has_tag_name("published")) {
            metadata.published = published_node
                .text()
                .unwrap_or("")
                .trim()
                .to_string();
        }
        
        // Extract updated date
        if let Some(updated_node) = entry.children().find(|n| n.has_tag_name("updated")) {
            metadata.updated = updated_node
                .text()
                .map(|s| s.trim().to_string());
        }
        
        // Extract categories
        for category_node in entry.children().filter(|n| n.has_tag_name("category")) {
            if let Some(term) = category_node.attribute("term") {
                metadata.categories.push(term.to_string());
                
                // First category is typically the primary one
                if metadata.primary_category.is_empty() {
                    metadata.primary_category = term.to_string();
                }
            }
        }
        
        // Extract DOI if available
        for link_node in entry.children().filter(|n| n.has_tag_name("link")) {
            if let Some(title) = link_node.attribute("title") {
                if title.contains("doi") {
                    if let Some(href) = link_node.attribute("href") {
                        metadata.doi = Some(href.replace("http://dx.doi.org/", "").to_string());
                    }
                }
            }
            
            if let Some(rel) = link_node.attribute("rel") {
                if let Some(href) = link_node.attribute("href") {
                    let title = link_node.attribute("title").map(|s| s.to_string());
                    
                    // Add to links collection
                    metadata.links.push(Link {
                        rel: rel.to_string(),
                        href: href.to_string(),
                        title,
                    });
                    
                    // Set PDF URL if this is the PDF link
                    if rel == "alternate" && href.ends_with(".pdf") {
                        metadata.pdf_url = href.to_string();
                    }
                }
            }
        }
        
        // Extract journal reference if available
        if let Some(journal_ref_node) = entry
            .children()
            .find(|n| n.has_tag_name("arxiv:journal_ref"))
        {
            metadata.journal_ref = journal_ref_node.text().map(|s| s.trim().to_string());
        }
        
        // Extract comment if available
        if let Some(comment_node) = entry
            .children()
            .find(|n| n.has_tag_name("arxiv:comment"))
        {
            metadata.comment = comment_node.text().map(|s| s.trim().to_string());
        }
        
        Ok(metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;
    use std::time::Duration;
    
    fn get_test_config() -> ApiConfig {
        ApiConfig {
            base_url: "http://export.arxiv.org/api/query".to_string(),
            user_agent: Some("llama-arxiv-test/0.1.0".to_string()),
            timeout: Duration::from_secs(30),
            max_results: 10,
        }
    }
    
    #[test]
    fn test_normalize_id() {
        let client = ArxivClient::new(get_test_config()).unwrap();
        
        // Modern format
        assert_eq!(client.normalize_id("2101.12345").unwrap(), "2101.12345");
        assert_eq!(client.normalize_id("2101.12345v2").unwrap(), "2101.12345v2");
        
        // With arXiv prefix
        assert_eq!(client.normalize_id("arXiv:2101.12345").unwrap(), "2101.12345");
        
        // Legacy format
        assert_eq!(client.normalize_id("hep-th/9901001").unwrap(), "hep-th/9901001");
        
        // Invalid format should return an error
        assert!(client.normalize_id("invalid").is_err());
    }
} 