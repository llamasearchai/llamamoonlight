//! Tor SOCKS proxy management
//!
//! This module provides functionality for interacting with the Tor SOCKS proxy,
//! allowing clients to route traffic through the Tor network.

use std::collections::HashMap;
use std::io::{Error as IoError, ErrorKind};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::process::{Command, Child, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use thiserror::Error;

use crate::{Result, Error, TorConfig};

/// Errors that can occur during proxy operations
#[derive(Debug, Error)]
pub enum ProxyError {
    /// Failed to connect to the SOCKS proxy
    #[error("Failed to connect to SOCKS proxy: {0}")]
    ConnectionError(String),
    
    /// Failed to start Tor process
    #[error("Failed to start Tor process: {0}")]
    StartError(String),
    
    /// Timeout waiting for proxy to become available
    #[error("Timeout waiting for proxy to become available")]
    TimeoutError,
    
    /// Invalid proxy configuration
    #[error("Invalid proxy configuration: {0}")]
    ConfigurationError(String),
    
    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] IoError),
}

/// SOCKS proxy version
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocksVersion {
    /// SOCKS 4
    Socks4,
    /// SOCKS 5
    Socks5,
}

impl std::fmt::Display for SocksVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SocksVersion::Socks4 => write!(f, "socks4"),
            SocksVersion::Socks5 => write!(f, "socks5"),
        }
    }
}

/// Manages the Tor SOCKS proxy
#[derive(Debug)]
pub struct TorProxy {
    /// Tor configuration
    config: TorConfig,
    
    /// Tor process, if managed by this proxy
    tor_process: Option<Child>,
    
    /// Whether the proxy is connected
    connected: bool,
    
    /// Last check time
    last_check: Option<Instant>,
    
    /// SOCKS version
    socks_version: SocksVersion,
}

impl TorProxy {
    /// Create a new Tor proxy with the given configuration
    pub fn new(config: TorConfig) -> Self {
        Self {
            config,
            tor_process: None,
            connected: false,
            last_check: None,
            socks_version: SocksVersion::Socks5,
        }
    }
    
    /// Initialize the proxy
    pub async fn init(&self) -> Result<()> {
        // Check if Tor is already running on the configured port
        if self.is_tor_running().await {
            return Ok(());
        }
        
        // If we're using Tor Browser, no need to start Tor
        if self.config.use_tor_browser {
            return Ok(());
        }
        
        // Otherwise, start Tor if needed
        self.start_tor_if_needed().await
    }
    
    /// Get the SOCKS proxy URL in the format expected by reqwest
    pub fn get_proxy_url(&self) -> String {
        format!("{}://{}:{}", 
            self.socks_version, 
            self.config.socks_host, 
            self.config.socks_port)
    }
    
    /// Check if Tor is running on the configured port
    pub async fn is_tor_running(&self) -> bool {
        // If we checked recently, return cached result
        if let Some(last_check) = self.last_check {
            if last_check.elapsed() < Duration::from_secs(5) {
                return self.connected;
            }
        }
        
        // Try to connect to the SOCKS port
        let socket_addr = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 
            self.config.socks_port
        );
        
        // Use a non-blocking check
        let connected = tokio::net::TcpStream::connect(socket_addr)
            .await
            .is_ok();
        
        // Return the result
        connected
    }
    
    /// Start Tor if it's not already running
    pub async fn start_tor_if_needed(&self) -> Result<()> {
        // If Tor is already running, no need to start it
        if self.is_tor_running().await {
            return Ok(());
        }
        
        // Start Tor
        self.start_tor().await?;
        
        // Wait for Tor to become available
        self.wait_for_tor().await?;
        
        Ok(())
    }
    
    /// Start the Tor process
    async fn start_tor(&self) -> Result<()> {
        // Default Tor binary path - may need to adjust for different platforms
        let tor_bin = if cfg!(target_os = "windows") {
            "tor.exe"
        } else {
            "tor"
        };
        
        // Build arguments for Tor
        let mut cmd = Command::new(tor_bin);
        
        // Configure data directory
        if let Some(data_dir) = self.config.data_dir.to_str() {
            cmd.arg("--DataDirectory").arg(data_dir);
        }
        
        // Configure SOCKS port
        cmd.arg("--SocksPort").arg(format!("{}", self.config.socks_port));
        
        // Configure control port
        cmd.arg("--ControlPort").arg(format!("{}", self.config.control_port));
        
        // If we have a control password, add it
        if let Some(pass) = &self.config.control_password {
            cmd.arg("--HashedControlPassword").arg(pass);
        } else {
            // Otherwise, allow connections without authentication
            cmd.arg("--CookieAuthentication").arg("1");
        }
        
        // Configure exit nodes if specified
        if let Some(exit_nodes) = &self.config.exit_nodes {
            cmd.arg("--ExitNodes").arg(exit_nodes);
        }
        
        // Configure bridges if specified
        if self.config.use_bridges {
            cmd.arg("--UseBridges").arg("1");
            
            for bridge in &self.config.bridges {
                cmd.arg("--Bridge").arg(bridge);
            }
        }
        
        // Configure additional options
        for (key, value) in &self.config.options {
            cmd.arg(format!("--{}", key)).arg(value);
        }
        
        // Redirect output to null
        cmd.stdout(Stdio::null())
           .stderr(Stdio::null());
        
        // Start the process
        let process = cmd
            .spawn()
            .map_err(|e| Error::ProxyError(format!("Failed to start Tor: {}", e)))?;
        
        // Store the process
        let mut this = self.clone();
        this.tor_process = Some(process);
        
        Ok(())
    }
    
    /// Wait for Tor to become available
    async fn wait_for_tor(&self) -> Result<()> {
        // Maximum time to wait
        let max_wait = Duration::from_secs(60);
        let start_time = Instant::now();
        
        // Poll the SOCKS port until it's available
        loop {
            if self.is_tor_running().await {
                return Ok(());
            }
            
            // Check if we've waited too long
            if start_time.elapsed() > max_wait {
                return Err(Error::ProxyError("Timeout waiting for Tor to start".to_string()));
            }
            
            // Wait a bit before trying again
            sleep(Duration::from_millis(500)).await;
        }
    }
    
    /// Check if the proxy is available and working
    pub async fn check_proxy(&self) -> Result<bool> {
        // Try to connect to the SOCKS port
        let socket_addr = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 
            self.config.socks_port
        );
        
        // Use a non-blocking check
        match tokio::net::TcpStream::connect(socket_addr).await {
            Ok(_) => Ok(true),
            Err(e) => Err(Error::ProxyError(format!("Failed to connect to SOCKS proxy: {}", e))),
        }
    }
    
    /// Get the current SOCKS version
    pub fn socks_version(&self) -> SocksVersion {
        self.socks_version
    }
    
    /// Set the SOCKS version
    pub fn set_socks_version(&mut self, version: SocksVersion) {
        self.socks_version = version;
    }
    
    /// Get the proxy host and port
    pub fn get_proxy_address(&self) -> (String, u16) {
        (self.config.socks_host.clone(), self.config.socks_port)
    }
    
    /// Get the Tor config
    pub fn config(&self) -> &TorConfig {
        &self.config
    }
    
    /// Check if the proxy is connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }
}

impl Clone for TorProxy {
    fn clone(&self) -> Self {
        // We can't clone the process, so we create a new instance without it
        Self {
            config: self.config.clone(),
            tor_process: None,
            connected: self.connected,
            last_check: self.last_check,
            socks_version: self.socks_version,
        }
    }
}

impl Drop for TorProxy {
    fn drop(&mut self) {
        // Kill the Tor process if we started it
        if let Some(mut process) = self.tor_process.take() {
            let _ = process.kill();
        }
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
    
    #[test]
    fn test_proxy_url() {
        let config = create_test_config();
        let proxy = TorProxy::new(config);
        
        assert_eq!(proxy.get_proxy_url(), "socks5://127.0.0.1:9050");
    }
    
    #[test]
    fn test_socks_version() {
        let config = create_test_config();
        let mut proxy = TorProxy::new(config);
        
        assert_eq!(proxy.socks_version(), SocksVersion::Socks5);
        
        proxy.set_socks_version(SocksVersion::Socks4);
        assert_eq!(proxy.socks_version(), SocksVersion::Socks4);
    }
} 