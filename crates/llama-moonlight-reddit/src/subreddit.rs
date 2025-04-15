//! Subreddit interactions
//!
//! This module provides functionality for interacting with Reddit subreddits.

use std::collections::HashMap;
use std::fmt;
use serde::{Serialize, Deserialize};

use crate::{Result, Error, Sort, TimeRange};
use crate::client::RedditClient;
use crate::models::{Thing, Listing, Post, Comment, Subreddit, SubredditRule};

/// Listing filter for subreddit listings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListingFilter {
    /// Hot posts
    Hot,
    /// New posts
    New,
    /// Rising posts
    Rising,
    /// Top posts
    Top(TimeRange),
    /// Controversial posts
    Controversial(TimeRange),
}

impl ListingFilter {
    /// Convert the filter to a path segment for the API
    pub fn to_path(&self) -> String {
        match self {
            ListingFilter::Hot => "hot".to_string(),
            ListingFilter::New => "new".to_string(),
            ListingFilter::Rising => "rising".to_string(),
            ListingFilter::Top(_) => "top".to_string(),
            ListingFilter::Controversial(_) => "controversial".to_string(),
        }
    }
    
    /// Get the time range parameter, if any
    pub fn time_param(&self) -> Option<&'static str> {
        match self {
            ListingFilter::Top(time) | ListingFilter::Controversial(time) => {
                Some(match time {
                    TimeRange::Hour => "hour",
                    TimeRange::Day => "day",
                    TimeRange::Week => "week",
                    TimeRange::Month => "month",
                    TimeRange::Year => "year",
                    TimeRange::All => "all",
                })
            }
            _ => None,
        }
    }
}

/// Subreddit information with additional metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubredditInfo {
    /// Basic subreddit data
    pub info: Subreddit,
    
    /// Subreddit rules
    pub rules: Vec<SubredditRule>,
    
    /// Moderators
    pub moderators: Vec<String>,
    
    /// Related subreddits
    pub related: Vec<String>,
}

/// A client for interacting with a specific subreddit
#[derive(Debug, Clone)]
pub struct SubredditClient {
    /// Reddit client
    client: RedditClient,
    
    /// Subreddit name
    name: String,
}

impl SubredditClient {
    /// Create a new subreddit client
    pub fn new(client: RedditClient, name: &str) -> Self {
        // Remove the r/ prefix if present
        let name = if name.starts_with("r/") {
            name[2..].to_string()
        } else {
            name.to_string()
        };
        
        Self {
            client,
            name,
        }
    }
    
    /// Get the subreddit name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get the subreddit name with r/ prefix
    pub fn prefixed_name(&self) -> String {
        format!("r/{}", self.name)
    }
    
    /// Get information about the subreddit
    pub async fn about(&self) -> Result<Subreddit> {
        let endpoint = format!("/r/{}/about", self.name);
        let response: Thing<Subreddit> = self.client.get(&endpoint, None).await?;
        Ok(response.data)
    }
    
    /// Get detailed information about the subreddit, including rules and moderators
    pub async fn info(&self) -> Result<SubredditInfo> {
        // Get basic info
        let subreddit = self.about().await?;
        
        // Get rules
        let rules = self.rules().await?;
        
        // Get moderators
        let moderators = self.moderators().await?;
        
        // Get related subreddits (stub for now)
        let related = Vec::new();
        
        Ok(SubredditInfo {
            info: subreddit,
            rules,
            moderators,
            related,
        })
    }
    
    /// Get the subreddit's rules
    pub async fn rules(&self) -> Result<Vec<SubredditRule>> {
        let endpoint = format!("/r/{}/about/rules", self.name);
        
        #[derive(Deserialize)]
        struct RulesResponse {
            rules: Vec<SubredditRule>,
        }
        
        let response: RulesResponse = self.client.get(&endpoint, None).await?;
        Ok(response.rules)
    }
    
    /// Get the subreddit's moderators
    pub async fn moderators(&self) -> Result<Vec<String>> {
        let endpoint = format!("/r/{}/about/moderators", self.name);
        
        #[derive(Deserialize)]
        struct ModsResponse {
            data: ModsData,
        }
        
        #[derive(Deserialize)]
        struct ModsData {
            children: Vec<ModInfo>,
        }
        
        #[derive(Deserialize)]
        struct ModInfo {
            name: String,
        }
        
        let response: ModsResponse = self.client.get(&endpoint, None).await?;
        
        let mods = response.data.children.into_iter()
            .map(|m| m.name)
            .collect();
        
        Ok(mods)
    }
    
    /// Subscribe to the subreddit
    pub async fn subscribe(&self) -> Result<()> {
        let mut params = HashMap::new();
        params.insert("action".to_string(), "sub".to_string());
        params.insert("sr_name".to_string(), self.name.clone());
        
        self.client.post::<()>("/api/subscribe", Some(params), None).await?;
        
        Ok(())
    }
    
    /// Unsubscribe from the subreddit
    pub async fn unsubscribe(&self) -> Result<()> {
        let mut params = HashMap::new();
        params.insert("action".to_string(), "unsub".to_string());
        params.insert("sr_name".to_string(), self.name.clone());
        
        self.client.post::<()>("/api/subscribe", Some(params), None).await?;
        
        Ok(())
    }
    
    /// Get posts from the subreddit
    pub async fn posts(
        &self,
        filter: ListingFilter,
        limit: Option<u32>,
        after: Option<&str>,
        before: Option<&str>,
    ) -> Result<Vec<Post>> {
        // Build the endpoint
        let endpoint = format!("/r/{}/{}", self.name, filter.to_path());
        
        // Build parameters
        let mut params = HashMap::new();
        
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        
        if let Some(after) = after {
            params.insert("after".to_string(), after.to_string());
        }
        
        if let Some(before) = before {
            params.insert("before".to_string(), before.to_string());
        }
        
        if let Some(time) = filter.time_param() {
            params.insert("t".to_string(), time.to_string());
        }
        
        // Make the request
        let response: Listing<Thing<Post>> = self.client.get(&endpoint, Some(params)).await?;
        
        // Extract the posts
        let posts = response.data.children.into_iter()
            .map(|p| p.data)
            .collect();
        
        Ok(posts)
    }
    
    /// Get hot posts
    pub fn hot(&self) -> ListingBuilder<Post> {
        ListingBuilder::new(self.clone(), ListingFilter::Hot)
    }
    
    /// Get new posts
    pub fn new(&self) -> ListingBuilder<Post> {
        ListingBuilder::new(self.clone(), ListingFilter::New)
    }
    
    /// Get rising posts
    pub fn rising(&self) -> ListingBuilder<Post> {
        ListingBuilder::new(self.clone(), ListingFilter::Rising)
    }
    
    /// Get top posts
    pub fn top(&self) -> ListingBuilder<Post> {
        ListingBuilder::new(self.clone(), ListingFilter::Top(TimeRange::Day))
    }
    
    /// Get controversial posts
    pub fn controversial(&self) -> ListingBuilder<Post> {
        ListingBuilder::new(self.clone(), ListingFilter::Controversial(TimeRange::Day))
    }
    
    /// Search within the subreddit
    pub async fn search(
        &self,
        query: &str,
        sort: Option<Sort>,
        limit: Option<u32>,
        after: Option<&str>,
        before: Option<&str>,
    ) -> Result<Vec<Post>> {
        // Build parameters
        let mut params = HashMap::new();
        params.insert("q".to_string(), query.to_string());
        params.insert("restrict_sr".to_string(), "true".to_string());
        
        if let Some(sort) = sort {
            params.insert("sort".to_string(), format!("{:?}", sort).to_lowercase());
        }
        
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        
        if let Some(after) = after {
            params.insert("after".to_string(), after.to_string());
        }
        
        if let Some(before) = before {
            params.insert("before".to_string(), before.to_string());
        }
        
        // Build the endpoint
        let endpoint = format!("/r/{}/search", self.name);
        
        // Make the request
        let response: Listing<Thing<Post>> = self.client.get(&endpoint, Some(params)).await?;
        
        // Extract the posts
        let posts = response.data.children.into_iter()
            .map(|p| p.data)
            .collect();
        
        Ok(posts)
    }
    
    /// Submit a post to the subreddit
    pub async fn submit(
        &self,
        title: &str,
        kind: &str,
        text: Option<&str>,
        url: Option<&str>,
        nsfw: bool,
        spoiler: bool,
        flair_id: Option<&str>,
        flair_text: Option<&str>,
    ) -> Result<String> {
        // Build parameters
        let mut params = HashMap::new();
        params.insert("sr".to_string(), self.name.clone());
        params.insert("title".to_string(), title.to_string());
        params.insert("kind".to_string(), kind.to_string());
        
        if let Some(text) = text {
            params.insert("text".to_string(), text.to_string());
        }
        
        if let Some(url) = url {
            params.insert("url".to_string(), url.to_string());
        }
        
        if nsfw {
            params.insert("nsfw".to_string(), "true".to_string());
        }
        
        if spoiler {
            params.insert("spoiler".to_string(), "true".to_string());
        }
        
        if let Some(flair_id) = flair_id {
            params.insert("flair_id".to_string(), flair_id.to_string());
        }
        
        if let Some(flair_text) = flair_text {
            params.insert("flair_text".to_string(), flair_text.to_string());
        }
        
        // Make the request
        #[derive(Deserialize)]
        struct SubmitResponse {
            json: SubmitResponseJson,
        }
        
        #[derive(Deserialize)]
        struct SubmitResponseJson {
            data: SubmitResponseData,
        }
        
        #[derive(Deserialize)]
        struct SubmitResponseData {
            name: String,
        }
        
        let response: SubmitResponse = self.client.post("/api/submit", Some(params), None).await?;
        
        Ok(response.json.data.name)
    }
    
    /// Get the subreddit's wiki index
    pub async fn wiki_index(&self) -> Result<String> {
        let endpoint = format!("/r/{}/wiki/index", self.name);
        
        #[derive(Deserialize)]
        struct WikiResponse {
            data: WikiData,
        }
        
        #[derive(Deserialize)]
        struct WikiData {
            content_html: String,
        }
        
        let response: WikiResponse = self.client.get(&endpoint, None).await?;
        
        Ok(response.data.content_html)
    }
    
    /// Get a wiki page
    pub async fn wiki_page(&self, page: &str) -> Result<String> {
        let endpoint = format!("/r/{}/wiki/{}", self.name, page);
        
        #[derive(Deserialize)]
        struct WikiResponse {
            data: WikiData,
        }
        
        #[derive(Deserialize)]
        struct WikiData {
            content_html: String,
        }
        
        let response: WikiResponse = self.client.get(&endpoint, None).await?;
        
        Ok(response.data.content_html)
    }
}

/// A builder for listing requests
#[derive(Debug, Clone)]
pub struct ListingBuilder<T> {
    /// Subreddit client
    client: SubredditClient,
    
    /// Listing filter
    filter: ListingFilter,
    
    /// Maximum number of items to return
    limit: Option<u32>,
    
    /// Fullname of item to fetch items after
    after: Option<String>,
    
    /// Fullname of item to fetch items before
    before: Option<String>,
    
    /// Phantom type for the listing item type
    _phantom: std::marker::PhantomData<T>,
}

impl<T> ListingBuilder<T> {
    /// Create a new listing builder
    pub fn new(client: SubredditClient, filter: ListingFilter) -> Self {
        Self {
            client,
            filter,
            limit: None,
            after: None,
            before: None,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Set the maximum number of items to return
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
    
    /// Set the item to fetch items after
    pub fn after(mut self, after: &str) -> Self {
        self.after = Some(after.to_string());
        self
    }
    
    /// Set the item to fetch items before
    pub fn before(mut self, before: &str) -> Self {
        self.before = Some(before.to_string());
        self
    }
    
    /// Set the time range for top/controversial listings
    pub fn time(mut self, time: TimeRange) -> Self {
        self.filter = match self.filter {
            ListingFilter::Top(_) => ListingFilter::Top(time),
            ListingFilter::Controversial(_) => ListingFilter::Controversial(time),
            _ => self.filter,
        };
        self
    }
}

impl ListingBuilder<Post> {
    /// Fetch the listing posts
    pub async fn fetch(self) -> Result<Vec<Post>> {
        self.client.posts(
            self.filter,
            self.limit,
            self.after.as_deref(),
            self.before.as_deref(),
        ).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_listing_filter() {
        assert_eq!(ListingFilter::Hot.to_path(), "hot");
        assert_eq!(ListingFilter::New.to_path(), "new");
        assert_eq!(ListingFilter::Rising.to_path(), "rising");
        assert_eq!(ListingFilter::Top(TimeRange::Day).to_path(), "top");
        assert_eq!(ListingFilter::Controversial(TimeRange::Week).to_path(), "controversial");
        
        assert_eq!(ListingFilter::Hot.time_param(), None);
        assert_eq!(ListingFilter::Top(TimeRange::Hour).time_param(), Some("hour"));
        assert_eq!(ListingFilter::Top(TimeRange::Day).time_param(), Some("day"));
        assert_eq!(ListingFilter::Top(TimeRange::Week).time_param(), Some("week"));
        assert_eq!(ListingFilter::Top(TimeRange::Month).time_param(), Some("month"));
        assert_eq!(ListingFilter::Top(TimeRange::Year).time_param(), Some("year"));
        assert_eq!(ListingFilter::Top(TimeRange::All).time_param(), Some("all"));
    }
    
    #[test]
    fn test_subreddit_client_name() {
        let client = RedditClient::new(Default::default()).unwrap();
        
        let subreddit = SubredditClient::new(client.clone(), "rust");
        assert_eq!(subreddit.name(), "rust");
        assert_eq!(subreddit.prefixed_name(), "r/rust");
        
        let subreddit = SubredditClient::new(client, "r/rust");
        assert_eq!(subreddit.name(), "rust");
        assert_eq!(subreddit.prefixed_name(), "r/rust");
    }
} 