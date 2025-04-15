//! Page management.
//!
//! This module provides functionality for interacting with pages.

use crate::errors::{Error, Result};
use crate::element::ElementHandle;
use crate::protocol::Connection;
use crate::options::PageOptions;
use std::sync::Arc;
use log::{debug, info, warn};
use tokio::time::{timeout, Duration};
use std::path::Path;

/// Represents a page in a browser.
#[derive(Debug)]
pub struct Page {
    /// Connection to the browser.
    pub(crate) connection: Arc<Connection>,
    
    /// Target ID.
    pub(crate) target_id: String,
    
    /// Session ID.
    pub(crate) session_id: String,
    
    /// Context ID.
    pub(crate) context_id: String,
    
    /// Browser type.
    pub(crate) browser_type: String,
    
    /// Page options.
    pub(crate) options: PageOptions,
}

impl Page {
    /// Navigates to the specified URL.
    pub async fn goto(&self, url: &str) -> Result<()> {
        info!("Navigating to {}", url);
        
        let timeout_ms = self.options.navigation_timeout_ms.unwrap_or(30000);
        
        let params = serde_json::json!({
            "url": url,
        });
        
        let timeout_future = timeout(
            Duration::from_millis(timeout_ms),
            self.send_session_command("Page.navigate", Some(params)),
        );
        
        match timeout_future.await {
            Ok(result) => {
                match result {
                    Ok(_) => {
                        info!("Successfully navigated to {}", url);
                        
                        // Wait for the page to be loaded
                        if let Some(wait_until) = &self.options.wait_until {
                            self.wait_for_navigation(wait_until).await?;
                        }
                        
                        Ok(())
                    },
                    Err(e) => {
                        warn!("Failed to navigate to {}: {}", url, e);
                        Err(Error::NavigationError(format!("Failed to navigate to {}: {}", url, e)))
                    }
                }
            },
            Err(_) => {
                warn!("Navigation to {} timed out after {}ms", url, timeout_ms);
                Err(Error::TimeoutError(format!("Navigation to {} timed out after {}ms", url, timeout_ms)))
            }
        }
    }
    
    /// Waits for navigation to complete.
    async fn wait_for_navigation(&self, wait_until: &crate::options::WaitUntilState) -> Result<()> {
        let event = match wait_until {
            crate::options::WaitUntilState::Load => "Page.loadEventFired",
            crate::options::WaitUntilState::DomContentLoaded => "Page.domContentEventFired",
            _ => "Page.loadEventFired", // Default to load for other states
        };
        
        // Enable page events
        let _ = self.send_session_command("Page.enable", None).await?;
        
        // Subscribe to the event
        let mut event_receiver = self.connection.subscribe(event.to_string()).await?;
        
        // Wait for the event
        let timeout_ms = self.options.navigation_timeout_ms.unwrap_or(30000);
        match timeout(Duration::from_millis(timeout_ms), event_receiver.recv()).await {
            Ok(Some(_)) => {
                debug!("Navigation completed: event {} received", event);
                Ok(())
            },
            Ok(None) => {
                warn!("Event channel closed before receiving {}", event);
                Err(Error::NavigationError("Event channel closed".to_string()))
            },
            Err(_) => {
                warn!("Waiting for {} timed out after {}ms", event, timeout_ms);
                Err(Error::TimeoutError(format!("Waiting for {} timed out after {}ms", event, timeout_ms)))
            }
        }
    }
    
    /// Takes a screenshot of the page.
    pub async fn screenshot(&self, path: &str) -> Result<()> {
        info!("Taking screenshot and saving to {}", path);
        
        let params = serde_json::json!({
            "format": "png",
            "fullPage": true,
        });
        
        let result = self.send_session_command("Page.captureScreenshot", Some(params)).await?;
        
        let data = result["data"].as_str()
            .ok_or_else(|| Error::ScreenshotError("Failed to get screenshot data".to_string()))?;
        
        // Decode the base64 data
        let decoded = base64::decode(data)
            .map_err(|e| Error::ScreenshotError(format!("Failed to decode base64 data: {}", e)))?;
        
        // Save to file
        std::fs::write(path, decoded)
            .map_err(|e| Error::FileError(e))?;
        
        info!("Screenshot saved to {}", path);
        Ok(())
    }
    
    /// Closes the page.
    pub async fn close(&self) -> Result<()> {
        info!("Closing page {}", self.target_id);
        
        let params = serde_json::json!({
            "targetId": self.target_id,
        });
        
        let _ = self.connection.send_request(
            "Target.closeTarget".to_string(),
            Some(params),
        ).await?;
        
        info!("Page {} closed", self.target_id);
        Ok(())
    }
    
    /// Finds an element matching the specified selector.
    pub async fn query_selector(&self, selector: &str) -> Result<Option<ElementHandle>> {
        info!("Finding element with selector '{}'", selector);
        
        // First we need to evaluate the document.querySelector in the page
        let expression = format!(r#"document.querySelector("{}")"#, escape_string(selector));
        
        let params = serde_json::json!({
            "expression": expression,
            "returnByValue": false,
            "awaitPromise": true,
        });
        
        let result = self.send_session_command("Runtime.evaluate", Some(params)).await?;
        
        // Check if the element was found
        if let Some(error) = result["exceptionDetails"].as_object() {
            let error_message = error["exception"]["description"].as_str()
                .unwrap_or("Unknown error during element selection");
            
            return Err(Error::JavaScriptError(error_message.to_string()));
        }
        
        let object_id = result["result"]["objectId"].as_str();
        
        if let Some(id) = object_id {
            // Create an ElementHandle with the object ID
            let element = ElementHandle {
                connection: self.connection.clone(),
                session_id: self.session_id.clone(),
                object_id: id.to_string(),
                page: self,
            };
            
            Ok(Some(element))
        } else {
            // No element found
            Ok(None)
        }
    }
    
    /// Finds all elements matching the specified selector.
    pub async fn query_selector_all(&self, selector: &str) -> Result<Vec<ElementHandle>> {
        info!("Finding all elements with selector '{}'", selector);
        
        // Evaluate document.querySelectorAll in the page
        let expression = format!(r#"Array.from(document.querySelectorAll("{}"))"#, escape_string(selector));
        
        let params = serde_json::json!({
            "expression": expression,
            "returnByValue": false,
            "awaitPromise": true,
        });
        
        let result = self.send_session_command("Runtime.evaluate", Some(params)).await?;
        
        // Check if there was an error
        if let Some(error) = result["exceptionDetails"].as_object() {
            let error_message = error["exception"]["description"].as_str()
                .unwrap_or("Unknown error during element selection");
            
            return Err(Error::JavaScriptError(error_message.to_string()));
        }
        
        // Get the array object ID
        let array_id = result["result"]["objectId"].as_str()
            .ok_or_else(|| Error::ElementNotFoundError("Failed to get array object ID".to_string()))?;
        
        // Get the properties of the array
        let params = serde_json::json!({
            "objectId": array_id,
            "ownProperties": true,
        });
        
        let result = self.send_session_command("Runtime.getProperties", Some(params)).await?;
        
        // Process the properties to find elements
        let properties = result["result"].as_array()
            .ok_or_else(|| Error::ElementNotFoundError("Failed to get array properties".to_string()))?;
        
        let mut elements = Vec::new();
        
        for property in properties {
            if let Some(object_id) = property["value"]["objectId"].as_str() {
                // Check if this is an element
                let params = serde_json::json!({
                    "objectId": object_id,
                    "expression": "this instanceof Element",
                    "returnByValue": true,
                });
                
                let result = self.send_session_command("Runtime.callFunctionOn", Some(params)).await?;
                
                if let Some(is_element) = result["result"]["value"].as_bool() {
                    if is_element {
                        // Create an ElementHandle
                        let element = ElementHandle {
                            connection: self.connection.clone(),
                            session_id: self.session_id.clone(),
                            object_id: object_id.to_string(),
                            page: self,
                        };
                        
                        elements.push(element);
                    }
                }
            }
        }
        
        info!("Found {} elements with selector '{}'", elements.len(), selector);
        Ok(elements)
    }
    
    /// Evaluates JavaScript code in the page context.
    pub async fn evaluate<T: serde::de::DeserializeOwned>(&self, expression: &str) -> Result<T> {
        info!("Evaluating JavaScript in page: {}", expression);
        
        let params = serde_json::json!({
            "expression": expression,
            "returnByValue": true,
            "awaitPromise": true,
        });
        
        let result = self.send_session_command("Runtime.evaluate", Some(params)).await?;
        
        // Check if there was an error
        if let Some(error) = result["exceptionDetails"].as_object() {
            let error_message = error["exception"]["description"].as_str()
                .unwrap_or("Unknown error during JavaScript evaluation");
            
            return Err(Error::JavaScriptError(error_message.to_string()));
        }
        
        // Get the result value
        let value = &result["result"]["value"];
        
        serde_json::from_value(value.clone())
            .map_err(|e| Error::JavaScriptError(format!("Failed to deserialize result: {}", e)))
    }
    
    /// Returns the page title.
    pub async fn title(&self) -> Result<String> {
        info!("Getting page title");
        
        let title: String = self.evaluate("document.title").await?;
        
        info!("Page title: {}", title);
        Ok(title)
    }
    
    /// Returns the page URL.
    pub async fn url(&self) -> Result<String> {
        info!("Getting page URL");
        
        let url: String = self.evaluate("window.location.href").await?;
        
        info!("Page URL: {}", url);
        Ok(url)
    }
    
    /// Returns the page content (HTML).
    pub async fn content(&self) -> Result<String> {
        info!("Getting page content");
        
        let content: String = self.evaluate("document.documentElement.outerHTML").await?;
        
        info!("Retrieved page content ({} characters)", content.len());
        Ok(content)
    }
    
    /// Sets the page content (HTML).
    pub async fn set_content(&self, html: &str) -> Result<()> {
        info!("Setting page content");
        
        // Escape the HTML string for JavaScript
        let escaped_html = escape_string(html);
        
        let expression = format!("document.open(); document.write(\"{}\"); document.close();", escaped_html);
        
        let _: () = self.evaluate(&expression).await?;
        
        info!("Page content set");
        Ok(())
    }
    
    /// Waits for a selector to appear in the page.
    pub async fn wait_for_selector(&self, selector: &str, timeout_ms: Option<u64>) -> Result<Option<ElementHandle>> {
        info!("Waiting for selector '{}'", selector);
        
        let timeout_ms = timeout_ms.unwrap_or_else(|| self.options.timeout_ms.unwrap_or(30000));
        
        // Create a poll function that checks for the selector
        let poll_function = async {
            let mut result = None;
            
            while result.is_none() {
                match self.query_selector(selector).await {
                    Ok(Some(element)) => {
                        result = Some(Ok(Some(element)));
                        break;
                    },
                    Ok(None) => {
                        // Element not found yet, wait a bit and try again
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    },
                    Err(e) => {
                        result = Some(Err(e));
                        break;
                    }
                }
            }
            
            result.unwrap()
        };
        
        // Run the poll function with timeout
        match timeout(Duration::from_millis(timeout_ms), poll_function).await {
            Ok(result) => result,
            Err(_) => {
                warn!("Waiting for selector '{}' timed out after {}ms", selector, timeout_ms);
                Err(Error::TimeoutError(format!("Waiting for selector '{}' timed out after {}ms", selector, timeout_ms)))
            }
        }
    }
    
    /// Clicks on an element matching the selector.
    pub async fn click(&self, selector: &str) -> Result<()> {
        info!("Clicking on element with selector '{}'", selector);
        
        // Find the element
        let element = self.query_selector(selector).await?
            .ok_or_else(|| Error::ElementNotFoundError(format!("Element with selector '{}' not found", selector)))?;
        
        // Click the element
        element.click().await?;
        
        info!("Clicked on element with selector '{}'", selector);
        Ok(())
    }
    
    /// Types text into an element matching the selector.
    pub async fn type_text(&self, selector: &str, text: &str) -> Result<()> {
        info!("Typing text into element with selector '{}'", selector);
        
        // Find the element
        let element = self.query_selector(selector).await?
            .ok_or_else(|| Error::ElementNotFoundError(format!("Element with selector '{}' not found", selector)))?;
        
        // Type text into the element
        element.type_text(text).await?;
        
        info!("Typed text into element with selector '{}'", selector);
        Ok(())
    }
    
    /// Sends a protocol command to the page session.
    async fn send_session_command(&self, method: &str, params: Option<serde_json::Value>) -> Result<serde_json::Value> {
        // For session commands, we need to wrap the method and params in a Target.sendMessageToTarget command
        let session_params = serde_json::json!({
            "sessionId": self.session_id,
            "message": serde_json::to_string(&serde_json::json!({
                "id": 1,
                "method": method,
                "params": params.unwrap_or(serde_json::json!({})),
            })).unwrap(),
        });
        
        self.connection.send_request(
            "Target.sendMessageToTarget".to_string(),
            Some(session_params),
        ).await.map_err(Error::ProtocolError)
    }
}

/// Escapes a string for JavaScript.
fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace('"', "\\\"")
     .replace('\n', "\\n")
     .replace('\r', "\\r")
     .replace('\t', "\\t")
} 