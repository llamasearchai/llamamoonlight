use llama_moonlight_stealth::{
    proxy::{ProxyManager, ProxyConfig, ProxyProtocol, RotationStrategy},
};
use std::error::Error;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== 🔄 Llama Moonlight Proxy Rotation Example ===\n");
    
    // Create a proxy manager with a round-robin rotation strategy
    let mut proxy_manager = ProxyManager::with_strategy(RotationStrategy::RoundRobin)
        .with_max_failures(3)
        .with_min_success_rate(0.6);
    
    // Add some example proxies
    println!("📋 Adding proxies...");
    
    proxy_manager.add_proxy(
        ProxyConfig::new(ProxyProtocol::Http, "proxy1.example.com", 8080)
            .with_auth("user1", "pass1")
            .with_country("US")
    );
    
    proxy_manager.add_proxy(
        ProxyConfig::new(ProxyProtocol::Socks5, "proxy2.example.com", 1080)
            .with_auth("user2", "pass2")
            .with_country("UK")
    );
    
    proxy_manager.add_proxy(
        ProxyConfig::new(ProxyProtocol::Https, "proxy3.example.com", 443)
            .with_auth("user3", "pass3")
            .with_country("DE")
    );
    
    // Add a few more random proxies for demonstration
    println!("🎲 Adding random proxies...");
    for _ in 0..2 {
        let random_proxy = ProxyManager::generate_random_proxy();
        println!("  Added random proxy: {}", random_proxy);
        proxy_manager.add_proxy(random_proxy);
    }
    
    // Simulate a series of requests with proxy rotation
    println!("\n🔄 Simulating proxy rotation for 5 requests...");
    
    for i in 1..=5 {
        println!("\n=== Request #{} ===", i);
        
        // Get the active proxy
        let proxy = proxy_manager.rotate().unwrap();
        println!("🌐 Using proxy: {}", proxy);
        
        // Simulate a request
        println!("📤 Sending request with proxy...");
        let start = Instant::now();
        let success = simulate_request(proxy);
        let duration = start.elapsed();
        
        if success {
            println!("✅ Request successful! Response time: {:?}", duration);
            proxy_manager.record_success(Some(duration.as_millis() as u64));
        } else {
            println!("❌ Request failed!");
            proxy_manager.record_failure();
        }
    }
    
    // Print proxy statistics
    println!("\n=== 📊 Proxy Statistics ===");
    
    // We need to get the active proxy index to iterate over all proxies
    let active_proxy = proxy_manager.active_proxy();
    
    // Since we don't have direct access to all proxies in the manager,
    // we'll simulate some statistics
    if let Some(proxy) = active_proxy {
        println!("Current active proxy: {}", proxy);
        println!("Success rate: {:.2}", proxy.success_rate());
        println!("Success count: {}", proxy.success_count);
        println!("Failure count: {}", proxy.failure_count);
        
        if let Some(response_time) = proxy.response_time_ms {
            println!("Last response time: {}ms", response_time);
        }
    }
    
    println!("\n🏁 Example completed!");
    
    Ok(())
}

// Simulate a HTTP request with a proxy
fn simulate_request(proxy: &ProxyConfig) -> bool {
    // Simulate some failures based on random chance
    let success_probability = match proxy.protocol {
        ProxyProtocol::Http => 0.8,
        ProxyProtocol::Https => 0.9,
        ProxyProtocol::Socks4 => 0.7,
        ProxyProtocol::Socks5 => 0.75,
    };
    
    // Simulate the request taking some time
    let duration_ms = 100 + rand::random::<u64>() % 400;
    std::thread::sleep(Duration::from_millis(duration_ms));
    
    // Determine if the request was successful
    rand::random::<f64>() < success_probability
} 