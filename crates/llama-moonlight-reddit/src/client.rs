//! Reddit client implementation
//!
//! This module provides the main entry point for interacting with the Reddit API.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use chrono::{DateTime, Utc};
use reqwest::{Client, Response, Method, StatusCode, header};
use tokio::sync::{RwLock, Mutex};
use url::Url;
use serde::{Serialize, Deserialize};
use serde_json::json;
use log::{debug, info, warn, error};

use crate::{Result, Error, API_BASE, DEFAULT_USER_AGENT, Scope, Sort, TimeRange, VoteDirection};
use crate::auth::{Authenticator, TokenResponse, Credentials, TokenStore, MemoryTokenStore};
use crate::models::{Thing, Listing, ListingData, Post, Comment, Subreddit, User, Message};
use crate::throttle::RateLimiter;
use crate::subreddit::SubredditClient;
use crate::user::UserClient;
use crate::post::PostClient;
use crate::search::SearchClient;
use crate::multireddit::MultiredditClient;
use crate::message::MessageClient;

#[cfg(feature = "stealth")]
use crate::stealth::StealthConfig;

#[cfg(feature = "tor")]
use crate::tor::TorConfig;

/// Configuration for the Reddit client
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// User agent string to use for requests
    pub user_agent: String,

    /// Request timeout in seconds
    pub timeout: Duration,

    /// Base URL for the Reddit API
    pub api_base: String,

    /// Whether to automatically retry failed requests
    pub auto_retry: bool,

    /// Maximum number of retries
    pub max_retries: u32,

    /// Whether to use the rate limiter
    pub use_rate_limiter: bool,

    /// Whether to log requests
    pub log_requests: bool,

    /// Stealth configuration for avoiding detection
    #[cfg(feature = "stealth")]
    pub stealth_config: Option<StealthConfig>,

    /// Tor configuration for anonymous browsing
    #[cfg(feature = "tor")]
    pub tor_config: Option<TorConfig>,
    
    /// Request headers to include with every request
    pub custom_headers: HashMap<String, String>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            user_agent: DEFAULT_USER_AGENT.to_string(),
            timeout: Duration::from_secs(30),
            api_base: API_BASE.to_string(),
            auto_retry: true,
            max_retries: 3,
            use_rate_limiter: true,
            log_requests: true,
            #[cfg(feature = "stealth")]
            stealth_config: None,
            #[cfg(feature = "tor")]
            tor_config: None,
            custom_headers: HashMap::new(),
        }
    }
}

impl ClientConfig {
    /// Create a new client configuration with the specified user agent
    pub fn with_user_agent(mut self, user_agent: &str) -> Self {
        self.user_agent = user_agent.to_string();
        self
    }

    /// Set the request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the base API URL
    pub fn with_api_base(mut self, api_base: &str) -> Self {
        self.api_base = api_base.to_string();
        self
    }

    /// Enable or disable automatic retries
    pub fn with_auto_retry(mut self, auto_retry: bool) -> Self {
        self.auto_retry = auto_retry;
        self
    }

    /// Set the maximum number of retries
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Enable or disable the rate limiter
    pub fn with_rate_limiter(mut self, use_rate_limiter: bool) -> Self {
        self.use_rate_limiter = use_rate_limiter;
        self
    }

    /// Enable or disable request logging
    pub fn with_log_requests(mut self, log_requests: bool) -> Self {
        self.log_requests = log_requests;
        self
    }

    /// Add a custom header to be included with every request
    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        self.custom_headers.insert(name.to_string(), value.to_string());
        self
    }

    /// Configure stealth mode (requires stealth feature)
    #[cfg(feature = "stealth")]
    pub fn with_stealth(mut self, stealth_config: StealthConfig) -> Self {
        self.stealth_config = Some(stealth_config);
        self
    }

    /// Configure Tor integration (requires tor feature)
    #[cfg(feature = "tor")]
    pub fn with_tor(mut self, tor_config: TorConfig) -> Self {
        self.tor_config = Some(tor_config);
        self
    }
}

/// Client state for authenticated Reddit API access
#[derive(Debug, Clone)]
pub struct ClientState {
    /// Reddit username
    pub username: Option<String>,
    
    /// Whether the client is authenticated
    pub authenticated: bool,
    
    /// Authentication scopes
    pub scopes: Vec<Scope>,
    
    /// When the current token expires
    pub token_expires_at: Option<DateTime<Utc>>,
}

/// The main Reddit client
pub struct RedditClient {
    /// HTTP client for making requests
    client: Client,
    
    /// Client configuration
    config: ClientConfig,
    
    /// Authenticator for handling Reddit OAuth
    authenticator: Arc<Authenticator>,
    
    /// Token store for managing tokens
    token_store: Arc<dyn TokenStore>,
    
    /// Rate limiter for managing request rates
    rate_limiter: Arc<RwLock<RateLimiter>>,
    
    /// Client state
    state: Arc<RwLock<ClientState>>,
    
    /// Request counter
    request_count: Arc<Mutex<u64>>,
}

impl RedditClient {
    /// Create a new Reddit client with the specified configuration
    pub async fn new(config: ClientConfig) -> Result<Self> {
        // Create HTTP client
        let client_builder = Client::builder()
            .user_agent(&config.user_agent)
            .timeout(config.timeout);
        
        // Configure proxy if Tor is enabled
        #[cfg(feature = "tor")]
        let client_builder = if let Some(tor_config) = &config.tor_config {
            crate::tor::configure_client(client_builder, tor_config)?
        } else {
            client_builder
        };
        
        // Build the client
        let client = client_builder.build()
            .map_err(|e| Error::HttpError(e))?;
        
        // Create the authenticator
        let authenticator = Arc::new(Authenticator::new());
        
        // Create the token store
        let token_store: Arc<dyn TokenStore> = Arc::new(MemoryTokenStore::new());
        
        // Create the rate limiter
        let rate_limiter = Arc::new(RwLock::new(RateLimiter::new(600, Duration::from_secs(600))));
        
        // Create the client state
        let state = Arc::new(RwLock::new(ClientState {
            username: None,
            authenticated: false,
            scopes: Vec::new(),
            token_expires_at: None,
        }));
        
        // Create the request counter
        let request_count = Arc::new(Mutex::new(0));
        
        Ok(Self {
            client,
            config,
            authenticator,
            token_store,
            rate_limiter,
            state,
            request_count,
        })
    }
    
    /// Set a custom token store for persistent token storage
    pub fn with_token_store<T: TokenStore + 'static>(mut self, token_store: T) -> Self {
        self.token_store = Arc::new(token_store);
        self
    }
    
    /// Authenticate with a username and password (Resource Owner Password Credentials flow)
    pub async fn authenticate_username_password(
        &self,
        client_id: &str,
        client_secret: &str,
        username: &str,
        password: &str,
    ) -> Result<Self> {
        let credentials = Credentials::new_password(
            client_id.to_string(),
            client_secret.to_string(),
            username.to_string(),
            password.to_string(),
        );
        
        self.authenticate(credentials).await
    }
    
    /// Authenticate with a refresh token
    pub async fn authenticate_refresh_token(
        &self,
        client_id: &str,
        client_secret: &str,
        refresh_token: &str,
    ) -> Result<Self> {
        let credentials = Credentials::new_refresh_token(
            client_id.to_string(),
            client_secret.to_string(),
            refresh_token.to_string(),
        );
        
        self.authenticate(credentials).await
    }
    
    /// Authenticate with the specified credentials
    pub async fn authenticate(&self, credentials: Credentials) -> Result<Self> {
        // Request token from Reddit
        let token_response = self.authenticator.authenticate(
            &self.client,
            &credentials,
        ).await?;
        
        // Store the token
        self.token_store.store_token(&token_response).await?;
        
        // Update client state
        let mut state = self.state.write().await;
        state.authenticated = true;
        state.username = credentials.username().cloned();
        state.token_expires_at = Some(token_response.expires_at);
        
        // Parse scopes
        if let Some(scope_str) = &token_response.scope {
            // Parse scope string (space-separated)
            let scopes = scope_str.split(' ')
                .filter(|s| !s.is_empty())
                .map(|s| match s {
                    "identity" => Scope::Identity,
                    "edit" => Scope::Edit,
                    "flair" => Scope::Flair,
                    "history" => Scope::History,
                    "modconfig" => Scope::ModConfig,
                    "modlog" => Scope::ModLog,
                    "modposts" => Scope::ModPosts,
                    "modwiki" => Scope::ModWiki,
                    "mysubreddits" => Scope::Read,
                    "privatemessages" => Scope::PrivateMessages,
                    "read" => Scope::Read,
                    "report" => Scope::Report,
                    "save" => Scope::Save,
                    "submit" => Scope::Submit,
                    "subscribe" => Scope::Vote,
                    "vote" => Scope::Vote,
                    "*" => Scope::All,
                    _ => Scope::Read, // Default to Read for unknown scopes
                })
                .collect();
            
            state.scopes = scopes;
        }
        
        // Clone the client with the updated state
        let mut new_client = self.clone();
        
        // Update the state on the new client
        let mut new_state = new_client.state.write().await;
        *new_state = state.clone();
        
        Ok(new_client)
    }
    
    /// Check if the client is authenticated
    pub async fn is_authenticated(&self) -> bool {
        let state = self.state.read().await;
        state.authenticated
    }
    
    /// Get the client's username
    pub async fn username(&self) -> Option<String> {
        let state = self.state.read().await;
        state.username.clone()
    }
    
    /// Get the client's scopes
    pub async fn scopes(&self) -> Vec<Scope> {
        let state = self.state.read().await;
        state.scopes.clone()
    }
    
    /// Check if the client has a specific scope
    pub async fn has_scope(&self, scope: Scope) -> bool {
        let state = self.state.read().await;
        state.scopes.contains(&scope) || state.scopes.contains(&Scope::All)
    }
    
    /// Get the current client configuration
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }
    
    /// Make a GET request to the Reddit API
    pub async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &str,
        params: Option<HashMap<String, String>>,
    ) -> Result<T> {
        self.request::<(), T>(Method::GET, endpoint, params, None).await
    }
    
    /// Make a POST request to the Reddit API
    pub async fn post<T: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &str,
        params: Option<HashMap<String, String>>,
        body: Option<serde_json::Value>,
    ) -> Result<T> {
        self.request::<serde_json::Value, T>(Method::POST, endpoint, params, body).await
    }
    
    /// Make a PUT request to the Reddit API
    pub async fn put<T: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &str,
        params: Option<HashMap<String, String>>,
        body: Option<serde_json::Value>,
    ) -> Result<T> {
        self.request::<serde_json::Value, T>(Method::PUT, endpoint, params, body).await
    }
    
    /// Make a DELETE request to the Reddit API
    pub async fn delete<T: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &str,
        params: Option<HashMap<String, String>>,
    ) -> Result<T> {
        self.request::<(), T>(Method::DELETE, endpoint, params, None).await
    }
    
    /// Make a request to the Reddit API
    pub async fn request<B: Serialize, T: for<'de> Deserialize<'de>>(
        &self,
        method: Method,
        endpoint: &str,
        params: Option<HashMap<String, String>>,
        body: Option<B>,
    ) -> Result<T> {
        // Check if the client is authenticated when needed
        if !endpoint.contains("/api/v1/access_token") {
            let is_authenticated = self.is_authenticated().await;
            if !is_authenticated {
                return Err(Error::AuthError("Not authenticated".to_string()));
            }
            
            // Check token expiration
            let needs_refresh = {
                let state = self.state.read().await;
                if let Some(expires_at) = state.token_expires_at {
                    let now = Utc::now();
                    expires_at < now + chrono::Duration::seconds(60)
                } else {
                    false
                }
            };
            
            if needs_refresh {
                debug!("Access token is expired or about to expire, refreshing");
                
                // Get the refresh token
                let token = self.token_store.get_token().await?;
                if let Some(refresh_token) = token.refresh_token {
                    // Get client credentials
                    let client_id = token.client_id.ok_or_else(|| Error::AuthError("Missing client ID".to_string()))?;
                    let client_secret = token.client_secret.ok_or_else(|| Error::AuthError("Missing client secret".to_string()))?;
                    
                    // Refresh the token
                    let credentials = Credentials::new_refresh_token(
                        client_id,
                        client_secret,
                        refresh_token,
                    );
                    
                    let token_response = self.authenticator.authenticate(
                        &self.client,
                        &credentials,
                    ).await?;
                    
                    // Store the new token
                    self.token_store.store_token(&token_response).await?;
                    
                    // Update the client state
                    let mut state = self.state.write().await;
                    state.token_expires_at = Some(token_response.expires_at);
                } else {
                    return Err(Error::AuthError("No refresh token available".to_string()));
                }
            }
        }
        
        // Apply rate limiting if enabled
        if self.config.use_rate_limiter {
            let mut rate_limiter = self.rate_limiter.write().await;
            rate_limiter.acquire().await?;
        }
        
        // Build the URL
        let url = if endpoint.starts_with("https://") || endpoint.starts_with("http://") {
            endpoint.to_string()
        } else {
            format!("{}{}", self.config.api_base, endpoint)
        };
        
        // Build the request
        let mut request_builder = self.client.request(method.clone(), &url);
        
        // Add query parameters
        if let Some(params) = params {
            request_builder = request_builder.query(&params);
        }
        
        // Add body if any
        if let Some(body) = body {
            request_builder = request_builder.json(&body);
        }
        
        // Add authorization header
        if !endpoint.contains("/api/v1/access_token") {
            let token = self.token_store.get_token().await?;
            if let Some(access_token) = token.access_token {
                request_builder = request_builder.header(
                    header::AUTHORIZATION,
                    format!("Bearer {}", access_token),
                );
            }
        }
        
        // Add custom headers
        for (name, value) in &self.config.custom_headers {
            request_builder = request_builder.header(name, value);
        }
        
        // Log the request if enabled
        if self.config.log_requests {
            debug!("{} {}", method, url);
        }
        
        // Update request counter
        {
            let mut count = self.request_count.lock().await;
            *count += 1;
        }
        
        // Send the request
        let response = request_builder.send().await?;
        
        // Update rate limits from response headers
        if self.config.use_rate_limiter {
            let mut rate_limiter = self.rate_limiter.write().await;
            
            if let Some(remaining) = response.headers().get("x-ratelimit-remaining") {
                if let Ok(remaining) = remaining.to_str() {
                    if let Ok(remaining) = remaining.parse::<u32>() {
                        rate_limiter.set_remaining(remaining);
                    }
                }
            }
            
            if let Some(reset) = response.headers().get("x-ratelimit-reset") {
                if let Ok(reset) = reset.to_str() {
                    if let Ok(reset) = reset.parse::<u64>() {
                        rate_limiter.set_reset(Duration::from_secs(reset));
                    }
                }
            }
        }
        
        // Check the response status
        let status = response.status();
        if !status.is_success() {
            return Err(handle_error_response(response).await);
        }
        
        // Parse the response
        let response_data = response.json::<T>().await
            .map_err(|e| Error::ParseError(format!("Failed to parse response: {}", e)))?;
        
        Ok(response_data)
    }
    
    /// Get information about the current user
    pub async fn me(&self) -> Result<User> {
        let response: Thing<User> = self.get("/api/v1/me", None).await?;
        Ok(response.data)
    }
    
    /// Get a subreddit client for the specified subreddit
    pub fn subreddit(&self, name: &str) -> SubredditClient {
        SubredditClient::new(self.clone(), name)
    }
    
    /// Get a user client for the specified user
    pub fn user(&self, username: &str) -> UserClient {
        UserClient::new(self.clone(), username)
    }
    
    /// Get a post client for the specified post
    pub fn post(&self, id: &str) -> PostClient {
        PostClient::new(self.clone(), id)
    }
    
    /// Get a search client for searching Reddit
    pub fn search(&self) -> SearchClient {
        SearchClient::new(self.clone())
    }
    
    /// Get a multireddit client for the specified multireddit
    pub fn multireddit(&self, user: &str, name: &str) -> MultiredditClient {
        MultiredditClient::new(self.clone(), user, name)
    }
    
    /// Get a message client for handling private messages
    pub fn messages(&self) -> MessageClient {
        MessageClient::new(self.clone())
    }
    
    /// Submit a new post
    pub async fn submit_post(
        &self,
        subreddit: &str,
        title: &str,
        kind: PostKind,
        content: &str,
        nsfw: bool,
        spoiler: bool,
        flair_id: Option<&str>,
        flair_text: Option<&str>,
    ) -> Result<String> {
        let mut params = HashMap::new();
        params.insert("sr".to_string(), subreddit.to_string());
        params.insert("title".to_string(), title.to_string());
        
        match kind {
            PostKind::Link => {
                params.insert("kind".to_string(), "link".to_string());
                params.insert("url".to_string(), content.to_string());
            }
            PostKind::Self_ => {
                params.insert("kind".to_string(), "self".to_string());
                params.insert("text".to_string(), content.to_string());
            }
            PostKind::Image => {
                params.insert("kind".to_string(), "image".to_string());
                params.insert("url".to_string(), content.to_string());
            }
            PostKind::Video => {
                params.insert("kind".to_string(), "video".to_string());
                params.insert("url".to_string(), content.to_string());
            }
            PostKind::Poll => {
                params.insert("kind".to_string(), "poll".to_string());
                // Poll options would go here
                params.insert("text".to_string(), content.to_string());
            }
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
            id: String,
            url: String,
            name: String,
        }
        
        let response: SubmitResponse = self.post("/api/submit", Some(params), None).await?;
        
        Ok(response.json.data.name)
    }
    
    /// Submit a comment on a post or comment
    pub async fn submit_comment(&self, parent_id: &str, text: &str) -> Result<String> {
        let mut params = HashMap::new();
        params.insert("parent".to_string(), parent_id.to_string());
        params.insert("text".to_string(), text.to_string());
        
        #[derive(Deserialize)]
        struct CommentResponse {
            json: CommentResponseJson,
        }
        
        #[derive(Deserialize)]
        struct CommentResponseJson {
            data: CommentResponseData,
        }
        
        #[derive(Deserialize)]
        struct CommentResponseData {
            things: Vec<CommentResponseThing>,
        }
        
        #[derive(Deserialize)]
        struct CommentResponseThing {
            data: CommentResponseThingData,
        }
        
        #[derive(Deserialize)]
        struct CommentResponseThingData {
            id: String,
            name: String,
        }
        
        let response: CommentResponse = self.post("/api/comment", Some(params), None).await?;
        
        Ok(response.json.data.things[0].data.name.clone())
    }
    
    /// Vote on a post or comment
    pub async fn vote(&self, id: &str, direction: VoteDirection) -> Result<()> {
        let mut params = HashMap::new();
        params.insert("id".to_string(), id.to_string());
        params.insert("dir".to_string(), (direction as i32).to_string());
        
        self.post::<()>("/api/vote", Some(params), None).await?;
        
        Ok(())
    }
    
    /// Get statistics about the client's API usage
    pub async fn stats(&self) -> ClientStats {
        let requests = {
            let count = self.request_count.lock().await;
            *count
        };
        
        let rate_limits = {
            let rate_limiter = self.rate_limiter.read().await;
            (rate_limiter.remaining(), rate_limiter.reset_time())
        };
        
        ClientStats {
            requests,
            remaining_requests: rate_limits.0,
            reset_time: rate_limits.1,
        }
    }
}

impl Clone for RedditClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            config: self.config.clone(),
            authenticator: self.authenticator.clone(),
            token_store: self.token_store.clone(),
            rate_limiter: self.rate_limiter.clone(),
            state: self.state.clone(),
            request_count: self.request_count.clone(),
        }
    }
}

/// Stats about the client's API usage
#[derive(Debug, Clone)]
pub struct ClientStats {
    /// Number of requests made
    pub requests: u64,
    
    /// Number of remaining requests before rate limiting
    pub remaining_requests: u32,
    
    /// Time until rate limit reset
    pub reset_time: Duration,
}

/// Type of post to submit
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostKind {
    /// Link post
    Link,
    
    /// Self/text post
    Self_,
    
    /// Image post
    Image,
    
    /// Video post
    Video,
    
    /// Poll post
    Poll,
}

/// Handle an error response from the Reddit API
async fn handle_error_response(response: Response) -> Error {
    let status = response.status();
    let status_code = status.as_u16();
    
    // Try to parse the error message from the response
    let body = match response.text().await {
        Ok(body) => body,
        Err(_) => String::from("Unknown error"),
    };
    
    match status_code {
        401 => Error::AuthError("Unauthorized: Invalid or expired token".to_string()),
        403 => Error::AuthError("Forbidden: Insufficient permissions".to_string()),
        429 => Error::RateLimitError(format!("Rate limit exceeded: {}", body)),
        500..=599 => Error::ApiError {
            status_code,
            message: format!("Server error: {}", body),
        },
        _ => Error::ApiError {
            status_code,
            message: body,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_client_config() {
        let config = ClientConfig::default()
            .with_user_agent("test-agent")
            .with_timeout(Duration::from_secs(60))
            .with_api_base("https://example.com")
            .with_auto_retry(false)
            .with_max_retries(5)
            .with_rate_limiter(false)
            .with_log_requests(false)
            .with_header("X-Test", "value");
        
        assert_eq!(config.user_agent, "test-agent");
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.api_base, "https://example.com");
        assert_eq!(config.auto_retry, false);
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.use_rate_limiter, false);
        assert_eq!(config.log_requests, false);
        assert_eq!(config.custom_headers.get("X-Test"), Some(&"value".to_string()));
    }
} 