use reqwest::Client;
use std::path::{Path, PathBuf};
use std::fs;
use thiserror::Error;
use log::{debug, error, info};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};

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

pub type DownloadResult<T> = Result<T, DownloadError>;

pub async fn download_pdf(
    client: &Client,
    pmid: &str,
    name: &str,
    output_dir: &Path,
    max_retries: u32,
    finders: &[String],
) -> DownloadResult<PathBuf> {
    let filename = format!("{}.pdf", name);
    let path = output_dir.join(&filename);

    if path.exists() {
        return Err(DownloadError::FileExists(path.to_string_lossy().to_string()));
    }

    // Placeholder URL for PDF download
    let url = format!("https://example.com/{}.pdf", pmid);

    debug!("Downloading {} to {}", url, path.display());
    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        return Err(DownloadError::DownloadFailed(format!(
            "Failed to download PDF: HTTP {}",
            response.status()
        )));
    }

    let total_size = response.content_length().unwrap_or(0);

    // Create progress bar if size is available
    let progress = if total_size > 0 {
        let pb = ProgressBar::new(total_size);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("#>-");
        Some(pb)
    } else {
        None
    };

    let mut file = File::create(&path).await?;
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
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use reqwest::Client;

    #[tokio::test]
    async fn test_download_pdf() {
        let temp_dir = tempdir().unwrap();
        let client = Client::new();
        let result = download_pdf(&client, "1234567", "test", temp_dir.path(), 3, &[]).await;
        assert!(result.is_err()); // Placeholder URL will fail
    }
} 