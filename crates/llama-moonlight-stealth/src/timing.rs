//! Timing-based stealth operations
//!
//! This module provides capabilities for timing-based stealth operations,
//! such as realistic delays, throttling, and timing attack prevention.

use std::time::{Duration, Instant};
use rand::{Rng, thread_rng};

use crate::Result;
use crate::Error;

/// Human-like delay configuration
#[derive(Debug, Clone)]
pub struct DelayConfig {
    /// Minimum delay time in milliseconds
    pub min_delay_ms: u64,
    
    /// Maximum delay time in milliseconds
    pub max_delay_ms: u64,
    
    /// Whether to use a normal distribution for delays
    pub use_normal_distribution: bool,
    
    /// Mean delay time for normal distribution (milliseconds)
    pub mean_delay_ms: u64,
    
    /// Standard deviation for normal distribution (milliseconds)
    pub std_dev_ms: u64,
}

impl DelayConfig {
    /// Create a new delay configuration with uniform distribution
    pub fn uniform(min_delay_ms: u64, max_delay_ms: u64) -> Self {
        Self {
            min_delay_ms,
            max_delay_ms,
            use_normal_distribution: false,
            mean_delay_ms: 0,
            std_dev_ms: 0,
        }
    }
    
    /// Create a new delay configuration with normal distribution
    pub fn normal(mean_delay_ms: u64, std_dev_ms: u64, min_delay_ms: u64, max_delay_ms: u64) -> Self {
        Self {
            min_delay_ms,
            max_delay_ms,
            use_normal_distribution: true,
            mean_delay_ms,
            std_dev_ms,
        }
    }
    
    /// Get a random delay using the configuration
    pub fn random_delay(&self) -> Duration {
        let mut rng = thread_rng();
        
        let delay_ms = if self.use_normal_distribution {
            // Normal distribution
            let normal = rand_distr::Normal::new(self.mean_delay_ms as f64, self.std_dev_ms as f64)
                .unwrap_or(rand_distr::Normal::new(500.0, 100.0).unwrap());
            
            let delay = rng.sample(normal) as u64;
            
            // Clamp to min/max
            delay.clamp(self.min_delay_ms, self.max_delay_ms)
        } else {
            // Uniform distribution
            rng.gen_range(self.min_delay_ms..=self.max_delay_ms)
        };
        
        Duration::from_millis(delay_ms)
    }
}

impl Default for DelayConfig {
    fn default() -> Self {
        Self {
            min_delay_ms: 100,
            max_delay_ms: 1000,
            use_normal_distribution: false,
            mean_delay_ms: 500,
            std_dev_ms: 200,
        }
    }
}

/// Manager for timing operations
#[derive(Debug)]
pub struct TimingManager {
    /// Configuration for delays
    delay_config: DelayConfig,
    
    /// Last action time
    last_action_time: Option<Instant>,
}

impl TimingManager {
    /// Create a new timing manager
    pub fn new() -> Self {
        Self {
            delay_config: DelayConfig::default(),
            last_action_time: None,
        }
    }
    
    /// Create a timing manager with a specific delay configuration
    pub fn with_config(delay_config: DelayConfig) -> Self {
        Self {
            delay_config,
            last_action_time: None,
        }
    }
    
    /// Record an action
    pub fn record_action(&mut self) {
        self.last_action_time = Some(Instant::now());
    }
    
    /// Get the time since the last action
    pub fn time_since_last_action(&self) -> Option<Duration> {
        self.last_action_time.map(|t| t.elapsed())
    }
    
    /// Get a human-like delay
    pub fn human_delay(&self) -> Duration {
        self.delay_config.random_delay()
    }
    
    /// Get a typing delay based on text length
    pub fn typing_delay(&self, text_length: usize) -> Duration {
        // Assume an average typing speed of 5 characters per second
        let base_typing_ms = (text_length as u64 * 200) / 5;
        
        // Add some randomness
        let variation = base_typing_ms / 5; // 20% variation
        let min = base_typing_ms.saturating_sub(variation);
        let max = base_typing_ms + variation;
        
        let config = DelayConfig::uniform(min, max);
        config.random_delay()
    }
    
    /// Wait for a human-like delay
    pub async fn wait_human_like(&self) {
        let delay = self.human_delay();
        tokio::time::sleep(delay).await;
    }
    
    /// Calculate a natural pause between actions
    pub fn natural_pause(&self, action_type: ActionType) -> Duration {
        let base_ms = match action_type {
            ActionType::Navigation => 2000,
            ActionType::Click => 300,
            ActionType::Type => 100,
            ActionType::Scroll => 500,
            ActionType::Read => 3000,
            ActionType::Custom(ms) => ms,
        };
        
        // Add 30% randomness
        let variation = base_ms / 3;
        let config = DelayConfig::uniform(base_ms - variation, base_ms + variation);
        config.random_delay()
    }
}

impl Default for TimingManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Type of action for timing calculations
#[derive(Debug, Clone, Copy)]
pub enum ActionType {
    /// Navigation to a new page
    Navigation,
    
    /// Click on an element
    Click,
    
    /// Type text
    Type,
    
    /// Scroll the page
    Scroll,
    
    /// Read content on the page
    Read,
    
    /// Custom action with base milliseconds
    Custom(u64),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_delay_config() {
        let config = DelayConfig::uniform(100, 200);
        
        // Test random delay is within range
        for _ in 0..100 {
            let delay = config.random_delay();
            let ms = delay.as_millis() as u64;
            assert!(ms >= 100 && ms <= 200);
        }
        
        // Test normal distribution
        let config = DelayConfig::normal(500, 100, 300, 700);
        
        // Test random delay is within range
        for _ in 0..100 {
            let delay = config.random_delay();
            let ms = delay.as_millis() as u64;
            assert!(ms >= 300 && ms <= 700);
        }
    }
    
    #[test]
    fn test_timing_manager() {
        let mut manager = TimingManager::new();
        
        // Test record action
        assert!(manager.time_since_last_action().is_none());
        manager.record_action();
        assert!(manager.time_since_last_action().is_some());
        
        // Test human delay
        let delay = manager.human_delay();
        let ms = delay.as_millis() as u64;
        assert!(ms >= 100 && ms <= 1000);
        
        // Test typing delay
        let delay = manager.typing_delay(10);
        let ms = delay.as_millis() as u64;
        assert!(ms > 0);
        
        // Test natural pause
        let delay = manager.natural_pause(ActionType::Click);
        let ms = delay.as_millis() as u64;
        assert!(ms >= 200 && ms <= 400);
    }
} 