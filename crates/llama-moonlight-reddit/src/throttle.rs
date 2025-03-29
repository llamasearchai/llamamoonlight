//! Rate limiting for Reddit API requests
//!
//! This module provides functionality to respect Reddit's rate limits.
//! Reddit limits API usage to a specific number of requests per minute,
//! and this module helps ensure those limits are not exceeded.

use std::time::{Duration, Instant};
use tokio::time::sleep;
use log::{debug, warn};

use crate::{Result, Error};

/// A rate limiter for Reddit API requests
#[derive(Debug)]
pub struct RateLimiter {
    /// Maximum number of requests allowed in the time window
    max_requests: u32,
    
    /// Duration of the time window
    time_window: Duration,
    
    /// Number of requests remaining in the current window
    remaining: u32,
    
    /// Time when the rate limit resets
    reset_time: Duration,
    
    /// Last time the rate limiter was reset
    last_reset: Instant,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(max_requests: u32, time_window: Duration) -> Self {
        Self {
            max_requests,
            time_window,
            remaining: max_requests,
            reset_time: time_window,
            last_reset: Instant::now(),
        }
    }
    
    /// Acquire permission to make a request, waiting if necessary
    pub async fn acquire(&mut self) -> Result<()> {
        // Check if the time window has elapsed
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_reset);
        
        if elapsed >= self.time_window {
            // Reset the rate limiter
            self.remaining = self.max_requests;
            self.last_reset = now;
            self.reset_time = self.time_window;
        } else {
            // Update the reset time
            self.reset_time = self.time_window.saturating_sub(elapsed);
        }
        
        // Check if we have any requests remaining
        if self.remaining == 0 {
            // Calculate how long to wait
            let wait_time = self.reset_time;
            
            debug!("Rate limit exceeded, waiting for {} seconds", wait_time.as_secs());
            
            // Wait until the rate limit resets
            sleep(wait_time).await;
            
            // Reset the rate limiter
            self.remaining = self.max_requests;
            self.last_reset = Instant::now();
            self.reset_time = self.time_window;
        }
        
        // Decrement the remaining requests
        self.remaining -= 1;
        
        Ok(())
    }
    
    /// Get the number of requests remaining
    pub fn remaining(&self) -> u32 {
        self.remaining
    }
    
    /// Get the time until the rate limit resets
    pub fn reset_time(&self) -> Duration {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_reset);
        
        if elapsed >= self.time_window {
            Duration::from_secs(0)
        } else {
            self.time_window.saturating_sub(elapsed)
        }
    }
    
    /// Set the remaining requests based on response headers
    pub fn set_remaining(&mut self, remaining: u32) {
        self.remaining = remaining;
    }
    
    /// Set the reset time based on response headers
    pub fn set_reset(&mut self, reset: Duration) {
        self.reset_time = reset;
    }
    
    /// Check if a request can be made immediately
    pub fn can_request(&self) -> bool {
        self.remaining > 0
    }
    
    /// Get the maximum number of requests
    pub fn max_requests(&self) -> u32 {
        self.max_requests
    }
    
    /// Set the maximum number of requests
    pub fn set_max_requests(&mut self, max_requests: u32) {
        self.max_requests = max_requests;
    }
    
    /// Get the time window
    pub fn time_window(&self) -> Duration {
        self.time_window
    }
    
    /// Set the time window
    pub fn set_time_window(&mut self, time_window: Duration) {
        self.time_window = time_window;
    }
}

/// Throttle settings for different Reddit API endpoints
#[derive(Debug, Clone)]
pub struct ThrottleSettings {
    /// Rate limit for regular API requests
    pub api: (u32, Duration),
    
    /// Rate limit for OAuth API requests
    pub oauth_api: (u32, Duration),
    
    /// Rate limit for listing endpoint requests
    pub listings: (u32, Duration),
    
    /// Rate limit for search endpoint requests
    pub search: (u32, Duration),
    
    /// Rate limit for posting comments
    pub post_comment: (u32, Duration),
    
    /// Rate limit for submitting posts
    pub submit_post: (u32, Duration),
    
    /// Rate limit for voting
    pub vote: (u32, Duration),
}

impl Default for ThrottleSettings {
    fn default() -> Self {
        Self {
            // Reddit's limits are:
            // - 60 requests per minute for OAuth clients
            // - 30 requests per minute for regular API
            // - Lower limits for some specific endpoints
            
            // We'll be conservative with our defaults
            api: (30, Duration::from_secs(60)),
            oauth_api: (50, Duration::from_secs(60)),
            listings: (25, Duration::from_secs(60)),
            search: (15, Duration::from_secs(60)),
            post_comment: (5, Duration::from_secs(60)),
            submit_post: (3, Duration::from_secs(60)),
            vote: (10, Duration::from_secs(60)),
        }
    }
}

/// A manager for multiple rate limiters
#[derive(Debug)]
pub struct ThrottleManager {
    /// Settings for rate limiting
    settings: ThrottleSettings,
    
    /// Rate limiter for regular API requests
    api_limiter: RateLimiter,
    
    /// Rate limiter for OAuth API requests
    oauth_api_limiter: RateLimiter,
    
    /// Rate limiter for listing endpoint requests
    listings_limiter: RateLimiter,
    
    /// Rate limiter for search endpoint requests
    search_limiter: RateLimiter,
    
    /// Rate limiter for posting comments
    post_comment_limiter: RateLimiter,
    
    /// Rate limiter for submitting posts
    submit_post_limiter: RateLimiter,
    
    /// Rate limiter for voting
    vote_limiter: RateLimiter,
}

impl ThrottleManager {
    /// Create a new throttle manager with default settings
    pub fn new() -> Self {
        Self::with_settings(ThrottleSettings::default())
    }
    
    /// Create a new throttle manager with custom settings
    pub fn with_settings(settings: ThrottleSettings) -> Self {
        Self {
            api_limiter: RateLimiter::new(settings.api.0, settings.api.1),
            oauth_api_limiter: RateLimiter::new(settings.oauth_api.0, settings.oauth_api.1),
            listings_limiter: RateLimiter::new(settings.listings.0, settings.listings.1),
            search_limiter: RateLimiter::new(settings.search.0, settings.search.1),
            post_comment_limiter: RateLimiter::new(settings.post_comment.0, settings.post_comment.1),
            submit_post_limiter: RateLimiter::new(settings.submit_post.0, settings.submit_post.1),
            vote_limiter: RateLimiter::new(settings.vote.0, settings.vote.1),
            settings,
        }
    }
    
    /// Get the appropriate rate limiter for an endpoint
    pub fn get_limiter(&mut self, endpoint: &str) -> &mut RateLimiter {
        if endpoint.contains("/api/v1/") {
            // OAuth API
            &mut self.oauth_api_limiter
        } else if endpoint.contains("/search") {
            // Search endpoint
            &mut self.search_limiter
        } else if endpoint.contains("/comments") || endpoint.contains("/api/comment") {
            // Comment endpoint
            &mut self.post_comment_limiter
        } else if endpoint.contains("/api/submit") {
            // Submit endpoint
            &mut self.submit_post_limiter
        } else if endpoint.contains("/api/vote") {
            // Vote endpoint
            &mut self.vote_limiter
        } else if is_listing_endpoint(endpoint) {
            // Listing endpoint
            &mut self.listings_limiter
        } else {
            // Regular API
            &mut self.api_limiter
        }
    }
    
    /// Acquire permission to make a request to the specified endpoint
    pub async fn acquire(&mut self, endpoint: &str) -> Result<()> {
        let limiter = self.get_limiter(endpoint);
        limiter.acquire().await
    }
    
    /// Update throttle settings
    pub fn update_settings(&mut self, settings: ThrottleSettings) {
        self.settings = settings;
        
        self.api_limiter.set_max_requests(settings.api.0);
        self.api_limiter.set_time_window(settings.api.1);
        
        self.oauth_api_limiter.set_max_requests(settings.oauth_api.0);
        self.oauth_api_limiter.set_time_window(settings.oauth_api.1);
        
        self.listings_limiter.set_max_requests(settings.listings.0);
        self.listings_limiter.set_time_window(settings.listings.1);
        
        self.search_limiter.set_max_requests(settings.search.0);
        self.search_limiter.set_time_window(settings.search.1);
        
        self.post_comment_limiter.set_max_requests(settings.post_comment.0);
        self.post_comment_limiter.set_time_window(settings.post_comment.1);
        
        self.submit_post_limiter.set_max_requests(settings.submit_post.0);
        self.submit_post_limiter.set_time_window(settings.submit_post.1);
        
        self.vote_limiter.set_max_requests(settings.vote.0);
        self.vote_limiter.set_time_window(settings.vote.1);
    }
}

/// Check if an endpoint is a listing endpoint
fn is_listing_endpoint(endpoint: &str) -> bool {
    // Listing endpoints include:
    // - /hot
    // - /new
    // - /top
    // - /rising
    // - /controversial
    // - /best
    // - /random
    
    endpoint.contains("/hot") ||
    endpoint.contains("/new") ||
    endpoint.contains("/top") ||
    endpoint.contains("/rising") ||
    endpoint.contains("/controversial") ||
    endpoint.contains("/best") ||
    endpoint.contains("/random")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(5, Duration::from_secs(1));
        
        assert_eq!(limiter.remaining(), 5);
        assert_eq!(limiter.max_requests(), 5);
        assert_eq!(limiter.time_window(), Duration::from_secs(1));
        assert!(limiter.can_request());
    }
    
    #[test]
    fn test_throttle_settings() {
        let settings = ThrottleSettings::default();
        
        assert_eq!(settings.api.0, 30);
        assert_eq!(settings.api.1, Duration::from_secs(60));
        
        assert_eq!(settings.oauth_api.0, 50);
        assert_eq!(settings.oauth_api.1, Duration::from_secs(60));
    }
    
    #[test]
    fn test_is_listing_endpoint() {
        assert!(is_listing_endpoint("/r/rust/hot"));
        assert!(is_listing_endpoint("/r/rust/new"));
        assert!(is_listing_endpoint("/r/rust/top"));
        assert!(is_listing_endpoint("/r/rust/rising"));
        assert!(is_listing_endpoint("/r/rust/controversial"));
        assert!(is_listing_endpoint("/r/rust/best"));
        assert!(is_listing_endpoint("/r/rust/random"));
        
        assert!(!is_listing_endpoint("/r/rust/about"));
        assert!(!is_listing_endpoint("/api/submit"));
        assert!(!is_listing_endpoint("/api/v1/me"));
    }
} 