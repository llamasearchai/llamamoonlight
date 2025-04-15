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
        .progress_chars("#>-");
    
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