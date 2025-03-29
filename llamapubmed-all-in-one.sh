#!/usr/bin/env bash
set -eo pipefail
# LlamaPubMed - All-in-One Installation and Execution Script
# This script sets up and runs the LlamaPubMed program

# Color definitions for console output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

log_section() {
    echo -e "\n${PURPLE}=== $1 ===${NC}"
}

# Check for required tools
check_requirements() {
    log_section "Checking Requirements"
    
    local requirements=("cargo" "rustc")
    local missing=()
    
    for cmd in "${requirements[@]}"; do
        if ! command -v "$cmd" &> /dev/null; then
            missing+=("$cmd")
        fi
    done
    
    if [ ${#missing[@]} -ne 0 ]; then
        log_error "Missing required tools: ${missing[*]}"
        log_info "Installing Rust and Cargo..."
        
        # Install Rust and Cargo
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        
        # Add Cargo to the path for this session
        source "$HOME/.cargo/env"
        
        log_success "Rust and Cargo have been installed."
    else
        log_success "All required tools are installed."
    fi
}

# Create a temporary directory for the project
create_project_dir() {
    log_section "Creating Project Directory"
    
    # Create a temporary directory
    PROJECT_DIR=$(mktemp -d)
    log_info "Project directory created at: $PROJECT_DIR"
    
    # Make sure the directory is cleaned up on exit
    trap 'log_info "Cleaning up temporary files..."; rm -rf "$PROJECT_DIR"' EXIT
    
    # Change to the project directory
    cd "$PROJECT_DIR"
    log_success "Now working in temporary project directory."
}

# Create all project files
create_project_files() {
    log_section "Creating Project Files"
    
    # Create Cargo.toml
    log_info "Creating Cargo.toml..."
    cat > Cargo.toml << 'EOF'
[package]
name = "llamapubmed"
version = "0.1.0"
edition = "2021"
authors = ["LlamaPubMed Contributors"]
description = "A command-line tool for downloading scientific papers from PubMed"
readme = "README.md"
license = "MIT"
keywords = ["pubmed", "science", "papers", "pdf", "downloader"]
categories = ["command-line-utilities", "science"]

[dependencies]
clap = { version = "4.3", features = ["derive"] }
reqwest = { version = "0.11", features = ["json", "rustls-tls"] }
tokio = { version = "1.28", features = ["full"] }
log = "0.4"
env_logger = "0.10"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
scraper = "0.17"
quick-xml = { version = "0.30", features = ["serialize"] }
dirs = "5.0"
pdf = "0.8"
urlencoding = "2.1"
thiserror = "1.0"
anyhow = "1.0"
chrono = { version = "0.4", features = ["serde"] }
futures = "0.3"
indicatif = "0.17"
colored = "2.0"
async-trait = "0.1"
llama-arxiv = { path = "path/to/llama-arxiv" }
llama-pubmed = { path = "path/to/llama-pubmed" }

[dev-dependencies]
pretty_assertions = "1.3"
tempfile = "3.5"
httpmock = "0.6"
tokio-test = "0.4"
EOF
    log_success "Cargo.toml created."
    
    # Create README.md
    log_info "Creating README.md..."
    cat > README.md << 'EOF'
# LlamaPubMed

A command-line tool for downloading scientific papers from PubMed.

## Features

- Download papers by PMID or search query
- Save PDFs with custom naming
- Extract and store metadata from downloaded PDFs
- Configure and customize PDF finders for different publishers
- Retry mechanisms for failed downloads
- Search PubMed with various filtering options
- Robust error handling and logging

## Usage

```bash
# Download papers by PMID
llamapubmed download --pmids 33000000,33000001 --output-dir ./downloads

# Search PubMed and display results
llamapubmed search --query "cancer AND immunotherapy" --max-results 20

# Show current configuration
llamapubmed config show
```

## License

This project is licensed under the MIT License.
EOF
    log_success "README.md created."
    
    # Create source directory
    mkdir -p src
    
    # Create source files
    log_info "Creating source files..."
    
    # Create main.rs
    cat > src/main.rs << 'EOF'
mod cli;
mod config_manager;
mod error_handling;
mod metadata_manager;
mod pdf_downloader;
mod pubmed_api;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "A tool for downloading scientific papers from PubMed", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand, Debug)]
enum Command {
    /// Download PDFs from PubMed based on PMIDs, search queries, or files
    Download(DownloadArgs),

    /// Search PubMed and display results
    Search(SearchArgs),

    /// Manage the LlamaPubMed configuration
    Config(ConfigArgs),
}

#[derive(Parser, Debug)]
struct DownloadArgs {
    /// Comma-separated list of PMIDs to fetch (e.g., 1234567,7654321)
    #[arg(short, long, value_name = "PMIDS")]
    pmids: Option<String>,

    /// File containing a list of PMIDs, one per line
    #[arg(short, long, value_name = "FILE")]
    pmids_file: Option<PathBuf>,

    /// PubMed search query (e.g., "cancer AND immunotherapy")
    #[arg(short, long, value_name = "QUERY")]
    search: Option<String>,

    /// Output directory for downloaded PDFs
    #[arg(short, long, value_name = "DIR", default_value = ".")]
    output_dir: PathBuf,

    /// Output file path for PMIDs that failed to download
    #[arg(short, long, value_name = "FILE", default_value = "failed_pmids.txt")]
    errors_file: PathBuf,

    /// Maximum number of retry attempts for failed downloads
    #[arg(short, long, value_name = "NUM", default_value_t = 3)]
    max_retries: u32,

    /// Custom user-agent string
    #[arg(long, value_name = "USER_AGENT")]
    user_agent: Option<String>,

    /// Load configuration from a file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Download articles within a date range (YYYY-MM-DD,YYYY-MM-DD)
    #[arg(long, value_name = "START_DATE,END_DATE")]
    date_range: Option<String>,

    /// Download articles from a specified journal (e.g., "Nature")
    #[arg(long, value_name = "JOURNAL")]
    journal: Option<String>,

    /// Format for metadata output (json, yaml, bibtex)
    #[arg(long, value_name = "FORMAT")]
    metadata_format: Option<String>,
}

#[derive(Parser, Debug)]
struct SearchArgs {
    /// PubMed search query
    #[arg(short, long, value_name = "QUERY")]
    query: String,

    /// Output format for search results (text, json, csv)
    #[arg(short, long, value_name = "FORMAT", default_value = "text")]
    output_format: String,

    /// Maximum number of search results to display
    #[arg(short = 'n', long, value_name = "NUM", default_value_t = 10)]
    max_results: u32,

    /// Download articles within a date range (YYYY-MM-DD,YYYY-MM-DD)
    #[arg(long, value_name = "START_DATE,END_DATE")]
    date_range: Option<String>,

    /// Download articles from a specified journal (e.g., "Nature")
    #[arg(long, value_name = "JOURNAL")]
    journal: Option<String>,
}

#[derive(clap::Subcommand, Debug)]
enum ConfigArgs {
    /// Show the current configuration
    Show,

    /// Edit the configuration (opens in a text editor)
    Edit,

    /// Reset the configuration to default values
    Reset,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::parse();

    match args.command {
        Command::Download(download_args) => {
            cli::handle_download(download_args).await?;
        }
        Command::Search(search_args) => {
            cli::handle_search(search_args).await?;
        }
        Command::Config(config_args) => {
            cli::handle_config(config_args)?;
        }
    }

    Ok(())
}
EOF
    
    # Create cli.rs
    cat > src/cli.rs << 'EOF'
use crate::config_manager;
use crate::error_handling;
use crate::metadata_manager;
use crate::pdf_downloader;
use crate::pubmed_api;
use colored::Colorize;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use log::{debug, error, info};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

// Import the structs from the main module to handle args.
use crate::{ConfigArgs, DownloadArgs, SearchArgs};

// Define types for error handling throughout the CLI module.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub async fn handle_download(args: DownloadArgs) -> Result<()> {
    // Load config.  If --config is specified, use it, otherwise, load the default.
    let config = match args.config {
        Some(config_path) => config_manager::load_config_from_file(&config_path)?,
        None => config_manager::load_default_config()?,
    };

    let output_dir = args.output_dir;

    // Create output dir if it does not exist.
    if !output_dir.exists() {
        std::fs::create_dir_all(&output_dir)?;
        info!("Created output directory: {:?}", output_dir);
    }

    let user_agent = args.user_agent.unwrap_or_else(|| {
        config.user_agent.clone().unwrap_or_else(|| {
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/56.0.2924.87 Safari/537.36".to_string()
        })
    });

    let client = reqwest::Client::builder()
        .user_agent(user_agent)
        .build()?;

    let mut pmids: Vec<String> = Vec::new();
    let mut names: Vec<String> = Vec::new();

    // Determine where the list of pmids comes from.
    if let Some(pmids_str) = args.pmids {
        pmids.extend(pmids_str.split(',').map(|s| s.trim().to_string()));
        names.extend(pmids.clone());
    } else if let Some(pmids_file_path) = args.pmids_file {
        let file = File::open(pmids_file_path)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 1 {
                pmids.push(parts[0].to_string());
                if parts.len() > 1 {
                    names.push(parts[1].to_string());
                } else {
                    names.push(parts[0].to_string());
                }
            }
        }
    } else if let Some(search_query) = args.search {
        println!("Searching PubMed for: {}", search_query.bright_cyan());
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
                .template("{spinner} {msg}").unwrap(),
        );
        pb.set_message("Fetching results from PubMed...");
        pb.enable_steady_tick(Duration::from_millis(100));
        
        let search_results = pubmed_api::search_pubmed(&client, &search_query, args.date_range, args.journal).await?;
        
        pb.finish_with_message(format!("Found {} articles", search_results.len()));
        
        if search_results.is_empty() {
            println!("{}", "No articles found matching your search criteria.".bright_red());
            return Ok(());
        }
        
        pmids.extend(search_results);
        names.extend(pmids.clone());  // For simplicity, use PMIDs as names in this case
    }

    if pmids.is_empty() {
        error!("No PMIDs provided or found from search.");
        println!("{}", "No PMIDs provided or found from search.".bright_red());
        return Ok(());
    }

    let errors_file = args.errors_file;
    let mut failed_pmids: Vec<String> = Vec::new();

    println!("Ready to download {} articles", pmids.len().to_string().bright_green());
    
    let pb = ProgressBar::new(pmids.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}").unwrap()
        .progress_chars("#>-"));
    
    // Process each PMID.
    for (i, (pmid, name)) in pmids.iter().zip(names.iter()).enumerate() {
        pb.set_message(format!("Downloading PMID: {}", pmid));
        debug!("Attempting to download PMID: {}", pmid);
        
        let download_result = pdf_downloader::download_pdf(
            &client,
            pmid,
            name,
            &output_dir,
            args.max_retries,
            &config.finders,
        )
        .await;

        match download_result {
            Ok(_) => {
                debug!("Successfully downloaded PMID: {}", pmid);
            }
            Err(e) => {
                error!("Failed to download PMID {}: {}", pmid, e);
                failed_pmids.push(pmid.clone()); // Clone to avoid borrowing issues
            }
        }
        
        pb.inc(1);
    }
    
    pb.finish_with_message(format!("Downloaded {} of {} articles", 
        (pmids.len() - failed_pmids.len()).to_string().bright_green(), 
        pmids.len().to_string().bright_cyan()
    ));

    // Write failed PMIDs to the specified file.
    if !failed_pmids.is_empty() {
        let mut error_file = File::create(&errors_file)?;
        for pmid in &failed_pmids {
            writeln!(error_file, "{}", pmid)?;
        }
        println!("Failed to download {} articles. Failed PMIDs written to: {}", 
            failed_pmids.len().to_string().bright_yellow(), 
            errors_file.display().to_string().bright_cyan()
        );
    }

    println!("{}", "Download process complete.".bright_green());
    println!("Downloaded PDFs can be found in: {}", output_dir.display().to_string().bright_cyan());

    Ok(())
}

pub async fn handle_search(args: SearchArgs) -> Result<()> {
    let client = reqwest::Client::new();
    
    println!("Searching PubMed for: {}", args.query.bright_cyan());
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner} {msg}").unwrap(),
    );
    pb.set_message("Fetching results from PubMed...");
    pb.enable_steady_tick(Duration::from_millis(100));
    
    let search_results = pubmed_api::search_pubmed(
        &client,
        &args.query,
        args.date_range,
        args.journal,
    )
    .await?;
    
    pb.finish_with_message(format!("Found {} articles", search_results.len()));

    if search_results.is_empty() {
        println!("{}", "No articles found matching your search criteria.".bright_yellow());
        return Ok(());
    }

    let display_results = if args.max_results as usize > search_results.len() {
        &search_results[..]
    } else {
        &search_results[..args.max_results as usize]
    };

    match args.output_format.as_str() {
        "text" => {
            println!("PMIDs for query: {} (showing {} of {} results)", 
                args.query.bright_green(), 
                display_results.len().to_string().bright_yellow(),
                search_results.len().to_string().bright_cyan()
            );
            
            for (i, pmid) in display_results.iter().enumerate() {
                println!("  {}. {}", (i + 1).to_string().bright_cyan(), pmid);
            }
        }
        "json" => {
            // Serialize results to JSON
            let json_output = serde_json::to_string_pretty(&search_results)?;
            println!("{}", json_output);
        }
        "csv" => {
            // Output the search results in CSV format
            println!("PMID");
            for pmid in display_results {
                println!("{}", pmid);
            }
        }
        _ => {
            error!("Unsupported output format: {}", args.output_format);
            println!("{}", format!("Unsupported output format: {}", args.output_format).bright_red());
        }
    }

    Ok(())
}

pub fn handle_config(args: ConfigArgs) -> Result<()> {
    match args {
        ConfigArgs::Show => {
            let config = config_manager::load_default_config()?;
            let config_path = config_manager::get_config_file_path()?;
            
            println!("Configuration file location: {}", config_path.display().to_string().bright_cyan());
            println!("{}", "Current Configuration:".bright_green());
            
            let serialized_config = serde_yaml::to_string(&config)?;
            println!("{}", serialized_config);
        }
        ConfigArgs::Edit => {
            let config_path = config_manager::get_config_file_path()?;
            
            // Check if the file exists, if not create it
            if !config_path.exists() {
                let default_config = config_manager::Config::default();
                config_manager::save_config(&default_config)?;
                println!("Created default config file at: {}", config_path.display().to_string().bright_cyan());
            }
            
            // Try to determine a suitable editor
            let editor = std::env::var("EDITOR")
                .unwrap_or_else(|_| "vi".to_string());  // Default to vi if EDITOR not set
            
            println!("Opening configuration with {} at {}", 
                editor.bright_green(), 
                config_path.display().to_string().bright_cyan()
            );
            
            // Execute the editor command
            let status = std::process::Command::new(editor)
                .arg(config_path)
                .status()?;
            
            if status.success() {
                println!("{}", "Configuration updated successfully.".bright_green());
            } else {
                println!("{}", "Editor exited with an error. Configuration may not be updated.".bright_red());
            }
        }
        ConfigArgs::Reset => {
            config_manager::reset_config()?;
            println!("{}", "Configuration reset to default values.".bright_green());
            
            // Show the reset configuration
            let config = config_manager::load_default_config()?;
            let serialized_config = serde_yaml::to_string(&config)?;
            println!("New configuration:");
            println!("{}", serialized_config);
        }
    }
    Ok(())
}
EOF

    # Create config_manager.rs
    cat > src/config_manager.rs << 'EOF'
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_yaml::Error),
    
    #[error("Could not determine config directory")]
    NoConfigDir,
    
    #[error("Configuration file not found at {0}")]
    ConfigNotFound(PathBuf),
}

// Define the configuration struct.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub download_dir: PathBuf,
    pub max_retries: u32,
    pub user_agent: Option<String>,
    pub finders: Vec<String>,
}

// Define the default configuration values.
impl Default for Config {
    fn default() -> Self {
        Config {
            download_dir: PathBuf::from("./downloads"),
            max_retries: 3,
            user_agent: Some(
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/56.0.2924.87 Safari/537.36".to_string()
            ),
            finders: vec![
                "genericCitationLabelled".to_string(),
                "pubmed_central_v2".to_string(),
                "acsPublications".to_string(),
                "uchicagoPress".to_string(),
                "nejm".to_string(),
                "futureMedicine".to_string(),
                "science_direct".to_string(),
                "direct_pdf_link".to_string(),
            ],
        }
    }
}

const CONFIG_FILE_NAME: &str = "llamapubmed_config.yaml";

// Function to get the config file path.
pub fn get_config_file_path() -> Result<PathBuf, ConfigError> {
    let config_dir = dirs::config_dir().ok_or(ConfigError::NoConfigDir)?;
    let config_file_path = config_dir.join(CONFIG_FILE_NAME);
    Ok(config_file_path)
}

// Function to load the configuration from a file.
pub fn load_config_from_file(file_path: &Path) -> Result<Config, ConfigError> {
    if !file_path.exists() {
        return Err(ConfigError::ConfigNotFound(file_path.to_path_buf()));
    }
    
    let file = File::open(file_path)?;
    let config: Config = serde_yaml::from_reader(file)?;
    info!("Loaded configuration from: {:?}", file_path);
    Ok(config)
}

// Function to load the default configuration.
pub fn load_default_config() -> Result<Config, ConfigError> {
    let config_file_path = get_config_file_path()?;
    if config_file_path.exists() {
        load_config_from_file(&config_file_path)
    } else {
        // If the config file doesn't exist, create it with default settings.
        let default_config = Config::default();
        save_config(&default_config)?;
        info!("Created default config file at: {:?}", config_file_path);
        Ok(default_config)
    }
}

// Function to save the config to disk
pub fn save_config(config: &Config) -> Result<(), ConfigError> {
    let config_file_path = get_config_file_path()?;

    // Ensure the directory exists
    if let Some(dir) = config_file_path.parent() {
        fs::create_dir_all(dir)?;
    }

    let mut file = File::create(&config_file_path)?;
    serde_yaml::to_writer(&mut file, config)?;
    info!("Configuration saved to: {:?}", config_file_path);
    Ok(())
}

// Function to reset the configuration to default values.
pub fn reset_config() -> Result<(), ConfigError> {
    let default_config = Config::default();
    save_config(&default_config)?;
    Ok(())
}
EOF

    # Create error_handling.rs
    cat > src/error_handling.rs << 'EOF'
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
EOF

    # Create metadata_manager.rs
    cat > src/metadata_manager.rs << 'EOF'
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
    
    #[error("