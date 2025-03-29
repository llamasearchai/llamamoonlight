use llama_headers_rs::{get_header, get_headers, Config};
use llama_headers_rs::user_agent::UserAgent;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Simple usage with defaults
    let url = "https://example.com";
    let header = get_header(url, None)?;
    
    println!("=== Simple Header ===");
    println!("{}", header);
    println!();
    
    // Custom config for a mobile browser in Germany
    let mobile_config = Config::new()
        .with_language("de-DE")
        .with_mobile(true)
        .with_referer("https://www.google.de");
    
    let mobile_header = get_header("https://example.de", Some(mobile_config))?;
    
    println!("=== Mobile Header (German) ===");
    println!("{}", mobile_header);
    println!();
    
    // Generate multiple different headers
    println!("=== Multiple Headers ===");
    let headers = get_headers(url, 3, None)?;
    for (i, header) in headers.iter().enumerate() {
        println!("Header #{}", i + 1);
        println!("User-Agent: {}", header.user_agent);
        println!("Accept-Language: {}", header.get("Accept-Language").unwrap_or(&"".to_string()));
        println!();
    }
    
    // Using specific browser factories
    println!("=== Specific Browser Examples ===");
    
    // Chrome on Windows
    let chrome_ua = UserAgent::chrome(true)?;
    let chrome_config = Config::new().with_user_agent(chrome_ua);
    let chrome_header = get_header(url, Some(chrome_config))?;
    println!("Chrome on Windows:");
    println!("User-Agent: {}", chrome_header.user_agent);
    println!("Sec-Ch-Ua: {}", chrome_header.get("Sec-Ch-Ua").unwrap_or(&"".to_string()));
    println!();
    
    // Firefox on macOS
    let firefox_ua = UserAgent::firefox(false)?;
    let firefox_config = Config::new().with_user_agent(firefox_ua);
    let firefox_header = get_header(url, Some(firefox_config))?;
    println!("Firefox on macOS:");
    println!("User-Agent: {}", firefox_header.user_agent);
    println!();
    
    // Safari
    let safari_ua = UserAgent::safari()?;
    let safari_config = Config::new().with_user_agent(safari_ua);
    let safari_header = get_header(url, Some(safari_config))?;
    println!("Safari:");
    println!("User-Agent: {}", safari_header.user_agent);
    println!();
    
    // Custom modification of headers
    let custom_header = get_header(url, None)?
        .with_header("X-Custom-Header", "CustomValue")
        .without_header("Referer");
    
    println!("=== Customized Header ===");
    println!("{}", custom_header);
    
    Ok(())
} 