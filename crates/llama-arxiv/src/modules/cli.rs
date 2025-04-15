use std::path::PathBuf;
use clap::{Parser, ValueEnum, Args};
use anyhow::Result;
use directories::ProjectDirs;

/// The output format for extracted text
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    /// Plain text format
    Text,
    
    /// HTML format
    Html,
    
    /// Markdown format
    Markdown,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Text => write!(f, "text"),
            OutputFormat::Html => write!(f, "html"),
            OutputFormat::Markdown => write!(f, "markdown"),
        }
    }
}

/// A robust command-line tool for downloading and processing arXiv papers
#[derive(Parser, Debug)]
#[command(name = "llama-arxiv")]
#[command(Author: Nik Jois
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Download, process, and organize papers from arXiv", long_about = None)]
pub struct Cli {
    /// ArXiv IDs or URLs to process
    #[arg(required = true)]
    targets: Vec<String>,
    
    /// Output directory for downloaded files
    #[arg(short, long)]
    output_dir: Option<PathBuf>,
    
    /// Format for extracted text (requires --process-pdf)
    #[arg(short, long, value_enum)]
    format: Option<OutputFormat>,
    
    /// Extract and save BibTeX citations (requires --process-pdf)
    #[arg(short, long)]
    citations: bool,
    
    /// Skip PDF download, only fetch metadata
    #[arg(short = 'M', long)]
    metadata_only: bool,
    
    /// Skip PDF processing, only download
    #[arg(short = 'D', long)]
    download_only: bool,
    
    /// Force re-download of existing files
    #[arg(short, long)]
    force: bool,
    
    /// Path to configuration file
    #[arg(short, long)]
    config: Option<PathBuf>,
    
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
    
    /// Quiet mode, minimal output
    #[arg(short, long)]
    quiet: bool,
}

/// Application configuration derived from command-line arguments
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// ArXiv IDs or URLs to process
    pub targets: Vec<String>,
    
    /// Output directory for downloaded files
    pub output_dir: Option<PathBuf>,
    
    /// Format for extracted text
    pub output_format: Option<OutputFormat>,
    
    /// Extract and save BibTeX citations
    pub extract_citations: bool,
    
    /// Whether to download PDFs
    pub download: bool,
    
    /// Whether to process PDFs
    pub process_pdf: bool,
    
    /// Force re-download of existing files
    pub force: bool,
    
    /// Path to configuration file
    pub config_path: PathBuf,
    
    /// Verbose output
    pub verbose: bool,
    
    /// Quiet mode, minimal output
    pub quiet: bool,
}

/// Parse command-line arguments into an AppConfig
pub fn parse_args() -> Result<AppConfig> {
    let cli = Cli::parse();
    
    // Determine default configuration path
    let config_path = match &cli.config {
        Some(path) => path.clone(),
        None => {
            ProjectDirs::from("com", "llamamoonlight", "llama-arxiv")
                .map(|dirs| dirs.config_dir().join("config.toml"))
                .unwrap_or_else(|| PathBuf::from("config.toml"))
        }
    };
    
    Ok(AppConfig {
        targets: cli.targets,
        output_dir: cli.output_dir,
        output_format: cli.format,
        extract_citations: cli.citations,
        download: !cli.metadata_only,
        process_pdf: !cli.download_only && cli.format.is_some() || cli.citations,
        force: cli.force,
        config_path,
        verbose: cli.verbose,
        quiet: cli.quiet,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_output_format_display() {
        assert_eq!(OutputFormat::Text.to_string(), "text");
        assert_eq!(OutputFormat::Html.to_string(), "html");
        assert_eq!(OutputFormat::Markdown.to_string(), "markdown");
    }
} 