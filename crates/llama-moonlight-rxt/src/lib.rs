use anyhow::Result;
use futures::stream::Stream;
use log::{debug, info, warn};
use llama_moonlight_core::{
    Browser, BrowserContext, BrowserType, Moonlight, Page,
    options::{BrowserOptions, ContextOptions, PageOptions}
};
use rxrust::{
    prelude::*,
    ops::{map, filter, flat_map, take, merge},
};
use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};
use thiserror::Error;
use tokio::sync::mpsc;

#[derive(Error, Debug)]
pub enum RxtError {
    #[error("Rxrust error: {0}")]
    RxrustError(String),
    
    #[error("Moonlight error: {0}")]
    MoonlightError(#[from] llama_moonlight_core::Error),
    
    #[error("Timeout error: {0}")]
    TimeoutError(String),
    
    #[error("Channel closed")]
    ChannelClosed,
    
    #[error("Other error: {0}")]
    Other(String),
}

#[derive(Debug, Clone)]
pub enum BrowserEvent {
    /// Browser has been launched
    Launched { browser_id: String },
    /// Browser has been closed
    Closed { browser_id: String },
    /// Context has been created
    ContextCreated { context_id: String, browser_id: String },
    /// Context has been closed
    ContextClosed { context_id: String, browser_id: String },
    /// Page has been created
    PageCreated { page_id: String, context_id: String, browser_id: String },
    /// Page has been closed
    PageClosed { page_id: String, context_id: String, browser_id: String },
    /// Navigation has started
    NavigationStarted { page_id: String, url: String },
    /// Navigation has completed
    NavigationCompleted { page_id: String, url: String, status: u16 },
    /// Console message received
    ConsoleMessage { page_id: String, text: String, level: String },
    /// JavaScript dialog opened
    Dialog { page_id: String, dialog_type: String, message: String },
    /// Network request started
    RequestStarted { page_id: String, request_id: String, url: String, method: String },
    /// Network request completed
    RequestCompleted { page_id: String, request_id: String, url: String, status: u16 },
    /// Network request failed
    RequestFailed { page_id: String, request_id: String, url: String, error: String },
    /// Download started
    DownloadStarted { page_id: String, download_id: String, url: String },
    /// Download completed
    DownloadCompleted { page_id: String, download_id: String, path: String },
    /// Custom event
    Custom { event_type: String, data: serde_json::Value },
}

/// A browser with reactive extension functionality
pub struct RxBrowser {
    browser: Arc<Browser>,
    events: Observable<BrowserEvent>,
}

impl RxBrowser {
    /// Create a new RxBrowser from a Browser instance
    pub fn new(browser: Arc<Browser>) -> Self {
        let browser_id = browser.browser_type().name().to_string();
        
        // Create a channel for browser events
        let (tx, rx) = mpsc::channel(100);
        
        // Send launched event
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            let _ = tx_clone.send(BrowserEvent::Launched {
                browser_id: browser_id.clone(),
            }).await;
        });
        
        // Create observable from the receiver
        let events = Observable::create(move |s| {
            let mut rx = rx.clone();
            tokio::spawn(async move {
                while let Some(event) = rx.recv().await {
                    if s.is_subscribed() {
                        s.next(event);
                    } else {
                        break;
                    }
                }
                s.complete();
            });
        });
        
        Self { browser, events }
    }
    
    /// Get the wrapped browser instance
    pub fn browser(&self) -> Arc<Browser> {
        self.browser.clone()
    }
    
    /// Get the observable stream of browser events
    pub fn events(&self) -> Observable<BrowserEvent> {
        self.events.clone()
    }
    
    /// Create a new context and return it as an RxContext
    pub async fn new_context(&self) -> Result<RxContext, RxtError> {
        let context = self.browser.new_context().await?;
        let context_id = context.id().to_string();
        let browser_id = self.browser.browser_type().name().to_string();
        
        let context_events = self.events
            .filter(move |event| {
                matches!(event, 
                    BrowserEvent::ContextCreated { context_id: id, .. } |
                    BrowserEvent::ContextClosed { context_id: id, .. } |
                    BrowserEvent::PageCreated { context_id: id, .. } |
                    BrowserEvent::PageClosed { context_id: id, .. }
                    if id == &context_id
                )
            });
        
        Ok(RxContext::new(Arc::new(context), context_events))
    }
    
    /// Create a new context with options and return it as an RxContext
    pub async fn new_context_with_options(&self, options: ContextOptions) -> Result<RxContext, RxtError> {
        let context = self.browser.new_context_with_options(options).await?;
        let context_id = context.id().to_string();
        let browser_id = self.browser.browser_type().name().to_string();
        
        let context_events = self.events
            .filter(move |event| {
                matches!(event, 
                    BrowserEvent::ContextCreated { context_id: id, .. } |
                    BrowserEvent::ContextClosed { context_id: id, .. } |
                    BrowserEvent::PageCreated { context_id: id, .. } |
                    BrowserEvent::PageClosed { context_id: id, .. }
                    if id == &context_id
                )
            });
        
        Ok(RxContext::new(Arc::new(context), context_events))
    }
    
    /// Close the browser
    pub async fn close(&self) -> Result<(), RxtError> {
        self.browser.close().await?;
        Ok(())
    }
}

/// A browser context with reactive extension functionality
pub struct RxContext {
    context: Arc<BrowserContext>,
    events: Observable<BrowserEvent>,
}

impl RxContext {
    /// Create a new RxContext from a BrowserContext instance
    fn new(context: Arc<BrowserContext>, events: Observable<BrowserEvent>) -> Self {
        Self { context, events }
    }
    
    /// Get the wrapped context instance
    pub fn context(&self) -> Arc<BrowserContext> {
        self.context.clone()
    }
    
    /// Get the observable stream of context events
    pub fn events(&self) -> Observable<BrowserEvent> {
        self.events.clone()
    }
    
    /// Create a new page and return it as an RxPage
    pub async fn new_page(&self) -> Result<RxPage, RxtError> {
        let page = self.context.new_page().await?;
        let page_id = page.url().await?;
        let context_id = self.context.id().to_string();
        
        let page_events = self.events
            .filter(move |event| {
                matches!(event, 
                    BrowserEvent::PageCreated { page_id: id, .. } |
                    BrowserEvent::PageClosed { page_id: id, .. } |
                    BrowserEvent::NavigationStarted { page_id: id, .. } |
                    BrowserEvent::NavigationCompleted { page_id: id, .. } |
                    BrowserEvent::ConsoleMessage { page_id: id, .. } |
                    BrowserEvent::Dialog { page_id: id, .. } |
                    BrowserEvent::RequestStarted { page_id: id, .. } |
                    BrowserEvent::RequestCompleted { page_id: id, .. } |
                    BrowserEvent::RequestFailed { page_id: id, .. } |
                    BrowserEvent::DownloadStarted { page_id: id, .. } |
                    BrowserEvent::DownloadCompleted { page_id: id, .. }
                    if id == &page_id
                )
            });
        
        Ok(RxPage::new(Arc::new(page), page_events))
    }
    
    /// Close the context
    pub async fn close(&self) -> Result<(), RxtError> {
        self.context.close().await?;
        Ok(())
    }
}

/// A page with reactive extension functionality
pub struct RxPage {
    page: Arc<Page>,
    events: Observable<BrowserEvent>,
}

impl RxPage {
    /// Create a new RxPage from a Page instance
    fn new(page: Arc<Page>, events: Observable<BrowserEvent>) -> Self {
        Self { page, events }
    }
    
    /// Get the wrapped page instance
    pub fn page(&self) -> Arc<Page> {
        self.page.clone()
    }
    
    /// Get the observable stream of page events
    pub fn events(&self) -> Observable<BrowserEvent> {
        self.events.clone()
    }
    
    /// Navigate to a URL and return an observable of navigation events
    pub async fn goto(&self, url: &str) -> Result<Observable<BrowserEvent>, RxtError> {
        let page_id = self.page.url().await?;
        
        // Filter for navigation events for this page and URL
        let nav_events = self.events
            .filter(move |event| {
                matches!(event, 
                    BrowserEvent::NavigationStarted { page_id: id, url: u } |
                    BrowserEvent::NavigationCompleted { page_id: id, url: u, .. }
                    if id == &page_id && u == url
                )
            })
            .take(2); // Take start and completion events
        
        // Start navigation
        self.page.goto(url).await?;
        
        Ok(nav_events)
    }
    
    /// Wait for a specific event with a timeout
    pub async fn wait_for_event(&self, predicate: impl Fn(&BrowserEvent) -> bool + Send + 'static, timeout_ms: u64) -> Result<BrowserEvent, RxtError> {
        let (tx, mut rx) = tokio::sync::oneshot::channel();
        
        let subscription = self.events
            .filter(move |event| predicate(event))
            .take(1)
            .subscribe(move |event| {
                let _ = tx.send(event);
            });
        
        // Set up timeout
        let timeout = tokio::time::timeout(Duration::from_millis(timeout_ms), rx).await;
        
        match timeout {
            Ok(Ok(event)) => Ok(event),
            Ok(Err(_)) => Err(RxtError::ChannelClosed),
            Err(_) => Err(RxtError::TimeoutError(format!("Timeout after {}ms", timeout_ms))),
        }
    }
    
    /// Take a screenshot
    pub async fn screenshot(&self, path: &str) -> Result<(), RxtError> {
        self.page.screenshot(path).await?;
        Ok(())
    }
    
    /// Close the page
    pub async fn close(&self) -> Result<(), RxtError> {
        self.page.close().await?;
        Ok(())
    }
    
    /// Get network requests as an observable stream
    pub fn network_requests(&self) -> Observable<BrowserEvent> {
        self.events
            .filter(|event| {
                matches!(event, 
                    BrowserEvent::RequestStarted { .. } |
                    BrowserEvent::RequestCompleted { .. } |
                    BrowserEvent::RequestFailed { .. }
                )
            })
    }
    
    /// Get console messages as an observable stream
    pub fn console_messages(&self) -> Observable<String> {
        self.events
            .filter(|event| matches!(event, BrowserEvent::ConsoleMessage { .. }))
            .map(|event| {
                if let BrowserEvent::ConsoleMessage { text, .. } = event {
                    text
                } else {
                    "".to_string() // This should never happen due to the filter
                }
            })
    }
}

/// Helper functions to create RxBrowser instances
pub async fn launch_browser(browser_type: &str) -> Result<RxBrowser, RxtError> {
    let moonlight = Moonlight::new().await?;
    let browser_type = moonlight.browser_type(browser_type)
        .ok_or_else(|| RxtError::Other(format!("Browser type '{}' not found", browser_type)))?;
    
    let browser = browser_type.launch().await?;
    Ok(RxBrowser::new(Arc::new(browser)))
}

/// Launch a browser with options
pub async fn launch_browser_with_options(browser_type: &str, options: BrowserOptions) -> Result<RxBrowser, RxtError> {
    let moonlight = Moonlight::new().await?;
    let browser_type = moonlight.browser_type(browser_type)
        .ok_or_else(|| RxtError::Other(format!("Browser type '{}' not found", browser_type)))?;
    
    let browser = browser_type.launch_with_options(options).await?;
    Ok(RxBrowser::new(Arc::new(browser)))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_rx_browser_events() {
        // This is just a skeleton test - would need actual browser for real testing
        let moonlight = Moonlight::new().await.unwrap();
        // Test implementation would continue here
    }
} 