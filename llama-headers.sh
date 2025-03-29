#!/bin/bash

# --- Master Shell Script: Self-install, test, and run llama-headers-rs ---

# --- Configuration ---
CRATE_NAME="llama-headers-rs"
EXECUTABLE_NAME="$CRATE_NAME" #  library crate, no executable in bin
COLOR_GREEN="\033[0;32m"
COLOR_YELLOW="\033[0;33m"
COLOR_RED="\033[0;31m"
COLOR_BLUE="\033[0;34m"
COLOR_NC="\033[0m" # No Color

# --- Helper Functions for Colored Output ---
green_echo() {
    echo -e "${COLOR_GREEN}$1${COLOR_NC}"
}

yellow_echo() {
    echo -e "${COLOR_YELLOW}$1${COLOR_NC}"
}

red_echo() {
    echo -e "${COLOR_RED}$1${COLOR_NC}"
}

blue_echo() {
    echo -e "${COLOR_BLUE}$1${COLOR_NC}"
}

# --- Check OS ---
if [[ "$(uname -s)" != "Darwin" ]]; then
    red_echo "This script is designed for macOS."
    exit 1
fi

green_echo "--- Starting ${CRATE_NAME} Installation Script ---"

# --- Check for Rust and Cargo ---
if ! command -v rustc &> /dev/null || ! command -v cargo &> /dev/null; then
    red_echo "Rust and Cargo are required to build and install ${CRATE_NAME}."
    yellow_echo "Please install Rust using rustup: https://rustup.rs/"
    exit 1
fi
green_echo "✓ Rust and Cargo are installed."

# --- Create Cargo.toml ---
yellow_echo "--- Creating Cargo.toml ---"
cat <<EOF > Cargo.toml
[package]
name = "llama-headers-rs"
version = "0.1.0"
edition = "2021"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
rand = "0.8"
regex = "1"
lazy_static = "1.4"
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
EOF
if [ $? -eq 0 ]; then
    green_echo "✓ Cargo.toml created."
else
    red_echo "✗ Failed to create Cargo.toml."
    exit 1
fi

# --- Create src directory ---
mkdir -p src
if [ $? -eq 0 ]; then
    green_echo "✓ src directory created."
else
    red_echo "✗ Failed to create src directory."
    exit 1
fi
mkdir -p tests # Create tests directory

# --- Create src/lib.rs ---
yellow_echo "--- Creating src/lib.rs ---"
cat <<EOF > src/lib.rs
pub mod header;
pub mod user_agent;
pub mod utils;
pub mod errors;
pub mod config;

use crate::header::Header;
use crate::user_agent::UserAgent;
use crate::utils::{get_domain, get_random_referer, get_sec_ch_ua, get_accept_encoding, get_accept_language, get_sec_fetch_dest, get_sec_fetch_mode, get_sec_fetch_site, get_sec_fetch_user, get_connection};
use crate::errors::LlamaHeadersError;
use crate::config::Config;
use std::collections::HashMap;


/// Generates a single \`Header\` instance based on the provided parameters.
///
/// # Arguments
///
/// * \`url\` - The URL for which to generate headers.
/// * \`config\` - Optional configuration settings.
///
/// # Returns
///
/// A \`Result\` containing a \`Header\` instance or a \`LlamaHeadersError\`.
pub fn get_header(url: &str, config: Option<Config>) -> Result<Header, LlamaHeadersError> {
    let config = config.unwrap_or_default();  // Use default config if none provided
    let domain = get_domain(url)?;
    let language = config.language.unwrap_or_else(|| utils::get_language_from_domain(&domain));
    let user_agent = match config.user_agent {
        Some(ua) => ua,
        None => UserAgent::get_random_user_agent(config.mobile.unwrap_or(false))?,
    };


    let referer = config.referer.unwrap_or_else(||get_random_referer(&language, &domain).unwrap_or(domain.clone()));
    let sec_ch_ua = get_sec_ch_ua(&user_agent);
    let accept_encoding = get_accept_encoding();
    let accept_language = get_accept_language(&language);

    let mut headers = HashMap::new();
    headers.insert("Host".to_string(), domain.clone());
    headers.insert("Connection".to_string(), get_connection());
    headers.insert("Upgrade-Insecure-Requests".to_string(), "1".to_string()); //Always 1 for simplicity
    headers.insert("User-Agent".to_string(), user_agent.to_string());
    headers.insert("Accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7".to_string()); // A common accept header
    headers.insert("Sec-Fetch-Site".to_string(), get_sec_fetch_site(&referer, &domain));
    headers.insert("Sec-Fetch-Mode".to_string(), get_sec_fetch_mode());
    headers.insert("Sec-Fetch-User".to_string(), get_sec_fetch_user());
    headers.insert("Sec-Fetch-Dest".to_string(), get_sec_fetch_dest());
    headers.insert("Referer".to_string(), referer);
     if let Some(sec_ch_ua_value) = sec_ch_ua {
         headers.insert("Sec-Ch-Ua".to_string(), sec_ch_ua_value.clone());
         if user_agent.is_mobile() {
             headers.insert("Sec-Ch-Ua-Mobile".to_string(), "?1".to_string());
         } else {
             headers.insert("Sec-Ch-Ua-Mobile".to_string(), "?0".to_string());
         }
          headers.insert("Sec-Ch-Ua-Platform".to_string(), format!("\\"{}\\"",user_agent.get_platform_for_sec_ch_ua()));
     }

    headers.insert("Accept-Encoding".to_string(), accept_encoding);
    headers.insert("Accept-Language".to_string(), accept_language);

    Ok(Header::new(user_agent, headers))
}



/// Generates multiple \`Header\` instances.
///
/// # Arguments
///
/// * \`url\` - The URL for which to generate headers.
/// * \`num\` - The number of \`Header\` instances to generate.
/// * \`config\` - Optional configuration settings.
///
/// # Returns
///
/// A \`Result\` containing a \`Vec<Header>\` or a \`LlamaHeadersError\`.
pub fn get_headers(url: &str, num: usize, config: Option<Config>) -> Result<Vec<Header>, LlamaHeadersError> {
    let mut headers_list = Vec::with_capacity(num);
    for _ in 0..num {
        headers_list.push(get_header(url, config.clone())?);
    }
    Ok(headers_list)
}
EOF
if [ $? -eq 0 ]; then
    green_echo "✓ src/lib.rs created."
else
    red_echo "✗ Failed to create src/lib.rs."
    exit 1
fi

# --- Create src/header.rs ---
yellow_echo "--- Creating src/header.rs ---"
cat <<EOF > src/header.rs
use crate::user_agent::UserAgent;
use std::collections::HashMap;


/// Represents a complete set of HTTP headers.
#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    pub user_agent: UserAgent,
    pub headers: HashMap<String, String>,
}

impl Header {
    /// Creates a new \`Header\` instance.
    pub fn new(user_agent: UserAgent, headers: HashMap<String, String>) -> Self {
        Header { user_agent, headers }
    }

    /// Returns the headers as a \`HashMap\`.
    pub fn get_map(&self) -> &HashMap<String, String> {
        &self.headers
    }
      /// Returns the value associated with a specific header key.
    pub fn get(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }
}
impl std::fmt::Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (key, value) in &self.headers {
            writeln!(f, "{}: {}", key, value)?;
        }
        Ok(())
    }
}
EOF
if [ $? -eq 0 ]; then
    green_echo "✓ src/header.rs created."
else
    red_echo "✗ Failed to create src/header.rs."
    exit 1
fi

# --- Create src/user_agent.rs ---
yellow_echo "--- Creating src/user_agent.rs ---"
cat <<EOF > src/user_agent.rs
use crate::errors::LlamaHeadersError;
use rand::seq::SliceRandom;
use rand::thread_rng;
use regex::Regex;
use lazy_static::lazy_static;


/// Represents a User-Agent string and its parsed components.
#[derive(Debug, Clone, PartialEq)]
pub struct UserAgent {
    pub string: String,
    pub browser: String,
    pub browser_version: String,
    pub os: String,
    pub os_version: String,
    mobile: bool,
}

impl UserAgent {
      /// Parses a User-Agent string and creates a \`UserAgent\` instance.
    pub fn parse(ua_string: &str) -> Result<Self, LlamaHeadersError> {
        lazy_static! {
            static ref RE_MOBILE: Regex = Regex::new(r"(Mobile|Tablet)").unwrap();

            //Regex for major browsers and OS, keep expanding
            static ref RE_BROWSER_OS: Regex = Regex::new(r"(Chrome|Firefox|Safari|Edge|MSIE|Opera|Trident)/(?P<version>\d+(\.\d+)*).*\((?P<os>[^;]+)").unwrap();

        }

          let mobile = RE_MOBILE.is_match(ua_string);
          let (browser, browser_version, os, os_version) = if let Some(caps) = RE_BROWSER_OS.captures(ua_string) {
              let browser = caps.get(1).map_or("", |m| m.as_str()).to_string();
              let version = caps.name("version").map_or("", |m| m.as_str()).to_string();
              let os_name = caps.name("os").map_or("", |m| m.as_str()).to_string();

              let os_parts: Vec<&str> = os_name.split_whitespace().collect();
              let mut extracted_os = "Other".to_string();
              let mut extracted_os_version = "".to_string();

               if !os_parts.is_empty() {
                    extracted_os = match os_parts[0] {
                        "Windows" => "Windows".to_string(),
                        "Macintosh" => "macOS".to_string(),
                        "Linux" => "Linux".to_string(),
                        "Android" => "Android".to_string(),
                        "iPhone" | "iPad" => "iOS".to_string(), // Consider both iPhone and iPad as iOS
                        _ => "Other".to_string(),
                      };

                      if extracted_os == "Windows" && os_parts.len() >= 3 {
                          extracted_os_version = os_parts[2].to_string();
                      } else if (extracted_os == "macOS" || extracted_os == "iOS" || extracted_os == "Android") && os_parts.len() >= 2{
                          extracted_os_version = os_parts[1].to_string();
                      }
                  }
                  (browser, version, extracted_os, extracted_os_version)
          } else {
              ("Other".to_string(), "0".to_string(), "Other".to_string(), "0".to_string())
          };

        Ok(UserAgent {
            string: ua_string.to_string(),
            browser,
            browser_version,
            os,
            os_version,
            mobile
        })
    }

    /// Returns \`true\` if the User-Agent represents a mobile device.
    pub fn is_mobile(&self) -> bool {
        self.mobile
    }


    /// Gets a random User-Agent string.
    pub fn get_random_user_agent(mobile: bool) -> Result<Self, LlamaHeadersError> {
        let user_agents = if mobile {
            vec![
                "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.0 Mobile/15E148 Safari/604.1",
                "Mozilla/5.0 (Linux; Android 12; Pixel 6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.87 Mobile Safari/537.36",
                // Add more mobile user agents here
            ]
        } else {
            vec![
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.102 Safari/537.36",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.0 Safari/605.1.15",
                "Mozilla/5.0 (X11; Linux x86_64; rv:97.0) Gecko/20100101 Firefox/97.0",
                // Add more desktop user agents here
            ]
        };

        let mut rng = thread_rng();
        let ua_string = user_agents.choose(&mut rng).ok_or(LlamaHeadersError::NoUserAgentAvailable)?.to_string();
        UserAgent::parse(&ua_string)

    }
        /// Returns the platform string suitable for the \`Sec-CH-UA-Platform\` header.
    pub fn get_platform_for_sec_ch_ua(&self) -> String {
        match self.os.as_str() {
            "Windows" => "Windows".to_string(),
            "macOS" => "macOS".to_string(),
            "Linux" => "Linux".to_string(),
            "Android" => "Android".to_string(),
            "iOS" => "iOS".to_string(), // Correctly handle iOS
            _ => "Unknown".to_string(),  // Provide a default value
        }
    }
}

impl std::fmt::Display for UserAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string)
    }
}
EOF
if [ $? -eq 0 ]; then
    green_echo "✓ src/user_agent.rs created."
else
    red_echo "✗ Failed to create src/user_agent.rs."
    exit 1
fi

# --- Create src/utils.rs ---
yellow_echo "--- Creating src/utils.rs ---"
cat <<EOF > src/utils.rs
use crate::errors::LlamaHeadersError;
use crate::user_agent::UserAgent;
use rand::seq::SliceRandom;
use rand::thread_rng;
use regex::Regex;
use lazy_static::lazy_static;
use std::collections::HashMap;


/// Extracts the domain from a URL.
pub fn get_domain(url: &str) -> Result<String, LlamaHeadersError> {
    lazy_static! {
        static ref RE_DOMAIN: Regex = Regex::new(r"https?://([^/]+)").unwrap();
    }
    RE_DOMAIN
        .captures(url)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or(LlamaHeadersError::InvalidUrl(url.to_string()))
}

pub fn get_language_from_domain(domain: &str) -> String {
      lazy_static! {
        static ref RE_TLD: Regex = Regex::new(r"\\.([a-z]{2,})$").unwrap();
    }

    if let Some(caps) = RE_TLD.captures(domain) {
        if let Some(tld) = caps.get(1) {
            match tld.as_str() {
                "de" => return "de-DE".to_string(),
                "fr" => return "fr-FR".to_string(),
                "jp" => return "ja-JP".to_string(),
                "uk" | "gb" => return "en-GB".to_string(), // Handle .uk and .gb
                // Add more country codes as needed
                _ => return "en-US".to_string(), // Default to English (US)
            }
        }
    }

    "en-US".to_string()
}

pub fn get_random_referer(language: &str, domain: &str) -> Result<String, LlamaHeadersError> {
      let mut referers = HashMap::new();
    referers.insert("en-US", vec!["https://www.google.com", "https://www.bing.com"]);
    referers.insert("de-DE", vec!["https://www.google.de", "https://www.bing.de"]);
    // Add more language-specific referers
      let referer_list = referers.get(language).ok_or(LlamaHeadersError::NoRefererAvailable)?;

      let mut rng = thread_rng();
    let referer = referer_list.choose(&mut rng).ok_or(LlamaHeadersError::NoRefererAvailable)?;
    Ok(referer.to_string())
}


pub fn get_sec_ch_ua(ua: &UserAgent) -> Option<String> {
    // Basic implementation; expand as needed
    if ua.browser == "Chrome" || ua.browser == "Edge" {
        Some(format!("\\\"Not A(Brand\\\";v=\\\"99\\\", \\\"{}\\\";v=\\\"{}\\\", \\\"Chromium\\\";v=\\\"{}\\\"", ua.browser, ua.browser_version, ua.browser_version))
    } else {
        None
    }
}
pub fn get_accept_encoding() -> String {
    "gzip, deflate, br".to_string()
}

pub fn get_accept_language(language: &str) -> String {
   format!("{},{};q=0.9", language, language.split('-').next().unwrap_or(""))
}

pub fn get_sec_fetch_site(referer: &str, domain: &str) -> String {
    if referer.contains(domain) {
        "same-origin".to_string()
    } else {
        "cross-site".to_string() //Simplified
    }
}
pub fn get_sec_fetch_mode() -> String {
    "navigate".to_string() // Simplification, can be expanded based on context
}

pub fn get_sec_fetch_user() -> String {
    "?1".to_string() // Simplification for interactive requests
}

pub fn get_sec_fetch_dest() -> String {
     "document".to_string() // Simplified for top-level navigation
}
pub fn get_connection() -> String {
    "keep-alive".to_string() // Common for persistent connections
}
EOF
if [ $? -eq 0 ]; then
    green_echo "✓ src/utils.rs created."
else
    red_echo "✗ Failed to create src/utils.rs."
    exit 1
fi

# --- Create src/errors.rs ---
yellow_echo "--- Creating src/errors.rs ---"
cat <<EOF > src/errors.rs
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum LlamaHeadersError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("No User-Agent available")]
    NoUserAgentAvailable,
    #[error("No referer available for the given language.")]
    NoRefererAvailable,
      #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),  // Now correctly uses #[from]
    #[error("Regex Error: {0}")]
    Regex(#[from] regex::Error),  // Add Regex error
    // Add more error types as needed
}
EOF
if [ $? -eq 0 ]; then
    green_echo "✓ src/errors.rs created."
else
    red_echo "✗ Failed to create src/errors.rs."
    exit 1
fi

# --- Create src/config.rs ---
yellow_echo "--- Creating src/config.rs ---"
cat <<EOF > src/config.rs
use crate::user_agent::UserAgent;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub language: Option<String>,
    pub user_agent: Option<UserAgent>,
    pub mobile: Option<bool>,
    pub referer: Option<String>,
      // Add other configurable options here
}

impl Default for Config {
    fn default() -> Self {
        Config {
            language: None,
            user_agent: None,
            mobile: Some(false),
            referer: None,
        }
    }
}

impl Config {
    pub fn load(config_path: Option<&str>) -> Result<Self, Box<dyn std::error::Error>> {
        let path_str = config_path.unwrap_or("config.toml"); // Default config path
        let path = Path::new(path_str);

        if path.exists() {
            let contents = fs::read_to_string(path)?;
            let config: Config = toml::from_str(&contents)?;
            println!("Loaded configuration from '{}'", path.display());
            Ok(config)
        } else {
             println!("Configuration file '{}' not found. Using default settings or command-line arguments.", path.display());
            Ok(Config::default()) // Return default config
        }
    }

}
EOF
if [ $? -eq 0 ]; then
    green_echo "✓ src/config.rs created."
else
    red_echo "✗ Failed to create src/config.rs."
    exit 1
fi

# --- Create tests/header_test.rs ---
yellow_echo "--- Creating tests/header_test.rs ---"
cat <<EOF > tests/header_test.rs
#[cfg(test)]
mod tests {
    use super::*;
    use llama_headers_rs::get_header;
    use llama_headers_rs::get_headers;
    use llama_headers_rs::errors::LlamaHeadersError;
    use llama_headers_rs::config::Config;
    use llama_headers_rs::user_agent::UserAgent;

 #[test]
    fn test_get_header_basic() {
        let url = "https://www.example.com";
        let header = get_header(url, None).unwrap();

        assert!(header.get("User-Agent").is_some());
        assert!(header.get("Host") == Some(&"www.example.com".to_string()));
        assert!(header.get("Accept-Language").is_some());
        // Add more assertions based on the expected headers
    }

    #[test]
    fn test_get_header_custom_ua() {
        let url = "https://www.example.com";
        let ua_string = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/99.0.1234.56 Safari/537.36";
        let custom_ua = UserAgent::parse(ua_string).unwrap();
        let config = Config {
            user_agent: Some(custom_ua.clone()),
            ..Default::default()
        };
        let header = get_header(url, Some(config)).unwrap();
        assert_eq!(header.get("User-Agent"), Some(&ua_string.to_string()));
    }

      #[test]
    fn test_get_header_invalid_url() {
        let invalid_url = "invalid-url";
        let result = get_header(invalid_url, None);
        assert!(matches!(result, Err(LlamaHeadersError::InvalidUrl(_))));
    }

    #[test]
    fn test_get_headers_multiple() {
        let url = "https://www.example.com";
        let num_headers = 3;
        let headers = get_headers(url, num_headers, None).unwrap();
        assert_eq!(headers.len(), num_headers);

        for header in headers {
            assert!(header.get("User-Agent").is_some());
             // Add more assertions as needed
        }
    }

     #[test]
    fn test_get_header_with_config() {
        let url = "https://www.example.de";
        let config = Config {
            language: Some("de-DE".to_string()),
            mobile: Some(true),
             ..Default::default()
        };

        let header = get_header(url, Some(config)).unwrap();
        assert_eq!(header.get("Accept-Language"), Some(&"de-DE,de;q=0.9".to_string())); //Check for german language
        assert!(header.user_agent.is_mobile()); // Check for mobile user agent
    }

    #[test]
    fn test_header_display_impl() {
        let url = "https://www.example.com";
        let header = get_header(url, None).unwrap();
        let header_string = format!("{}", header);  // Use the Display trait

        // Basic checks to ensure that some key headers are present in the output
        assert!(header_string.contains("User-Agent:"));
        assert!(header_string.contains("Host:"));
        assert!(header_string.contains("Accept-Language:"));
    }


}
EOF
if [ $? -eq 0 ]; then
    green_echo "✓ tests/header_test.rs created."
else
    red_echo "✗ Failed to create tests/header_test.rs."
    exit 1
fi

# --- Create tests/user_agent_test.rs ---
yellow_echo "--- Creating tests/user_agent_test.rs ---"
cat <<EOF > tests/user_agent_test.rs
#[cfg(test)]
mod user_agent_tests {
    use super::*;
    use llama_headers_rs::user_agent::UserAgent;
    use llama_headers_rs::errors::LlamaHeadersError;

    #[test]
    fn test_parse_valid_user_agent() {
        let ua_string = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.102 Safari/537.36";
        let ua = UserAgent::parse(ua_string).unwrap();
        assert_eq!(ua.string, ua_string);
        assert_eq!(ua.browser, "Chrome");
        assert_eq!(ua.browser_version, "98.0.4758.102");
        assert_eq!(ua.os, "Windows");
        assert_eq!(ua.os_version, "NT");
        assert!(!ua.is_mobile());
    }
     #[test]
    fn test_parse_mobile_user_agent() {
        let ua_string = "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.0 Mobile/15E148 Safari/604.1";
        let ua = UserAgent::parse(ua_string).unwrap();
        assert!(ua.is_mobile());
        assert_eq!(ua.os, "iOS"); //Check os
    }

      #[test]
    fn test_parse_invalid_user_agent() {
        let ua_string = "Invalid User Agent String";
        let result = UserAgent::parse(ua_string);
        // Expecting "Other" values for an invalid UA string
        if let Ok(ua) = result {
            assert_eq!(ua.browser, "Other");
            assert_eq!(ua.os, "Other");
        }
    }

    #[test]
    fn test_get_random_user_agent_desktop() {
        let ua = UserAgent::get_random_user_agent(false).unwrap();
        assert!(!ua.is_mobile());
    }

    #[test]
    fn test_get_random_user_agent_mobile() {
        let ua = UserAgent::get_random_user_agent(true).unwrap();
        assert!(ua.is_mobile());
    }

      #[test]
    fn test_get_platform_for_sec_ch_ua() {
        let ua_windows = UserAgent::parse("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/99.0.1234.56 Safari/537.36").unwrap();
        assert_eq!(ua_windows.get_platform_for_sec_ch_ua(), "Windows");

        let ua_macos = UserAgent::parse("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.0 Safari/605.1.15").unwrap();
        assert_eq!(ua_macos.get_platform_for_sec_ch_ua(), "macOS");

        let ua_linux = UserAgent::parse("Mozilla/5.0 (X11; Linux x86_64; rv:97.0) Gecko/20100101 Firefox/97.0").unwrap();
        assert_eq!(ua_linux.get_platform_for_sec_ch_ua(), "Linux");

        let ua_android = UserAgent::parse("Mozilla/5.0 (Linux; Android 12; Pixel 6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.87 Mobile Safari/537.36").unwrap();
        assert_eq!(ua_android.get_platform_for_sec_ch_ua(), "Android");

        let ua_ios = UserAgent::parse("Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.0 Mobile/15E148 Safari/604.1").unwrap();
        assert_eq!(ua_ios.get_platform_for_sec_ch_ua(), "iOS");

        //Test with an "Other" OS
        let ua_other = UserAgent::parse("Mozilla/5.0 (Unknown; Unknown) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/99.0.1234.56 Safari/537.36").unwrap();
        assert_eq!(ua_other.get_platform_for_sec_ch_ua(), "Unknown");
    }

    #[test]
    fn test_user_agent_display() {
        let ua_string = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.102 Safari/537.36";
        let ua = UserAgent::parse(ua_string).unwrap();
        assert_eq!(format!("{}", ua), ua_string);  // Check Display implementation
    }
}
EOF
if [ $? -eq 0 ]; then
    green_echo "✓ tests/user_agent_test.rs created."
else
    red_echo "✗ Failed to create tests/user_agent_test.rs."
    exit 1
fi

# --- Create tests/utils_test.rs ---
yellow_echo "--- Creating tests/utils_test.rs ---"
cat <<EOF > tests/utils_test.rs
#[cfg(test)]
mod utils_tests {
    use super::*;
    use llama_headers_rs::utils::*;
    use llama_headers_rs::errors::LlamaHeadersError;
    use llama_headers_rs::user_agent::UserAgent;


    #[test]
    fn test_get_domain_valid() {
        let url = "https://www.example.com/path?query=string";
        let domain = get_domain(url).unwrap();
        assert_eq!(domain, "www.example.com");
    }

    #[test]
    fn test_get_domain_invalid() {
        let url = "invalid-url";
        let result = get_domain(url);
        assert!(matches!(result, Err(LlamaHeadersError::InvalidUrl(_))));
    }

    #[test]
    fn test_get_language_from_domain_known() {
        assert_eq!(get_language_from_domain("example.de"), "de-DE");
        assert_eq!(get_language_from_domain("example.fr"), "fr-FR");
        assert_eq!(get_language_from_domain("example.co.uk"), "en-GB"); // Test .co.uk
    }

    #[test]
    fn test_get_language_from_domain_unknown() {
        assert_eq!(get_language_from_domain("example.com"), "en-US");
        assert_eq!(get_language_from_domain("example.org"), "en-US");
        assert_eq!(get_language_from_domain("example.unknown"), "en-US"); //Test with an unknown TLD

    }

      #[test]
    fn test_get_random_referer_known_language() {
        let referer = get_random_referer("en-US", "example.com").unwrap();
        assert!(referer == "https://www.google.com" || referer == "https://www.bing.com");
    }

    #[test]
    fn test_get_random_referer_unknown_language() {
        let result = get_random_referer("xx-XX", "example.com");
        assert!(matches!(result, Err(LlamaHeadersError::NoRefererAvailable)));
    }

        #[test]
    fn test_get_sec_ch_ua_chrome() {
        let ua = UserAgent::parse("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/99.0.1234.56 Safari/537.36").unwrap();
        let sec_ch_ua = get_sec_ch_ua(&ua).unwrap();
        assert!(sec_ch_ua.contains("Chrome"));
        assert!(sec_ch_ua.contains("99.0.1234.56")); // Check for version
    }

    #[test]
    fn test_get_sec_ch_ua_firefox() {
        let ua = UserAgent::parse("Mozilla/5.0 (X11; Linux x86_64; rv:97.0) Gecko/20100101 Firefox/97.0").unwrap();
        let sec_ch_ua = get_sec_ch_ua(&ua);
        assert!(sec_ch_ua.is_none());  // Should be None for Firefox
    }

      #[test]
    fn test_get_accept_encoding_value() {
        assert_eq!(get_accept_encoding(), "gzip, deflate, br");
    }

    #[test]
    fn test_get_accept_language_value() {
        assert_eq!(get_accept_language("en-US"), "en-US,en;q=0.9");
        assert_eq!(get_accept_language("de-DE"), "de-DE,de;q=0.9");
        assert_eq!(get_accept_language("fr"), "fr,fr;q=0.9"); // Test with only language code
    }

     #[test]
    fn test_get_sec_fetch_site_same_origin() {
        let referer = "https://www.example.com/page1";
        let domain = "www.example.com";
        assert_eq!(get_sec_fetch_site(referer, domain), "same-origin");
    }

    #[test]
    fn test_get_sec_fetch_site_cross_site() {
        let referer = "https://www.google.com";
        let domain = "www.example.com";
        assert_eq!(get_sec_fetch_site(referer, domain), "cross-site");
    }
     #[test]
    fn test_get_sec_fetch_mode_value() {
        assert_eq!(get_sec_fetch_mode(), "navigate");
    }

    #[test]
    fn test_get_sec_fetch_user_value() {
        assert_eq!(get_sec_fetch_user(), "?1");
    }

    #[test]
    fn test_get_sec_fetch_dest_value() {
        assert_eq!(get_sec_fetch_dest(), "document");
    }
      #[test]
    fn test_get_connection_value() {
        assert_eq!(get_connection(), "keep-alive");
    }
}
EOF
if [ $? -eq 0 ]; then
    green_echo "✓ tests/utils_test.rs created."
else
    red_echo "✗ Failed to create tests/utils_test.rs."
    exit 1
fi

# --- Build the crate ---
yellow_echo "--- Building ${CRATE_NAME} ---"
if cargo build 2> build_errors.log; then
    green_echo "✓ Build successful!"
else
    red_echo "✗ Build failed! Check build_errors.log for details."
    cat build_errors.log
    rm build_errors.log
    exit 1
fi
rm build_errors.log

# --- Run Tests ---
yellow_echo "--- Running tests ---"
if cargo test 2> test_errors.log; then
    green_echo "✓ All tests passed!"
else
    red_echo "✗ Some tests failed! Check test_errors.log for details."
    cat test_errors.log
    rm test_errors.log
    exit 1
fi
rm test_errors.log

# --- Confirmation Screen ---
green_echo ""
green_echo "-------------------------------------------------------------------"
blue_echo   "🎉🎉🎉 ${CRATE_NAME} Installation and Tests Successful! 🎉🎉🎉"
green_echo "-------------------------------------------------------------------"
green_echo "✅  ${CRATE_NAME} has been successfully built and tested on your macOS system."
green_echo "✅  All tests passed, indicating the library is functioning correctly."
green_echo ""
yellow_echo "As ${CRATE_NAME} is a library crate, there is no executable to run directly."
yellow_echo "You can use this library in your other Rust projects by adding it as a dependency in your Cargo.toml file:"
yellow_echo ""
yellow_echo "[dependencies]"
yellow_echo "${CRATE_NAME} = { path = './' }" # Assuming your crate is in the same directory
green_echo ""
green_echo "Ready to upload/publish ${CRATE_NAME} as a crate!"
green_echo "-------------------------------------------------------------------"
green_echo ""

exit 0