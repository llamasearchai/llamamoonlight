use llama_moonlight_core::{Moonlight, BrowserType};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use anyhow::{Result, anyhow};

#[derive(Parser)]
#[command(name = "llama-moonlight")]
#[command(author = "Your Name <your.email@example.com>")]
#[command(version = "0.1.0")]
#[command(about = "A powerful browser automation framework with MLX and Llama integration", long_about = None)]
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
    #[arg(short, long, default_value = "false")]
    stealth: bool,
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
    },
    
    /// Get the content of a webpage
    Content {
        /// The URL to navigate to
        url: String,
        
        /// The path to save the content to
        #[arg(short, long)]
        output: Option<PathBuf>,
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
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();
    
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Create the Moonlight instance
    let moonlight = Moonlight::new().await
        .map_err(|e| anyhow!("Failed to initialize Moonlight: {}", e))?;
    
    // Get the browser type
    let browser_type = moonlight.browser_type(&cli.browser)
        .ok_or_else(|| anyhow!("Unsupported browser type: {}", cli.browser))?;
    
    // Create browser options
    let options = llama_moonlight_core::options::BrowserOptions {
        headless: Some(cli.headless),
        stealth: Some(cli.stealth),
        ..Default::default()
    };
    
    // Launch the browser
    let browser = browser_type.launch_with_options(options).await
        .map_err(|e| anyhow!("Failed to launch browser: {}", e))?;
    
    // Create a new context
    let context = browser.new_context().await
        .map_err(|e| anyhow!("Failed to create context: {}", e))?;
    
    // Execute the command
    match &cli.command {
        Commands::Screenshot { url, output } => {
            // Create a new page
            let page = context.new_page().await
                .map_err(|e| anyhow!("Failed to create page: {}", e))?;
            
            // Navigate to the URL
            page.goto(url).await
                .map_err(|e| anyhow!("Failed to navigate to {}: {}", url, e))?;
            
            // Take a screenshot
            page.screenshot(output.to_str().unwrap()).await
                .map_err(|e| anyhow!("Failed to take screenshot: {}", e))?;
            
            println!("Screenshot saved to {:?}", output);
        },
        Commands::Content { url, output } => {
            // Create a new page
            let page = context.new_page().await
                .map_err(|e| anyhow!("Failed to create page: {}", e))?;
            
            // Navigate to the URL
            page.goto(url).await
                .map_err(|e| anyhow!("Failed to navigate to {}: {}", url, e))?;
            
            // Get the content
            let content = page.content().await
                .map_err(|e| anyhow!("Failed to get content: {}", e))?;
            
            // Either save to file or print to stdout
            if let Some(path) = output {
                std::fs::write(path, content)
                    .map_err(|e| anyhow!("Failed to write content to file: {}", e))?;
                println!("Content saved to {:?}", path);
            } else {
                println!("{}", content);
            }
        },
        Commands::Evaluate { url, script } => {
            // Create a new page
            let page = context.new_page().await
                .map_err(|e| anyhow!("Failed to create page: {}", e))?;
            
            // Navigate to the URL
            page.goto(url).await
                .map_err(|e| anyhow!("Failed to navigate to {}: {}", url, e))?;
            
            // Evaluate the script
            let result: serde_json::Value = page.evaluate(script).await
                .map_err(|e| anyhow!("Failed to evaluate script: {}", e))?;
            
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
        },
        Commands::Click { url, selector } => {
            // Create a new page
            let page = context.new_page().await
                .map_err(|e| anyhow!("Failed to create page: {}", e))?;
            
            // Navigate to the URL
            page.goto(url).await
                .map_err(|e| anyhow!("Failed to navigate to {}: {}", url, e))?;
            
            // Click the element
            page.click(selector).await
                .map_err(|e| anyhow!("Failed to click element: {}", e))?;
            
            println!("Clicked element with selector: {}", selector);
        },
        Commands::Fill { url, selector, text } => {
            // Create a new page
            let page = context.new_page().await
                .map_err(|e| anyhow!("Failed to create page: {}", e))?;
            
            // Navigate to the URL
            page.goto(url).await
                .map_err(|e| anyhow!("Failed to navigate to {}: {}", url, e))?;
            
            // Fill the input
            page.type_text(selector, text).await
                .map_err(|e| anyhow!("Failed to fill input: {}", e))?;
            
            println!("Filled input with selector: {}", selector);
        },
    }
    
    // Close the browser
    browser.close().await
        .map_err(|e| anyhow!("Failed to close browser: {}", e))?;
    
    Ok(())
} 