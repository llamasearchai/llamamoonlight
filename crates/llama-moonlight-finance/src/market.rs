use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Order book entry (bid or ask)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookEntry {
    /// Price level
    pub price: f64,
    
    /// Quantity available at this price level
    pub quantity: f64,
    
    /// Number of orders at this price level (if available)
    pub order_count: Option<u32>,
    
    /// Additional data about this price level
    #[serde(flatten)]
    pub additional_data: HashMap<String, serde_json::Value>,
}

/// Market order book (bids and asks)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    /// Market symbol
    pub symbol: String,
    
    /// Timestamp when the order book was captured
    pub timestamp: DateTime<Utc>,
    
    /// Bid price levels (sorted from highest to lowest)
    pub bids: Vec<OrderBookEntry>,
    
    /// Ask price levels (sorted from lowest to highest)
    pub asks: Vec<OrderBookEntry>,
}

impl OrderBook {
    /// Create a new order book
    pub fn new(
        symbol: String,
        timestamp: DateTime<Utc>,
        bids: Vec<OrderBookEntry>,
        asks: Vec<OrderBookEntry>,
    ) -> Self {
        Self {
            symbol,
            timestamp,
            bids,
            asks,
        }
    }
    
    /// Get the best bid (highest buy price)
    pub fn best_bid(&self) -> Option<&OrderBookEntry> {
        self.bids.first()
    }
    
    /// Get the best ask (lowest sell price)
    pub fn best_ask(&self) -> Option<&OrderBookEntry> {
        self.asks.first()
    }
    
    /// Get the spread (difference between best ask and best bid)
    pub fn spread(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(ask.price - bid.price),
            _ => None,
        }
    }
    
    /// Get the spread as a percentage of the mid price
    pub fn spread_percentage(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => {
                let mid = (bid.price + ask.price) / 2.0;
                let spread = ask.price - bid.price;
                Some((spread / mid) * 100.0)
            },
            _ => None,
        }
    }
    
    /// Get the mid price (average of best bid and best ask)
    pub fn mid_price(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some((bid.price + ask.price) / 2.0),
            _ => None,
        }
    }
    
    /// Calculate the volume at a specific price level
    pub fn volume_at_price(&self, price: f64, tolerance: f64) -> f64 {
        // Calculate volume from bids
        let bid_volume = self.bids.iter()
            .filter(|entry| (entry.price - price).abs() <= tolerance)
            .map(|entry| entry.quantity)
            .sum::<f64>();
            
        // Calculate volume from asks
        let ask_volume = self.asks.iter()
            .filter(|entry| (entry.price - price).abs() <= tolerance)
            .map(|entry| entry.quantity)
            .sum::<f64>();
            
        bid_volume + ask_volume
    }
    
    /// Calculate the cumulative volume up to a certain price level
    pub fn cumulative_bid_volume(&self, price: f64) -> f64 {
        self.bids.iter()
            .filter(|entry| entry.price >= price)
            .map(|entry| entry.quantity)
            .sum()
    }
    
    /// Calculate the cumulative ask volume down to a certain price level
    pub fn cumulative_ask_volume(&self, price: f64) -> f64 {
        self.asks.iter()
            .filter(|entry| entry.price <= price)
            .map(|entry| entry.quantity)
            .sum()
    }
    
    /// Calculate the total volume on the bid side
    pub fn total_bid_volume(&self) -> f64 {
        self.bids.iter().map(|entry| entry.quantity).sum()
    }
    
    /// Calculate the total volume on the ask side
    pub fn total_ask_volume(&self) -> f64 {
        self.asks.iter().map(|entry| entry.quantity).sum()
    }
    
    /// Calculate the total value (price * quantity) on the bid side
    pub fn total_bid_value(&self) -> f64 {
        self.bids.iter().map(|entry| entry.price * entry.quantity).sum()
    }
    
    /// Calculate the total value (price * quantity) on the ask side
    pub fn total_ask_value(&self) -> f64 {
        self.asks.iter().map(|entry| entry.price * entry.quantity).sum()
    }
    
    /// Calculate the market pressure (bid volume / ask volume)
    pub fn market_pressure(&self) -> Option<f64> {
        let ask_volume = self.total_ask_volume();
        if ask_volume == 0.0 {
            None
        } else {
            Some(self.total_bid_volume() / ask_volume)
        }
    }
}

/// A single trade record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    /// Unique trade ID (if available)
    pub id: Option<String>,
    
    /// Trading symbol
    pub symbol: String,
    
    /// Trade price
    pub price: f64,
    
    /// Trade quantity
    pub quantity: f64,
    
    /// Trade side (Buy or Sell)
    pub side: Option<String>,
    
    /// Trade timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Trade maker/taker flag (if available)
    pub is_maker: Option<bool>,
    
    /// Trade value (price * quantity)
    pub value: Option<f64>,
    
    /// Trading fee (if available)
    pub fee: Option<f64>,
    
    /// Fee currency (if available)
    pub fee_currency: Option<String>,
    
    /// Additional trade data
    #[serde(flatten)]
    pub additional_data: HashMap<String, serde_json::Value>,
}

impl Trade {
    /// Create a new trade record
    pub fn new(
        symbol: String,
        price: f64,
        quantity: f64,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            id: None,
            symbol,
            price,
            quantity,
            side: None,
            timestamp,
            is_maker: None,
            value: Some(price * quantity),
            fee: None,
            fee_currency: None,
            additional_data: HashMap::new(),
        }
    }
    
    /// Set the trade ID
    pub fn with_id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }
    
    /// Set the trade side
    pub fn with_side(mut self, side: String) -> Self {
        self.side = Some(side);
        self
    }
    
    /// Set the maker/taker flag
    pub fn with_maker(mut self, is_maker: bool) -> Self {
        self.is_maker = Some(is_maker);
        self
    }
    
    /// Set the fee information
    pub fn with_fee(mut self, fee: f64, currency: String) -> Self {
        self.fee = Some(fee);
        self.fee_currency = Some(currency);
        self
    }
    
    /// Get the trade value (price * quantity)
    pub fn get_value(&self) -> f64 {
        self.value.unwrap_or(self.price * self.quantity)
    }
}

/// Collection of trades
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeHistory {
    /// Market symbol
    pub symbol: String,
    
    /// List of trades
    pub trades: Vec<Trade>,
}

impl TradeHistory {
    /// Create a new trade history
    pub fn new(symbol: String, trades: Vec<Trade>) -> Self {
        Self {
            symbol,
            trades,
        }
    }
    
    /// Get the number of trades
    pub fn len(&self) -> usize {
        self.trades.len()
    }
    
    /// Check if the trade history is empty
    pub fn is_empty(&self) -> bool {
        self.trades.is_empty()
    }
    
    /// Get the most recent trade
    pub fn latest_trade(&self) -> Option<&Trade> {
        self.trades.first()
    }
    
    /// Get the oldest trade in the history
    pub fn oldest_trade(&self) -> Option<&Trade> {
        self.trades.last()
    }
    
    /// Calculate the total volume in the trade history
    pub fn total_volume(&self) -> f64 {
        self.trades.iter().map(|trade| trade.quantity).sum()
    }
    
    /// Calculate the total value in the trade history
    pub fn total_value(&self) -> f64 {
        self.trades.iter().map(|trade| trade.get_value()).sum()
    }
    
    /// Calculate the volume-weighted average price (VWAP)
    pub fn vwap(&self) -> Option<f64> {
        if self.trades.is_empty() {
            return None;
        }
        
        let total_value = self.trades.iter()
            .map(|trade| trade.price * trade.quantity)
            .sum::<f64>();
            
        let total_volume = self.trades.iter()
            .map(|trade| trade.quantity)
            .sum::<f64>();
            
        if total_volume == 0.0 {
            None
        } else {
            Some(total_value / total_volume)
        }
    }
    
    /// Calculate the average price
    pub fn average_price(&self) -> Option<f64> {
        if self.trades.is_empty() {
            return None;
        }
        
        let sum = self.trades.iter().map(|trade| trade.price).sum::<f64>();
        Some(sum / self.trades.len() as f64)
    }
    
    /// Calculate the highest trade price
    pub fn high_price(&self) -> Option<f64> {
        self.trades.iter().map(|trade| trade.price).fold(None, |max, price| {
            match max {
                None => Some(price),
                Some(max_price) => Some(max_price.max(price)),
            }
        })
    }
    
    /// Calculate the lowest trade price
    pub fn low_price(&self) -> Option<f64> {
        self.trades.iter().map(|trade| trade.price).fold(None, |min, price| {
            match min {
                None => Some(price),
                Some(min_price) => Some(min_price.min(price)),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    
    #[test]
    fn test_order_book_creation() {
        let now = Utc::now();
        let bids = vec![
            OrderBookEntry {
                price: 100.0,
                quantity: 10.0,
                order_count: Some(5),
                additional_data: HashMap::new(),
            },
            OrderBookEntry {
                price: 99.0,
                quantity: 20.0,
                order_count: Some(8),
                additional_data: HashMap::new(),
            },
        ];
        
        let asks = vec![
            OrderBookEntry {
                price: 101.0,
                quantity: 15.0,
                order_count: Some(7),
                additional_data: HashMap::new(),
            },
            OrderBookEntry {
                price: 102.0,
                quantity: 25.0,
                order_count: Some(10),
                additional_data: HashMap::new(),
            },
        ];
        
        let order_book = OrderBook::new("AAPL".to_string(), now, bids, asks);
        
        assert_eq!(order_book.symbol, "AAPL");
        assert_eq!(order_book.timestamp, now);
        assert_eq!(order_book.bids.len(), 2);
        assert_eq!(order_book.asks.len(), 2);
    }
    
    #[test]
    fn test_order_book_metrics() {
        let now = Utc::now();
        let bids = vec![
            OrderBookEntry {
                price: 100.0,
                quantity: 10.0,
                order_count: Some(5),
                additional_data: HashMap::new(),
            },
            OrderBookEntry {
                price: 99.0,
                quantity: 20.0,
                order_count: Some(8),
                additional_data: HashMap::new(),
            },
        ];
        
        let asks = vec![
            OrderBookEntry {
                price: 101.0,
                quantity: 15.0,
                order_count: Some(7),
                additional_data: HashMap::new(),
            },
            OrderBookEntry {
                price: 102.0,
                quantity: 25.0,
                order_count: Some(10),
                additional_data: HashMap::new(),
            },
        ];
        
        let order_book = OrderBook::new("AAPL".to_string(), now, bids, asks);
        
        // Test metrics
        assert_eq!(order_book.best_bid().unwrap().price, 100.0);
        assert_eq!(order_book.best_ask().unwrap().price, 101.0);
        assert_eq!(order_book.spread().unwrap(), 1.0);
        assert_eq!(order_book.mid_price().unwrap(), 100.5);
        assert_eq!(order_book.total_bid_volume(), 30.0);
        assert_eq!(order_book.total_ask_volume(), 40.0);
        assert_eq!(order_book.total_bid_value(), 100.0 * 10.0 + 99.0 * 20.0);
        assert_eq!(order_book.total_ask_value(), 101.0 * 15.0 + 102.0 * 25.0);
        assert_eq!(order_book.market_pressure().unwrap(), 30.0 / 40.0);
    }
    
    #[test]
    fn test_trade_creation() {
        let now = Utc::now();
        let trade = Trade::new(
            "AAPL".to_string(),
            150.0,
            10.0,
            now,
        )
        .with_id("trade123".to_string())
        .with_side("buy".to_string())
        .with_maker(true)
        .with_fee(0.1, "USD".to_string());
        
        assert_eq!(trade.symbol, "AAPL");
        assert_eq!(trade.price, 150.0);
        assert_eq!(trade.quantity, 10.0);
        assert_eq!(trade.timestamp, now);
        assert_eq!(trade.id.unwrap(), "trade123");
        assert_eq!(trade.side.unwrap(), "buy");
        assert!(trade.is_maker.unwrap());
        assert_eq!(trade.fee.unwrap(), 0.1);
        assert_eq!(trade.fee_currency.unwrap(), "USD");
        assert_eq!(trade.get_value(), 1500.0);
    }
    
    #[test]
    fn test_trade_history_metrics() {
        let now = Utc::now();
        let trades = vec![
            Trade::new("AAPL".to_string(), 150.0, 10.0, now),
            Trade::new("AAPL".to_string(), 151.0, 5.0, now),
            Trade::new("AAPL".to_string(), 149.0, 15.0, now),
        ];
        
        let history = TradeHistory::new("AAPL".to_string(), trades);
        
        assert_eq!(history.len(), 3);
        assert_eq!(history.total_volume(), 30.0);
        assert_eq!(history.total_value(), 150.0 * 10.0 + 151.0 * 5.0 + 149.0 * 15.0);
        assert_eq!(history.average_price().unwrap(), (150.0 + 151.0 + 149.0) / 3.0);
        assert_eq!(history.high_price().unwrap(), 151.0);
        assert_eq!(history.low_price().unwrap(), 149.0);
        
        // Test VWAP calculation
        let vwap = history.vwap().unwrap();
        let expected_vwap = (150.0 * 10.0 + 151.0 * 5.0 + 149.0 * 15.0) / 30.0;
        assert!((vwap - expected_vwap).abs() < 0.0001);
    }
} 