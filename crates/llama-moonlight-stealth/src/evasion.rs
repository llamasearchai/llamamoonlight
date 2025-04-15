//! Evasion techniques for stealth browser automation
//!
//! This module provides various techniques to evade bot detection systems.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use crate::Result;
use crate::Error;
use llama_moonlight_headers::{BrowserType, DeviceType, PlatformType};

/// Priority level for evasion techniques
pub type Priority = u8;

/// Function signature for applying an evasion technique
pub type EvasionFn = Arc<dyn Fn(&mut dyn StealthTarget) -> Result<()> + Send + Sync>;

/// Target for applying stealth techniques
pub trait StealthTarget: Debug {
    /// Execute JavaScript in the browser
    fn execute_script(&mut self, script: &str) -> Result<String>;
    
    /// Get the browser type
    fn browser_type(&self) -> BrowserType;
    
    /// Get the device type
    fn device_type(&self) -> DeviceType;
    
    /// Get the platform type
    fn platform_type(&self) -> PlatformType;
    
    /// Set a header for future requests
    fn set_header(&mut self, name: &str, value: &str) -> Result<()>;
    
    /// Get the value of a header
    fn get_header(&self, name: &str) -> Option<String>;
    
    /// Remove a header
    fn remove_header(&mut self, name: &str) -> Result<()>;
    
    /// Intercept requests matching a pattern
    fn intercept_requests(&mut self, pattern: &str, handler: InterceptHandler) -> Result<()>;
    
    /// Set a cookie
    fn set_cookie(&mut self, name: &str, value: &str, domain: &str) -> Result<()>;
}

/// Handler for intercepted requests
pub type InterceptHandler = Arc<dyn Fn(&mut InterceptedRequest) -> Result<()> + Send + Sync>;

/// Intercepted request information
#[derive(Debug)]
pub struct InterceptedRequest {
    /// URL of the request
    pub url: String,
    
    /// Method of the request
    pub method: String,
    
    /// Headers of the request
    pub headers: HashMap<String, String>,
    
    /// Body of the request
    pub body: Option<Vec<u8>>,
    
    /// Whether to abort the request
    pub abort: bool,
    
    /// Whether to continue with the modified request
    pub continue_with_modifications: bool,
    
    /// New URL, if modified
    pub new_url: Option<String>,
    
    /// New headers, if modified
    pub new_headers: Option<HashMap<String, String>>,
    
    /// New body, if modified
    pub new_body: Option<Vec<u8>>,
}

/// An evasion technique
#[derive(Debug)]
pub struct EvasionTechnique {
    /// Name of the evasion technique
    name: String,
    
    /// Description of what the evasion technique does
    description: String,
    
    /// Priority of the evasion technique (higher numbers are applied later)
    priority: Priority,
    
    /// Function to apply the evasion
    apply_fn: EvasionFn,
    
    /// Whether the evasion is enabled
    enabled: bool,
    
    /// JavaScript code for this evasion (if applicable)
    js_code: Option<String>,
}

impl EvasionTechnique {
    /// Create a new evasion technique
    pub fn new<F>(name: &str, description: &str, priority: Priority, apply_fn: F) -> Self
    where
        F: Fn(&mut dyn StealthTarget) -> Result<()> + 'static + Send + Sync,
    {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            priority,
            apply_fn: Arc::new(apply_fn),
            enabled: true,
            js_code: None,
        }
    }
    
    /// Create a new evasion technique with JavaScript code
    pub fn with_js<F>(name: &str, description: &str, priority: Priority, apply_fn: F, js_code: &str) -> Self
    where
        F: Fn(&mut dyn StealthTarget) -> Result<()> + 'static + Send + Sync,
    {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            priority,
            apply_fn: Arc::new(apply_fn),
            enabled: true,
            js_code: Some(js_code.to_string()),
        }
    }
    
    /// Get the name of the evasion technique
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get the description of the evasion technique
    pub fn description(&self) -> &str {
        &self.description
    }
    
    /// Get the priority of the evasion technique
    pub fn priority(&self) -> Priority {
        self.priority
    }
    
    /// Check if the evasion technique is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Enable the evasion technique
    pub fn enable(&mut self) {
        self.enabled = true;
    }
    
    /// Disable the evasion technique
    pub fn disable(&mut self) {
        self.enabled = false;
    }
    
    /// Get the JavaScript code for this evasion (if applicable)
    pub fn js_code(&self) -> Option<&str> {
        self.js_code.as_deref()
    }
    
    /// Apply the evasion technique
    pub fn apply(&self, target: &mut dyn StealthTarget) -> Result<()> {
        if self.enabled {
            (self.apply_fn)(target)
        } else {
            Ok(())
        }
    }
}

/// Manager for evasion techniques
#[derive(Debug, Default)]
pub struct EvasionManager {
    /// Collection of evasion techniques
    evasions: Vec<EvasionTechnique>,
}

impl EvasionManager {
    /// Create a new evasion manager
    pub fn new() -> Self {
        Self {
            evasions: Vec::new(),
        }
    }
    
    /// Register an evasion technique
    pub fn register(&mut self, evasion: EvasionTechnique) -> &mut Self {
        self.evasions.push(evasion);
        self
    }
    
    /// Get an evasion technique by name
    pub fn get(&self, name: &str) -> Option<&EvasionTechnique> {
        self.evasions.iter().find(|e| e.name() == name)
    }
    
    /// Get a mutable reference to an evasion technique by name
    pub fn get_mut(&mut self, name: &str) -> Option<&mut EvasionTechnique> {
        self.evasions.iter_mut().find(|e| e.name() == name)
    }
    
    /// Apply all enabled evasion techniques to a target
    pub fn apply_all(&self, target: &mut dyn StealthTarget) -> Result<()> {
        // Sort by priority
        let mut evasions = self.evasions.clone();
        evasions.sort_by_key(|e| e.priority());
        
        // Apply all enabled evasions
        for evasion in evasions.iter().filter(|e| e.is_enabled()) {
            evasion.apply(target)?;
        }
        
        Ok(())
    }
    
    /// Enable an evasion technique
    pub fn enable(&mut self, name: &str) -> Result<()> {
        match self.get_mut(name) {
            Some(evasion) => {
                evasion.enable();
                Ok(())
            },
            None => Err(Error::Other(format!("Evasion technique not found: {}", name))),
        }
    }
    
    /// Disable an evasion technique
    pub fn disable(&mut self, name: &str) -> Result<()> {
        match self.get_mut(name) {
            Some(evasion) => {
                evasion.disable();
                Ok(())
            },
            None => Err(Error::Other(format!("Evasion technique not found: {}", name))),
        }
    }
    
    /// Get a list of all evasion technique names
    pub fn list_names(&self) -> Vec<String> {
        self.evasions.iter().map(|e| e.name().to_string()).collect()
    }
    
    /// Get a list of enabled evasion technique names
    pub fn list_enabled(&self) -> Vec<String> {
        self.evasions.iter()
            .filter(|e| e.is_enabled())
            .map(|e| e.name().to_string())
            .collect()
    }
    
    /// Create a standard set of evasion techniques
    pub fn standard_evasions() -> Self {
        let mut manager = Self::new();
        
        // Webdriver
        manager.register(EvasionTechnique::with_js(
            "webdriver_disable",
            "Hide the navigator.webdriver property",
            10,
            |target| {
                target.execute_script(r#"
                    Object.defineProperty(navigator, 'webdriver', {
                        get: () => false
                    });
                "#)?;
                Ok(())
            },
            r#"
                Object.defineProperty(navigator, 'webdriver', {
                    get: () => false
                });
            "#
        ));
        
        // Plugins
        manager.register(EvasionTechnique::with_js(
            "plugins_spoof",
            "Add fake browser plugins",
            20,
            |target| {
                target.execute_script(r#"
                    (() => {
                        const makePlugin = (name, filename, description, suffixes) => {
                            const plugin = { name, description, filename };
                            plugin.__proto__ = Plugin.prototype;
                            plugin.length = suffixes.length;
                            suffixes.forEach((suffix, i) => {
                                const mimeType = { 
                                    type: `application/${suffix}`, 
                                    suffixes: suffix,
                                    description: `${name} format`
                                };
                                mimeType.__proto__ = MimeType.prototype;
                                plugin[i] = mimeType;
                            });
                            return plugin;
                        };
                        
                        const plugins = [
                            makePlugin('PDF Viewer', 'internal-pdf-viewer', 'Portable Document Format', ['pdf']),
                            makePlugin('Chrome PDF Viewer', 'chrome-pdf-viewer', 'Portable Document Format', ['pdf']),
                            makePlugin('Chromium PDF Viewer', 'chromium-pdf-viewer', 'Portable Document Format', ['pdf']),
                            makePlugin('Microsoft Edge PDF Viewer', 'edge-pdf-viewer', 'Portable Document Format', ['pdf']),
                            makePlugin('WebKit built-in PDF', 'webkit-pdf-viewer', 'Portable Document Format', ['pdf']),
                        ];
                        
                        // Define plugins property
                        Object.defineProperty(navigator, 'plugins', {
                            get: () => {
                                const pluginArray = Array.from(plugins);
                                pluginArray.__proto__ = PluginArray.prototype;
                                return pluginArray;
                            },
                        });
                    })();
                "#)?;
                Ok(())
            },
            r#"
                (() => {
                    const makePlugin = (name, filename, description, suffixes) => {
                        const plugin = { name, description, filename };
                        plugin.__proto__ = Plugin.prototype;
                        plugin.length = suffixes.length;
                        suffixes.forEach((suffix, i) => {
                            const mimeType = { 
                                type: `application/${suffix}`, 
                                suffixes: suffix,
                                description: `${name} format`
                            };
                            mimeType.__proto__ = MimeType.prototype;
                            plugin[i] = mimeType;
                        });
                        return plugin;
                    };
                    
                    const plugins = [
                        makePlugin('PDF Viewer', 'internal-pdf-viewer', 'Portable Document Format', ['pdf']),
                        makePlugin('Chrome PDF Viewer', 'chrome-pdf-viewer', 'Portable Document Format', ['pdf']),
                        makePlugin('Chromium PDF Viewer', 'chromium-pdf-viewer', 'Portable Document Format', ['pdf']),
                        makePlugin('Microsoft Edge PDF Viewer', 'edge-pdf-viewer', 'Portable Document Format', ['pdf']),
                        makePlugin('WebKit built-in PDF', 'webkit-pdf-viewer', 'Portable Document Format', ['pdf']),
                    ];
                    
                    // Define plugins property
                    Object.defineProperty(navigator, 'plugins', {
                        get: () => {
                            const pluginArray = Array.from(plugins);
                            pluginArray.__proto__ = PluginArray.prototype;
                            return pluginArray;
                        },
                    });
                })();
            "#
        ));
        
        // Canvas fingerprinting protection
        manager.register(EvasionTechnique::with_js(
            "canvas_protection",
            "Protect against canvas fingerprinting",
            30,
            |target| {
                target.execute_script(r#"
                    (() => {
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
                        
                        const originalToDataURL = HTMLCanvasElement.prototype.toDataURL;
                        HTMLCanvasElement.prototype.toDataURL = function(type, quality) {
                            // For tiny canvases (used for fingerprinting), add some noise
                            if (this.width <= 16 && this.height <= 16) {
                                const ctx = this.getContext('2d');
                                if (ctx) {
                                    ctx.fillStyle = `rgba(${Math.floor(Math.random() * 2)}, ${Math.floor(Math.random() * 2)}, ${Math.floor(Math.random() * 2)}, 0.01)`;
                                    ctx.fillRect(0, 0, 1, 1);
                                }
                            }
                            
                            return originalToDataURL.call(this, type, quality);
                        };
                    })();
                "#)?;
                Ok(())
            },
            r#"
                (() => {
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
                    
                    const originalToDataURL = HTMLCanvasElement.prototype.toDataURL;
                    HTMLCanvasElement.prototype.toDataURL = function(type, quality) {
                        // For tiny canvases (used for fingerprinting), add some noise
                        if (this.width <= 16 && this.height <= 16) {
                            const ctx = this.getContext('2d');
                            if (ctx) {
                                ctx.fillStyle = `rgba(${Math.floor(Math.random() * 2)}, ${Math.floor(Math.random() * 2)}, ${Math.floor(Math.random() * 2)}, 0.01)`;
                                ctx.fillRect(0, 0, 1, 1);
                            }
                        }
                        
                        return originalToDataURL.call(this, type, quality);
                    };
                })();
            "#
        ));
        
        // WebGL fingerprinting protection
        manager.register(EvasionTechnique::with_js(
            "webgl_protection",
            "Protect against WebGL fingerprinting",
            40,
            |target| {
                target.execute_script(r#"
                    (() => {
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
                    })();
                "#)?;
                Ok(())
            },
            r#"
                (() => {
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
                })();
            "#
        ));
        
        // Font enumeration protection
        manager.register(EvasionTechnique::with_js(
            "font_enumeration_protection",
            "Protect against font enumeration fingerprinting",
            50,
            |target| {
                target.execute_script(r#"
                    (() => {
                        // Override font measurement methods used to detect installed fonts
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
                    })();
                "#)?;
                Ok(())
            },
            r#"
                (() => {
                    // Override font measurement methods used to detect installed fonts
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
                })();
            "#
        ));
        
        // Stack trace hiding
        manager.register(EvasionTechnique::with_js(
            "stack_trace_hiding",
            "Hide automation markers in error stack traces",
            60,
            |target| {
                target.execute_script(r#"
                    (() => {
                        const originalError = Error;
                        Error = function(message) {
                            const error = new originalError(message);
                            const stackLines = error.stack ? error.stack.split('\n') : [];
                            
                            if (stackLines.length > 0) {
                                error.stack = stackLines[0] + '\n' + 
                                    stackLines.slice(1)
                                        .filter(line => !line.includes('selenium') && 
                                                       !line.includes('webdriver') && 
                                                       !line.includes('driver') &&
                                                       !line.includes('chrome.automation'))
                                        .join('\n');
                            }
                            
                            return error;
                        };
                        
                        Error.prototype = originalError.prototype;
                        
                        // Also cover EvalError, RangeError, ReferenceError, SyntaxError, TypeError, URIError
                        const errorTypes = ['EvalError', 'RangeError', 'ReferenceError', 'SyntaxError', 'TypeError', 'URIError'];
                        
                        errorTypes.forEach(errorType => {
                            const originalType = window[errorType];
                            window[errorType] = function(message) {
                                const error = new originalType(message);
                                if (error.stack) {
                                    const stackLines = error.stack.split('\n');
                                    error.stack = stackLines[0] + '\n' + 
                                        stackLines.slice(1)
                                            .filter(line => !line.includes('selenium') && 
                                                           !line.includes('webdriver') && 
                                                           !line.includes('driver') &&
                                                           !line.includes('chrome.automation'))
                                            .join('\n');
                                }
                                return error;
                            };
                            window[errorType].prototype = originalType.prototype;
                        });
                    })();
                "#)?;
                Ok(())
            },
            r#"
                (() => {
                    const originalError = Error;
                    Error = function(message) {
                        const error = new originalError(message);
                        const stackLines = error.stack ? error.stack.split('\n') : [];
                        
                        if (stackLines.length > 0) {
                            error.stack = stackLines[0] + '\n' + 
                                stackLines.slice(1)
                                    .filter(line => !line.includes('selenium') && 
                                                   !line.includes('webdriver') && 
                                                   !line.includes('driver') &&
                                                   !line.includes('chrome.automation'))
                                    .join('\n');
                        }
                        
                        return error;
                    };
                    
                    Error.prototype = originalError.prototype;
                    
                    // Also cover EvalError, RangeError, ReferenceError, SyntaxError, TypeError, URIError
                    const errorTypes = ['EvalError', 'RangeError', 'ReferenceError', 'SyntaxError', 'TypeError', 'URIError'];
                    
                    errorTypes.forEach(errorType => {
                        const originalType = window[errorType];
                        window[errorType] = function(message) {
                            const error = new originalType(message);
                            if (error.stack) {
                                const stackLines = error.stack.split('\n');
                                error.stack = stackLines[0] + '\n' + 
                                    stackLines.slice(1)
                                        .filter(line => !line.includes('selenium') && 
                                                       !line.includes('webdriver') && 
                                                       !line.includes('driver') &&
                                                       !line.includes('chrome.automation'))
                                        .join('\n');
                            }
                            return error;
                        };
                        window[errorType].prototype = originalType.prototype;
                    });
                })();
            "#
        ));
        
        manager
    }
    
    /// Create advanced evasion techniques
    #[cfg(feature = "advanced")]
    pub fn advanced_evasions() -> Self {
        let mut manager = Self::standard_evasions();
        
        // Add advanced evasion techniques here
        
        manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
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
        
        fn intercept_requests(&mut self, _pattern: &str, _handler: InterceptHandler) -> Result<()> {
            Ok(())
        }
        
        fn set_cookie(&mut self, name: &str, value: &str, _domain: &str) -> Result<()> {
            self.headers.insert(format!("Cookie-{}", name), value.to_string());
            Ok(())
        }
    }
    
    #[test]
    fn test_evasion_technique() {
        let evasion = EvasionTechnique::new(
            "test_evasion",
            "Test evasion description",
            10,
            |target| {
                target.set_header("X-Test", "test")?;
                Ok(())
            },
        );
        
        assert_eq!(evasion.name(), "test_evasion");
        assert_eq!(evasion.description(), "Test evasion description");
        assert_eq!(evasion.priority(), 10);
        assert!(evasion.is_enabled());
        assert!(evasion.js_code().is_none());
        
        let mut target = MockStealthTarget::new();
        evasion.apply(&mut target).unwrap();
        
        assert_eq!(target.headers.get("X-Test").unwrap(), "test");
    }
    
    #[test]
    fn test_evasion_manager() {
        let mut manager = EvasionManager::new();
        
        manager.register(EvasionTechnique::new(
            "evasion1",
            "Test evasion 1",
            10,
            |target| {
                target.set_header("X-Evasion1", "applied")?;
                Ok(())
            },
        ));
        
        manager.register(EvasionTechnique::new(
            "evasion2",
            "Test evasion 2",
            20,
            |target| {
                target.set_header("X-Evasion2", "applied")?;
                Ok(())
            },
        ));
        
        assert_eq!(manager.list_names(), vec!["evasion1", "evasion2"]);
        assert_eq!(manager.list_enabled(), vec!["evasion1", "evasion2"]);
        
        let mut target = MockStealthTarget::new();
        manager.apply_all(&mut target).unwrap();
        
        assert_eq!(target.headers.get("X-Evasion1").unwrap(), "applied");
        assert_eq!(target.headers.get("X-Evasion2").unwrap(), "applied");
        
        manager.disable("evasion1").unwrap();
        assert_eq!(manager.list_enabled(), vec!["evasion2"]);
        
        let mut target = MockStealthTarget::new();
        manager.apply_all(&mut target).unwrap();
        
        assert!(target.headers.get("X-Evasion1").is_none());
        assert_eq!(target.headers.get("X-Evasion2").unwrap(), "applied");
    }
    
    #[test]
    fn test_standard_evasions() {
        let manager = EvasionManager::standard_evasions();
        assert!(manager.get("webdriver_disable").is_some());
        assert!(manager.get("canvas_protection").is_some());
        assert!(manager.get("webgl_protection").is_some());
        
        let mut target = MockStealthTarget::new();
        manager.apply_all(&mut target).unwrap();
        
        assert!(!target.executed_scripts.is_empty());
        assert!(target.executed_scripts.iter().any(|s| s.contains("webdriver")));
        assert!(target.executed_scripts.iter().any(|s| s.contains("CanvasRenderingContext2D")));
        assert!(target.executed_scripts.iter().any(|s| s.contains("WebGLRenderingContext")));
    }
} 