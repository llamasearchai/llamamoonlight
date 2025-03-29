use std::fmt;
use serde::{Deserialize, Serialize};

/// Enumeration of supported browser types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum BrowserType {
    /// Google Chrome
    Chrome,
    
    /// Mozilla Firefox
    Firefox,
    
    /// Apple Safari
    Safari,
    
    /// Microsoft Edge
    Edge,
    
    /// Opera
    Opera,
    
    /// Custom browser with a name
    Custom(String),
}

impl BrowserType {
    /// Get the browser name as a string
    pub fn name(&self) -> String {
        match self {
            BrowserType::Chrome => "Chrome".to_string(),
            BrowserType::Firefox => "Firefox".to_string(),
            BrowserType::Safari => "Safari".to_string(),
            BrowserType::Edge => "Edge".to_string(),
            BrowserType::Opera => "Opera".to_string(),
            BrowserType::Custom(name) => name.clone(),
        }
    }
    
    /// Get the browser engine
    pub fn engine(&self) -> String {
        match self {
            BrowserType::Chrome | BrowserType::Edge | BrowserType::Opera => "Blink".to_string(),
            BrowserType::Firefox => "Gecko".to_string(),
            BrowserType::Safari => "WebKit".to_string(),
            BrowserType::Custom(_) => "Unknown".to_string(),
        }
    }
    
    /// Get the browser vendor
    pub fn vendor(&self) -> String {
        match self {
            BrowserType::Chrome => "Google Inc.".to_string(),
            BrowserType::Firefox => "Mozilla Foundation".to_string(),
            BrowserType::Safari => "Apple Inc.".to_string(),
            BrowserType::Edge => "Microsoft Corporation".to_string(),
            BrowserType::Opera => "Opera Software".to_string(),
            BrowserType::Custom(_) => "Unknown".to_string(),
        }
    }
    
    /// Get the latest version of the browser
    pub fn latest_version(&self) -> String {
        match self {
            BrowserType::Chrome => "117.0.0.0".to_string(),
            BrowserType::Firefox => "117.0".to_string(),
            BrowserType::Safari => "16.5".to_string(),
            BrowserType::Edge => "117.0.2045.47".to_string(),
            BrowserType::Opera => "101.0.0.0".to_string(),
            BrowserType::Custom(_) => "1.0.0".to_string(),
        }
    }
    
    /// Parse a browser type from a string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "chrome" => Some(BrowserType::Chrome),
            "firefox" => Some(BrowserType::Firefox),
            "safari" => Some(BrowserType::Safari),
            "edge" => Some(BrowserType::Edge),
            "opera" => Some(BrowserType::Opera),
            _ => Some(BrowserType::Custom(s.to_string())),
        }
    }
    
    /// Get a list of all standard browser types
    pub fn all() -> Vec<BrowserType> {
        vec![
            BrowserType::Chrome,
            BrowserType::Firefox,
            BrowserType::Safari,
            BrowserType::Edge,
            BrowserType::Opera,
        ]
    }
    
    /// Get a random browser type (excluding custom)
    pub fn random() -> BrowserType {
        use rand::seq::SliceRandom;
        let browsers = BrowserType::all();
        let mut rng = rand::thread_rng();
        browsers.choose(&mut rng).unwrap().clone()
    }
}

impl fmt::Display for BrowserType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BrowserType::Chrome => write!(f, "Chrome"),
            BrowserType::Firefox => write!(f, "Firefox"),
            BrowserType::Safari => write!(f, "Safari"),
            BrowserType::Edge => write!(f, "Edge"),
            BrowserType::Opera => write!(f, "Opera"),
            BrowserType::Custom(name) => write!(f, "{}", name),
        }
    }
}

impl Default for BrowserType {
    fn default() -> Self {
        BrowserType::Chrome
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_browser_type_name() {
        assert_eq!(BrowserType::Chrome.name(), "Chrome");
        assert_eq!(BrowserType::Firefox.name(), "Firefox");
        assert_eq!(BrowserType::Safari.name(), "Safari");
        assert_eq!(BrowserType::Edge.name(), "Edge");
        assert_eq!(BrowserType::Opera.name(), "Opera");
        assert_eq!(BrowserType::Custom("Test".to_string()).name(), "Test");
    }
    
    #[test]
    fn test_browser_type_engine() {
        assert_eq!(BrowserType::Chrome.engine(), "Blink");
        assert_eq!(BrowserType::Firefox.engine(), "Gecko");
        assert_eq!(BrowserType::Safari.engine(), "WebKit");
        assert_eq!(BrowserType::Edge.engine(), "Blink");
        assert_eq!(BrowserType::Opera.engine(), "Blink");
    }
    
    #[test]
    fn test_browser_type_from_str() {
        assert_eq!(BrowserType::from_str("chrome"), Some(BrowserType::Chrome));
        assert_eq!(BrowserType::from_str("Chrome"), Some(BrowserType::Chrome));
        assert_eq!(BrowserType::from_str("CHROME"), Some(BrowserType::Chrome));
        assert_eq!(BrowserType::from_str("firefox"), Some(BrowserType::Firefox));
        assert_eq!(BrowserType::from_str("safari"), Some(BrowserType::Safari));
        assert_eq!(BrowserType::from_str("edge"), Some(BrowserType::Edge));
        assert_eq!(BrowserType::from_str("opera"), Some(BrowserType::Opera));
        
        if let Some(BrowserType::Custom(name)) = BrowserType::from_str("custom") {
            assert_eq!(name, "custom");
        } else {
            panic!("Expected Custom browser type");
        }
    }
    
    #[test]
    fn test_browser_type_display() {
        assert_eq!(format!("{}", BrowserType::Chrome), "Chrome");
        assert_eq!(format!("{}", BrowserType::Firefox), "Firefox");
        assert_eq!(format!("{}", BrowserType::Safari), "Safari");
        assert_eq!(format!("{}", BrowserType::Edge), "Edge");
        assert_eq!(format!("{}", BrowserType::Opera), "Opera");
        assert_eq!(format!("{}", BrowserType::Custom("Test".to_string())), "Test");
    }
    
    #[test]
    fn test_browser_type_all() {
        let all = BrowserType::all();
        assert_eq!(all.len(), 5);
        assert!(all.contains(&BrowserType::Chrome));
        assert!(all.contains(&BrowserType::Firefox));
        assert!(all.contains(&BrowserType::Safari));
        assert!(all.contains(&BrowserType::Edge));
        assert!(all.contains(&BrowserType::Opera));
    }
    
    #[test]
    fn test_browser_type_random() {
        let random = BrowserType::random();
        assert!(BrowserType::all().contains(&random));
    }
} 