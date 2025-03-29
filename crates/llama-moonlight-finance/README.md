# 🦙 llama-moonlight-finance

[![Crates.io](https://img.shields.io/crates/v/llama-moonlight-finance.svg)](https://crates.io/crates/llama-moonlight-finance)
[![Documentation](https://docs.rs/llama-moonlight-finance/badge.svg)](https://docs.rs/llama-moonlight-finance)
[![MIT/Apache-2.0 licensed](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](./LICENSE)

A powerful, comprehensive financial data and trading integration library for the Llama Moonlight ecosystem, providing access to market data, technical analysis, trading capabilities, and portfolio management.

## Features

- **Multi-Provider Support**: Integrates with various financial data providers:
  - Yahoo Finance, Alpha Vantage, CoinMarketCap, and more
  - Seamlessly switch between providers or use multiple simultaneously
  
- **Market Data Access**:
  - Real-time and historical price data
  - Company fundamentals and financial metrics
  - Options and futures chains
  - Forex and cryptocurrency markets
  
- **Trading Capabilities**:
  - Connect to brokerages and exchanges
  - Place, modify, and cancel orders
  - Monitor executions and positions
  - Paper trading for strategy testing
  
- **Technical Analysis**:
  - 50+ common technical indicators (SMA, EMA, RSI, MACD, etc.)
  - Pattern recognition and signals
  - Custom indicator creation
  - Backtesting frameworks
  
- **Portfolio Management**:
  - Track portfolio performance
  - Risk analysis and metrics
  - Asset allocation tools
  - Transaction history and reporting
  
- **Market Screeners**:
  - Multi-factor screening
  - Technical and fundamental filters
  - Customizable search criteria
  - Alert systems
  
- **Privacy and Stealth**:
  - Optional Tor integration for anonymous requests
  - Stealth mode to avoid detection
  - Request throttling to respect API limits
  
- **Advanced Capabilities**:
  - WebSocket streaming for real-time data
  - Sentiment analysis
  - Economic data and calendar events
  - Price visualization

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
llama-moonlight-finance = "0.1.0"
```

For additional features:

```toml
[dependencies]
llama-moonlight-finance = { version = "0.1.0", features = ["yahoo", "technical-analysis", "visualization"] }
```

## Quick Start

```rust
use llama_moonlight_finance::{FinanceClient, YahooProvider, Result};
use llama_moonlight_finance::data::{TimeInterval, TimeRange};

#[tokio::main]
async fn main() -> Result<()> {
    // Create a client with the Yahoo Finance provider
    let client = FinanceClient::new()
        .with_provider(YahooProvider::default())
        .build();
    
    // Fetch historical prices for Apple stock
    let prices = client.historical_prices("AAPL")
        .interval(TimeInterval::Daily)
        .range(TimeRange::Year(1))
        .fetch()
        .await?;
    
    // Calculate some basic statistics
    let stats = prices.statistics();
    println!("AAPL Average Price: ${:.2}", stats.mean);
    println!("AAPL Volatility: {:.2}%", stats.volatility * 100.0);
    
    Ok(())
}
```

## Market Data Providers

### Yahoo Finance

```rust
use llama_moonlight_finance::{FinanceClient, YahooProvider};

let yahoo = YahooProvider::default();
let client = FinanceClient::new().with_provider(yahoo).build();

// Get real-time quote
let quote = client.quote("MSFT").await?;
println!("Microsoft: ${} ({:.2}%)", quote.price, quote.change_percent);
```

### Alpha Vantage

```rust
use llama_moonlight_finance::{FinanceClient, AlphaVantageProvider};

let alpha = AlphaVantageProvider::new("YOUR_API_KEY");
let client = FinanceClient::new().with_provider(alpha).build();

// Get intraday prices
let prices = client.historical_prices("TSLA")
    .interval(TimeInterval::Minute5)
    .range(TimeRange::Day(1))
    .fetch()
    .await?;
```

### Multiple Providers

```rust
use llama_moonlight_finance::{FinanceClient, YahooProvider, AlphaVantageProvider};

let client = FinanceClient::new()
    .with_provider(YahooProvider::default())
    .with_provider(AlphaVantageProvider::new("YOUR_API_KEY"))
    .build();

// Use specific provider
let prices = client.historical_prices("BTC-USD")
    .provider("alphavantage")
    .fetch()
    .await?;
```

## Technical Analysis

```rust
use llama_moonlight_finance::analysis::indicators;

// Calculate technical indicators
let prices = client.historical_prices("SPY").fetch().await?;

// Simple Moving Average
let sma_20 = indicators::sma(&prices, 20);

// Exponential Moving Average
let ema_50 = indicators::ema(&prices, 50);

// Relative Strength Index
let rsi_14 = indicators::rsi(&prices, 14);

// MACD
let macd = indicators::macd(&prices, 12, 26, 9);

// Custom indicators can also be created
```

## Trading Integration

```rust
use llama_moonlight_finance::{trading::Order, trading::OrderType};

// Create a market order
let order = Order::new("AAPL", "buy", 10.0)
    .order_type(OrderType::Market)
    .time_in_force("day");

// Submit the order
let result = client.place_order(order).await?;
println!("Order ID: {}", result.order_id);

// Get current positions
let positions = client.positions().await?;
for position in positions {
    println!("{}: {} shares at ${}", position.symbol, position.quantity, position.average_price);
}
```

## Portfolio Management

```rust
use llama_moonlight_finance::portfolio::{Portfolio, Transaction, TransactionType};
use chrono::Utc;

// Create a portfolio
let mut portfolio = Portfolio::new("My Portfolio");

// Add transactions
portfolio.add_transaction(Transaction::new(
    "AAPL",
    TransactionType::Buy,
    10.0,
    150.0,
    Utc::now(),
));

// Calculate portfolio metrics
let metrics = portfolio.calculate_metrics(&client).await?;
println!("Portfolio Value: ${:.2}", metrics.total_value);
println!("Daily Change: {:.2}%", metrics.daily_change_percent);
```

## Visualization

With the `visualization` feature enabled:

```rust
use llama_moonlight_finance::visualization::ChartBuilder;

// Fetch data
let prices = client.historical_prices("NVDA").fetch().await?;

// Create and save a price chart
ChartBuilder::new(prices)
    .title("NVIDIA Stock Price")
    .add_sma(50)
    .add_sma(200)
    .add_volume()
    .add_rsi(14)
    .save("nvidia_chart.png")?;
```

## Privacy and Stealth

### Tor Integration

With the `tor` feature enabled:

```rust
use llama_moonlight_finance::{ClientConfig, FinanceClient};
use llama_moonlight_finance::tor::TorConfig;

// Configure Tor
let config = ClientConfig::default()
    .with_tor(TorConfig::default());

let client = FinanceClient::with_config(config).build();

// All requests will now go through Tor
let prices = client.historical_prices("BTC-USD").fetch().await?;
```

### Stealth Mode

With the `stealth` feature enabled:

```rust
use llama_moonlight_finance::{ClientConfig, FinanceClient};
use llama_moonlight_finance::stealth::StealthConfig;

// Configure stealth mode
let config = ClientConfig::default()
    .with_stealth(StealthConfig::default());

let client = FinanceClient::with_config(config).build();

// Requests will use techniques to avoid detection
```

## Features

The crate can be compiled with the following features:

- `standard` (default): Core functionality for financial data access
- `yahoo`: Yahoo Finance API integration
- `alphavantage`: Alpha Vantage API integration
- `coinmarketcap`: CoinMarketCap API integration
- `binance`: Binance exchange integration
- `ftx`: FTX exchange integration
- `tradingview`: TradingView integration
- `technical-analysis`: Technical indicators and analysis tools
- `visualization`: Charting and visualization tools
- `data-science`: Advanced statistical and machine learning tools
- `stealth`: Features to help avoid detection
- `tor`: Integration with Tor for enhanced privacy
- `browser`: Integration with browser automation
- `full`: Enables all features

## Examples

See the [examples](examples/) directory for more comprehensive examples:

- `basic_client.rs`: Simple client for accessing market data
- `historical_data.rs`: Working with historical price data
- `technical_analysis.rs`: Using technical indicators
- `portfolio_tracking.rs`: Managing a portfolio
- `trading.rs`: Trading API integration
- `crypto.rs`: Cryptocurrency market data
- `visualization.rs`: Chart creation
- `screening.rs`: Stock screening
- `stealth.rs`: Stealth mode to avoid detection
- `tor.rs`: Anonymous browsing via Tor

## Documentation

For full API documentation, please visit [docs.rs/llama-moonlight-finance](https://docs.rs/llama-moonlight-finance).

## License

This project is licensed under either of:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Disclaimer

This library is not officially affiliated with any of the data providers or trading platforms it integrates with. Use responsibly and in accordance with the terms of service of each provider. Financial data and trading involve risks, and this library does not provide financial advice. 