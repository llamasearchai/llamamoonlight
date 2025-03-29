# 🌙 Llama Moonlight

[![Crates.io](https://img.shields.io/crates/v/llama-moonlight.svg)](https://crates.io/crates/llama-moonlight)
[![Documentation](https://docs.rs/llama-moonlight/badge.svg)](https://docs.rs/llama-moonlight)
[![License](https://img.shields.io/crates/l/llama-moonlight.svg)](https://github.com/yourusername/llama-ecosystem/blob/main/llama-moonlight/LICENSE)
[![Build Status](https://github.com/yourusername/llama-ecosystem/workflows/Rust/badge.svg)](https://github.com/yourusername/llama-ecosystem/actions)

A powerful browser automation framework with MLX and Llama integration, written in Rust.

## Features

- **Multi-browser Support**: Chrome/Chromium, Firefox, and WebKit (Safari)
- **Headless & Headed Modes**: Run browsers in headless mode for automation or headed mode for debugging
- **Modern API**: Async/await-based API for modern Rust applications
- **Network Interception**: Monitor and modify network requests and responses
- **Screenshots & Videos**: Capture screenshots and videos of pages
- **Stealth Mode**: Avoid detection using [llama-headers-rs](https://crates.io/crates/llama-headers-rs) integration
- **AI Integration**: Optional MLX integration for AI-powered automation
- **Command-line Interface**: Easy to use CLI for common tasks

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
llama-moonlight = "0.1.0"
```

For CLI usage, install with:

```bash
cargo install llama-moonlight
```

## Quick Start

### Library Usage

```rust
use llama_moonlight_core::{Moonlight, BrowserType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the framework
    let moonlight = Moonlight::new().await?;
    
    // Launch a browser
    let browser_type = moonlight.browser_type("chromium").unwrap();
    let browser = browser_type.launch().await?;
    
    // Create a new page
    let context = browser.new_context().await?;
    let page = context.new_page().await?;
    
    // Navigate to a URL
    page.goto("https://example.com").await?;
    
    // Take a screenshot
    page.screenshot("example.png").await?;
    
    // Get page content
    let content = page.content().await?;
    println!("Page content: {}", content);
    
    // Find and click an element
    page.click("a").await?;
    
    // Close the browser
    browser.close().await?;
    
    Ok(())
}
```

### CLI Usage

Take a screenshot:

```bash
llama-moonlight screenshot https://example.com --output example.png
```

Get page content:

```bash
llama-moonlight content https://example.com
```

Evaluate JavaScript:

```bash
llama-moonlight evaluate https://example.com --script "document.title"
```

Click an element:

```bash
llama-moonlight click https://example.com --selector "a.button"
```

Fill out a form field:

```bash
llama-moonlight fill https://example.com --selector "input#username" --text "user123"
```

## Advanced Features

### Stealth Mode

Avoid detection by using stealth mode, which integrates with llama-headers-rs:

```rust
// Enable stealth mode when launching the browser
let options = BrowserOptions {
    stealth: Some(true),
    ..Default::default()
};

let browser = browser_type.launch_with_options(options).await?;

// Or use the stealth_context method
let context = browser.stealth_context("https://example.com").await?;
```

### AI Integration

With the MLX feature enabled, you can use AI for automating tasks:

```rust
// Enable the MLX feature in Cargo.toml
// llama-moonlight = { version = "0.1.0", features = ["mlx"] }

use llama_moonlight::LlamaModel;

// Load a model
let model = LlamaModel::load("path/to/model").await?;

// Use the model for tasks
let decision = model.predict("What action should I take?").await?;
```

## Architecture

Llama Moonlight is organized as a workspace with multiple crates:

- **llama-moonlight**: Main crate and CLI
- **llama-moonlight-core**: Core browser automation functionality
- **llama-moonlight-cli**: Command-line interface
- **llama-moonlight-pool**: Browser pool for parallel automation
- **llama-moonlight-rxt**: Reactive extensions for browser events
- **llama-moonlight-testutil**: Testing utilities
- **llama-moonlight-mlx**: MLX integration for AI capabilities

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

- The Rust community for their amazing resources and libraries
- The Playwright team for inspiration
- All contributors to this project 