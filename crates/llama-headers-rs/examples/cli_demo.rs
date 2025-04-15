use llama_headers_rs::{get_header, get_headers, Config};
use llama_headers_rs::user_agent::UserAgent;
use std::io::{self, Write};
use std::process;

// ANSI color codes
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const GREEN: &str = "\x1b[32m";
const BLUE: &str = "\x1b[34m";
const CYAN: &str = "\x1b[36m";
const MAGENTA: &str = "\x1b[35m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";

fn print_header() {
    println!();
    println!("{}{}", MAGENTA, BOLD);
    println!("  _      _                                _    _                _               ");
    println!(" | |    | |                              | |  | |              | |              ");
    println!(" | |    | | __ _ _ __ ___   __ _       __| |__| | ___  __ _  __| | ___ _ __ ___ ");
    println!(" | |    | |/ _` | '_ ` _ \\ / _` |     / _` |__  |/ _ \\/ _` |/ _` |/ _ \\ '__/ __|");
    println!(" | |____| | (_| | | | | | | (_| |    | (_| |  | |  __/ (_| | (_| |  __/ |  \\__ \\");
    println!(" |______|_|\\__,_|_| |_| |_|\\__,_|     \\__,_|  |_|\\___|\\__,_|\\__,_|\\___|_|  |___/");
    println!("                                                                               ");
    println!("{}", RESET);
    println!("{}            A sophisticated HTTP header generator for Rust{}", YELLOW, RESET);
    println!();
}

fn print_menu() {
    println!("\n{}{}===== MENU ====={}", BLUE, BOLD, RESET);
    println!("{}1. Generate a default header", CYAN);
    println!("2. Generate a mobile header");
    println!("3. Generate multiple different headers");
    println!("4. Generate browser-specific headers");
    println!("5. Generate a custom header");
    println!("6. Generate a header and test with httpbin.org");
    println!("{}0. Exit{}", RED, RESET);
    print!("\n{}Enter your choice: {}", GREEN, RESET);
    io::stdout().flush().unwrap();
}

fn get_input() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    input.trim().to_string()
}

fn generate_default_header() {
    println!("\n{}{}===== DEFAULT HEADER ====={}", BLUE, BOLD, RESET);
    
    print!("{}Enter URL (default: https://example.com): {}", GREEN, RESET);
    io::stdout().flush().unwrap();
    let input = get_input();
    let url = if input.is_empty() { "https://example.com" } else { &input };
    
    match get_header(url, None) {
        Ok(header) => {
            println!("\n{}{}Generated Headers:{}", CYAN, BOLD, RESET);
            for (key, value) in header.get_map() {
                println!("{}{}{}: {}", YELLOW, key, RESET, value);
            }
        },
        Err(e) => println!("{}Error: {}{}", RED, e, RESET),
    }
}

fn generate_mobile_header() {
    println!("\n{}{}===== MOBILE HEADER ====={}", BLUE, BOLD, RESET);
    
    print!("{}Enter URL (default: https://example.com): {}", GREEN, RESET);
    io::stdout().flush().unwrap();
    let input = get_input();
    let url = if input.is_empty() { "https://example.com" } else { &input };
    
    print!("{}Enter language (default: en-US): {}", GREEN, RESET);
    io::stdout().flush().unwrap();
    let input = get_input();
    let language = if input.is_empty() { "en-US" } else { &input };
    
    let config = Config::new()
        .with_language(language)
        .with_mobile(true);
    
    match get_header(url, Some(config)) {
        Ok(header) => {
            println!("\n{}{}Generated Mobile Headers:{}", CYAN, BOLD, RESET);
            for (key, value) in header.get_map() {
                println!("{}{}{}: {}", YELLOW, key, RESET, value);
            }
        },
        Err(e) => println!("{}Error: {}{}", RED, e, RESET),
    }
}

fn generate_multiple_headers() {
    println!("\n{}{}===== MULTIPLE HEADERS ====={}", BLUE, BOLD, RESET);
    
    print!("{}Enter URL (default: https://example.com): {}", GREEN, RESET);
    io::stdout().flush().unwrap();
    let input = get_input();
    let url = if input.is_empty() { "https://example.com" } else { &input };
    
    print!("{}How many headers? (default: 3): {}", GREEN, RESET);
    io::stdout().flush().unwrap();
    let input = get_input();
    let count = if input.is_empty() { 
        3 
    } else { 
        input.parse::<usize>().unwrap_or(3) 
    };
    
    match get_headers(url, count, None) {
        Ok(headers) => {
            for (i, header) in headers.iter().enumerate() {
                println!("\n{}{}Header #{}{}", CYAN, BOLD, i+1, RESET);
                println!("{}User-Agent: {}{}", MAGENTA, RESET, header.user_agent);
                
                // Print a selection of important headers
                let important_headers = ["Host", "Accept-Language", "Referer", "Sec-Ch-Ua", "Sec-Ch-Ua-Mobile"];
                for &key in &important_headers {
                    if let Some(value) = header.get(key) {
                        println!("{}{}{}: {}", YELLOW, key, RESET, value);
                    }
                }
            }
        },
        Err(e) => println!("{}Error: {}{}", RED, e, RESET),
    }
}

fn generate_browser_specific_headers() {
    println!("\n{}{}===== BROWSER-SPECIFIC HEADERS ====={}", BLUE, BOLD, RESET);
    
    print!("{}Enter URL (default: https://example.com): {}", GREEN, RESET);
    io::stdout().flush().unwrap();
    let input = get_input();
    let url = if input.is_empty() { "https://example.com" } else { &input };
    
    // Chrome
    println!("\n{}{}Chrome on Windows:{}", CYAN, BOLD, RESET);
    match UserAgent::chrome(true) {
        Ok(ua) => {
            let config = Config::new().with_user_agent(ua);
            match get_header(url, Some(config)) {
                Ok(header) => {
                    println!("{}User-Agent: {}{}", MAGENTA, RESET, header.user_agent);
                    if let Some(sec_ch_ua) = header.get("Sec-Ch-Ua") {
                        println!("{}Sec-Ch-Ua: {}{}", YELLOW, RESET, sec_ch_ua);
                    }
                },
                Err(e) => println!("{}Error: {}{}", RED, e, RESET),
            }
        },
        Err(e) => println!("{}Error: {}{}", RED, e, RESET),
    }
    
    // Firefox
    println!("\n{}{}Firefox on macOS:{}", CYAN, BOLD, RESET);
    match UserAgent::firefox(false) {
        Ok(ua) => {
            let config = Config::new().with_user_agent(ua);
            match get_header(url, Some(config)) {
                Ok(header) => {
                    println!("{}User-Agent: {}{}", MAGENTA, RESET, header.user_agent);
                },
                Err(e) => println!("{}Error: {}{}", RED, e, RESET),
            }
        },
        Err(e) => println!("{}Error: {}{}", RED, e, RESET),
    }
    
    // Safari
    println!("\n{}{}Safari:{}", CYAN, BOLD, RESET);
    match UserAgent::safari() {
        Ok(ua) => {
            let config = Config::new().with_user_agent(ua);
            match get_header(url, Some(config)) {
                Ok(header) => {
                    println!("{}User-Agent: {}{}", MAGENTA, RESET, header.user_agent);
                },
                Err(e) => println!("{}Error: {}{}", RED, e, RESET),
            }
        },
        Err(e) => println!("{}Error: {}{}", RED, e, RESET),
    }
}

fn generate_custom_header() {
    println!("\n{}{}===== CUSTOM HEADER ====={}", BLUE, BOLD, RESET);
    
    print!("{}Enter URL (default: https://example.com): {}", GREEN, RESET);
    io::stdout().flush().unwrap();
    let input = get_input();
    let url = if input.is_empty() { "https://example.com" } else { &input };
    
    let mut config = Config::new();
    
    // Language
    print!("{}Enter language (default: en-US): {}", GREEN, RESET);
    io::stdout().flush().unwrap();
    let input = get_input();
    if !input.is_empty() {
        config = config.with_language(&input);
    }
    
    // Mobile
    print!("{}Mobile browser? (y/N): {}", GREEN, RESET);
    io::stdout().flush().unwrap();
    let input = get_input();
    if input.to_lowercase() == "y" {
        config = config.with_mobile(true);
    }
    
    // Referer
    print!("{}Enter referer (optional): {}", GREEN, RESET);
    io::stdout().flush().unwrap();
    let input = get_input();
    if !input.is_empty() {
        config = config.with_referer(&input);
    }
    
    // Custom header
    print!("{}Add custom header? (y/N): {}", GREEN, RESET);
    io::stdout().flush().unwrap();
    let input = get_input();
    if input.to_lowercase() == "y" {
        print!("{}Header name: {}", GREEN, RESET);
        io::stdout().flush().unwrap();
        let key = get_input();
        
        print!("{}Header value: {}", GREEN, RESET);
        io::stdout().flush().unwrap();
        let value = get_input();
        
        if !key.is_empty() && !value.is_empty() {
            config = config.with_custom_header(&key, &value);
        }
    }
    
    match get_header(url, Some(config)) {
        Ok(header) => {
            println!("\n{}{}Generated Custom Headers:{}", CYAN, BOLD, RESET);
            for (key, value) in header.get_map() {
                println!("{}{}{}: {}", YELLOW, key, RESET, value);
            }
        },
        Err(e) => println!("{}Error: {}{}", RED, e, RESET),
    }
}

fn test_with_httpbin() {
    println!("{}{}This feature requires the reqwest feature which is enabled in the actual demo, but shown here for demonstration purposes.{}", 
        YELLOW, BOLD, RESET);
    println!("{}To run this, add the reqwest dependency and tokio runtime.{}", YELLOW, RESET);
}

fn main() {
    print_header();
    
    loop {
        print_menu();
        let choice = get_input();
        
        match choice.as_str() {
            "0" => {
                println!("\n{}Goodbye!{}", GREEN, RESET);
                process::exit(0);
            },
            "1" => generate_default_header(),
            "2" => generate_mobile_header(),
            "3" => generate_multiple_headers(),
            "4" => generate_browser_specific_headers(),
            "5" => generate_custom_header(),
            "6" => test_with_httpbin(),
            _ => println!("{}Invalid choice, please try again.{}", RED, RESET),
        }
        
        println!("\n{}Press Enter to continue...{}", GREEN, RESET);
        let _ = get_input();
    }
} 