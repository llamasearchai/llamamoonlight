use std::path::{Path, PathBuf};
use std::fs;
use url::Url;
use regex::Regex;
use lazy_static::lazy_static;
use crate::error::ArxivError;
use crate::modules::cli::OutputFormat;

lazy_static! {
    /// Regex for extracting arXiv ID from text
    static ref ARXIV_ID_REGEX: Regex = Regex::new(
        r"(?:arxiv:)?([0-9]{4}\.[0-9]{4,5}|[a-z\-]+(\.[A-Z]{2})?/[0-9]{7})"
    ).unwrap();
    
    /// Regex for validating arXiv URLs
    static ref ARXIV_URL_REGEX: Regex = Regex::new(
        r"https?://(?:www\.)?arxiv\.org/(?:abs|pdf)/([0-9]{4}\.[0-9]{4,5}|[a-z\-]+(\.[A-Z]{2})?/[0-9]{7})(?:v[0-9]+)?"
    ).unwrap();
    
    /// Regex for sanitizing filenames
    static ref FILENAME_SANITIZER: Regex = Regex::new(r"[^\w\s\-\.]").unwrap();
}

/// Result type used throughout the application
pub type Result<T> = std::result::Result<T, ArxivError>;

/// Ensure a directory exists, creating it if necessary
pub fn ensure_directory(path: &str) -> Result<()> {
    let path = Path::new(path);
    if !path.exists() {
        fs::create_dir_all(path)?;
    } else if !path.is_dir() {
        return Err(ArxivError::config_error(
            format!("Path exists but is not a directory: {}", path.display())
        ));
    }
    Ok(())
}

/// Parse an arXiv identifier from a string (could be an ID or URL)
pub fn parse_arxiv_id(input: &str) -> Result<String> {
    lazy_static! {
        // Pattern for modern arXiv IDs (e.g., 2101.12345)
        static ref MODERN_ID: Regex = Regex::new(r"^\d{4}\.\d{4,5}(v\d+)?$").unwrap();
        
        // Pattern for legacy arXiv IDs (e.g., hep-th/9901001)
        static ref LEGACY_ID: Regex = Regex::new(r"^[a-z\-]+/\d{7}(v\d+)?$").unwrap();
    }
    
    // Check if input is already an arXiv ID
    if MODERN_ID.is_match(input) || LEGACY_ID.is_match(input) {
        return Ok(input.to_string());
    }
    
    // Check if it has the "arXiv:" prefix
    if let Some(stripped) = input.strip_prefix("arXiv:") {
        if MODERN_ID.is_match(stripped) || LEGACY_ID.is_match(stripped) {
            return Ok(stripped.to_string());
        }
    }
    
    // Try to parse as URL
    if input.starts_with("http") {
        match Url::parse(input) {
            Ok(url) => {
                if url.host_str() == Some("arxiv.org") {
                    let path = url.path();
                    
                    // Handle /abs/2101.12345 format
                    if path.starts_with("/abs/") {
                        let id = path.replace("/abs/", "");
                        if MODERN_ID.is_match(&id) || LEGACY_ID.is_match(&id) {
                            return Ok(id);
                        }
                    }
                    
                    // Handle /pdf/2101.12345.pdf format
                    if path.starts_with("/pdf/") {
                        let id = path.replace("/pdf/", "").replace(".pdf", "");
                        if MODERN_ID.is_match(&id) || LEGACY_ID.is_match(&id) {
                            return Ok(id);
                        }
                    }
                }
            },
            Err(e) => return Err(ArxivError::UrlParse(e)),
        }
    }
    
    Err(ArxivError::invalid_id(input))
}

/// Generate a filename for a paper based on ID and first author
pub fn generate_filename(id: &str, author: &str, title: &str, year: Option<u32>) -> String {
    let sanitized_title = sanitize_filename(title);
    let short_title = if sanitized_title.len() > 30 {
        sanitized_title[..30].to_string()
    } else {
        sanitized_title
    };
    
    let sanitized_author = sanitize_filename(author.split_whitespace().last().unwrap_or("unknown"));
    
    if let Some(year) = year {
        format!("{}_{}_{}_{}", id, sanitized_author, year, short_title)
    } else {
        format!("{}_{}_{}", id, sanitized_author, short_title)
    }
}

/// Sanitize a string for use in filenames
pub fn sanitize_filename(name: &str) -> String {
    let invalid_chars = [
        '/', '\\', ':', '*', '?', '"', '<', '>', '|', 
        '\0', '\t', '\n', '\r'
    ];
    
    let mut result = name.replace(&invalid_chars[..], "_");
    
    // Replace multiple underscores with a single one
    let multi_underscore = Regex::new(r"_+").unwrap();
    result = multi_underscore.replace_all(&result, "_").to_string();
    
    // Trim leading/trailing underscores
    result.trim_matches('_').to_string()
}

/// Expand a path that might contain ~ for home directory
pub fn expand_tilde(path: &str) -> PathBuf {
    if !path.starts_with('~') {
        return PathBuf::from(path);
    }
    
    let path_str = if path.starts_with("~/") || path == "~" {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        
        if path == "~" {
            home.to_string_lossy().to_string()
        } else {
            format!("{}{}", home.to_string_lossy(), &path[1..])
        }
    } else {
        // Handle ~user/path format (not commonly used, simplistic implementation)
        path.replace('~', "/home")
    };
    
    PathBuf::from(path_str)
}

/// Check if a path exists and is accessible
pub fn check_path_accessible(path: &Path) -> bool {
    path.exists() && (path.is_file() || path.is_dir())
}

/// Generate an output path for processed content based on the PDF path
pub fn generate_output_path(pdf_path: &Path, format: &OutputFormat) -> PathBuf {
    let extension = match format {
        OutputFormat::Text => "txt",
        OutputFormat::Html => "html",
        OutputFormat::Markdown => "md",
    };
    
    pdf_path.with_extension(extension)
}

/// Check if a file exists and is not empty
pub fn file_exists_and_not_empty(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    
    match fs::metadata(path) {
        Ok(metadata) => metadata.len() > 0,
        Err(_) => false,
    }
}

/// Format a file size in human-readable format
pub fn format_file_size(size: u64) -> String {
    const KILO: u64 = 1024;
    const MEGA: u64 = KILO * 1024;
    const GIGA: u64 = MEGA * 1024;
    
    if size < KILO {
        format!("{} B", size)
    } else if size < MEGA {
        format!("{:.1} KB", size as f64 / KILO as f64)
    } else if size < GIGA {
        format!("{:.1} MB", size as f64 / MEGA as f64)
    } else {
        format!("{:.1} GB", size as f64 / GIGA as f64)
    }
}

/// Extract the year from an arXiv ID
pub fn extract_year_from_arxiv_id(id: &str) -> Option<u32> {
    // For new-style IDs (YYMM.nnnnn)
    if let Some(captures) = Regex::new(r"^([0-9]{2})([0-9]{2})\.[0-9]+").unwrap().captures(id) {
        let yy: u32 = captures.get(1).unwrap().as_str().parse().unwrap_or(0);
        let year = if yy < 90 { 2000 + yy } else { 1900 + yy };
        return Some(year);
    }
    
    // For old-style IDs (archive/YYMMnnn)
    if let Some(captures) = Regex::new(r"/([0-9]{2})([0-9]{2})[0-9]+").unwrap().captures(id) {
        let yy: u32 = captures.get(1).unwrap().as_str().parse().unwrap_or(0);
        let year = if yy < 90 { 2000 + yy } else { 1900 + yy };
        return Some(year);
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_ensure_directory() {
        // Test creating a new directory
        let dir = tempdir().unwrap();
        let test_path = dir.path().join("test_dir");
        assert!(!test_path.exists());
        
        ensure_directory(&test_path.to_string_lossy()).unwrap();
        assert!(test_path.exists());
        assert!(test_path.is_dir());
        
        // Test with existing directory
        ensure_directory(&test_path.to_string_lossy()).unwrap();
        
        // Test with existing file (should fail)
        let file_path = dir.path().join("test_file");
        fs::write(&file_path, "test").unwrap();
        assert!(ensure_directory(&file_path.to_string_lossy()).is_err());
    }
    
    #[test]
    fn test_parse_arxiv_id_modern() {
        assert_eq!(parse_arxiv_id("2101.12345").unwrap(), "2101.12345");
        assert_eq!(parse_arxiv_id("2101.12345v2").unwrap(), "2101.12345v2");
    }
    
    #[test]
    fn test_parse_arxiv_id_legacy() {
        assert_eq!(parse_arxiv_id("hep-th/9901001").unwrap(), "hep-th/9901001");
        assert_eq!(parse_arxiv_id("cond-mat/0211002v2").unwrap(), "cond-mat/0211002v2");
    }
    
    #[test]
    fn test_parse_arxiv_id_with_prefix() {
        assert_eq!(parse_arxiv_id("arXiv:2101.12345").unwrap(), "2101.12345");
    }
    
    #[test]
    fn test_parse_arxiv_id_url() {
        assert_eq!(
            parse_arxiv_id("https://arxiv.org/abs/2101.12345").unwrap(),
            "2101.12345"
        );
        assert_eq!(
            parse_arxiv_id("https://arxiv.org/pdf/2101.12345.pdf").unwrap(),
            "2101.12345"
        );
    }
    
    #[test]
    fn test_parse_arxiv_id_invalid() {
        assert!(parse_arxiv_id("invalid").is_err());
        assert!(parse_arxiv_id("https://example.com").is_err());
    }
    
    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("Test: File"), "Test_File");
        assert_eq!(sanitize_filename("File/with\\many:invalid*chars?"), "File_with_many_invalid_chars");
        assert_eq!(sanitize_filename("_leading_and_trailing_"), "leading_and_trailing");
        assert_eq!(sanitize_filename("multiple___underscores"), "multiple_underscores");
    }
    
    #[test]
    fn test_generate_filename() {
        assert_eq!(
            generate_filename("2101.12345", "John Smith", "A Test Paper", Some(2021)),
            "2101.12345_Smith_2021_A_Test_Paper"
        );
        
        assert_eq!(
            generate_filename("2101.12345", "John Smith", "A Very Very Very Very Very Very Very Long Title", None),
            "2101.12345_Smith_A_Very_Very_Very_Very_Very_Ve"
        );
    }
    
    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(500), "500 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1048576), "1.0 MB");
        assert_eq!(format_file_size(1073741824), "1.0 GB");
    }
    
    #[test]
    fn test_extract_year_from_arxiv_id() {
        // Test with new-style IDs
        assert_eq!(extract_year_from_arxiv_id("2101.12345"), Some(2021));
        assert_eq!(extract_year_from_arxiv_id("9901.12345"), Some(1999));
        
        // Test with old-style IDs
        assert_eq!(extract_year_from_arxiv_id("hep-th/9901001"), Some(1999));
        assert_eq!(extract_year_from_arxiv_id("cond-mat/0001001"), Some(2000));
        
        // Test with invalid IDs
        assert_eq!(extract_year_from_arxiv_id("invalid"), None);
    }
} 