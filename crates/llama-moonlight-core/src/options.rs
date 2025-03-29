//! Configuration options for browser, context, and page.
//!
//! This module defines the configuration options for different components.

use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Configuration options for launching a browser.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BrowserOptions {
    /// Path to browser executable. If not specified, the default installation will be used.
    pub executable_path: Option<String>,
    
    /// Whether to run browser in headless mode.
    pub headless: Option<bool>,
    
    /// Path to a custom user data directory.
    pub user_data_dir: Option<String>,
    
    /// Additional arguments to pass to the browser process.
    pub args: Option<Vec<String>>,
    
    /// Environment variables to set for the browser process.
    pub env: Option<HashMap<String, String>>,
    
    /// Whether to ignore HTTPS errors.
    pub ignore_https_errors: Option<bool>,
    
    /// Whether to enable stealth mode.
    pub stealth: Option<bool>,
    
    /// Proxy settings.
    pub proxy: Option<ProxySettings>,
    
    /// Timeout for launching browser in milliseconds.
    pub timeout_ms: Option<u64>,
    
    /// Whether to slow down operations by the specified amount of milliseconds.
    pub slow_mo: Option<u64>,
    
    /// Whether to enable DevTools.
    pub devtools: Option<bool>,
    
    /// Download path.
    pub downloads_path: Option<PathBuf>,
}

impl Default for BrowserOptions {
    fn default() -> Self {
        Self {
            executable_path: None,
            headless: Some(true),
            user_data_dir: None,
            args: None,
            env: None,
            ignore_https_errors: Some(false),
            stealth: Some(false),
            proxy: None,
            timeout_ms: Some(30000),
            slow_mo: None,
            devtools: Some(false),
            downloads_path: None,
        }
    }
}

/// Configuration options for a browser context.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContextOptions {
    /// Custom user agent.
    pub user_agent: Option<String>,
    
    /// Browser locale.
    pub locale: Option<String>,
    
    /// Browser timezone ID.
    pub timezone_id: Option<String>,
    
    /// Geolocation.
    pub geolocation: Option<Geolocation>,
    
    /// Permissions to grant.
    pub permissions: Option<Vec<String>>,
    
    /// Viewport dimensions.
    pub viewport: Option<Viewport>,
    
    /// Whether the viewport is mobile.
    pub is_mobile: Option<bool>,
    
    /// Device scale factor.
    pub device_scale_factor: Option<f64>,
    
    /// Whether to ignore HTTPS errors.
    pub ignore_https_errors: Option<bool>,
    
    /// Whether to enable JavaScript.
    pub javascript_enabled: Option<bool>,
    
    /// Whether to automatically download attachments.
    pub accept_downloads: Option<bool>,
    
    /// Whether to bypass CSP.
    pub bypass_csp: Option<bool>,
    
    /// Proxy settings.
    pub proxy: Option<ProxySettings>,
    
    /// Cookies to set.
    pub cookies: Option<Vec<Cookie>>,
    
    /// HTTP credentials.
    pub http_credentials: Option<HttpCredentials>,
    
    /// Offline mode.
    pub offline: Option<bool>,
    
    /// Color scheme to emulate.
    pub color_scheme: Option<ColorScheme>,
    
    /// Whether to record videos.
    pub record_video: Option<RecordVideo>,
}

impl Default for ContextOptions {
    fn default() -> Self {
        Self {
            user_agent: None,
            locale: None,
            timezone_id: None,
            geolocation: None,
            permissions: None,
            viewport: Some(Viewport {
                width: 1280,
                height: 720,
            }),
            is_mobile: Some(false),
            device_scale_factor: Some(1.0),
            ignore_https_errors: Some(false),
            javascript_enabled: Some(true),
            accept_downloads: Some(true),
            bypass_csp: Some(false),
            proxy: None,
            cookies: None,
            http_credentials: None,
            offline: Some(false),
            color_scheme: Some(ColorScheme::Light),
            record_video: None,
        }
    }
}

/// Configuration options for a page.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PageOptions {
    /// Page timeout in milliseconds.
    pub timeout_ms: Option<u64>,
    
    /// Navigation timeout in milliseconds.
    pub navigation_timeout_ms: Option<u64>,
    
    /// Whether to wait for network idle before considering navigation complete.
    pub wait_until_network_idle: Option<bool>,
    
    /// Whether to wait for the page to be loaded before continuing.
    pub wait_until: Option<WaitUntilState>,
    
    /// Whether to automatically dismiss dialogs.
    pub auto_dismiss_dialogs: Option<bool>,
    
    /// Whether to enable request interception.
    pub request_interception_enabled: Option<bool>,
    
    /// Whether to enable JavaScript.
    pub javascript_enabled: Option<bool>,
    
    /// Whether to bypass the Content Security Policy.
    pub bypass_csp: Option<bool>,
    
    /// User agent to use for this page.
    pub user_agent: Option<String>,
}

impl Default for PageOptions {
    fn default() -> Self {
        Self {
            timeout_ms: Some(30000),
            navigation_timeout_ms: Some(30000),
            wait_until_network_idle: Some(false),
            wait_until: Some(WaitUntilState::Load),
            auto_dismiss_dialogs: Some(false),
            request_interception_enabled: Some(false),
            javascript_enabled: Some(true),
            bypass_csp: Some(false),
            user_agent: None,
        }
    }
}

/// Viewport dimensions.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Viewport {
    /// Width in pixels.
    pub width: i32,
    
    /// Height in pixels.
    pub height: i32,
}

/// Geolocation coordinates.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Geolocation {
    /// Latitude between -90 and 90.
    pub latitude: f64,
    
    /// Longitude between -180 and 180.
    pub longitude: f64,
    
    /// Accuracy in meters (optional).
    pub accuracy: Option<f64>,
}

/// Proxy settings.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProxySettings {
    /// Proxy server address.
    pub server: String,
    
    /// Bypass list.
    pub bypass: Option<String>,
    
    /// Username for proxy authentication.
    pub username: Option<String>,
    
    /// Password for proxy authentication.
    pub password: Option<String>,
}

/// HTTP authentication credentials.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpCredentials {
    /// Username.
    pub username: String,
    
    /// Password.
    pub password: String,
}

/// Cookie data.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Cookie {
    /// Cookie name.
    pub name: String,
    
    /// Cookie value.
    pub value: String,
    
    /// Cookie domain.
    pub domain: String,
    
    /// Cookie path.
    pub path: String,
    
    /// Expiration date in seconds since the UNIX epoch.
    pub expires: Option<f64>,
    
    /// Whether the cookie is HTTP only.
    pub http_only: Option<bool>,
    
    /// Whether the cookie is secure.
    pub secure: Option<bool>,
    
    /// Same site policy.
    pub same_site: Option<SameSite>,
}

/// Same site cookie policy.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SameSite {
    /// Strict same site policy.
    #[serde(rename = "Strict")]
    Strict,
    
    /// Lax same site policy.
    #[serde(rename = "Lax")]
    Lax,
    
    /// None same site policy.
    #[serde(rename = "None")]
    None,
}

/// Page load states.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum WaitUntilState {
    /// Wait until the load event is fired.
    #[serde(rename = "load")]
    Load,
    
    /// Wait until the DOMContentLoaded event is fired.
    #[serde(rename = "domcontentloaded")]
    DomContentLoaded,
    
    /// Wait until there are no more than 0 network connections for at least 500 ms.
    #[serde(rename = "networkidle")]
    NetworkIdle,
    
    /// Wait until there are no more than 2 network connections for at least 500 ms.
    #[serde(rename = "networkidle2")]
    NetworkIdle2,
}

/// Color schemes for emulation.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ColorScheme {
    /// Light mode.
    #[serde(rename = "light")]
    Light,
    
    /// Dark mode.
    #[serde(rename = "dark")]
    Dark,
    
    /// No color scheme preference.
    #[serde(rename = "no-preference")]
    NoPreference,
}

/// Video recording options.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RecordVideo {
    /// Directory to save videos to.
    pub dir: PathBuf,
    
    /// Video size, defaults to viewport size.
    pub size: Option<VideoSize>,
}

/// Video dimensions.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VideoSize {
    /// Width in pixels.
    pub width: i32,
    
    /// Height in pixels.
    pub height: i32,
} 