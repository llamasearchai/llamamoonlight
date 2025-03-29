use thiserror::Error;

#[derive(Error, Debug)]
pub enum LlamaError {
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("PDF extraction error: {0}")]
    PdfExtractionError(String),
    
    #[error("PubMed API error: {0}")]
    PubMedApiError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    
    #[error("PDF download failed: {0}")]
    DownloadError(String),
    
    #[error("PDF not found for PMID: {0}")]
    PdfNotFound(String),
    
    #[error("XML parsing error: {0}")]
    XmlParsingError(#[from] quick_xml::DeError),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_yaml::Error),
    
    #[error("Other error: {0}")]
    Other(String),
}

// Helper function to log and handle errors
pub fn handle_error(error: &LlamaError) {
    use colored::Colorize;
    use log::error;
    
    error!("{}", error);
    
    match error {
        LlamaError::NetworkError(e) => {
            println!("{} {}", "Network Error:".bright_red(), e);
            println!("Please check your internet connection and try again.");
        },
        LlamaError::IoError(e) => {
            println!("{} {}", "I/O Error:".bright_red(), e);
            println!("There was a problem reading/writing files. Check permissions and disk space.");
        },
        LlamaError::PdfExtractionError(msg) => {
            println!("{} {}", "PDF Extraction Error:".bright_red(), msg);
        },
        LlamaError::PubMedApiError(msg) => {
            println!("{} {}", "PubMed API Error:".bright_red(), msg);
            println!("The PubMed API may be temporarily unavailable or has changed.");
        },
        LlamaError::ConfigError(msg) => {
            println!("{} {}", "Configuration Error:".bright_red(), msg);
            println!("Try resetting your configuration with 'llamapubmed config reset'");
        },
        LlamaError::InvalidArgument(msg) => {
            println!("{} {}", "Invalid Argument:".bright_red(), msg);
            println!("Check the command syntax and try again.");
        },
        LlamaError::DownloadError(msg) => {
            println!("{} {}", "Download Error:".bright_red(), msg);
        },
        LlamaError::PdfNotFound(pmid) => {
            println!("{} {}", "PDF Not Found:".bright_red(), pmid);
            println!("The article may not have an accessible PDF version.");
        },
        LlamaError::XmlParsingError(e) => {
            println!("{} {}", "XML Parsing Error:".bright_red(), e);
            println!("The response from PubMed could not be parsed correctly.");
        },
        LlamaError::SerializationError(e) => {
            println!("{} {}", "Serialization Error:".bright_red(), e);
        },
        LlamaError::Other(msg) => {
            println!("{} {}", "Error:".bright_red(), msg);
        },
    }
}

// Convert any error type to Box<dyn std::error::Error>
pub fn convert_error<E: std::error::Error + 'static>(err: E) -> Box<dyn std::error::Error> {
    Box::new(err)
} 