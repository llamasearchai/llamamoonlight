//! Browser fingerprinting management and protection
//!
//! This module provides capabilities to manage browser fingerprints and
//! protection against fingerprinting techniques.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use crate::Result;
use crate::Error;
use crate::evasion::StealthTarget;
use llama_moonlight_headers::{BrowserType, DeviceType, PlatformType};

// Re-export the headers fingerprint for convenience
pub use llama_moonlight_headers::fingerprint::BrowserFingerprint;

/// Manager for browser fingerprinting
#[derive(Debug)]
pub struct FingerprintManager {
    /// Current active fingerprint
    active_fingerprint: Option<BrowserFingerprint>,
    
    /// Whether to use consistent fingerprints
    consistent: bool,
    
    /// Seed for consistent fingerprints
    seed: Option<u64>,
    
    /// JavaScript code cache
    js_cache: HashMap<String, String>,
}

impl FingerprintManager {
    /// Create a new fingerprint manager
    pub fn new() -> Self {
        Self {
            active_fingerprint: None,
            consistent: false,
            seed: None,
            js_cache: HashMap::new(),
        }
    }
    
    /// Create a fingerprint manager with a consistent fingerprint using the provided seed
    pub fn with_seed(seed: u64) -> Self {
        Self {
            active_fingerprint: None,
            consistent: true,
            seed: Some(seed),
            js_cache: HashMap::new(),
        }
    }
    
    /// Get the active fingerprint
    pub fn active_fingerprint(&self) -> Option<&BrowserFingerprint> {
        self.active_fingerprint.as_ref()
    }
    
    /// Generate a new fingerprint based on the target's browser, device, and platform types
    pub fn generate_fingerprint(&mut self, target: &dyn StealthTarget) -> Result<&BrowserFingerprint> {
        let browser_type = target.browser_type();
        let device_type = target.device_type();
        let platform_type = target.platform_type();
        
        let fingerprint = if self.consistent && self.seed.is_some() {
            BrowserFingerprint::consistent(&browser_type, &device_type, &platform_type, self.seed.unwrap())
        } else {
            BrowserFingerprint::new(&browser_type, &device_type, &platform_type)
        };
        
        self.active_fingerprint = Some(fingerprint);
        Ok(self.active_fingerprint.as_ref().unwrap())
    }
    
    /// Set a specific fingerprint
    pub fn set_fingerprint(&mut self, fingerprint: BrowserFingerprint) {
        self.active_fingerprint = Some(fingerprint);
    }
    
    /// Apply the active fingerprint to a browser target
    pub fn apply_fingerprint(&self, target: &mut dyn StealthTarget) -> Result<()> {
        if let Some(fingerprint) = &self.active_fingerprint {
            let js_code = fingerprint.to_js();
            
            // Execute the JavaScript to apply the fingerprint
            target.execute_script(&js_code)?;
            
            // Also set some common headers based on the fingerprint
            target.set_header("User-Agent", &fingerprint.user_agent)?;
            
            // Set Accept-Language based on fingerprint language
            let accept_language = format!("{},en-US;q=0.9,en;q=0.8", fingerprint.language);
            target.set_header("Accept-Language", &accept_language)?;
            
            Ok(())
        } else {
            Err(Error::FingerprintError("No active fingerprint to apply".to_string()))
        }
    }
    
    /// Generate a random fingerprint and apply it to a browser target
    pub fn apply_random_fingerprint(&mut self, target: &mut dyn StealthTarget) -> Result<()> {
        self.generate_fingerprint(target)?;
        self.apply_fingerprint(target)
    }
    
    /// Clear the active fingerprint
    pub fn clear_fingerprint(&mut self) {
        self.active_fingerprint = None;
    }
    
    /// Get or create fingerprint protection JavaScript code
    pub fn get_protection_js(&mut self) -> String {
        let cache_key = "protection";
        if let Some(js) = self.js_cache.get(cache_key) {
            return js.clone();
        }
        
        let js = r#"
        (() => {
            // Protect against canvas fingerprinting
            const originalGetImageData = CanvasRenderingContext2D.prototype.getImageData;
            CanvasRenderingContext2D.prototype.getImageData = function(x, y, width, height) {
                const imageData = originalGetImageData.call(this, x, y, width, height);
                
                // Add subtle noise to the canvas data
                const data = imageData.data;
                for (let i = 0; i < data.length; i += 4) {
                    // Modify only a small percentage of pixels
                    if (Math.random() < 0.01) {
                        data[i] = data[i] ^ 1;     // Red
                        data[i + 1] = data[i + 1] ^ 1; // Green
                        data[i + 2] = data[i + 2] ^ 1; // Blue
                        // Don't modify alpha
                    }
                }
                
                return imageData;
            };
            
            // Protect against WebGL fingerprinting
            const getParameterProxies = {
                WebGLRenderingContext: WebGLRenderingContext.prototype.getParameter,
                WebGL2RenderingContext: WebGL2RenderingContext.prototype.getParameter,
            };
            
            // List of WebGL parameters that can be used for fingerprinting
            const FINGERPRINTING_PARAMS = new Set([
                0x1F01, // VENDOR
                0x1F00, // RENDERER
                0x9245, // UNMASKED_VENDOR_WEBGL
                0x9246, // UNMASKED_RENDERER_WEBGL
            ]);
            
            const overrideGetParameter = (contextType) => {
                const original = getParameterProxies[contextType];
                
                if (!original) return;
                
                contextType.prototype.getParameter = function(parameter) {
                    // Override fingerprinting-related parameters
                    if (FINGERPRINTING_PARAMS.has(parameter)) {
                        switch (parameter) {
                            case 0x1F00: // RENDERER
                            case 0x9246: // UNMASKED_RENDERER_WEBGL
                                return "Intel Iris OpenGL Engine";
                            case 0x1F01: // VENDOR
                            case 0x9245: // UNMASKED_VENDOR_WEBGL
                                return "Intel Inc.";
                            default:
                                break;
                        }
                    }
                    
                    // Use the original for non-fingerprinting parameters
                    return original.call(this, parameter);
                };
            };
            
            overrideGetParameter('WebGLRenderingContext');
            overrideGetParameter('WebGL2RenderingContext');
            
            // Protect against font enumeration
            const originalMeasureText = CanvasRenderingContext2D.prototype.measureText;
            CanvasRenderingContext2D.prototype.measureText = function(text) {
                const result = originalMeasureText.call(this, text);
                
                // If this looks like a font enumeration attempt, add subtle noise
                if (text.length <= 2) {
                    const originalWidth = result.width;
                    // Modify width property dynamically to add slight noise
                    Object.defineProperty(result, 'width', {
                        get: () => originalWidth * (1 + Math.random() * 0.0001)
                    });
                }
                
                return result;
            };
            
            // Protect against AudioContext fingerprinting
            if (typeof AudioContext !== 'undefined') {
                const originalGetChannelData = AudioBuffer.prototype.getChannelData;
                AudioBuffer.prototype.getChannelData = function(channel) {
                    const data = originalGetChannelData.call(this, channel);
                    
                    // Only modify if this is likely a fingerprinting attempt
                    // (very small buffers are often used for fingerprinting)
                    if (this.length < 200) {
                        const noise = 0.0001;
                        const output = new Float32Array(data.length);
                        
                        for (let i = 0; i < data.length; i++) {
                            output[i] = data[i] + (Math.random() * noise - noise/2);
                        }
                        
                        return output;
                    }
                    
                    return data;
                };
            }
        })();
        "#.to_string();
        
        self.js_cache.insert(cache_key.to_string(), js.clone());
        js
    }
    
    /// Apply fingerprint protection to a browser target
    pub fn apply_protection(&mut self, target: &mut dyn StealthTarget) -> Result<()> {
        let js = self.get_protection_js();
        target.execute_script(&js)?;
        Ok(())
    }
}

impl Default for FingerprintManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a fingerprint hash that stays consistent for a given domain
pub fn domain_consistent_hash(domain: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    use std::collections::hash_map::DefaultHasher;
    
    let mut hasher = DefaultHasher::new();
    domain.hash(&mut hasher);
    hasher.finish()
}

/// Generate a fingerprint that stays consistent for a given domain
pub fn domain_consistent_fingerprint(
    domain: &str,
    browser_type: &BrowserType,
    device_type: &DeviceType,
    platform_type: &PlatformType,
) -> BrowserFingerprint {
    let seed = domain_consistent_hash(domain);
    BrowserFingerprint::consistent(browser_type, device_type, platform_type, seed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evasion::StealthTarget;
    
    #[derive(Debug)]
    struct MockStealthTarget {
        browser_type: BrowserType,
        device_type: DeviceType,
        platform_type: PlatformType,
        executed_scripts: Vec<String>,
        headers: HashMap<String, String>,
    }
    
    impl MockStealthTarget {
        fn new() -> Self {
            Self {
                browser_type: BrowserType::Chrome,
                device_type: DeviceType::Desktop,
                platform_type: PlatformType::Windows,
                executed_scripts: Vec::new(),
                headers: HashMap::new(),
            }
        }
    }
    
    impl StealthTarget for MockStealthTarget {
        fn execute_script(&mut self, script: &str) -> Result<String> {
            self.executed_scripts.push(script.to_string());
            Ok("".to_string())
        }
        
        fn browser_type(&self) -> BrowserType {
            self.browser_type.clone()
        }
        
        fn device_type(&self) -> DeviceType {
            self.device_type.clone()
        }
        
        fn platform_type(&self) -> PlatformType {
            self.platform_type.clone()
        }
        
        fn set_header(&mut self, name: &str, value: &str) -> Result<()> {
            self.headers.insert(name.to_string(), value.to_string());
            Ok(())
        }
        
        fn get_header(&self, name: &str) -> Option<String> {
            self.headers.get(name).cloned()
        }
        
        fn remove_header(&mut self, name: &str) -> Result<()> {
            self.headers.remove(name);
            Ok(())
        }
        
        fn intercept_requests(&mut self, _pattern: &str, _handler: crate::evasion::InterceptHandler) -> Result<()> {
            Ok(())
        }
        
        fn set_cookie(&mut self, name: &str, value: &str, _domain: &str) -> Result<()> {
            self.headers.insert(format!("Cookie-{}", name), value.to_string());
            Ok(())
        }
    }
    
    #[test]
    fn test_fingerprint_manager() {
        let mut manager = FingerprintManager::new();
        let mut target = MockStealthTarget::new();
        
        // Test generating a fingerprint
        let fingerprint = manager.generate_fingerprint(&target).unwrap();
        assert!(fingerprint.user_agent.contains("Chrome"));
        
        // Test applying a fingerprint
        manager.apply_fingerprint(&mut target).unwrap();
        assert!(!target.executed_scripts.is_empty());
        assert!(target.headers.contains_key("User-Agent"));
        assert!(target.headers.contains_key("Accept-Language"));
        
        // Test applying protection
        manager.apply_protection(&mut target).unwrap();
        assert!(target.executed_scripts.len() > 1);
        assert!(target.executed_scripts[1].contains("CanvasRenderingContext2D"));
    }
    
    #[test]
    fn test_domain_consistent_fingerprint() {
        // Test that fingerprints are consistent for the same domain
        let fingerprint1 = domain_consistent_fingerprint(
            "example.com",
            &BrowserType::Chrome,
            &DeviceType::Desktop,
            &PlatformType::Windows,
        );
        
        let fingerprint2 = domain_consistent_fingerprint(
            "example.com",
            &BrowserType::Chrome,
            &DeviceType::Desktop,
            &PlatformType::Windows,
        );
        
        let fingerprint3 = domain_consistent_fingerprint(
            "different-domain.com",
            &BrowserType::Chrome,
            &DeviceType::Desktop,
            &PlatformType::Windows,
        );
        
        // Same domain should produce the same fingerprint
        assert_eq!(fingerprint1.canvas_hash, fingerprint2.canvas_hash);
        assert_eq!(fingerprint1.user_agent, fingerprint2.user_agent);
        
        // Different domains should produce different fingerprints
        assert_ne!(fingerprint1.canvas_hash, fingerprint3.canvas_hash);
    }
} 