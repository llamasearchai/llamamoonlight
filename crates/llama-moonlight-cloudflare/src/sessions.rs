use crate::{CloudflareConfig, CloudflareError};
use reqwest::{Client, ClientBuilder, Method, RequestBuilder, Response};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

/// A persistent session for making requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Session ID
    pub id: String,
    
    /// Session name
    pub name: Option<String>,
    
    /// Session creation time
    #[serde(skip)]
    pub created_at: Instant,
    
    /// Last activity time
    #[serde(skip)]
    pub last_activity: Instant,
    
    /// Session configuration
    pub config: CloudflareConfig,
    
    /// Cookies
    pub cookies: HashMap<String, String>,
    
    /// User agent
    pub user_agent: Option<String>,
    
    /// Custom headers
    pub headers: HashMap<String, String>,
}

impl Session {
    /// Create a new session
    pub fn new(config: CloudflareConfig, cookies: HashMap<String, String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: None,
            created_at: Instant::now(),
            last_activity: Instant::now(),
            config,
            cookies,
            user_agent: None,
            headers: HashMap::new(),
        }
    }
    
    /// Create a new session with a name
    pub fn new_with_name(name: &str, config: CloudflareConfig, cookies: HashMap<String, String>) -> Self {
        let mut session = Self::new(config, cookies);
        session.name = Some(name.to_string());
        session
    }
    
    /// Set a cookie
    pub fn set_cookie(&mut self, name: &str, value: &str) {
        self.cookies.insert(name.to_string(), value.to_string());
    }
    
    /// Get a cookie
    pub fn get_cookie(&self, name: &str) -> Option<&String> {
        self.cookies.get(name)
    }
    
    /// Remove a cookie
    pub fn remove_cookie(&mut self, name: &str) {
        self.cookies.remove(name);
    }
    
    /// Clear all cookies
    pub fn clear_cookies(&mut self) {
        self.cookies.clear();
    }
    
    /// Set a header
    pub fn set_header(&mut self, name: &str, value: &str) {
        self.headers.insert(name.to_string(), value.to_string());
    }
    
    /// Get a header
    pub fn get_header(&self, name: &str) -> Option<&String> {
        self.headers.get(name)
    }
    
    /// Remove a header
    pub fn remove_header(&mut self, name: &str) {
        self.headers.remove(name);
    }
    
    /// Clear all headers
    pub fn clear_headers(&mut self) {
        self.headers.clear();
    }
    
    /// Set the user agent
    pub fn set_user_agent(&mut self, user_agent: &str) {
        self.user_agent = Some(user_agent.to_string());
    }
    
    /// Get the user agent
    pub fn user_agent(&self) -> Option<&String> {
        self.user_agent.as_ref()
    }
    
    /// Update the last activity time
    pub fn update_activity(&mut self) {
        self.last_activity = Instant::now();
    }
    
    /// Check if the session has expired
    pub fn has_expired(&self, timeout_seconds: u64) -> bool {
        self.last_activity.elapsed() > Duration::from_secs(timeout_seconds)
    }
    
    /// Create a Client for this session
    pub fn create_client(&self) -> Result<Client, CloudflareError> {
        let mut client_builder = Client::builder()
            .timeout(Duration::from_secs(self.config.timeout_seconds))
            .cookie_store(true)
            .redirect(if self.config.follow_redirects {
                reqwest::redirect::Policy::limited(10)
            } else {
                reqwest::redirect::Policy::none()
            });
            
        // Add proxy if configured
        if let Some(ref proxy_url) = self.config.proxy {
            let proxy = reqwest::Proxy::all(proxy_url)
                .map_err(|e| CloudflareError::ProxyError(format!("Failed to create proxy: {}", e)))?;
            client_builder = client_builder.proxy(proxy);
        }
        
        // Build client
        client_builder.build()
            .map_err(|e| CloudflareError::HttpError(e))
    }
    
    /// Create a request builder for this session
    pub fn create_request(&self, method: Method, url: &str) -> Result<RequestBuilder, CloudflareError> {
        let client = self.create_client()?;
        let mut builder = client.request(method, url);
        
        // Add headers
        for (key, value) in &self.headers {
            builder = builder.header(key, value);
        }
        
        // Add user agent
        if let Some(ref user_agent) = self.user_agent {
            builder = builder.header("User-Agent", user_agent);
        } else if let Some(ref user_agent) = self.config.user_agent {
            builder = builder.header("User-Agent", user_agent);
        }
        
        // Add cookies
        if !self.cookies.is_empty() {
            let cookie_header = self.cookies.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<String>>()
                .join("; ");
            
            builder = builder.header("Cookie", cookie_header);
        }
        
        Ok(builder)
    }
    
    /// Execute a request using this session
    pub async fn execute(&mut self, request: RequestBuilder) -> Result<Response, CloudflareError> {
        self.update_activity();
        
        // Build and execute the request
        let response = request.build()
            .map_err(|e| CloudflareError::HttpError(e))?
            .send()
            .await
            .map_err(|e| CloudflareError::HttpError(e))?;
        
        // Extract cookies
        if let Some(headers) = response.headers().get_all("set-cookie").iter().next() {
            for header in response.headers().get_all("set-cookie") {
                if let Ok(cookie_str) = header.to_str() {
                    if let Some((name, value)) = parse_cookie(cookie_str) {
                        self.set_cookie(&name, &value);
                    }
                }
            }
        }
        
        Ok(response)
    }
    
    /// Save the session to a file
    pub fn save_to_file(&self, file_path: &PathBuf) -> Result<(), CloudflareError> {
        // Create a serializable version
        let serializable = SerializableSession {
            id: self.id.clone(),
            name: self.name.clone(),
            config: self.config.clone(),
            cookies: self.cookies.clone(),
            user_agent: self.user_agent.clone(),
            headers: self.headers.clone(),
        };
        
        // Serialize to JSON
        let json = serde_json::to_string_pretty(&serializable)
            .map_err(|e| CloudflareError::Other(format!("Failed to serialize session: {}", e)))?;
        
        // Write to file
        std::fs::write(file_path, json)
            .map_err(|e| CloudflareError::Other(format!("Failed to write session to file: {}", e)))?;
        
        Ok(())
    }
    
    /// Load a session from a file
    pub fn load_from_file(file_path: &PathBuf) -> Result<Self, CloudflareError> {
        // Read from file
        let json = std::fs::read_to_string(file_path)
            .map_err(|e| CloudflareError::Other(format!("Failed to read session from file: {}", e)))?;
        
        // Deserialize from JSON
        let serializable: SerializableSession = serde_json::from_str(&json)
            .map_err(|e| CloudflareError::Other(format!("Failed to deserialize session: {}", e)))?;
        
        // Create a session
        Ok(Self {
            id: serializable.id,
            name: serializable.name,
            created_at: Instant::now(),
            last_activity: Instant::now(),
            config: serializable.config,
            cookies: serializable.cookies,
            user_agent: serializable.user_agent,
            headers: serializable.headers,
        })
    }
}

/// A serializable version of the session for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializableSession {
    /// Session ID
    pub id: String,
    
    /// Session name
    pub name: Option<String>,
    
    /// Session configuration
    pub config: CloudflareConfig,
    
    /// Cookies
    pub cookies: HashMap<String, String>,
    
    /// User agent
    pub user_agent: Option<String>,
    
    /// Custom headers
    pub headers: HashMap<String, String>,
}

/// A session manager for multiple sessions
pub struct SessionManager {
    /// Active sessions
    sessions: HashMap<String, Session>,
    
    /// Default session timeout in seconds
    timeout_seconds: u64,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            timeout_seconds: 3600, // 1 hour
        }
    }
    
    /// Create a new session manager with a custom timeout
    pub fn new_with_timeout(timeout_seconds: u64) -> Self {
        Self {
            sessions: HashMap::new(),
            timeout_seconds,
        }
    }
    
    /// Add a session
    pub fn add_session(&mut self, session: Session) {
        self.sessions.insert(session.id.clone(), session);
    }
    
    /// Get a session by ID
    pub fn get_session(&self, id: &str) -> Option<&Session> {
        self.sessions.get(id)
    }
    
    /// Get a mutable session by ID
    pub fn get_session_mut(&mut self, id: &str) -> Option<&mut Session> {
        self.sessions.get_mut(id)
    }
    
    /// Remove a session
    pub fn remove_session(&mut self, id: &str) {
        self.sessions.remove(id);
    }
    
    /// Get all sessions
    pub fn all_sessions(&self) -> impl Iterator<Item = &Session> {
        self.sessions.values()
    }
    
    /// Clean up expired sessions
    pub fn cleanup(&mut self) {
        let expired_ids: Vec<String> = self.sessions.iter()
            .filter(|(_, session)| session.has_expired(self.timeout_seconds))
            .map(|(id, _)| id.clone())
            .collect();
        
        for id in expired_ids {
            self.sessions.remove(&id);
        }
    }
    
    /// Save all sessions to a directory
    pub fn save_all(&self, dir_path: &PathBuf) -> Result<(), CloudflareError> {
        // Create directory if it doesn't exist
        if !dir_path.exists() {
            std::fs::create_dir_all(dir_path)
                .map_err(|e| CloudflareError::Other(format!("Failed to create directory: {}", e)))?;
        }
        
        // Save each session
        for (id, session) in &self.sessions {
            let file_path = dir_path.join(format!("{}.json", id));
            session.save_to_file(&file_path)?;
        }
        
        Ok(())
    }
    
    /// Load all sessions from a directory
    pub fn load_all(&mut self, dir_path: &PathBuf) -> Result<(), CloudflareError> {
        // Check if directory exists
        if !dir_path.exists() {
            return Ok(());
        }
        
        // Read directory
        let entries = std::fs::read_dir(dir_path)
            .map_err(|e| CloudflareError::Other(format!("Failed to read directory: {}", e)))?;
        
        // Load each session
        for entry in entries {
            let entry = entry.map_err(|e| CloudflareError::Other(format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                match Session::load_from_file(&path) {
                    Ok(session) => {
                        self.sessions.insert(session.id.clone(), session);
                    }
                    Err(e) => {
                        // Just log the error and continue
                        eprintln!("Failed to load session from {}: {}", path.display(), e);
                    }
                }
            }
        }
        
        Ok(())
    }
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