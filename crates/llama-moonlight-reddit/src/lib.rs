//! # llama-moonlight-reddit
//!
//! A comprehensive Reddit automation and interaction library for the Llama Moonlight ecosystem.
//!
//! This crate provides high-level abstractions and tools for interacting with Reddit,
//! including authentication, browsing, posting, commenting, and moderation capabilities.
//!
//! ## Features
//!
//! - OAuth2 authentication with Reddit API
//! - Rate limiting and automatic throttling
//! - Content interaction (posting, commenting, voting)
//! - Subreddit management and discovery
//! - Stealth mode to avoid detection
//! - Proxy support, including Tor integration
//! - Browser automation integration
//! - Stateful session management
//!
//! ## Example
//!
//! ```rust,no_run
//! use llama_moonlight_reddit::{RedditClient, ClientConfig, Result};
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Create a new Reddit client with default configuration
//!     let client = RedditClient::new(ClientConfig::default())
//!         .await?;
//!     
//!     // Authenticate with username and password
//!     let authenticated_client = client.authenticate_username_password(
//!         "your_client_id",
//!         "your_client_secret",
//!         "your_username",
//!         "your_password"
//!     ).await?;
//!     
//!     // Get the top posts from a subreddit
//!     let posts = authenticated_client.subreddit("rust")
//!         .top()
//!         .limit(5)
//!         .fetch()
//!         .await?;
//!     
//!     // Print the titles of the posts
//!     for post in posts {
//!         println!("{}: {}", post.author, post.title);
//!     }
//!     
//!     Ok(())
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::time::Duration;
use std::sync::Arc;
use thiserror::Error;
use url::Url;
use serde::{Serialize, Deserialize};

// Public modules
pub mod auth;
pub mod client;
pub mod models;
pub mod api;
pub mod subreddit;
pub mod user;
pub mod post;
pub mod comment;
pub mod message;
pub mod search;
pub mod multireddit;
pub mod moderation;
pub mod flair;
pub mod awards;
pub mod widgets;
pub mod stream;
pub mod throttle;
pub mod parsing;
pub mod utils;

// Feature-gated modules
#[cfg(feature = "browser")]
pub mod browser;

#[cfg(feature = "stealth")]
pub mod stealth;

#[cfg(feature = "tor")]
pub mod tor;

#[cfg(feature = "mock")]
pub mod mock;

// Re-exports for common types
pub use client::{RedditClient, ClientConfig};
pub use auth::{Authenticator, Credentials, TokenStore};
pub use models::{Thing, Listing, ThingKind};
pub use throttle::RateLimiter;

/// Custom result type for Reddit operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for Reddit operations
#[derive(Debug, Error)]
pub enum Error {
    /// Error related to authentication
    #[error("Authentication error: {0}")]
    AuthError(String),
    
    /// Error returned by Reddit API
    #[error("Reddit API error: {status_code} - {message}")]
    ApiError {
        /// HTTP status code
        status_code: u16,
        /// Error message from Reddit
        message: String,
    },
    
    /// Error handling rate limits
    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),
    
    /// Error parsing Reddit responses
    #[error("Parse error: {0}")]
    ParseError(String),
    
    /// Network errors
    #[error("Network error: {0}")]
    NetworkError(String),
    
    /// HTTP errors from reqwest
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    
    /// URL parsing errors
    #[error("URL error: {0}")]
    UrlError(#[from] url::ParseError),
    
    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    /// IO errors
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    /// OAuth2 errors
    #[error("OAuth2 error: {0}")]
    OAuth2Error(String),
    
    /// Browser automation errors
    #[cfg(feature = "browser")]
    #[error("Browser error: {0}")]
    BrowserError(String),
    
    /// Stealth mode errors
    #[cfg(feature = "stealth")]
    #[error("Stealth error: {0}")]
    StealthError(String),
    
    /// Tor-related errors
    #[cfg(feature = "tor")]
    #[error("Tor error: {0}")]
    TorError(String),
    
    /// Other/unexpected errors
    #[error("Other error: {0}")]
    Other(String),
}

/// Reddit API version
pub const API_VERSION: &str = "v1";

/// Base URL for Reddit API
pub const API_BASE: &str = "https://oauth.reddit.com";

/// Base URL for Reddit authentication
pub const AUTH_BASE: &str = "https://www.reddit.com/api/v1/authorize";

/// Base URL for Reddit token requests
pub const TOKEN_URL: &str = "https://www.reddit.com/api/v1/access_token";

/// Default user agent for Reddit API requests
pub const DEFAULT_USER_AGENT: &str = "llama-moonlight-reddit:v0.1.0 (by /u/llama_moonlight_agent)";

/// Reddit API scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Scope {
    /// View and post identity information
    Identity,
    /// Edit user profile information
    Edit,
    /// Access friend-related sections
    Flair,
    /// Access browsing history 
    History,
    /// Access moderation features
    ModConfig,
    /// Access posts and comments from moderation tools
    ModLog,
    /// Add/remove moderators and contributors
    ModPermissions,
    /// Access and manage moderation mail
    ModMail,
    /// Use moderation tools in subreddits
    ModPosts,
    /// Manage wiki pages
    ModWiki,
    /// Update moderation queue items
    ModWikiEdit,
    /// Access private messages
    PrivateMessages,
    /// Read wiki pages
    Read,
    /// Access posts and comments through your account
    Report,
    /// Save posts and comments
    Save,
    /// Submit and change user's votes on comments and submissions
    Vote,
    /// Submit links and comments
    Submit,
    /// Accept moderator invites
    ModerateInvites,
    /// Manage collections and events
    Collections,
    /// All scopes
    All,
}

/// Listing sort options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Sort {
    /// Sort by hot (default)
    Hot,
    /// Sort by new
    New,
    /// Sort by top
    Top,
    /// Sort by rising
    Rising,
    /// Sort by controversial
    Controversial,
    /// Sort by best (for comments)
    Best,
    /// Sort by relevance (for search)
    Relevance,
}

/// Time range for listings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeRange {
    /// Past hour
    Hour,
    /// Past day
    Day,
    /// Past week
    Week,
    /// Past month
    Month,
    /// Past year
    Year,
    /// All time
    All,
}

/// Vote direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VoteDirection {
    /// Upvote (1)
    Up = 1,
    /// No vote (0)
    None = 0,
    /// Downvote (-1)
    Down = -1,
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let scope_str = match self {
            Scope::Identity => "identity",
            Scope::Edit => "edit",
            Scope::Flair => "flair",
            Scope::History => "history",
            Scope::ModConfig => "modconfig",
            Scope::ModLog => "modlog",
            Scope::ModPermissions => "modpermissions",
            Scope::ModMail => "modmail",
            Scope::ModPosts => "modposts",
            Scope::ModWiki => "modwiki",
            Scope::ModWikiEdit => "modwikiedit",
            Scope::PrivateMessages => "privatemessages",
            Scope::Read => "read",
            Scope::Report => "report",
            Scope::Save => "save",
            Scope::Vote => "vote",
            Scope::Submit => "submit",
            Scope::ModerateInvites => "moderateinvites",
            Scope::Collections => "collections",
            Scope::All => "*",
        };
        write!(f, "{}", scope_str)
    }
}

/// Gets the version of the crate
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_to_string() {
        assert_eq!(Scope::Identity.to_string(), "identity");
        assert_eq!(Scope::All.to_string(), "*");
    }

    #[test]
    fn test_version() {
        assert_eq!(version(), env!("CARGO_PKG_VERSION"));
    }
} 