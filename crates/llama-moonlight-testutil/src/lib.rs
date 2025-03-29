use anyhow::Result;
use llama_moonlight_core::{
    Browser, BrowserContext, BrowserType, Moonlight, Page,
    options::{BrowserOptions, ContextOptions, PageOptions},
};
use log::{debug, info, warn};
use mockito::{mock, server_url, Matcher};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::TcpListener,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tempfile::TempDir;
use thiserror::Error;
use uuid::Uuid;
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{method, path};

pub mod mocks;
pub mod fixtures;
pub mod assertions;

#[derive(Error, Debug)]
pub enum TestUtilError {
    #[error("Mock server error: {0}")]
    MockServerError(String),
    
    #[error("File I/O error: {0}")]
    FileError(#[from] std::io::Error),
    
    #[error("Moonlight error: {0}")]
    MoonlightError(#[from] llama_moonlight_core::Error),
    
    #[error("Test setup error: {0}")]
    SetupError(String),
    
    #[error("Assertion error: {0}")]
    AssertionError(String),
    
    #[error("Timeout error")]
    TimeoutError,
    
    #[error("Other error: {0}")]
    Other(String),
}

/// A fixture that runs a mock browser for testing
pub struct BrowserFixture {
    /// The browser instance
    pub browser: Browser,
    /// The root path for any temp files created during the test
    pub temp_dir: TempDir,
    /// The browser type used for this fixture
    pub browser_type: String,
}

impl BrowserFixture {
    /// Create a new browser fixture with default options
    pub async fn new(browser_type: &str) -> Result<Self, TestUtilError> {
        let options = BrowserOptions {
            headless: Some(true),
            ..BrowserOptions::default()
        };
        Self::with_options(browser_type, options).await
    }
    
    /// Create a new browser fixture with custom options
    pub async fn with_options(browser_type: &str, options: BrowserOptions) -> Result<Self, TestUtilError> {
        let temp_dir = TempDir::new()?;
        
        let moonlight = Moonlight::new().await?;
        let browser_type_obj = moonlight.browser_type(browser_type)
            .ok_or_else(|| TestUtilError::SetupError(format!("Browser type '{}' not found", browser_type)))?;
        
        let browser = browser_type_obj.launch_with_options(options).await?;
        
        Ok(Self {
            browser,
            temp_dir,
            browser_type: browser_type.to_string(),
        })
    }
    
    /// Create a new context
    pub async fn new_context(&self) -> Result<BrowserContext, TestUtilError> {
        let context = self.browser.new_context().await?;
        Ok(context)
    }
    
    /// Create a new context with options
    pub async fn new_context_with_options(&self, options: ContextOptions) -> Result<BrowserContext, TestUtilError> {
        let context = self.browser.new_context_with_options(options).await?;
        Ok(context)
    }
    
    /// Create a new page
    pub async fn new_page(&self) -> Result<Page, TestUtilError> {
        let context = self.new_context().await?;
        let page = context.new_page().await?;
        Ok(page)
    }
    
    /// Create a temporary file in the fixture's temp directory
    pub fn create_temp_file(&self, name: &str, contents: &str) -> Result<PathBuf, TestUtilError> {
        let path = self.temp_dir.path().join(name);
        std::fs::write(&path, contents)?;
        Ok(path)
    }
}

impl Drop for BrowserFixture {
    fn drop(&mut self) {
        // Close the browser when the fixture is dropped
        // This is intentionally blocking to ensure the browser is closed
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async {
                if let Err(e) = self.browser.close().await {
                    warn!("Error closing browser in fixture drop: {}", e);
                }
            });
    }
}

/// A fixture that runs a mock HTTP server for testing
pub struct HttpServerFixture {
    /// The mock server instance
    pub server: MockServer,
    /// The base URL of the mock server
    pub url: String,
}

impl HttpServerFixture {
    /// Create a new HTTP server fixture
    pub async fn new() -> Result<Self, TestUtilError> {
        let server = MockServer::start().await;
        let url = server.uri();
        
        Ok(Self {
            server,
            url,
        })
    }
    
    /// Add a mock response for a GET request
    pub async fn mock_get(&self, path: &str, status: u16, body: &str) -> Result<(), TestUtilError> {
        Mock::given(method("GET"))
            .and(path(path))
            .respond_with(ResponseTemplate::new(status).set_body_string(body))
            .mount(&self.server)
            .await;
        
        Ok(())
    }
    
    /// Add a mock response for a POST request
    pub async fn mock_post(&self, path: &str, status: u16, body: &str) -> Result<(), TestUtilError> {
        Mock::given(method("POST"))
            .and(path(path))
            .respond_with(ResponseTemplate::new(status).set_body_string(body))
            .mount(&self.server)
            .await;
        
        Ok(())
    }
    
    /// Add a mock response with delay for testing timeouts
    pub async fn mock_with_delay(&self, path: &str, status: u16, body: &str, delay_ms: u64) -> Result<(), TestUtilError> {
        Mock::given(method("GET"))
            .and(path(path))
            .respond_with(ResponseTemplate::new(status)
                .set_body_string(body)
                .set_delay(Duration::from_millis(delay_ms)))
            .mount(&self.server)
            .await;
        
        Ok(())
    }
    
    /// Get the full URL for a path
    pub fn url_for(&self, path: &str) -> String {
        format!("{}{}", self.url, path)
    }
}

/// A combined fixture with both browser and HTTP server
pub struct IntegrationTestFixture {
    /// The browser fixture
    pub browser: BrowserFixture,
    /// The HTTP server fixture
    pub http: HttpServerFixture,
}

impl IntegrationTestFixture {
    /// Create a new integration test fixture
    pub async fn new(browser_type: &str) -> Result<Self, TestUtilError> {
        let browser = BrowserFixture::new(browser_type).await?;
        let http = HttpServerFixture::new().await?;
        
        Ok(Self {
            browser,
            http,
        })
    }
    
    /// Navigate to a path on the HTTP server
    pub async fn navigate_to(&self, path: &str) -> Result<Page, TestUtilError> {
        let page = self.browser.new_page().await?;
        let url = self.http.url_for(path);
        page.goto(&url).await?;
        Ok(page)
    }
}

/// Helper function to find an available port
pub fn find_available_port() -> Result<u16, TestUtilError> {
    // Try to bind to a random port (0) and let the OS choose an available one
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    Ok(port)
}

/// A helper to generate random data for tests
pub struct TestDataGenerator;

impl TestDataGenerator {
    /// Generate a random string
    pub fn random_string(length: usize) -> String {
        use rand::{distributions::Alphanumeric, Rng};
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(length)
            .map(char::from)
            .collect()
    }
    
    /// Generate a random email
    pub fn random_email() -> String {
        let username = Self::random_string(8);
        let domain = Self::random_string(5);
        format!("{}@{}.com", username.to_lowercase(), domain.to_lowercase())
    }
    
    /// Generate a random URL
    pub fn random_url() -> String {
        let domain = Self::random_string(8);
        let path = Self::random_string(5);
        format!("https://{}.com/{}", domain.to_lowercase(), path.to_lowercase())
    }
    
    /// Generate a UUID
    pub fn uuid() -> String {
        Uuid::new_v4().to_string()
    }
}

/// Custom test macro to set up an integration test
#[macro_export]
macro_rules! integration_test {
    ($name:ident, $browser_type:expr, $test:expr) => {
        #[tokio::test]
        async fn $name() -> Result<(), TestUtilError> {
            let fixture = IntegrationTestFixture::new($browser_type).await?;
            $test(fixture).await
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_browser_fixture() -> Result<(), TestUtilError> {
        let fixture = BrowserFixture::new("chromium").await?;
        let page = fixture.new_page().await?;
        // Test is successful if we can create a fixture and page
        Ok(())
    }
    
    #[tokio::test]
    async fn test_http_server_fixture() -> Result<(), TestUtilError> {
        let fixture = HttpServerFixture::new().await?;
        fixture.mock_get("/test", 200, "Hello World").await?;
        
        // Test the mock server (would use reqwest in a real test)
        Ok(())
    }
    
    #[tokio::test]
    async fn test_find_available_port() -> Result<(), TestUtilError> {
        let port = find_available_port()?;
        assert!(port > 0);
        Ok(())
    }
} 