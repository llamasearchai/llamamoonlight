use std::path::{Path, PathBuf};
use std::fs;
use std::io::Write;
use reqwest::{Client, header};
use thiserror::Error;
use log::{debug, info, warn, error};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};

use crate::modules::metadata::PaperMetadata;
use crate::modules::config::DownloadConfig;

/// Error types for PDF downloading operations
#[derive(Error, Debug)]
pub enum DownloadError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),
    
    #[error("PDF download failed: {0}")]
    DownloadFailed(String),
    
    #[error("File already exists: {0}")]
    FileExists(String),
}

/// Result type for download operations
pub type DownloadResult<T> = Result<T, DownloadError>;

/// Structure to store download success/failure information
#[derive(Debug)]
pub struct DownloadInfo {
    /// ID of the paper
    pub id: String,
    
    /// Local path to the downloaded file, if successful
    pub local_path: Option<PathBuf>,
    
    /// Error message in case of failure
    pub error: Option<String>,
    
    /// Whether the download was skipped (e.g., file exists)
    pub skipped: bool,
}

impl DownloadInfo {
    /// Check if the download was successful
    pub fn is_success(&self) -> bool {
        self.local_path.is_some() && self.error.is_none()
    }
    
    /// Create a new download info for a successful download
    pub fn success(id: &str, path: PathBuf) -> Self {
        Self {
            id: id.to_string(),
            local_path: Some(path),
            error: None,
            skipped: false,
        }
    }
    
    /// Create a new download info for a failed download
    pub fn failure(id: &str, error: impl ToString) -> Self {
        Self {
            id: id.to_string(),
            local_path: None,
            error: Some(error.to_string()),
            skipped: false,
        }
    }
    
    /// Create a new download info for a skipped download
    pub fn skipped(id: &str, path: PathBuf) -> Self {
        Self {
            id: id.to_string(),
            local_path: Some(path),
            error: None,
            skipped: true,
        }
    }
}

/// PDF downloader for arXiv papers
pub struct PdfDownloader {
    /// HTTP client
    client: Client,
    
    /// Download configuration
    config: DownloadConfig,
}

impl PdfDownloader {
    /// Create a new PDF downloader
    pub fn new(config: DownloadConfig) -> DownloadResult<Self> {
        // Create download directory if it doesn't exist
        if !Path::new(&config.download_dir).exists() {
            fs::create_dir_all(&config.download_dir)?;
        }
        
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("llama-arxiv/0.1.0"),
        );
        
        let client = Client::builder()
            .default_headers(headers)
            .timeout(config.timeout)
            .build()?;
            
        Ok(Self { client, config })
    }
    
    /// Download a PDF from a given URL to a specified path
    pub async fn download_file(&self, url: &str, path: &Path, force: bool) -> DownloadResult<()> {
        // Check if file already exists
        if path.exists() && !force {
            return Err(DownloadError::FileExists(
                path.to_string_lossy().to_string()
            ));
        }
        
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        
        debug!("Downloading {} to {}", url, path.display());
        let response = self.client.get(url).send().await?;
        
        if !response.status().is_success() {
            return Err(DownloadError::DownloadFailed(format!(
                "Failed to download PDF: HTTP {}",
                response.status()
            )));
        }
        
        let total_size = response
            .content_length()
            .unwrap_or(0);
            
        // Create progress bar if size is available
        let progress = if total_size > 0 {
            let pb = ProgressBar::new(total_size);
            pb.set_style(ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("#>-"));
            Some(pb)
        } else {
            None
        };
        
        let mut file = File::create(path).await?;
        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();
        
        while let Some(item) = stream.next().await {
            let chunk = item?;
            file.write_all(&chunk).await?;
            
            if let Some(ref pb) = progress {
                downloaded += chunk.len() as u64;
                pb.set_position(downloaded);
            }
        }
        
        // Finish the progress bar
        if let Some(pb) = progress {
            pb.finish_with_message("Download complete");
        }
        
        file.flush().await?;
        Ok(())
    }
    
    /// Download a PDF for a paper using its metadata
    pub async fn download_pdf(
        &self, 
        metadata: &PaperMetadata, 
        force: bool
    ) -> DownloadResult<PathBuf> {
        let filename = if self.config.use_id_as_filename {
            format!("{}.pdf", metadata.id)
        } else {
            // Use a combination of first author, year, and title
            let author = metadata.first_author()
                .split_whitespace()
                .last()
                .unwrap_or("unknown");
                
            let year = metadata.year().unwrap_or(0);
            let title = metadata.sanitized_title();
            
            format!("{}_{}__{}.pdf", author, year, title)
                .replace(' ', "_")
        };
        
        let path = Path::new(&self.config.download_dir).join(&filename);
        
        if path.exists() && !force {
            return Err(DownloadError::FileExists(
                path.to_string_lossy().to_string()
            ));
        }
        
        self.download_file(&metadata.pdf_url, &path, force).await?;
        
        Ok(path)
    }
    
    /// Batch download multiple papers
    pub async fn batch_download(
        &self,
        papers: &[PaperMetadata],
        force: bool,
        concurrency: usize
    ) -> Vec<DownloadInfo> {
        let concurrency = concurrency.min(5); // Cap concurrency
        
        stream::iter(papers)
            .map(|paper| async {
                let result = self.download_pdf(paper, force).await;
                
                match result {
                    Ok(path) => DownloadInfo::success(&paper.id, path),
                    Err(DownloadError::FileExists(path)) => {
                        // File exists and force is false
                        let path_buf = PathBuf::from(path);
                        DownloadInfo::skipped(&paper.id, path_buf)
                    },
                    Err(e) => DownloadInfo::failure(&paper.id, e.to_string()),
                }
            })
            .buffer_unordered(concurrency)
            .collect::<Vec<_>>()
            .await
    }
    
    /// Generate a filename for a paper
    pub fn generate_filename(&self, metadata: &PaperMetadata) -> String {
        if self.config.use_id_as_filename {
            format!("{}.pdf", metadata.id)
        } else {
            // Use a combination of first author, year, and title
            let author = metadata.first_author()
                .split_whitespace()
                .last()
                .unwrap_or("unknown");
                
            let year = metadata.year().unwrap_or(0);
            let title = metadata.sanitized_title();
            
            format!("{}_{}__{}.pdf", author, year, title)
                .replace(' ', "_")
        }
    }
    
    /// Get the path where a paper would be saved
    pub fn get_save_path(&self, metadata: &PaperMetadata) -> PathBuf {
        let filename = self.generate_filename(metadata);
        Path::new(&self.config.download_dir).join(filename)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::time::Duration;
    
    #[test]
    fn test_generate_filename() {
        let temp_dir = tempdir().unwrap();
        let config = DownloadConfig {
            download_dir: temp_dir.path().to_string_lossy().to_string(),
            use_id_as_filename: false,
            timeout: Duration::from_secs(30),
        };
        
        let downloader = PdfDownloader::new(config).unwrap();
        
        let mut metadata = PaperMetadata::new("2101.12345");
        metadata.title = "A Test Paper Title".to_string();
        metadata.authors = vec!["John Smith".to_string()];
        metadata.published = "2021-01-01".to_string();
        
        let filename = downloader.generate_filename(&metadata);
        assert!(filename.contains("smith_2021"));
        assert!(filename.contains("Test_Paper_Title"));
        assert!(filename.ends_with(".pdf"));
    }
    
    #[test]
    fn test_id_filename() {
        let temp_dir = tempdir().unwrap();
        let config = DownloadConfig {
            download_dir: temp_dir.path().to_string_lossy().to_string(),
            use_id_as_filename: true,
            timeout: Duration::from_secs(30),
        };
        
        let downloader = PdfDownloader::new(config).unwrap();
        
        let metadata = PaperMetadata::new("2101.12345");
        let filename = downloader.generate_filename(&metadata);
        
        assert_eq!(filename, "2101.12345.pdf");
    }
    
    #[test]
    fn test_get_save_path() {
        let temp_dir = tempdir().unwrap();
        let dir_path = temp_dir.path().to_string_lossy().to_string();
        
        let config = DownloadConfig {
            download_dir: dir_path.clone(),
            use_id_as_filename: true,
            timeout: Duration::from_secs(30),
        };
        
        let downloader = PdfDownloader::new(config).unwrap();
        
        let metadata = PaperMetadata::new("2101.12345");
        let save_path = downloader.get_save_path(&metadata);
        
        assert_eq!(
            save_path,
            Path::new(&dir_path).join("2101.12345.pdf")
        );
    }
} 