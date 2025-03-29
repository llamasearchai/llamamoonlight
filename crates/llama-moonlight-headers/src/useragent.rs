use rand::prelude::*;
use crate::{BrowserType, DeviceType, PlatformType};

/// Generate a user agent string for the given browser, device, and platform
pub fn generate_user_agent(
    browser_type: &BrowserType,
    device_type: &DeviceType,
    platform_type: &PlatformType,
) -> String {
    match browser_type {
        BrowserType::Chrome => generate_chrome_user_agent(device_type, platform_type),
        BrowserType::Firefox => generate_firefox_user_agent(device_type, platform_type),
        BrowserType::Safari => generate_safari_user_agent(device_type, platform_type),
        BrowserType::Edge => generate_edge_user_agent(device_type, platform_type),
        BrowserType::Opera => generate_opera_user_agent(device_type, platform_type),
        BrowserType::Custom(name) => format!("{}/{}", name, "1.0.0"),
    }
}

/// Generate a random user agent string
pub fn random_user_agent() -> String {
    let browser_type = BrowserType::random();
    let device_type = DeviceType::random();
    let platform_type = crate::platform::platform_for_device(&device_type);
    
    generate_user_agent(&browser_type, &device_type, &platform_type)
}

/// Generate a random mobile user agent string
pub fn random_mobile_user_agent() -> String {
    let browser_type = BrowserType::random();
    let device_type = DeviceType::random_mobile();
    let platform_type = crate::platform::platform_for_device(&device_type);
    
    generate_user_agent(&browser_type, &device_type, &platform_type)
}

/// Generate a random desktop user agent string
pub fn random_desktop_user_agent() -> String {
    let browser_type = BrowserType::random();
    let device_type = DeviceType::Desktop;
    let platform_type = crate::platform::platform_for_device(&device_type);
    
    generate_user_agent(&browser_type, &device_type, &platform_type)
}

/// Generate a Chrome user agent
fn generate_chrome_user_agent(device_type: &DeviceType, platform_type: &PlatformType) -> String {
    let mut rng = rand::thread_rng();
    let chrome_version = format!("{}.0.{}.{}",
        rng.gen_range(90..118),
        rng.gen_range(4000..5000),
        rng.gen_range(80..200)
    );
    
    match (device_type, platform_type) {
        (DeviceType::Mobile, PlatformType::Android) => {
            let android_version = platform_type.random_version();
            let device = "Linux; Android";
            format!(
                "Mozilla/5.0 ({} {}; wv) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/{} Mobile Safari/537.36",
                device, android_version, chrome_version
            )
        },
        (DeviceType::Mobile, PlatformType::IOS) | (DeviceType::Tablet, PlatformType::IOS) => {
            let ios_version = platform_type.random_version();
            let device = if matches!(device_type, DeviceType::Mobile) { "iPhone" } else { "iPad" };
            format!(
                "Mozilla/5.0 ({} OS {} like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) CriOS/{} Mobile/15E148 Safari/604.1",
                device, ios_version.replace('.', "_"), chrome_version
            )
        },
        (DeviceType::Tablet, PlatformType::Android) => {
            let android_version = platform_type.random_version();
            format!(
                "Mozilla/5.0 (Linux; Android {}; Tablet) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Safari/537.36",
                android_version, chrome_version
            )
        },
        (_, PlatformType::Windows) => {
            let windows_version = platform_type.random_version();
            format!(
                "Mozilla/5.0 (Windows NT {}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Safari/537.36",
                windows_version, chrome_version
            )
        },
        (_, PlatformType::MacOS) => {
            let macos_version = platform_type.random_version();
            format!(
                "Mozilla/5.0 (Macintosh; Intel Mac OS X {}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Safari/537.36",
                macos_version.replace('.', "_"), chrome_version
            )
        },
        (_, PlatformType::Linux) => {
            format!(
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Safari/537.36",
                chrome_version
            )
        },
        (_, PlatformType::ChromeOS) => {
            format!(
                "Mozilla/5.0 (X11; CrOS x86_64 {}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Safari/537.36",
                platform_type.random_version(), chrome_version
            )
        },
        _ => {
            format!(
                "Mozilla/5.0 (Unknown; {}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Safari/537.36",
                platform_type.name(), chrome_version
            )
        },
    }
}

/// Generate a Firefox user agent
fn generate_firefox_user_agent(device_type: &DeviceType, platform_type: &PlatformType) -> String {
    let mut rng = rand::thread_rng();
    let firefox_version = format!("{}",
        rng.gen_range(90..118),
    );
    
    match (device_type, platform_type) {
        (DeviceType::Mobile, PlatformType::Android) => {
            let android_version = platform_type.random_version();
            format!(
                "Mozilla/5.0 (Android {}; Mobile; rv:{}.0) Gecko/{}.0 Firefox/{}.0",
                android_version, firefox_version, firefox_version, firefox_version
            )
        },
        (DeviceType::Tablet, PlatformType::Android) => {
            let android_version = platform_type.random_version();
            format!(
                "Mozilla/5.0 (Android {}; Tablet; rv:{}.0) Gecko/{}.0 Firefox/{}.0",
                android_version, firefox_version, firefox_version, firefox_version
            )
        },
        (DeviceType::Mobile, PlatformType::IOS) | (DeviceType::Tablet, PlatformType::IOS) => {
            let ios_version = platform_type.random_version();
            let device = if matches!(device_type, DeviceType::Mobile) { "iPhone" } else { "iPad" };
            format!(
                "Mozilla/5.0 ({} OS {} like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) FxiOS/{} Mobile/15E148 Safari/605.1.15",
                device, ios_version.replace('.', "_"), firefox_version
            )
        },
        (_, PlatformType::Windows) => {
            let windows_version = platform_type.random_version();
            format!(
                "Mozilla/5.0 (Windows NT {}; rv:{}.0) Gecko/{}.0 Firefox/{}.0",
                windows_version, firefox_version, firefox_version, firefox_version
            )
        },
        (_, PlatformType::MacOS) => {
            let macos_version = platform_type.random_version();
            format!(
                "Mozilla/5.0 (Macintosh; Intel Mac OS X {}; rv:{}.0) Gecko/{}.0 Firefox/{}.0",
                macos_version.replace('.', "_"), firefox_version, firefox_version, firefox_version
            )
        },
        (_, PlatformType::Linux) => {
            format!(
                "Mozilla/5.0 (X11; Linux x86_64; rv:{}.0) Gecko/{}.0 Firefox/{}.0",
                firefox_version, firefox_version, firefox_version
            )
        },
        _ => {
            format!(
                "Mozilla/5.0 (Unknown; {}; rv:{}.0) Gecko/{}.0 Firefox/{}.0",
                platform_type.name(), firefox_version, firefox_version, firefox_version
            )
        },
    }
}

/// Generate a Safari user agent
fn generate_safari_user_agent(device_type: &DeviceType, platform_type: &PlatformType) -> String {
    let mut rng = rand::thread_rng();
    let safari_version = format!("{}.{}.{}",
        rng.gen_range(12..17),
        rng.gen_range(0..5),
        rng.gen_range(0..20)
    );
    
    let webkit_version = format!("605.1.{}",
        rng.gen_range(1..16)
    );
    
    match (device_type, platform_type) {
        (DeviceType::Mobile, PlatformType::IOS) => {
            let ios_version = platform_type.random_version();
            format!(
                "Mozilla/5.0 (iPhone; CPU iPhone OS {} like Mac OS X) AppleWebKit/{} (KHTML, like Gecko) Version/{} Mobile/15E148 Safari/{}",
                ios_version.replace('.', "_"), webkit_version, safari_version, webkit_version
            )
        },
        (DeviceType::Tablet, PlatformType::IOS) => {
            let ios_version = platform_type.random_version();
            format!(
                "Mozilla/5.0 (iPad; CPU OS {} like Mac OS X) AppleWebKit/{} (KHTML, like Gecko) Version/{} Mobile/15E148 Safari/{}",
                ios_version.replace('.', "_"), webkit_version, safari_version, webkit_version
            )
        },
        (_, PlatformType::MacOS) => {
            let macos_version = platform_type.random_version();
            format!(
                "Mozilla/5.0 (Macintosh; Intel Mac OS X {}) AppleWebKit/{} (KHTML, like Gecko) Version/{} Safari/{}",
                macos_version.replace('.', "_"), webkit_version, safari_version, webkit_version
            )
        },
        _ => {
            // Safari is primarily available on Apple platforms, so this is a fallback
            format!(
                "Mozilla/5.0 (Unknown; {}; rv:{}) AppleWebKit/{} (KHTML, like Gecko) Version/{} Safari/{}",
                platform_type.name(), platform_type.random_version(), webkit_version, safari_version, webkit_version
            )
        },
    }
}

/// Generate an Edge user agent
fn generate_edge_user_agent(device_type: &DeviceType, platform_type: &PlatformType) -> String {
    let mut rng = rand::thread_rng();
    let edge_version = format!("{}.0.{}.{}",
        rng.gen_range(90..118),
        rng.gen_range(1000..2000),
        rng.gen_range(0..200)
    );
    
    let chrome_version = format!("{}.0.{}.{}",
        rng.gen_range(90..118),
        rng.gen_range(4000..5000),
        rng.gen_range(80..200)
    );
    
    match (device_type, platform_type) {
        (DeviceType::Mobile, PlatformType::Android) => {
            let android_version = platform_type.random_version();
            format!(
                "Mozilla/5.0 (Linux; Android {}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Mobile Safari/537.36 EdgA/{}",
                android_version, chrome_version, edge_version
            )
        },
        (DeviceType::Mobile, PlatformType::IOS) => {
            let ios_version = platform_type.random_version();
            format!(
                "Mozilla/5.0 (iPhone; CPU iPhone OS {} like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.0 EdgiOS/{} Mobile/15E148 Safari/605.1.15",
                ios_version.replace('.', "_"), edge_version
            )
        },
        (DeviceType::Tablet, PlatformType::IOS) => {
            let ios_version = platform_type.random_version();
            format!(
                "Mozilla/5.0 (iPad; CPU OS {} like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.0 EdgiOS/{} Mobile/15E148 Safari/605.1.15",
                ios_version.replace('.', "_"), edge_version
            )
        },
        (_, PlatformType::Windows) => {
            let windows_version = platform_type.random_version();
            format!(
                "Mozilla/5.0 (Windows NT {}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Safari/537.36 Edg/{}",
                windows_version, chrome_version, edge_version
            )
        },
        (_, PlatformType::MacOS) => {
            let macos_version = platform_type.random_version();
            format!(
                "Mozilla/5.0 (Macintosh; Intel Mac OS X {}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Safari/537.36 Edg/{}",
                macos_version.replace('.', "_"), chrome_version, edge_version
            )
        },
        (_, PlatformType::Linux) => {
            format!(
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Safari/537.36 Edg/{}",
                chrome_version, edge_version
            )
        },
        _ => {
            format!(
                "Mozilla/5.0 (Unknown; {}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Safari/537.36 Edg/{}",
                platform_type.name(), chrome_version, edge_version
            )
        },
    }
}

/// Generate an Opera user agent
fn generate_opera_user_agent(device_type: &DeviceType, platform_type: &PlatformType) -> String {
    let mut rng = rand::thread_rng();
    let opera_version = format!("{}.0.{}.{}",
        rng.gen_range(80..103),
        rng.gen_range(0..5),
        rng.gen_range(0..200)
    );
    
    let chrome_version = format!("{}.0.{}.{}",
        rng.gen_range(90..118),
        rng.gen_range(4000..5000),
        rng.gen_range(80..200)
    );
    
    match (device_type, platform_type) {
        (DeviceType::Mobile, PlatformType::Android) => {
            let android_version = platform_type.random_version();
            format!(
                "Mozilla/5.0 (Linux; Android {}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Mobile Safari/537.36 OPR/{}",
                android_version, chrome_version, opera_version
            )
        },
        (DeviceType::Mobile, PlatformType::IOS) => {
            let ios_version = platform_type.random_version();
            format!(
                "Mozilla/5.0 (iPhone; CPU iPhone OS {} like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) OPT/{} Mobile/15E148",
                ios_version.replace('.', "_"), opera_version
            )
        },
        (DeviceType::Tablet, PlatformType::IOS) => {
            let ios_version = platform_type.random_version();
            format!(
                "Mozilla/5.0 (iPad; CPU OS {} like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) OPT/{} Mobile/15E148",
                ios_version.replace('.', "_"), opera_version
            )
        },
        (_, PlatformType::Windows) => {
            let windows_version = platform_type.random_version();
            format!(
                "Mozilla/5.0 (Windows NT {}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Safari/537.36 OPR/{}",
                windows_version, chrome_version, opera_version
            )
        },
        (_, PlatformType::MacOS) => {
            let macos_version = platform_type.random_version();
            format!(
                "Mozilla/5.0 (Macintosh; Intel Mac OS X {}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Safari/537.36 OPR/{}",
                macos_version.replace('.', "_"), chrome_version, opera_version
            )
        },
        (_, PlatformType::Linux) => {
            format!(
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Safari/537.36 OPR/{}",
                chrome_version, opera_version
            )
        },
        _ => {
            format!(
                "Mozilla/5.0 (Unknown; {}) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Safari/537.36 OPR/{}",
                platform_type.name(), chrome_version, opera_version
            )
        },
    }
}

/// Check if a user agent is mobile
pub fn is_mobile_user_agent(user_agent: &str) -> bool {
    let ua = user_agent.to_lowercase();
    ua.contains("mobile") || 
    ua.contains("android") ||
    ua.contains("iphone") || 
    ua.contains("ipad") ||
    ua.contains("ipod") ||
    ua.contains("windows phone") ||
    ua.contains("blackberry")
}

/// Parse a user agent string to extract the browser type
pub fn parse_browser_from_user_agent(user_agent: &str) -> Option<BrowserType> {
    let ua = user_agent.to_lowercase();
    
    // Check for different browsers in order of specificity
    if ua.contains("edg/") || ua.contains("edge/") || ua.contains("edgios/") || ua.contains("edga/") {
        Some(BrowserType::Edge)
    } else if ua.contains("opr/") || ua.contains("opt/") || ua.contains("opera") {
        Some(BrowserType::Opera)
    } else if ua.contains("firefox/") || ua.contains("fxios/") {
        Some(BrowserType::Firefox)
    } else if ua.contains("chrome/") || ua.contains("crios/") {
        Some(BrowserType::Chrome)
    } else if ua.contains("safari/") && !ua.contains("chrome") && !ua.contains("chromium") {
        Some(BrowserType::Safari)
    } else {
        None
    }
}

/// Parse a user agent string to extract the platform type
pub fn parse_platform_from_user_agent(user_agent: &str) -> Option<PlatformType> {
    let ua = user_agent.to_lowercase();
    
    if ua.contains("windows") || ua.contains("win64") || ua.contains("win32") {
        Some(PlatformType::Windows)
    } else if ua.contains("android") {
        Some(PlatformType::Android)
    } else if ua.contains("iphone") || ua.contains("ipad") || (ua.contains("ios") && !ua.contains("mac os")) {
        Some(PlatformType::IOS)
    } else if ua.contains("macintosh") || ua.contains("mac os") {
        Some(PlatformType::MacOS)
    } else if ua.contains("linux") && !ua.contains("android") {
        Some(PlatformType::Linux)
    } else if ua.contains("cros") {
        Some(PlatformType::ChromeOS)
    } else {
        None
    }
}

/// Parse a user agent string to extract the device type
pub fn parse_device_from_user_agent(user_agent: &str) -> Option<DeviceType> {
    let ua = user_agent.to_lowercase();
    
    if ua.contains("mobile") || ua.contains("iphone") || ua.contains("android") && !ua.contains("tablet") {
        Some(DeviceType::Mobile)
    } else if ua.contains("ipad") || ua.contains("tablet") {
        Some(DeviceType::Tablet)
    } else if ua.contains("tv") || ua.contains("smart-tv") || ua.contains("appletv") {
        Some(DeviceType::TV)
    } else if ua.contains("nintendo") || ua.contains("playstation") || ua.contains("xbox") {
        Some(DeviceType::Console)
    } else if !is_mobile_user_agent(&ua) {
        Some(DeviceType::Desktop)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_user_agent() {
        // Test Chrome on Windows
        let ua = generate_user_agent(
            &BrowserType::Chrome,
            &DeviceType::Desktop,
            &PlatformType::Windows
        );
        assert!(ua.contains("Chrome"));
        assert!(ua.contains("Windows"));
        
        // Test Firefox on Android
        let ua = generate_user_agent(
            &BrowserType::Firefox,
            &DeviceType::Mobile,
            &PlatformType::Android
        );
        assert!(ua.contains("Firefox"));
        assert!(ua.contains("Android"));
        assert!(ua.contains("Mobile"));
        
        // Test Safari on iOS
        let ua = generate_user_agent(
            &BrowserType::Safari,
            &DeviceType::Mobile,
            &PlatformType::IOS
        );
        assert!(ua.contains("Safari"));
        assert!(ua.contains("iPhone"));
    }
    
    #[test]
    fn test_random_user_agent() {
        let ua = random_user_agent();
        assert!(!ua.is_empty());
    }
    
    #[test]
    fn test_parse_browser_from_user_agent() {
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";
        assert_eq!(parse_browser_from_user_agent(ua), Some(BrowserType::Chrome));
        
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:89.0) Gecko/20100101 Firefox/89.0";
        assert_eq!(parse_browser_from_user_agent(ua), Some(BrowserType::Firefox));
        
        let ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.1 Safari/605.1.15";
        assert_eq!(parse_browser_from_user_agent(ua), Some(BrowserType::Safari));
    }
    
    #[test]
    fn test_parse_platform_from_user_agent() {
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";
        assert_eq!(parse_platform_from_user_agent(ua), Some(PlatformType::Windows));
        
        let ua = "Mozilla/5.0 (iPhone; CPU iPhone OS 14_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0 Mobile/15E148 Safari/604.1";
        assert_eq!(parse_platform_from_user_agent(ua), Some(PlatformType::IOS));
        
        let ua = "Mozilla/5.0 (Linux; Android 11; SM-G991B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.120 Mobile Safari/537.36";
        assert_eq!(parse_platform_from_user_agent(ua), Some(PlatformType::Android));
    }
    
    #[test]
    fn test_parse_device_from_user_agent() {
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";
        assert_eq!(parse_device_from_user_agent(ua), Some(DeviceType::Desktop));
        
        let ua = "Mozilla/5.0 (iPhone; CPU iPhone OS 14_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0 Mobile/15E148 Safari/604.1";
        assert_eq!(parse_device_from_user_agent(ua), Some(DeviceType::Mobile));
        
        let ua = "Mozilla/5.0 (iPad; CPU OS 14_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0 Mobile/15E148 Safari/604.1";
        assert_eq!(parse_device_from_user_agent(ua), Some(DeviceType::Tablet));
    }
} 