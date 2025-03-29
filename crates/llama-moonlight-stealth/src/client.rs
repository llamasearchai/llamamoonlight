//! Stealth client for browser automation
//!
//! This module provides a high-level client for stealth browser automation
//! that integrates the various stealth components.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use lazy_static::lazy_static;
use log::{debug, info, warn, error};

use crate::Result;
use crate::Error;
use crate::StealthConfig;
use crate::StealthCapabilities;
use crate::evasion::{EvasionManager, StealthTarget, InterceptHandler, InterceptedRequest};
use crate::fingerprint::{FingerprintManager, BrowserFingerprint};
use crate::proxy::{ProxyManager, ProxyConfig};
use crate::humanize::HumanizationManager;
use llama_moonlight_headers::{BrowserType, DeviceType, PlatformType, HeaderGenerator};

/// Client for stealth browser automation
#[derive(Debug)]
pub struct StealthClient {
    /// Configuration for stealth operations
    config: StealthConfig,
    
    /// Browser type
    browser_type: BrowserType,
    
    /// Device type
    device_type: DeviceType,
    
    /// Platform type
    platform_type: PlatformType,
    
    /// Evasion manager
    evasion_manager: EvasionManager,
    
    /// Fingerprint manager
    fingerprint_manager: FingerprintManager,
    
    /// Proxy manager
    proxy_manager: Option<ProxyManager>,
    
    /// Header generator
    header_generator: HeaderGenerator,
    
    /// Humanization manager
    humanization_manager: HumanizationManager,
    
    /// Whether stealth has been applied
    stealth_applied: bool,
    
    /// Domains that have been visited
    visited_domains: HashMap<String, u32>,
}

impl StealthClient {
    /// Create a new stealth client with default configuration
    pub fn new() -> Self {
        Self::with_config(StealthConfig::default())
    }
    
    /// Create a new stealth client with the specified configuration
    pub fn with_config(config: StealthConfig) -> Self {
        let browser_type = BrowserType::Chrome;
        let device_type = DeviceType::Desktop;
        let platform_type = PlatformType::Windows;
        
        let evasion_manager = EvasionManager::standard_evasions();
        let fingerprint_manager = FingerprintManager::new();
        let header_generator = HeaderGenerator::new(browser_type.clone())
            .with_device(device_type.clone())
            .with_platform(platform_type.clone())
            .with_stealth(config.stealth_enabled);
        
        Self {
            config,
            browser_type,
            device_type,
            platform_type,
            evasion_manager,
            fingerprint_manager,
            proxy_manager: None,
            header_generator,
            humanization_manager: HumanizationManager::new(),
            stealth_applied: false,
            visited_domains: HashMap::new(),
        }
    }
    
    /// Set the browser type
    pub fn with_browser(mut self, browser_type: BrowserType) -> Self {
        self.browser_type = browser_type.clone();
        self.header_generator = self.header_generator.with_browser(browser_type);
        self
    }
    
    /// Set the device type
    pub fn with_device(mut self, device_type: DeviceType) -> Self {
        self.device_type = device_type.clone();
        self.header_generator = self.header_generator.with_device(device_type);
        self
    }
    
    /// Set the platform type
    pub fn with_platform(mut self, platform_type: PlatformType) -> Self {
        self.platform_type = platform_type.clone();
        self.header_generator = self.header_generator.with_platform(platform_type);
        self
    }
    
    /// Set a custom user agent
    pub fn with_user_agent(mut self, user_agent: &str) -> Self {
        self.header_generator = self.header_generator.with_user_agent(user_agent);
        self
    }
    
    /// Enable proxy support with the provided proxy configuration
    pub fn with_proxy(mut self, proxy: ProxyConfig) -> Self {
        let mut proxy_manager = ProxyManager::new();
        proxy_manager.add_proxy(proxy);
        self.proxy_manager = Some(proxy_manager);
        self
    }
    
    /// Enable proxy support with the provided proxy manager
    pub fn with_proxy_manager(mut self, proxy_manager: ProxyManager) -> Self {
        self.proxy_manager = Some(proxy_manager);
        self
    }
    
    /// Apply stealth techniques to the target
    pub fn apply_stealth<T: StealthTarget + StealthCapabilities>(&mut self, target: &mut T) -> Result<()> {
        if self.stealth_applied {
            debug!("Stealth already applied, skipping");
            return Ok(());
        }
        
        info!("Applying stealth techniques");
        
        // Apply fingerprinting
        if self.config.random_fingerprints {
            debug!("Applying random fingerprint");
            self.fingerprint_manager.apply_random_fingerprint(target)?;
        } else {
            debug!("Applying consistent fingerprint");
            let fingerprint = self.fingerprint_manager.generate_fingerprint(target)?;
            target.set_fingerprint(fingerprint)?;
        }
        
        // Apply proxy if enabled
        if self.config.use_proxies && self.proxy_manager.is_some() {
            debug!("Applying proxy");
            if let Some(proxy) = self.proxy_manager.as_mut().and_then(|pm| pm.active_proxy()) {
                target.set_proxy(proxy)?;
            }
        }
        
        // Apply evasion techniques
        debug!("Applying evasion techniques");
        self.evasion_manager.apply_all(target)?;
        
        // Apply interception if needed
        if self.config.intercept_canvas || self.config.intercept_webgl || self.config.intercept_fonts {
            debug!("Setting up fingerprinting interception");
            self.setup_interception(target)?;
        }
        
        // Apply human-like behavior if enabled
        if self.config.emulate_human {
            debug!("Setting up human-like behavior");
            target.emulate_human()?;
        }
        
        // Apply automation hiding if enabled
        if self.config.hide_automation {
            debug!("Hiding automation markers");
            target.hide_automation_markers()?;
        }
        
        self.stealth_applied = true;
        info!("Stealth techniques applied successfully");
        
        Ok(())
    }
    
    /// Generate stealth headers for a URL
    pub fn generate_headers(&self, url: &str) -> HashMap<String, String> {
        debug!("Generating stealth headers for URL: {}", url);
        self.header_generator.generate(url)
    }
    
    /// Setup fingerprinting interception
    fn setup_interception<T: StealthTarget + StealthCapabilities>(&self, target: &mut T) -> Result<()> {
        // Canvas interception
        if self.config.intercept_canvas {
            debug!("Setting up canvas interception");
            let handler = Arc::new(|req: &mut InterceptedRequest| {
                if req.url.contains("canvas") || req.url.contains("html2canvas") {
                    debug!("Intercepted canvas request: {}", req.url);
                    // Add headers to make it look more legitimate
                    if let Some(headers) = &mut req.new_headers {
                        headers.insert("Accept".to_string(), "image/webp,*/*".to_string());
                    }
                }
                Ok(())
            });
            
            target.intercept_requests("**/canvas/**", handler)?;
        }
        
        // WebGL interception
        if self.config.intercept_webgl {
            debug!("Setting up WebGL interception");
            let handler = Arc::new(|req: &mut InterceptedRequest| {
                if req.url.contains("webgl") || req.url.contains("gl") {
                    debug!("Intercepted WebGL request: {}", req.url);
                    // Modify headers for WebGL
                    if let Some(headers) = &mut req.new_headers {
                        headers.insert("Accept".to_string(), "application/json".to_string());
                    }
                }
                Ok(())
            });
            
            target.intercept_requests("**/gl/**", handler)?;
        }
        
        // Font enumeration interception
        if self.config.intercept_fonts {
            debug!("Setting up font enumeration interception");
            let handler = Arc::new(|req: &mut InterceptedRequest| {
                if req.url.contains("fonts") || req.url.contains("font") {
                    debug!("Intercepted font request: {}", req.url);
                    // Don't block legitimate font requests, just modify them
                    if let Some(headers) = &mut req.new_headers {
                        headers.insert("Accept".to_string(), "font/woff2,font/woff,*/*".to_string());
                    }
                }
                Ok(())
            });
            
            target.intercept_requests("**/fonts/**", handler)?;
        }
        
        Ok(())
    }
    
    /// Record a visit to a domain
    pub fn record_visit(&mut self, url: &str) -> Result<()> {
        let domain = extract_domain(url)?;
        let count = self.visited_domains.entry(domain.clone()).or_insert(0);
        *count += 1;
        
        debug!("Recorded visit to domain: {} (count: {})", domain, count);
        
        Ok(())
    }
    
    /// Get the number of visits to a domain
    pub fn visit_count(&self, url: &str) -> Result<u32> {
        let domain = extract_domain(url)?;
        Ok(*self.visited_domains.get(&domain).unwrap_or(&0))
    }
    
    /// Rotate the proxy if proxy support is enabled
    pub fn rotate_proxy(&mut self) -> Option<&ProxyConfig> {
        if let Some(proxy_manager) = &mut self.proxy_manager {
            debug!("Rotating proxy");
            proxy_manager.rotate()
        } else {
            None
        }
    }
    
    /// Record a successful request with the current proxy
    pub fn record_proxy_success(&mut self, response_time_ms: Option<u64>) {
        if let Some(proxy_manager) = &mut self.proxy_manager {
            proxy_manager.record_success(response_time_ms);
        }
    }
    
    /// Record a failed request with the current proxy
    pub fn record_proxy_failure(&mut self) {
        if let Some(proxy_manager) = &mut self.proxy_manager {
            proxy_manager.record_failure();
        }
    }
    
    /// Get the current active proxy
    pub fn active_proxy(&self) -> Option<&ProxyConfig> {
        self.proxy_manager.as_ref().and_then(|pm| pm.active_proxy())
    }
    
    /// Get the evasion manager
    pub fn evasion_manager(&self) -> &EvasionManager {
        &self.evasion_manager
    }
    
    /// Get a mutable reference to the evasion manager
    pub fn evasion_manager_mut(&mut self) -> &mut EvasionManager {
        &mut self.evasion_manager
    }
    
    /// Get the fingerprint manager
    pub fn fingerprint_manager(&self) -> &FingerprintManager {
        &self.fingerprint_manager
    }
    
    /// Get a mutable reference to the fingerprint manager
    pub fn fingerprint_manager_mut(&mut self) -> &mut FingerprintManager {
        &mut self.fingerprint_manager
    }
    
    /// Get the browser type
    pub fn browser_type(&self) -> &BrowserType {
        &self.browser_type
    }
    
    /// Get the device type
    pub fn device_type(&self) -> &DeviceType {
        &self.device_type
    }
    
    /// Get the platform type
    pub fn platform_type(&self) -> &PlatformType {
        &self.platform_type
    }
}

impl Default for StealthClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract the domain from a URL
fn extract_domain(url: &str) -> Result<String> {
    let url = url::Url::parse(url).map_err(|e| Error::Other(format!("Failed to parse URL: {}", e)))?;
    let domain = url.host_str().ok_or_else(|| Error::Other("URL has no host".to_string()))?.to_string();
    Ok(domain)
}

/// Humanization manager for emulating human behavior
#[derive(Debug, Default)]
pub struct HumanizationManager {
    /// Configuration for humanization
    config: HumanizationConfig,
}

/// Configuration for humanization
#[derive(Debug, Clone)]
pub struct HumanizationConfig {
    /// Minimum delay between actions (milliseconds)
    pub min_delay_ms: u64,
    
    /// Maximum delay between actions (milliseconds)
    pub max_delay_ms: u64,
    
    /// Mouse move jitter factor (0.0 - 1.0)
    pub mouse_jitter: f64,
    
    /// Typing speed (characters per second)
    pub typing_speed: f64,
    
    /// Typing mistake probability (0.0 - 1.0)
    pub typing_mistake_prob: f64,
    
    /// Whether to enable random scrolling
    pub random_scrolling: bool,
    
    /// Probability of correcting a typing mistake (0.0 - 1.0)
    pub correction_prob: f64,
}

impl Default for HumanizationConfig {
    fn default() -> Self {
        Self {
            min_delay_ms: 500,
            max_delay_ms: 3000,
            mouse_jitter: 0.1,
            typing_speed: 8.0,
            typing_mistake_prob: 0.03,
            random_scrolling: true,
            correction_prob: 0.9,
        }
    }
}

impl HumanizationManager {
    /// Create a new humanization manager
    pub fn new() -> Self {
        Self {
            config: HumanizationConfig::default(),
        }
    }
    
    /// Create a new humanization manager with the specified configuration
    pub fn with_config(config: HumanizationConfig) -> Self {
        Self {
            config,
        }
    }
    
    /// Get a random delay between actions
    pub fn random_delay(&self) -> Duration {
        let range = self.config.max_delay_ms - self.config.min_delay_ms;
        let random_ms = rand::random::<u64>() % range;
        Duration::from_millis(self.config.min_delay_ms + random_ms)
    }
    
    /// Generate a human-like mouse movement path
    pub fn mouse_path(&self, start_x: f64, start_y: f64, end_x: f64, end_y: f64, steps: usize) -> Vec<(f64, f64)> {
        let mut path = Vec::with_capacity(steps);
        
        // Control points for Bezier curve
        let cp1_x = start_x + (end_x - start_x) / 3.0 + (rand::random::<f64>() - 0.5) * self.config.mouse_jitter * 100.0;
        let cp1_y = start_y + (end_y - start_y) / 3.0 + (rand::random::<f64>() - 0.5) * self.config.mouse_jitter * 100.0;
        
        let cp2_x = start_x + 2.0 * (end_x - start_x) / 3.0 + (rand::random::<f64>() - 0.5) * self.config.mouse_jitter * 100.0;
        let cp2_y = start_y + 2.0 * (end_y - start_y) / 3.0 + (rand::random::<f64>() - 0.5) * self.config.mouse_jitter * 100.0;
        
        for i in 0..steps {
            let t = i as f64 / (steps - 1) as f64;
            
            // Cubic Bezier formula
            let x = (1.0 - t).powi(3) * start_x +
                   3.0 * (1.0 - t).powi(2) * t * cp1_x +
                   3.0 * (1.0 - t) * t.powi(2) * cp2_x +
                   t.powi(3) * end_x;
            
            let y = (1.0 - t).powi(3) * start_y +
                   3.0 * (1.0 - t).powi(2) * t * cp1_y +
                   3.0 * (1.0 - t) * t.powi(2) * cp2_y +
                   t.powi(3) * end_y;
            
            // Add jitter
            let jitter_x = (rand::random::<f64>() - 0.5) * self.config.mouse_jitter * 5.0;
            let jitter_y = (rand::random::<f64>() - 0.5) * self.config.mouse_jitter * 5.0;
            
            path.push((x + jitter_x, y + jitter_y));
        }
        
        path
    }
    
    /// Calculate typing delays based on characters
    pub fn typing_delays(&self, text: &str) -> Vec<Duration> {
        let chars = text.chars().collect::<Vec<_>>();
        let mut delays = Vec::with_capacity(chars.len());
        
        for (i, c) in chars.iter().enumerate() {
            // Base delay from typing speed
            let base_delay_ms = (1000.0 / self.config.typing_speed) as u64;
            
            // Add variance based on character
            let mut delay = match c {
                // Space and punctuation usually take longer
                ' ' | '.' | ',' | '!' | '?' | ';' | ':' => base_delay_ms + rand::random::<u64>() % 300,
                
                // Shift characters might take longer
                'A'..='Z' | '@' | '#' | '$' | '%' | '^' | '&' | '*' | '(' | ')' | '_' | '+' | '{' | '}' | '|' | '"' | '<' | '>' | '?' => 
                    base_delay_ms + rand::random::<u64>() % 200,
                
                // Numbers and letters are faster
                'a'..='z' | '0'..='9' => 
                    base_delay_ms + rand::random::<u64>() % 100,
                
                // Default
                _ => base_delay_ms + rand::random::<u64>() % 150,
            };
            
            // If the previous character is on the opposite side of the keyboard, add a bit more delay
            if i > 0 {
                if is_opposite_side_of_keyboard(chars[i-1], *c) {
                    delay += rand::random::<u64>() % 50 + 50;
                }
            }
            
            delays.push(Duration::from_millis(delay));
        }
        
        delays
    }
    
    /// Humanize text input with potential typos and corrections
    pub fn humanize_text(&self, text: &str) -> (String, Vec<Duration>) {
        let mut result = String::new();
        let mut delays = Vec::new();
        
        for c in text.chars() {
            // Chance of making a typo
            if rand::random::<f64>() < self.config.typing_mistake_prob {
                // Add a typo
                let typo = generate_typo(c);
                result.push(typo);
                delays.push(self.random_delay());
                
                // Chance of correcting the typo
                if rand::random::<f64>() < self.config.correction_prob {
                    // Add a backspace
                    result.push('\u{0008}'); // Backspace character
                    delays.push(Duration::from_millis(300));
                    
                    // Add the correct character
                    result.push(c);
                    delays.push(self.random_delay());
                }
            } else {
                // Add the correct character
                result.push(c);
                delays.push(self.random_delay());
            }
        }
        
        (result, delays)
    }
}

/// Check if two characters are on opposite sides of the keyboard
fn is_opposite_side_of_keyboard(a: char, b: char) -> bool {
    lazy_static! {
        static ref LEFT_SIDE: HashMap<char, bool> = {
            let mut m = HashMap::new();
            for c in "qwertasdfgzxcvb12345".chars() {
                m.insert(c, true);
            }
            m
        };
        
        static ref RIGHT_SIDE: HashMap<char, bool> = {
            let mut m = HashMap::new();
            for c in "yuiophjklnm67890".chars() {
                m.insert(c, true);
            }
            m
        };
    }
    
    let a_lower = a.to_ascii_lowercase();
    let b_lower = b.to_ascii_lowercase();
    
    (LEFT_SIDE.contains_key(&a_lower) && RIGHT_SIDE.contains_key(&b_lower)) ||
    (RIGHT_SIDE.contains_key(&a_lower) && LEFT_SIDE.contains_key(&b_lower))
}

/// Generate a typo for a character based on QWERTY keyboard layout
fn generate_typo(c: char) -> char {
    lazy_static! {
        static ref ADJACENT_KEYS: HashMap<char, Vec<char>> = {
            let mut m = HashMap::new();
            m.insert('q', vec!['w', 'a', '1']);
            m.insert('w', vec!['q', 'e', 's', 'a', '2']);
            m.insert('e', vec!['w', 'r', 'd', 's', '3']);
            m.insert('r', vec!['e', 't', 'f', 'd', '4']);
            m.insert('t', vec!['r', 'y', 'g', 'f', '5']);
            m.insert('y', vec!['t', 'u', 'h', 'g', '6']);
            m.insert('u', vec!['y', 'i', 'j', 'h', '7']);
            m.insert('i', vec!['u', 'o', 'k', 'j', '8']);
            m.insert('o', vec!['i', 'p', 'l', 'k', '9']);
            m.insert('p', vec!['o', '[', ';', 'l', '0']);
            m.insert('a', vec!['q', 'w', 's', 'z']);
            m.insert('s', vec!['a', 'w', 'e', 'd', 'x', 'z']);
            m.insert('d', vec!['s', 'e', 'r', 'f', 'c', 'x']);
            m.insert('f', vec!['d', 'r', 't', 'g', 'v', 'c']);
            m.insert('g', vec!['f', 't', 'y', 'h', 'b', 'v']);
            m.insert('h', vec!['g', 'y', 'u', 'j', 'n', 'b']);
            m.insert('j', vec!['h', 'u', 'i', 'k', 'm', 'n']);
            m.insert('k', vec!['j', 'i', 'o', 'l', ',', 'm']);
            m.insert('l', vec!['k', 'o', 'p', ';', '.', ',']);
            m.insert('z', vec!['a', 's', 'x']);
            m.insert('x', vec!['z', 's', 'd', 'c']);
            m.insert('c', vec!['x', 'd', 'f', 'v']);
            m.insert('v', vec!['c', 'f', 'g', 'b']);
            m.insert('b', vec!['v', 'g', 'h', 'n']);
            m.insert('n', vec!['b', 'h', 'j', 'm']);
            m.insert('m', vec!['n', 'j', 'k', ',']);
            m
        };
    }
    
    let lower_c = c.to_ascii_lowercase();
    let was_upper = c.is_uppercase();
    
    if let Some(adjacent) = ADJACENT_KEYS.get(&lower_c) {
        let typo_index = rand::random::<usize>() % adjacent.len();
        let typo_char = adjacent[typo_index];
        
        if was_upper {
            typo_char.to_ascii_uppercase()
        } else {
            typo_char
        }
    } else {
        // If we don't have adjacent keys for this character, just return the original
        c
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evasion::StealthTarget;
    
    #[derive(Debug)]
    struct MockStealthCapabilities {
        fingerprint_applied: bool,
        headers_applied: bool,
        proxy_applied: bool,
        human_applied: bool,
        automation_hidden: bool,
        interceptors: Vec<String>,
    }
    
    #[derive(Debug)]
    struct MockStealthTarget {
        browser_type: BrowserType,
        device_type: DeviceType,
        platform_type: PlatformType,
        executed_scripts: Vec<String>,
        headers: HashMap<String, String>,
        capabilities: MockStealthCapabilities,
    }
    
    impl MockStealthTarget {
        fn new() -> Self {
            Self {
                browser_type: BrowserType::Chrome,
                device_type: DeviceType::Desktop,
                platform_type: PlatformType::Windows,
                executed_scripts: Vec::new(),
                headers: HashMap::new(),
                capabilities: MockStealthCapabilities {
                    fingerprint_applied: false,
                    headers_applied: false,
                    proxy_applied: false,
                    human_applied: false,
                    automation_hidden: false,
                    interceptors: Vec::new(),
                },
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
        
        fn intercept_requests(&mut self, pattern: &str, _handler: crate::evasion::InterceptHandler) -> Result<()> {
            self.capabilities.interceptors.push(pattern.to_string());
            Ok(())
        }
        
        fn set_cookie(&mut self, name: &str, value: &str, _domain: &str) -> Result<()> {
            self.headers.insert(format!("Cookie-{}", name), value.to_string());
            Ok(())
        }
    }
    
    impl StealthCapabilities for MockStealthTarget {
        fn apply_stealth(&mut self) -> Result<()> {
            Ok(())
        }
        
        fn set_fingerprint(&mut self, _fingerprint: &BrowserFingerprint) -> Result<()> {
            self.capabilities.fingerprint_applied = true;
            Ok(())
        }
        
        fn set_headers(&mut self, headers: std::collections::HashMap<String, String>) -> Result<()> {
            self.headers = headers;
            self.capabilities.headers_applied = true;
            Ok(())
        }
        
        fn set_proxy(&mut self, _proxy: &ProxyConfig) -> Result<()> {
            self.capabilities.proxy_applied = true;
            Ok(())
        }
        
        fn emulate_human(&mut self) -> Result<()> {
            self.capabilities.human_applied = true;
            Ok(())
        }
        
        fn hide_automation_markers(&mut self) -> Result<()> {
            self.capabilities.automation_hidden = true;
            Ok(())
        }
    }
    
    #[test]
    fn test_stealth_client() {
        let mut client = StealthClient::new();
        let mut target = MockStealthTarget::new();
        
        // Apply stealth
        client.apply_stealth(&mut target).unwrap();
        
        // Check what was applied
        assert!(target.capabilities.fingerprint_applied);
        assert!(target.capabilities.automation_hidden);
        assert!(!target.executed_scripts.is_empty());
        
        // Check that interception was set up
        assert!(target.capabilities.interceptors.contains(&"**/canvas/**".to_string()));
        assert!(target.capabilities.interceptors.contains(&"**/gl/**".to_string()));
        assert!(target.capabilities.interceptors.contains(&"**/fonts/**".to_string()));
    }
    
    #[test]
    fn test_extract_domain() {
        assert_eq!(extract_domain("https://www.example.com/path").unwrap(), "www.example.com");
        assert_eq!(extract_domain("http://sub.example.org:8080/").unwrap(), "sub.example.org");
    }
    
    #[test]
    fn test_humanization() {
        let manager = HumanizationManager::new();
        
        // Test typing delays
        let delays = manager.typing_delays("Hello, world!");
        assert_eq!(delays.len(), 13);
        
        // Test mouse path
        let path = manager.mouse_path(0.0, 0.0, 100.0, 100.0, 10);
        assert_eq!(path.len(), 10);
        
        // Test humanize text
        let (text, delays) = manager.humanize_text("test");
        assert!(text.len() >= 4); // May be longer if it includes typos and corrections
        assert_eq!(delays.len(), text.len());
    }
} 