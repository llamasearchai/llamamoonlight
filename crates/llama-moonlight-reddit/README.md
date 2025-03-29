# 🦙 llama-moonlight-reddit

[![Crates.io](https://img.shields.io/crates/v/llama-moonlight-reddit.svg)](https://crates.io/crates/llama-moonlight-reddit)
[![Documentation](https://docs.rs/llama-moonlight-reddit/badge.svg)](https://docs.rs/llama-moonlight-reddit)
[![MIT/Apache-2.0 licensed](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](./LICENSE)

A powerful, comprehensive Reddit API client for the Llama Moonlight ecosystem, providing robust automation and interaction capabilities with Reddit's API.

## Features

- **Full API Coverage**: Comprehensive support for Reddit's API endpoints
- **Type-Safe Interface**: Strongly typed models for all Reddit entities
- **OAuth Authentication**: Support for multiple authentication flows
- **Rate Limiting**: Built-in rate limiting to respect Reddit's API guidelines
- **Async/Await**: Built on Tokio for non-blocking operations
- **Fluent Builder API**: Intuitive, chainable methods for constructing requests
- **Error Handling**: Detailed error types and informative messages
- **Pagination Support**: Easy handling of paginated responses
- **Stealth Mode**: Options to avoid detection (with stealth feature)
- **Tor Integration**: Anonymous browsing capability (with tor feature)
- **Customizable**: Configurable user agents, proxies, and timeouts
- **Streaming**: Real-time updates via Reddit's streaming API
- **Moderation Tools**: Comprehensive moderator action support
- **Search**: Advanced search capabilities with multiple filters

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
llama-moonlight-reddit = "0.1.0"
```

For additional features:

```toml
[dependencies]
llama-moonlight-reddit = { version = "0.1.0", features = ["stealth", "tor"] }
```

## Quick Start

```rust
use llama_moonlight_reddit::{RedditClient, ClientConfig, Result};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<()> {
    // Create a client with default configuration
    let config = ClientConfig::default()
        .with_user_agent("MyApp/1.0 (by /u/username)");
    
    let client = RedditClient::new(config).await?;
    
    // For read-only browsing, authentication is optional
    // Get hot posts from r/rust
    let subreddit = client.subreddit("rust");
    let posts = subreddit.hot().limit(5).fetch().await?;
    
    // Display the posts
    for post in posts {
        println!("{}: {}", post.author, post.title);
    }
    
    Ok(())
}
```

## Authentication

### Password Flow

```rust
let authenticated_client = client.authenticate_username_password(
    "your_client_id",
    "your_client_secret",
    "your_username",
    "your_password"
).await?;
```

### Refresh Token Flow

```rust
let authenticated_client = client.authenticate_refresh_token(
    "your_client_id",
    "your_client_secret",
    "your_refresh_token"
).await?;
```

## Browsing Content

### Subreddits

```rust
// Get information about a subreddit
let subreddit = client.subreddit("rust");
let info = subreddit.about().await?;
println!("{}: {}", info.display_name, info.public_description);

// Get posts from a subreddit
let posts = subreddit.hot().limit(10).fetch().await?;
// or .new(), .rising(), .top(), .controversial()

// With time filter for top/controversial
let top_posts = subreddit.top()
    .time(TimeRange::Week)
    .limit(25)
    .fetch()
    .await?;
```

### Posts and Comments

```rust
// Get a specific post
let post = client.post("t3_abcdef").fetch().await?;

// Get comments on a post
let comments = client.post("t3_abcdef").comments().fetch().await?;

// Submit a new post
let post_id = client.subreddit("test").submit(
    "Test post title",
    "self", // or "link", "image", "video", "poll"
    Some("Post content"), 
    None, // URL if "link" type
    false, // NSFW
    false, // Spoiler
    None, // Flair ID
    None, // Flair text
).await?;
```

### User Profiles

```rust
// Get a user's profile
let user = client.user("spez").about().await?;
println!("Created: {}", user.created_utc);

// Get a user's posts
let user_posts = client.user("spez").submitted()
    .limit(10)
    .fetch()
    .await?;

// Get a user's comments
let user_comments = client.user("spez").comments()
    .limit(10)
    .fetch()
    .await?;
```

### Searching

```rust
// Search all of Reddit
let search_results = client.search()
    .query("async programming")
    .limit(25)
    .sort(Sort::Relevance)
    .time(TimeRange::Month)
    .execute()
    .await?;

// Search within a subreddit
let subreddit_results = client.search()
    .query("async programming")
    .subreddit("rust")
    .limit(25)
    .execute()
    .await?;

// Search for subreddits
let subreddits = client.search()
    .query("programming")
    .type_(SearchType::Subreddit)
    .limit(10)
    .execute_subreddits()
    .await?;
```

### Interacting with Content

```rust
// Vote on a post or comment
client.vote("t3_abcdef", VoteDirection::Up).await?;

// Save a post or comment
client.post("t3_abcdef").save().await?;

// Comment on a post
let comment_id = client.submit_comment("t3_abcdef", "This is my comment").await?;

// Reply to a comment
let reply_id = client.submit_comment("t1_ghijkl", "This is my reply").await?;
```

## Advanced Features

### Stealth Mode (with `stealth` feature)

```rust
use llama_moonlight_reddit::stealth::StealthConfig;

let config = ClientConfig::default()
    .with_stealth(StealthConfig::default());

let client = RedditClient::new(config).await?;
```

### Tor Integration (with `tor` feature)

```rust
use llama_moonlight_reddit::tor::TorConfig;

let config = ClientConfig::default()
    .with_tor(TorConfig::default());

let client = RedditClient::new(config).await?;
```

### Streaming Comments (Real-time Updates)

```rust
use futures::StreamExt;

let mut stream = client.subreddit("askreddit")
    .stream_comments()
    .await?;

while let Some(comment) = stream.next().await {
    println!("New comment: {}", comment.body);
}
```

### Custom Rate Limits

```rust
use std::time::Duration;
use llama_moonlight_reddit::throttle::ThrottleSettings;

let settings = ThrottleSettings {
    api: (30, Duration::from_secs(60)),
    oauth_api: (60, Duration::from_secs(60)),
    // ... other settings
    ..Default::default()
};

let config = ClientConfig::default()
    .with_rate_limiter(true);

let client = RedditClient::new(config).await?;
// Update the rate limiter with custom settings
client.update_rate_limits(settings).await?;
```

## Error Handling

The library uses a custom `Error` type for all errors:

```rust
match result {
    Ok(posts) => {
        // Handle success
    },
    Err(e) => match e {
        Error::AuthError(msg) => {
            // Handle authentication errors
        },
        Error::RateLimitError(msg) => {
            // Handle rate limit errors
        },
        Error::ApiError { status_code, message } => {
            // Handle API errors
        },
        // Handle other error types
        _ => println!("Error: {}", e),
    }
}
```

## Features

The crate can be compiled with the following features:

- `standard` (default): Core functionality for Reddit API access
- `browser`: Integration with browser automation for enhanced capabilities
- `stealth`: Features to help avoid detection
- `tor`: Integration with Tor for anonymous browsing
- `api-extended`: Extended API functionality
- `moderation`: Moderation tools for subreddit moderators
- `full`: Enables all features

## Examples

See the [examples](examples/) directory for more comprehensive examples:

- `basic_client.rs`: Simple client for browsing Reddit
- `auth_flows.rs`: Different authentication flows
- `subreddit_browser.rs`: Browsing subreddit content
- `post_and_comment.rs`: Creating and interacting with posts/comments
- `search.rs`: Advanced search capabilities
- `streaming.rs`: Real-time streaming of new content
- `moderation.rs`: Moderation tools and actions
- `stealth_mode.rs`: Using stealth mode to avoid detection
- `tor_browsing.rs`: Anonymous browsing via Tor

## Concurrency and Rate Limiting

The library handles concurrency properly with tokio and respects Reddit's rate limits:

- Default rate limit is 60 requests per minute for authenticated requests
- Different rate limits for different endpoints
- Automatic throttling when approaching limits
- Headers from Reddit are used to update rate limit information

## Documentation

For full API documentation, please visit [docs.rs/llama-moonlight-reddit](https://docs.rs/llama-moonlight-reddit).

## Comparison with Other Reddit Clients

| Feature                | llama-moonlight-reddit | roux | reddit-api-rs |
|------------------------|:----------------------:|:----:|:-------------:|
| Async/Await            | ✅                     | ✅   | ❌            |
| Full API Coverage      | ✅                     | ⚠️   | ⚠️            |
| OAuth Authentication   | ✅                     | ✅   | ⚠️            |
| Rate Limiting          | ✅                     | ❌   | ❌            |
| Streaming              | ✅                     | ❌   | ❌            |
| Stealth/Tor Support    | ✅                     | ❌   | ❌            |
| Builder Pattern API    | ✅                     | ⚠️   | ⚠️            |
| Comprehensive Models   | ✅                     | ⚠️   | ⚠️            |
| Active Development     | ✅                     | ✅   | ❌            |

## License

This project is licensed under either of:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Disclaimer

This library is not officially affiliated with Reddit, Inc. Use responsibly and in accordance with [Reddit's API Terms](https://www.reddit.com/dev/api/). Automated usage of Reddit should follow their guidelines to avoid rate limiting or account suspension. 