//! Browser management module.
//!
//! This module provides functionality for launching and interacting with browsers.

use crate::context::{BrowserContext, ContextOptions};
use crate::errors::{Error, Result};
use crate::protocol::Connection;
use crate::options::BrowserOptions;
use std::process::{Child, Command};
use std::sync::Arc;
use tokio::sync::Mutex;
use log::{debug, info, warn};
use std::path::{Path, PathBuf};
use tokio::time::{timeout, Duration};
use uuid::Uuid;
use std::fs;
use tempfile::TempDir;
use llama_headers_rs;

/// Represents a type of browser (chromium, firefox, webkit).
#[derive(Debug, Clone)]
pub struct BrowserType {
    name: String,
    executable_path: Option<PathBuf>,
}

impl BrowserType {
    /// Creates a new browser type.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            executable_path: None,
        }
    }
    
    /// Creates a new browser type with a specific executable path.
    pub fn new_with_path(name: &str, path: &Path) -> Self {
        Self {
            name: name.to_string(),
            executable_path: Some(path.to_path_buf()),
        }
    }
    
    /// Returns the name of the browser type.
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Launches a browser instance with default options.
    pub async fn launch(&self) -> Result<Browser> {
        self.launch_with_options(BrowserOptions::default()).await
    }
    
    /// Launches a browser instance with the specified options.
    pub async fn launch_with_options(&self, options: BrowserOptions) -> Result<Browser> {
        info!("Launching {} browser", self.name);
        
        // Create a temporary user data directory if none is specified
        let user_data_dir = if let Some(dir) = options.user_data_dir.clone() {
            PathBuf::from(dir)
        } else {
            let temp_dir = TempDir::new()
                .map_err(|e| Error::BrowserLaunchError(format!("Failed to create temp directory: {}", e)))?;
            temp_dir.into_path()
        };
        
        // Prepare browser executable and arguments
        let (executable, args) = self.prepare_launch_command(&user_data_dir, &options)?;
        
        // Launch browser process
        let mut cmd = Command::new(executable);
        cmd.args(args);
        
        // Add environment variables
        if let Some(env_vars) = &options.env {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }
        
        // Start the process
        let child = cmd
            .spawn()
            .map_err(|e| Error::BrowserLaunchError(format!("Failed to spawn browser process: {}", e)))?;
        
        // Wait for the WebSocket endpoint to be available
        let ws_endpoint = self.wait_for_websocket_endpoint(&user_data_dir, options.timeout_ms.unwrap_or(30000)).await?;
        
        // Connect to the browser
        let connection = Connection::connect(&ws_endpoint).await?;
        
        // Create browser object
        let browser = Browser {
            connection: Arc::new(connection),
            process: Arc::new(Mutex::new(Some(child))),
            browser_type: self.clone(),
            user_data_dir,
        };
        
        info!("Successfully launched {} browser", self.name);
        Ok(browser)
    }
    
    /// Prepares the launch command for the specific browser type.
    fn prepare_launch_command(&self, user_data_dir: &Path, options: &BrowserOptions) -> Result<(String, Vec<String>)> {
        match self.name.as_str() {
            "chromium" => {
                let executable = if let Some(path) = &self.executable_path {
                    path.to_string_lossy().to_string()
                } else {
                    self.find_executable("chrome")?
                };
                
                let mut args = vec![
                    format!("--user-data-dir={}", user_data_dir.to_string_lossy()),
                    "--remote-debugging-port=0".to_string(),
                    "--no-first-run".to_string(),
                ];
                
                if options.headless.unwrap_or(true) {
                    args.push("--headless".to_string());
                }
                
                if let Some(args_option) = &options.args {
                    args.extend(args_option.clone());
                }
                
                Ok((executable, args))
            },
            "firefox" => {
                let executable = if let Some(path) = &self.executable_path {
                    path.to_string_lossy().to_string()
                } else {
                    self.find_executable("firefox")?
                };
                
                let mut args = vec![
                    "-profile".to_string(),
                    user_data_dir.to_string_lossy().to_string(),
                    "-remote-debugging-port".to_string(),
                    "0".to_string(),
                ];
                
                if options.headless.unwrap_or(true) {
                    args.push("-headless".to_string());
                }
                
                if let Some(args_option) = &options.args {
                    args.extend(args_option.clone());
                }
                
                Ok((executable, args))
            },
            "webkit" => {
                let executable = if let Some(path) = &self.executable_path {
                    path.to_string_lossy().to_string()
                } else {
                    self.find_executable("webkit_server")?
                };
                
                let mut args = vec![
                    format!("--user-data-dir={}", user_data_dir.to_string_lossy()),
                ];
                
                if options.headless.unwrap_or(true) {
                    args.push("--headless".to_string());
                }
                
                if let Some(args_option) = &options.args {
                    args.extend(args_option.clone());
                }
                
                Ok((executable, args))
            },
            _ => Err(Error::BrowserTypeNotFound(self.name.clone())),
        }
    }
    
    /// Finds the browser executable in the system.
    fn find_executable(&self, name: &str) -> Result<String> {
        // First check common locations
        let locations = match self.name.as_str() {
            "chromium" => {
                if cfg!(target_os = "macos") {
                    vec![
                        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
                        "/Applications/Chromium.app/Contents/MacOS/Chromium",
                    ]
                } else if cfg!(target_os = "windows") {
                    vec![
                        r"C:\Program Files\Google\Chrome\Application\chrome.exe",
                        r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
                    ]
                } else {
                    vec![
                        "/usr/bin/google-chrome",
                        "/usr/bin/chromium",
                        "/usr/bin/chromium-browser",
                    ]
                }
            },
            "firefox" => {
                if cfg!(target_os = "macos") {
                    vec![
                        "/Applications/Firefox.app/Contents/MacOS/firefox",
                    ]
                } else if cfg!(target_os = "windows") {
                    vec![
                        r"C:\Program Files\Mozilla Firefox\firefox.exe",
                        r"C:\Program Files (x86)\Mozilla Firefox\firefox.exe",
                    ]
                } else {
                    vec![
                        "/usr/bin/firefox",
                    ]
                }
            },
            "webkit" => {
                if cfg!(target_os = "macos") {
                    vec![
                        "/Applications/Safari.app/Contents/MacOS/Safari",
                    ]
                } else if cfg!(target_os = "windows") {
                    vec![
                        r"C:\Program Files\Safari\Safari.exe",
                    ]
                } else {
                    vec![
                        "/usr/bin/webkit_server",
                    ]
                }
            },
            _ => vec![],
        };
        
        // Check if any of the common locations exist
        for location in locations {
            if Path::new(location).exists() {
                return Ok(location.to_string());
            }
        }
        
        // If not found in common locations, try to find in PATH
        let output = Command::new("which")
            .arg(name)
            .output()
            .map_err(|_| Error::BrowserLaunchError(format!("Failed to find {} executable", name)))?;
        
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Ok(path);
            }
        }
        
        Err(Error::BrowserLaunchError(format!("Failed to find {} executable", name)))
    }
    
    /// Wait for the WebSocket endpoint to be available after browser launch.
    async fn wait_for_websocket_endpoint(&self, user_data_dir: &Path, timeout_ms: u64) -> Result<String> {
        let start_time = std::time::Instant::now();
        let timeout_duration = Duration::from_millis(timeout_ms);
        
        // Path to the DevTools JSON file depends on the browser type
        let devtools_file = match self.name.as_str() {
            "chromium" => user_data_dir.join("DevTools/DevToolsActivePort"),
            "firefox" => user_data_dir.join("firefox_debug.json"),
            "webkit" => user_data_dir.join("webkit_debug.json"),
            _ => return Err(Error::BrowserTypeNotFound(self.name.clone())),
        };
        
        // Wait for the file to be created
        while start_time.elapsed() < timeout_duration {
            if devtools_file.exists() {
                // Parse the file to get WebSocket endpoint
                match self.name.as_str() {
                    "chromium" => {
                        let content = fs::read_to_string(&devtools_file)?;
                        let lines: Vec<&str> = content.lines().collect();
                        if lines.len() >= 2 {
                            let port = lines[0].trim();
                            return Ok(format!("ws://127.0.0.1:{}/devtools/browser", port));
                        }
                    },
                    "firefox" | "webkit" => {
                        let content = fs::read_to_string(&devtools_file)?;
                        let data: serde_json::Value = serde_json::from_str(&content)?;
                        if let Some(ws_url) = data["webSocketDebuggerUrl"].as_str() {
                            return Ok(ws_url.to_string());
                        }
                    },
                    _ => return Err(Error::BrowserTypeNotFound(self.name.clone())),
                }
            }
            
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        Err(Error::TimeoutError(format!("Timed out waiting for browser WebSocket endpoint")))
    }
}

/// Represents a browser instance.
#[derive(Debug)]
pub struct Browser {
    connection: Arc<Connection>,
    process: Arc<Mutex<Option<Child>>>,
    browser_type: BrowserType,
    user_data_dir: PathBuf,
}

impl Browser {
    /// Creates a new browser context (similar to an incognito window).
    pub async fn new_context(&self) -> Result<BrowserContext> {
        self.new_context_with_options(ContextOptions::default()).await
    }
    
    /// Creates a new browser context with the specified options.
    pub async fn new_context_with_options(&self, options: ContextOptions) -> Result<BrowserContext> {
        info!("Creating new browser context");
        
        // Call the protocol method to create a new context
        let params = match self.browser_type.name() {
            "chromium" => {
                serde_json::json!({
                    "disposeOnDetach": true,
                    "userAgent": options.user_agent,
                    "locale": options.locale,
                    "viewport": options.viewport,
                    "ignoreHTTPSErrors": options.ignore_https_errors,
                    "acceptDownloads": options.accept_downloads,
                })
            },
            "firefox" => {
                serde_json::json!({
                    "disposeOnDetach": true,
                    "userAgent": options.user_agent,
                    "locale": options.locale,
                    "viewport": options.viewport,
                    "ignoreHTTPSErrors": options.ignore_https_errors,
                    "acceptDownloads": options.accept_downloads,
                })
            },
            "webkit" => {
                serde_json::json!({
                    "disposeOnDetach": true,
                    "userAgent": options.user_agent,
                    "locale": options.locale,
                    "viewport": options.viewport,
                    "ignoreHTTPSErrors": options.ignore_https_errors,
                    "acceptDownloads": options.accept_downloads,
                })
            },
            _ => return Err(Error::BrowserTypeNotFound(self.browser_type.name().to_string())),
        };
        
        let result = self.connection.send_request(
            "Browser.createContext".to_string(),
            Some(params),
        ).await?;
        
        let context_id = result["contextId"].as_str()
            .ok_or_else(|| Error::ContextCreationError("Failed to get context ID".to_string()))?
            .to_string();
        
        let context = BrowserContext {
            connection: self.connection.clone(),
            id: context_id,
            browser_type: self.browser_type.name().to_string(),
            options,
        };
        
        info!("Successfully created browser context");
        Ok(context)
    }
    
    /// Returns the browser type.
    pub fn browser_type(&self) -> &BrowserType {
        &self.browser_type
    }
    
    /// Returns the WebSocket connection.
    pub fn connection(&self) -> Arc<Connection> {
        self.connection.clone()
    }
    
    /// Closes the browser.
    pub async fn close(&self) -> Result<()> {
        info!("Closing browser");
        
        // Send browser close command via protocol
        let _ = self.connection.send_request(
            "Browser.close".to_string(),
            None,
        ).await;
        
        // Close the connection
        let _ = self.connection.close().await;
        
        // Kill the process if it's still running
        let mut process = self.process.lock().await;
        if let Some(child) = process.take() {
            if let Err(e) = child.kill() {
                warn!("Failed to kill browser process: {}", e);
            }
        }
        
        // Clean up the user data directory if it's temporary
        if self.user_data_dir.exists() {
            if let Err(e) = fs::remove_dir_all(&self.user_data_dir) {
                warn!("Failed to remove user data directory: {}", e);
            }
        }
        
        info!("Browser closed");
        Ok(())
    }
    
    /// Sets up a stealth browser context using llama-headers-rs.
    #[cfg(feature = "stealth")]
    pub async fn stealth_context(&self, url: &str) -> Result<BrowserContext> {
        use llama_headers_rs::get_header;
        
        let header = get_header(url, None).map_err(Error::HeadersError)?;
        
        let mut options = ContextOptions::default();
        options.user_agent = Some(header.user_agent.to_string());
        
        // Create a context with the user agent from llama-headers-rs
        let context = self.new_context_with_options(options).await?;
        
        info!("Created stealth browser context with llama-headers-rs");
        Ok(context)
    }
}

impl Drop for Browser {
    fn drop(&mut self) {
        if let Ok(mut process) = self.process.try_lock() {
            if let Some(mut child) = process.take() {
                let _ = child.kill();
            }
        }
        
        if self.user_data_dir.exists() {
            let _ = fs::remove_dir_all(&self.user_data_dir);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // These tests are commented out as they require actual browser binaries
    // Uncomment and modify them for your testing environment
    
    /*
    #[tokio::test]
    async fn test_browser_type_new() {
        let browser_type = BrowserType::new("chromium");
        assert_eq!(browser_type.name(), "chromium");
    }
    
    #[tokio::test]
    async fn test_browser_launch_and_close() {
        let browser_type = BrowserType::new("chromium");
        let browser = browser_type.launch().await.unwrap();
        
        assert_eq!(browser.browser_type().name(), "chromium");
        
        browser.close().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_browser_new_context() {
        let browser_type = BrowserType::new("chromium");
        let browser = browser_type.launch().await.unwrap();
        
        let context = browser.new_context().await.unwrap();
        assert!(!context.id.is_empty());
        
        browser.close().await.unwrap();
    }
    */
} 