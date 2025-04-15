//! Module for handling User-Agent strings
use crate::errors::LlamaHeadersError;
use rand::seq::SliceRandom;
use rand::thread_rng;
use regex::Regex;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

/// Represents a User-Agent string and its parsed components.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct UserAgent {
    pub string: String,
    pub browser: String,
    pub browser_version: String,
    pub os: String,
    pub os_version: String,
    mobile: bool,
}

impl UserAgent {
    /// Parses a User-Agent string and creates a `UserAgent` instance.
    pub fn parse(ua_string: &str) -> Result<Self, LlamaHeadersError> {
        lazy_static! {
            static ref RE_MOBILE: Regex = Regex::new(r"(Mobile|Tablet)").unwrap();

            // Regex for major browsers and OS, keep expanding
            static ref RE_BROWSER_OS: Regex = Regex::new(r"(Chrome|Firefox|Safari|Edge|MSIE|Opera|Trident)/(?P<version>\d+(\.\d+)*).*\((?P<os>[^;]+)").unwrap();
        }

        let mobile = RE_MOBILE.is_match(ua_string);
        let (browser, browser_version, os, os_version) = if let Some(caps) = RE_BROWSER_OS.captures(ua_string) {
            let browser = caps.get(1).map_or("", |m| m.as_str()).to_string();
            let version = caps.name("version").map_or("", |m| m.as_str()).to_string();
            let os_name = caps.name("os").map_or("", |m| m.as_str()).to_string();

            let os_parts: Vec<&str> = os_name.split_whitespace().collect();
            let mut extracted_os = "Other".to_string();
            let mut extracted_os_version = "".to_string();

            if !os_parts.is_empty() {
                extracted_os = match os_parts[0] {
                    "Windows" => "Windows".to_string(),
                    "Macintosh" => "macOS".to_string(),
                    "Linux" => "Linux".to_string(),
                    "Android" => "Android".to_string(),
                    "iPhone" | "iPad" => "iOS".to_string(), // Consider both iPhone and iPad as iOS
                    _ => "Other".to_string(),
                };

                if extracted_os == "Windows" && os_parts.len() >= 3 {
                    extracted_os_version = os_parts[2].to_string();
                } else if (extracted_os == "macOS" || extracted_os == "iOS" || extracted_os == "Android") && os_parts.len() >= 2 {
                    extracted_os_version = os_parts[1].to_string();
                }
            }
            (browser, version, extracted_os, extracted_os_version)
        } else {
            ("Other".to_string(), "0".to_string(), "Other".to_string(), "0".to_string())
        };

        Ok(UserAgent {
            string: ua_string.to_string(),
            browser,
            browser_version,
            os,
            os_version,
            mobile
        })
    }

    /// Returns `true` if the User-Agent represents a mobile device.
    pub fn is_mobile(&self) -> bool {
        self.mobile
    }

    /// Gets a random User-Agent string.
    pub fn get_random_user_agent(mobile: bool) -> Result<Self, LlamaHeadersError> {
        let user_agents = if mobile {
            vec![
                "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.0 Mobile/15E148 Safari/604.1",
                "Mozilla/5.0 (Linux; Android 12; Pixel 6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.87 Mobile Safari/537.36",
                "Mozilla/5.0 (Linux; Android 12; SM-S908B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.0.4896.127 Mobile Safari/537.36",
                "Mozilla/5.0 (iPhone; CPU iPhone OS 15_4 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) CriOS/100.0.4896.85 Mobile/15E148 Safari/604.1",
                "Mozilla/5.0 (iPad; CPU OS 15_4 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.4 Mobile/15E148 Safari/604.1",
            ]
        } else {
            vec![
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.102 Safari/537.36",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.0 Safari/605.1.15",
                "Mozilla/5.0 (X11; Linux x86_64; rv:97.0) Gecko/20100101 Firefox/97.0",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.0.4896.127 Safari/537.36",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.0.4896.127 Safari/537.36",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:99.0) Gecko/20100101 Firefox/99.0",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:99.0) Gecko/20100101 Firefox/99.0",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.0.4896.127 Safari/537.36 Edg/100.0.1185.44",
            ]
        };

        let mut rng = thread_rng();
        let ua_string = user_agents.choose(&mut rng).ok_or(LlamaHeadersError::NoUserAgentAvailable)?.to_string();
        UserAgent::parse(&ua_string)
    }
    
    /// Returns the platform string suitable for the `Sec-CH-UA-Platform` header.
    pub fn get_platform_for_sec_ch_ua(&self) -> String {
        match self.os.as_str() {
            "Windows" => "Windows".to_string(),
            "macOS" => "macOS".to_string(),
            "Linux" => "Linux".to_string(),
            "Android" => "Android".to_string(),
            "iOS" => "iOS".to_string(),
            _ => "Unknown".to_string(),
        }
    }
    
    /// Creates a new UserAgent with a custom browser version
    pub fn with_browser_version(mut self, version: &str) -> Self {
        self.browser_version = version.to_string();
        
        // Update the full UA string to reflect the change
        let updated_ua = self.string.replace(
            &format!("{}/{}", self.browser, self.browser_version),
            &format!("{}/{}", self.browser, version)
        );
        
        self.string = updated_ua;
        self
    }
    
    /// Creates a specific Chrome UserAgent
    pub fn chrome(windows: bool) -> Result<Self, LlamaHeadersError> {
        let base = if windows {
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.0.4896.127 Safari/537.36"
        } else {
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.0.4896.127 Safari/537.36"
        };
        
        UserAgent::parse(base)
    }
    
    /// Creates a specific Firefox UserAgent
    pub fn firefox(windows: bool) -> Result<Self, LlamaHeadersError> {
        let base = if windows {
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:99.0) Gecko/20100101 Firefox/99.0"
        } else {
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:99.0) Gecko/20100101 Firefox/99.0"
        };
        
        UserAgent::parse(base)
    }
    
    /// Creates a specific Safari UserAgent
    pub fn safari() -> Result<Self, LlamaHeadersError> {
        let base = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.0 Safari/605.1.15";
        UserAgent::parse(base)
    }
}

impl std::fmt::Display for UserAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_chrome_windows() {
        let ua_string = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.102 Safari/537.36";
        let ua = UserAgent::parse(ua_string).unwrap();
        
        assert_eq!(ua.browser, "Chrome");
        assert_eq!(ua.browser_version, "98.0.4758.102");
        assert_eq!(ua.os, "Windows");
        assert!(!ua.is_mobile());
    }
    
    #[test]
    fn test_parse_safari_mac() {
        let ua_string = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.0 Safari/605.1.15";
        let ua = UserAgent::parse(ua_string).unwrap();
        
        assert_eq!(ua.browser, "Safari");
        assert_eq!(ua.os, "macOS");
    }
    
    #[test]
    fn test_is_mobile() {
        let mobile_ua = "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.0 Mobile/15E148 Safari/604.1";
        let ua = UserAgent::parse(mobile_ua).unwrap();
        
        assert!(ua.is_mobile());
        assert_eq!(ua.os, "iOS");
    }
    
    #[test]
    fn test_get_random_user_agent() {
        let ua = UserAgent::get_random_user_agent(false).unwrap();
        assert!(!ua.is_mobile());
        
        let mobile_ua = UserAgent::get_random_user_agent(true).unwrap();
        assert!(mobile_ua.is_mobile());
    }
    
    #[test]
    fn test_with_browser_version() {
        let ua = UserAgent::chrome(true).unwrap()
            .with_browser_version("99.0.1234.56");
        
        assert_eq!(ua.browser_version, "99.0.1234.56");
        assert!(ua.string.contains("Chrome/99.0.1234.56"));
    }
    
    #[test]
    fn test_browser_factory_methods() {
        let chrome = UserAgent::chrome(true).unwrap();
        assert_eq!(chrome.browser, "Chrome");
        assert_eq!(chrome.os, "Windows");
        
        let firefox = UserAgent::firefox(false).unwrap();
        assert_eq!(firefox.browser, "Firefox");
        assert_eq!(firefox.os, "macOS");
        
        let safari = UserAgent::safari().unwrap();
        assert_eq!(safari.browser, "Safari");
        assert_eq!(safari.os, "macOS");
    }
} 