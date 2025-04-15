use anyhow::{anyhow, Result};
use async_semaphore::Semaphore;
use dashmap::DashMap;
use futures::{future, StreamExt};
use llama_moonlight_core::{
    options::{BrowserOptions, ContextOptions},
    Browser, BrowserType, Moonlight,
};
use log::{debug, error, info, warn};
use metrics::{counter, gauge};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use thiserror::Error;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Errors specific to the browser pool
#[derive(Error, Debug)]
pub enum PoolError {
    /// Error when trying to get a browser from an empty pool
    #[error("No browsers available in pool")]
    NoBrowsersAvailable,

    /// Error when pool size limit is reached
    #[error("Pool size limit reached")]
    PoolSizeLimitReached,

    /// Error when trying to use an invalid browser ID
    #[error("Invalid browser ID: {0}")]
    InvalidBrowserId(String),

    /// Error when trying to use a browser that is not in the idle state
    #[error("Browser is not idle")]
    BrowserNotIdle,

    /// Error from the core library
    #[error("Core error: {0}")]
    CoreError(#[from] llama_moonlight_core::Error),

    /// Other errors
    #[error("Pool error: {0}")]
    Other(String),
}

/// Status of a browser in the pool
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserStatus {
    /// Browser is idle and ready to be used
    Idle,
    /// Browser is in use by a client
    InUse,
    /// Browser is being initialized
    Initializing,
    /// Browser is being cleaned up
    CleaningUp,
    /// Browser has failed and should be recreated
    Failed,
}

/// Information about a browser in the pool
#[derive(Debug)]
struct BrowserInfo {
    /// Unique ID for this browser instance
    id: String,
    /// The actual browser instance
    browser: Arc<Browser>,
    /// Current status of the browser
    status: BrowserStatus,
    /// Timestamp when the browser was created
    created_at: Instant,
    /// Timestamp when the browser was last used
    last_used: Instant,
    /// Number of times this browser has been used
    use_count: u32,
    /// Browser type name
    browser_type: String,
}

/// Configuration for a browser pool
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Minimum number of browsers to keep in the pool
    pub min_size: usize,
    /// Maximum number of browsers in the pool
    pub max_size: usize,
    /// Maximum number of uses before recycling a browser
    pub max_uses: u32,
    /// Maximum idle time before recycling a browser (in seconds)
    pub max_idle_time_secs: u64,
    /// Browser type to use (chromium, firefox, webkit)
    pub browser_type: String,
    /// Browser launch options
    pub browser_options: BrowserOptions,
    /// Default context options for browsers in this pool
    pub context_options: ContextOptions,
    /// Enable browser reuse (if false, browser is closed after each use)
    pub enable_reuse: bool,
    /// Time to wait between browser creation attempts (in milliseconds)
    pub creation_retry_delay_ms: u64,
    /// Maximum number of creation retries
    pub max_creation_retries: u32,
    /// Enable metrics collection
    pub enable_metrics: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_size: 1,
            max_size: 10,
            max_uses: 100,
            max_idle_time_secs: 300, // 5 minutes
            browser_type: "chromium".to_string(),
            browser_options: BrowserOptions {
                headless: Some(true),
                stealth: Some(true),
                ..BrowserOptions::default()
            },
            context_options: ContextOptions::default(),
            enable_reuse: true,
            creation_retry_delay_ms: 1000,
            max_creation_retries: 3,
            enable_metrics: true,
        }
    }
}

/// A handle to a browser from the pool
pub struct PooledBrowser {
    /// Browser instance
    browser: Arc<Browser>,
    /// Browser ID in the pool
    id: String,
    /// Pool that owns this browser
    pool: Arc<BrowserPool>,
}

impl PooledBrowser {
    /// Create a new browser context
    pub async fn new_context(&self) -> Result<Arc<llama_moonlight_core::BrowserContext>> {
        let context = self.browser.new_context().await?;
        Ok(Arc::new(context))
    }

    /// Create a new browser context with custom options
    pub async fn new_context_with_options(
        &self,
        options: ContextOptions,
    ) -> Result<Arc<llama_moonlight_core::BrowserContext>> {
        let context = self.browser.new_context_with_options(options).await?;
        Ok(Arc::new(context))
    }

    /// Get the browser instance
    pub fn browser(&self) -> Arc<Browser> {
        self.browser.clone()
    }

    /// Get the browser ID
    pub fn id(&self) -> &str {
        &self.id
    }
}

impl Drop for PooledBrowser {
    fn drop(&mut self) {
        let id = self.id.clone();
        let pool = self.pool.clone();

        // Schedule the browser to be returned to the pool
        tokio::spawn(async move {
            if let Err(e) = pool.return_browser(&id).await {
                error!("Failed to return browser to pool: {}", e);
            }
        });
    }
}

/// A pool of browser instances for parallel automation
pub struct BrowserPool {
    /// Map of browser ID to browser info
    browsers: DashMap<String, BrowserInfo>,
    /// Semaphore to limit concurrent browser creations
    creation_semaphore: Semaphore,
    /// Moonlight instance
    moonlight: Arc<Mutex<Moonlight>>,
    /// Pool configuration
    config: PoolConfig,
    /// Maintenance task handle
    maintenance_task: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl BrowserPool {
    /// Create a new browser pool with default configuration
    pub async fn new() -> Result<Arc<Self>> {
        Self::with_config(PoolConfig::default()).await
    }

    /// Create a new browser pool with custom configuration
    pub async fn with_config(config: PoolConfig) -> Result<Arc<Self>> {
        // Initialize Moonlight
        let moonlight = Moonlight::new().await?;

        let pool = Arc::new(Self {
            browsers: DashMap::new(),
            creation_semaphore: Semaphore::new(config.max_size),
            moonlight: Arc::new(Mutex::new(moonlight)),
            config,
            maintenance_task: Mutex::new(None),
        });

        // Start maintenance task
        pool.start_maintenance_task();

        // Initialize the pool with minimum browsers
        pool.initialize().await?;

        Ok(pool)
    }

    /// Initialize the pool with minimum browsers
    async fn initialize(&self) -> Result<()> {
        info!("Initializing browser pool with {} browsers", self.config.min_size);

        let mut futures = Vec::with_capacity(self.config.min_size);

        for _ in 0..self.config.min_size {
            futures.push(self.create_browser());
        }

        // Wait for all browsers to be created
        let results = future::join_all(futures).await;

        // Count failures
        let failures = results.iter().filter(|r| r.is_err()).count();
        if failures > 0 {
            warn!("{} browsers failed to initialize", failures);
        }

        if self.config.enable_metrics {
            gauge!("browser_pool.size", self.browsers.len() as f64);
            gauge!("browser_pool.available", self.available_count() as f64);
        }

        Ok(())
    }

    /// Get the number of browsers in the pool
    pub fn size(&self) -> usize {
        self.browsers.len()
    }

    /// Get the number of available browsers
    pub fn available_count(&self) -> usize {
        self.browsers
            .iter()
            .filter(|pair| pair.value().status == BrowserStatus::Idle)
            .count()
    }

    /// Get the number of browsers in use
    pub fn in_use_count(&self) -> usize {
        self.browsers
            .iter()
            .filter(|pair| pair.value().status == BrowserStatus::InUse)
            .count()
    }

    /// Get a browser from the pool
    pub async fn get_browser(&self) -> Result<PooledBrowser, PoolError> {
        // Try to find an idle browser
        let mut browser_id = None;

        for pair in self.browsers.iter() {
            let info = pair.value();
            if info.status == BrowserStatus::Idle {
                browser_id = Some(info.id.clone());
                break;
            }
        }

        // If we found an idle browser, try to claim it
        if let Some(id) = browser_id {
            return self.claim_browser(&id).await;
        }

        // No idle browsers available, try to create a new one if we're below max_size
        if self.browsers.len() < self.config.max_size {
            debug!("No idle browsers available, creating a new one");
            let browser_id = match self.create_browser().await {
                Ok(id) => id,
                Err(e) => {
                    error!("Failed to create new browser: {}", e);
                    return Err(PoolError::Other(format!("Failed to create new browser: {}", e)));
                }
            };
            return self.claim_browser(&browser_id).await;
        }

        // We're at max capacity and no browsers are available
        Err(PoolError::NoBrowsersAvailable)
    }

    /// Return a browser to the pool
    async fn return_browser(&self, browser_id: &str) -> Result<(), PoolError> {
        let mut entry = match self.browsers.get_mut(browser_id) {
            Some(entry) => entry,
            None => {
                return Err(PoolError::InvalidBrowserId(browser_id.to_string()));
            }
        };

        // Update browser info
        let browser_info = entry.value_mut();
        browser_info.status = BrowserStatus::Idle;
        browser_info.last_used = Instant::now();
        browser_info.use_count += 1;

        debug!(
            "Browser {} returned to pool (use count: {})",
            browser_id, browser_info.use_count
        );

        // Check if we should recycle this browser
        if browser_info.use_count >= self.config.max_uses {
            debug!(
                "Browser {} reached max uses ({}), scheduling recycling",
                browser_id, browser_info.use_count
            );
            
            // Schedule browser for recycling
            let pool = Arc::new(self.clone());
            let browser_id = browser_id.to_string();
            tokio::spawn(async move {
                if let Err(e) = pool.recycle_browser(&browser_id).await {
                    error!("Failed to recycle browser {}: {}", browser_id, e);
                }
            });
        }

        if self.config.enable_metrics {
            gauge!("browser_pool.available", self.available_count() as f64);
            counter!("browser_pool.returns", 1);
        }

        Ok(())
    }

    /// Claim a browser from the pool
    async fn claim_browser(&self, browser_id: &str) -> Result<PooledBrowser, PoolError> {
        let mut entry = match self.browsers.get_mut(browser_id) {
            Some(entry) => entry,
            None => {
                return Err(PoolError::InvalidBrowserId(browser_id.to_string()));
            }
        };

        let browser_info = entry.value_mut();
        
        // Check if the browser is idle
        if browser_info.status != BrowserStatus::Idle {
            return Err(PoolError::BrowserNotIdle);
        }

        // Mark as in use
        browser_info.status = BrowserStatus::InUse;
        browser_info.last_used = Instant::now();

        debug!("Browser {} claimed from pool", browser_id);

        if self.config.enable_metrics {
            gauge!("browser_pool.available", self.available_count() as f64);
            counter!("browser_pool.claims", 1);
        }

        Ok(PooledBrowser {
            browser: browser_info.browser.clone(),
            id: browser_id.to_string(),
            pool: Arc::new(self.clone()),
        })
    }

    /// Create a new browser
    async fn create_browser(&self) -> Result<String> {
        // Acquire a permit from the semaphore
        let _permit = self.creation_semaphore.acquire().await;

        let browser_id = Uuid::new_v4().to_string();
        debug!("Creating new browser with ID: {}", browser_id);

        // Mark as initializing
        self.browsers.insert(
            browser_id.clone(),
            BrowserInfo {
                id: browser_id.clone(),
                browser: Arc::new(Browser::default()), // Placeholder
                status: BrowserStatus::Initializing,
                created_at: Instant::now(),
                last_used: Instant::now(),
                use_count: 0,
                browser_type: self.config.browser_type.clone(),
            },
        );

        let mut moonlight = self.moonlight.lock().await;
        let browser_type = match moonlight.browser_type(&self.config.browser_type) {
            Some(bt) => bt,
            None => {
                self.browsers.remove(&browser_id);
                return Err(anyhow!("Browser type '{}' not found", self.config.browser_type));
            }
        };

        // Launch browser with retries
        let mut browser = None;
        let mut last_error = None;

        for attempt in 1..=self.config.max_creation_retries {
            match browser_type.launch_with_options(self.config.browser_options.clone()).await {
                Ok(b) => {
                    browser = Some(b);
                    break;
                }
                Err(e) => {
                    warn!(
                        "Failed to create browser (attempt {}/{}): {}",
                        attempt, self.config.max_creation_retries, e
                    );
                    last_error = Some(e);
                    
                    // Wait before retrying
                    tokio::time::sleep(Duration::from_millis(self.config.creation_retry_delay_ms)).await;
                }
            }
        }

        // Check if browser creation succeeded
        let browser = match browser {
            Some(b) => b,
            None => {
                self.browsers.remove(&browser_id);
                return Err(anyhow!(
                    "Failed to create browser after {} attempts: {}",
                    self.config.max_creation_retries,
                    last_error.unwrap_or_else(|| anyhow!("Unknown error"))
                ));
            }
        };

        // Update browser info with actual browser
        if let Some(mut entry) = self.browsers.get_mut(&browser_id) {
            let browser_info = entry.value_mut();
            browser_info.browser = Arc::new(browser);
            browser_info.status = BrowserStatus::Idle;
        } else {
            // This shouldn't happen, but just in case
            return Err(anyhow!("Browser ID {} not found in pool", browser_id));
        }

        info!("Browser {} created successfully", browser_id);

        if self.config.enable_metrics {
            gauge!("browser_pool.size", self.browsers.len() as f64);
            gauge!("browser_pool.available", self.available_count() as f64);
            counter!("browser_pool.creations", 1);
        }

        Ok(browser_id)
    }

    /// Recycle a browser (close and create a new one)
    async fn recycle_browser(&self, browser_id: &str) -> Result<()> {
        debug!("Recycling browser {}", browser_id);

        // Mark as cleaning up
        if let Some(mut entry) = self.browsers.get_mut(browser_id) {
            entry.value_mut().status = BrowserStatus::CleaningUp;
        } else {
            return Err(anyhow!("Browser ID {} not found in pool", browser_id));
        }

        // Get the browser to close
        let browser_to_close = if let Some(entry) = self.browsers.get(browser_id) {
            entry.value().browser.clone()
        } else {
            return Err(anyhow!("Browser ID {} not found in pool", browser_id));
        };

        // Close the browser
        if let Err(e) = browser_to_close.close().await {
            warn!("Error closing browser {}: {}", browser_id, e);
        }

        // Remove from pool
        self.browsers.remove(browser_id);

        // Create a new browser if we're below min_size
        if self.browsers.len() < self.config.min_size {
            match self.create_browser().await {
                Ok(id) => debug!("Created replacement browser {}", id),
                Err(e) => warn!("Failed to create replacement browser: {}", e),
            }
        }

        if self.config.enable_metrics {
            gauge!("browser_pool.size", self.browsers.len() as f64);
            gauge!("browser_pool.available", self.available_count() as f64);
            counter!("browser_pool.recycled", 1);
        }

        Ok(())
    }

    /// Start the maintenance task
    fn start_maintenance_task(&self) {
        let pool = Arc::new(self.clone());
        
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                if let Err(e) = pool.perform_maintenance().await {
                    error!("Error during pool maintenance: {}", e);
                }
            }
        });

        let mut maintenance_task = self.maintenance_task.try_lock().unwrap();
        *maintenance_task = Some(handle);
    }

    /// Perform maintenance on the pool
    async fn perform_maintenance(&self) -> Result<()> {
        debug!("Performing pool maintenance");

        // Check for idle browsers that have been unused for too long
        let now = Instant::now();
        let max_idle_duration = Duration::from_secs(self.config.max_idle_time_secs);
        
        let browsers_to_recycle: Vec<String> = self
            .browsers
            .iter()
            .filter_map(|pair| {
                let info = pair.value();
                if info.status == BrowserStatus::Idle
                    && now.duration_since(info.last_used) > max_idle_duration
                    && self.browsers.len() > self.config.min_size
                {
                    Some(info.id.clone())
                } else {
                    None
                }
            })
            .collect();

        for browser_id in browsers_to_recycle {
            debug!(
                "Recycling idle browser {} (exceeded max idle time of {} seconds)",
                browser_id, self.config.max_idle_time_secs
            );
            
            if let Err(e) = self.recycle_browser(&browser_id).await {
                warn!("Failed to recycle idle browser {}: {}", browser_id, e);
            }
        }

        // Ensure we have at least min_size browsers
        let shortfall = self.config.min_size.saturating_sub(self.browsers.len());
        if shortfall > 0 {
            info!("Creating {} browsers to maintain minimum pool size", shortfall);
            
            let futures: Vec<_> = (0..shortfall).map(|_| self.create_browser()).collect();
            let results = future::join_all(futures).await;
            
            let success_count = results.iter().filter(|r| r.is_ok()).count();
            if success_count < shortfall {
                warn!(
                    "Only created {}/{} browsers to maintain minimum pool size",
                    success_count, shortfall
                );
            }
        }

        if self.config.enable_metrics {
            gauge!("browser_pool.size", self.browsers.len() as f64);
            gauge!("browser_pool.available", self.available_count() as f64);
        }

        Ok(())
    }

    /// Close all browsers and shut down the pool
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down browser pool");

        // Stop maintenance task
        let mut maintenance_task = self.maintenance_task.lock().await;
        if let Some(handle) = maintenance_task.take() {
            handle.abort();
        }

        // Get all browser IDs
        let browser_ids: Vec<String> = self.browsers.iter().map(|pair| pair.key().clone()).collect();

        // Close all browsers
        for browser_id in browser_ids {
            if let Some(entry) = self.browsers.get(&browser_id) {
                let browser = entry.value().browser.clone();
                if let Err(e) = browser.close().await {
                    warn!("Error closing browser {}: {}", browser_id, e);
                }
            }
        }

        // Clear the pool
        self.browsers.clear();

        if self.config.enable_metrics {
            gauge!("browser_pool.size", 0.0);
            gauge!("browser_pool.available", 0.0);
        }

        Ok(())
    }
}

impl Clone for BrowserPool {
    fn clone(&self) -> Self {
        Self {
            browsers: self.browsers.clone(),
            creation_semaphore: Semaphore::new(self.config.max_size),
            moonlight: self.moonlight.clone(),
            config: self.config.clone(),
            maintenance_task: Mutex::new(None),
        }
    }
}

impl Drop for BrowserPool {
    fn drop(&mut self) {
        // Check if this is the last reference to the pool
        if Arc::strong_count(&self.moonlight) == 1 {
            // Schedule shutdown
            let pool = self.clone();
            tokio::spawn(async move {
                if let Err(e) = pool.shutdown().await {
                    error!("Error shutting down browser pool: {}", e);
                }
            });
        }
    }
}

// This is a placeholder implementation for Browser to make the code compile
impl Default for Browser {
    fn default() -> Self {
        unimplemented!("This is a placeholder and should never be called")
    }
} 