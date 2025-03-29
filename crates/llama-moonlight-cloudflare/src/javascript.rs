use crate::CloudflareError;
use quick_js::{Context, JsValue};
use std::{collections::HashMap, sync::Arc, time::Duration};
use lazy_static::lazy_static;

/// A JavaScript evaluator for solving Cloudflare challenges
pub struct JsEvaluator {
    context: Context,
}

impl JsEvaluator {
    /// Create a new JsEvaluator
    pub fn new() -> Result<Self, CloudflareError> {
        let context = Context::new().map_err(|e| {
            CloudflareError::JavaScriptError(format!("Failed to create JavaScript context: {}", e))
        })?;
        
        Ok(Self { context })
    }
    
    /// Evaluate a JavaScript expression and return the result as a string
    pub fn evaluate_to_string(&self, script: &str) -> Result<String, CloudflareError> {
        self.context.eval(script)
            .map_err(|e| CloudflareError::JavaScriptError(format!("Failed to evaluate JavaScript: {}", e)))
            .and_then(|v| match v {
                JsValue::String(s) => Ok(s),
                JsValue::Number(n) => Ok(n.to_string()),
                JsValue::Bool(b) => Ok(b.to_string()),
                JsValue::Null => Ok("null".to_string()),
                JsValue::Undefined => Ok("undefined".to_string()),
                _ => Err(CloudflareError::JavaScriptError("Unexpected return type".to_string())),
            })
    }
    
    /// Evaluate a JavaScript expression and return the result as an i64
    pub fn evaluate_to_int(&self, script: &str) -> Result<i64, CloudflareError> {
        self.context.eval(script)
            .map_err(|e| CloudflareError::JavaScriptError(format!("Failed to evaluate JavaScript: {}", e)))
            .and_then(|v| match v {
                JsValue::Number(n) => Ok(n as i64),
                _ => Err(CloudflareError::JavaScriptError("Expected number return type".to_string())),
            })
    }
    
    /// Evaluate a JavaScript expression and return the result as an f64
    pub fn evaluate_to_float(&self, script: &str) -> Result<f64, CloudflareError> {
        self.context.eval(script)
            .map_err(|e| CloudflareError::JavaScriptError(format!("Failed to evaluate JavaScript: {}", e)))
            .and_then(|v| match v {
                JsValue::Number(n) => Ok(n),
                _ => Err(CloudflareError::JavaScriptError("Expected number return type".to_string())),
            })
    }
    
    /// Evaluate a JavaScript expression and return the result as a boolean
    pub fn evaluate_to_bool(&self, script: &str) -> Result<bool, CloudflareError> {
        self.context.eval(script)
            .map_err(|e| CloudflareError::JavaScriptError(format!("Failed to evaluate JavaScript: {}", e)))
            .and_then(|v| match v {
                JsValue::Bool(b) => Ok(b),
                _ => Err(CloudflareError::JavaScriptError("Expected boolean return type".to_string())),
            })
    }
    
    /// Set a global variable in the JavaScript context
    pub fn set_global(&self, name: &str, value: &JsValue) -> Result<(), CloudflareError> {
        self.context.set_global(name, value)
            .map_err(|e| CloudflareError::JavaScriptError(format!("Failed to set global variable: {}", e)))
    }
    
    /// Call a JavaScript function
    pub fn call_function(&self, func_name: &str, args: &[JsValue]) -> Result<JsValue, CloudflareError> {
        self.context.call_function(func_name, args)
            .map_err(|e| CloudflareError::JavaScriptError(format!("Failed to call function: {}", e)))
    }
    
    /// Execute multiple scripts in the same context
    pub fn execute_scripts(&self, scripts: &[&str]) -> Result<JsValue, CloudflareError> {
        let combined_script = scripts.join("\n");
        self.context.eval(&combined_script)
            .map_err(|e| CloudflareError::JavaScriptError(format!("Failed to execute scripts: {}", e)))
    }
    
    /// Evaluate a Cloudflare IUAM challenge
    pub fn solve_challenge(&self, challenge_script: &str, domain: &str) -> Result<String, CloudflareError> {
        // Set up a browser-like environment
        let setup_script = r#"
            var window = {};
            var document = {
                createElement: function() { return { firstChild: { href: "https://DOMAIN" } }; },
                getElementById: function() { return { appendChild: function() {} }; }
            };
            var location = { href: "https://DOMAIN" };
            var navigator = { userAgent: "Mozilla/5.0" };
        "#.replace("DOMAIN", domain);
        
        // Execute the setup script
        self.context.eval(&setup_script)
            .map_err(|e| CloudflareError::JavaScriptError(format!("Failed to set up JavaScript environment: {}", e)))?;
        
        // Some challenge scripts expect 'a' to be defined
        let challenge_setup = "var a = document.createElement('a');";
        self.context.eval(challenge_setup)
            .map_err(|e| CloudflareError::JavaScriptError(format!("Failed to set up challenge: {}", e)))?;
        
        // Execute the challenge script
        self.context.eval(challenge_script)
            .map_err(|e| CloudflareError::JavaScriptError(format!("Failed to execute challenge script: {}", e)))?;
        
        // Get the answer
        let jschl_answer = self.context.eval("a.value")
            .map_err(|e| CloudflareError::JavaScriptError(format!("Failed to get challenge answer: {}", e)))?;
        
        match jschl_answer {
            JsValue::String(answer) => Ok(answer),
            JsValue::Number(answer) => Ok(answer.to_string()),
            _ => Err(CloudflareError::JavaScriptError("Unexpected answer type".to_string())),
        }
    }
}

/// Create a complete browser-like JavaScript environment
pub fn create_browser_env(domain: &str) -> String {
    format!(r#"
        var window = {{
            innerHeight: 1080,
            innerWidth: 1920,
            location: {{
                href: "https://{}",
                hostname: "{}",
                host: "{}",
                protocol: "https:",
                href: "https://{}/",
                toString: function() {{ return this.href; }}
            }},
            navigator: {{
                userAgent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36",
                appName: "Netscape",
                appVersion: "5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36",
                language: "en-US",
                languages: ["en-US", "en"],
                cookieEnabled: true,
                platform: "Win32",
                userAgentData: {{
                    brands: [
                        {{ brand: "Google Chrome", version: "91" }},
                        {{ brand: "Chromium", version: "91" }}
                    ],
                    mobile: false,
                    platform: "Windows"
                }}
            }},
            Date: Date,
            Math: Math,
            parseInt: parseInt,
            parseFloat: parseFloat,
            isNaN: isNaN,
            isFinite: isFinite,
            btoa: btoa,
            atob: atob,
            setTimeout: setTimeout,
            clearTimeout: clearTimeout,
            setInterval: setInterval,
            clearInterval: clearInterval,
            document: {{
                createElement: function(tagName) {{ 
                    return {{ 
                        tagName: tagName.toUpperCase(),
                        firstChild: {{ href: "https://{}" }},
                        style: {{}},
                        appendChild: function() {{ return null; }}
                    }};
                }},
                getElementById: function(id) {{
                    return {{ 
                        id: id,
                        innerHTML: "",
                        style: {{}},
                        appendChild: function() {{ return null; }}
                    }};
                }},
                querySelector: function() {{ 
                    return {{ 
                        id: "challenge-form",
                        action: "/cdn-cgi/challenge-platform"
                    }};
                }},
                cookie: "",
                title: "{} - Cloudflare",
                referrer: ""
            }}
        }};
        var document = window.document;
        var location = window.location;
        var navigator = window.navigator;
    "#, domain, domain, domain, domain, domain)
}

/// Test if a JavaScript environment works correctly
pub fn test_js_env(js_evaluator: &JsEvaluator) -> Result<bool, CloudflareError> {
    let test_script = r#"
        (function() {
            try {
                var a = document.createElement('a');
                a.href = location.href;
                return a.hostname === location.hostname;
            } catch (e) {
                return false;
            }
        })();
    "#;
    
    js_evaluator.evaluate_to_bool(test_script)
}

/// Extract and decode the Cloudflare payload
pub fn decode_cf_payload(js_evaluator: &JsEvaluator, payload: &str) -> Result<String, CloudflareError> {
    let decode_script = format!(
        r#"
        (function() {{
            try {{
                return atob("{}");
            }} catch (e) {{
                return "Failed to decode: " + e.message;
            }}
        }})();
        "#,
        payload
    );
    
    js_evaluator.evaluate_to_string(&decode_script)
} 