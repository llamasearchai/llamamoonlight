use std::time::Duration;
use std::collections::HashMap;
use url::Url;

use crate::VERSION;

/// Configuration options for the finance client
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// User agent for HTTP requests
    pub user_agent: String,
    
    /// Request timeout
    pub timeout: Duration,
    
    /// Whether to automatically retry failed requests
    pub auto_retry: bool,
    
    /// Maximum number of retries
    pub max_retries: u32,
    
    /// Delay between retries (in milliseconds)
    pub retry_delay: u32,
    
    /// Whether to enable rate limiting
    pub rate_limiting: bool,
    
    /// Whether to cache API responses
    pub cache_responses: bool,
    
    /// Cache expiration time
    pub cache_expiration: Duration,
    
    /// Default API keys for various providers
    pub api_keys: HashMap<String, String>,
    
    /// Custom HTTP headers
    pub custom_headers: HashMap<String, String>,
    
    /// Proxy configuration
    pub proxy: Option<ProxyConfig>,
    
    /// Whether to enable verbose logging
    pub verbose_logging: bool,
    
    /// Default currency for price data
    pub default_currency: String,
    
    /// Whether to automatically adjust for splits and dividends
    pub adjust_prices: bool,
    
    /// Whether to include extended hours data
    pub include_extended_hours: bool,
    
    /// Stealth mode configuration (if enabled)
    #[cfg(feature = "stealth")]
    pub stealth_config: Option<crate::stealth::StealthConfig>,
    
    /// Tor configuration (if enabled)
    #[cfg(feature = "tor")]
    pub tor_config: Option<crate::tor::TorConfig>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            user_agent: format!("llama-moonlight-finance/{} (https://github.com/llamamoonlight/llama-ecosystem)", VERSION),
            timeout: Duration::from_secs(30),
            auto_retry: true,
            max_retries: 3,
            retry_delay: 1000,
            rate_limiting: true,
            cache_responses: true,
            cache_expiration: Duration::from_secs(300), // 5 minutes
            api_keys: HashMap::new(),
            custom_headers: HashMap::new(),
            proxy: None,
            verbose_logging: false,
            default_currency: "USD".to_string(),
            adjust_prices: true,
            include_extended_hours: false,
            #[cfg(feature = "stealth")]
            stealth_config: None,
            #[cfg(feature = "tor")]
            tor_config: None,
        }
    }
}

impl ClientConfig {
    /// Create a new client configuration
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set the user agent
    pub fn with_user_agent(mut self, user_agent: &str) -> Self {
        self.user_agent = user_agent.to_string();
        self
    }
    
    /// Set the request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    
    /// Set the auto retry option
    pub fn with_auto_retry(mut self, auto_retry: bool) -> Self {
        self.auto_retry = auto_retry;
        self
    }
    
    /// Set the maximum number of retries
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }
    
    /// Set the retry delay
    pub fn with_retry_delay(mut self, retry_delay: u32) -> Self {
        self.retry_delay = retry_delay;
        self
    }
    
    /// Set the rate limiting option
    pub fn with_rate_limiting(mut self, rate_limiting: bool) -> Self {
        self.rate_limiting = rate_limiting;
        self
    }
    
    /// Set the cache responses option
    pub fn with_cache_responses(mut self, cache_responses: bool) -> Self {
        self.cache_responses = cache_responses;
        self
    }
    
    /// Set the cache expiration time
    pub fn with_cache_expiration(mut self, cache_expiration: Duration) -> Self {
        self.cache_expiration = cache_expiration;
        self
    }
    
    /// Add an API key for a provider
    pub fn with_api_key(mut self, provider: &str, api_key: &str) -> Self {
        self.api_keys.insert(provider.to_string(), api_key.to_string());
        self
    }
    
    /// Add multiple API keys
    pub fn with_api_keys(mut self, api_keys: HashMap<String, String>) -> Self {
        self.api_keys.extend(api_keys);
        self
    }
    
    /// Add a custom HTTP header
    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        self.custom_headers.insert(name.to_string(), value.to_string());
        self
    }
    
    /// Set the proxy configuration
    pub fn with_proxy(mut self, proxy: ProxyConfig) -> Self {
        self.proxy = Some(proxy);
        self
    }
    
    /// Set the verbose logging option
    pub fn with_verbose_logging(mut self, verbose_logging: bool) -> Self {
        self.verbose_logging = verbose_logging;
        self
    }
    
    /// Set the default currency
    pub fn with_default_currency(mut self, currency: &str) -> Self {
        self.default_currency = currency.to_string();
        self
    }
    
    /// Set the adjust prices option
    pub fn with_adjust_prices(mut self, adjust_prices: bool) -> Self {
        self.adjust_prices = adjust_prices;
        self
    }
    
    /// Set the include extended hours option
    pub fn with_include_extended_hours(mut self, include_extended_hours: bool) -> Self {
        self.include_extended_hours = include_extended_hours;
        self
    }
    
    /// Set the stealth configuration
    #[cfg(feature = "stealth")]
    pub fn with_stealth(mut self, stealth_config: crate::stealth::StealthConfig) -> Self {
        self.stealth_config = Some(stealth_config);
        self
    }
    
    /// Set the Tor configuration
    #[cfg(feature = "tor")]
    pub fn with_tor(mut self, tor_config: crate::tor::TorConfig) -> Self {
        self.tor_config = Some(tor_config);
        self
    }
    
    /// Get an API key for a provider
    pub fn api_key(&self, provider: &str) -> Option<&String> {
        self.api_keys.get(provider)
    }
    
    /// Get the proxy URL as a reqwest proxy
    pub fn reqwest_proxy(&self) -> Option<reqwest::Proxy> {
        if let Some(proxy_config) = &self.proxy {
            return proxy_config.to_reqwest_proxy().ok();
        }
        None
    }
}

/// Proxy configuration for HTTP requests
#[derive(Debug, Clone)]
pub struct ProxyConfig {
    /// Proxy type
    pub proxy_type: ProxyType,
    
    /// Proxy host
    pub host: String,
    
    /// Proxy port
    pub port: u16,
    
    /// Username for authentication
    pub username: Option<String>,
    
    /// Password for authentication
    pub password: Option<String>,
}

impl ProxyConfig {
    /// Create a new HTTP proxy configuration
    pub fn http(host: &str, port: u16) -> Self {
        Self {
            proxy_type: ProxyType::Http,
            host: host.to_string(),
            port,
            username: None,
            password: None,
        }
    }
    
    /// Create a new HTTPS proxy configuration
    pub fn https(host: &str, port: u16) -> Self {
        Self {
            proxy_type: ProxyType::Https,
            host: host.to_string(),
            port,
            username: None,
            password: None,
        }
    }
    
    /// Create a new SOCKS5 proxy configuration
    pub fn socks5(host: &str, port: u16) -> Self {
        Self {
            proxy_type: ProxyType::Socks5,
            host: host.to_string(),
            port,
            username: None,
            password: None,
        }
    }
    
    /// Set authentication credentials
    pub fn with_auth(mut self, username: &str, password: &str) -> Self {
        self.username = Some(username.to_string());
        self.password = Some(password.to_string());
        self
    }
    
    /// Convert to a reqwest proxy
    pub fn to_reqwest_proxy(&self) -> Result<reqwest::Proxy, url::ParseError> {
        let scheme = match self.proxy_type {
            ProxyType::Http => "http",
            ProxyType::Https => "https",
            ProxyType::Socks5 => "socks5",
        };
        
        let mut url_str = format!("{}://{}:{}", scheme, self.host, self.port);
        let proxy = reqwest::Proxy::all(&url_str)?;
        
        // Add authentication if provided
        if let (Some(username), Some(password)) = (&self.username, &self.password) {
            Ok(proxy.basic_auth(username, password))
        } else {
            Ok(proxy)
        }
    }
    
    /// Get the proxy URL as a string
    pub fn to_url_string(&self) -> String {
        let scheme = match self.proxy_type {
            ProxyType::Http => "http",
            ProxyType::Https => "https",
            ProxyType::Socks5 => "socks5",
        };
        
        if let (Some(username), Some(password)) = (&self.username, &self.password) {
            format!("{}://{}:{}@{}:{}", scheme, username, password, self.host, self.port)
        } else {
            format!("{}://{}:{}", scheme, self.host, self.port)
        }
    }
}

/// Types of proxies supported
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProxyType {
    /// HTTP proxy
    Http,
    
    /// HTTPS proxy
    Https,
    
    /// SOCKS5 proxy
    Socks5,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = ClientConfig::default();
        
        assert!(config.user_agent.contains("llama-moonlight-finance"));
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert!(config.auto_retry);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.default_currency, "USD");
    }
    
    #[test]
    fn test_config_builder() {
        let config = ClientConfig::new()
            .with_user_agent("test-agent")
            .with_timeout(Duration::from_secs(60))
            .with_auto_retry(false)
            .with_max_retries(5)
            .with_api_key("yahoo", "test-key")
            .with_default_currency("EUR")
            .with_verbose_logging(true);
            
        assert_eq!(config.user_agent, "test-agent");
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert!(!config.auto_retry);
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.api_key("yahoo"), Some(&"test-key".to_string()));
        assert_eq!(config.default_currency, "EUR");
        assert!(config.verbose_logging);
    }
    
    #[test]
    fn test_proxy_config() {
        let http_proxy = ProxyConfig::http("proxy.example.com", 8080)
            .with_auth("user", "pass");
            
        assert_eq!(http_proxy.proxy_type, ProxyType::Http);
        assert_eq!(http_proxy.host, "proxy.example.com");
        assert_eq!(http_proxy.port, 8080);
        assert_eq!(http_proxy.username, Some("user".to_string()));
        assert_eq!(http_proxy.password, Some("pass".to_string()));
        
        let url_str = http_proxy.to_url_string();
        assert_eq!(url_str, "http://user:pass@proxy.example.com:8080");
    }
    
    #[test]
    fn test_reqwest_proxy() {
        let socks_proxy = ProxyConfig::socks5("localhost", 9050);
        let proxy = socks_proxy.to_reqwest_proxy().unwrap();
        
        // We can't directly inspect the reqwest::Proxy internals,
        // but we can verify it was created without error
        assert!(proxy.intercept_https_connector(|_| Ok(())).is_ok());
    }
} 