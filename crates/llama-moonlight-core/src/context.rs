//! Browser context management.
//!
//! This module provides functionality for browser contexts, which are similar to incognito windows.

use crate::errors::{Error, Result};
use crate::page::Page;
use crate::protocol::Connection;
use crate::options::{ContextOptions, PageOptions};
use std::sync::Arc;
use log::{debug, info};

/// Represents a browser context (similar to an incognito window).
#[derive(Debug)]
pub struct BrowserContext {
    /// Connection to the browser.
    pub(crate) connection: Arc<Connection>,
    
    /// Context ID.
    pub(crate) id: String,
    
    /// Browser type.
    pub(crate) browser_type: String,
    
    /// Context options.
    pub(crate) options: ContextOptions,
}

impl BrowserContext {
    /// Creates a new page in the context.
    pub async fn new_page(&self) -> Result<Page> {
        info!("Creating new page in context {}", self.id);
        
        let params = serde_json::json!({
            "contextId": self.id,
        });
        
        let result = self.connection.send_request(
            "Target.createPage".to_string(),
            Some(params),
        ).await?;
        
        let target_id = result["targetId"].as_str()
            .ok_or_else(|| Error::PageCreationError("Failed to get target ID".to_string()))?
            .to_string();
        
        // Attach to the page
        let params = serde_json::json!({
            "targetId": target_id,
        });
        
        let result = self.connection.send_request(
            "Target.attachToTarget".to_string(),
            Some(params),
        ).await?;
        
        let session_id = result["sessionId"].as_str()
            .ok_or_else(|| Error::PageCreationError("Failed to get session ID".to_string()))?
            .to_string();
        
        let page = Page {
            connection: self.connection.clone(),
            target_id,
            session_id,
            context_id: self.id.clone(),
            browser_type: self.browser_type.clone(),
            options: PageOptions::default(),
        };
        
        info!("Successfully created page in context {}", self.id);
        Ok(page)
    }
    
    /// Creates a new page with the specified options.
    pub async fn new_page_with_options(&self, options: PageOptions) -> Result<Page> {
        info!("Creating new page with options in context {}", self.id);
        
        let params = serde_json::json!({
            "contextId": self.id,
        });
        
        let result = self.connection.send_request(
            "Target.createPage".to_string(),
            Some(params),
        ).await?;
        
        let target_id = result["targetId"].as_str()
            .ok_or_else(|| Error::PageCreationError("Failed to get target ID".to_string()))?
            .to_string();
        
        // Attach to the page
        let params = serde_json::json!({
            "targetId": target_id,
        });
        
        let result = self.connection.send_request(
            "Target.attachToTarget".to_string(),
            Some(params),
        ).await?;
        
        let session_id = result["sessionId"].as_str()
            .ok_or_else(|| Error::PageCreationError("Failed to get session ID".to_string()))?
            .to_string();
        
        let page = Page {
            connection: self.connection.clone(),
            target_id,
            session_id,
            context_id: self.id.clone(),
            browser_type: self.browser_type.clone(),
            options,
        };
        
        info!("Successfully created page with options in context {}", self.id);
        Ok(page)
    }
    
    /// Closes the context.
    pub async fn close(&self) -> Result<()> {
        info!("Closing context {}", self.id);
        
        let params = serde_json::json!({
            "contextId": self.id,
        });
        
        let _ = self.connection.send_request(
            "Target.disposeBrowserContext".to_string(),
            Some(params),
        ).await?;
        
        info!("Context {} closed", self.id);
        Ok(())
    }
    
    /// Returns the context ID.
    pub fn id(&self) -> &str {
        &self.id
    }
    
    /// Sets cookies for the context.
    pub async fn set_cookies(&self, cookies: Vec<crate::options::Cookie>) -> Result<()> {
        info!("Setting cookies for context {}", self.id);
        
        let params = serde_json::json!({
            "contextId": self.id,
            "cookies": cookies,
        });
        
        let _ = self.connection.send_request(
            "Network.setCookies".to_string(),
            Some(params),
        ).await?;
        
        info!("Cookies set for context {}", self.id);
        Ok(())
    }
    
    /// Clears cookies for the context.
    pub async fn clear_cookies(&self) -> Result<()> {
        info!("Clearing cookies for context {}", self.id);
        
        let params = serde_json::json!({
            "contextId": self.id,
        });
        
        let _ = self.connection.send_request(
            "Network.clearBrowserCookies".to_string(),
            Some(params),
        ).await?;
        
        info!("Cookies cleared for context {}", self.id);
        Ok(())
    }
    
    /// Grants permissions to the context.
    pub async fn grant_permissions(&self, permissions: Vec<String>) -> Result<()> {
        info!("Granting permissions for context {}", self.id);
        
        let params = serde_json::json!({
            "contextId": self.id,
            "permissions": permissions,
        });
        
        let _ = self.connection.send_request(
            "Browser.grantPermissions".to_string(),
            Some(params),
        ).await?;
        
        info!("Permissions granted for context {}", self.id);
        Ok(())
    }
    
    /// Sets geolocation for the context.
    pub async fn set_geolocation(&self, geolocation: crate::options::Geolocation) -> Result<()> {
        info!("Setting geolocation for context {}", self.id);
        
        let params = serde_json::json!({
            "contextId": self.id,
            "geolocation": geolocation,
        });
        
        let _ = self.connection.send_request(
            "Emulation.setGeolocationOverride".to_string(),
            Some(params),
        ).await?;
        
        info!("Geolocation set for context {}", self.id);
        Ok(())
    }
    
    /// Sets the color scheme for the context.
    pub async fn set_color_scheme(&self, color_scheme: crate::options::ColorScheme) -> Result<()> {
        info!("Setting color scheme for context {}", self.id);
        
        let scheme = match color_scheme {
            crate::options::ColorScheme::Light => "light",
            crate::options::ColorScheme::Dark => "dark",
            crate::options::ColorScheme::NoPreference => "no-preference",
        };
        
        let params = serde_json::json!({
            "contextId": self.id,
            "colorScheme": scheme,
        });
        
        let _ = self.connection.send_request(
            "Emulation.setEmulatedMedia".to_string(),
            Some(params),
        ).await?;
        
        info!("Color scheme set for context {}", self.id);
        Ok(())
    }
    
    /// Exports the HAR (HTTP Archive) for the context.
    pub async fn export_har(&self, path: &str) -> Result<()> {
        info!("Exporting HAR for context {}", self.id);
        
        let params = serde_json::json!({
            "contextId": self.id,
            "path": path,
        });
        
        let _ = self.connection.send_request(
            "Network.exportHAR".to_string(),
            Some(params),
        ).await?;
        
        info!("HAR exported for context {}", self.id);
        Ok(())
    }
} 