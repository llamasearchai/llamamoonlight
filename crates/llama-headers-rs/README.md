# 🔍 llama-headers-rs

[![Crates.io](https://img.shields.io/crates/v/llama-headers-rs.svg)](https://crates.io/crates/llama-headers-rs)
[![Documentation](https://docs.rs/llama-headers-rs/badge.svg)](https://docs.rs/llama-headers-rs)
[![License](https://img.shields.io/crates/l/llama-headers-rs.svg)](https://github.com/yourusername/llama-ecosystem/blob/main/llama-headers-rs/LICENSE)
[![Build Status](https://github.com/yourusername/llama-ecosystem/workflows/Rust/badge.svg)](https://github.com/yourusername/llama-ecosystem/actions)

A sophisticated HTTP header generation library for realistic browser emulation.

## Features

- **User-Agent Generation**: Create realistic browser fingerprints
- **Language Detection**: Locale-aware header generation
- **Context-Aware Referers**: Referer generation based on domain context
- **Modern Browser Headers**: Support for Sec-CH-UA and other modern headers
- **Mobile Emulation**: Generate mobile browser fingerprints
- **Configurable**: Easy customization via TOML or code
- **Performance**: Highly optimized for high-volume use cases

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
llama-headers-rs = "0.1.0"
```

## Quick Start

```rust
use llama_headers_rs::{get_header, Config};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Simple usage with defaults
    let header = get_header("https://example.com", None)?;
    println!("{}", header);
    
    // Advanced configuration
    let config = Config::new()
        .with_language("de-DE")
        .with_mobile(true)
        .with_referer("https://www.google.de");
    
    let mobile_header = get_header("https://example.de", Some(config))?;
    println!("{}", mobile_header);
    
    Ok(())
}
```

## Example: Integration with HTTP Clients

```rust
use llama_headers_rs::get_header;
use reqwest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate header for target URL
    let url = "https://httpbin.org/headers";
    let header = get_header(url, None)?;
    
    // Create a reqwest client and request
    let client = reqwest::Client::new();
    let mut request_builder = client.get(url);
    
    // Add all headers from our Header instance
    for (key, value) in header.get_map() {
        request_builder = request_builder.header(key, value);
    }
    
    // Send the request
    let response = request_builder.send().await?;
    
    println!("Status: {}", response.status());
    println!("Body: {}", response.text().await?);
    
    Ok(())
}
```

## Browser Support

The library provides realistic headers for:

- **Chrome/Chromium**: Desktop and mobile
- **Firefox**: Desktop and mobile
- **Safari**: Desktop and mobile
- **Edge**: Desktop and mobile

## Configuration

You can customize the header generation with the `Config` struct:

```rust
let config = Config::new()
    .with_language("fr-FR")               // Set language
    .with_mobile(true)                   // Use mobile browser fingerprint
    .with_referer("https://example.com") // Custom referer
    .with_custom_header("X-Custom", "Value"); // Add additional headers
```

## Performance

`llama-headers-rs` is designed to be high-performance. Some benchmark results:

| Operation | Time |
|-----------|------|
| Generate single header | ~50 μs |
| Generate 100 headers | ~5 ms |
| Parse User-Agent | ~10 μs |

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

- The Rust community for their amazing resources and libraries
- All contributors to this project 