//! Basic example of using the Reddit client
//!
//! This example demonstrates how to create a Reddit client,
//! authenticate, and browse content.

use std::error::Error;
use std::env;
use llama_moonlight_reddit::{RedditClient, ClientConfig, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    
    // Create a basic Reddit client
    let config = ClientConfig::default()
        .with_user_agent("llama-moonlight-reddit:examples:v0.1.0");
    
    let client = RedditClient::new(config).await?;
    
    println!("Created unauthenticated Reddit client");
    
    // Check if credentials are provided via environment variables
    let client_id = env::var("REDDIT_CLIENT_ID");
    let client_secret = env::var("REDDIT_CLIENT_SECRET");
    let username = env::var("REDDIT_USERNAME");
    let password = env::var("REDDIT_PASSWORD");
    
    // Try to authenticate if all credentials are available
    let client = if client_id.is_ok() && client_secret.is_ok() && username.is_ok() && password.is_ok() {
        println!("Authenticating with username/password...");
        
        client.authenticate_username_password(
            &client_id.unwrap(),
            &client_secret.unwrap(),
            &username.unwrap(),
            &password.unwrap(),
        ).await?
    } else {
        println!("No authentication credentials found in environment variables.");
        println!("Running in read-only mode.\n");
        
        // We can still use the client for browsing without authentication
        client
    };
    
    // Get front page posts
    println!("Fetching front page posts...");
    let subreddit = client.subreddit("all");
    let posts = subreddit.hot().limit(5).fetch().await?;
    
    println!("Retrieved {} front page posts:", posts.len());
    for (i, post) in posts.iter().enumerate() {
        println!("{}. [{}] {} (by u/{}) - {} upvotes, {} comments",
            i + 1,
            post.subreddit_name_prefixed,
            post.title,
            post.author,
            post.score,
            post.num_comments
        );
    }
    
    // Get posts from a specific subreddit
    let subreddit_name = "rust";
    println!("\nFetching top posts from r/{}...", subreddit_name);
    
    let subreddit = client.subreddit(subreddit_name);
    let posts = subreddit.top().time(llama_moonlight_reddit::TimeRange::Week).limit(5).fetch().await?;
    
    println!("Retrieved {} posts from r/{}:", posts.len(), subreddit_name);
    for (i, post) in posts.iter().enumerate() {
        println!("{}. {} (by u/{}) - {} upvotes, {} comments",
            i + 1,
            post.title,
            post.author,
            post.score,
            post.num_comments
        );
    }
    
    // Get subreddit information
    println!("\nFetching information about r/{}...", subreddit_name);
    let info = subreddit.about().await?;
    
    println!("Subreddit: r/{}", info.display_name);
    println!("Title: {}", info.title);
    println!("Description: {}", info.public_description);
    println!("Subscribers: {}", info.subscribers);
    println!("Created: {}", info.created_utc);
    
    // Get subreddit rules
    println!("\nFetching rules for r/{}...", subreddit_name);
    let rules = subreddit.rules().await?;
    
    println!("Rules:");
    for (i, rule) in rules.iter().enumerate() {
        println!("{}. {}: {}", i + 1, rule.short_name, rule.description);
    }
    
    // Fetch user information (if authenticated)
    if client.is_authenticated().await {
        println!("\nFetching user information...");
        
        let me = client.me().await?;
        println!("Logged in as: u/{}", me.username);
        println!("Post karma: {}", me.link_karma);
        println!("Comment karma: {}", me.comment_karma);
        
        // Fetch user's subreddits
        println!("\nFetching subscribed subreddits...");
        // Note: This would require the 'mysubreddits' scope
        // Implementation would go here
    }
    
    // Search for posts
    let query = "async programming";
    println!("\nSearching for posts about '{}'...", query);
    
    let search = client.search()
        .query(query)
        .subreddit(subreddit_name) // optional
        .limit(5)
        .execute().await?;
    
    println!("Search results:");
    for (i, result) in search.iter().enumerate() {
        println!("{}. [{}] {} (by u/{}) - {} upvotes",
            i + 1,
            result.subreddit_name_prefixed,
            result.title,
            result.author,
            result.score
        );
    }
    
    println!("\nExample completed successfully!");
    Ok(())
} 