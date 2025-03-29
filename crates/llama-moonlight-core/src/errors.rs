//! Error types for the llama-moonlight-core crate
use thiserror::Error;
use crate::protocol::ProtocolError;
use llama_headers_rs::errors::LlamaHeadersError;

/// Errors that can occur in the llama-moonlight-core crate
#[derive(Error, Debug)]
pub enum Error {
    /// Error during browser launch
    #[error("Failed to launch browser: {0}")]
    BrowserLaunchError(String),
    
    /// Error when browser context creation fails
    #[error("Failed to create browser context: {0}")]
    ContextCreationError(String),
    
    /// Error when page creation fails
    #[error("Failed to create page: {0}")]
    PageCreationError(String),
    
    /// Error when navigation fails
    #[error("Navigation error: {0}")]
    NavigationError(String),
    
    /// Error when an element cannot be found
    #[error("Element not found: {0}")]
    ElementNotFoundError(String),
    
    /// Error when timeout occurs
    #[error("Timeout: {0}")]
    TimeoutError(String),
    
    /// Error when the specified browser type is not found
    #[error("Browser type not found: {0}")]
    BrowserTypeNotFound(String),
    
    /// Error in protocol communication
    #[error("Protocol error: {0}")]
    ProtocolError(#[from] ProtocolError),
    
    /// Error from llama-headers-rs
    #[error("Headers error: {0}")]
    HeadersError(#[from] LlamaHeadersError),
    
    /// Error from JavaScript evaluation
    #[error("JavaScript evaluation error: {0}")]
    JavaScriptError(String),
    
    /// Error when screenshot fails
    #[error("Screenshot error: {0}")]
    ScreenshotError(String),
    
    /// Error when file operation fails
    #[error("File error: {0}")]
    FileError(#[from] std::io::Error),
    
    /// Error from JSON serialization/deserialization
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    /// Generic error type
    #[error("Error: {0}")]
    Generic(String),
}

/// Result type alias for llama-moonlight-core operations
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_display() {
        let err = Error::BrowserLaunchError("Chrome not found".to_string());
        assert_eq!(format!("{}", err), "Failed to launch browser: Chrome not found");
        
        let err = Error::ElementNotFoundError("div#main".to_string());
        assert_eq!(format!("{}", err), "Element not found: div#main");
        
        let err = Error::Generic("Test error".to_string());
        assert_eq!(format!("{}", err), "Error: Test error");
    }
    
    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: Error = io_err.into();
        
        match err {
            Error::FileError(_) => assert!(true),
            _ => panic!("Expected FileError variant"),
        }
    }
} 