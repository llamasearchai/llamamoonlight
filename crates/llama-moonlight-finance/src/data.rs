use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use rust_decimal::Decimal;

/// A single price data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Price {
    /// Open price for the period
    pub open: f64,
    
    /// High price for the period
    pub high: f64,
    
    /// Low price for the period
    pub low: f64,
    
    /// Close price for the period
    pub close: f64,
    
    /// Trading volume for the period
    pub volume: u64,
    
    /// Timestamp for the price data
    pub timestamp: DateTime<Utc>,
    
    /// Additional data provided by the data source
    #[serde(flatten)]
    pub additional_data: HashMap<String, serde_json::Value>,
}

impl Price {
    /// Create a new price data point
    pub fn new(
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: u64,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            open,
            high,
            low,
            close,
            volume,
            timestamp,
            additional_data: HashMap::new(),
        }
    }
    
    /// Add additional data to the price point
    pub fn with_data<T: Serialize>(mut self, key: &str, value: T) -> Result<Self, serde_json::Error> {
        let json_value = serde_json::to_value(value)?;
        self.additional_data.insert(key.to_string(), json_value);
        Ok(self)
    }
    
    /// Calculate the typical price (average of high, low, and close)
    pub fn typical_price(&self) -> f64 {
        (self.high + self.low + self.close) / 3.0
    }
    
    /// Calculate the true range
    pub fn true_range(&self, previous: &Self) -> f64 {
        let high_low = self.high - self.low;
        let high_close = (self.high - previous.close).abs();
        let low_close = (self.low - previous.close).abs();
        
        high_low.max(high_close).max(low_close)
    }
}

/// A series of price data points with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeries<T> {
    /// Symbol identifier
    pub symbol: String,
    
    /// Time interval for the data
    pub interval: TimeInterval,
    
    /// Data points in the series
    pub prices: Vec<T>,
    
    /// Start time of the series
    pub start_time: DateTime<Utc>,
    
    /// End time of the series
    pub end_time: DateTime<Utc>,
    
    /// Timezone of the data
    pub timezone: String,
    
    /// Currency of the price data
    pub currency: String,
}

impl<T> TimeSeries<T> {
    /// Create a new time series
    pub fn new(
        symbol: String,
        interval: TimeInterval,
        prices: Vec<T>,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        timezone: String,
        currency: String,
    ) -> Self {
        Self {
            symbol,
            interval,
            prices,
            start_time,
            end_time,
            timezone,
            currency,
        }
    }
    
    /// Get the number of data points in the series
    pub fn len(&self) -> usize {
        self.prices.len()
    }
    
    /// Check if the series is empty
    pub fn is_empty(&self) -> bool {
        self.prices.is_empty()
    }
}

impl TimeSeries<Price> {
    /// Calculate basic statistics for the price series
    pub fn statistics(&self) -> PriceStatistics {
        if self.prices.is_empty() {
            return PriceStatistics::default();
        }
        
        let mut sum = 0.0;
        let mut max = f64::MIN;
        let mut min = f64::MAX;
        let mut volume_sum = 0;
        
        for price in &self.prices {
            sum += price.close;
            max = max.max(price.close);
            min = min.min(price.close);
            volume_sum += price.volume;
        }
        
        let mean = sum / self.prices.len() as f64;
        
        // Calculate variance and standard deviation
        let mut variance_sum = 0.0;
        for price in &self.prices {
            let diff = price.close - mean;
            variance_sum += diff * diff;
        }
        
        let variance = variance_sum / self.prices.len() as f64;
        let std_dev = variance.sqrt();
        
        // Calculate volatility (annualized standard deviation)
        let annualization_factor = match self.interval {
            TimeInterval::Minute1 => (252.0 * 6.5 * 60.0).sqrt(),
            TimeInterval::Minute5 => (252.0 * 6.5 * 12.0).sqrt(),
            TimeInterval::Minute15 => (252.0 * 6.5 * 4.0).sqrt(),
            TimeInterval::Minute30 => (252.0 * 6.5 * 2.0).sqrt(),
            TimeInterval::Hourly => (252.0 * 6.5).sqrt(),
            TimeInterval::Daily => 252.0_f64.sqrt(),
            TimeInterval::Weekly => 52.0_f64.sqrt(),
            TimeInterval::Monthly => 12.0_f64.sqrt(),
            TimeInterval::Quarterly => 4.0_f64.sqrt(),
            TimeInterval::Yearly => 1.0,
            TimeInterval::Custom(_) => 1.0,
        };
        
        let volatility = (std_dev / mean) * annualization_factor;
        
        // Calculate returns
        let mut returns = Vec::with_capacity(self.prices.len() - 1);
        for i in 1..self.prices.len() {
            let prev_close = self.prices[i - 1].close;
            let curr_close = self.prices[i].close;
            returns.push((curr_close - prev_close) / prev_close);
        }
        
        // Calculate mean return
        let return_sum: f64 = returns.iter().sum();
        let return_mean = if !returns.is_empty() {
            return_sum / returns.len() as f64
        } else {
            0.0
        };
        
        PriceStatistics {
            mean,
            median: self.median_price(),
            min,
            max,
            range: max - min,
            std_dev,
            variance,
            volatility,
            return_mean,
            total_volume: volume_sum,
            count: self.prices.len(),
        }
    }
    
    /// Calculate the median price in the series
    pub fn median_price(&self) -> f64 {
        if self.prices.is_empty() {
            return 0.0;
        }
        
        let mut prices: Vec<f64> = self.prices.iter().map(|p| p.close).collect();
        prices.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        
        let mid = prices.len() / 2;
        if prices.len() % 2 == 0 {
            (prices[mid - 1] + prices[mid]) / 2.0
        } else {
            prices[mid]
        }
    }
    
    /// Calculate simple moving average for a given period
    pub fn sma(&self, period: usize) -> Vec<f64> {
        if period > self.prices.len() || period == 0 {
            return vec![];
        }
        
        let mut result = Vec::with_capacity(self.prices.len() - period + 1);
        let mut sum = 0.0;
        
        // Calculate initial sum
        for i in 0..period {
            sum += self.prices[i].close;
        }
        
        // Add first SMA
        result.push(sum / period as f64);
        
        // Calculate remaining SMAs using sliding window
        for i in period..self.prices.len() {
            sum = sum - self.prices[i - period].close + self.prices[i].close;
            result.push(sum / period as f64);
        }
        
        result
    }
    
    /// Calculate exponential moving average for a given period
    pub fn ema(&self, period: usize) -> Vec<f64> {
        if period > self.prices.len() || period == 0 {
            return vec![];
        }
        
        let mut result = Vec::with_capacity(self.prices.len() - period + 1);
        
        // Calculate SMA for initial EMA value
        let mut sum = 0.0;
        for i in 0..period {
            sum += self.prices[i].close;
        }
        let sma = sum / period as f64;
        
        // Add first EMA (which is the SMA)
        result.push(sma);
        
        // Calculate remaining EMAs
        let multiplier = 2.0 / (period as f64 + 1.0);
        let mut prev_ema = sma;
        
        for i in period..self.prices.len() {
            let close = self.prices[i].close;
            let ema = (close - prev_ema) * multiplier + prev_ema;
            result.push(ema);
            prev_ema = ema;
        }
        
        result
    }
}

/// Realtime market quote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    /// Symbol identifier
    pub symbol: String,
    
    /// Current price
    pub price: f64,
    
    /// Price change
    pub change: f64,
    
    /// Percentage change
    pub change_percent: f64,
    
    /// Trading volume
    pub volume: u64,
    
    /// Market capitalization
    pub market_cap: Option<f64>,
    
    /// Quote timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Exchange
    pub exchange: Option<String>,
    
    /// Currency
    pub currency: Option<String>,
    
    /// Additional data fields
    #[serde(flatten)]
    pub additional_data: HashMap<String, serde_json::Value>,
}

/// General market data about a financial instrument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    /// Symbol identifier
    pub symbol: String,
    
    /// Full name
    pub name: String,
    
    /// Type of instrument
    pub instrument_type: String,
    
    /// Trading exchange
    pub exchange: String,
    
    /// Currency
    pub currency: String,
    
    /// Country
    pub country: Option<String>,
    
    /// Index membership (e.g., S&P 500, NASDAQ 100)
    pub index_membership: Option<Vec<String>>,
    
    /// Sector
    pub sector: Option<String>,
    
    /// Industry
    pub industry: Option<String>,
    
    /// Market capitalization category
    pub market_cap_category: Option<String>,
    
    /// Is ETF
    pub is_etf: bool,
    
    /// Additional data fields
    #[serde(flatten)]
    pub additional_data: HashMap<String, serde_json::Value>,
}

/// Price statistics for a time series
#[derive(Debug, Clone)]
pub struct PriceStatistics {
    /// Mean (average) price
    pub mean: f64,
    
    /// Median price
    pub median: f64,
    
    /// Minimum price
    pub min: f64,
    
    /// Maximum price
    pub max: f64,
    
    /// Price range (max - min)
    pub range: f64,
    
    /// Standard deviation of prices
    pub std_dev: f64,
    
    /// Variance of prices
    pub variance: f64,
    
    /// Volatility (annualized)
    pub volatility: f64,
    
    /// Mean return
    pub return_mean: f64,
    
    /// Total trading volume
    pub total_volume: u64,
    
    /// Number of data points
    pub count: usize,
}

impl Default for PriceStatistics {
    fn default() -> Self {
        Self {
            mean: 0.0,
            median: 0.0,
            min: 0.0,
            max: 0.0,
            range: 0.0,
            std_dev: 0.0,
            variance: 0.0,
            volatility: 0.0,
            return_mean: 0.0,
            total_volume: 0,
            count: 0,
        }
    }
}

/// Time interval for price data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeInterval {
    /// 1-minute intervals
    Minute1,
    
    /// 5-minute intervals
    Minute5,
    
    /// 15-minute intervals
    Minute15,
    
    /// 30-minute intervals
    Minute30,
    
    /// 1-hour intervals
    Hourly,
    
    /// Daily intervals
    Daily,
    
    /// Weekly intervals
    Weekly,
    
    /// Monthly intervals
    Monthly,
    
    /// Quarterly intervals
    Quarterly,
    
    /// Yearly intervals
    Yearly,
    
    /// Custom interval in minutes
    Custom(u32),
}

impl TimeInterval {
    /// Convert the interval to a duration
    pub fn to_duration(&self) -> Duration {
        match self {
            TimeInterval::Minute1 => Duration::minutes(1),
            TimeInterval::Minute5 => Duration::minutes(5),
            TimeInterval::Minute15 => Duration::minutes(15),
            TimeInterval::Minute30 => Duration::minutes(30),
            TimeInterval::Hourly => Duration::hours(1),
            TimeInterval::Daily => Duration::days(1),
            TimeInterval::Weekly => Duration::weeks(1),
            TimeInterval::Monthly => Duration::days(30), // Approximate
            TimeInterval::Quarterly => Duration::days(90), // Approximate
            TimeInterval::Yearly => Duration::days(365), // Approximate
            TimeInterval::Custom(minutes) => Duration::minutes(*minutes as i64),
        }
    }
    
    /// Convert the interval to a string representation
    pub fn to_string_representation(&self) -> String {
        match self {
            TimeInterval::Minute1 => "1m".to_string(),
            TimeInterval::Minute5 => "5m".to_string(),
            TimeInterval::Minute15 => "15m".to_string(),
            TimeInterval::Minute30 => "30m".to_string(),
            TimeInterval::Hourly => "1h".to_string(),
            TimeInterval::Daily => "1d".to_string(),
            TimeInterval::Weekly => "1wk".to_string(),
            TimeInterval::Monthly => "1mo".to_string(),
            TimeInterval::Quarterly => "3mo".to_string(),
            TimeInterval::Yearly => "1y".to_string(),
            TimeInterval::Custom(minutes) => format!("{}m", minutes),
        }
    }
}

/// Time range for data queries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeRange {
    /// Range in days
    Day(u32),
    
    /// Range in weeks
    Week(u32),
    
    /// Range in months
    Month(u32),
    
    /// Range in years
    Year(u32),
    
    /// Custom range with start and end timestamps
    Custom(DateTime<Utc>, DateTime<Utc>),
    
    /// Maximum available data
    Max,
}

impl TimeRange {
    /// Convert the range to a pair of DateTime values
    pub fn to_date_range(&self) -> (DateTime<Utc>, DateTime<Utc>) {
        let now = Utc::now();
        
        match self {
            TimeRange::Day(days) => {
                let start = now - Duration::days(*days as i64);
                (start, now)
            },
            TimeRange::Week(weeks) => {
                let start = now - Duration::weeks(*weeks as i64);
                (start, now)
            },
            TimeRange::Month(months) => {
                let start = now - Duration::days(30 * *months as i64);
                (start, now)
            },
            TimeRange::Year(years) => {
                let start = now - Duration::days(365 * *years as i64);
                (start, now)
            },
            TimeRange::Custom(start, end) => (*start, *end),
            TimeRange::Max => {
                // A date far in the past for maximum range
                let start = DateTime::<Utc>::from_timestamp(0, 0).unwrap_or(now - Duration::days(365 * 20));
                (start, now)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    
    #[test]
    fn test_price_creation() {
        let timestamp = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap();
        let price = Price::new(100.0, 105.0, 98.0, 103.0, 1000, timestamp);
        
        assert_eq!(price.open, 100.0);
        assert_eq!(price.high, 105.0);
        assert_eq!(price.low, 98.0);
        assert_eq!(price.close, 103.0);
        assert_eq!(price.volume, 1000);
        assert_eq!(price.timestamp, timestamp);
    }
    
    #[test]
    fn test_typical_price() {
        let timestamp = Utc::now();
        let price = Price::new(100.0, 105.0, 95.0, 103.0, 1000, timestamp);
        
        let typical = price.typical_price();
        assert_eq!(typical, (105.0 + 95.0 + 103.0) / 3.0);
    }
    
    #[test]
    fn test_true_range() {
        let timestamp = Utc::now();
        let prev_price = Price::new(100.0, 105.0, 95.0, 101.0, 1000, timestamp);
        let curr_price = Price::new(102.0, 108.0, 98.0, 106.0, 1000, timestamp);
        
        let tr = curr_price.true_range(&prev_price);
        let high_low = curr_price.high - curr_price.low; // 108 - 98 = 10
        let high_close = (curr_price.high - prev_price.close).abs(); // |108 - 101| = 7
        let low_close = (curr_price.low - prev_price.close).abs(); // |98 - 101| = 3
        
        assert_eq!(tr, high_low.max(high_close).max(low_close));
        assert_eq!(tr, 10.0);
    }
    
    #[test]
    fn test_time_series_creation() {
        let timestamp = Utc::now();
        let prices = vec![
            Price::new(100.0, 105.0, 95.0, 101.0, 1000, timestamp),
            Price::new(102.0, 108.0, 98.0, 106.0, 1200, timestamp),
        ];
        
        let ts = TimeSeries::new(
            "AAPL".to_string(),
            TimeInterval::Daily,
            prices.clone(),
            timestamp,
            timestamp,
            "UTC".to_string(),
            "USD".to_string(),
        );
        
        assert_eq!(ts.symbol, "AAPL");
        assert_eq!(ts.interval, TimeInterval::Daily);
        assert_eq!(ts.prices.len(), 2);
        assert_eq!(ts.timezone, "UTC");
        assert_eq!(ts.currency, "USD");
    }
    
    #[test]
    fn test_sma_calculation() {
        let timestamp = Utc::now();
        let prices = vec![
            Price::new(100.0, 105.0, 95.0, 100.0, 1000, timestamp),
            Price::new(101.0, 106.0, 96.0, 101.0, 1100, timestamp),
            Price::new(102.0, 107.0, 97.0, 102.0, 1200, timestamp),
            Price::new(103.0, 108.0, 98.0, 103.0, 1300, timestamp),
            Price::new(104.0, 109.0, 99.0, 104.0, 1400, timestamp),
        ];
        
        let ts = TimeSeries::new(
            "AAPL".to_string(),
            TimeInterval::Daily,
            prices,
            timestamp,
            timestamp,
            "UTC".to_string(),
            "USD".to_string(),
        );
        
        // Test 3-period SMA
        let sma3 = ts.sma(3);
        assert_eq!(sma3.len(), 3);
        assert_eq!(sma3[0], (100.0 + 101.0 + 102.0) / 3.0);
        assert_eq!(sma3[1], (101.0 + 102.0 + 103.0) / 3.0);
        assert_eq!(sma3[2], (102.0 + 103.0 + 104.0) / 3.0);
    }
    
    #[test]
    fn test_time_interval_to_duration() {
        assert_eq!(TimeInterval::Minute1.to_duration(), Duration::minutes(1));
        assert_eq!(TimeInterval::Hourly.to_duration(), Duration::hours(1));
        assert_eq!(TimeInterval::Daily.to_duration(), Duration::days(1));
        assert_eq!(TimeInterval::Custom(45).to_duration(), Duration::minutes(45));
    }
    
    #[test]
    fn test_time_range_to_date_range() {
        let now = Utc::now();
        
        let (start, end) = TimeRange::Day(7).to_date_range();
        assert!(end - start >= Duration::days(6));
        assert!(end - start <= Duration::days(8)); // Allow for small timing differences
        
        let custom_start = now - Duration::days(10);
        let custom_end = now - Duration::days(5);
        let (start, end) = TimeRange::Custom(custom_start, custom_end).to_date_range();
        assert_eq!(start, custom_start);
        assert_eq!(end, custom_end);
    }
} 