/// Main entry point for the Llama-Arxiv application, a robust command-line tool for
/// downloading, processing, and organizing papers from the arXiv repository.
use std::process;
use colored::Colorize;
use log::{info, error};
use anyhow::Result;
use std::path::{Path, PathBuf};
use clap::Parser;
use thiserror::Error;
use tokio;
use url::Url;
use std::io::Write;
use std::fs;
use env_logger::Builder;

mod modules;
mod error;
mod utils;

use modules::cli::{Cli, OutputFormat};
use modules::config::Config;
use modules::arxiv::ArxivClient;
use modules::download::{PdfDownloader, DownloadInfo};
use modules::parser::PdfParser;
use modules::metadata::PaperMetadata;
use modules::Context;

/// Application error types
#[derive(Error, Debug)]
enum AppError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Invalid arXiv ID or URL: {0}")]
    InvalidInput(String),
    
    #[error("ArXiv API error: {0}")]
    ArxivApi(#[from] modules::arxiv::ArxivError),
    
    #[error("Download error: {0}")]
    Download(#[from] modules::download::DownloadError),
    
    #[error("Parser error: {0}")]
    Parser(#[from] modules::parser::ParserError),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

type AppResult<T> = Result<T, AppError>;

/// Process a single arXiv identifier
async fn process_id(id: &str, context: &Context) -> AppResult<()> {
    info!("Processing arXiv ID: {}", id);
    
    // Get paper metadata
    let client = ArxivClient::new(context.config.api.clone())
        .map_err(AppError::ArxivApi)?;
    
    let metadata = client.get_paper(id).await?;
    
    // If metadata only, print and exit
    if context.args.metadata_only {
        println!("{}", metadata);
        if context.args.format == OutputFormat::Html {
            // Create simple HTML output for metadata
            let html = format!(
                "<!DOCTYPE html>\n<html>\n<head>\n<title>{}</title>\n</head>\n<body>\n<pre>{}</pre>\n</body>\n</html>",
                metadata.title,
                metadata
            );
            
            let output_path = get_output_path(&metadata, &context.args.output_dir, "html")?;
            fs::write(output_path, html)?;
            info!("Saved HTML metadata to {}", output_path.display());
        }
        return Ok(());
    }
    
    // Download PDF if not download-only
    let downloader = PdfDownloader::new(context.config.download.clone())
        .map_err(AppError::Download)?;
    
    let pdf_path = match downloader.download_pdf(&metadata, context.args.force).await {
        Ok(path) => {
            info!("Downloaded PDF to {}", path.display());
            path
        },
        Err(modules::download::DownloadError::FileExists(path)) => {
            let path_buf = PathBuf::from(path);
            info!("PDF already exists at {}", path_buf.display());
            path_buf
        },
        Err(e) => {
            return Err(AppError::Download(e));
        }
    };
    
    // Exit early if download-only
    if context.args.download_only {
        return Ok(());
    }
    
    // Parse PDF
    let parser = PdfParser::new(context.config.pdf.clone());
    let mut parsed = parser.parse_pdf(&pdf_path)?;
    
    // Add metadata to parsed PDF
    parsed.metadata = Some(metadata.clone());
    
    // Save output in the requested format
    let extension = match context.args.format {
        OutputFormat::Text => "txt",
        OutputFormat::Markdown => "md",
        OutputFormat::Html => "html",
    };
    
    let output_path = get_output_path(&metadata, &context.args.output_dir, extension)?;
    
    match context.args.format {
        OutputFormat::Text => {
            parsed.save_text(&output_path)?;
        },
        OutputFormat::Markdown => {
            parsed.save_markdown(&output_path)?;
        },
        OutputFormat::Html => {
            parsed.save_html(&output_path)?;
        }
    }
    
    println!("{} Saved {} output to {}", 
        "✓".green(), 
        context.args.format, 
        output_path.display().to_string().blue());
    
    // Save citations if requested
    if context.args.citations {
        let citation_path = output_path.with_extension("bib");
        fs::write(&citation_path, metadata.to_bibtex())?;
        println!("{} Saved BibTeX citation to {}", 
            "✓".green(), 
            citation_path.display().to_string().blue());
    }
    
    Ok(())
}

/// Generate an output file path based on metadata
fn get_output_path(
    metadata: &PaperMetadata, 
    output_dir: &str, 
    extension: &str
) -> AppResult<PathBuf> {
    let filename = format!(
        "{}_{}_{}",
        metadata.id,
        metadata.first_author().split_whitespace().last().unwrap_or("unknown"),
        metadata.sanitized_title()
    );
    
    let path = Path::new(output_dir)
        .join(format!("{}.{}", filename, extension));
        
    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    
    Ok(path)
}

/// Parse an arXiv URL to extract the ID
fn parse_arxiv_url(url_str: &str) -> Option<String> {
    if let Ok(url) = Url::parse(url_str) {
        if url.host_str() == Some("arxiv.org") {
            let path = url.path();
            
            // Handle /abs/2101.12345 format
            if path.starts_with("/abs/") {
                return Some(path.replace("/abs/", ""));
            }
            
            // Handle /pdf/2101.12345.pdf format
            if path.starts_with("/pdf/") {
                return Some(path.replace("/pdf/", "").replace(".pdf", ""));
            }
        }
    }
    
    None
}

/// Setup logging based on verbosity
fn setup_logging(verbose: bool, quiet: bool) {
    let mut builder = Builder::from_default_env();
    
    let level = if quiet {
        log::LevelFilter::Error
    } else if verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };
    
    builder
        .filter_level(level)
        .format(|buf, record| {
            let level_color = match record.level() {
                log::Level::Error => "red",
                log::Level::Warn => "yellow",
                log::Level::Info => "green",
                log::Level::Debug => "blue",
                log::Level::Trace => "magenta",
            };
            
            writeln!(
                buf,
                "[{}] {}",
                record.level().to_string().color(level_color),
                record.args()
            )
        })
        .init();
}

#[tokio::main]
async fn main() {
    // Parse command-line arguments
    let cli = Cli::parse();
    
    // Setup logging
    setup_logging(cli.verbose, cli.quiet);
    
    // Load configuration
    let config = match Config::load(&cli.config) {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            process::exit(1);
        }
    };
    
    // Create application context
    let context = Context::new(cli, config);
    
    // Process each target (ID or URL)
    let mut errors = false;
    
    for target in &context.args.targets {
        let id = if target.starts_with("http") {
            match parse_arxiv_url(target) {
                Some(id) => id,
                None => {
                    error!("Invalid arXiv URL: {}", target);
                    errors = true;
                    continue;
                }
            }
        } else {
            target.clone()
        };
        
        match process_id(&id, &context).await {
            Ok(_) => {
                info!("Successfully processed {}", id);
            },
            Err(e) => {
                error!("Failed to process {}: {}", id, e);
                errors = true;
            }
        }
    }
    
    // Exit with error code if any processing failed
    if errors {
        process::exit(1);
    }
} 