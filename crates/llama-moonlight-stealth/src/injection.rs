//! JavaScript script injection
//!
//! This module provides capabilities to inject JavaScript scripts into browser pages
//! for various stealth and automation purposes.

use std::collections::HashMap;
use std::sync::Arc;

use crate::Result;
use crate::Error;

/// Script type for injection
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ScriptType {
    /// Stealth script to avoid detection
    Stealth,
    
    /// Utility script with helper functions
    Utility,
    
    /// Event handling script
    Event,
    
    /// Data extraction script
    Extraction,
    
    /// User interface manipulation script
    UI,
    
    /// Custom script
    Custom(String),
}

/// Script to inject into a page
#[derive(Debug, Clone)]
pub struct Script {
    /// Unique identifier for the script
    pub id: String,
    
    /// Type of the script
    pub script_type: ScriptType,
    
    /// JavaScript code for the script
    pub code: String,
    
    /// Whether to inject automatically when a page loads
    pub auto_inject: bool,
    
    /// Script metadata
    pub metadata: HashMap<String, String>,
}

impl Script {
    /// Create a new script
    pub fn new(id: &str, script_type: ScriptType, code: &str) -> Self {
        Self {
            id: id.to_string(),
            script_type,
            code: code.to_string(),
            auto_inject: false,
            metadata: HashMap::new(),
        }
    }
    
    /// Set whether the script should be auto-injected
    pub fn with_auto_inject(mut self, auto_inject: bool) -> Self {
        self.auto_inject = auto_inject;
        self
    }
    
    /// Add metadata to the script
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Add multiple metadata entries to the script
    pub fn with_metadata_map(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata.extend(metadata);
        self
    }
    
    /// Minify the script code
    pub fn minify(&mut self) {
        // Simple minification for example purposes
        self.code = self.code
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.starts_with("//") && !line.is_empty())
            .collect::<Vec<&str>>()
            .join(" ")
            .replace("{ ", "{")
            .replace(" }", "}")
            .replace("; ", ";")
            .replace(" = ", "=")
            .replace("=  ", "=")
            .replace("  =", "=");
    }
    
    /// Wrap the script in an IIFE (Immediately Invoked Function Expression)
    pub fn as_iife(&self) -> String {
        format!("(function() {{\n{}\n}})();", self.code)
    }
}

/// Manager for script injection
#[derive(Debug, Default)]
pub struct InjectionManager {
    /// Scripts to inject
    scripts: HashMap<String, Script>,
}

impl InjectionManager {
    /// Create a new injection manager
    pub fn new() -> Self {
        Self {
            scripts: HashMap::new(),
        }
    }
    
    /// Register a script
    pub fn register_script(&mut self, script: Script) -> &mut Self {
        self.scripts.insert(script.id.clone(), script);
        self
    }
    
    /// Get a script by ID
    pub fn get_script(&self, id: &str) -> Option<&Script> {
        self.scripts.get(id)
    }
    
    /// Get a mutable reference to a script by ID
    pub fn get_script_mut(&mut self, id: &str) -> Option<&mut Script> {
        self.scripts.get_mut(id)
    }
    
    /// Remove a script
    pub fn remove_script(&mut self, id: &str) -> Option<Script> {
        self.scripts.remove(id)
    }
    
    /// Get all scripts of a specific type
    pub fn get_scripts_by_type(&self, script_type: &ScriptType) -> Vec<&Script> {
        self.scripts
            .values()
            .filter(|s| &s.script_type == script_type)
            .collect()
    }
    
    /// Get all scripts that should be auto-injected
    pub fn get_auto_inject_scripts(&self) -> Vec<&Script> {
        self.scripts
            .values()
            .filter(|s| s.auto_inject)
            .collect()
    }
    
    /// Combine all scripts of a type into a single script
    pub fn combine_scripts(&self, script_type: &ScriptType) -> String {
        let scripts = self.get_scripts_by_type(script_type);
        
        let combined = scripts
            .iter()
            .map(|s| format!("// {}\n{}", s.id, s.code))
            .collect::<Vec<_>>()
            .join("\n\n");
        
        format!("// Combined {} scripts\n(function() {{\n{}\n}})();", 
            match script_type {
                ScriptType::Stealth => "stealth",
                ScriptType::Utility => "utility",
                ScriptType::Event => "event",
                ScriptType::Extraction => "extraction",
                ScriptType::UI => "UI",
                ScriptType::Custom(name) => name,
            },
            combined
        )
    }
    
    /// Create standard stealth scripts
    pub fn create_stealth_scripts() -> Vec<Script> {
        vec![
            // WebDriver detection evasion
            Script::new(
                "webdriver_disable",
                ScriptType::Stealth,
                r#"
                // Hide the WebDriver property
                Object.defineProperty(navigator, 'webdriver', {
                    get: () => false
                });
                "#,
            ).with_auto_inject(true),
            
            // Automation properties evasion
            Script::new(
                "automation_properties",
                ScriptType::Stealth,
                r#"
                // Hide Chrome's automation property
                if (window.chrome) {
                    Object.defineProperty(window.chrome, 'automation', {
                        get: () => undefined
                    });
                }
                
                // Clean up other automation-related properties
                const cleanProperties = [
                    'webdriver', '_selenium', '__webdriver_script_fn', 
                    '__driver_evaluate', '__webdriver_evaluate', '__selenium_evaluate', 
                    '__selenium_unwrapped', '__webdriver_script_function', '__webdriver_script_func'
                ];
                
                cleanProperties.forEach(prop => {
                    if (window[prop]) {
                        delete window[prop];
                    }
                });
                
                // Clean up document.$cdc_ property (ChromeDriver)
                Object.keys(document).forEach(key => {
                    if (key.startsWith('$cdc_') || key.startsWith('$chrome_')) {
                        delete document[key];
                    }
                });
                "#,
            ).with_auto_inject(true),
            
            // Permission handling
            Script::new(
                "permission_handler",
                ScriptType::Utility,
                r#"
                // Mock permission API to always return granted for common permissions
                if (navigator.permissions) {
                    const originalQuery = navigator.permissions.query;
                    navigator.permissions.query = function(parameters) {
                        const name = parameters.name || '';
                        if (name.toLowerCase().includes('notifications') || 
                            name.toLowerCase().includes('push') || 
                            name.toLowerCase().includes('midi') || 
                            name.toLowerCase().includes('camera') || 
                            name.toLowerCase().includes('microphone') || 
                            name.toLowerCase().includes('geolocation')) {
                            return Promise.resolve({ state: 'granted', onchange: null });
                        }
                        return originalQuery.call(navigator.permissions, parameters);
                    };
                }
                "#,
            ).with_auto_inject(false),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_script_creation() {
        let script = Script::new(
            "test_script",
            ScriptType::Stealth,
            "console.log('Hello, world!');"
        );
        
        assert_eq!(script.id, "test_script");
        assert_eq!(script.script_type, ScriptType::Stealth);
        assert_eq!(script.code, "console.log('Hello, world!');");
        assert_eq!(script.auto_inject, false);
    }
    
    #[test]
    fn test_script_minification() {
        let mut script = Script::new(
            "test_script",
            ScriptType::Stealth,
            r#"
            // This is a comment
            function test() {
                console.log('Hello, world!');
                return true;
            }
            "#
        );
        
        script.minify();
        
        // Check that comments are removed and whitespace is reduced
        assert!(!script.code.contains("// This is a comment"));
        assert!(script.code.contains("function test(){"));
    }
    
    #[test]
    fn test_injection_manager() {
        let mut manager = InjectionManager::new();
        
        // Register a script
        manager.register_script(Script::new(
            "test_script",
            ScriptType::Stealth,
            "console.log('Hello, world!');"
        ));
        
        // Check that the script was registered
        let script = manager.get_script("test_script");
        assert!(script.is_some());
        assert_eq!(script.unwrap().id, "test_script");
        
        // Check scripts by type
        let stealth_scripts = manager.get_scripts_by_type(&ScriptType::Stealth);
        assert_eq!(stealth_scripts.len(), 1);
        
        // Check auto-inject scripts (none should be auto-inject)
        let auto_scripts = manager.get_auto_inject_scripts();
        assert_eq!(auto_scripts.len(), 0);
        
        // Add an auto-inject script
        manager.register_script(Script::new(
            "auto_script",
            ScriptType::Stealth,
            "console.log('Auto injected');"
        ).with_auto_inject(true));
        
        // Check auto-inject scripts again
        let auto_scripts = manager.get_auto_inject_scripts();
        assert_eq!(auto_scripts.len(), 1);
    }
    
    #[test]
    fn test_create_stealth_scripts() {
        let scripts = InjectionManager::create_stealth_scripts();
        
        // Check that we have at least one stealth script
        assert!(!scripts.is_empty());
        
        // Check that all scripts are of type Stealth
        for script in &scripts {
            assert_eq!(script.script_type, ScriptType::Stealth);
        }
    }
} 