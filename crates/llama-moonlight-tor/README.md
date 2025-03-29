# 🪐 llama-moonlight-tor

A powerful, privacy-focused Tor integration library for the Llama Moonlight browser automation ecosystem.

[![Crates.io](https://img.shields.io/crates/v/llama-moonlight-tor.svg)](https://crates.io/crates/llama-moonlight-tor)
[![Documentation](https://docs.rs/llama-moonlight-tor/badge.svg)](https://docs.rs/llama-moonlight-tor)
[![License: MIT/Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

## Features

- **Tor Network Integration**: Connect your applications to the Tor network for enhanced privacy and anonymity
- **Circuit Management**: Create, manage, and customize Tor circuits for tailored routing
- **Identity Rotation**: Easily rotate Tor identities to change your apparent IP address
- **Onion Service Access**: Seamlessly access .onion services on the dark web
- **Dark Web Search Capabilities**: Aggregate search results from dark web search engines
- **Full Async Support**: Built with Tokio and async/await for non-blocking operations
- **Metadata Collection**: Extract and manage metadata from onion services
- **Bridge Mode Support**: Connect through Tor bridges in censored environments
- **Circuit Isolation**: Configure isolated circuits for different tasks
- **Proxy Protocol Support**: SOCKS5 proxy integration for various applications
- **Safety Focused**: Written in safe Rust with comprehensive error handling
- **Elegant API**: Simple, intuitive API that integrates with the Llama Moonlight ecosystem

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
llama-moonlight-tor = "0.1.0"
```

## Quick Start

```rust
use llama_moonlight_tor::{TorClient, TorConfig};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Create a basic Tor configuration
    let config = TorConfig::default();
    
    // Create and initialize a Tor client
    let tor_client = TorClient::new(config);
    tor_client.init().await?;
    
    // Verify connection is working
    if tor_client.is_using_tor().await? {
        println!("Successfully connected to Tor network!");
        
        // Get the current exit node IP
        let ip = tor_client.get_ip().await?;
        println!("Current Tor exit IP: {}", ip);
        
        // Get a new identity (new circuit)
        let circuit_id = tor_client.new_circuit().await?;
        println!("Created new circuit: {}", circuit_id);
    }
    
    Ok(())
}
```

## Examples

### Accessing an Onion Service

```rust
use llama_moonlight_tor::{TorClient, TorConfig};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = TorConfig::default();
    let tor_client = TorClient::new(config);
    tor_client.init().await?;
    
    // Access the Tor Project's onion site
    let response = tor_client.access_onion("http://2gzyxa5ihm7nsggfxnu52rck2vv4rvmdlkiu3zzui5du4xyclen53wid.onion").await?;
    
    if response.status().is_success() {
        let body = response.text().await?;
        println!("Successfully accessed onion service! Page title: {:?}", 
            extract_title(&body).unwrap_or_else(|| "No title found".to_string()));
    }
    
    Ok(())
}

// Extract title from HTML
fn extract_title(html: &str) -> Option<String> {
    let re = regex::Regex::new(r"<title[^>]*>(.*?)</title>").ok()?;
    re.captures(html).and_then(|cap| {
        cap.get(1).map(|m| m.as_str().to_string())
    })
}
```

### Custom Circuit Configuration

```rust
use llama_moonlight_tor::{TorClient, TorConfig};
use std::collections::HashMap;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Create a custom configuration
    let mut options = HashMap::new();
    options.insert("StrictNodes".to_string(), "1".to_string());
    
    let config = TorConfig {
        // Only use exit nodes in these countries
        exit_nodes: Some("{us},{ca},{fr}".to_string()),
        // Add custom Tor configuration options
        options,
        ..TorConfig::default()
    };
    
    let tor_client = TorClient::new(config);
    tor_client.init().await?;
    
    // Make a request through the configured circuit
    let response = tor_client.get("https://check.torproject.org").await?;
    println!("Response status: {}", response.status());
    
    // Get circuit information
    if let Some(circuit) = tor_client.get_circuit_info().await? {
        println!("Circuit ID: {}", circuit.id);
        println!("Exit Node: {:?}", circuit.get_exit_node());
        println!("Circuit Countries: {:?}", circuit.get_countries());
    }
    
    Ok(())
}
```

### Using Bridges in Restricted Environments

```rust
use llama_moonlight_tor::{TorClient, TorConfig};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Configure Tor to use bridges
    let config = TorConfig {
        use_bridges: true,
        bridges: vec![
            // Example bridge lines - replace with actual bridges
            "obfs4 X.X.X.X:YYYY FINGERPRINT cert=CERT iat-mode=0".to_string(),
            "snowflake 0.0.3.0:1 FINGERPRINT".to_string(),
        ],
        ..TorConfig::default()
    };
    
    let tor_client = TorClient::new(config);
    tor_client.init().await?;
    
    // Verify connectivity
    if tor_client.is_using_tor().await? {
        println!("Successfully connected to Tor via bridges!");
    }
    
    Ok(())
}
```

### Collecting Onion Service Metadata

```rust
use llama_moonlight_tor::{TorClient, TorConfig, onion::OnionService};
use std::error::Error;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = TorConfig::default();
    let tor_client = Arc::new(TorClient::new(config));
    tor_client.init().await?;
    
    // Create an onion service handler
    let onion_service = OnionService::new(tor_client.clone());
    
    // List of onion services to check
    let onion_addresses = [
        "duckduckgogg42xjoc72x3sjasowoarfbgcmvfimaftt6twagswzczad.onion",
        "darkfailenbsdla5mal2mxn2uz66od5vtzd5qozslagrfzachha3f3id.onion",
    ];
    
    for address in &onion_addresses {
        match onion_service.extract_metadata(address).await {
            Ok(metadata) => {
                println!("Onion service: {}", address);
                println!("  Title: {:?}", metadata.title);
                println!("  Description: {:?}", metadata.description);
                println!("  Online: {}", metadata.is_online);
                println!("  Response time: {:?}ms", metadata.avg_response_time_ms);
            },
            Err(e) => println!("Failed to access {}: {}", address, e),
        }
    }
    
    Ok(())
}
```

## Advanced Usage

For advanced usage and full API documentation, please see our [documentation](https://docs.rs/llama-moonlight-tor).

### Integration with Llama Moonlight Ecosystem

This crate is designed to work seamlessly with the other components of the Llama Moonlight ecosystem:

- `llama-moonlight-core` - Core functionality for browser automation
- `llama-moonlight-headers` - HTTP header generation and management
- `llama-moonlight-stealth` - Browser fingerprint protection and bot detection evasion

## Configuration

The `TorConfig` struct allows for flexible configuration:

```rust
let config = TorConfig {
    data_dir: std::path::PathBuf::from("./tor_data"),
    socks_host: "127.0.0.1".to_string(),
    socks_port: 9050,
    control_port: 9051,
    control_password: Some("your_password_hash".to_string()),
    exit_nodes: Some("{us},{ca}".to_string()),
    use_bridges: false,
    bridges: Vec::new(),
    options: HashMap::new(),
    timeout_secs: 60,
    ..TorConfig::default()
};
```

## Safety and Security Considerations

- This library is designed for legitimate privacy-focused applications
- Use responsibly and in accordance with applicable laws and regulations
- Be aware that while Tor provides anonymity, it is not perfect; practice good operational security
- We strongly recommend reading the [Tor Project's documentation](https://www.torproject.org/docs/documentation.html) for best practices

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Disclaimer

This software is provided for legitimate privacy and security research purposes. The authors and contributors are not responsible for any misuse or illegal activities conducted with this software. 