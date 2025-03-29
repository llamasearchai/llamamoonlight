//! Reddit search functionality
//!
//! This module provides functionality for searching Reddit content.

use std::collections::HashMap;
use std::fmt;

use crate::{Result, Error, Sort, TimeRange};
use crate::client::RedditClient;
use crate::models::{Thing, Listing, Post, Comment, Subreddit};

/// The type of content to search for
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SearchType {
    /// Search for posts (default)
    Post,
    
    /// Search for comments
    Comment,
    
    /// Search for subreddits
    Subreddit,
    
    /// Search for users
    User,
}

impl fmt::Display for SearchType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SearchType::Post => write!(f, "link"),
            SearchType::Comment => write!(f, "comment"),
            SearchType::Subreddit => write!(f, "sr"),
            SearchType::User => write!(f, "user"),
        }
    }
}

/// Option for restricting search to NSFW content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NsfwOption {
    /// Include both SFW and NSFW content (default)
    All,
    
    /// Include only SFW content
    SfwOnly,
    
    /// Include only NSFW content
    NsfwOnly,
}

impl fmt::Display for NsfwOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NsfwOption::All => write!(f, ""),
            NsfwOption::SfwOnly => write!(f, "no"),
            NsfwOption::NsfwOnly => write!(f, "yes"),
        }
    }
}

/// A client for searching Reddit
#[derive(Debug, Clone)]
pub struct SearchClient {
    /// Reddit client
    client: RedditClient,
}

impl SearchClient {
    /// Create a new search client
    pub fn new(client: RedditClient) -> Self {
        Self { client }
    }
    
    /// Create a new search query
    pub fn query(&self, query: &str) -> SearchBuilder {
        SearchBuilder::new(self.client.clone(), query)
    }
    
    /// Search for posts
    pub async fn search_posts(&self, query: &str, limit: Option<u32>) -> Result<Vec<Post>> {
        self.query(query).limit(limit.unwrap_or(25)).execute().await
    }
    
    /// Search for subreddits
    pub async fn search_subreddits(&self, query: &str, limit: Option<u32>) -> Result<Vec<Subreddit>> {
        self.query(query)
            .type_(SearchType::Subreddit)
            .limit(limit.unwrap_or(25))
            .execute_subreddits()
            .await
    }
    
    /// Search for comments
    pub async fn search_comments(&self, query: &str, limit: Option<u32>) -> Result<Vec<Comment>> {
        self.query(query)
            .type_(SearchType::Comment)
            .limit(limit.unwrap_or(25))
            .execute_comments()
            .await
    }
}

/// A builder for search queries
#[derive(Debug, Clone)]
pub struct SearchBuilder {
    /// Reddit client
    client: RedditClient,
    
    /// Search query
    query: String,
    
    /// Maximum number of results to return
    limit: Option<u32>,
    
    /// Subreddit to restrict search to
    subreddit: Option<String>,
    
    /// Type of content to search for
    type_: Option<SearchType>,
    
    /// Sort order
    sort: Option<Sort>,
    
    /// Time range for relevance
    time: Option<TimeRange>,
    
    /// NSFW option
    nsfw: Option<NsfwOption>,
    
    /// Include syntax in results
    include_syntax: bool,
    
    /// Item to fetch results after
    after: Option<String>,
    
    /// Item to fetch results before
    before: Option<String>,
}

impl SearchBuilder {
    /// Create a new search builder
    pub fn new(client: RedditClient, query: &str) -> Self {
        Self {
            client,
            query: query.to_string(),
            limit: None,
            subreddit: None,
            type_: None,
            sort: None,
            time: None,
            nsfw: None,
            include_syntax: false,
            after: None,
            before: None,
        }
    }
    
    /// Set the maximum number of results to return
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
    
    /// Set the subreddit to restrict search to
    pub fn subreddit(mut self, subreddit: &str) -> Self {
        // Remove the r/ prefix if present
        let subreddit = if subreddit.starts_with("r/") {
            subreddit[2..].to_string()
        } else {
            subreddit.to_string()
        };
        
        self.subreddit = Some(subreddit);
        self
    }
    
    /// Set the type of content to search for
    pub fn type_(mut self, type_: SearchType) -> Self {
        self.type_ = Some(type_);
        self
    }
    
    /// Set the sort order
    pub fn sort(mut self, sort: Sort) -> Self {
        self.sort = Some(sort);
        self
    }
    
    /// Set the time range for relevance
    pub fn time(mut self, time: TimeRange) -> Self {
        self.time = Some(time);
        self
    }
    
    /// Set the NSFW option
    pub fn nsfw(mut self, nsfw: NsfwOption) -> Self {
        self.nsfw = Some(nsfw);
        self
    }
    
    /// Set whether to include syntax in results
    pub fn include_syntax(mut self, include_syntax: bool) -> Self {
        self.include_syntax = include_syntax;
        self
    }
    
    /// Set the item to fetch results after
    pub fn after(mut self, after: &str) -> Self {
        self.after = Some(after.to_string());
        self
    }
    
    /// Set the item to fetch results before
    pub fn before(mut self, before: &str) -> Self {
        self.before = Some(before.to_string());
        self
    }
    
    /// Build the search parameters
    fn build_params(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        // Add the query
        params.insert("q".to_string(), self.query.clone());
        
        // Add the limit
        if let Some(limit) = self.limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        
        // Add the subreddit restriction
        if let Some(subreddit) = &self.subreddit {
            params.insert("restrict_sr".to_string(), "true".to_string());
            // Note: We're already including the subreddit in the path
        }
        
        // Add the type
        if let Some(type_) = &self.type_ {
            params.insert("type".to_string(), type_.to_string());
        }
        
        // Add the sort order
        if let Some(sort) = &self.sort {
            params.insert("sort".to_string(), format!("{:?}", sort).to_lowercase());
        }
        
        // Add the time range
        if let Some(time) = &self.time {
            params.insert("t".to_string(), format!("{:?}", time).to_lowercase());
        }
        
        // Add the NSFW option
        if let Some(nsfw) = &self.nsfw {
            let nsfw_str = nsfw.to_string();
            if !nsfw_str.is_empty() {
                params.insert("include_over_18".to_string(), nsfw_str);
            }
        }
        
        // Add the syntax option
        if self.include_syntax {
            params.insert("include_facets".to_string(), "true".to_string());
        }
        
        // Add pagination
        if let Some(after) = &self.after {
            params.insert("after".to_string(), after.clone());
        }
        
        if let Some(before) = &self.before {
            params.insert("before".to_string(), before.clone());
        }
        
        params
    }
    
    /// Execute the search for posts
    pub async fn execute(self) -> Result<Vec<Post>> {
        // Build parameters
        let params = self.build_params();
        
        // Build the endpoint
        let endpoint = if let Some(subreddit) = &self.subreddit {
            format!("/r/{}/search", subreddit)
        } else {
            "/search".to_string()
        };
        
        // Make the request
        let response: Listing<Thing<Post>> = self.client.get(&endpoint, Some(params)).await?;
        
        // Extract the posts
        let posts = response.data.children.into_iter()
            .map(|p| p.data)
            .collect();
        
        Ok(posts)
    }
    
    /// Execute the search for subreddits
    pub async fn execute_subreddits(self) -> Result<Vec<Subreddit>> {
        // Ensure the type is set to Subreddit
        let builder = self.type_(SearchType::Subreddit);
        
        // Build parameters
        let params = builder.build_params();
        
        // Build the endpoint
        let endpoint = if let Some(subreddit) = &builder.subreddit {
            format!("/r/{}/search", subreddit)
        } else {
            "/search".to_string()
        };
        
        // Make the request
        let response: Listing<Thing<Subreddit>> = builder.client.get(&endpoint, Some(params)).await?;
        
        // Extract the subreddits
        let subreddits = response.data.children.into_iter()
            .map(|s| s.data)
            .collect();
        
        Ok(subreddits)
    }
    
    /// Execute the search for comments
    pub async fn execute_comments(self) -> Result<Vec<Comment>> {
        // Ensure the type is set to Comment
        let builder = self.type_(SearchType::Comment);
        
        // Build parameters
        let params = builder.build_params();
        
        // Build the endpoint
        let endpoint = if let Some(subreddit) = &builder.subreddit {
            format!("/r/{}/search", subreddit)
        } else {
            "/search".to_string()
        };
        
        // Make the request
        let response: Listing<Thing<Comment>> = builder.client.get(&endpoint, Some(params)).await?;
        
        // Extract the comments
        let comments = response.data.children.into_iter()
            .map(|c| c.data)
            .collect();
        
        Ok(comments)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_search_type_display() {
        assert_eq!(SearchType::Post.to_string(), "link");
        assert_eq!(SearchType::Comment.to_string(), "comment");
        assert_eq!(SearchType::Subreddit.to_string(), "sr");
        assert_eq!(SearchType::User.to_string(), "user");
    }
    
    #[test]
    fn test_nsfw_option_display() {
        assert_eq!(NsfwOption::All.to_string(), "");
        assert_eq!(NsfwOption::SfwOnly.to_string(), "no");
        assert_eq!(NsfwOption::NsfwOnly.to_string(), "yes");
    }
    
    #[test]
    fn test_search_builder_params() {
        let client = RedditClient::new(Default::default()).unwrap();
        
        let builder = SearchBuilder::new(client, "test query")
            .limit(25)
            .subreddit("rust")
            .type_(SearchType::Post)
            .sort(Sort::Relevance)
            .time(TimeRange::Week)
            .nsfw(NsfwOption::SfwOnly)
            .include_syntax(true)
            .after("t3_123456");
        
        let params = builder.build_params();
        
        assert_eq!(params.get("q"), Some(&"test query".to_string()));
        assert_eq!(params.get("limit"), Some(&"25".to_string()));
        assert_eq!(params.get("restrict_sr"), Some(&"true".to_string()));
        assert_eq!(params.get("type"), Some(&"link".to_string()));
        assert_eq!(params.get("sort"), Some(&"relevance".to_string()));
        assert_eq!(params.get("t"), Some(&"week".to_string()));
        assert_eq!(params.get("include_over_18"), Some(&"no".to_string()));
        assert_eq!(params.get("include_facets"), Some(&"true".to_string()));
        assert_eq!(params.get("after"), Some(&"t3_123456".to_string()));
    }
} 