//! Tor circuit management
//!
//! This module provides functionality for creating and managing Tor circuits.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::Result;
use crate::Error;
use crate::TorConfig;

/// Information about a node in the Tor circuit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    /// Nickname of the node
    pub name: String,
    
    /// Fingerprint of the node
    pub fingerprint: String,
    
    /// IP address of the node
    pub address: String,
    
    /// Country code of the node
    pub country: Option<String>,
    
    /// Whether this is an exit node
    pub is_exit: bool,
    
    /// Whether this is a guard node
    pub is_guard: bool,
    
    /// Whether this is a relay node
    pub is_relay: bool,
    
    /// Additional metadata about the node
    pub metadata: HashMap<String, String>,
}

impl NodeInfo {
    /// Create a new node information object
    pub fn new(name: &str, fingerprint: &str, address: &str) -> Self {
        Self {
            name: name.to_string(),
            fingerprint: fingerprint.to_string(),
            address: address.to_string(),
            country: None,
            is_exit: false,
            is_guard: false,
            is_relay: true,
            metadata: HashMap::new(),
        }
    }
    
    /// Set the country code for this node
    pub fn with_country(mut self, country: &str) -> Self {
        self.country = Some(country.to_string());
        self
    }
    
    /// Set whether this is an exit node
    pub fn with_exit(mut self, is_exit: bool) -> Self {
        self.is_exit = is_exit;
        self
    }
    
    /// Set whether this is a guard node
    pub fn with_guard(mut self, is_guard: bool) -> Self {
        self.is_guard = is_guard;
        self
    }
    
    /// Set whether this is a relay node
    pub fn with_relay(mut self, is_relay: bool) -> Self {
        self.is_relay = is_relay;
        self
    }
    
    /// Add metadata to this node
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// Information about a Tor circuit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitInfo {
    /// Unique ID of the circuit
    pub id: String,
    
    /// Status of the circuit
    pub status: CircuitStatus,
    
    /// Nodes in the circuit
    pub nodes: Vec<NodeInfo>,
    
    /// Time when the circuit was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// How long the circuit has been established for
    pub established_for: Option<Duration>,
    
    /// Purpose of the circuit
    pub purpose: CircuitPurpose,
    
    /// Reason for the circuit to be built
    pub build_reason: Option<String>,
    
    /// How the circuit was built
    pub build_flags: Vec<String>,
    
    /// Whether the circuit is currently in use
    pub in_use: bool,
    
    /// Whether the circuit is a hidden service circuit
    pub is_hidden_service: bool,
    
    /// Onion service information, if applicable
    pub onion_service: Option<String>,
}

/// Status of a Tor circuit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitStatus {
    /// Circuit is being built
    Building,
    
    /// Circuit is established and ready
    Established,
    
    /// Circuit has failed
    Failed,
    
    /// Circuit has been closed
    Closed,
}

/// Purpose of a Tor circuit
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitPurpose {
    /// General purpose circuit
    General,
    
    /// Circuit for client connections
    ClientGeneral,
    
    /// Circuit for hidden service connections
    HiddenService,
    
    /// Circuit for testing
    Testing,
    
    /// Circuit for controller-specified connections
    Controller,
    
    /// Circuit for rendezvous points
    Rendezvous,
    
    /// Circuit for introduction points
    Introduction,
    
    /// Custom purpose
    Custom(String),
}

impl CircuitInfo {
    /// Check if the circuit passes through a specific country
    pub fn passes_through_country(&self, country_code: &str) -> bool {
        self.nodes.iter().any(|node| 
            node.country.as_ref().map_or(false, |c| c == country_code)
        )
    }
    
    /// Check if the circuit uses a specific node
    pub fn uses_node(&self, fingerprint: &str) -> bool {
        self.nodes.iter().any(|node| node.fingerprint == fingerprint)
    }
    
    /// Get the exit node of the circuit
    pub fn exit_node(&self) -> Option<&NodeInfo> {
        self.nodes.iter().find(|node| node.is_exit)
    }
    
    /// Get the guard node of the circuit
    pub fn guard_node(&self) -> Option<&NodeInfo> {
        self.nodes.iter().find(|node| node.is_guard)
    }
    
    /// Check if the circuit is healthy
    pub fn is_healthy(&self) -> bool {
        self.status == CircuitStatus::Established && !self.nodes.is_empty()
    }
    
    /// Format the circuit as a string path
    pub fn path_string(&self) -> String {
        self.nodes
            .iter()
            .map(|node| {
                let country = node.country.as_deref().unwrap_or("??");
                format!("{}({})", node.name, country)
            })
            .collect::<Vec<_>>()
            .join(" -> ")
    }
}

/// Tor circuit manager
#[derive(Debug)]
pub struct TorCircuit {
    /// Tor controller for creating and managing circuits
    controller: std::sync::Arc<crate::controller::TorController>,
    
    /// Current active circuits
    circuits: HashMap<String, CircuitInfo>,
    
    /// Default circuit for general usage
    default_circuit: Option<String>,
    
    /// Configuration for the circuit manager
    config: TorConfig,
    
    /// Time when the last circuit was built
    last_circuit_built: Option<Instant>,
}

impl TorCircuit {
    /// Create a new circuit manager with the given controller
    pub fn new(controller: std::sync::Arc<crate::controller::TorController>, config: TorConfig) -> Self {
        Self {
            controller,
            circuits: HashMap::new(),
            default_circuit: None,
            config,
            last_circuit_built: None,
        }
    }
    
    /// Initialize the circuit manager
    pub async fn init(&mut self) -> Result<()> {
        // Fetch existing circuits
        self.refresh_circuits().await?;
        
        // Create a default circuit if needed
        if self.default_circuit.is_none() {
            let circuit_id = self.create_circuit().await?;
            self.default_circuit = Some(circuit_id);
        }
        
        Ok(())
    }
    
    /// Refresh the list of circuits
    pub async fn refresh_circuits(&mut self) -> Result<&HashMap<String, CircuitInfo>> {
        let circuits = self.controller.get_circuits().await?;
        self.circuits = circuits;
        
        if let Some(default_id) = &self.default_circuit {
            if !self.circuits.contains_key(default_id) {
                self.default_circuit = None;
            }
        }
        
        Ok(&self.circuits)
    }
    
    /// Create a new circuit
    pub async fn create_circuit(&mut self) -> Result<String> {
        // Check if we need to wait before creating a new circuit
        if let Some(last_built) = self.last_circuit_built {
            let min_wait = Duration::from_secs(10); // Minimum wait time between circuit creations
            let elapsed = last_built.elapsed();
            
            if elapsed < min_wait {
                let wait_time = min_wait - elapsed;
                log::debug!("Waiting for {:?} before creating a new circuit", wait_time);
                tokio::time::sleep(wait_time).await;
            }
        }
        
        // Build exit node specification if needed
        let exit_spec = if let Some(exit_nodes) = &self.config.exit_nodes {
            format!("ExitNodes={}", exit_nodes)
        } else {
            String::new()
        };
        
        // Create the circuit
        let circuit_id = if exit_spec.is_empty() {
            self.controller.create_circuit(None).await?
        } else {
            self.controller.create_circuit(Some(&exit_spec)).await?
        };
        
        // Update the last build time
        self.last_circuit_built = Some(Instant::now());
        
        // Refresh circuit list to get the new circuit
        self.refresh_circuits().await?;
        
        Ok(circuit_id)
    }
    
    /// Close a specific circuit
    pub async fn close_circuit(&mut self, circuit_id: &str) -> Result<()> {
        if self.default_circuit.as_deref() == Some(circuit_id) {
            self.default_circuit = None;
        }
        
        self.controller.close_circuit(circuit_id).await?;
        self.circuits.remove(circuit_id);
        
        Ok(())
    }
    
    /// Create a new identity (close all circuits and create a new one)
    pub async fn new_identity(&mut self) -> Result<String> {
        // Close all circuits
        let circuit_ids: Vec<String> = self.circuits.keys().cloned().collect();
        for id in circuit_ids {
            let _ = self.controller.close_circuit(&id).await;
        }
        
        self.circuits.clear();
        self.default_circuit = None;
        
        // Signal for a new identity
        self.controller.signal_newnym().await?;
        
        // Create a new default circuit
        let circuit_id = self.create_circuit().await?;
        self.default_circuit = Some(circuit_id);
        
        Ok(circuit_id)
    }
    
    /// Get information about the default circuit
    pub fn default_circuit_info(&self) -> Option<&CircuitInfo> {
        self.default_circuit.as_ref().and_then(|id| self.circuits.get(id))
    }
    
    /// Get information about a specific circuit
    pub fn get_circuit_info(&self, circuit_id: &str) -> Option<&CircuitInfo> {
        self.circuits.get(circuit_id)
    }
    
    /// Get all circuit information
    pub fn all_circuits(&self) -> &HashMap<String, CircuitInfo> {
        &self.circuits
    }
    
    /// Check if a circuit with specific characteristics exists
    pub fn find_circuit(&self, predicate: impl Fn(&CircuitInfo) -> bool) -> Option<&CircuitInfo> {
        self.circuits.values().find(|&circuit| predicate(circuit))
    }
    
    /// Set a specific circuit as the default
    pub fn set_default_circuit(&mut self, circuit_id: &str) -> Result<()> {
        if !self.circuits.contains_key(circuit_id) {
            return Err(Error::CircuitError(format!("Circuit not found: {}", circuit_id)));
        }
        
        self.default_circuit = Some(circuit_id.to_string());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_node_info() -> NodeInfo {
        NodeInfo::new("test-relay", "ABCDEF1234567890", "192.168.1.1:9001")
            .with_country("US")
            .with_exit(false)
            .with_guard(true)
    }
    
    fn create_test_circuit_info() -> CircuitInfo {
        CircuitInfo {
            id: "1".to_string(),
            status: CircuitStatus::Established,
            nodes: vec![
                NodeInfo::new("guard1", "AAAA1111", "1.1.1.1:9001")
                    .with_country("US")
                    .with_guard(true),
                NodeInfo::new("middle", "BBBB2222", "2.2.2.2:9001")
                    .with_country("FR"),
                NodeInfo::new("exit1", "CCCC3333", "3.3.3.3:9001")
                    .with_country("DE")
                    .with_exit(true),
            ],
            created_at: chrono::Utc::now(),
            established_for: Some(Duration::from_secs(60)),
            purpose: CircuitPurpose::General,
            build_reason: None,
            build_flags: vec!["NEED_CAPACITY".to_string()],
            in_use: true,
            is_hidden_service: false,
            onion_service: None,
        }
    }
    
    #[test]
    fn test_node_info() {
        let node = create_test_node_info();
        
        assert_eq!(node.name, "test-relay");
        assert_eq!(node.fingerprint, "ABCDEF1234567890");
        assert_eq!(node.address, "192.168.1.1:9001");
        assert_eq!(node.country, Some("US".to_string()));
        assert_eq!(node.is_exit, false);
        assert_eq!(node.is_guard, true);
    }
    
    #[test]
    fn test_circuit_info() {
        let circuit = create_test_circuit_info();
        
        assert_eq!(circuit.id, "1");
        assert_eq!(circuit.status, CircuitStatus::Established);
        assert_eq!(circuit.nodes.len(), 3);
        assert_eq!(circuit.purpose, CircuitPurpose::General);
        
        // Test circuit methods
        assert!(circuit.passes_through_country("FR"));
        assert!(!circuit.passes_through_country("GB"));
        assert!(circuit.uses_node("AAAA1111"));
        assert!(!circuit.uses_node("DDDD4444"));
        
        // Test exit and guard node methods
        let exit = circuit.exit_node();
        assert!(exit.is_some());
        assert_eq!(exit.unwrap().fingerprint, "CCCC3333");
        
        let guard = circuit.guard_node();
        assert!(guard.is_some());
        assert_eq!(guard.unwrap().fingerprint, "AAAA1111");
        
        // Test health check
        assert!(circuit.is_healthy());
        
        // Test path formatting
        let path = circuit.path_string();
        assert_eq!(path, "guard1(US) -> middle(FR) -> exit1(DE)");
    }
} 