//! Tor controller interface
//!
//! This module provides a interface to the Tor control protocol for
//! managing Tor instances, creating circuits, and accessing information.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, RwLock};

use crate::Result;
use crate::Error;
use crate::circuit::{CircuitInfo, CircuitStatus, CircuitPurpose, NodeInfo};
use crate::TorConfig;

/// Controller for Tor instances
#[derive(Debug)]
pub struct TorController {
    /// Configuration for the controller
    config: TorConfig,
    
    /// Connection to the Tor control port, wrapped in a mutex for exclusive access
    connection: Arc<Mutex<Option<TcpStream>>>,
    
    /// Whether the controller has been authenticated
    authenticated: Arc<RwLock<bool>>,
    
    /// Version of the Tor instance
    version: Arc<RwLock<Option<String>>>,
    
    /// Whether to automatically reconnect on disconnection
    auto_reconnect: bool,
    
    /// Timeout for control commands
    timeout: Duration,
}

impl TorController {
    /// Create a new controller instance
    pub fn new(config: TorConfig) -> Self {
        let timeout = Duration::from_secs(config.timeout_secs);
        
        Self {
            config,
            connection: Arc::new(Mutex::new(None)),
            authenticated: Arc::new(RwLock::new(false)),
            version: Arc::new(RwLock::new(None)),
            auto_reconnect: true,
            timeout,
        }
    }
    
    /// Connect to the Tor control port
    pub async fn connect(&self) -> Result<()> {
        let addr = format!("{}:{}", self.config.socks_host, self.config.control_port);
        let sock_addr: SocketAddr = addr.parse().map_err(|e| Error::ControllerError(format!("Invalid address: {}", e)))?;
        
        let connection = tokio::time::timeout(
            self.timeout,
            TcpStream::connect(sock_addr)
        ).await
        .map_err(|_| Error::ControllerError("Connection timeout".to_string()))?
        .map_err(|e| Error::ControllerError(format!("Connection failed: {}", e)))?;
        
        let mut guard = self.connection.lock().await;
        *guard = Some(connection);
        
        // Get the Tor version
        let version = self.get_info("version").await?;
        *self.version.write().await = Some(version);
        
        // Authenticate if a password is provided
        if self.config.control_password.is_some() {
            self.authenticate().await?;
        } else {
            // Try a null authentication
            self.authenticate_null().await?;
        }
        
        Ok(())
    }
    
    /// Check if connected to the Tor control port
    pub async fn is_connected(&self) -> bool {
        let connection = self.connection.lock().await;
        connection.is_some()
    }
    
    /// Authenticate with the Tor control port using a password
    pub async fn authenticate(&self) -> Result<()> {
        if let Some(password) = &self.config.control_password {
            // Hash the password
            let hashed_password = format!("\"{}\"", password); // In a real implementation, this should be hashed properly
            
            // Send authentication command
            let response = self.send_command(&format!("AUTHENTICATE {}", hashed_password)).await?;
            
            // Check if authentication was successful
            if response.starts_with("250 ") {
                *self.authenticated.write().await = true;
                Ok(())
            } else {
                Err(Error::ControllerError(format!("Authentication failed: {}", response)))
            }
        } else {
            Err(Error::ControllerError("No password provided for authentication".to_string()))
        }
    }
    
    /// Authenticate with the Tor control port using null authentication
    pub async fn authenticate_null(&self) -> Result<()> {
        // Send null authentication command
        let response = self.send_command("AUTHENTICATE").await?;
        
        // Check if authentication was successful
        if response.starts_with("250 ") {
            *self.authenticated.write().await = true;
            Ok(())
        } else {
            Err(Error::ControllerError(format!("Null authentication failed: {}", response)))
        }
    }
    
    /// Send a command to the Tor control port
    pub async fn send_command(&self, command: &str) -> Result<String> {
        // Ensure we're connected
        if !self.is_connected().await {
            if self.auto_reconnect {
                self.connect().await?;
            } else {
                return Err(Error::ControllerError("Not connected".to_string()));
            }
        }
        
        // Get connection
        let mut conn_guard = self.connection.lock().await;
        let connection = conn_guard.as_mut().ok_or_else(|| Error::ControllerError("No connection".to_string()))?;
        
        // Write command
        let cmd = format!("{}\r\n", command);
        connection.write_all(cmd.as_bytes()).await
            .map_err(|e| Error::ControllerError(format!("Failed to send command: {}", e)))?;
        
        // Read response
        let mut reader = BufReader::new(connection);
        let mut response = String::new();
        let mut line = String::new();
        
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    response.push_str(&line);
                    if line.starts_with("250 ") || line.starts_with("550 ") {
                        break;
                    }
                },
                Err(e) => return Err(Error::ControllerError(format!("Failed to read response: {}", e))),
            }
        }
        
        Ok(response.trim().to_string())
    }
    
    /// Get information from Tor
    pub async fn get_info(&self, key: &str) -> Result<String> {
        let response = self.send_command(&format!("GETINFO {}", key)).await?;
        
        // Parse the response
        if response.starts_with("250-") {
            // Multi-line response
            let lines: Vec<&str> = response.lines().collect();
            for line in lines {
                if line.starts_with(&format!("250-{}", key)) {
                    let value = line.trim_start_matches(&format!("250-{}", key)).trim_start_matches('=').trim();
                    return Ok(value.to_string());
                }
            }
            Err(Error::ControllerError(format!("Key not found in response: {}", key)))
        } else if response.starts_with(&format!("250 {}", key)) {
            // Single-line response
            let value = response.trim_start_matches(&format!("250 {}", key)).trim_start_matches('=').trim();
            Ok(value.to_string())
        } else {
            // Error or unexpected response
            Err(Error::ControllerError(format!("Unexpected response: {}", response)))
        }
    }
    
    /// Create a new circuit
    pub async fn create_circuit(&self, exit_spec: Option<&str>) -> Result<String> {
        let cmd = if let Some(spec) = exit_spec {
            format!("EXTENDCIRCUIT 0 {}", spec)
        } else {
            "EXTENDCIRCUIT 0".to_string()
        };
        
        let response = self.send_command(&cmd).await?;
        
        // Parse the response to get the circuit ID
        if response.starts_with("250 ") {
            let parts: Vec<&str> = response.splitn(3, ' ').collect();
            if parts.len() >= 3 {
                Ok(parts[2].to_string())
            } else {
                Err(Error::ControllerError(format!("Invalid response format: {}", response)))
            }
        } else {
            Err(Error::ControllerError(format!("Failed to create circuit: {}", response)))
        }
    }
    
    /// Close a circuit
    pub async fn close_circuit(&self, circuit_id: &str) -> Result<()> {
        let response = self.send_command(&format!("CLOSECIRCUIT {}", circuit_id)).await?;
        
        if response.starts_with("250 ") {
            Ok(())
        } else {
            Err(Error::ControllerError(format!("Failed to close circuit: {}", response)))
        }
    }
    
    /// Signal Tor to create a new identity (new circuits)
    pub async fn signal_newnym(&self) -> Result<()> {
        let response = self.send_command("SIGNAL NEWNYM").await?;
        
        if response.starts_with("250 ") {
            Ok(())
        } else {
            Err(Error::ControllerError(format!("Failed to signal new identity: {}", response)))
        }
    }
    
    /// Get information about all circuits
    pub async fn get_circuits(&self) -> Result<HashMap<String, CircuitInfo>> {
        let response = self.send_command("GETINFO circuit-status").await?;
        
        let mut circuits = HashMap::new();
        
        // Parse the response
        for line in response.lines() {
            if line.starts_with("250-circuit-status=") || line.starts_with("250 circuit-status=") {
                let circuit_text = line.trim_start_matches("250-circuit-status=").trim_start_matches("250 circuit-status=");
                
                // Parse each circuit
                for circuit_line in circuit_text.lines() {
                    if let Some(circuit) = parse_circuit_line(circuit_line) {
                        circuits.insert(circuit.id.clone(), circuit);
                    }
                }
            }
        }
        
        // Fetch node information for each circuit
        for circuit in circuits.values_mut() {
            // Fetch node details if needed
            for node in &mut circuit.nodes {
                if let Some(country) = self.get_node_country(&node.fingerprint).await? {
                    node.country = Some(country);
                }
            }
        }
        
        Ok(circuits)
    }
    
    /// Get the country code for a node
    pub async fn get_node_country(&self, fingerprint: &str) -> Result<Option<String>> {
        let response = self.send_command(&format!("GETINFO ns/id/{}", fingerprint)).await;
        
        match response {
            Ok(resp) => {
                // Parse the node details to find the country code
                for line in resp.lines() {
                    if line.contains("country=") {
                        let parts: Vec<&str> = line.split("country=").collect();
                        if parts.len() > 1 {
                            let country = parts[1].split_whitespace().next().unwrap_or("");
                            if !country.is_empty() {
                                return Ok(Some(country.to_string()));
                            }
                        }
                    }
                }
                Ok(None)
            },
            Err(_) => Ok(None), // Ignore errors when getting country information
        }
    }
    
    /// Get the version of the Tor instance
    pub async fn get_version(&self) -> Option<String> {
        let version = self.version.read().await;
        version.clone()
    }
    
    /// Set whether to automatically reconnect
    pub fn set_auto_reconnect(&mut self, auto_reconnect: bool) {
        self.auto_reconnect = auto_reconnect;
    }
    
    /// Disconnect from the Tor control port
    pub async fn disconnect(&self) -> Result<()> {
        let mut conn_guard = self.connection.lock().await;
        *conn_guard = None;
        
        // Update authenticated state
        *self.authenticated.write().await = false;
        
        Ok(())
    }
}

/// Parse a circuit status line from the Tor control protocol
fn parse_circuit_line(line: &str) -> Option<CircuitInfo> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }
    
    let id = parts[0].to_string();
    
    let status = match parts[1] {
        "BUILT" => CircuitStatus::Established,
        "EXTENDED" | "BUILDING" => CircuitStatus::Building,
        "FAILED" => CircuitStatus::Failed,
        "CLOSED" => CircuitStatus::Closed,
        _ => CircuitStatus::Building, // Default
    };
    
    let mut nodes = Vec::new();
    if parts.len() > 2 {
        let path = parts[2];
        for node_str in path.split(',') {
            let node_parts: Vec<&str> = node_str.split('~').collect();
            if node_parts.len() >= 2 {
                let name = node_parts[0].to_string();
                let fingerprint = node_parts[1].to_string();
                
                let mut node = NodeInfo::new(&name, &fingerprint, "");
                
                // Determine if this is an exit/guard node based on its position
                if path.ends_with(&format!("~{}", fingerprint)) {
                    node = node.with_exit(true);
                }
                
                if path.starts_with(&format!("{}~", name)) {
                    node = node.with_guard(true);
                }
                
                nodes.push(node);
            }
        }
    }
    
    // Extract circuit purpose if provided
    let purpose = if parts.len() > 3 {
        match parts[3] {
            "PURPOSE=GENERAL" => CircuitPurpose::General,
            "PURPOSE=HS_CLIENT_INTRO" | "PURPOSE=HS_CLIENT_REND" => CircuitPurpose::HiddenService,
            "PURPOSE=TESTING" => CircuitPurpose::Testing,
            "PURPOSE=CONTROLLER" => CircuitPurpose::Controller,
            "PURPOSE=HS_VANGUARDS" => CircuitPurpose::Custom("HS_VANGUARDS".to_string()),
            _ => CircuitPurpose::General,
        }
    } else {
        CircuitPurpose::General
    };
    
    // Extract build flags if provided
    let mut build_flags = Vec::new();
    if parts.len() > 4 {
        for part in &parts[4..] {
            if part.starts_with("BUILD_FLAGS=") {
                let flags = part.trim_start_matches("BUILD_FLAGS=");
                build_flags = flags.split(',').map(|s| s.to_string()).collect();
            }
        }
    }
    
    // Construct the circuit info
    Some(CircuitInfo {
        id,
        status,
        nodes,
        created_at: chrono::Utc::now(),
        established_for: None,
        purpose,
        build_reason: None,
        build_flags,
        in_use: false,
        is_hidden_service: matches!(purpose, CircuitPurpose::HiddenService),
        onion_service: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Mock for testing
    struct MockTorController {
        responses: HashMap<String, String>,
    }
    
    impl MockTorController {
        fn new() -> Self {
            let mut responses = HashMap::new();
            
            // Add mock responses
            responses.insert(
                "GETINFO version".to_string(),
                "250-version=0.4.7.8\r\n250 OK".to_string(),
            );
            
            responses.insert(
                "GETINFO circuit-status".to_string(),
                "250-circuit-status=1 BUILT A~AAAA,B~BBBB,C~CCCC PURPOSE=GENERAL\r\n\
                 2 BUILT D~DDDD,E~EEEE,F~FFFF PURPOSE=HS_CLIENT_REND\r\n\
                 3 FAILED G~GGGG,H~HHHH BUILD_FLAGS=NEED_CAPACITY\r\n\
                 250 OK".to_string(),
            );
            
            Self {
                responses,
            }
        }
        
        fn get_response(&self, command: &str) -> String {
            // Find an exact match first
            if let Some(response) = self.responses.get(command) {
                return response.clone();
            }
            
            // Try to find a partial match
            for (cmd, resp) in &self.responses {
                if command.starts_with(cmd) {
                    return resp.clone();
                }
            }
            
            // Default response
            "250 OK".to_string()
        }
    }
    
    #[test]
    fn test_parse_circuit_line() {
        let line = "1 BUILT A~AAAA,B~BBBB,C~CCCC PURPOSE=GENERAL";
        let circuit = parse_circuit_line(line).unwrap();
        
        assert_eq!(circuit.id, "1");
        assert_eq!(circuit.status, CircuitStatus::Established);
        assert_eq!(circuit.nodes.len(), 3);
        assert_eq!(circuit.nodes[0].name, "A");
        assert_eq!(circuit.nodes[0].fingerprint, "AAAA");
        assert_eq!(circuit.nodes[0].is_guard, true);
        assert_eq!(circuit.nodes[2].is_exit, true);
        assert_eq!(circuit.purpose, CircuitPurpose::General);
    }
    
    #[test]
    fn test_parse_circuit_line_with_build_flags() {
        let line = "3 FAILED G~GGGG,H~HHHH BUILD_FLAGS=NEED_CAPACITY,NEED_UPTIME";
        let circuit = parse_circuit_line(line).unwrap();
        
        assert_eq!(circuit.id, "3");
        assert_eq!(circuit.status, CircuitStatus::Failed);
        assert_eq!(circuit.build_flags.len(), 2);
        assert_eq!(circuit.build_flags[0], "NEED_CAPACITY");
        assert_eq!(circuit.build_flags[1], "NEED_UPTIME");
    }
} 