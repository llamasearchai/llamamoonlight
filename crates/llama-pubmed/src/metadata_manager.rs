use chrono::{DateTime, Utc};
use log::{debug, error, info};
use pdf::file::File as PdfFile;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MetadataError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("PDF parsing error: {0}")]
    PdfParseError(#[from] pdf::error::PdfError),
    
    #[error("Metadata extraction error: {0}")]
    ExtractionError(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaperMetadata {
    pub pmid: String,
    pub title: String,
    pub authors: Vec<String>,
    pub journal: String,
    pub publication_date: DateTime<Utc>,
    pub abstract_text: String,
}

impl PaperMetadata {
    pub fn new(pmid: &str) -> Self {
        Self {
            pmid: pmid.to_string(),
            title: String::new(),
            authors: Vec::new(),
            journal: String::new(),
            publication_date: Utc::now(),
            abstract_text: String::new(),
        }
    }

    pub fn from_pdf(path: &Path) -> Result<Self, MetadataError> {
        let file = PdfFile::open(path)?;
        let mut metadata = PaperMetadata::new("");

        // Extract metadata from the PDF file
        // This is a placeholder for actual extraction logic
        metadata.title = "Extracted Title".to_string();
        metadata.authors = vec!["Author One".to_string(), "Author Two".to_string()];
        metadata.journal = "Journal Name".to_string();
        metadata.abstract_text = "This is an abstract.".to_string();

        Ok(metadata)
    }

    pub fn save_to_file(&self, path: &Path) -> Result<(), MetadataError> {
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, &self)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_metadata_creation() {
        let metadata = PaperMetadata::new("1234567");
        assert_eq!(metadata.pmid, "1234567");
    }

    #[test]
    fn test_metadata_extraction() {
        let temp_dir = tempdir().unwrap();
        let pdf_path = temp_dir.path().join("test.pdf");
        let metadata = PaperMetadata::from_pdf(&pdf_path);
        assert!(metadata.is_ok());
    }

    #[test]
    fn test_metadata_save() {
        let temp_dir = tempdir().unwrap();
        let metadata = PaperMetadata::new("1234567");
        let json_path = temp_dir.path().join("metadata.json");
        let result = metadata.save_to_file(&json_path);
        assert!(result.is_ok());
        assert!(json_path.exists());
    }
} 