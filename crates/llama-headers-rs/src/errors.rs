//! Error types for the llama-headers-rs crate
use thiserror::Error;

/// Errors that can occur in the llama-headers-rs crate
#[derive(Error, Debug, PartialEq)]
pub enum LlamaHeadersError {
    /// Error when the URL is invalid
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    
    /// Error when no User-Agent is available
    #[error("No User-Agent available")]
    NoUserAgentAvailable,
    
    /// Error when no referer is available for the given language
    #[error("No referer available for the given language")]
    NoRefererAvailable,
    
    /// Error wrapping an IO error
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Error wrapping a Regex error
    #[error("Regex Error: {0}")]
    Regex(#[from] regex::Error),
    
    /// Error when parsing a User-Agent string
    #[error("User-Agent parsing error: {0}")]
    UserAgentParsingError(String),
    
    /// Error when handling configuration
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    /// Generic error type
    #[error("Error: {0}")]
    Generic(String),
}

/// Result type alias for llama-headers-rs operations
pub type Result<T> = std::result::Result<T, LlamaHeadersError>;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_display() {
        let err = LlamaHeadersError::InvalidUrl("https://invalid".to_string());
        assert_eq!(format!("{}", err), "Invalid URL: https://invalid");
        
        let err = LlamaHeadersError::NoUserAgentAvailable;
        assert_eq!(format!("{}", err), "No User-Agent available");
        
        let err = LlamaHeadersError::Generic("Test error".to_string());
        assert_eq!(format!("{}", err), "Error: Test error");
    }
    
    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: LlamaHeadersError = io_err.into();
        
        match err {
            LlamaHeadersError::Io(_) => assert!(true),
            _ => panic!("Expected Io error variant"),
        }
    }
} 