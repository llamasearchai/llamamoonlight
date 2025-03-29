//! Tor client implementation
//!
//! This module provides a high-level client for making HTTP requests through
//! the Tor network, with capabilities for circuit management and stealth options.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use reqwest::{Client, ClientBuilder, Proxy, Method, RequestBuilder, Response, StatusCode};
use serde::{Deserialize, Serialize};
use url::Url;
use tokio::sync::{Mutex, RwLock};

use crate::Result;
use crate::Error;
use crate::TorConfig;
use crate::TorCapable;
use crate::circuit::{CircuitInfo, TorCircuit};
use crate::controller::TorController;
use crate::proxy::TorProxy;
use crate::onion::OnionService;

/// HTTP client that routes traffic through Tor
#[derive(Debug)]
pub struct TorClient {
    /// Inner reqwest client
    client: Arc<Mutex<Client>>,
    
    /// Tor configuration
    config: TorConfig,
    
    /// Tor controller for managing circuits
    controller: Arc<TorController>,
    
    /// Circuit manager
    circuit: Arc<RwLock<TorCircuit>>,
    
    /// SOCKS proxy
    proxy: Arc<TorProxy>,
    
    /// Default request headers
    default_headers: Arc<RwLock<HashMap<String, String>>>,
    
    /// User agent to use for requests
    user_agent: String,
    
    /// Request timeout in seconds
    timeout_secs: u64,
    
    /// Whether to automatically rotate circuits
    auto_rotate_circuits: bool,
    
    /// Number of requests before rotating circuit
    requests_per_circuit: usize,
    
    /// Number of requests made with the current circuit
    request_count: Arc<Mutex<usize>>,
    
    /// Whether to use stealth mode
    stealth_mode: bool,
    
    /// Whether the client is initialized
    initialized: Arc<RwLock<bool>>,
}

impl TorClient {
    /// Create a new Tor client with the given configuration
    pub fn new(config: TorConfig) -> Self {
        let controller = Arc::new(TorController::new(config.clone()));
        let circuit = Arc::new(RwLock::new(TorCircuit::new(controller.clone(), config.clone())));
        let proxy = Arc::new(TorProxy::new(config.clone()));
        
        Self {
            client: Arc::new(Mutex::new(Client::new())),
            config,
            controller,
            circuit,
            proxy,
            default_headers: Arc::new(RwLock::new(HashMap::new())),
            user_agent: "Mozilla/5.0 (Windows NT 10.0; rv:100.0) Gecko/20100101 Firefox/100.0".to_string(),
            timeout_secs: 30,
            auto_rotate_circuits: true,
            requests_per_circuit: 10,
            request_count: Arc::new(Mutex::new(0)),
            stealth_mode: false,
            initialized: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Initialize the Tor client
    pub async fn init(&self) -> Result<()> {
        let mut initialized = self.initialized.write().await;
        if *initialized {
            return Ok(());
        }
        
        // Connect to the Tor controller
        self.controller.connect().await?;
        
        // Initialize the circuit manager
        self.circuit.write().await.init().await?;
        
        // Initialize the proxy
        self.proxy.init().await?;
        
        // Create the HTTP client with the Tor proxy
        let mut client_builder = ClientBuilder::new()
            .user_agent(&self.user_agent)
            .timeout(Duration::from_secs(self.timeout_secs));
        
        // Add the SOCKS proxy
        let proxy_url = format!("socks5://{}:{}", self.config.socks_host, self.config.socks_port);
        let proxy = Proxy::all(&proxy_url)
            .map_err(|e| Error::ProxyError(format!("Failed to create proxy: {}", e)))?;
        client_builder = client_builder.proxy(proxy);
        
        // Build the client
        let client = client_builder
            .build()
            .map_err(|e| Error::Other(format!("Failed to build HTTP client: {}", e)))?;
        
        // Update the client
        let mut client_guard = self.client.lock().await;
        *client_guard = client;
        
        *initialized = true;
        
        Ok(())
    }
    
    /// Set the user agent for requests
    pub fn with_user_agent(mut self, user_agent: &str) -> Self {
        self.user_agent = user_agent.to_string();
        self
    }
    
    /// Set the request timeout
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }
    
    /// Set whether to automatically rotate circuits
    pub fn with_auto_rotate_circuits(mut self, auto_rotate: bool) -> Self {
        self.auto_rotate_circuits = auto_rotate;
        self
    }
    
    /// Set the number of requests before rotating circuit
    pub fn with_requests_per_circuit(mut self, requests: usize) -> Self {
        self.requests_per_circuit = requests;
        self
    }
    
    /// Enable stealth mode
    pub fn with_stealth_mode(mut self, stealth: bool) -> Self {
        self.stealth_mode = stealth;
        self
    }
    
    /// Add a default header to all requests
    pub async fn add_default_header(&self, name: &str, value: &str) -> Result<()> {
        let mut headers = self.default_headers.write().await;
        headers.insert(name.to_string(), value.to_string());
        Ok(())
    }
    
    /// Remove a default header
    pub async fn remove_default_header(&self, name: &str) -> Result<()> {
        let mut headers = self.default_headers.write().await;
        headers.remove(name);
        Ok(())
    }
    
    /// Get the current default headers
    pub async fn get_default_headers(&self) -> HashMap<String, String> {
        let headers = self.default_headers.read().await;
        headers.clone()
    }
    
    /// Get the current circuit information
    pub async fn get_circuit_info(&self) -> Result<Option<CircuitInfo>> {
        let circuit = self.circuit.read().await;
        Ok(circuit.default_circuit_info().cloned())
    }
    
    /// Request a new circuit
    pub async fn new_circuit(&self) -> Result<String> {
        let mut circuit = self.circuit.write().await;
        let circuit_id = circuit.new_identity().await?;
        
        // Reset request count
        let mut count = self.request_count.lock().await;
        *count = 0;
        
        Ok(circuit_id)
    }
    
    /// Get the current IP address as seen from the Internet
    pub async fn get_ip(&self) -> Result<String> {
        // Ensure client is initialized
        self.ensure_initialized().await?;
        
        // Try multiple IP checking services
        let services = [
            "https://api.ipify.org",
            "https://ifconfig.me/ip",
            "https://icanhazip.com",
        ];
        
        for service in &services {
            match self.get(*service).await {
                Ok(response) => {
                    if response.status().is_success() {
                        let ip = response.text().await.map_err(|e| 
                            Error::NetworkError(format!("Failed to read IP response: {}", e))
                        )?;
                        return Ok(ip.trim().to_string());
                    }
                },
                Err(_) => continue, // Try next service
            }
        }
        
        Err(Error::NetworkError("Failed to get IP address from any service".to_string()))
    }
    
    /// Check if the connection is using Tor
    pub async fn is_using_tor(&self) -> Result<bool> {
        // Ensure client is initialized
        self.ensure_initialized().await?;
        
        // Try to access the Tor check service
        let response = match self.get("https://check.torproject.org").await {
            Ok(resp) => resp,
            Err(e) => return Err(Error::NetworkError(format!("Failed to check Tor: {}", e))),
        };
        
        if !response.status().is_success() {
            return Err(Error::NetworkError(format!("Tor check failed with status: {}", response.status())));
        }
        
        let body = response.text().await.map_err(|e| 
            Error::NetworkError(format!("Failed to read Tor check response: {}", e))
        )?;
        
        // Check if the response contains the "Congratulations" message
        Ok(body.contains("Congratulations. This browser is configured to use Tor"))
    }
    
    /// Make a GET request
    pub async fn get(&self, url: &str) -> Result<Response> {
        self.request(Method::GET, url).await
    }
    
    /// Make a POST request
    pub async fn post(&self, url: &str, body: Option<String>) -> Result<Response> {
        let mut req = self.prepare_request(Method::POST, url).await?;
        
        if let Some(body_content) = body {
            req = req.body(body_content);
        }
        
        self.send_request(req).await
    }
    
    /// Make a PUT request
    pub async fn put(&self, url: &str, body: Option<String>) -> Result<Response> {
        let mut req = self.prepare_request(Method::PUT, url).await?;
        
        if let Some(body_content) = body {
            req = req.body(body_content);
        }
        
        self.send_request(req).await
    }
    
    /// Make a DELETE request
    pub async fn delete(&self, url: &str) -> Result<Response> {
        self.request(Method::DELETE, url).await
    }
    
    /// Make a request with the specified method
    pub async fn request(&self, method: Method, url: &str) -> Result<Response> {
        let req = self.prepare_request(method, url).await?;
        self.send_request(req).await
    }
    
    /// Prepare a request for the given method and URL
    async fn prepare_request(&self, method: Method, url: &str) -> Result<RequestBuilder> {
        // Ensure client is initialized
        self.ensure_initialized().await?;
        
        // Check if we need to rotate circuit
        if self.auto_rotate_circuits {
            let mut count = self.request_count.lock().await;
            if *count >= self.requests_per_circuit {
                // Reset the count
                *count = 0;
                
                // Drop the lock before creating a new circuit
                drop(count);
                
                // Create a new circuit
                self.new_circuit().await?;
            } else {
                // Increment the count
                *count += 1;
            }
        }
        
        // Get the client
        let client = self.client.lock().await;
        
        // Parse the URL
        let url_parsed = Url::parse(url).map_err(|e| Error::UrlError(e))?;
        
        // Create the request builder
        let mut builder = client.request(method, url_parsed);
        
        // Add default headers
        let headers = self.default_headers.read().await;
        for (name, value) in headers.iter() {
            builder = builder.header(name, value);
        }
        
        Ok(builder)
    }
    
    /// Send a prepared request
    async fn send_request(&self, request: RequestBuilder) -> Result<Response> {
        // Send the request
        let response = request.send().await.map_err(|e| Error::HttpError(e))?;
        
        Ok(response)
    }
    
    /// Ensure the client is initialized
    async fn ensure_initialized(&self) -> Result<()> {
        let initialized = self.initialized.read().await;
        if !*initialized {
            // Drop the read lock
            drop(initialized);
            
            // Initialize
            self.init().await?;
        }
        
        Ok(())
    }
    
    /// Access an onion service
    pub async fn access_onion(&self, onion_url: &str) -> Result<Response> {
        // Ensure URL is an onion service
        if !onion_url.contains(".onion") {
            return Err(Error::OnionError(format!("Not an onion service URL: {}", onion_url)));
        }
        
        // Make the request
        self.get(onion_url).await
    }
    
    /// Get the current Tor controller
    pub fn controller(&self) -> Arc<TorController> {
        self.controller.clone()
    }
    
    /// Get the current circuit manager
    pub fn circuit_manager(&self) -> Arc<RwLock<TorCircuit>> {
        self.circuit.clone()
    }
    
    /// Get the current proxy
    pub fn proxy(&self) -> Arc<TorProxy> {
        self.proxy.clone()
    }
}

impl TorCapable for TorClient {
    fn configure_tor(&mut self, config: &TorConfig) -> Result<()> {
        self.config = config.clone();
        Ok(())
    }
    
    fn is_using_tor(&self) -> Result<bool> {
        // Convert to a future and block on it
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async { self.is_using_tor().await })
    }
    
    fn get_circuit_info(&self) -> Result<CircuitInfo> {
        // Convert to a future and block on it
        let rt = tokio::runtime::Runtime::new().unwrap();
        let info = rt.block_on(async { self.get_circuit_info().await })?;
        
        info.ok_or_else(|| Error::CircuitError("No active circuit".to_string()))
    }
    
    fn new_circuit(&mut self) -> Result<()> {
        // Convert to a future and block on it
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async { self.new_circuit().await })?;
        Ok(())
    }
    
    fn access_onion(&mut self, onion_url: &str) -> Result<()> {
        // Convert to a future and block on it
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async { 
            self.access_onion(onion_url).await?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_config() -> TorConfig {
        TorConfig {
            data_dir: std::path::PathBuf::from("./test_data"),
            use_tor_browser: false,
            tor_browser_path: None,
            socks_host: "127.0.0.1".to_string(),
            socks_port: 9050,
            control_port: 9051,
            control_password: None,
            exit_nodes: None,
            use_bridges: false,
            bridges: Vec::new(),
            options: HashMap::new(),
            timeout_secs: 10,
        }
    }
    
    #[ignore] // Ignore this test as it requires a real Tor instance
    #[tokio::test]
    async fn test_create_client() {
        let config = create_test_config();
        let client = TorClient::new(config);
        
        // This test doesn't do anything but create the client
        assert_eq!(client.user_agent, "Mozilla/5.0 (Windows NT 10.0; rv:100.0) Gecko/20100101 Firefox/100.0");
    }
} 