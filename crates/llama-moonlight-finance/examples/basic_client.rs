use llama_moonlight_finance::{FinanceClient, Result};
use llama_moonlight_finance::data::{TimeInterval, TimeRange};
use llama_moonlight_finance::config::ClientConfig;
use std::time::Duration;

// This example demonstrates basic usage of the FinanceClient to fetch
// and analyze historical stock price data

#[tokio::main]
async fn main() -> Result<()> {
    // Create a client configuration
    let config = ClientConfig::default()
        .with_user_agent("llama-moonlight-finance-example/0.1.0")
        .with_timeout(Duration::from_secs(30))
        .with_verbose_logging(true);
        
    // In a real application, you would use a real provider:
    // use llama_moonlight_finance::providers::yahoo::YahooProvider;
    // let client = FinanceClient::with_config(config)
    //     .with_provider(YahooProvider::default())
    //     .build();
    
    // For this example, we'll use a mock provider that returns fake data
    let client = create_mock_client(config);
    
    // Define the symbols we want to analyze
    let symbols = vec!["AAPL", "MSFT", "GOOG", "AMZN"];
    
    // Fetch and analyze historical data for each symbol
    for symbol in symbols {
        println!("\n--- Analyzing {} ---", symbol);
        
        // Fetch historical prices for the past year
        let prices = client.historical_prices(symbol)
            .interval(TimeInterval::Daily)
            .range(TimeRange::Year(1))
            .fetch()
            .await?;
            
        // Calculate basic statistics
        let stats = prices.statistics();
        
        println!("Period: {} to {}", prices.start_time.date(), prices.end_time.date());
        println!("Data points: {}", stats.count);
        println!("Average price: ${:.2}", stats.mean);
        println!("Median price: ${:.2}", stats.median);
        println!("Price range: ${:.2} - ${:.2}", stats.min, stats.max);
        println!("Volatility: {:.2}%", stats.volatility * 100.0);
        println!("Total volume: {}", stats.total_volume);
        
        // Calculate technical indicators
        let sma_50 = prices.sma(50);
        let ema_20 = prices.ema(20);
        
        if !sma_50.is_empty() && !ema_20.is_empty() {
            println!("Last SMA(50): ${:.2}", sma_50.last().unwrap());
            println!("Last EMA(20): ${:.2}", ema_20.last().unwrap());
        }
        
        // Get the latest quote
        let quote = client.quote(symbol).await?;
        println!("Current price: ${:.2} ({:+.2}%)", quote.price, quote.change_percent);
        
        // Simple trading signal example
        if !sma_50.is_empty() && !ema_20.is_empty() {
            let latest_sma = *sma_50.last().unwrap();
            let latest_ema = *ema_20.last().unwrap();
            
            if latest_ema > latest_sma {
                println!("Signal: BULLISH (EMA20 above SMA50)");
            } else {
                println!("Signal: BEARISH (EMA20 below SMA50)");
            }
        }
    }
    
    // Get quotes for all symbols at once
    let symbols_slice: Vec<&str> = symbols.iter().map(|s| *s).collect();
    let quotes = client.quotes(&symbols_slice).await?;
    
    println!("\n--- Current Quotes ---");
    for (symbol, quote) in quotes {
        println!("{}: ${:.2} ({:+.2}%)", symbol, quote.price, quote.change_percent);
    }
    
    // In a real application, you might want to execute trades:
    // use llama_moonlight_finance::trading::{Order, OrderType};
    //
    // let order = Order::new("AAPL", "buy", 10.0)
    //     .order_type(OrderType::Market.to_string());
    //
    // let order_status = client.place_order(order).await?;
    // println!("Order placed: {}", order_status.order_id);
    
    // Get client usage statistics
    let stats = client.stats().await;
    println!("\nClient statistics:");
    println!("Request count: {}", stats.request_count);
    println!("Providers: {:?}", stats.data_providers);
    
    Ok(())
}

// Create a mock client for demonstration purposes
fn create_mock_client(config: ClientConfig) -> FinanceClient {
    use std::sync::Arc;
    use async_trait::async_trait;
    use chrono::Utc;
    use std::collections::HashMap;
    use llama_moonlight_finance::provider::{Provider, DataProvider, ProviderType, Capability};
    use llama_moonlight_finance::data::{TimeInterval, TimeRange, TimeSeries, Price, Quote, MarketData};
    use llama_moonlight_finance::AssetClass;
    
    struct MockProvider;
    
    impl Provider for MockProvider {
        fn name(&self) -> &str {
            "mock"
        }
        
        fn provider_type(&self) -> ProviderType {
            ProviderType::MarketData
        }
        
        fn capabilities(&self) -> Vec<Capability> {
            vec![Capability::HistoricalPrices, Capability::RealTimeQuotes]
        }
        
        fn as_data_provider(&self) -> Option<Arc<dyn DataProvider>> {
            Some(Arc::new(MockProvider))
        }
    }
    
    #[async_trait]
    impl DataProvider for MockProvider {
        async fn quote(&self, symbol: &str) -> Result<Quote> {
            let base_price = match symbol {
                "AAPL" => 175.0,
                "MSFT" => 350.0,
                "GOOG" => 140.0,
                "AMZN" => 178.0,
                _ => 100.0,
            };
            
            // Add a small random variation
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let change_percent = rng.gen_range(-2.0..2.0);
            let change = base_price * change_percent / 100.0;
            let price = base_price + change;
            
            Ok(Quote {
                symbol: symbol.to_string(),
                price,
                change,
                change_percent,
                volume: rng.gen_range(1_000_000..10_000_000),
                market_cap: Some(base_price * 1_000_000_000.0),
                timestamp: Utc::now(),
                exchange: Some("NASDAQ".to_string()),
                currency: Some("USD".to_string()),
                additional_data: HashMap::new(),
            })
        }
        
        async fn quotes(&self, symbols: &[&str]) -> Result<HashMap<String, Quote>> {
            let mut result = HashMap::new();
            for symbol in symbols {
                let quote = self.quote(symbol).await?;
                result.insert(symbol.to_string(), quote);
            }
            Ok(result)
        }
        
        async fn historical_prices(
            &self,
            symbol: &str,
            interval: TimeInterval,
            range: TimeRange,
            _include_extended: bool,
            _adjust: bool,
            _limit: Option<u32>,
        ) -> Result<TimeSeries<Price>> {
            use chrono::Duration;
            
            let (start_date, end_date) = range.to_date_range();
            let step = match interval {
                TimeInterval::Daily => Duration::days(1),
                TimeInterval::Weekly => Duration::weeks(1),
                TimeInterval::Monthly => Duration::days(30),
                _ => Duration::days(1), // Default to daily for this example
            };
            
            let base_price = match symbol {
                "AAPL" => 175.0,
                "MSFT" => 350.0,
                "GOOG" => 140.0,
                "AMZN" => 178.0,
                _ => 100.0,
            };
            
            let mut prices = Vec::new();
            let mut current_date = start_date;
            let mut current_price = base_price * 0.8; // Start 20% below current price
            
            let mut rng = rand::thread_rng();
            
            while current_date <= end_date {
                // Skip weekends
                let weekday = current_date.weekday().num_days_from_monday();
                if weekday <= 4 { // Monday to Friday
                    let daily_change = current_price * rng.gen_range(-0.02..0.02);
                    current_price += daily_change;
                    
                    let price = Price {
                        open: current_price * (1.0 - rng.gen_range(0.0..0.01)),
                        high: current_price * (1.0 + rng.gen_range(0.0..0.02)),
                        low: current_price * (1.0 - rng.gen_range(0.0..0.02)),
                        close: current_price,
                        volume: rng.gen_range(100_000..10_000_000),
                        timestamp: current_date,
                        additional_data: HashMap::new(),
                    };
                    
                    prices.push(price);
                }
                
                current_date = current_date + step;
            }
            
            Ok(TimeSeries {
                symbol: symbol.to_string(),
                interval,
                prices,
                start_time: start_date,
                end_time: end_date,
                timezone: "UTC".to_string(),
                currency: "USD".to_string(),
            })
        }
        
        async fn search(&self, query: &str, _asset_class: Option<AssetClass>) -> Result<Vec<MarketData>> {
            let symbols = ["AAPL", "MSFT", "GOOG", "AMZN"];
            let mut results = Vec::new();
            
            for &symbol in symbols.iter() {
                if symbol.to_lowercase().contains(&query.to_lowercase()) {
                    let name = match symbol {
                        "AAPL" => "Apple Inc.",
                        "MSFT" => "Microsoft Corporation",
                        "GOOG" => "Alphabet Inc.",
                        "AMZN" => "Amazon.com, Inc.",
                        _ => "Unknown Corporation",
                    };
                    
                    results.push(MarketData {
                        symbol: symbol.to_string(),
                        name: name.to_string(),
                        instrument_type: "stock".to_string(),
                        exchange: "NASDAQ".to_string(),
                        currency: "USD".to_string(),
                        country: Some("US".to_string()),
                        index_membership: Some(vec!["S&P 500".to_string(), "NASDAQ 100".to_string()]),
                        sector: Some("Technology".to_string()),
                        industry: Some("Consumer Electronics".to_string()),
                        market_cap_category: Some("Large Cap".to_string()),
                        is_etf: false,
                        additional_data: HashMap::new(),
                    });
                }
            }
            
            Ok(results)
        }
    }
    
    FinanceClient::with_config(config)
        .with_provider(MockProvider)
        .build()
} 