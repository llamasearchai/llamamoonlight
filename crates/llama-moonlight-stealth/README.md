# Llama Moonlight Stealth

Advanced stealth capabilities for web automation using the Llama Moonlight framework. This crate integrates stealth techniques to bypass anti-bot systems and provide realistic browser behavior.

## Features

- **Anti-Detection Technology**: Avoid bot detection systems with advanced fingerprint spoofing
- **Headers and Fingerprinting**: Generate and manage realistic browser headers and fingerprints
- **Proxies and IP Rotation**: Support for multiple proxy protocols and IP rotation strategies
- **Human Behavior Emulation**: Simulate human-like mouse movements, typing patterns, and browsing behavior
- **Seamless Integration**: Works with the Llama Moonlight browser automation framework
- **Advanced Interception**: Intercept and modify requests to evade fingerprinting

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
llama-moonlight-stealth = "0.1.0"
```

## Example

```rust
use llama_moonlight_core::browser::Browser;
use llama_moonlight_stealth::{StealthClient, ProxyConfig, ProxyProtocol};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a stealth client
    let mut stealth_client = StealthClient::new()
        .with_proxy(
            ProxyConfig::new(ProxyProtocol::Http, "proxy.example.com", 8080)
                .with_auth("username", "password")
        );
    
    // Launch a browser with stealth capabilities
    let browser = Browser::launch().await?;
    let mut context = browser.new_context().await?;
    
    // Apply stealth techniques to the browser context
    stealth_client.apply_stealth(&mut context)?;
    
    // Navigate to a URL with stealth headers
    let page = context.new_page().await?;
    page.navigate("https://bot-detection-test.example.com").await?;
    
    // Emulate human-like interaction
    page.wait_for_selector("input[type=text]").await?;
    
    // Record a successful visit
    stealth_client.record_proxy_success(Some(250)); // Response time in ms
    
    Ok(())
}
```

## Stealth Techniques

Llama Moonlight Stealth implements a variety of anti-detection techniques:

### WebDriver Detection Evasion
- Hides `navigator.webdriver` property
- Removes automation flags
- Patches automation-related JavaScript properties

### Fingerprinting Protection
- Canvas fingerprinting protection
- WebGL fingerprinting protection
- Audio fingerprinting protection
- Font enumeration protection

### Browser Identity Management
- Realistic header generation
- Consistent browser fingerprinting
- Cookie and storage management

### Network Management
- Multiple proxy protocols (HTTP, HTTPS, SOCKS4, SOCKS5)
- Proxy rotation strategies
- IP rotation and management

### Human Behavior Simulation
- Realistic mouse movement patterns
- Human-like typing with occasional typos and corrections
- Natural timing and delays between actions

## Configuration

The `StealthClient` can be customized with various options:

```rust
use llama_moonlight_stealth::{StealthClient, StealthConfig, BrowserType, DeviceType, PlatformType};

let config = StealthConfig {
    stealth_enabled: true,
    random_fingerprints: false,
    random_user_agents: false,
    emulate_human: true,
    use_proxies: true,
    intercept_webgl: true,
    intercept_canvas: true,
    intercept_fonts: true,
    hide_automation: true,
    custom_headers: std::collections::HashMap::new(),
};

let client = StealthClient::with_config(config)
    .with_browser(BrowserType::Firefox)
    .with_device(DeviceType::Desktop)
    .with_platform(PlatformType::MacOS);
```

## Advanced Usage

### Domain-Consistent Fingerprinting

Generate fingerprints that remain consistent for specific domains:

```rust
use llama_moonlight_stealth::fingerprint;

let domain = "example.com";
let browser_type = BrowserType::Chrome;
let device_type = DeviceType::Desktop;
let platform_type = PlatformType::Windows;

// Generate a fingerprint that will be consistent for the domain
let fp = fingerprint::domain_consistent_fingerprint(
    domain,
    &browser_type,
    &device_type,
    &platform_type
);
```

### Proxy Rotation

Implement advanced proxy rotation strategies:

```rust
use llama_moonlight_stealth::proxy::{ProxyManager, RotationStrategy};

let mut proxy_manager = ProxyManager::with_strategy(RotationStrategy::LeastRecentlyUsed)
    .with_max_failures(5)
    .with_min_success_rate(0.7);
    
proxy_manager.add_proxies_from_urls(&[
    "http://user:pass@proxy1.example.com:8080",
    "socks5://user:pass@proxy2.example.com:1080",
    "https://user:pass@proxy3.example.com:443",
])?;

// Get the next proxy
let proxy = proxy_manager.rotate();
```

## License

This project is licensed under either:

- MIT License
- Apache License, Version 2.0

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. 