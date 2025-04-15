//! Request and response interception
//!
//! This module provides capabilities to intercept and modify browser requests
//! and responses for stealth purposes.

use std::collections::HashMap;
use std::sync::Arc;

use crate::Result;
use crate::Error;
use crate::evasion::InterceptedRequest;

/// Handler for intercepted requests
pub type RequestHandler = Arc<dyn Fn(&mut InterceptedRequest) -> Result<()> + Send + Sync>;

/// Type of interception pattern
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterceptPattern {
    /// Pattern matches a URL
    Url(String),
    
    /// Pattern matches a resource type
    ResourceType(String),
    
    /// Pattern matches both URL and resource type
    Combined(String, String),
}

impl InterceptPattern {
    /// Create a new pattern for URL matching
    pub fn url(pattern: &str) -> Self {
        Self::Url(pattern.to_string())
    }
    
    /// Create a new pattern for resource type matching
    pub fn resource_type(resource_type: &str) -> Self {
        Self::ResourceType(resource_type.to_string())
    }
    
    /// Create a new pattern for combined URL and resource type matching
    pub fn combined(url_pattern: &str, resource_type: &str) -> Self {
        Self::Combined(url_pattern.to_string(), resource_type.to_string())
    }
}

/// Interception rule
#[derive(Debug)]
pub struct InterceptRule {
    /// Pattern to match requests
    pub pattern: InterceptPattern,
    
    /// Handler for matched requests
    pub handler: RequestHandler,
    
    /// Priority of the rule (higher numbers are processed later)
    pub priority: i32,
}

impl InterceptRule {
    /// Create a new interception rule
    pub fn new<F>(pattern: InterceptPattern, handler: F) -> Self
    where
        F: Fn(&mut InterceptedRequest) -> Result<()> + 'static + Send + Sync,
    {
        Self {
            pattern,
            handler: Arc::new(handler),
            priority: 0,
        }
    }
    
    /// Set the priority of the rule
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

/// Manager for request and response interception
#[derive(Debug)]
pub struct InterceptManager {
    /// Rules for interception
    rules: Vec<InterceptRule>,
}

impl InterceptManager {
    /// Create a new interception manager
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
        }
    }
    
    /// Add a rule to the manager
    pub fn add_rule(&mut self, rule: InterceptRule) -> &mut Self {
        self.rules.push(rule);
        // Sort rules by priority (highest first)
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        self
    }
    
    /// Process an intercepted request through all matching rules
    pub fn process_request(&self, request: &mut InterceptedRequest) -> Result<()> {
        for rule in &self.rules {
            let matches = match &rule.pattern {
                InterceptPattern::Url(url_pattern) => {
                    request.url.contains(url_pattern)
                },
                InterceptPattern::ResourceType(resource_type) => {
                    request.headers.get("Content-Type")
                        .map(|ct| ct.contains(resource_type))
                        .unwrap_or(false)
                },
                InterceptPattern::Combined(url_pattern, resource_type) => {
                    request.url.contains(url_pattern) &&
                    request.headers.get("Content-Type")
                        .map(|ct| ct.contains(resource_type))
                        .unwrap_or(false)
                },
            };
            
            if matches {
                (rule.handler)(request)?;
                
                // If the request was aborted, stop processing further rules
                if request.abort {
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    /// Create a standard set of stealth intercept rules
    pub fn create_stealth_rules() -> Vec<InterceptRule> {
        vec![
            // Block common fingerprinting scripts
            InterceptRule::new(
                InterceptPattern::Url("fingerprint"),
                |req| {
                    req.abort = true;
                    Ok(())
                }
            ).with_priority(100),
            
            // Modify canvas fingerprinting requests
            InterceptRule::new(
                InterceptPattern::Url("canvas"),
                |req| {
                    if let Some(headers) = &mut req.new_headers {
                        headers.insert("Cache-Control".to_string(), "no-cache".to_string());
                    } else {
                        let mut headers = HashMap::new();
                        headers.insert("Cache-Control".to_string(), "no-cache".to_string());
                        req.new_headers = Some(headers);
                    }
                    req.continue_with_modifications = true;
                    Ok(())
                }
            ),
            
            // Modify WebGL fingerprinting requests
            InterceptRule::new(
                InterceptPattern::combined("gl", "application/json"),
                |req| {
                    if let Some(headers) = &mut req.new_headers {
                        headers.insert("Accept".to_string(), "application/json".to_string());
                    }
                    req.continue_with_modifications = true;
                    Ok(())
                }
            ),
        ]
    }
}

impl Default for InterceptManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evasion::InterceptedRequest;
    
    fn create_test_request() -> InterceptedRequest {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "text/html".to_string());
        
        InterceptedRequest {
            url: "https://example.com/test".to_string(),
            method: "GET".to_string(),
            headers,
            body: None,
            abort: false,
            continue_with_modifications: false,
            new_url: None,
            new_headers: None,
            new_body: None,
        }
    }
    
    #[test]
    fn test_intercept_rule() {
        let rule = InterceptRule::new(
            InterceptPattern::Url("example.com"),
            |req| {
                req.abort = true;
                Ok(())
            }
        );
        
        assert_eq!(rule.priority, 0);
        
        let rule = rule.with_priority(10);
        assert_eq!(rule.priority, 10);
    }
    
    #[test]
    fn test_intercept_manager() {
        let mut manager = InterceptManager::new();
        
        // Add a rule to abort requests to example.com
        manager.add_rule(InterceptRule::new(
            InterceptPattern::Url("example.com"),
            |req| {
                req.abort = true;
                Ok(())
            }
        ));
        
        // Process a request
        let mut request = create_test_request();
        manager.process_request(&mut request).unwrap();
        
        // The request should be aborted
        assert!(request.abort);
    }
    
    #[test]
    fn test_create_stealth_rules() {
        let rules = InterceptManager::create_stealth_rules();
        
        // Check that we have at least one rule
        assert!(!rules.is_empty());
        
        // Check that the fingerprint rule has the highest priority
        let highest_priority = rules.iter().map(|r| r.priority).max().unwrap();
        let fingerprint_rule = rules.iter().find(|r| matches!(&r.pattern, InterceptPattern::Url(p) if p == "fingerprint")).unwrap();
        assert_eq!(fingerprint_rule.priority, highest_priority);
    }
} 