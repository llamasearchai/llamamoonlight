//! Dark Web Search Example
//!
//! This example demonstrates how to use the dark web search aggregator
//! to search for information across multiple dark web search engines.

use llama_moonlight_tor::{TorClient, TorConfig};
use llama_moonlight_tor::search::{DarkWebSearch, SearchResult};
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

/// Print a search result in a formatted way
fn print_result(index: usize, result: &SearchResult) {
    println!("{}. {} [{}]", index + 1, result.title, result.source);
    println!("   URL: {}", result.url);
    if let Some(snippet) = &result.snippet {
        let shortened = if snippet.len() > 100 {
            format!("{}...", &snippet[0..100])
        } else {
            snippet.clone()
        };
        println!("   Snippet: {}", shortened);
    }
    println!("   Relevance: {:.2}", result.relevance);
    println!("   Verified: {}", if result.verified { "✓" } else { "✗" });
    println!();
}

/// Main function that demonstrates dark web search functionality
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Starting dark web search example...");
    
    // Read search query from arguments or use default
    let args: Vec<String> = std::env::args().collect();
    let query = if args.len() > 1 {
        args[1].clone()
    } else {
        "privacy tools".to_string()
    };
    
    println!("Initializing Tor client...");
    
    // Create a basic Tor configuration
    let config = TorConfig::default();
    
    // Create a Tor client
    let tor_client = Arc::new(TorClient::new(config));
    
    // Initialize the client
    tor_client.init().await?;
    
    // Wait a moment for Tor to initialize
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // Check if we're using Tor
    if tor_client.is_using_tor().await? {
        println!("Successfully connected to Tor!");
        
        // Create the dark web search aggregator
        println!("Creating dark web search aggregator...");
        let mut search = DarkWebSearch::new(tor_client.clone());
        
        // Configure search settings
        search.set_timeout(Duration::from_secs(30));
        search.set_max_concurrent_requests(2); // Limit concurrent requests to avoid overwhelming Tor
        
        // Perform the search
        println!("\nSearching for '{}' across dark web search engines...", query);
        println!("This may take a while depending on Tor connection speed...\n");
        
        let results = match search.search(&query).await {
            Ok(res) => res,
            Err(e) => {
                println!("Search failed: {}", e);
                return Ok(());
            }
        };
        
        // Print search statistics
        println!("Search complete! Found {} results.", results.len());
        
        // Get top 10 results
        let top_results = if results.len() > 10 {
            &results[0..10]
        } else {
            &results
        };
        
        // Print the results
        println!("\nTop results:");
        for (i, result) in top_results.iter().enumerate() {
            print_result(i, result);
        }
        
        // Verify the top 5 results are online
        if !results.is_empty() {
            let verification_count = std::cmp::min(5, results.len());
            let mut results_to_verify = results[0..verification_count].to_vec();
            
            println!("\nVerifying top {} results are online...", verification_count);
            if let Err(e) = search.verify_results(&mut results_to_verify).await {
                println!("Verification error: {}", e);
            }
            
            println!("\nVerified results:");
            for (i, result) in results_to_verify.iter().enumerate() {
                print_result(i, result);
            }
            
            // Extract metadata for the first verified result
            if let Some(first_verified) = results_to_verify.iter().find(|r| r.verified) {
                println!("\nExtracting metadata for '{}' ({})", first_verified.title, first_verified.url);
                let mut result_with_metadata = vec![first_verified.clone()];
                
                if let Err(e) = search.extract_metadata(&mut result_with_metadata).await {
                    println!("Metadata extraction error: {}", e);
                } else if !result_with_metadata.is_empty() {
                    let enriched = &result_with_metadata[0];
                    
                    println!("Enriched result:");
                    println!("  Title: {}", enriched.title);
                    println!("  Snippet: {:?}", enriched.snippet);
                    
                    // Print any additional metadata
                    if !enriched.metadata.is_empty() {
                        println!("  Additional metadata:");
                        for (key, value) in &enriched.metadata {
                            println!("    {}: {}", key, value);
                        }
                    }
                }
            }
        }
    } else {
        println!("Not connected to Tor. Please check your Tor configuration.");
    }
    
    println!("\nExiting dark web search example.");
    Ok(())
}