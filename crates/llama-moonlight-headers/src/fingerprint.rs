use crate::{BrowserType, DeviceType, PlatformType};
use rand::prelude::*;
use std::collections::HashMap;

/// Browser fingerprint data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BrowserFingerprint {
    /// User agent string
    pub user_agent: String,
    
    /// Screen width
    pub screen_width: u32,
    
    /// Screen height
    pub screen_height: u32,
    
    /// Available screen width
    pub available_width: u32,
    
    /// Available screen height
    pub available_height: u32,
    
    /// Color depth
    pub color_depth: u8,
    
    /// Device pixel ratio
    pub pixel_ratio: f32,
    
    /// Browser language
    pub language: String,
    
    /// Browser languages
    pub languages: Vec<String>,
    
    /// Timezone
    pub timezone: String,
    
    /// Timezone offset in minutes
    pub timezone_offset: i32,
    
    /// Whether Do Not Track is enabled
    pub do_not_track: Option<bool>,
    
    /// Whether cookies are enabled
    pub cookies_enabled: bool,
    
    /// Whether localStorage is available
    pub local_storage: bool,
    
    /// Whether sessionStorage is available
    pub session_storage: bool,
    
    /// Whether IndexedDB is available
    pub indexed_db: bool,
    
    /// Whether canvas fingerprinting is blocked
    pub canvas_blocked: bool,
    
    /// Canvas fingerprint hash
    pub canvas_hash: Option<String>,
    
    /// WebGL fingerprint hash
    pub webgl_hash: Option<String>,
    
    /// WebGL vendor
    pub webgl_vendor: Option<String>,
    
    /// WebGL renderer
    pub webgl_renderer: Option<String>,
    
    /// Audio fingerprint hash
    pub audio_hash: Option<String>,
    
    /// Browser platform
    pub platform: String,
    
    /// Browser vendor
    pub vendor: String,
    
    /// Browser product
    pub product: String,
    
    /// Whether touch is supported
    pub touch_supported: bool,
    
    /// Maximum touch points
    pub max_touch_points: u8,
    
    /// Hardware concurrency (CPU cores)
    pub hardware_concurrency: u8,
    
    /// Device memory in GB
    pub device_memory: u8,
    
    /// List of installed plugins
    pub plugins: Vec<String>,
    
    /// List of installed fonts
    pub fonts: Vec<String>,
    
    /// Whether battery status API is supported
    pub battery_api: bool,
    
    /// Whether gamepads are supported
    pub gamepads: bool,
    
    /// Whether Web Bluetooth is supported
    pub bluetooth: bool,
    
    /// Whether Web USB is supported
    pub usb: bool,
    
    /// Whether notifications are supported
    pub notifications: bool,
    
    /// Whether microphone is available
    pub microphone: bool,
    
    /// Whether camera is available
    pub camera: bool,
}

impl Default for BrowserFingerprint {
    fn default() -> Self {
        Self {
            user_agent: "".to_string(),
            screen_width: 1920,
            screen_height: 1080,
            available_width: 1920,
            available_height: 1080,
            color_depth: 24,
            pixel_ratio: 1.0,
            language: "en-US".to_string(),
            languages: vec!["en-US".to_string(), "en".to_string()],
            timezone: "America/New_York".to_string(),
            timezone_offset: -240,
            do_not_track: None,
            cookies_enabled: true,
            local_storage: true,
            session_storage: true,
            indexed_db: true,
            canvas_blocked: false,
            canvas_hash: None,
            webgl_hash: None,
            webgl_vendor: None,
            webgl_renderer: None,
            audio_hash: None,
            platform: "Win32".to_string(),
            vendor: "".to_string(),
            product: "Gecko".to_string(),
            touch_supported: false,
            max_touch_points: 0,
            hardware_concurrency: 8,
            device_memory: 8,
            plugins: vec![],
            fonts: vec![],
            battery_api: true,
            gamepads: true,
            bluetooth: false,
            usb: false,
            notifications: true,
            microphone: false,
            camera: false,
        }
    }
}

impl BrowserFingerprint {
    /// Create a new fingerprint for the given browser, device, and platform
    pub fn new(
        browser_type: &BrowserType,
        device_type: &DeviceType,
        platform_type: &PlatformType,
    ) -> Self {
        let mut rng = rand::thread_rng();
        
        // Generate a user agent for this configuration
        let user_agent = crate::useragent::generate_user_agent(browser_type, device_type, platform_type);
        
        // Determine screen size based on device type
        let (screen_width, screen_height) = match device_type {
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
        
        // Available size is usually slightly smaller due to OS UI
        let available_width = (screen_width as f32 * 0.98) as u32;
        let available_height = (screen_height as f32 * 0.92) as u32;
        
        // Pixel ratio is higher on mobile devices
        let pixel_ratio = match device_type {
            DeviceType::Mobile | DeviceType::Tablet => rng.gen_range(2.0..3.0),
            _ => rng.gen_range(1.0..2.0),
        };
        
        // Hardware specs based on device type
        let hardware_concurrency = match device_type {
            DeviceType::Mobile | DeviceType::Tablet => rng.gen_range(4..8),
            _ => rng.gen_range(4..16),
        };
        
        let device_memory = match device_type {
            DeviceType::Mobile | DeviceType::Tablet => rng.gen_range(2..4),
            _ => rng.gen_range(4..16),
        };
        
        // Platform string based on platform type
        let platform = match platform_type {
            PlatformType::Windows => "Win32".to_string(),
            PlatformType::MacOS => "MacIntel".to_string(),
            PlatformType::Linux => "Linux x86_64".to_string(),
            PlatformType::Android => "Linux armv8l".to_string(),
            PlatformType::IOS => "iPhone".to_string(),
            PlatformType::ChromeOS => "CrOS x86_64".to_string(),
            PlatformType::Custom(name) => name.clone(),
        };
        
        // Create and return the fingerprint
        Self {
            user_agent,
            screen_width,
            screen_height,
            available_width,
            available_height,
            color_depth: 24, // Standard for modern displays
            pixel_ratio,
            language: "en-US".to_string(),
            languages: vec!["en-US".to_string(), "en".to_string()],
            timezone: "America/New_York".to_string(), // Default timezone
            timezone_offset: -240, // EDT offset in minutes
            do_not_track: if rng.gen_bool(0.2) { Some(true) } else { None },
            cookies_enabled: true,
            local_storage: true,
            session_storage: true,
            indexed_db: true,
            canvas_blocked: rng.gen_bool(0.05), // 5% chance of canvas blocking
            canvas_hash: Some(generate_random_hash(16)),
            webgl_hash: Some(generate_random_hash(16)),
            webgl_vendor: match (browser_type, platform_type) {
                (_, PlatformType::Windows) => Some("Google Inc.".to_string()),
                (_, PlatformType::MacOS) => Some("Apple Inc.".to_string()),
                _ => Some("WebKit".to_string()),
            },
            webgl_renderer: match platform_type {
                PlatformType::Windows => Some("ANGLE (Intel, Intel(R) UHD Graphics 620 Direct3D11 vs_5_0 ps_5_0)".to_string()),
                PlatformType::MacOS => Some("Apple GPU".to_string()),
                _ => Some("Mesa Intel HD Graphics 620".to_string()),
            },
            audio_hash: Some(generate_random_hash(8)),
            platform,
            vendor: browser_type.vendor(),
            product: "Gecko".to_string(),
            touch_supported: device_type.is_mobile(),
            max_touch_points: if device_type.is_mobile() { 5 } else { 0 },
            hardware_concurrency,
            device_memory,
            plugins: generate_plugins_for_browser(browser_type),
            fonts: generate_common_fonts(),
            battery_api: rng.gen_bool(0.8),
            gamepads: rng.gen_bool(0.5),
            bluetooth: rng.gen_bool(0.2),
            usb: rng.gen_bool(0.2),
            notifications: true,
            microphone: rng.gen_bool(0.3),
            camera: rng.gen_bool(0.3),
        }
    }
    
    /// Generate a consistent fingerprint for a specific configuration
    pub fn consistent(
        browser_type: &BrowserType,
        device_type: &DeviceType,
        platform_type: &PlatformType,
        seed: u64,
    ) -> Self {
        // Use a seeded RNG for consistency
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        
        // Generate a user agent for this configuration
        let user_agent = crate::useragent::generate_user_agent(browser_type, device_type, platform_type);
        
        // Determine screen size based on device type
        let (screen_width, screen_height) = match device_type {
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
        
        // Available size is usually slightly smaller due to OS UI
        let available_width = (screen_width as f32 * 0.98) as u32;
        let available_height = (screen_height as f32 * 0.92) as u32;
        
        // Pixel ratio is higher on mobile devices
        let pixel_ratio = match device_type {
            DeviceType::Mobile | DeviceType::Tablet => rng.gen_range(2.0..3.0),
            _ => rng.gen_range(1.0..2.0),
        };
        
        // Create the canvas hash based on the seed
        let mut canvas_hash = [0u8; 16];
        for i in 0..16 {
            canvas_hash[i] = ((seed + i as u64) % 256) as u8;
        }
        
        // Create the webgl hash based on the seed
        let mut webgl_hash = [0u8; 16];
        for i in 0..16 {
            webgl_hash[i] = ((seed + i as u64 + 100) % 256) as u8;
        }
        
        // Create the audio hash based on the seed
        let mut audio_hash = [0u8; 8];
        for i in 0..8 {
            audio_hash[i] = ((seed + i as u64 + 200) % 256) as u8;
        }
        
        // Platform string based on platform type
        let platform = match platform_type {
            PlatformType::Windows => "Win32".to_string(),
            PlatformType::MacOS => "MacIntel".to_string(),
            PlatformType::Linux => "Linux x86_64".to_string(),
            PlatformType::Android => "Linux armv8l".to_string(),
            PlatformType::IOS => "iPhone".to_string(),
            PlatformType::ChromeOS => "CrOS x86_64".to_string(),
            PlatformType::Custom(name) => name.clone(),
        };
        
        Self {
            user_agent,
            screen_width,
            screen_height,
            available_width,
            available_height,
            color_depth: 24,
            pixel_ratio,
            language: "en-US".to_string(),
            languages: vec!["en-US".to_string(), "en".to_string()],
            timezone: "America/New_York".to_string(),
            timezone_offset: -240,
            do_not_track: None,
            cookies_enabled: true,
            local_storage: true,
            session_storage: true,
            indexed_db: true,
            canvas_blocked: false,
            canvas_hash: Some(hex::encode(canvas_hash)),
            webgl_hash: Some(hex::encode(webgl_hash)),
            webgl_vendor: match (browser_type, platform_type) {
                (_, PlatformType::Windows) => Some("Google Inc.".to_string()),
                (_, PlatformType::MacOS) => Some("Apple Inc.".to_string()),
                _ => Some("WebKit".to_string()),
            },
            webgl_renderer: match platform_type {
                PlatformType::Windows => Some("ANGLE (Intel, Intel(R) UHD Graphics 620 Direct3D11 vs_5_0 ps_5_0)".to_string()),
                PlatformType::MacOS => Some("Apple GPU".to_string()),
                _ => Some("Mesa Intel HD Graphics 620".to_string()),
            },
            audio_hash: Some(hex::encode(audio_hash)),
            platform,
            vendor: browser_type.vendor(),
            product: "Gecko".to_string(),
            touch_supported: device_type.is_mobile(),
            max_touch_points: if device_type.is_mobile() { 5 } else { 0 },
            hardware_concurrency: 4, // Consistent value
            device_memory: 8, // Consistent value
            plugins: generate_plugins_for_browser(browser_type),
            fonts: generate_common_fonts(),
            battery_api: true,
            gamepads: false,
            bluetooth: false,
            usb: false,
            notifications: true,
            microphone: false,
            camera: false,
        }
    }
    
    /// Convert the fingerprint to a JavaScript snippet
    pub fn to_js(&self) -> String {
        let js = format!(
            r#"
            // Browser fingerprint override script
            (() => {{
                // Helper to override property getters
                const override_getter = (obj, prop, value) => {{
                    try {{
                        Object.defineProperty(obj, prop, {{ 
                            get: () => value, 
                            configurable: true 
                        }});
                    }} catch (e) {{ 
                        console.log(`Failed to override ${{prop}}: ${{e}}`); 
                    }}
                }};
                
                // Override navigator properties
                override_getter(navigator, 'userAgent', '{user_agent}');
                override_getter(navigator, 'platform', '{platform}');
                override_getter(navigator, 'vendor', '{vendor}');
                override_getter(navigator, 'language', '{language}');
                override_getter(navigator, 'languages', {languages});
                override_getter(navigator, 'hardwareConcurrency', {hardware_concurrency});
                override_getter(navigator, 'deviceMemory', {device_memory});
                override_getter(navigator, 'maxTouchPoints', {max_touch_points});
                
                // Override screen properties
                override_getter(screen, 'width', {screen_width});
                override_getter(screen, 'height', {screen_height});
                override_getter(screen, 'availWidth', {available_width});
                override_getter(screen, 'availHeight', {available_height});
                override_getter(screen, 'colorDepth', {color_depth});
                override_getter(window, 'devicePixelRatio', {pixel_ratio});
                
                // Override Chrome's chrome.app.getDetails
                if (window.chrome && chrome.app) {{
                    chrome.app.getDetails = () => null;
                }}
                
                // Canvas fingerprinting protection
                if ({canvas_blocked}) {{
                    const canvasProto = CanvasRenderingContext2D.prototype;
                    const originalGetImageData = canvasProto.getImageData;
                    canvasProto.getImageData = function(x, y, width, height) {{
                        const imageData = originalGetImageData.call(this, x, y, width, height);
                        const pixels = imageData.data;
                        
                        // Add some noise to the canvas data
                        for (let i = 0; i < pixels.length; i += 4) {{
                            pixels[i] = pixels[i] + Math.floor(Math.random() * 10) - 5;     // red
                            pixels[i+1] = pixels[i+1] + Math.floor(Math.random() * 10) - 5; // green
                            pixels[i+2] = pixels[i+2] + Math.floor(Math.random() * 10) - 5; // blue
                        }}
                        
                        return imageData;
                    }};
                }}
                
                // WebGL fingerprinting protection
                try {{
                    const getParameterProto = WebGLRenderingContext.prototype.getParameter;
                    WebGLRenderingContext.prototype.getParameter = function(parameter) {{
                        // Override vendor and renderer strings
                        if (parameter === 37445) {{ // UNMASKED_VENDOR_WEBGL
                            return '{webgl_vendor}';
                        }}
                        if (parameter === 37446) {{ // UNMASKED_RENDERER_WEBGL
                            return '{webgl_renderer}';
                        }}
                        
                        return getParameterProto.call(this, parameter);
                    }};
                }} catch (e) {{}}
            }})();
            "#,
            user_agent = self.user_agent,
            platform = self.platform,
            vendor = self.vendor,
            language = self.language,
            languages = serde_json::to_string(&self.languages).unwrap_or_else(|_| "['en-US', 'en']".to_string()),
            hardware_concurrency = self.hardware_concurrency,
            device_memory = self.device_memory,
            max_touch_points = self.max_touch_points,
            screen_width = self.screen_width,
            screen_height = self.screen_height,
            available_width = self.available_width,
            available_height = self.available_height,
            color_depth = self.color_depth,
            pixel_ratio = self.pixel_ratio,
            canvas_blocked = self.canvas_blocked,
            webgl_vendor = self.webgl_vendor.clone().unwrap_or_else(|| "Google Inc.".to_string()),
            webgl_renderer = self.webgl_renderer.clone().unwrap_or_else(|| "ANGLE (Intel HD Graphics)".to_string()),
        );
        
        js
    }
    
    /// Convert the fingerprint to a JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
    
    /// Parse a fingerprint from a JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Generate a random fingerprint that looks realistic
pub fn generate_random_fingerprint() -> BrowserFingerprint {
    let browser_type = BrowserType::random();
    let device_type = DeviceType::random();
    let platform_type = crate::platform::platform_for_device(&device_type);
    
    BrowserFingerprint::new(&browser_type, &device_type, &platform_type)
}

/// Generate a random hex string of the given length (in bytes)
fn generate_random_hash(byte_length: usize) -> String {
    let mut rng = rand::thread_rng();
    let mut bytes = vec![0u8; byte_length];
    rng.fill_bytes(&mut bytes);
    
    hex::encode(bytes)
}

/// Generate a list of plugins for the given browser
fn generate_plugins_for_browser(browser_type: &BrowserType) -> Vec<String> {
    match browser_type {
        BrowserType::Chrome => vec![
            "Chrome PDF Plugin".to_string(),
            "Chrome PDF Viewer".to_string(),
            "Native Client".to_string(),
        ],
        BrowserType::Firefox => vec![
            "PDF Viewer".to_string(),
            "Firefox Default PDF Handler".to_string(),
        ],
        BrowserType::Edge => vec![
            "PDF Viewer".to_string(),
            "Microsoft Edge PDF Plugin".to_string(),
        ],
        BrowserType::Safari => vec![
            "QuickTime Plugin".to_string(),
            "WebKit built-in PDF".to_string(),
        ],
        _ => vec![],
    }
}

/// Generate a list of common fonts
fn generate_common_fonts() -> Vec<String> {
    vec![
        "Arial".to_string(),
        "Arial Black".to_string(),
        "Calibri".to_string(),
        "Cambria".to_string(),
        "Candara".to_string(),
        "Comic Sans MS".to_string(),
        "Courier".to_string(),
        "Courier New".to_string(),
        "Georgia".to_string(),
        "Helvetica".to_string(),
        "Impact".to_string(),
        "Lucida Console".to_string(),
        "Tahoma".to_string(),
        "Times".to_string(),
        "Times New Roman".to_string(),
        "Trebuchet MS".to_string(),
        "Verdana".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_browser_fingerprint_new() {
        let fp = BrowserFingerprint::new(
            &BrowserType::Chrome,
            &DeviceType::Desktop,
            &PlatformType::Windows,
        );
        
        assert!(fp.user_agent.contains("Chrome"));
        assert!(fp.user_agent.contains("Windows"));
        assert_eq!(fp.platform, "Win32");
        assert_eq!(fp.color_depth, 24);
        assert!(fp.hardware_concurrency >= 4);
        assert!(fp.device_memory >= 4);
    }
    
    #[test]
    fn test_browser_fingerprint_consistent() {
        let fp1 = BrowserFingerprint::consistent(
            &BrowserType::Chrome,
            &DeviceType::Desktop,
            &PlatformType::Windows,
            12345,
        );
        
        let fp2 = BrowserFingerprint::consistent(
            &BrowserType::Chrome,
            &DeviceType::Desktop,
            &PlatformType::Windows,
            12345,
        );
        
        // Same seed should produce identical fingerprints
        assert_eq!(fp1.canvas_hash, fp2.canvas_hash);
        assert_eq!(fp1.webgl_hash, fp2.webgl_hash);
        assert_eq!(fp1.audio_hash, fp2.audio_hash);
    }
    
    #[test]
    fn test_browser_fingerprint_to_js() {
        let fp = BrowserFingerprint::new(
            &BrowserType::Chrome,
            &DeviceType::Desktop,
            &PlatformType::Windows,
        );
        
        let js = fp.to_js();
        assert!(js.contains("override_getter(navigator, 'userAgent'"));
        assert!(js.contains("override_getter(screen, 'width'"));
    }
    
    #[test]
    fn test_browser_fingerprint_json() {
        let fp = BrowserFingerprint::new(
            &BrowserType::Chrome,
            &DeviceType::Desktop,
            &PlatformType::Windows,
        );
        
        let json = fp.to_json().unwrap();
        let parsed = BrowserFingerprint::from_json(&json).unwrap();
        
        assert_eq!(fp.user_agent, parsed.user_agent);
        assert_eq!(fp.screen_width, parsed.screen_width);
        assert_eq!(fp.screen_height, parsed.screen_height);
    }
} 