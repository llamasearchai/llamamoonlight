//! Utility functions for header generation
use crate::errors::LlamaHeadersError;
use crate::user_agent::UserAgent;
use rand::seq::SliceRandom;
use rand::thread_rng;
use regex::Regex;
use lazy_static::lazy_static;
use std::collections::HashMap;

/// Extracts the domain from a URL.
pub fn get_domain(url: &str) -> Result<String, LlamaHeadersError> {
    lazy_static! {
        static ref RE_DOMAIN: Regex = Regex::new(r"https?://([^/]+)").unwrap();
    }
    RE_DOMAIN
        .captures(url)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or(LlamaHeadersError::InvalidUrl(url.to_string()))
}

/// Determines a language based on the domain TLD.
pub fn get_language_from_domain(domain: &str) -> String {
    lazy_static! {
        static ref RE_TLD: Regex = Regex::new(r"\.([a-z]{2,})$").unwrap();
    }

    if let Some(caps) = RE_TLD.captures(domain) {
        if let Some(tld) = caps.get(1) {
            match tld.as_str() {
                "de" => return "de-DE".to_string(),
                "fr" => return "fr-FR".to_string(),
                "jp" => return "ja-JP".to_string(),
                "uk" | "gb" => return "en-GB".to_string(), // Handle .uk and .gb
                "es" => return "es-ES".to_string(),
                "it" => return "it-IT".to_string(),
                "nl" => return "nl-NL".to_string(),
                "ru" => return "ru-RU".to_string(),
                "br" => return "pt-BR".to_string(),
                "cn" => return "zh-CN".to_string(),
                "tw" => return "zh-TW".to_string(),
                "kr" => return "ko-KR".to_string(),
                // Add more country codes as needed
                _ => return "en-US".to_string(), // Default to English (US)
            }
        }
    }

    "en-US".to_string()
}

/// Gets a random referer based on language and domain context.
pub fn get_random_referer(language: &str, domain: &str) -> Result<String, LlamaHeadersError> {
    let mut referers = HashMap::new();
    referers.insert("en-US", vec!["https://www.google.com", "https://www.bing.com", "https://duckduckgo.com", "https://www.reddit.com"]);
    referers.insert("en-GB", vec!["https://www.google.co.uk", "https://www.bing.com", "https://duckduckgo.com", "https://www.bbc.co.uk"]);
    referers.insert("de-DE", vec!["https://www.google.de", "https://www.bing.de", "https://duckduckgo.com", "https://www.t-online.de"]);
    referers.insert("fr-FR", vec!["https://www.google.fr", "https://www.bing.fr", "https://duckduckgo.com", "https://www.lemonde.fr"]);
    referers.insert("es-ES", vec!["https://www.google.es", "https://www.bing.es", "https://duckduckgo.com", "https://www.marca.com"]);
    referers.insert("it-IT", vec!["https://www.google.it", "https://www.bing.it", "https://duckduckgo.com", "https://www.repubblica.it"]);
    // Add more language-specific referers

    let referer_list = referers.get(language).ok_or(LlamaHeadersError::NoRefererAvailable)?;

    let mut rng = thread_rng();
    let referer = referer_list.choose(&mut rng).ok_or(LlamaHeadersError::NoRefererAvailable)?;
    Ok(referer.to_string())
}

/// Generates the Sec-CH-UA header value for a given User-Agent.
pub fn get_sec_ch_ua(ua: &UserAgent) -> Option<String> {
    match ua.browser.as_str() {
        "Chrome" => Some(format!("\"Not A(Brand\";v=\"99\", \"{}\";v=\"{}\", \"Chromium\";v=\"{}\"", 
            ua.browser, 
            ua.browser_version.split('.').next().unwrap_or("0"), 
            ua.browser_version.split('.').next().unwrap_or("0"))),
        "Edge" => Some(format!("\"Not A(Brand\";v=\"99\", \"{}\";v=\"{}\", \"Microsoft Edge\";v=\"{}\"", 
            ua.browser, 
            ua.browser_version.split('.').next().unwrap_or("0"), 
            ua.browser_version.split('.').next().unwrap_or("0"))),
        _ => None
    }
}

/// Gets a standard Accept-Encoding header value.
pub fn get_accept_encoding() -> String {
    "gzip, deflate, br".to_string()
}

/// Gets an Accept-Language header based on the specified language.
pub fn get_accept_language(language: &str) -> String {
    format!("{},{};q=0.9", language, language.split('-').next().unwrap_or(""))
}

/// Determines the Sec-Fetch-Site header based on referer and domain.
pub fn get_sec_fetch_site(referer: &str, domain: &str) -> String {
    if referer.contains(domain) {
        "same-origin".to_string()
    } else if referer.starts_with("https://") || referer.starts_with("http://") {
        "cross-site".to_string()
    } else {
        "none".to_string()
    }
}

/// Gets a standard Sec-Fetch-Mode header value.
pub fn get_sec_fetch_mode() -> String {
    "navigate".to_string() // Standard for top-level navigation
}

/// Gets a standard Sec-Fetch-User header value.
pub fn get_sec_fetch_user() -> String {
    "?1".to_string() // Standard for user-initiated requests
}

/// Gets a standard Sec-Fetch-Dest header value.
pub fn get_sec_fetch_dest() -> String {
    "document".to_string() // Standard for document/page loads
}

/// Gets a standard Connection header value.
pub fn get_connection() -> String {
    "keep-alive".to_string() // Standard for persistent connections
}

/// Generates a random viewport dimensions string.
pub fn get_random_viewport() -> String {
    let common_widths = [1366, 1440, 1536, 1680, 1920, 2560];
    let common_heights = [768, 900, 864, 1050, 1080, 1440];
    
    let mut rng = thread_rng();
    let width = common_widths.choose(&mut rng).unwrap_or(&1920);
    let height = common_heights.choose(&mut rng).unwrap_or(&1080);
    
    format!("{}x{}", width, height)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_domain_valid() {
        assert_eq!(get_domain("https://www.example.com/path").unwrap(), "www.example.com");
        assert_eq!(get_domain("http://subdomain.example.co.uk/").unwrap(), "subdomain.example.co.uk");
    }
    
    #[test]
    fn test_get_domain_invalid() {
        assert!(get_domain("invalid-url").is_err());
    }
    
    #[test]
    fn test_get_language_from_domain() {
        assert_eq!(get_language_from_domain("example.de"), "de-DE");
        assert_eq!(get_language_from_domain("example.fr"), "fr-FR");
        assert_eq!(get_language_from_domain("example.co.uk"), "en-GB");
        assert_eq!(get_language_from_domain("example.jp"), "ja-JP");
        
        // Default case
        assert_eq!(get_language_from_domain("example.com"), "en-US");
    }
    
    #[test]
    fn test_get_random_referer() {
        let referer = get_random_referer("en-US", "example.com").unwrap();
        assert!(referer.starts_with("https://"));
        
        let referer_de = get_random_referer("de-DE", "example.de").unwrap();
        assert!(referer_de.contains("google.de") || referer_de.contains("bing.de") || referer_de.contains("duckduckgo.com"));
    }
    
    #[test]
    fn test_sec_ch_ua_generation() {
        let chrome_ua = UserAgent::parse("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.102 Safari/537.36").unwrap();
        let sec_ch_ua = get_sec_ch_ua(&chrome_ua).unwrap();
        assert!(sec_ch_ua.contains("Chrome"));
        assert!(sec_ch_ua.contains("98"));
        
        let firefox_ua = UserAgent::parse("Mozilla/5.0 (X11; Linux x86_64; rv:97.0) Gecko/20100101 Firefox/97.0").unwrap();
        assert!(get_sec_ch_ua(&firefox_ua).is_none());
    }
    
    #[test]
    fn test_sec_fetch_site() {
        // Same origin
        assert_eq!(get_sec_fetch_site("https://example.com/page1", "example.com"), "same-origin");
        
        // Cross site
        assert_eq!(get_sec_fetch_site("https://google.com", "example.com"), "cross-site");
        
        // None (direct navigation)
        assert_eq!(get_sec_fetch_site("", "example.com"), "none");
    }
    
    #[test]
    fn test_random_viewport() {
        let viewport = get_random_viewport();
        assert!(viewport.contains('x'));
        
        let parts: Vec<&str> = viewport.split('x').collect();
        assert_eq!(parts.len(), 2);
        
        let width: i32 = parts[0].parse().unwrap();
        let height: i32 = parts[1].parse().unwrap();
        
        assert!(width >= 800 && width <= 3000);
        assert!(height >= 600 && height <= 2000);
    }
} 