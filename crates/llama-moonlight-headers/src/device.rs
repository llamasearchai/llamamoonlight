use std::fmt;
use serde::{Deserialize, Serialize};

/// Enumeration of supported device types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum DeviceType {
    /// Desktop computer
    Desktop,
    
    /// Mobile phone
    Mobile,
    
    /// Tablet device
    Tablet,
    
    /// Game console
    Console,
    
    /// Smart TV
    TV,
    
    /// Smart watch
    Watch,
    
    /// Custom device type
    Custom(String),
}

impl DeviceType {
    /// Get the device type name as a string
    pub fn name(&self) -> String {
        match self {
            DeviceType::Desktop => "Desktop".to_string(),
            DeviceType::Mobile => "Mobile".to_string(),
            DeviceType::Tablet => "Tablet".to_string(),
            DeviceType::Console => "Console".to_string(),
            DeviceType::TV => "TV".to_string(),
            DeviceType::Watch => "Watch".to_string(),
            DeviceType::Custom(name) => name.clone(),
        }
    }
    
    /// Get a boolean indicating if the device is mobile
    pub fn is_mobile(&self) -> bool {
        matches!(self, DeviceType::Mobile | DeviceType::Tablet | DeviceType::Watch)
    }
    
    /// Get the typical screen size range in inches
    pub fn screen_size_range(&self) -> (f32, f32) {
        match self {
            DeviceType::Desktop => (21.0, 32.0),
            DeviceType::Mobile => (4.0, 6.9),
            DeviceType::Tablet => (7.0, 13.0),
            DeviceType::Console => (24.0, 65.0),
            DeviceType::TV => (32.0, 85.0),
            DeviceType::Watch => (1.0, 2.0),
            DeviceType::Custom(_) => (0.0, 100.0),
        }
    }
    
    /// Get the typical screen resolution range (width, height)
    pub fn resolution_range(&self) -> ((u32, u32), (u32, u32)) {
        match self {
            DeviceType::Desktop => ((1366, 768), (3840, 2160)),
            DeviceType::Mobile => ((320, 568), (1440, 3200)),
            DeviceType::Tablet => ((768, 1024), (2732, 2048)),
            DeviceType::Console => ((1280, 720), (3840, 2160)),
            DeviceType::TV => ((1280, 720), (7680, 4320)),
            DeviceType::Watch => ((272, 340), (396, 484)),
            DeviceType::Custom(_) => ((0, 0), (10000, 10000)),
        }
    }
    
    /// Parse a device type from a string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "desktop" => Some(DeviceType::Desktop),
            "mobile" => Some(DeviceType::Mobile),
            "tablet" => Some(DeviceType::Tablet),
            "console" => Some(DeviceType::Console),
            "tv" => Some(DeviceType::TV),
            "watch" => Some(DeviceType::Watch),
            _ => Some(DeviceType::Custom(s.to_string())),
        }
    }
    
    /// Get a list of all standard device types
    pub fn all() -> Vec<DeviceType> {
        vec![
            DeviceType::Desktop,
            DeviceType::Mobile,
            DeviceType::Tablet,
            DeviceType::Console,
            DeviceType::TV,
            DeviceType::Watch,
        ]
    }
    
    /// Get a random device type (excluding custom)
    pub fn random() -> DeviceType {
        use rand::seq::SliceRandom;
        let devices = DeviceType::all();
        let mut rng = rand::thread_rng();
        devices.choose(&mut rng).unwrap().clone()
    }
    
    /// Get a random mobile device type
    pub fn random_mobile() -> DeviceType {
        use rand::seq::SliceRandom;
        let devices = vec![
            DeviceType::Mobile,
            DeviceType::Tablet,
        ];
        let mut rng = rand::thread_rng();
        devices.choose(&mut rng).unwrap().clone()
    }
    
    /// Get a random desktop-like device type
    pub fn random_desktop() -> DeviceType {
        use rand::seq::SliceRandom;
        let devices = vec![
            DeviceType::Desktop,
            DeviceType::Console,
            DeviceType::TV,
        ];
        let mut rng = rand::thread_rng();
        devices.choose(&mut rng).unwrap().clone()
    }
}

impl fmt::Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceType::Desktop => write!(f, "Desktop"),
            DeviceType::Mobile => write!(f, "Mobile"),
            DeviceType::Tablet => write!(f, "Tablet"),
            DeviceType::Console => write!(f, "Console"),
            DeviceType::TV => write!(f, "TV"),
            DeviceType::Watch => write!(f, "Watch"),
            DeviceType::Custom(name) => write!(f, "{}", name),
        }
    }
}

impl Default for DeviceType {
    fn default() -> Self {
        DeviceType::Desktop
    }
}

/// Device brands for generating device models
pub enum DeviceBrand {
    /// Apple devices
    Apple,
    
    /// Samsung devices
    Samsung,
    
    /// Google devices
    Google,
    
    /// Xiaomi devices
    Xiaomi,
    
    /// Huawei devices
    Huawei,
    
    /// OnePlus devices
    OnePlus,
    
    /// Sony devices
    Sony,
    
    /// LG devices
    LG,
    
    /// Motorola devices
    Motorola,
    
    /// Nokia devices
    Nokia,
    
    /// Microsoft devices
    Microsoft,
    
    /// HP devices
    HP,
    
    /// Dell devices
    Dell,
    
    /// Lenovo devices
    Lenovo,
    
    /// Acer devices
    Acer,
    
    /// Asus devices
    Asus,
}

impl DeviceBrand {
    /// Get the brand name as a string
    pub fn name(&self) -> String {
        match self {
            DeviceBrand::Apple => "Apple".to_string(),
            DeviceBrand::Samsung => "Samsung".to_string(),
            DeviceBrand::Google => "Google".to_string(),
            DeviceBrand::Xiaomi => "Xiaomi".to_string(),
            DeviceBrand::Huawei => "Huawei".to_string(),
            DeviceBrand::OnePlus => "OnePlus".to_string(),
            DeviceBrand::Sony => "Sony".to_string(),
            DeviceBrand::LG => "LG".to_string(),
            DeviceBrand::Motorola => "Motorola".to_string(),
            DeviceBrand::Nokia => "Nokia".to_string(),
            DeviceBrand::Microsoft => "Microsoft".to_string(),
            DeviceBrand::HP => "HP".to_string(),
            DeviceBrand::Dell => "Dell".to_string(),
            DeviceBrand::Lenovo => "Lenovo".to_string(),
            DeviceBrand::Acer => "Acer".to_string(),
            DeviceBrand::Asus => "Asus".to_string(),
        }
    }
    
    /// Get a random device brand
    pub fn random() -> DeviceBrand {
        use rand::seq::SliceRandom;
        let brands = [
            DeviceBrand::Apple,
            DeviceBrand::Samsung,
            DeviceBrand::Google,
            DeviceBrand::Xiaomi,
            DeviceBrand::Huawei,
            DeviceBrand::OnePlus,
            DeviceBrand::Sony,
            DeviceBrand::LG,
            DeviceBrand::Motorola,
            DeviceBrand::Nokia,
            DeviceBrand::Microsoft,
            DeviceBrand::HP,
            DeviceBrand::Dell,
            DeviceBrand::Lenovo,
            DeviceBrand::Acer,
            DeviceBrand::Asus,
        ];
        let mut rng = rand::thread_rng();
        brands.choose(&mut rng).unwrap().clone()
    }
    
    /// Get a random mobile brand
    pub fn random_mobile() -> DeviceBrand {
        use rand::seq::SliceRandom;
        let brands = [
            DeviceBrand::Apple,
            DeviceBrand::Samsung,
            DeviceBrand::Google,
            DeviceBrand::Xiaomi,
            DeviceBrand::Huawei,
            DeviceBrand::OnePlus,
            DeviceBrand::Sony,
            DeviceBrand::LG,
            DeviceBrand::Motorola,
            DeviceBrand::Nokia,
        ];
        let mut rng = rand::thread_rng();
        brands.choose(&mut rng).unwrap().clone()
    }
    
    /// Get a random desktop brand
    pub fn random_desktop() -> DeviceBrand {
        use rand::seq::SliceRandom;
        let brands = [
            DeviceBrand::Apple,
            DeviceBrand::Microsoft,
            DeviceBrand::HP,
            DeviceBrand::Dell,
            DeviceBrand::Lenovo,
            DeviceBrand::Acer,
            DeviceBrand::Asus,
        ];
        let mut rng = rand::thread_rng();
        brands.choose(&mut rng).unwrap().clone()
    }
    
    /// Get a random device model for this brand and device type
    pub fn random_model(&self, device_type: &DeviceType) -> String {
        // Simplified model generation
        match (self, device_type) {
            (DeviceBrand::Apple, DeviceType::Mobile) => {
                use rand::seq::SliceRandom;
                let models = ["iPhone 13", "iPhone 14", "iPhone 14 Pro", "iPhone 15", "iPhone 15 Pro"];
                let mut rng = rand::thread_rng();
                models.choose(&mut rng).unwrap().to_string()
            },
            (DeviceBrand::Apple, DeviceType::Tablet) => {
                use rand::seq::SliceRandom;
                let models = ["iPad", "iPad Pro", "iPad Air", "iPad Mini"];
                let mut rng = rand::thread_rng();
                models.choose(&mut rng).unwrap().to_string()
            },
            (DeviceBrand::Apple, DeviceType::Desktop) => {
                use rand::seq::SliceRandom;
                let models = ["MacBook Air", "MacBook Pro", "iMac", "Mac Mini", "Mac Pro"];
                let mut rng = rand::thread_rng();
                models.choose(&mut rng).unwrap().to_string()
            },
            (DeviceBrand::Samsung, DeviceType::Mobile) => {
                use rand::seq::SliceRandom;
                let models = ["Galaxy S22", "Galaxy S23", "Galaxy S23 Ultra", "Galaxy A54", "Galaxy Z Fold4"];
                let mut rng = rand::thread_rng();
                models.choose(&mut rng).unwrap().to_string()
            },
            // Default to a generic model based on brand and device type
            _ => format!("{} {}", self.name(), device_type.name()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_device_type_name() {
        assert_eq!(DeviceType::Desktop.name(), "Desktop");
        assert_eq!(DeviceType::Mobile.name(), "Mobile");
        assert_eq!(DeviceType::Tablet.name(), "Tablet");
        assert_eq!(DeviceType::Console.name(), "Console");
        assert_eq!(DeviceType::TV.name(), "TV");
        assert_eq!(DeviceType::Watch.name(), "Watch");
        assert_eq!(DeviceType::Custom("Test".to_string()).name(), "Test");
    }
    
    #[test]
    fn test_device_type_is_mobile() {
        assert!(!DeviceType::Desktop.is_mobile());
        assert!(DeviceType::Mobile.is_mobile());
        assert!(DeviceType::Tablet.is_mobile());
        assert!(!DeviceType::Console.is_mobile());
        assert!(!DeviceType::TV.is_mobile());
        assert!(DeviceType::Watch.is_mobile());
    }
    
    #[test]
    fn test_device_type_from_str() {
        assert_eq!(DeviceType::from_str("desktop"), Some(DeviceType::Desktop));
        assert_eq!(DeviceType::from_str("Desktop"), Some(DeviceType::Desktop));
        assert_eq!(DeviceType::from_str("DESKTOP"), Some(DeviceType::Desktop));
        assert_eq!(DeviceType::from_str("mobile"), Some(DeviceType::Mobile));
        assert_eq!(DeviceType::from_str("tablet"), Some(DeviceType::Tablet));
        assert_eq!(DeviceType::from_str("console"), Some(DeviceType::Console));
        assert_eq!(DeviceType::from_str("tv"), Some(DeviceType::TV));
        assert_eq!(DeviceType::from_str("watch"), Some(DeviceType::Watch));
        
        if let Some(DeviceType::Custom(name)) = DeviceType::from_str("custom") {
            assert_eq!(name, "custom");
        } else {
            panic!("Expected Custom device type");
        }
    }
    
    #[test]
    fn test_device_type_all() {
        let all = DeviceType::all();
        assert_eq!(all.len(), 6);
        assert!(all.contains(&DeviceType::Desktop));
        assert!(all.contains(&DeviceType::Mobile));
        assert!(all.contains(&DeviceType::Tablet));
        assert!(all.contains(&DeviceType::Console));
        assert!(all.contains(&DeviceType::TV));
        assert!(all.contains(&DeviceType::Watch));
    }
    
    #[test]
    fn test_device_brand_name() {
        assert_eq!(DeviceBrand::Apple.name(), "Apple");
        assert_eq!(DeviceBrand::Samsung.name(), "Samsung");
        assert_eq!(DeviceBrand::Google.name(), "Google");
    }
    
    #[test]
    fn test_device_brand_random_model() {
        let model = DeviceBrand::Apple.random_model(&DeviceType::Mobile);
        assert!(!model.is_empty());
        
        let model = DeviceBrand::Samsung.random_model(&DeviceType::Mobile);
        assert!(!model.is_empty());
    }
} 