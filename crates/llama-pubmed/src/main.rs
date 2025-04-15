mod cli;
mod config_manager;
mod error_handling;
mod metadata_manager;
mod pdf_downloader;
mod pubmed_api;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(Author: Nik Jois
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