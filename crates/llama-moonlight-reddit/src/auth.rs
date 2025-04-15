//! Authentication for the Reddit API
//!
//! This module provides authentication functionality for the Reddit API
//! using the OAuth2 protocol.

use std::sync::Arc;
use std::collections::HashMap;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use reqwest::{Client, StatusCode};
use serde::{Serialize, Deserialize};
use tokio::sync::{RwLock, Mutex};
use log::{debug, info, warn, error};

use crate::{Result, Error, TOKEN_URL};

/// OAuth2 token response from Reddit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    /// Access token for API requests
    pub access_token: Option<String>,
    
    /// Token type (usually "bearer")
    pub token_type: Option<String>,
    
    /// Time in seconds until token expires
    pub expires_in: Option<u64>,
    
    /// Refresh token for obtaining new access tokens
    pub refresh_token: Option<String>,
    
    /// Scope of the token
    pub scope: Option<String>,
    
    /// Client ID used to obtain token
    #[serde(skip)]
    pub client_id: Option<String>,
    
    /// Client secret used to obtain token
    #[serde(skip)]
    pub client_secret: Option<String>,
    
    /// When the token expires
    #[serde(skip)]
    pub expires_at: DateTime<Utc>,
}

impl TokenResponse {
    /// Create a new token response
    pub fn new(
        access_token: Option<String>,
        token_type: Option<String>,
        expires_in: Option<u64>,
        refresh_token: Option<String>,
        scope: Option<String>,
        client_id: Option<String>,
        client_secret: Option<String>,
    ) -> Self {
        // Calculate when the token expires
        let expires_in_duration = expires_in.unwrap_or(3600);
        let expires_at = Utc::now() + Duration::seconds(expires_in_duration as i64);
        
        Self {
            access_token,
            token_type,
            expires_in,
            refresh_token,
            scope,
            client_id,
            client_secret,
            expires_at,
        }
    }
    
    /// Check if the token is still valid
    pub fn is_valid(&self) -> bool {
        if let Some(access_token) = &self.access_token {
            if access_token.is_empty() {
                return false;
            }
            
            let now = Utc::now();
            return now < self.expires_at;
        }
        
        false
    }
    
    /// Time remaining until token expires
    pub fn time_remaining(&self) -> Option<Duration> {
        let now = Utc::now();
        if now < self.expires_at {
            Some(self.expires_at - now)
        } else {
            None
        }
    }
}

/// Credentials for authenticating with Reddit
#[derive(Debug, Clone)]
pub enum Credentials {
    /// Resource Owner Password Credentials
    Password {
        /// Client ID
        client_id: String,
        
        /// Client secret
        client_secret: String,
        
        /// Reddit username
        username: String,
        
        /// Reddit password
        password: String,
        
        /// Requested scopes
        scopes: Vec<String>,
    },
    
    /// Client Credentials
    ClientCredentials {
        /// Client ID
        client_id: String,
        
        /// Client secret
        client_secret: String,
        
        /// Requested scopes
        scopes: Vec<String>,
    },
    
    /// Refresh Token
    RefreshToken {
        /// Client ID
        client_id: String,
        
        /// Client secret
        client_secret: String,
        
        /// Refresh token
        refresh_token: String,
    },
    
    /// Application Only (Installed Apps)
    ApplicationOnly {
        /// Client ID
        client_id: String,
        
        /// Device ID
        device_id: String,
        
        /// Requested scopes
        scopes: Vec<String>,
    },
}

impl Credentials {
    /// Create new password credentials
    pub fn new_password(
        client_id: String,
        client_secret: String,
        username: String,
        password: String,
    ) -> Self {
        Self::Password {
            client_id,
            client_secret,
            username,
            password,
            scopes: vec!["*".to_string()], // Request all scopes by default
        }
    }
    
    /// Create new client credentials
    pub fn new_client_credentials(
        client_id: String,
        client_secret: String,
    ) -> Self {
        Self::ClientCredentials {
            client_id,
            client_secret,
            scopes: vec![],
        }
    }
    
    /// Create new refresh token credentials
    pub fn new_refresh_token(
        client_id: String,
        client_secret: String,
        refresh_token: String,
    ) -> Self {
        Self::RefreshToken {
            client_id,
            client_secret,
            refresh_token,
        }
    }
    
    /// Create new application-only credentials
    pub fn new_application_only(
        client_id: String,
        device_id: String,
    ) -> Self {
        Self::ApplicationOnly {
            client_id,
            device_id,
            scopes: vec![],
        }
    }
    
    /// Add scopes to the credentials
    pub fn with_scopes(mut self, scopes: Vec<String>) -> Self {
        match &mut self {
            Self::Password { scopes: s, .. } => *s = scopes,
            Self::ClientCredentials { scopes: s, .. } => *s = scopes,
            Self::ApplicationOnly { scopes: s, .. } => *s = scopes,
            _ => {}, // Refresh token doesn't have scopes
        }
        
        self
    }
    
    /// Get the client ID
    pub fn client_id(&self) -> &str {
        match self {
            Self::Password { client_id, .. } => client_id,
            Self::ClientCredentials { client_id, .. } => client_id,
            Self::RefreshToken { client_id, .. } => client_id,
            Self::ApplicationOnly { client_id, .. } => client_id,
        }
    }
    
    /// Get the client secret
    pub fn client_secret(&self) -> Option<&str> {
        match self {
            Self::Password { client_secret, .. } => Some(client_secret),
            Self::ClientCredentials { client_secret, .. } => Some(client_secret),
            Self::RefreshToken { client_secret, .. } => Some(client_secret),
            Self::ApplicationOnly { .. } => None,
        }
    }
    
    /// Get the username
    pub fn username(&self) -> Option<&str> {
        match self {
            Self::Password { username, .. } => Some(username),
            _ => None,
        }
    }
}

/// A store for tokens
#[async_trait]
pub trait TokenStore: Send + Sync {
    /// Store a token
    async fn store_token(&self, token: &TokenResponse) -> Result<()>;
    
    /// Get the stored token
    async fn get_token(&self) -> Result<TokenResponse>;
    
    /// Clear the stored token
    async fn clear_token(&self) -> Result<()>;
}

/// A token store that keeps tokens in memory
pub struct MemoryTokenStore {
    /// The stored token
    token: RwLock<Option<TokenResponse>>,
}

impl MemoryTokenStore {
    /// Create a new memory token store
    pub fn new() -> Self {
        Self {
            token: RwLock::new(None),
        }
    }
}

#[async_trait]
impl TokenStore for MemoryTokenStore {
    async fn store_token(&self, token: &TokenResponse) -> Result<()> {
        let mut store = self.token.write().await;
        *store = Some(token.clone());
        Ok(())
    }
    
    async fn get_token(&self) -> Result<TokenResponse> {
        let store = self.token.read().await;
        store.clone().ok_or_else(|| Error::AuthError("No token stored".to_string()))
    }
    
    async fn clear_token(&self) -> Result<()> {
        let mut store = self.token.write().await;
        *store = None;
        Ok(())
    }
}

/// Authenticator for handling Reddit OAuth2 authentication
pub struct Authenticator {
    /// URL for requesting tokens
    token_url: String,
}

impl Authenticator {
    /// Create a new authenticator
    pub fn new() -> Self {
        Self {
            token_url: TOKEN_URL.to_string(),
        }
    }
    
    /// Authenticate with the provided credentials
    pub async fn authenticate(
        &self,
        client: &Client,
        credentials: &Credentials,
    ) -> Result<TokenResponse> {
        // Build the request parameters
        let mut params = HashMap::new();
        
        match credentials {
            Credentials::Password { 
                client_id,
                client_secret,
                username,
                password,
                scopes 
            } => {
                params.insert("grant_type", "password");
                params.insert("username", username);
                params.insert("password", password);
                
                if !scopes.is_empty() {
                    params.insert("scope", &scopes.join(" "));
                }
                
                self.request_token(client, client_id, Some(client_secret), params).await
            }
            
            Credentials::ClientCredentials { 
                client_id,
                client_secret,
                scopes 
            } => {
                params.insert("grant_type", "client_credentials");
                
                if !scopes.is_empty() {
                    params.insert("scope", &scopes.join(" "));
                }
                
                self.request_token(client, client_id, Some(client_secret), params).await
            }
            
            Credentials::RefreshToken { 
                client_id,
                client_secret,
                refresh_token 
            } => {
                params.insert("grant_type", "refresh_token");
                params.insert("refresh_token", refresh_token);
                
                self.request_token(client, client_id, Some(client_secret), params).await
            }
            
            Credentials::ApplicationOnly { 
                client_id,
                device_id,
                scopes 
            } => {
                params.insert("grant_type", "https://oauth.reddit.com/grants/installed_client");
                params.insert("device_id", device_id);
                
                if !scopes.is_empty() {
                    params.insert("scope", &scopes.join(" "));
                }
                
                self.request_token(client, client_id, None, params).await
            }
        }
    }
    
    /// Make a token request to Reddit
    async fn request_token(
        &self,
        client: &Client,
        client_id: &str,
        client_secret: Option<&str>,
        params: HashMap<&str, &str>,
    ) -> Result<TokenResponse> {
        // Create the request
        let mut request_builder = client.post(&self.token_url);
        
        // Add basic auth if client_secret is provided
        if let Some(client_secret) = client_secret {
            request_builder = request_builder.basic_auth(client_id, Some(client_secret));
        } else {
            // Otherwise, add client_id as a parameter
            let mut params = params.clone();
            params.insert("client_id", client_id);
            request_builder = request_builder.form(&params);
        }
        
        // Add parameters
        request_builder = request_builder.form(&params);
        
        // Send the request
        let response = request_builder.send().await?;
        
        // Check for errors
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            
            return Err(Error::AuthError(format!(
                "Token request failed with status {}: {}",
                status, body
            )));
        }
        
        // Parse the response
        let mut token: TokenResponse = response.json().await?;
        
        // Add client credentials to the token
        token.client_id = Some(client_id.to_string());
        token.client_secret = client_secret.map(|s| s.to_string());
        
        Ok(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_token_response() {
        let token = TokenResponse::new(
            Some("access_token".to_string()),
            Some("bearer".to_string()),
            Some(3600),
            Some("refresh_token".to_string()),
            Some("read write".to_string()),
            Some("client_id".to_string()),
            Some("client_secret".to_string()),
        );
        
        assert!(token.is_valid());
        assert!(token.time_remaining().is_some());
    }
    
    #[test]
    fn test_credentials() {
        let password_creds = Credentials::new_password(
            "client_id".to_string(),
            "client_secret".to_string(),
            "username".to_string(),
            "password".to_string(),
        );
        
        let refresh_creds = Credentials::new_refresh_token(
            "client_id".to_string(),
            "client_secret".to_string(),
            "refresh_token".to_string(),
        );
        
        let client_creds = Credentials::new_client_credentials(
            "client_id".to_string(),
            "client_secret".to_string(),
        ).with_scopes(vec!["read".to_string(), "write".to_string()]);
        
        let app_creds = Credentials::new_application_only(
            "client_id".to_string(),
            "device_id".to_string(),
        );
        
        assert_eq!(password_creds.client_id(), "client_id");
        assert_eq!(password_creds.client_secret(), Some("client_secret"));
        assert_eq!(password_creds.username(), Some("username"));
        
        assert_eq!(refresh_creds.client_id(), "client_id");
        assert_eq!(refresh_creds.client_secret(), Some("client_secret"));
        assert_eq!(refresh_creds.username(), None);
        
        assert_eq!(client_creds.client_id(), "client_id");
        
        assert_eq!(app_creds.client_id(), "client_id");
        assert_eq!(app_creds.client_secret(), None);
    }
} 