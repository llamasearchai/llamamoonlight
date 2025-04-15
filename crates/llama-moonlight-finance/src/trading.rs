use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Represents a trading order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    /// Symbol to trade
    pub symbol: String,
    
    /// Order side (buy or sell)
    pub side: String,
    
    /// Order quantity
    pub quantity: f64,
    
    /// Order price (for limit/stop orders)
    pub price: Option<f64>,
    
    /// Stop price (for stop/stop-limit orders)
    pub stop_price: Option<f64>,
    
    /// Order type (market, limit, etc.)
    pub order_type: String,
    
    /// Time in force (day, gtc, etc.)
    pub time_in_force: Option<String>,
    
    /// Whether the order is an all-or-none order
    pub all_or_none: bool,
    
    /// Whether the order is a reduce-only order
    pub reduce_only: bool,
    
    /// Client order ID for tracking
    pub client_order_id: Option<String>,
    
    /// Additional order parameters
    #[serde(flatten)]
    pub additional_params: HashMap<String, serde_json::Value>,
}

impl Order {
    /// Create a new order
    pub fn new(symbol: &str, side: &str, quantity: f64) -> Self {
        Self {
            symbol: symbol.to_string(),
            side: side.to_lowercase(),
            quantity,
            price: None,
            stop_price: None,
            order_type: "market".to_string(),
            time_in_force: None,
            all_or_none: false,
            reduce_only: false,
            client_order_id: None,
            additional_params: HashMap::new(),
        }
    }
    
    /// Set the order type
    pub fn order_type<T: Into<String>>(mut self, order_type: T) -> Self {
        self.order_type = order_type.into().to_lowercase();
        self
    }
    
    /// Set the order price
    pub fn price(mut self, price: f64) -> Self {
        self.price = Some(price);
        self
    }
    
    /// Set the stop price
    pub fn stop_price(mut self, stop_price: f64) -> Self {
        self.stop_price = Some(stop_price);
        self
    }
    
    /// Set the time in force
    pub fn time_in_force<T: Into<String>>(mut self, time_in_force: T) -> Self {
        self.time_in_force = Some(time_in_force.into().to_lowercase());
        self
    }
    
    /// Set the all-or-none flag
    pub fn all_or_none(mut self, all_or_none: bool) -> Self {
        self.all_or_none = all_or_none;
        self
    }
    
    /// Set the reduce-only flag
    pub fn reduce_only(mut self, reduce_only: bool) -> Self {
        self.reduce_only = reduce_only;
        self
    }
    
    /// Set the client order ID
    pub fn client_order_id<T: Into<String>>(mut self, client_order_id: T) -> Self {
        self.client_order_id = Some(client_order_id.into());
        self
    }
    
    /// Add an additional parameter
    pub fn with_param<T: Serialize>(mut self, key: &str, value: T) -> Result<Self, serde_json::Error> {
        let json_value = serde_json::to_value(value)?;
        self.additional_params.insert(key.to_string(), json_value);
        Ok(self)
    }
    
    /// Verify that the order is valid
    pub fn validate(&self) -> Result<(), String> {
        if self.quantity <= 0.0 {
            return Err("Order quantity must be positive".to_string());
        }
        
        if !["buy", "sell"].contains(&self.side.as_str()) {
            return Err(format!("Invalid order side: {}", self.side));
        }
        
        match self.order_type.as_str() {
            "market" => {
                // Market orders don't require a price
            },
            "limit" => {
                if self.price.is_none() {
                    return Err("Limit orders require a price".to_string());
                }
            },
            "stop" => {
                if self.stop_price.is_none() {
                    return Err("Stop orders require a stop price".to_string());
                }
            },
            "stop_limit" => {
                if self.price.is_none() {
                    return Err("Stop-limit orders require a price".to_string());
                }
                if self.stop_price.is_none() {
                    return Err("Stop-limit orders require a stop price".to_string());
                }
            },
            _ => {
                return Err(format!("Unknown order type: {}", self.order_type));
            }
        }
        
        Ok(())
    }
}

/// Common order types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OrderType {
    /// Market order (execute immediately at market price)
    Market,
    
    /// Limit order (execute at specified price or better)
    Limit,
    
    /// Stop order (convert to market order when price reaches stop price)
    Stop,
    
    /// Stop-limit order (convert to limit order when price reaches stop price)
    StopLimit,
    
    /// Trailing stop order (stop price follows market at specified distance)
    TrailingStop,
    
    /// Fill-or-kill order (execute entirely immediately or cancel)
    FillOrKill,
    
    /// Immediate-or-cancel order (execute what is available immediately, cancel rest)
    ImmediateOrCancel,
}

impl ToString for OrderType {
    fn to_string(&self) -> String {
        match self {
            OrderType::Market => "market".to_string(),
            OrderType::Limit => "limit".to_string(),
            OrderType::Stop => "stop".to_string(),
            OrderType::StopLimit => "stop_limit".to_string(),
            OrderType::TrailingStop => "trailing_stop".to_string(),
            OrderType::FillOrKill => "fill_or_kill".to_string(),
            OrderType::ImmediateOrCancel => "immediate_or_cancel".to_string(),
        }
    }
}

/// Represents an order status from a trading provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderStatus {
    /// Order ID from the provider
    pub order_id: String,
    
    /// Symbol being traded
    pub symbol: String,
    
    /// Current status of the order (open, filled, cancelled, etc.)
    pub status: String,
    
    /// Order type
    pub order_type: String,
    
    /// Order side (buy or sell)
    pub side: String,
    
    /// Order quantity
    pub quantity: f64,
    
    /// Order price (for limit/stop orders)
    pub price: Option<f64>,
    
    /// Order timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Quantity that has been filled
    pub filled_quantity: f64,
    
    /// Quantity that remains to be filled
    pub remaining_quantity: f64,
    
    /// Average fill price
    pub average_price: Option<f64>,
    
    /// Individual trade executions for this order
    pub trades: Vec<TradeExecution>,
}

impl OrderStatus {
    /// Create a new order status
    pub fn new(
        order_id: &str,
        symbol: &str,
        status: &str,
        order_type: &str,
        side: &str,
        quantity: f64,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            order_id: order_id.to_string(),
            symbol: symbol.to_string(),
            status: status.to_string(),
            order_type: order_type.to_string(),
            side: side.to_string(),
            quantity,
            price: None,
            timestamp,
            filled_quantity: 0.0,
            remaining_quantity: quantity,
            average_price: None,
            trades: Vec::new(),
        }
    }
    
    /// Check if the order is open
    pub fn is_open(&self) -> bool {
        matches!(self.status.as_str(), "open" | "new" | "partially_filled")
    }
    
    /// Check if the order is filled
    pub fn is_filled(&self) -> bool {
        self.status == "filled" || self.filled_quantity == self.quantity
    }
    
    /// Check if the order is cancelled
    pub fn is_cancelled(&self) -> bool {
        self.status == "cancelled" || self.status == "canceled"
    }
    
    /// Check if the order is rejected
    pub fn is_rejected(&self) -> bool {
        self.status == "rejected"
    }
    
    /// Get the fill percentage
    pub fn fill_percentage(&self) -> f64 {
        if self.quantity == 0.0 {
            return 0.0;
        }
        (self.filled_quantity / self.quantity) * 100.0
    }
    
    /// Update the status with a new trade execution
    pub fn add_execution(&mut self, execution: TradeExecution) {
        // Update filled quantity
        self.filled_quantity += execution.quantity;
        self.remaining_quantity = (self.quantity - self.filled_quantity).max(0.0);
        
        // Update average price
        let total_value = match self.average_price {
            Some(avg_price) => avg_price * (self.filled_quantity - execution.quantity),
            None => 0.0,
        } + execution.price * execution.quantity;
        
        self.average_price = Some(total_value / self.filled_quantity);
        
        // Update status if fully filled
        if self.filled_quantity >= self.quantity {
            self.status = "filled".to_string();
        } else if self.filled_quantity > 0.0 {
            self.status = "partially_filled".to_string();
        }
        
        // Add the execution to the list
        self.trades.push(execution);
    }
}

/// Represents a single trade execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeExecution {
    /// Execution ID
    pub execution_id: String,
    
    /// Order ID this execution belongs to
    pub order_id: String,
    
    /// Symbol traded
    pub symbol: String,
    
    /// Trade side (buy or sell)
    pub side: String,
    
    /// Trade quantity
    pub quantity: f64,
    
    /// Execution price
    pub price: f64,
    
    /// Execution timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Commission/fee paid
    pub commission: Option<f64>,
    
    /// Commission/fee currency
    pub commission_currency: Option<String>,
    
    /// Additional execution data
    #[serde(flatten)]
    pub additional_data: HashMap<String, serde_json::Value>,
}

impl TradeExecution {
    /// Create a new trade execution
    pub fn new(
        execution_id: &str,
        order_id: &str,
        symbol: &str,
        side: &str,
        quantity: f64,
        price: f64,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            execution_id: execution_id.to_string(),
            order_id: order_id.to_string(),
            symbol: symbol.to_string(),
            side: side.to_string(),
            quantity,
            price,
            timestamp,
            commission: None,
            commission_currency: None,
            additional_data: HashMap::new(),
        }
    }
    
    /// Set the commission details
    pub fn with_commission<T: Into<String>>(mut self, commission: f64, currency: T) -> Self {
        self.commission = Some(commission);
        self.commission_currency = Some(currency.into());
        self
    }
    
    /// Add additional data
    pub fn with_data<T: Serialize>(mut self, key: &str, value: T) -> Result<Self, serde_json::Error> {
        let json_value = serde_json::to_value(value)?;
        self.additional_data.insert(key.to_string(), json_value);
        Ok(self)
    }
    
    /// Get the trade value (price * quantity)
    pub fn value(&self) -> f64 {
        self.price * self.quantity
    }
}

/// Represents a trading position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// Symbol of the instrument
    pub symbol: String,
    
    /// Position quantity (positive for long, negative for short)
    pub quantity: f64,
    
    /// Average entry price
    pub average_price: f64,
    
    /// Current market price
    pub market_price: Option<f64>,
    
    /// Unrealized profit/loss
    pub unrealized_pnl: Option<f64>,
    
    /// Realized profit/loss
    pub realized_pnl: Option<f64>,
    
    /// Initial margin requirement
    pub initial_margin: Option<f64>,
    
    /// Maintenance margin requirement
    pub maintenance_margin: Option<f64>,
    
    /// Position open timestamp
    pub open_timestamp: Option<DateTime<Utc>>,
    
    /// Position update timestamp
    pub update_timestamp: Option<DateTime<Utc>>,
    
    /// Additional position data
    #[serde(flatten)]
    pub additional_data: HashMap<String, serde_json::Value>,
}

impl Position {
    /// Create a new position
    pub fn new(symbol: &str, quantity: f64, average_price: f64) -> Self {
        Self {
            symbol: symbol.to_string(),
            quantity,
            average_price,
            market_price: None,
            unrealized_pnl: None,
            realized_pnl: None,
            initial_margin: None,
            maintenance_margin: None,
            open_timestamp: None,
            update_timestamp: None,
            additional_data: HashMap::new(),
        }
    }
    
    /// Set the market price and calculate unrealized P&L
    pub fn with_market_price(mut self, market_price: f64) -> Self {
        self.market_price = Some(market_price);
        self.unrealized_pnl = Some(self.calculate_unrealized_pnl(market_price));
        self
    }
    
    /// Set the realized P&L
    pub fn with_realized_pnl(mut self, realized_pnl: f64) -> Self {
        self.realized_pnl = Some(realized_pnl);
        self
    }
    
    /// Set the margin requirements
    pub fn with_margin(mut self, initial_margin: f64, maintenance_margin: f64) -> Self {
        self.initial_margin = Some(initial_margin);
        self.maintenance_margin = Some(maintenance_margin);
        self
    }
    
    /// Set the timestamps
    pub fn with_timestamps(mut self, open_timestamp: DateTime<Utc>, update_timestamp: DateTime<Utc>) -> Self {
        self.open_timestamp = Some(open_timestamp);
        self.update_timestamp = Some(update_timestamp);
        self
    }
    
    /// Add additional data
    pub fn with_data<T: Serialize>(mut self, key: &str, value: T) -> Result<Self, serde_json::Error> {
        let json_value = serde_json::to_value(value)?;
        self.additional_data.insert(key.to_string(), json_value);
        Ok(self)
    }
    
    /// Calculate the position value
    pub fn value(&self) -> f64 {
        self.quantity.abs() * self.average_price
    }
    
    /// Calculate the current market value
    pub fn market_value(&self) -> Option<f64> {
        self.market_price.map(|price| self.quantity.abs() * price)
    }
    
    /// Calculate unrealized P&L
    pub fn calculate_unrealized_pnl(&self, market_price: f64) -> f64 {
        let direction = if self.quantity > 0.0 { 1.0 } else { -1.0 };
        direction * self.quantity.abs() * (market_price - self.average_price)
    }
    
    /// Check if this is a long position
    pub fn is_long(&self) -> bool {
        self.quantity > 0.0
    }
    
    /// Check if this is a short position
    pub fn is_short(&self) -> bool {
        self.quantity < 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_order_creation() {
        let order = Order::new("AAPL", "buy", 10.0)
            .order_type("limit")
            .price(150.0)
            .time_in_force("day")
            .client_order_id("test-order-1");
            
        assert_eq!(order.symbol, "AAPL");
        assert_eq!(order.side, "buy");
        assert_eq!(order.quantity, 10.0);
        assert_eq!(order.order_type, "limit");
        assert_eq!(order.price, Some(150.0));
        assert_eq!(order.time_in_force, Some("day".to_string()));
        assert_eq!(order.client_order_id, Some("test-order-1".to_string()));
    }
    
    #[test]
    fn test_order_validation() {
        // Valid market order
        let market_order = Order::new("AAPL", "buy", 10.0);
        assert!(market_order.validate().is_ok());
        
        // Valid limit order
        let limit_order = Order::new("AAPL", "sell", 10.0)
            .order_type("limit")
            .price(150.0);
        assert!(limit_order.validate().is_ok());
        
        // Invalid limit order (missing price)
        let invalid_limit = Order::new("AAPL", "buy", 10.0)
            .order_type("limit");
        assert!(invalid_limit.validate().is_err());
        
        // Invalid order (negative quantity)
        let invalid_qty = Order::new("AAPL", "buy", -5.0);
        assert!(invalid_qty.validate().is_err());
        
        // Invalid order (unknown side)
        let invalid_side = Order::new("AAPL", "hold", 10.0);
        assert!(invalid_side.validate().is_err());
    }
    
    #[test]
    fn test_order_status() {
        let mut status = OrderStatus::new(
            "order123",
            "AAPL",
            "open",
            "limit",
            "buy",
            10.0,
            Utc::now(),
        );
        
        assert!(status.is_open());
        assert!(!status.is_filled());
        assert_eq!(status.fill_percentage(), 0.0);
        
        // Add a partial execution
        let execution1 = TradeExecution::new(
            "exec1",
            "order123",
            "AAPL",
            "buy",
            4.0,
            149.5,
            Utc::now(),
        );
        
        status.add_execution(execution1);
        
        assert!(status.is_open());
        assert!(!status.is_filled());
        assert_eq!(status.filled_quantity, 4.0);
        assert_eq!(status.remaining_quantity, 6.0);
        assert_eq!(status.average_price, Some(149.5));
        assert_eq!(status.fill_percentage(), 40.0);
        assert_eq!(status.status, "partially_filled");
        
        // Add another execution to complete the order
        let execution2 = TradeExecution::new(
            "exec2",
            "order123",
            "AAPL",
            "buy",
            6.0,
            150.0,
            Utc::now(),
        );
        
        status.add_execution(execution2);
        
        assert!(!status.is_open());
        assert!(status.is_filled());
        assert_eq!(status.filled_quantity, 10.0);
        assert_eq!(status.remaining_quantity, 0.0);
        assert_eq!(status.fill_percentage(), 100.0);
        assert_eq!(status.status, "filled");
        
        // Check average price calculation
        let expected_avg = (4.0 * 149.5 + 6.0 * 150.0) / 10.0;
        assert!((status.average_price.unwrap() - expected_avg).abs() < 0.0001);
    }
    
    #[test]
    fn test_position() {
        let position = Position::new("AAPL", 100.0, 150.0)
            .with_market_price(155.0)
            .with_realized_pnl(500.0);
            
        assert_eq!(position.symbol, "AAPL");
        assert_eq!(position.quantity, 100.0);
        assert_eq!(position.average_price, 150.0);
        assert_eq!(position.market_price, Some(155.0));
        assert_eq!(position.unrealized_pnl, Some(500.0)); // (155 - 150) * 100
        assert_eq!(position.realized_pnl, Some(500.0));
        
        assert!(position.is_long());
        assert!(!position.is_short());
        
        assert_eq!(position.value(), 15000.0); // 100 * 150
        assert_eq!(position.market_value(), Some(15500.0)); // 100 * 155
        
        // Test a short position
        let short = Position::new("TSLA", -10.0, 200.0)
            .with_market_price(190.0);
            
        assert!(short.is_short());
        assert!(!short.is_long());
        assert_eq!(short.unrealized_pnl, Some(100.0)); // -10 * (190 - 200) = 100
    }
} 