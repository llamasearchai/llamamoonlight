use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{Mutex, RwLock};
use async_trait::async_trait;
use reqwest::Client as HttpClient;
use chrono::{DateTime, Utc};

use crate::{Error, Result, AssetClass};
use crate::config::ClientConfig;
use crate::provider::{Provider, DataProvider, TradingProvider};
use crate::data::{TimeInterval, TimeRange, TimeSeries, Price, Quote, MarketData};
use crate::market::{OrderBook, TradeHistory};
use crate::trading::{Order, OrderStatus, Position, TradeExecution};
use crate::portfolio::{Portfolio, Transaction};

/// The main client for accessing financial data and trading functionality
pub struct FinanceClient {
    /// HTTP client for API requests
    http_client: HttpClient,
    
    /// Client configuration
    config: ClientConfig,
    
    /// Registered data providers
    data_providers: HashMap<String, Arc<dyn DataProvider>>,
    
    /// Registered trading providers
    trading_providers: HashMap<String, Arc<dyn TradingProvider>>,
    
    /// Default data provider
    default_data_provider: Option<String>,
    
    /// Default trading provider
    default_trading_provider: Option<String>,
    
    /// User portfolio
    portfolio: Arc<RwLock<Option<Portfolio>>>,
    
    /// API request counter
    request_count: Arc<Mutex<u64>>,
}

impl FinanceClient {
    /// Create a new client with default configuration
    pub fn new() -> Self {
        Self::with_config(ClientConfig::default())
    }
    
    /// Create a new client with a custom configuration
    pub fn with_config(config: ClientConfig) -> Self {
        let http_client = HttpClient::builder()
            .timeout(config.timeout)
            .user_agent(&config.user_agent)
            .build()
            .unwrap_or_default();
            
        Self {
            http_client,
            config,
            data_providers: HashMap::new(),
            trading_providers: HashMap::new(),
            default_data_provider: None,
            default_trading_provider: None,
            portfolio: Arc::new(RwLock::new(None)),
            request_count: Arc::new(Mutex::new(0)),
        }
    }
    
    /// Add a provider to the client
    pub fn with_provider<P: Provider + 'static>(mut self, provider: P) -> Self {
        let provider_name = provider.name().to_string();
        
        // Register as data provider if it implements DataProvider
        if let Some(data_provider) = provider.as_data_provider() {
            self.data_providers.insert(provider_name.clone(), data_provider);
            if self.default_data_provider.is_none() {
                self.default_data_provider = Some(provider_name.clone());
            }
        }
        
        // Register as trading provider if it implements TradingProvider
        if let Some(trading_provider) = provider.as_trading_provider() {
            self.trading_providers.insert(provider_name.clone(), trading_provider);
            if self.default_trading_provider.is_none() {
                self.default_trading_provider = Some(provider_name);
            }
        }
        
        self
    }
    
    /// Set the default data provider
    pub fn with_default_data_provider(mut self, provider_name: &str) -> Self {
        if self.data_providers.contains_key(provider_name) {
            self.default_data_provider = Some(provider_name.to_string());
        }
        self
    }
    
    /// Set the default trading provider
    pub fn with_default_trading_provider(mut self, provider_name: &str) -> Self {
        if self.trading_providers.contains_key(provider_name) {
            self.default_trading_provider = Some(provider_name.to_string());
        }
        self
    }
    
    /// Build the client
    pub fn build(self) -> Self {
        self
    }
    
    /// Get a data provider by name
    pub fn data_provider(&self, name: &str) -> Option<Arc<dyn DataProvider>> {
        self.data_providers.get(name).cloned()
    }
    
    /// Get the default data provider
    pub fn default_data_provider(&self) -> Result<Arc<dyn DataProvider>> {
        match &self.default_data_provider {
            Some(name) => self.data_provider(name)
                .ok_or_else(|| Error::ProviderError(format!("Default data provider '{}' not found", name))),
            None => Err(Error::ProviderError("No default data provider set".to_string())),
        }
    }
    
    /// Get a trading provider by name
    pub fn trading_provider(&self, name: &str) -> Option<Arc<dyn TradingProvider>> {
        self.trading_providers.get(name).cloned()
    }
    
    /// Get the default trading provider
    pub fn default_trading_provider(&self) -> Result<Arc<dyn TradingProvider>> {
        match &self.default_trading_provider {
            Some(name) => self.trading_provider(name)
                .ok_or_else(|| Error::ProviderError(format!("Default trading provider '{}' not found", name))),
            None => Err(Error::ProviderError("No default trading provider set".to_string())),
        }
    }
    
    /// Get the client configuration
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }
    
    /// Create a historical price request builder
    pub fn historical_prices(&self, symbol: &str) -> HistoricalPriceBuilder {
        HistoricalPriceBuilder::new(self, symbol.to_string())
    }
    
    /// Get the latest quote for a symbol
    pub async fn quote(&self, symbol: &str) -> Result<Quote> {
        self.default_data_provider()?.quote(symbol).await
    }
    
    /// Get quotes for multiple symbols
    pub async fn quotes(&self, symbols: &[&str]) -> Result<HashMap<String, Quote>> {
        self.default_data_provider()?.quotes(symbols).await
    }
    
    /// Search for symbols by query
    pub async fn search(&self, query: &str, asset_class: Option<AssetClass>) -> Result<Vec<MarketData>> {
        self.default_data_provider()?.search(query, asset_class).await
    }
    
    /// Get an order book for a symbol
    pub async fn order_book(&self, symbol: &str, depth: Option<u32>) -> Result<OrderBook> {
        self.default_trading_provider()?.order_book(symbol, depth).await
    }
    
    /// Get recent trades for a symbol
    pub async fn recent_trades(&self, symbol: &str, limit: Option<u32>) -> Result<TradeHistory> {
        self.default_trading_provider()?.recent_trades(symbol, limit).await
    }
    
    /// Place a trading order
    pub async fn place_order(&self, order: Order) -> Result<OrderStatus> {
        self.default_trading_provider()?.place_order(order).await
    }
    
    /// Get the status of an order
    pub async fn order_status(&self, order_id: &str) -> Result<OrderStatus> {
        self.default_trading_provider()?.order_status(order_id).await
    }
    
    /// Cancel an order
    pub async fn cancel_order(&self, order_id: &str) -> Result<bool> {
        self.default_trading_provider()?.cancel_order(order_id).await
    }
    
    /// Get open positions
    pub async fn positions(&self) -> Result<Vec<Position>> {
        self.default_trading_provider()?.positions().await
    }
    
    /// Get portfolio data
    pub async fn portfolio(&self) -> Result<Option<Portfolio>> {
        Ok(self.portfolio.read().await.clone())
    }
    
    /// Set portfolio data
    pub async fn set_portfolio(&self, portfolio: Portfolio) -> Result<()> {
        *self.portfolio.write().await = Some(portfolio);
        Ok(())
    }
    
    /// Add a transaction to the portfolio
    pub async fn add_transaction(&self, transaction: Transaction) -> Result<()> {
        let mut portfolio_lock = self.portfolio.write().await;
        
        if let Some(portfolio) = &mut *portfolio_lock {
            portfolio.add_transaction(transaction);
            Ok(())
        } else {
            Err(Error::PortfolioError("No portfolio initialized".to_string()))
        }
    }
    
    /// Get statistics about API usage
    pub async fn stats(&self) -> ClientStats {
        ClientStats {
            request_count: *self.request_count.lock().await,
            data_providers: self.data_providers.keys().cloned().collect(),
            trading_providers: self.trading_providers.keys().cloned().collect(),
        }
    }
}

/// Builder for historical price requests
pub struct HistoricalPriceBuilder<'a> {
    /// Reference to the finance client
    client: &'a FinanceClient,
    
    /// Symbol to fetch prices for
    symbol: String,
    
    /// Time interval for price data
    interval: Option<TimeInterval>,
    
    /// Time range for price data
    range: Option<TimeRange>,
    
    /// Whether to include extended hours
    include_extended: bool,
    
    /// Whether to adjust for splits and dividends
    adjust: bool,
    
    /// Maximum number of data points to return
    limit: Option<u32>,
    
    /// Data provider to use
    provider: Option<String>,
}

impl<'a> HistoricalPriceBuilder<'a> {
    /// Create a new historical price builder
    fn new(client: &'a FinanceClient, symbol: String) -> Self {
        Self {
            client,
            symbol,
            interval: None,
            range: None,
            include_extended: false,
            adjust: true,
            limit: None,
            provider: None,
        }
    }
    
    /// Set the time interval
    pub fn interval(mut self, interval: TimeInterval) -> Self {
        self.interval = Some(interval);
        self
    }
    
    /// Set the time range
    pub fn range(mut self, range: TimeRange) -> Self {
        self.range = Some(range);
        self
    }
    
    /// Set whether to include extended hours
    pub fn include_extended(mut self, include_extended: bool) -> Self {
        self.include_extended = include_extended;
        self
    }
    
    /// Set whether to adjust for splits and dividends
    pub fn adjust(mut self, adjust: bool) -> Self {
        self.adjust = adjust;
        self
    }
    
    /// Set the maximum number of data points
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
    
    /// Set the data provider to use
    pub fn provider(mut self, provider: &str) -> Self {
        self.provider = Some(provider.to_string());
        self
    }
    
    /// Execute the request and fetch the historical price data
    pub async fn fetch(self) -> Result<TimeSeries<Price>> {
        // Determine which provider to use
        let provider = match self.provider {
            Some(ref name) => self.client.data_provider(name)
                .ok_or_else(|| Error::ProviderError(format!("Provider '{}' not found", name)))?,
            None => self.client.default_data_provider()?,
        };
        
        // Set defaults for interval and range if not specified
        let interval = self.interval.unwrap_or(TimeInterval::Daily);
        let range = self.range.unwrap_or(TimeRange::Month(1));
        
        // Fetch the historical prices
        provider.historical_prices(
            &self.symbol,
            interval,
            range,
            self.include_extended,
            self.adjust,
            self.limit,
        ).await
    }
}

/// Statistics about client usage
#[derive(Debug, Clone)]
pub struct ClientStats {
    /// Number of API requests made
    pub request_count: u64,
    
    /// List of registered data providers
    pub data_providers: Vec<String>,
    
    /// List of registered trading providers
    pub trading_providers: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{TimeInterval, TimeRange, Price, Quote, MarketData};
    use chrono::Utc;
    use std::sync::Arc;
    
    // Mock provider for testing
    struct MockProvider;
    
    impl Provider for MockProvider {
        fn name(&self) -> &str {
            "mock"
        }
        
        fn as_data_provider(&self) -> Option<Arc<dyn DataProvider>> {
            Some(Arc::new(MockProvider))
        }
        
        fn as_trading_provider(&self) -> Option<Arc<dyn TradingProvider>> {
            Some(Arc::new(MockProvider))
        }
    }
    
    #[async_trait]
    impl DataProvider for MockProvider {
        async fn quote(&self, _symbol: &str) -> Result<Quote> {
            Ok(Quote {
                symbol: "AAPL".to_string(),
                price: 150.0,
                change: 1.5,
                change_percent: 1.0,
                volume: 1000000,
                market_cap: Some(2500000000000.0),
                timestamp: Utc::now(),
                exchange: Some("NASDAQ".to_string()),
                currency: Some("USD".to_string()),
                additional_data: Default::default(),
            })
        }
        
        async fn quotes(&self, symbols: &[&str]) -> Result<HashMap<String, Quote>> {
            let mut result = HashMap::new();
            for symbol in symbols {
                result.insert(symbol.to_string(), self.quote(symbol).await?);
            }
            Ok(result)
        }
        
        async fn historical_prices(
            &self,
            _symbol: &str,
            _interval: TimeInterval,
            _range: TimeRange,
            _include_extended: bool,
            _adjust: bool,
            _limit: Option<u32>,
        ) -> Result<TimeSeries<Price>> {
            Ok(TimeSeries {
                symbol: "AAPL".to_string(),
                interval: TimeInterval::Daily,
                prices: vec![],
                start_time: Utc::now(),
                end_time: Utc::now(),
                timezone: "UTC".to_string(),
                currency: "USD".to_string(),
            })
        }
        
        async fn search(&self, _query: &str, _asset_class: Option<AssetClass>) -> Result<Vec<MarketData>> {
            Ok(vec![])
        }
    }
    
    #[async_trait]
    impl TradingProvider for MockProvider {
        async fn order_book(&self, _symbol: &str, _depth: Option<u32>) -> Result<OrderBook> {
            Ok(OrderBook {
                symbol: "AAPL".to_string(),
                timestamp: Utc::now(),
                bids: vec![],
                asks: vec![],
            })
        }
        
        async fn recent_trades(&self, _symbol: &str, _limit: Option<u32>) -> Result<TradeHistory> {
            Ok(TradeHistory {
                symbol: "AAPL".to_string(),
                trades: vec![],
            })
        }
        
        async fn place_order(&self, order: Order) -> Result<OrderStatus> {
            Ok(OrderStatus {
                order_id: "mockorder123".to_string(),
                symbol: order.symbol,
                status: "open".to_string(),
                order_type: order.order_type,
                side: order.side,
                quantity: order.quantity,
                price: order.price,
                timestamp: Utc::now(),
                filled_quantity: 0.0,
                remaining_quantity: order.quantity,
                average_price: None,
                trades: vec![],
            })
        }
        
        async fn order_status(&self, _order_id: &str) -> Result<OrderStatus> {
            Ok(OrderStatus {
                order_id: "mockorder123".to_string(),
                symbol: "AAPL".to_string(),
                status: "open".to_string(),
                order_type: "limit".to_string(),
                side: "buy".to_string(),
                quantity: 10.0,
                price: Some(150.0),
                timestamp: Utc::now(),
                filled_quantity: 0.0,
                remaining_quantity: 10.0,
                average_price: None,
                trades: vec![],
            })
        }
        
        async fn cancel_order(&self, _order_id: &str) -> Result<bool> {
            Ok(true)
        }
        
        async fn positions(&self) -> Result<Vec<Position>> {
            Ok(vec![])
        }
    }
    
    #[tokio::test]
    async fn test_client_with_provider() {
        let client = FinanceClient::new()
            .with_provider(MockProvider)
            .with_default_data_provider("mock")
            .with_default_trading_provider("mock")
            .build();
            
        assert!(client.data_provider("mock").is_some());
        assert!(client.trading_provider("mock").is_some());
        assert!(client.default_data_provider().is_ok());
        assert!(client.default_trading_provider().is_ok());
    }
} 