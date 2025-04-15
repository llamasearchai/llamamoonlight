use crate::{Challenge, ChallengeSolution, ChallengeType, CloudflareConfig, CloudflareError, extract_challenge, is_cloudflare_challenge, is_cloudflare_captcha};
use crate::challenge::solve_challenge;
use crate::cookie::get_cookies_from_response;
use crate::proxy::ProxyManager;
use crate::sessions::Session;
use futures::future::BoxFuture;
use log::{debug, error, info, warn};
use reqwest::{Client, ClientBuilder, Method, Request, RequestBuilder, Response, StatusCode, Url};
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::time::sleep;

/// A client for handling Cloudflare protected websites
pub struct CloudflareClient {
    /// HTTP client
    client: Client,
    /// Configuration
    config: CloudflareConfig,
    /// Proxy manager
    proxy_manager: Option<Arc<Mutex<ProxyManager>>>,
    /// Cookies
    cookies: Arc<Mutex<HashMap<String, String>>>,
    /// Challenge handler
    challenge_handler: Box<dyn Fn(Challenge, &str) -> BoxFuture<'static, Result<ChallengeSolution, CloudflareError>> + Send + Sync>,
    /// Session
    session: Option<Session>,
}

impl CloudflareClient {
    /// Create a new CloudflareClient with default configuration
    pub async fn new() -> Result<Self, CloudflareError> {
        Self::with_config(CloudflareConfig::default()).await
    }
    
    /// Create a new CloudflareClient with custom configuration
    pub async fn with_config(config: CloudflareConfig) -> Result<Self, CloudflareError> {
        let mut client_builder = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .cookie_store(true)
            .redirect(if config.follow_redirects {
                reqwest::redirect::Policy::limited(10)
            } else {
                reqwest::redirect::Policy::none()
            });
            
        // Add proxy if configured
        if let Some(ref proxy_url) = config.proxy {
            let proxy = reqwest::Proxy::all(proxy_url)
                .map_err(|e| CloudflareError::ProxyError(format!("Failed to create proxy: {}", e)))?;
            client_builder = client_builder.proxy(proxy);
        }
        
        // Build client
        let client = client_builder.build()
            .map_err(|e| CloudflareError::HttpError(e))?;
        
        // Initialize cookies
        let cookies = config.cookies.clone().unwrap_or_default();
        
        // Initialize proxy manager if needed
        let proxy_manager = if config.rotate_proxies && config.proxy_list.is_some() {
            let proxy_list = config.proxy_list.clone().unwrap();
            Some(Arc::new(Mutex::new(ProxyManager::new(proxy_list))))
        } else {
            None
        };
        
        // Default challenge handler
        let challenge_handler: Box<dyn Fn(Challenge, &str) -> BoxFuture<'static, Result<ChallengeSolution, CloudflareError>> + Send + Sync> = 
            Box::new(|challenge, domain| {
                Box::pin(async move {
                    solve_challenge(&challenge, domain)
                })
            });
        
        Ok(Self {
            client,
            config,
            proxy_manager,
            cookies: Arc::new(Mutex::new(cookies)),
            challenge_handler,
            session: None,
        })
    }
    
    /// Set a custom challenge handler
    pub fn with_challenge_handler(
        mut self,
        handler: impl Fn(Challenge, &str) -> BoxFuture<'static, Result<ChallengeSolution, CloudflareError>> + Send + Sync + 'static,
    ) -> Self {
        self.challenge_handler = Box::new(handler);
        self
    }
    
    /// Set a session for this client
    pub fn with_session(mut self, session: Session) -> Self {
        self.session = Some(session);
        self
    }
    
    /// Get the client configuration
    pub fn config(&self) -> &CloudflareConfig {
        &self.config
    }
    
    /// Get a mutable reference to the client configuration
    pub fn config_mut(&mut self) -> &mut CloudflareConfig {
        &mut self.config
    }
    
    /// Get the cookies
    pub fn cookies(&self) -> HashMap<String, String> {
        self.cookies.lock().unwrap().clone()
    }
    
    /// Set a cookie
    pub fn set_cookie(&self, name: &str, value: &str) {
        self.cookies.lock().unwrap().insert(name.to_string(), value.to_string());
    }
    
    /// Remove a cookie
    pub fn remove_cookie(&self, name: &str) {
        self.cookies.lock().unwrap().remove(name);
    }
    
    /// Clear all cookies
    pub fn clear_cookies(&self) {
        self.cookies.lock().unwrap().clear();
    }
    
    /// Get a proxy URL
    fn get_proxy(&self) -> Option<String> {
        if let Some(ref proxy_manager) = self.proxy_manager {
            proxy_manager.lock().unwrap().get_proxy()
        } else {
            self.config.proxy.clone()
        }
    }
    
    /// Get default headers for a request
    async fn get_default_headers(&self, url: &str) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        
        // Use llama-headers-rs if stealth mode is enabled
        if self.config.stealth_mode {
            match crate::get_default_bypass_headers(url) {
                Ok(h) => headers = h,
                Err(e) => {
                    warn!("Failed to get default headers: {}", e);
                    // Fall back to basic headers
                    headers.insert("User-Agent".to_string(), self.config.user_agent.clone().unwrap_or_else(|| {
                        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36".to_string()
                    }));
                    headers.insert("Accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8".to_string());
                    headers.insert("Accept-Language".to_string(), "en-US,en;q=0.5".to_string());
                    headers.insert("Accept-Encoding".to_string(), "gzip, deflate, br".to_string());
                    headers.insert("Connection".to_string(), "keep-alive".to_string());
                    headers.insert("Upgrade-Insecure-Requests".to_string(), "1".to_string());
                    headers.insert("Sec-Fetch-Dest".to_string(), "document".to_string());
                    headers.insert("Sec-Fetch-Mode".to_string(), "navigate".to_string());
                    headers.insert("Sec-Fetch-Site".to_string(), "none".to_string());
                    headers.insert("Sec-Fetch-User".to_string(), "?1".to_string());
                    headers.insert("Cache-Control".to_string(), "max-age=0".to_string());
                }
            }
        } else {
            // Basic headers
            headers.insert("User-Agent".to_string(), self.config.user_agent.clone().unwrap_or_else(|| {
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36".to_string()
            }));
            headers.insert("Accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8".to_string());
            headers.insert("Accept-Language".to_string(), "en-US,en;q=0.5".to_string());
            headers.insert("Accept-Encoding".to_string(), "gzip, deflate, br".to_string());
            headers.insert("Connection".to_string(), "keep-alive".to_string());
            headers.insert("Upgrade-Insecure-Requests".to_string(), "1".to_string());
        }
        
        // Add custom headers
        if let Some(ref custom_headers) = self.config.custom_headers {
            for (key, value) in custom_headers {
                headers.insert(key.clone(), value.clone());
            }
        }
        
        headers
    }
    
    /// Create a request
    async fn create_request(&self, method: Method, url: &str) -> Result<RequestBuilder, CloudflareError> {
        // Get headers
        let headers = self.get_default_headers(url).await;
        
        // Create request builder
        let mut builder = self.client.request(method, url);
        
        // Add headers
        for (key, value) in headers {
            builder = builder.header(key, value);
        }
        
        // Add cookies
        let cookies = self.cookies.lock().unwrap();
        if !cookies.is_empty() {
            let cookie_header = cookies.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<String>>()
                .join("; ");
            
            builder = builder.header("Cookie", cookie_header);
        }
        
        // Add proxy if configured
        if let Some(proxy_url) = self.get_proxy() {
            let proxy = reqwest::Proxy::all(&proxy_url)
                .map_err(|e| CloudflareError::ProxyError(format!("Failed to create proxy: {}", e)))?;
            builder = builder.proxy(proxy);
        }
        
        Ok(builder)
    }
    
    /// Send a request and handle Cloudflare challenges
    pub async fn send(&self, request: RequestBuilder) -> Result<Response, CloudflareError> {
        let mut retries = 0;
        let max_retries = self.config.max_retries;
        
        // Get the URL from the request
        let request = request.build().map_err(|e| CloudflareError::HttpError(e))?;
        let url = request.url().to_string();
        let host = request.url().host_str().unwrap_or("");
        
        // Create a new request each time as we can't reuse the original
        let mut current_request = request;
        
        loop {
            // Send the request
            let response = self.client.execute(current_request.try_clone().unwrap())
                .await
                .map_err(|e| CloudflareError::HttpError(e))?;
            
            // Check response status
            let status = response.status();
            
            // Check for Cloudflare challenges
            if is_cloudflare_challenge(&response) {
                info!("Cloudflare challenge detected for {}", url);
                
                if retries >= max_retries {
                    error!("Max retries reached for {}", url);
                    return Err(CloudflareError::ChallengeDetected(format!("Max retries reached for {}", url)));
                }
                
                // Extract challenge
                let challenge = extract_challenge(&response)?;
                
                // Get the domain from the URL
                let domain = Url::parse(&url)
                    .map_err(|e| CloudflareError::Other(format!("Failed to parse URL: {}", e)))?
                    .host_str()
                    .ok_or_else(|| CloudflareError::Other("No host in URL".to_string()))?
                    .to_string();
                
                // Solve challenge
                let solution = (self.challenge_handler)(challenge, &domain).await?;
                
                // Apply cookies from the solution
                for (name, value) in &solution.cookies {
                    self.set_cookie(name, value);
                }
                
                // Wait a bit to not trigger anti-bot measures
                sleep(Duration::from_millis(1000 + rand::random::<u64>() % 1000)).await;
                
                // Create a new request for the solution
                let mut solution_request = self.create_request(Method::GET, &solution.submit_url).await?;
                
                // Add parameters
                for (key, value) in &solution.params {
                    solution_request = solution_request.query(&[(key, value)]);
                }
                
                // Update current request
                current_request = solution_request.build().map_err(|e| CloudflareError::HttpError(e))?;
                
                retries += 1;
                continue;
            }
            
            // Check for CAPTCHA
            if is_cloudflare_captcha(&response) {
                if !self.config.solve_captchas {
                    return Err(CloudflareError::CaptchaRequired("CAPTCHA required but solving is disabled".to_string()));
                }
                
                // CAPTCHAs are not implemented in this basic version
                return Err(CloudflareError::CaptchaRequired("CAPTCHA solving not implemented".to_string()));
            }
            
            // Extract cookies from the response
            let response_cookies = get_cookies_from_response(&response);
            for (name, value) in response_cookies {
                self.set_cookie(&name, &value);
            }
            
            // Check for success
            if status.is_success() {
                return Ok(response);
            }
            
            // Check for redirects
            if status.is_redirection() && self.config.follow_redirects {
                if let Some(location) = response.headers().get("location") {
                    if let Ok(location_str) = location.to_str() {
                        let next_url = if location_str.starts_with("http") {
                            location_str.to_string()
                        } else {
                            format!("https://{}{}", host, location_str)
                        };
                        
                        debug!("Following redirect to {}", next_url);
                        
                        // Create a new request for the location
                        current_request = self.create_request(Method::GET, &next_url).await?
                            .build()
                            .map_err(|e| CloudflareError::HttpError(e))?;
                        
                        continue;
                    }
                }
            }
            
            // Return the response for other status codes
            return Ok(response);
        }
    }
    
    /// Send a GET request
    pub async fn get(&self, url: &str) -> Result<Response, CloudflareError> {
        let request = self.create_request(Method::GET, url).await?;
        self.send(request).await
    }
    
    /// Send a POST request
    pub async fn post<T: Serialize + ?Sized>(&self, url: &str, json: &T) -> Result<Response, CloudflareError> {
        let request = self.create_request(Method::POST, url).await?.json(json);
        self.send(request).await
    }
    
    /// Send a POST request with form data
    pub async fn post_form<T: Serialize + ?Sized>(&self, url: &str, form: &T) -> Result<Response, CloudflareError> {
        let request = self.create_request(Method::POST, url).await?.form(form);
        self.send(request).await
    }
    
    /// Create a new session
    pub fn create_session(&self) -> Session {
        Session::new(self.config.clone(), self.cookies.lock().unwrap().clone())
    }
} 