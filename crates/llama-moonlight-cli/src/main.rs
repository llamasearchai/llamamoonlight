use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use llama_moonlight_core::{
    options::{BrowserOptions, ContextOptions, PageOptions},
    BrowserType, Moonlight,
};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

/// Llama Moonlight - A browser automation CLI
#[derive(Parser)]
#[command(Author: Nik Jois
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// The browser to use (chromium, firefox, webkit)
    #[arg(short, long, default_value = "chromium")]
    browser: String,

    /// Run in headless mode
    #[arg(short, long, default_value = "true")]
    headless: bool,

    /// Enable stealth mode
    #[arg(short, long)]
    stealth: bool,

    /// Set custom user agent
    #[arg(short = 'u', long)]
    user_agent: Option<String>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Take a screenshot of a webpage
    Screenshot {
        /// The URL to navigate to
        url: String,

        /// The path to save the screenshot to
        #[arg(short, long)]
        output: PathBuf,

        /// Full page screenshot (otherwise viewport only)
        #[arg(short, long)]
        full_page: bool,
    },

    /// Get the content of a webpage
    Content {
        /// The URL to navigate to
        url: String,

        /// The path to save the content to
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output as HTML (default) or TEXT
        #[arg(short, long, default_value = "html")]
        format: String,
    },

    /// Evaluate JavaScript in a webpage
    Evaluate {
        /// The URL to navigate to
        url: String,

        /// The JavaScript to evaluate
        #[arg(short, long)]
        script: String,
    },

    /// Click on an element in a webpage
    Click {
        /// The URL to navigate to
        url: String,

        /// The selector of the element to click
        #[arg(short, long)]
        selector: String,

        /// Take a screenshot after clicking
        #[arg(short, long)]
        screenshot: Option<PathBuf>,
    },

    /// Fill out a form in a webpage
    Fill {
        /// The URL to navigate to
        url: String,

        /// The selector of the input to fill
        #[arg(short, long)]
        selector: String,

        /// The text to fill the input with
        #[arg(short, long)]
        text: String,

        /// Submit the form after filling (clicks the first submit button)
        #[arg(short, long)]
        submit: bool,
    },

    /// Extract data from a webpage using selectors
    Extract {
        /// The URL to navigate to
        url: String,

        /// The selector to extract data from
        #[arg(short, long)]
        selector: String,

        /// The attribute to extract (default is innerText)
        #[arg(short, long, default_value = "innerText")]
        attribute: String,

        /// Output format (json, csv, text)
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Output file (if not specified, prints to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Monitor network requests on a page
    Network {
        /// The URL to navigate to
        url: String,

        /// Filter requests by URL pattern
        #[arg(short, long)]
        filter: Option<String>,

        /// Save HAR file to path
        #[arg(short, long)]
        har: Option<PathBuf>,

        /// Duration to monitor in seconds
        #[arg(short, long, default_value = "30")]
        duration: u64,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();

    // Parse command line arguments
    let cli = Cli::parse();

    // Print banner
    print_banner();

    // Initialize the spinner
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{prefix:.bold.dim} {spinner} {wide_msg}")
            .unwrap(),
    );
    pb.set_prefix("[llama-moonlight]");
    
    // Initialize the framework
    pb.set_message("Initializing Llama Moonlight...".to_string());
    pb.enable_steady_tick(Duration::from_millis(100));
    
    let start = Instant::now();
    let moonlight = Moonlight::new().await?;
    
    // Get browser type
    let browser_type = moonlight
        .browser_type(&cli.browser)
        .ok_or_else(|| anyhow!("Browser type '{}' not found", cli.browser))?;
    
    // Configure browser options
    let mut options = BrowserOptions::default();
    options.headless = Some(cli.headless);
    options.stealth = Some(cli.stealth);
    
    // Launch browser
    pb.set_message(format!("Launching {} browser...", cli.browser));
    let browser = browser_type.launch_with_options(options).await?;
    
    // Configure context options
    let mut context_options = ContextOptions::default();
    if let Some(user_agent) = cli.user_agent {
        context_options.user_agent = Some(user_agent);
    }
    
    // Create a new context
    pb.set_message("Creating browser context...".to_string());
    let context = browser.new_context_with_options(context_options).await?;
    
    // Create a new page
    pb.set_message("Creating page...".to_string());
    let page = context.new_page().await?;
    
    // Execute the command
    match &cli.command {
        Commands::Screenshot { url, output, full_page } => {
            pb.set_message(format!("Navigating to {}", url));
            page.goto(url).await?;
            
            pb.set_message("Taking screenshot...".to_string());
            // TODO: Implement full_page screenshot option
            page.screenshot(output.to_str().unwrap()).await?;
            
            pb.finish_with_message(format!("Screenshot saved to {}", output.display()));
        }
        
        Commands::Content { url, output, format } => {
            pb.set_message(format!("Navigating to {}", url));
            page.goto(url).await?;
            
            pb.set_message("Getting page content...".to_string());
            let content = match format.to_lowercase().as_str() {
                "html" => page.content().await?,
                "text" => {
                    let script = "document.body.innerText";
                    page.evaluate::<String>(script).await?
                }
                _ => return Err(anyhow!("Unsupported format: {}", format)),
            };
            
            if let Some(path) = output {
                std::fs::write(path, content)?;
                pb.finish_with_message(format!("Content saved to {}", path.display()));
            } else {
                pb.finish();
                println!("{}", content);
            }
        }
        
        Commands::Evaluate { url, script } => {
            pb.set_message(format!("Navigating to {}", url));
            page.goto(url).await?;
            
            pb.set_message("Evaluating JavaScript...".to_string());
            let result = page.evaluate::<serde_json::Value>(script).await?;
            
            pb.finish();
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        
        Commands::Click { url, selector, screenshot } => {
            pb.set_message(format!("Navigating to {}", url));
            page.goto(url).await?;
            
            pb.set_message(format!("Clicking on element: {}", selector));
            page.click(selector).await?;
            
            if let Some(path) = screenshot {
                pb.set_message("Taking screenshot...".to_string());
                page.screenshot(path.to_str().unwrap()).await?;
                pb.finish_with_message(format!("Screenshot saved to {}", path.display()));
            } else {
                pb.finish_with_message("Click completed successfully".to_string());
            }
        }
        
        Commands::Fill { url, selector, text, submit } => {
            pb.set_message(format!("Navigating to {}", url));
            page.goto(url).await?;
            
            pb.set_message(format!("Filling in form field: {}", selector));
            page.type_text(selector, text).await?;
            
            if *submit {
                pb.set_message("Submitting form...".to_string());
                page.evaluate::<()>("document.querySelector('form').submit()").await?;
            }
            
            pb.finish_with_message("Form interaction completed successfully".to_string());
        }
        
        Commands::Extract { url, selector, attribute, format, output } => {
            pb.set_message(format!("Navigating to {}", url));
            page.goto(url).await?;
            
            pb.set_message(format!("Extracting data using selector: {}", selector));
            let script = format!(
                "Array.from(document.querySelectorAll('{}'))
                    .map(el => el.{})",
                selector, attribute
            );
            
            let data = page.evaluate::<Vec<String>>(&script).await?;
            
            let formatted_data = match format.to_lowercase().as_str() {
                "json" => serde_json::to_string_pretty(&data)?,
                "csv" => data.join("\n"),
                "text" => data.join("\n"),
                _ => return Err(anyhow!("Unsupported format: {}", format)),
            };
            
            if let Some(path) = output {
                std::fs::write(path, formatted_data)?;
                pb.finish_with_message(format!("Data saved to {}", path.display()));
            } else {
                pb.finish();
                println!("{}", formatted_data);
            }
        }
        
        Commands::Network { url, filter, har, duration } => {
            pb.set_message(format!("Navigating to {}", url));
            page.goto(url).await?;
            
            pb.set_message(format!("Monitoring network for {} seconds...", duration));
            
            // Set up network monitoring
            if let Some(path) = har {
                // TODO: Implement HAR recording functionality
            }
            
            // Monitor for specified duration
            tokio::time::sleep(Duration::from_secs(*duration)).await;
            
            pb.finish_with_message("Network monitoring completed".to_string());
        }
    }
    
    // Close the browser
    if cli.verbose {
        println!("Closing browser...");
    }
    browser.close().await?;
    
    // Print execution time
    let duration = start.elapsed();
    println!(
        "{} Operation completed in {:.2} seconds",
        "✓".green().bold(),
        duration.as_secs_f64()
    );
    
    Ok(())
}

fn print_banner() {
    println!("{}", "
 _      _                          __  __                     _ _       _     _   
| |    | |                        |  \\/  |                   | (_)     | |   | |  
| |    | | __ _ _ __ ___   __ _   | \\  / | ___   ___  _ __  | |_  __ _| |__ | |_ 
| |    | |/ _` | '_ ` _ \\ / _` |  | |\\/| |/ _ \\ / _ \\| '_ \\ | | |/ _` | '_ \\| __|
| |____| | (_| | | | | | | (_| |  | |  | | (_) | (_) | | | || | | (_| | | | | |_ 
|______|_|\\__,_|_| |_| |_|\\__,_|  |_|  |_|\\___/ \\___/|_| |_||_|_|\\__, |_| |_|\\__|
                                                                  __/ |          
                                                                 |___/           
    ".bright_magenta());
    println!("{}", "A powerful browser automation CLI for Rust".bright_yellow());
    println!("");
} 