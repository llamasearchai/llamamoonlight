use llama_headers_rs::{get_header, Config};
use llama_headers_rs::user_agent::UserAgent;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Target URL
    let url = "https://httpbin.org/headers";
    
    // Generate header with default settings
    let header = get_header(url, None)?;
    
    // Create a reqwest client
    let client = reqwest::Client::new();
    
    // Build a request with our generated headers
    let mut request_builder = client.get(url);
    
    // Add all headers from our Header instance
    for (key, value) in header.get_map() {
        request_builder = request_builder.header(key, value);
    }
    
    println!("🚀 Sending request to {} with llama-headers-rs", url);
    println!("\n📋 Request Headers:");
    for (key, value) in header.get_map() {
        println!("   {} = {}", key, value);
    }
    
    // Send the request
    let response = request_builder.send().await?;
    let status = response.status();
    let body = response.text().await?;
    
    println!("\n✅ Response Status: {}", status);
    println!("\n📝 Response Body:");
    println!("{}", body);
    
    // Now let's try with a different browser profile
    println!("\n\n🔄 Trying with a mobile Safari profile");
    
    // Create a mobile Safari configuration
    let mobile_ua = UserAgent::parse("Mozilla/5.0 (iPhone; CPU iPhone OS 15_4 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.4 Mobile/15E148 Safari/604.1")?;
    let config = Config::new()
        .with_user_agent(mobile_ua)
        .with_language("en-US")
        .with_mobile(true)
        .with_custom_header("X-Requested-With", "XMLHttpRequest");
    
    let mobile_header = get_header(url, Some(config))?;
    
    // Build a new request with mobile headers
    let mut mobile_request_builder = client.get(url);
    
    // Add all headers from our mobile Header instance
    for (key, value) in mobile_header.get_map() {
        mobile_request_builder = mobile_request_builder.header(key, value);
    }
    
    println!("\n📋 Mobile Request Headers:");
    for (key, value) in mobile_header.get_map() {
        println!("   {} = {}", key, value);
    }
    
    // Send the mobile request
    let mobile_response = mobile_request_builder.send().await?;
    let mobile_status = mobile_response.status();
    let mobile_body = mobile_response.text().await?;
    
    println!("\n✅ Mobile Response Status: {}", mobile_status);
    println!("\n📝 Mobile Response Body:");
    println!("{}", mobile_body);
    
    Ok(())
} 