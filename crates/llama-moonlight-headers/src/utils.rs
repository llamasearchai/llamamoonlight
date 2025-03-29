use rand::Rng;
use std::collections::HashMap;
use crate::HeaderError;

/// Generate a random UUID string
pub fn random_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Generate a random alphanumeric string of the given length
pub fn random_string(length: usize) -> String {
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();
    let mut result = String::with_capacity(length);
    
    for _ in 0..length {
        let idx = rng.gen_range(0..CHARSET.len());
        result.push(CHARSET[idx] as char);
    }
    
    result
}

/// Generate a random hexadecimal string of the given length
pub fn random_hex(length: usize) -> String {
    const CHARSET: &[u8] = b"0123456789abcdef";
    let mut rng = rand::thread_rng();
    let mut result = String::with_capacity(length);
    
    for _ in 0..length {
        let idx = rng.gen_range(0..CHARSET.len());
        result.push(CHARSET[idx] as char);
    }
    
    result
}

/// Calculate the MD5 hash of a string
pub fn md5_hash(input: &str) -> String {
    format!("{:x}", md5::compute(input))
}

/// Parse headers from a string (e.g., from curl -v output)
pub fn parse_headers(header_str: &str) -> Result<HashMap<String, String>, HeaderError> {
    let mut headers = HashMap::new();
    
    for line in header_str.lines() {
        let line = line.trim();
        if line.is_empty() || !line.contains(':') {
            continue;
        }
        
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() != 2 {
            continue;
        }
        
        let key = parts[0].trim();
        let value = parts[1].trim();
        
        headers.insert(key.to_string(), value.to_string());
    }
    
    Ok(headers)
}

/// Format headers as a string
pub fn format_headers(headers: &HashMap<String, String>) -> String {
    headers.iter()
        .map(|(name, value)| format!("{}: {}", name, value))
        .collect::<Vec<String>>()
        .join("\r\n")
}

/// Format headers as a curl command
pub fn headers_to_curl(headers: &HashMap<String, String>, url: &str) -> String {
    let mut command = format!("curl -X GET \"{}\"", url);
    
    for (name, value) in headers {
        command.push_str(&format!(" \\\n    -H \"{}: {}\"", name, value));
    }
    
    command
}

/// Format headers as key-value pairs for JavaScript (fetch API)
pub fn headers_to_js(headers: &HashMap<String, String>) -> String {
    let headers_str = headers.iter()
        .map(|(name, value)| format!("    \"{}\": \"{}\"", name, value))
        .collect::<Vec<String>>()
        .join(",\n");
    
    format!("const headers = {{\n{}\n}};", headers_str)
}

/// Extract cookies from headers to a cookie jar
pub fn extract_cookies(headers: &HashMap<String, String>) -> HashMap<String, String> {
    let mut cookies = HashMap::new();
    
    if let Some(cookie_header) = headers.get("Set-Cookie") {
        for cookie_str in cookie_header.split(';') {
            if let Some(pos) = cookie_str.find('=') {
                let (name, value) = cookie_str.split_at(pos);
                let value = &value[1..]; // Skip the '='
                cookies.insert(name.trim().to_string(), value.trim().to_string());
            }
        }
    }
    
    cookies
}

/// Format cookies as a string for a Cookie header
pub fn format_cookies(cookies: &HashMap<String, String>) -> String {
    cookies.iter()
        .map(|(name, value)| format!("{}={}", name, value))
        .collect::<Vec<String>>()
        .join("; ")
}

/// Generate a RFC-compliant Date header
pub fn generate_date_header() -> String {
    let now = chrono::Utc::now();
    now.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}

/// Parse a URL string
pub fn parse_url(url: &str) -> Result<url::Url, HeaderError> {
    url::Url::parse(url)
        .map_err(|e| HeaderError::Other(format!("Failed to parse URL: {}", e)))
}

/// Get the domain from a URL
pub fn get_domain(url: &str) -> Result<String, HeaderError> {
    let parsed_url = parse_url(url)?;
    parsed_url.host_str()
        .map(|s| s.to_string())
        .ok_or_else(|| HeaderError::Other("Failed to extract domain".to_string()))
}

/// Get the path from a URL
pub fn get_path(url: &str) -> Result<String, HeaderError> {
    let parsed_url = parse_url(url)?;
    Ok(parsed_url.path().to_string())
}

/// Get the scheme from a URL
pub fn get_scheme(url: &str) -> Result<String, HeaderError> {
    let parsed_url = parse_url(url)?;
    Ok(parsed_url.scheme().to_string())
}

/// Get the port from a URL
pub fn get_port(url: &str) -> Result<Option<u16>, HeaderError> {
    let parsed_url = parse_url(url)?;
    Ok(parsed_url.port())
}

/// Check if a URL is HTTPS
pub fn is_https(url: &str) -> Result<bool, HeaderError> {
    let scheme = get_scheme(url)?;
    Ok(scheme == "https")
}

/// Compute a base64 encoded string
pub fn base64_encode(input: &[u8]) -> String {
    base64::encode(input)
}

/// Decode a base64 encoded string
pub fn base64_decode(input: &str) -> Result<Vec<u8>, HeaderError> {
    base64::decode(input)
        .map_err(|e| HeaderError::Other(format!("Failed to decode base64: {}", e)))
}

/// Compute a percentage of a value
pub fn percentage(value: f64, percent: f64) -> f64 {
    value * (percent / 100.0)
}

/// Get a random element from a slice
pub fn random_element<T: Copy>(slice: &[T]) -> Option<T> {
    if slice.is_empty() {
        return None;
    }
    
    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0..slice.len());
    Some(slice[idx])
}

/// Get a random element from a slice with a specific seed
pub fn seeded_random_element<T: Copy>(slice: &[T], seed: u64) -> Option<T> {
    if slice.is_empty() {
        return None;
    }
    
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let idx = rng.gen_range(0..slice.len());
    Some(slice[idx])
}

/// Generate a random IP address
pub fn random_ip() -> String {
    let mut rng = rand::thread_rng();
    format!(
        "{}.{}.{}.{}",
        rng.gen_range(1..255),
        rng.gen_range(0..255),
        rng.gen_range(0..255),
        rng.gen_range(1..255)
    )
}

/// Generate a random localhost port
pub fn random_port() -> u16 {
    let mut rng = rand::thread_rng();
    rng.gen_range(1025..65535)
}

/// Sanitize a string for safe usage in headers
pub fn sanitize_header_value(value: &str) -> String {
    value.trim()
        .replace('\n', " ")
        .replace('\r', " ")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_random_uuid() {
        let uuid = random_uuid();
        assert_eq!(uuid.len(), 36);
        assert_eq!(uuid.chars().filter(|&c| c == '-').count(), 4);
    }
    
    #[test]
    fn test_random_string() {
        let str1 = random_string(10);
        let str2 = random_string(10);
        
        assert_eq!(str1.len(), 10);
        assert_eq!(str2.len(), 10);
        // Very unlikely that two random strings are equal
        assert_ne!(str1, str2);
    }
    
    #[test]
    fn test_md5_hash() {
        let hash = md5_hash("test");
        assert_eq!(hash, "098f6bcd4621d373cade4e832627b4f6");
    }
    
    #[test]
    fn test_parse_headers() {
        let header_str = "User-Agent: Mozilla/5.0\nAccept: text/html\nContent-Type: application/json";
        let headers = parse_headers(header_str).unwrap();
        
        assert_eq!(headers.len(), 3);
        assert_eq!(headers.get("User-Agent"), Some(&"Mozilla/5.0".to_string()));
        assert_eq!(headers.get("Accept"), Some(&"text/html".to_string()));
        assert_eq!(headers.get("Content-Type"), Some(&"application/json".to_string()));
    }
    
    #[test]
    fn test_format_headers() {
        let mut headers = HashMap::new();
        headers.insert("User-Agent".to_string(), "Mozilla/5.0".to_string());
        headers.insert("Accept".to_string(), "text/html".to_string());
        
        let formatted = format_headers(&headers);
        
        // Since HashMap iteration order is not guaranteed, we can't check exact string
        assert!(formatted.contains("User-Agent: Mozilla/5.0"));
        assert!(formatted.contains("Accept: text/html"));
    }
    
    #[test]
    fn test_headers_to_curl() {
        let mut headers = HashMap::new();
        headers.insert("User-Agent".to_string(), "Mozilla/5.0".to_string());
        headers.insert("Accept".to_string(), "text/html".to_string());
        
        let curl = headers_to_curl(&headers, "https://example.com");
        
        assert!(curl.starts_with("curl -X GET \"https://example.com\""));
        assert!(curl.contains("-H \"User-Agent: Mozilla/5.0\""));
        assert!(curl.contains("-H \"Accept: text/html\""));
    }
    
    #[test]
    fn test_parse_url() {
        let url = parse_url("https://example.com/path?query=1").unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host_str(), Some("example.com"));
        assert_eq!(url.path(), "/path");
        assert_eq!(url.query(), Some("query=1"));
    }
    
    #[test]
    fn test_get_domain() {
        assert_eq!(get_domain("https://example.com/path").unwrap(), "example.com");
        assert_eq!(get_domain("http://sub.example.com").unwrap(), "sub.example.com");
    }
    
    #[test]
    fn test_random_element() {
        let values = [1, 2, 3, 4, 5];
        let random = random_element(&values).unwrap();
        assert!(values.contains(&random));
    }
} 