use std::io;
use std::fmt;
use std::path::PathBuf;
use thiserror::Error;
use url::ParseError as UrlParseError;

/// Unified error type for the llama-arxiv application
#[derive(Error, Debug)]
pub enum ArxivError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    #[error("URL parse error: {0}")]
    UrlParse(#[from] UrlParseError),
    
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
    
    #[error("API response error: {0}")]
    ApiResponse(String),
    
    #[error("PDF processing error: {0}")]
    PdfProcessing(String),
    
    #[error("Invalid arXiv ID: {0}")]
    InvalidArxivId(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("File already exists: {0}")]
    FileExists(PathBuf),
    
    #[error("XML parsing error: {0}")]
    XmlParsing(String),
    
    #[error("No results found for query")]
    NoResults,
    
    #[error("Operation timed out")]
    Timeout,
}

impl ArxivError {
    /// Create an API response error
    pub fn api_error(message: impl Into<String>) -> Self {
        ArxivError::ApiResponse(message.into())
    }
    
    /// Create a PDF processing error
    pub fn pdf_error(message: impl Into<String>) -> Self {
        ArxivError::PdfProcessing(message.into())
    }
    
    /// Create a configuration error
    pub fn config_error(message: impl Into<String>) -> Self {
        ArxivError::Configuration(message.into())
    }
    
    /// Create an invalid ID error
    pub fn invalid_id(id: impl Into<String>) -> Self {
        ArxivError::InvalidArxivId(id.into())
    }
    
    /// Create an XML parsing error
    pub fn xml_error(message: impl Into<String>) -> Self {
        ArxivError::XmlParsing(message.into())
    }
    
    /// Check if the error is a "not found" error
    pub fn is_not_found(&self) -> bool {
        matches!(self, ArxivError::NoResults)
    }
    
    /// Check if the error is a timeout error
    pub fn is_timeout(&self) -> bool {
        matches!(self, ArxivError::Timeout) || 
        matches!(self, ArxivError::Request(e) if e.is_timeout())
    }
}

impl From<&str> for ArxivError {
    fn from(error: &str) -> Self {
        ArxivError::Other(error.to_string())
    }
}

impl From<String> for ArxivError {
    fn from(error: String) -> Self {
        ArxivError::Other(error)
    }
}

/// Extension for converting errors to ArxivError
pub trait IntoArxivError<T> {
    /// Convert an error into ArxivError
    fn into_arxiv_error(self, context: &str) -> Result<T, ArxivError>;
}

impl<T, E: std::fmt::Display> IntoArxivError<T> for Result<T, E> {
    fn into_arxiv_error(self, context: &str) -> Result<T, ArxivError> {
        self.map_err(|e| ArxivError::Other(format!("{}: {}", context, e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let arxiv_err: ArxivError = io_err.into();
        
        match arxiv_err {
            ArxivError::Io(_) => assert!(true),
            _ => assert!(false, "Wrong error conversion"),
        }
    }
    
    #[test]
    fn test_into_arxiv_error() {
        let result: Result<(), std::io::Error> = 
            Err(std::io::Error::new(std::io::ErrorKind::Other, "test error"));
            
        let arxiv_result = result.into_arxiv_error("Context message");
        assert!(arxiv_result.is_err());
        
        if let Err(e) = arxiv_result {
            match e {
                ArxivError::Other(msg) => {
                    assert!(msg.contains("Context message"));
                    assert!(msg.contains("test error"));
                },
                _ => assert!(false, "Wrong error conversion"),
            }
        }
    }
} 