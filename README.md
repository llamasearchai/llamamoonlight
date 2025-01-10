# 🦙 Llama Ecosystem

A comprehensive suite of Rust crates for browser automation, web scraping, and AI integration.

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![llama-headers-rs](https://img.shields.io/badge/llama--headers--rs-passing-brightgreen.svg)](llama-headers-rs)
[![llama-moonlight](https://img.shields.io/badge/llama--moonlight-passing-brightgreen.svg)](llama-moonlight)

## 🌟 Overview

The Llama Ecosystem is a collection of modular, interoperable Rust crates designed for sophisticated web automation, scraping, and AI integration. Each crate focuses on solving a specific problem while working seamlessly with the others.

## 🧩 Crates

### 🔍 [llama-headers-rs](llama-headers-rs)

A sophisticated HTTP header generation library for realistic browser emulation.

- Realistic browser fingerprinting
- User-agent generation
- Language and locale-aware headers
- Referer generation
- Modern security-related headers

```rust
use llama_headers_rs::{get_header, Config};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate realistic browser headers
    let header = get_header("https://example.com", None)?;
    println!("{}", header);
    
    Ok(())
}
```

### 🌙 [llama-moonlight](llama-moonlight)

A powerful browser automation framework with MLX and Llama integration.

- Multi-browser support (Chrome, Firefox, Safari)
- Headless & headed modes
- Network interception
- Screenshots & videos
- Stealth mode via llama-headers-rs integration
- AI integration

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
    
    Ok(())
}
```

### 🛡️ llama-cloudflare *(Coming Soon)*

Tools for bypassing Cloudflare and other anti-bot protections.

- TLS fingerprint customization
- JavaScript challenge solver
- Proxy rotation
- Rate limiting and request pacing

### 🤖 llama-mlx *(Coming Soon)*

MLX integration for AI-powered automation.

- MLX model loading and inference
- Computer vision for visual analysis
- Text recognition and processing
- Decision making for automation scenarios

### 🧠 llama-agents *(Coming Soon)*

AI agent abstractions for automation tasks.

- Autonomous web agent framework
- Task planning and execution
- LLM integration
- Memory and context management

## 🧪 Running the Tests

The ecosystem includes comprehensive test scripts for each crate and the entire ecosystem.

Test a specific crate:
```bash
cd llama-headers-rs
./test-and-publish.sh
```

Test the entire ecosystem:
```bash
./scripts/master-test.sh
```

## 🚀 Getting Started

1. Clone the repository:
```bash
git clone https://github.com/yourusername/llama-ecosystem.git
cd llama-ecosystem
```

2. Build the crates:
```bash
cargo build --all
```

3. Run the examples:
```bash
# For llama-headers-rs
cargo run --example simple --package llama-headers-rs

# For llama-moonlight CLI
cargo run --package llama-moonlight -- screenshot https://example.com --output example.png
```

## 📚 Documentation

Each crate includes comprehensive documentation. Generate the documentation with:

```bash
cargo doc --no-deps --open
```

## 🔧 Use Cases

- **Web Scraping**: Extract data from websites while avoiding bot detection
- **Browser Automation**: Automate UI testing and web interactions
- **Data Collection**: Gather information across multiple sites
- **Security Testing**: Test web applications for security vulnerabilities
- **AI-Powered Automation**: Combine browser automation with AI for intelligent decision-making

## 👥 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option. 
# Updated in commit 1 - 2025-04-04 17:22:51

# Updated in commit 9 - 2025-04-04 17:22:52

# Updated in commit 17 - 2025-04-04 17:22:53

# Updated in commit 25 - 2025-04-04 17:22:54

# Updated in commit 1 - 2025-04-05 14:31:02

# Updated in commit 9 - 2025-04-05 14:31:02

# Updated in commit 17 - 2025-04-05 14:31:02

# Updated in commit 25 - 2025-04-05 14:31:02

# Updated in commit 1 - 2025-04-05 15:17:29

# Updated in commit 9 - 2025-04-05 15:17:29

# Updated in commit 17 - 2025-04-05 15:17:29

# Updated in commit 25 - 2025-04-05 15:17:30

# Updated in commit 1 - 2025-04-05 15:48:15

# Updated in commit 9 - 2025-04-05 15:48:16

# Updated in commit 17 - 2025-04-05 15:48:16

# Updated in commit 25 - 2025-04-05 15:48:16

# Updated in commit 1 - 2025-04-05 16:53:28

# Updated in commit 9 - 2025-04-05 16:53:28
