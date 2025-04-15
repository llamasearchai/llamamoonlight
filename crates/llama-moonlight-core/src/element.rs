//! Element handling.
//!
//! This module provides functionality for interacting with DOM elements.

use crate::errors::{Error, Result};
use crate::protocol::Connection;
use crate::page::Page;
use std::sync::Arc;
use log::{debug, info, warn};

/// Represents a handle to a DOM element.
#[derive(Debug)]
pub struct ElementHandle<'a> {
    /// Connection to the browser.
    pub(crate) connection: Arc<Connection>,
    
    /// Session ID.
    pub(crate) session_id: String,
    
    /// Object ID of the element.
    pub(crate) object_id: String,
    
    /// Reference to the page.
    pub(crate) page: &'a Page,
}

impl<'a> ElementHandle<'a> {
    /// Clicks the element.
    pub async fn click(&self) -> Result<()> {
        info!("Clicking element with object ID {}", self.object_id);
        
        // First we need to get the center point of the element for clicking
        let box_model = self.get_box_model().await?;
        
        // Calculate the center point
        let center = self.calculate_center_point(&box_model);
        
        // Send the click event
        let params = serde_json::json!({
            "x": center.0,
            "y": center.1,
            "button": "left",
            "clickCount": 1,
        });
        
        let _ = self.send_session_command("Input.dispatchMouseEvent", Some(params)).await?;
        
        info!("Clicked element with object ID {}", self.object_id);
        Ok(())
    }
    
    /// Types text into the element.
    pub async fn type_text(&self, text: &str) -> Result<()> {
        info!("Typing text into element with object ID {}", self.object_id);
        
        // First focus the element
        let _ = self.focus().await?;
        
        // Then type the text character by character
        for c in text.chars() {
            let params = serde_json::json!({
                "text": c.to_string(),
            });
            
            let _ = self.send_session_command("Input.dispatchKeyEvent", Some(params)).await?;
        }
        
        info!("Typed text into element with object ID {}", self.object_id);
        Ok(())
    }
    
    /// Gets the text content of the element.
    pub async fn text_content(&self) -> Result<String> {
        info!("Getting text content of element with object ID {}", self.object_id);
        
        // Call function on the element to get its text content
        let params = serde_json::json!({
            "objectId": self.object_id,
            "functionDeclaration": "function() { return this.textContent; }",
            "returnByValue": true,
        });
        
        let result = self.send_session_command("Runtime.callFunctionOn", Some(params)).await?;
        
        // Check if there was an error
        if let Some(error) = result["exceptionDetails"].as_object() {
            let error_message = error["exception"]["description"].as_str()
                .unwrap_or("Unknown error getting text content");
            
            return Err(Error::JavaScriptError(error_message.to_string()));
        }
        
        // Get the result value
        let value = result["result"]["value"].as_str()
            .ok_or_else(|| Error::JavaScriptError("Failed to get text content".to_string()))?;
        
        info!("Got text content of element with object ID {}: {}", self.object_id, value);
        Ok(value.to_string())
    }
    
    /// Gets the inner HTML of the element.
    pub async fn inner_html(&self) -> Result<String> {
        info!("Getting inner HTML of element with object ID {}", self.object_id);
        
        // Call function on the element to get its innerHTML
        let params = serde_json::json!({
            "objectId": self.object_id,
            "functionDeclaration": "function() { return this.innerHTML; }",
            "returnByValue": true,
        });
        
        let result = self.send_session_command("Runtime.callFunctionOn", Some(params)).await?;
        
        // Check if there was an error
        if let Some(error) = result["exceptionDetails"].as_object() {
            let error_message = error["exception"]["description"].as_str()
                .unwrap_or("Unknown error getting inner HTML");
            
            return Err(Error::JavaScriptError(error_message.to_string()));
        }
        
        // Get the result value
        let value = result["result"]["value"].as_str()
            .ok_or_else(|| Error::JavaScriptError("Failed to get inner HTML".to_string()))?;
        
        info!("Got inner HTML of element with object ID {}", self.object_id);
        Ok(value.to_string())
    }
    
    /// Gets the outer HTML of the element.
    pub async fn outer_html(&self) -> Result<String> {
        info!("Getting outer HTML of element with object ID {}", self.object_id);
        
        // Call function on the element to get its outerHTML
        let params = serde_json::json!({
            "objectId": self.object_id,
            "functionDeclaration": "function() { return this.outerHTML; }",
            "returnByValue": true,
        });
        
        let result = self.send_session_command("Runtime.callFunctionOn", Some(params)).await?;
        
        // Check if there was an error
        if let Some(error) = result["exceptionDetails"].as_object() {
            let error_message = error["exception"]["description"].as_str()
                .unwrap_or("Unknown error getting outer HTML");
            
            return Err(Error::JavaScriptError(error_message.to_string()));
        }
        
        // Get the result value
        let value = result["result"]["value"].as_str()
            .ok_or_else(|| Error::JavaScriptError("Failed to get outer HTML".to_string()))?;
        
        info!("Got outer HTML of element with object ID {}", self.object_id);
        Ok(value.to_string())
    }
    
    /// Gets the value of an attribute.
    pub async fn get_attribute(&self, name: &str) -> Result<Option<String>> {
        info!("Getting attribute '{}' of element with object ID {}", name, self.object_id);
        
        // Call function on the element to get the attribute value
        let params = serde_json::json!({
            "objectId": self.object_id,
            "functionDeclaration": format!("function() {{ return this.getAttribute('{}'); }}", name),
            "returnByValue": true,
        });
        
        let result = self.send_session_command("Runtime.callFunctionOn", Some(params)).await?;
        
        // Check if there was an error
        if let Some(error) = result["exceptionDetails"].as_object() {
            let error_message = error["exception"]["description"].as_str()
                .unwrap_or("Unknown error getting attribute");
            
            return Err(Error::JavaScriptError(error_message.to_string()));
        }
        
        // Get the result value
        let value = result["result"]["value"].as_str();
        
        match value {
            Some(v) => {
                info!("Got attribute '{}' of element with object ID {}: {}", name, self.object_id, v);
                Ok(Some(v.to_string()))
            },
            None => {
                if result["result"]["value"].is_null() {
                    info!("Attribute '{}' of element with object ID {} not found", name, self.object_id);
                    Ok(None)
                } else {
                    Err(Error::JavaScriptError("Failed to get attribute value".to_string()))
                }
            }
        }
    }
    
    /// Sets the value of an attribute.
    pub async fn set_attribute(&self, name: &str, value: &str) -> Result<()> {
        info!("Setting attribute '{}' of element with object ID {} to '{}'", name, self.object_id, value);
        
        // Call function on the element to set the attribute value
        let params = serde_json::json!({
            "objectId": self.object_id,
            "functionDeclaration": format!("function() {{ this.setAttribute('{}', '{}'); }}", name, value),
            "returnByValue": true,
        });
        
        let result = self.send_session_command("Runtime.callFunctionOn", Some(params)).await?;
        
        // Check if there was an error
        if let Some(error) = result["exceptionDetails"].as_object() {
            let error_message = error["exception"]["description"].as_str()
                .unwrap_or("Unknown error setting attribute");
            
            return Err(Error::JavaScriptError(error_message.to_string()));
        }
        
        info!("Set attribute '{}' of element with object ID {} to '{}'", name, self.object_id, value);
        Ok(())
    }
    
    /// Focuses the element.
    pub async fn focus(&self) -> Result<()> {
        info!("Focusing element with object ID {}", self.object_id);
        
        // Call function on the element to focus it
        let params = serde_json::json!({
            "objectId": self.object_id,
            "functionDeclaration": "function() { this.focus(); }",
            "returnByValue": true,
        });
        
        let result = self.send_session_command("Runtime.callFunctionOn", Some(params)).await?;
        
        // Check if there was an error
        if let Some(error) = result["exceptionDetails"].as_object() {
            let error_message = error["exception"]["description"].as_str()
                .unwrap_or("Unknown error focusing element");
            
            return Err(Error::JavaScriptError(error_message.to_string()));
        }
        
        info!("Focused element with object ID {}", self.object_id);
        Ok(())
    }
    
    /// Takes a screenshot of the element.
    pub async fn screenshot(&self, path: &str) -> Result<()> {
        info!("Taking screenshot of element with object ID {} and saving to {}", self.object_id, path);
        
        // Get the box model of the element
        let box_model = self.get_box_model().await?;
        
        // Get the bounding box
        let clip = serde_json::json!({
            "x": box_model["content"][0].as_f64().unwrap_or(0.0),
            "y": box_model["content"][1].as_f64().unwrap_or(0.0),
            "width": box_model["width"].as_f64().unwrap_or(0.0),
            "height": box_model["height"].as_f64().unwrap_or(0.0),
            "scale": 1,
        });
        
        // Take the screenshot
        let params = serde_json::json!({
            "format": "png",
            "clip": clip,
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
        
        info!("Screenshot of element with object ID {} saved to {}", self.object_id, path);
        Ok(())
    }
    
    /// Gets the box model of the element.
    async fn get_box_model(&self) -> Result<serde_json::Value> {
        // Call function to get the bounding client rect
        let params = serde_json::json!({
            "objectId": self.object_id,
        });
        
        let result = self.send_session_command("DOM.getBoxModel", Some(params)).await?;
        
        let box_model = result["model"].as_object()
            .ok_or_else(|| Error::JavaScriptError("Failed to get box model".to_string()))?;
        
        Ok(serde_json::to_value(box_model).unwrap())
    }
    
    /// Calculates the center point of an element based on its box model.
    fn calculate_center_point(&self, box_model: &serde_json::Value) -> (f64, f64) {
        // Get the content quad points
        let content = box_model["content"].as_array().unwrap_or(&vec![]);
        
        // Calculate the center point
        if content.len() >= 4 {
            let x1 = content[0].as_f64().unwrap_or(0.0);
            let y1 = content[1].as_f64().unwrap_or(0.0);
            let x3 = content[4].as_f64().unwrap_or(0.0);
            let y3 = content[5].as_f64().unwrap_or(0.0);
            
            let center_x = (x1 + x3) / 2.0;
            let center_y = (y1 + y3) / 2.0;
            
            (center_x, center_y)
        } else {
            (0.0, 0.0)
        }
    }
    
    /// Sends a protocol command to the page session.
    async fn send_session_command(&self, method: &str, params: Option<serde_json::Value>) -> Result<serde_json::Value> {
        self.page.send_session_command(method, params).await
    }
} 