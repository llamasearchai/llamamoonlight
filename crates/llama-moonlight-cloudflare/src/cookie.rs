use crate::CloudflareError;
use reqwest::Response;
use regex::Regex;
use std::collections::HashMap;

/// Extract cookies from a response
pub fn get_cookies_from_response(response: &Response) -> HashMap<String, String> {
    let mut cookies = HashMap::new();
    
    if let Some(headers) = response.headers().get_all("set-cookie").iter().next() {
        for header in response.headers().get_all("set-cookie") {
            if let Ok(cookie_str) = header.to_str() {
                if let Some((name, value)) = parse_cookie(cookie_str) {
                    cookies.insert(name, value);
                }
            }
        }
    }
    
    cookies
}

/// Parse a cookie string into its name and value
fn parse_cookie(cookie_str: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = cookie_str.split(';').collect();
    if parts.is_empty() {
        return None;
    }
    
    let name_value: Vec<&str> = parts[0].split('=').collect();
    if name_value.len() < 2 {
        return None;
    }
    
    Some((name_value[0].trim().to_string(), name_value[1..].join("=").to_string()))
}

/// Check if a cookie jar has a Cloudflare clearance cookie
pub fn has_clearance_cookie(cookies: &HashMap<String, String>) -> bool {
    cookies.contains_key("cf_clearance")
}

/// Create a cookie string from a map of cookies
pub fn create_cookie_header(cookies: &HashMap<String, String>) -> String {
    cookies.iter()
        .map(|(name, value)| format!("{}={}", name, value))
        .collect::<Vec<String>>()
        .join("; ")
}

/// Get the Cloudflare clearance cookie
pub fn get_clearance_cookie(cookies: &HashMap<String, String>) -> Option<String> {
    cookies.get("cf_clearance").cloned()
}

/// Get the Cloudflare CFID cookie
pub fn get_cfid_cookie(cookies: &HashMap<String, String>) -> Option<String> {
    cookies.get("__cfduid").cloned()
}

/// Check if a cookie is expired
pub fn is_cookie_expired(cookie_str: &str) -> bool {
    let re = Regex::new(r"expires=([^;]+)").unwrap();
    
    if let Some(caps) = re.captures(cookie_str) {
        if let Some(expires_str) = caps.get(1) {
            if let Ok(expires) = chrono::DateTime::parse_from_rfc2822(expires_str.as_str()) {
                let now = chrono::Utc::now();
                return expires < now;
            }
        }
    }
    
    false
}

/// Get the domain from a cookie string
pub fn get_cookie_domain(cookie_str: &str) -> Option<String> {
    let re = Regex::new(r"domain=([^;]+)").unwrap();
    
    if let Some(caps) = re.captures(cookie_str) {
        if let Some(domain) = caps.get(1) {
            return Some(domain.as_str().to_string());
        }
    }
    
    None
}

/// Get the path from a cookie string
pub fn get_cookie_path(cookie_str: &str) -> Option<String> {
    let re = Regex::new(r"path=([^;]+)").unwrap();
    
    if let Some(caps) = re.captures(cookie_str) {
        if let Some(path) = caps.get(1) {
            return Some(path.as_str().to_string());
        }
    }
    
    None
}

/// Check if a cookie is secure
pub fn is_cookie_secure(cookie_str: &str) -> bool {
    cookie_str.contains("secure")
}

/// Check if a cookie is http only
pub fn is_cookie_http_only(cookie_str: &str) -> bool {
    cookie_str.contains("httponly")
}

/// Save cookies to a file
pub fn save_cookies_to_file(cookies: &HashMap<String, String>, file_path: &str) -> Result<(), CloudflareError> {
    let json = serde_json::to_string_pretty(cookies)
        .map_err(|e| CloudflareError::CookieError(format!("Failed to serialize cookies: {}", e)))?;
    
    std::fs::write(file_path, json)
        .map_err(|e| CloudflareError::CookieError(format!("Failed to write cookies to file: {}", e)))?;
    
    Ok(())
}

/// Load cookies from a file
pub fn load_cookies_from_file(file_path: &str) -> Result<HashMap<String, String>, CloudflareError> {
    let json = std::fs::read_to_string(file_path)
        .map_err(|e| CloudflareError::CookieError(format!("Failed to read cookies from file: {}", e)))?;
    
    let cookies: HashMap<String, String> = serde_json::from_str(&json)
        .map_err(|e| CloudflareError::CookieError(format!("Failed to parse cookies: {}", e)))?;
    
    Ok(cookies)
} 