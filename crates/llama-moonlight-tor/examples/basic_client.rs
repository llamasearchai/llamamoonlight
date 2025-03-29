//! Basic TorClient Example
//! 
//! This example demonstrates how to create and use a basic Tor client
//! to make requests through the Tor network.

use llama_moonlight_tor::{TorClient, TorConfig};
use std::error::Error;
use std::time::Duration;

/// Main function that demonstrates basic TorClient usage
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Starting Tor client example...");
    
    // Create a basic Tor configuration
    let config = TorConfig::default();
    
    // Create a Tor client
    let tor_client = TorClient::new(config)
        .with_timeout(60)
        .with_auto_rotate_circuits(true)
        .with_requests_per_circuit(5);
    
    println!("Initializing Tor client...");
    // Initialize the client
    tor_client.init().await?;
    
    // Check if we're using Tor
    if tor_client.is_using_tor().await? {
        println!("Successfully connected to Tor!");
        
        // Get our current IP as seen from the Internet
        let ip = tor_client.get_ip().await?;
        println!("Current Tor exit IP: {}", ip);
        
        // Get circuit information
        if let Some(circuit) = tor_client.get_circuit_info().await? {
            println!("Current circuit ID: {}", circuit.id);
            if let Some(exit_node) = circuit.get_exit_node() {
                println!("Exit node: {} in {}", exit_node.name, exit_node.country.unwrap_or("unknown".to_string()));
            }
        }
        
        // Make a request through Tor
        println!("\nMaking a request to check.torproject.org...");
        let response = tor_client.get("https://check.torproject.org").await?;
        
        println!("Response status: {}", response.status());
        let body = response.text().await?;
        
        // Check if the response contains the "Congratulations" message
        if body.contains("Congratulations. This browser is configured to use Tor") {
            println!("Tor check successful: We are using Tor!\n");
        } else {
            println!("Tor check indicates we might not be using Tor correctly.\n");
        }
        
        // Get a new identity (new circuit)
        println!("Requesting new Tor circuit...");
        let circuit_id = tor_client.new_circuit().await?;
        println!("Created new circuit with ID: {}", circuit_id);
        
        // Wait a moment for the circuit to establish
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // Get our new IP
        let new_ip = tor_client.get_ip().await?;
        println!("New Tor exit IP: {}", new_ip);
        
        if ip != new_ip {
            println!("Successfully changed identity!");
        } else {
            println!("IP didn't change. This can happen if exit nodes are limited or if rotation was too quick.");
        }
        
        // Get updated circuit information
        if let Some(circuit) = tor_client.get_circuit_info().await? {
            println!("New circuit ID: {}", circuit.id);
            if let Some(exit_node) = circuit.get_exit_node() {
                println!("New exit node: {} in {}", exit_node.name, exit_node.country.unwrap_or("unknown".to_string()));
            }
        }
    } else {
        println!("Not connected to Tor. Please check your Tor configuration.");
    }
    
    println!("\nExiting Tor client example.");
    Ok(())
} 