use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;

use crate::{Result, AssetClass};
use crate::data::{TimeInterval, TimeRange, TimeSeries, Price, Quote, MarketData};
use crate::market::{OrderBook, TradeHistory};
use crate::trading::{Order, OrderStatus, Position, TradeExecution};

/// Base trait for all providers
pub trait Provider {
    /// Get the provider name
    fn name(&self) -> &str;
    
    /// Get the provider type
    fn provider_type(&self) -> ProviderType {
        ProviderType::Other
    }
    
    /// Get the provider capabilities
    fn capabilities(&self) -> Vec<Capability> {
        vec![]
    }
    
    /// Check if the provider supports a capability
    fn supports(&self, capability: Capability) -> bool {
        self.capabilities().contains(&capability)
    }
    
    /// Convert to a data provider if supported
    fn as_data_provider(&self) -> Option<Arc<dyn DataProvider>> {
        None
    }
    
    /// Convert to a trading provider if supported
    fn as_trading_provider(&self) -> Option<Arc<dyn TradingProvider>> {
        None
    }
}

/// Provider type categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderType {
    /// Standard market data provider
    MarketData,
    
    /// Cryptocurrency exchange
    Crypto,
    
    /// Stock exchange or broker
    Stock,
    
    /// News and analysis provider
    News,
    
    /// Fundamental data provider
    Fundamental,
    
    /// Technical analysis provider
    Technical,
    
    /// Alternative data provider
    Alternative,
    
    /// Other provider type
    Other,
}

/// Provider capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    /// Real-time quotes
    RealTimeQuotes,
    
    /// Historical price data
    HistoricalPrices,
    
    /// Tick-by-tick data
    TickData,
    
    /// Market depth (order book) data
    MarketDepth,
    
    /// Fundamental data (financials, earnings, etc.)
    FundamentalData,
    
    /// Analyst estimates and ratings
    AnalystData,
    
    /// News and events
    NewsAndEvents,
    
    /// Option chains
    OptionChains,
    
    /// Futures chains
    FuturesChains,
    
    /// Trading execution
    Trading,
    
    /// Portfolio management
    Portfolio,
    
    /// Screener capabilities
    Screening,
    
    /// Technical indicators
    TechnicalIndicators,
    
    /// Economic data
    EconomicData,
    
    /// Alternative data
    AlternativeData,
    
    /// Real-time streaming
    Streaming,
    
    /// Paper trading
    PaperTrading,
}

/// Trait for market data providers
#[async_trait]
pub trait DataProvider: Provider + Send + Sync {
    /// Get a quote for a single symbol
    async fn quote(&self, symbol: &str) -> Result<Quote>;
    
    /// Get quotes for multiple symbols
    async fn quotes(&self, symbols: &[&str]) -> Result<HashMap<String, Quote>> {
        let mut result = HashMap::new();
        for symbol in symbols {
            let quote = self.quote(symbol).await?;
            result.insert(symbol.to_string(), quote);
        }
        Ok(result)
    }
    
    /// Get historical price data
    async fn historical_prices(
        &self,
        symbol: &str,
        interval: TimeInterval,
        range: TimeRange,
        include_extended: bool,
        adjust: bool,
        limit: Option<u32>,
    ) -> Result<TimeSeries<Price>>;
    
    /// Search for symbols
    async fn search(&self, query: &str, asset_class: Option<AssetClass>) -> Result<Vec<MarketData>>;
    
    /// Get market data for a symbol
    async fn market_data(&self, symbol: &str) -> Result<MarketData> {
        // Default implementation: search for the exact symbol and return the first result
        let results = self.search(symbol, None).await?;
        results.into_iter().find(|data| data.symbol == symbol)
            .ok_or_else(|| crate::Error::MarketDataError(format!("Symbol not found: {}", symbol)))
    }
    
    /// Get market data for multiple symbols
    async fn market_data_batch(&self, symbols: &[&str]) -> Result<HashMap<String, MarketData>> {
        let mut result = HashMap::new();
        for symbol in symbols {
            match self.market_data(symbol).await {
                Ok(data) => { result.insert(symbol.to_string(), data); },
                Err(_) => continue, // Skip symbols that can't be found
            }
        }
        Ok(result)
    }
}

/// Trait for trading providers
#[async_trait]
pub trait TradingProvider: Provider + Send + Sync {
    /// Get the order book (market depth)
    async fn order_book(&self, symbol: &str, depth: Option<u32>) -> Result<OrderBook>;
    
    /// Get recent trades
    async fn recent_trades(&self, symbol: &str, limit: Option<u32>) -> Result<TradeHistory>;
    
    /// Place an order
    async fn place_order(&self, order: Order) -> Result<OrderStatus>;
    
    /// Get order status
    async fn order_status(&self, order_id: &str) -> Result<OrderStatus>;
    
    /// Cancel an order
    async fn cancel_order(&self, order_id: &str) -> Result<bool>;
    
    /// Get current positions
    async fn positions(&self) -> Result<Vec<Position>>;
    
    /// Get account balance
    async fn account_balance(&self) -> Result<HashMap<String, f64>> {
        Err(crate::Error::ProviderError("Account balance not supported by this provider".to_string()))
    }
    
    /// Get recent executions
    async fn executions(&self, limit: Option<u32>) -> Result<Vec<TradeExecution>> {
        Err(crate::Error::ProviderError("Executions not supported by this provider".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    struct TestProvider;
    
    impl Provider for TestProvider {
        fn name(&self) -> &str {
            "test"
        }
        
        fn provider_type(&self) -> ProviderType {
            ProviderType::MarketData
        }
        
        fn capabilities(&self) -> Vec<Capability> {
            vec![
                Capability::RealTimeQuotes,
                Capability::HistoricalPrices,
                Capability::MarketDepth,
            ]
        }
    }
    
    #[test]
    fn test_provider_capabilities() {
        let provider = TestProvider;
        
        assert_eq!(provider.name(), "test");
        assert_eq!(provider.provider_type(), ProviderType::MarketData);
        assert!(provider.capabilities().contains(&Capability::RealTimeQuotes));
        assert!(provider.capabilities().contains(&Capability::HistoricalPrices));
        assert!(provider.capabilities().contains(&Capability::MarketDepth));
        assert!(!provider.capabilities().contains(&Capability::Trading));
        
        assert!(provider.supports(Capability::RealTimeQuotes));
        assert!(!provider.supports(Capability::Trading));
    }
} 