# Llama Moonlight Stealth - Code Structure Overview

This document provides an overview of the `llama-moonlight-stealth` crate structure and functionality.

## Crate Structure

```
llama-moonlight-stealth/
├── Cargo.toml              # Crate configuration and dependencies
├── README.md               # Documentation and usage examples
├── examples/               # Example code
│   ├── basic_stealth.rs    # Basic stealth usage example
│   └── proxy_rotation.rs   # Proxy rotation example
└── src/                    # Source code
    ├── lib.rs              # Main library entry point
    ├── evasion.rs          # Evasion techniques module
    ├── client.rs           # High-level client API
    ├── fingerprint.rs      # Browser fingerprinting capabilities
    ├── injection.rs        # JavaScript injection functionality
    ├── intercept.rs        # Request interception capabilities
    ├── proxy.rs            # Proxy configuration and rotation
    ├── detection.rs        # Anti-bot detection testing
    ├── humanize.rs         # Human behavior emulation
    └── timing.rs           # Timing-based stealth operations
```

## Module Descriptions

### `lib.rs`

This is the main entry point for the crate. It defines:
- The core `Error` and `Result` types
- The `StealthConfig` struct for configuring stealth operations
- The `StealthCapabilities` trait for implementing stealth capabilities
- Re-exports of key types for convenience

### `evasion.rs`

Provides the evasion system for bot detection:
- `StealthTarget` trait for targets that can have evasion techniques applied
- `EvasionTechnique` struct for specific techniques to avoid detection
- `EvasionManager` for managing and applying multiple evasion techniques
- Standard evasion techniques like WebDriver hiding, canvas fingerprinting protection, etc.

### `client.rs`

High-level API for stealth browser automation:
- `StealthClient` as the main interface for users
- Support for browser, device, and platform configuration
- Integration with fingerprinting, proxies, and evasion techniques
- The `HumanizationManager` to manage human-like behavior

### `fingerprint.rs`

Browser fingerprinting management:
- Re-exports `BrowserFingerprint` from `llama-moonlight-headers`
- `FingerprintManager` for managing consistent and random fingerprints
- Domain-consistent fingerprinting to maintain the same fingerprint per domain
- Fingerprint protection techniques

### `injection.rs`

JavaScript injection capabilities:
- `Script` struct for JavaScript code to inject
- `ScriptType` enum for different types of scripts (stealth, utility, etc.)
- `InjectionManager` for managing and combining scripts
- Standard stealth scripts for common evasion techniques

### `intercept.rs`

Request and response interception:
- `InterceptPattern` for matching requests by URL, resource type, etc.
- `InterceptRule` for defining interception rules
- `InterceptManager` for managing and applying interception rules
- Standard stealth interception rules for common scenarios

### `proxy.rs`

Proxy management and rotation:
- `ProxyConfig` for defining proxies with protocol, host, auth, etc.
- `ProxyManager` for managing multiple proxies
- `RotationStrategy` for different proxy rotation strategies
- Proxy health monitoring and failure handling

### `detection.rs`

Detection testing for anti-bot systems:
- `DetectionTest` for testing specific anti-bot detection methods
- `DetectionTestSuite` for running multiple tests together
- Standard tests for WebDriver detection, canvas fingerprinting, etc.
- Scoring system to evaluate stealth effectiveness

### `humanize.rs`

Human behavior emulation:
- Re-exports from the client module for convenience
- Reference to the `HumanizationManager` and `HumanizationConfig` types

### `timing.rs`

Timing-based stealth operations:
- `DelayConfig` for configuring delays with different distributions
- `TimingManager` for managing timing operations
- Human-like delays for different action types
- Methods to calculate natural pauses between actions

## Key Features

1. **Multiple Evasion Techniques**: WebDriver hiding, canvas/WebGL fingerprinting protection, etc.
2. **Browser Fingerprinting**: Generate and manage realistic fingerprints for stealth
3. **Proxy Management**: Configure, rotate, and monitor proxies for anonymity
4. **Human Emulation**: Realistic mouse movements, typing patterns, and timing
5. **Request Interception**: Intercept and modify requests for advanced stealth
6. **Script Injection**: Inject JavaScript for custom stealth capabilities
7. **Detection Testing**: Test the effectiveness of stealth techniques

## Usage Examples

The `examples/` directory contains working examples:

- `basic_stealth.rs`: Demonstrates basic stealth features
- `proxy_rotation.rs`: Shows proxy rotation capabilities

## Dependencies

This crate integrates with:

- `llama-moonlight-core`: Core browser automation framework
- `llama-moonlight-headers`: Header and fingerprint generation

## Testing

Each module contains unit tests to ensure functionality works as expected. 