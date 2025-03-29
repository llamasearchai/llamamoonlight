use std::fmt;
use serde::{Deserialize, Serialize};

/// Enumeration of supported platform types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum PlatformType {
    /// Microsoft Windows
    Windows,
    
    /// Apple macOS
    MacOS,
    
    /// Linux-based OS
    Linux,
    
    /// Google Android
    Android,
    
    /// Apple iOS
    IOS,
    
    /// Google Chrome OS
    ChromeOS,
    
    /// Custom platform type
    Custom(String),
}

impl PlatformType {
    /// Get the platform name as a string
    pub fn name(&self) -> String {
        match self {
            PlatformType::Windows => "Windows".to_string(),
            PlatformType::MacOS => "macOS".to_string(),
            PlatformType::Linux => "Linux".to_string(),
            PlatformType::Android => "Android".to_string(),
            PlatformType::IOS => "iOS".to_string(),
            PlatformType::ChromeOS => "Chrome OS".to_string(),
            PlatformType::Custom(name) => name.clone(),
        }
    }
    
    /// Get a boolean indicating if the platform is mobile
    pub fn is_mobile(&self) -> bool {
        matches!(self, PlatformType::Android | PlatformType::IOS)
    }
    
    /// Get the vendor name
    pub fn vendor(&self) -> String {
        match self {
            PlatformType::Windows => "Microsoft Corporation".to_string(),
            PlatformType::MacOS => "Apple Inc.".to_string(),
            PlatformType::Linux => "Various".to_string(),
            PlatformType::Android => "Google LLC".to_string(),
            PlatformType::IOS => "Apple Inc.".to_string(),
            PlatformType::ChromeOS => "Google LLC".to_string(),
            PlatformType::Custom(_) => "Unknown".to_string(),
        }
    }
    
    /// Get the latest version of the platform
    pub fn latest_version(&self) -> String {
        match self {
            PlatformType::Windows => "11".to_string(),
            PlatformType::MacOS => "14.0".to_string(),
            PlatformType::Linux => "6.5".to_string(), // Kernel version
            PlatformType::Android => "14.0".to_string(),
            PlatformType::IOS => "17.0".to_string(),
            PlatformType::ChromeOS => "116.0.5845.161".to_string(),
            PlatformType::Custom(_) => "1.0".to_string(),
        }
    }
    
    /// Get a random version of the platform
    pub fn random_version(&self) -> String {
        use rand::prelude::*;
        
        let mut rng = rand::thread_rng();
        match self {
            PlatformType::Windows => {
                // Select a Windows version
                let versions = [
                    "10.0; Win64; x64",
                    "10.0; WOW64",
                    "11.0; Win64; x64",
                ];
                versions.choose(&mut rng).unwrap().to_string()
            },
            PlatformType::MacOS => {
                // Select a macOS version
                let major = 10 + rng.gen_range(0..5); // 10-14
                let minor = rng.gen_range(0..16);
                if major >= 11 {
                    format!("{}.{}", major, minor)
                } else {
                    format!("{}.{}.{}", major, 15 - minor, rng.gen_range(0..8))
                }
            },
            PlatformType::Linux => {
                // Select a Linux kernel version
                format!("{}.{}.{}-{}", 
                    rng.gen_range(4..7),
                    rng.gen_range(0..20),
                    rng.gen_range(0..150),
                    rng.gen_range(1..50)
                )
            },
            PlatformType::Android => {
                // Select an Android version
                format!("{}.{}.{}", 
                    rng.gen_range(8..15),
                    rng.gen_range(0..4),
                    rng.gen_range(0..5)
                )
            },
            PlatformType::IOS => {
                // Select an iOS version
                format!("{}.{}.{}", 
                    rng.gen_range(13..18),
                    rng.gen_range(0..8),
                    rng.gen_range(0..5)
                )
            },
            PlatformType::ChromeOS => {
                // Select a Chrome OS version
                format!("{}.0.{}.{}", 
                    rng.gen_range(90..118),
                    rng.gen_range(5000..6000),
                    rng.gen_range(100..200)
                )
            },
            PlatformType::Custom(_) => "1.0".to_string(),
        }
    }
    
    /// Parse a platform type from a string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "windows" => Some(PlatformType::Windows),
            "macos" | "osx" | "mac os" | "mac os x" => Some(PlatformType::MacOS),
            "linux" | "ubuntu" | "debian" | "fedora" | "centos" | "arch" => Some(PlatformType::Linux),
            "android" => Some(PlatformType::Android),
            "ios" | "iphone os" => Some(PlatformType::IOS),
            "chromeos" | "chrome os" => Some(PlatformType::ChromeOS),
            _ => Some(PlatformType::Custom(s.to_string())),
        }
    }
    
    /// Get a list of all standard platform types
    pub fn all() -> Vec<PlatformType> {
        vec![
            PlatformType::Windows,
            PlatformType::MacOS,
            PlatformType::Linux,
            PlatformType::Android,
            PlatformType::IOS,
            PlatformType::ChromeOS,
        ]
    }
    
    /// Get a random platform type (excluding custom)
    pub fn random() -> PlatformType {
        use rand::seq::SliceRandom;
        let platforms = PlatformType::all();
        let mut rng = rand::thread_rng();
        platforms.choose(&mut rng).unwrap().clone()
    }
    
    /// Get a random desktop platform type
    pub fn random_desktop() -> PlatformType {
        use rand::seq::SliceRandom;
        let platforms = [
            PlatformType::Windows,
            PlatformType::MacOS,
            PlatformType::Linux,
            PlatformType::ChromeOS,
        ];
        let mut rng = rand::thread_rng();
        platforms.choose(&mut rng).unwrap().clone()
    }
    
    /// Get a random mobile platform type
    pub fn random_mobile() -> PlatformType {
        use rand::seq::SliceRandom;
        let platforms = [
            PlatformType::Android,
            PlatformType::IOS,
        ];
        let mut rng = rand::thread_rng();
        platforms.choose(&mut rng).unwrap().clone()
    }
}

impl fmt::Display for PlatformType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlatformType::Windows => write!(f, "Windows"),
            PlatformType::MacOS => write!(f, "macOS"),
            PlatformType::Linux => write!(f, "Linux"),
            PlatformType::Android => write!(f, "Android"),
            PlatformType::IOS => write!(f, "iOS"),
            PlatformType::ChromeOS => write!(f, "Chrome OS"),
            PlatformType::Custom(name) => write!(f, "{}", name),
        }
    }
}

impl Default for PlatformType {
    fn default() -> Self {
        PlatformType::Windows
    }
}

/// Get an appropriate platform for the given device type
pub fn platform_for_device(device_type: &crate::device::DeviceType) -> PlatformType {
    use crate::device::DeviceType;
    use rand::seq::SliceRandom;
    
    let mut rng = rand::thread_rng();
    
    match device_type {
        DeviceType::Desktop => {
            let platforms = [
                PlatformType::Windows,
                PlatformType::MacOS,
                PlatformType::Linux,
            ];
            platforms.choose(&mut rng).unwrap().clone()
        },
        DeviceType::Mobile => {
            let platforms = [
                PlatformType::Android,
                PlatformType::IOS,
            ];
            platforms.choose(&mut rng).unwrap().clone()
        },
        DeviceType::Tablet => {
            let platforms = [
                PlatformType::Android,
                PlatformType::IOS,
            ];
            platforms.choose(&mut rng).unwrap().clone()
        },
        DeviceType::Console => PlatformType::Custom("Console OS".to_string()),
        DeviceType::TV => {
            let platforms = [
                PlatformType::Android,
                PlatformType::Custom("Tizen".to_string()),
                PlatformType::Custom("webOS".to_string()),
            ];
            platforms.choose(&mut rng).unwrap().clone()
        },
        DeviceType::Watch => {
            let platforms = [
                PlatformType::IOS,
                PlatformType::Android,
                PlatformType::Custom("Tizen".to_string()),
            ];
            platforms.choose(&mut rng).unwrap().clone()
        },
        DeviceType::Custom(_) => PlatformType::random(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::DeviceType;
    
    #[test]
    fn test_platform_type_name() {
        assert_eq!(PlatformType::Windows.name(), "Windows");
        assert_eq!(PlatformType::MacOS.name(), "macOS");
        assert_eq!(PlatformType::Linux.name(), "Linux");
        assert_eq!(PlatformType::Android.name(), "Android");
        assert_eq!(PlatformType::IOS.name(), "iOS");
        assert_eq!(PlatformType::ChromeOS.name(), "Chrome OS");
        assert_eq!(PlatformType::Custom("Test".to_string()).name(), "Test");
    }
    
    #[test]
    fn test_platform_type_is_mobile() {
        assert!(!PlatformType::Windows.is_mobile());
        assert!(!PlatformType::MacOS.is_mobile());
        assert!(!PlatformType::Linux.is_mobile());
        assert!(PlatformType::Android.is_mobile());
        assert!(PlatformType::IOS.is_mobile());
        assert!(!PlatformType::ChromeOS.is_mobile());
    }
    
    #[test]
    fn test_platform_type_from_str() {
        assert_eq!(PlatformType::from_str("windows"), Some(PlatformType::Windows));
        assert_eq!(PlatformType::from_str("Windows"), Some(PlatformType::Windows));
        assert_eq!(PlatformType::from_str("WINDOWS"), Some(PlatformType::Windows));
        assert_eq!(PlatformType::from_str("macos"), Some(PlatformType::MacOS));
        assert_eq!(PlatformType::from_str("linux"), Some(PlatformType::Linux));
        assert_eq!(PlatformType::from_str("android"), Some(PlatformType::Android));
        assert_eq!(PlatformType::from_str("ios"), Some(PlatformType::IOS));
        assert_eq!(PlatformType::from_str("chromeos"), Some(PlatformType::ChromeOS));
        
        if let Some(PlatformType::Custom(name)) = PlatformType::from_str("custom") {
            assert_eq!(name, "custom");
        } else {
            panic!("Expected Custom platform type");
        }
    }
    
    #[test]
    fn test_platform_type_all() {
        let all = PlatformType::all();
        assert_eq!(all.len(), 6);
        assert!(all.contains(&PlatformType::Windows));
        assert!(all.contains(&PlatformType::MacOS));
        assert!(all.contains(&PlatformType::Linux));
        assert!(all.contains(&PlatformType::Android));
        assert!(all.contains(&PlatformType::IOS));
        assert!(all.contains(&PlatformType::ChromeOS));
    }
    
    #[test]
    fn test_platform_for_device() {
        let platform = platform_for_device(&DeviceType::Desktop);
        assert!(matches!(platform, PlatformType::Windows | PlatformType::MacOS | PlatformType::Linux));
        
        let platform = platform_for_device(&DeviceType::Mobile);
        assert!(matches!(platform, PlatformType::Android | PlatformType::IOS));
        
        let platform = platform_for_device(&DeviceType::Tablet);
        assert!(matches!(platform, PlatformType::Android | PlatformType::IOS));
    }
    
    #[test]
    fn test_random_version() {
        let version = PlatformType::Windows.random_version();
        assert!(!version.is_empty());
        
        let version = PlatformType::MacOS.random_version();
        assert!(!version.is_empty());
        
        let version = PlatformType::Android.random_version();
        assert!(!version.is_empty());
    }
} 