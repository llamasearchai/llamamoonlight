use std::collections::HashMap;
use rand::prelude::*;
use crate::{BrowserType, DeviceType};

/// Add stealth mode headers to avoid bot detection
pub fn add_stealth_headers(
    headers: &mut HashMap<String, String>,
    url: &str,
    browser_type: &BrowserType,
    device_type: &DeviceType,
) {
    // Additional bot detection avoidance headers
    add_cache_headers(headers);
    add_referrer_policy_headers(headers);
    add_do_not_track_header(headers);
    add_random_client_hints(headers, device_type);
    add_sec_fetch_headers(headers, url);
    
    // Browser-specific headers
    match browser_type {
        BrowserType::Chrome => add_chrome_stealth_headers(headers),
        BrowserType::Firefox => add_firefox_stealth_headers(headers),
        BrowserType::Safari => add_safari_stealth_headers(headers),
        BrowserType::Edge => add_edge_stealth_headers(headers),
        BrowserType::Opera => add_opera_stealth_headers(headers),
        BrowserType::Custom(_) => {},
    }
}

/// Add cache-related headers
fn add_cache_headers(headers: &mut HashMap<String, String>) {
    let mut rng = rand::thread_rng();
    
    // Random cache behavior to appear more natural
    if rng.gen_bool(0.8) {
        // Most of the time, behave like a normal browser
        headers.insert("Cache-Control".to_string(), "max-age=0".to_string());
    } else if rng.gen_bool(0.5) {
        // Sometimes use no-cache
        headers.insert("Cache-Control".to_string(), "no-cache".to_string());
        headers.insert("Pragma".to_string(), "no-cache".to_string());
    } else {
        // Sometimes use a specific max-age
        let max_age = rng.gen_range(60..3600);
        headers.insert("Cache-Control".to_string(), format!("max-age={}", max_age));
    }
}

/// Add Referrer-Policy header
fn add_referrer_policy_headers(headers: &mut HashMap<String, String>) {
    let mut rng = rand::thread_rng();
    
    // Random referrer policy
    let policies = [
        "strict-origin-when-cross-origin",
        "no-referrer-when-downgrade",
        "origin-when-cross-origin",
        "same-origin",
    ];
    
    if rng.gen_bool(0.7) {
        let policy = policies[rng.gen_range(0..policies.len())];
        headers.insert("Referrer-Policy".to_string(), policy.to_string());
    }
}

/// Add Do-Not-Track header
fn add_do_not_track_header(headers: &mut HashMap<String, String>) {
    let mut rng = rand::thread_rng();
    
    // 30% chance of adding DNT header
    if rng.gen_bool(0.3) {
        headers.insert("DNT".to_string(), "1".to_string());
    }
}

/// Add random Client Hints headers
fn add_random_client_hints(headers: &mut HashMap<String, String>, device_type: &DeviceType) {
    let mut rng = rand::thread_rng();
    
    // 70% chance of adding client hints
    if rng.gen_bool(0.7) {
        headers.insert("Sec-CH-UA-Platform".to_string(), 
            match device_type {
                DeviceType::Mobile => "\"Android\"".to_string(),
                DeviceType::Tablet => if rng.gen_bool(0.5) { "\"Android\"".to_string() } else { "\"iOS\"".to_string() },
                _ => "\"Windows\"".to_string(),
            }
        );
        
        headers.insert("Sec-CH-UA-Mobile".to_string(), 
            match device_type {
                DeviceType::Mobile | DeviceType::Tablet | DeviceType::Watch => "?1".to_string(),
                _ => "?0".to_string(),
            }
        );
        
        // Add viewport width
        let width = match device_type {
            DeviceType::Mobile => rng.gen_range(320..480),
            DeviceType::Tablet => rng.gen_range(768..1024),
            _ => rng.gen_range(1024..1920),
        };
        
        headers.insert("Viewport-Width".to_string(), width.to_string());
        
        // Add device pixel ratio
        let dpr = match device_type {
            DeviceType::Mobile | DeviceType::Tablet => format!("{:.1}", rng.gen_range(2..4)),
            _ => format!("{:.1}", rng.gen_range(1..3)),
        };
        
        headers.insert("DPR".to_string(), dpr);
    }
}

/// Add Sec-Fetch-* headers based on URL purpose
fn add_sec_fetch_headers(headers: &mut HashMap<String, String>, url: &str) {
    // These headers indicate the type of request and help websites identify legitimate browsers
    
    // For basic page load
    headers.insert("Sec-Fetch-Dest".to_string(), "document".to_string());
    headers.insert("Sec-Fetch-Mode".to_string(), "navigate".to_string());
    headers.insert("Sec-Fetch-Site".to_string(), "cross-site".to_string());
    
    // If it's the first navigation, add Sec-Fetch-User
    let mut rng = rand::thread_rng();
    if rng.gen_bool(0.7) {
        headers.insert("Sec-Fetch-User".to_string(), "?1".to_string());
    }
}

/// Add Chrome-specific stealth headers
fn add_chrome_stealth_headers(headers: &mut HashMap<String, String>) {
    // Priority header used by Chrome
    headers.insert("Priority".to_string(), "u=0, i".to_string());
    
    let mut rng = rand::thread_rng();
    if rng.gen_bool(0.3) {
        // Add Chrome extension-related headers to make the request look even more realistic
        headers.insert("X-Client-Data".to_string(), generate_chrome_client_data());
    }
}

/// Add Firefox-specific stealth headers
fn add_firefox_stealth_headers(headers: &mut HashMap<String, String>) {
    // Firefox-specific headers
    headers.insert("TE".to_string(), "trailers".to_string());
    
    let mut rng = rand::thread_rng();
    if rng.gen_bool(0.6) {
        headers.insert("Accept-Encoding".to_string(), "gzip, deflate, br".to_string());
    }
}

/// Add Safari-specific stealth headers
fn add_safari_stealth_headers(headers: &mut HashMap<String, String>) {
    // Safari is more minimal in its headers
    // Often includes a different Accept header
    headers.insert("Accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".to_string());
}

/// Add Edge-specific stealth headers
fn add_edge_stealth_headers(headers: &mut HashMap<String, String>) {
    // Similar to Chrome but with some differences
    headers.insert("Priority".to_string(), "u=0, i".to_string());
}

/// Add Opera-specific stealth headers
fn add_opera_stealth_headers(headers: &mut HashMap<String, String>) {
    // Similar to Chrome but with some differences
    headers.insert("Priority".to_string(), "u=0, i".to_string());
}

/// Generate a random but realistic X-Client-Data header (Chrome)
fn generate_chrome_client_data() -> String {
    // This is a placeholder that generates a format similar to Chrome's X-Client-Data
    // Real X-Client-Data is more complex and encodes Chrome feature flags
    
    let random_bytes = [
        rand::random::<u8>(),
        rand::random::<u8>(),
        rand::random::<u8>(),
        rand::random::<u8>(),
        rand::random::<u8>(),
        rand::random::<u8>(),
        rand::random::<u8>(),
        rand::random::<u8>(),
    ];
    
    let base64_str = base64::encode(&random_bytes);
    
    format!("CIi2yQEIpbbJAQjBtskBCKmdygEI4qjKAQiWocsBCNS0zAEI+bXMAQ==")
}

/// Generate a realistic browser fingerprint to avoid detection
pub fn generate_fingerprint(browser_type: &BrowserType, device_type: &DeviceType) -> serde_json::Value {
    let mut rng = rand::thread_rng();
    
    // Common browser features that are checked by fingerprinting systems
    let java_enabled = false; // Modern browsers rarely have Java
    let flash_enabled = false; // Flash is deprecated
    
    // Screen resolution based on device type
    let (width, height) = match device_type {
        DeviceType::Mobile => (
            rng.gen_range(320..480),
            rng.gen_range(568..896)
        ),
        DeviceType::Tablet => (
            rng.gen_range(768..1024),
            rng.gen_range(1024..1366)
        ),
        _ => (
            rng.gen_range(1024..1920),
            rng.gen_range(768..1080)
        ),
    };
    
    // Color depth and pixel depth
    let color_depth = 24; // Standard for modern browsers
    let pixel_depth = 24; // Standard for modern browsers
    
    // Platform based on device type
    let platform = match device_type {
        DeviceType::Mobile => if rng.gen_bool(0.5) { "Android" } else { "iPhone" },
        DeviceType::Tablet => if rng.gen_bool(0.5) { "Android" } else { "iPad" },
        _ => if rng.gen_bool(0.7) { "Win32" } else { "MacIntel" },
    };
    
    // Build the fingerprint JSON
    serde_json::json!({
        "userAgent": browser_type.name(),
        "screenResolution": [width, height],
        "availableScreenResolution": [width, height],
        "colorDepth": color_depth,
        "pixelRatio": if device_type.is_mobile() { rng.gen_range(2..4) } else { rng.gen_range(1..3) },
        "deviceMemory": if device_type.is_mobile() { rng.gen_range(2..4) } else { rng.gen_range(4..16) },
        "hardwareConcurrency": if device_type.is_mobile() { rng.gen_range(4..8) } else { rng.gen_range(4..16) },
        "platform": platform,
        "plugins": [],
        "canvas": generate_random_canvas_hash(),
        "webgl": generate_random_webgl_hash(),
        "fonts": [],
        "audio": generate_random_audio_hash(),
        "language": "en-US",
        "languages": ["en-US", "en"],
        "timezone": "America/New_York",
        "timezoneOffset": -240,
        "touchSupport": device_type.is_mobile(),
        "videoInputs": if device_type.is_mobile() { 1 } else { rng.gen_range(0..2) },
        "doNotTrack": if rng.gen_bool(0.3) { "1" } else { "unspecified" },
        "javaEnabled": java_enabled,
        "flashEnabled": flash_enabled,
    })
}

/// Generate a random canvas fingerprint hash
fn generate_random_canvas_hash() -> String {
    let mut rng = rand::thread_rng();
    let random_bytes = [
        rng.gen::<u8>(), rng.gen::<u8>(), rng.gen::<u8>(), rng.gen::<u8>(),
        rng.gen::<u8>(), rng.gen::<u8>(), rng.gen::<u8>(), rng.gen::<u8>(),
        rng.gen::<u8>(), rng.gen::<u8>(), rng.gen::<u8>(), rng.gen::<u8>(),
        rng.gen::<u8>(), rng.gen::<u8>(), rng.gen::<u8>(), rng.gen::<u8>(),
    ];
    
    hex::encode(random_bytes)
}

/// Generate a random WebGL fingerprint hash
fn generate_random_webgl_hash() -> String {
    let mut rng = rand::thread_rng();
    let random_bytes = [
        rng.gen::<u8>(), rng.gen::<u8>(), rng.gen::<u8>(), rng.gen::<u8>(),
        rng.gen::<u8>(), rng.gen::<u8>(), rng.gen::<u8>(), rng.gen::<u8>(),
        rng.gen::<u8>(), rng.gen::<u8>(), rng.gen::<u8>(), rng.gen::<u8>(),
        rng.gen::<u8>(), rng.gen::<u8>(), rng.gen::<u8>(), rng.gen::<u8>(),
    ];
    
    hex::encode(random_bytes)
}

/// Generate a random audio fingerprint hash
fn generate_random_audio_hash() -> String {
    let mut rng = rand::thread_rng();
    let random_bytes = [
        rng.gen::<u8>(), rng.gen::<u8>(), rng.gen::<u8>(), rng.gen::<u8>(),
        rng.gen::<u8>(), rng.gen::<u8>(), rng.gen::<u8>(), rng.gen::<u8>(),
    ];
    
    hex::encode(random_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add_stealth_headers() {
        let mut headers = HashMap::new();
        add_stealth_headers(&mut headers, "https://example.com", &BrowserType::Chrome, &DeviceType::Desktop);
        
        assert!(headers.contains_key("Cache-Control"));
        assert!(headers.contains_key("Sec-Fetch-Dest"));
        assert!(headers.contains_key("Sec-Fetch-Mode"));
        assert!(headers.contains_key("Sec-Fetch-Site"));
    }
    
    #[test]
    fn test_generate_fingerprint() {
        let fingerprint = generate_fingerprint(&BrowserType::Chrome, &DeviceType::Desktop);
        
        assert_eq!(fingerprint["userAgent"], "Chrome");
        assert!(fingerprint["screenResolution"].as_array().unwrap().len() == 2);
        assert!(fingerprint.get("canvas").is_some());
        assert!(fingerprint.get("webgl").is_some());
    }
    
    #[test]
    fn test_generate_chrome_client_data() {
        let client_data = generate_chrome_client_data();
        assert!(!client_data.is_empty());
    }
    
    #[test]
    fn test_add_browser_specific_headers() {
        // Test Chrome
        let mut headers = HashMap::new();
        add_chrome_stealth_headers(&mut headers);
        assert!(headers.contains_key("Priority"));
        
        // Test Firefox
        let mut headers = HashMap::new();
        add_firefox_stealth_headers(&mut headers);
        assert!(headers.contains_key("TE"));
        
        // Test Safari
        let mut headers = HashMap::new();
        add_safari_stealth_headers(&mut headers);
        assert!(headers.contains_key("Accept"));
    }
} 