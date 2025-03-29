//! Module for handling HTTP headers
use crate::user_agent::UserAgent;
use std::collections::HashMap;


/// Represents a complete set of HTTP headers.
#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    pub user_agent: UserAgent,
    pub headers: HashMap<String, String>,
}

impl Header {
    /// Creates a new `Header` instance.
    pub fn new(user_agent: UserAgent, headers: HashMap<String, String>) -> Self {
        Header { user_agent, headers }
    }

    /// Returns the headers as a `HashMap`.
    pub fn get_map(&self) -> &HashMap<String, String> {
        &self.headers
    }
    
    /// Returns the value associated with a specific header key.
    pub fn get(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }
    
    /// Convert headers to a format suitable for HTTP requests
    pub fn to_request_headers(&self) -> HashMap<String, String> {
        self.headers.clone()
    }
    
    /// Add a custom header
    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Remove a header
    pub fn without_header(mut self, key: &str) -> Self {
        self.headers.remove(key);
        self
    }
}

impl std::fmt::Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (key, value) in &self.headers {
            writeln!(f, "{}: {}", key, value)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user_agent::UserAgent;
    
    #[test]
    fn test_header_creation() {
        let ua = UserAgent::parse("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.102 Safari/537.36").unwrap();
        let mut headers = HashMap::new();
        headers.insert("Host".to_string(), "example.com".to_string());
        headers.insert("User-Agent".to_string(), ua.to_string());
        
        let header = Header::new(ua.clone(), headers);
        
        assert_eq!(header.get("Host"), Some(&"example.com".to_string()));
        assert_eq!(header.get("User-Agent"), Some(&ua.to_string()));
    }
    
    #[test]
    fn test_header_modification() {
        let ua = UserAgent::parse("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.102 Safari/537.36").unwrap();
        let mut headers = HashMap::new();
        headers.insert("Host".to_string(), "example.com".to_string());
        
        let header = Header::new(ua.clone(), headers)
            .with_header("Custom-Header", "Value")
            .without_header("Host");
        
        assert_eq!(header.get("Custom-Header"), Some(&"Value".to_string()));
        assert_eq!(header.get("Host"), None);
    }
} 