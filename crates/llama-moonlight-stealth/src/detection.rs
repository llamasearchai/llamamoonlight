//! Detection and testing of anti-bot systems
//!
//! This module provides capabilities to detect and test anti-bot systems
//! to evaluate the effectiveness of stealth techniques.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::Result;
use crate::Error;
use crate::evasion::StealthTarget;

/// Type of anti-bot detection test
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetectionTestType {
    /// Test for WebDriver properties
    WebDriver,
    
    /// Test for JavaScript automation properties
    AutomationProperties,
    
    /// Test for WebGL fingerprinting methods
    WebGLFingerprinting,
    
    /// Test for Canvas fingerprinting methods
    CanvasFingerprinting,
    
    /// Test for Audio fingerprinting methods
    AudioFingerprinting,
    
    /// Test for Font enumeration methods
    FontEnumeration,
    
    /// Test for navigator properties that might reveal automation
    NavigatorProperties,
    
    /// Test for browser plugins that might reveal automation
    PluginEnumeration,
    
    /// Test for CPU and hardware information
    HardwareInfo,
    
    /// Test for consistent fingerprints between sessions
    FingerprintConsistency,
    
    /// Test for request headers that might reveal automation
    RequestHeaders,
    
    /// Custom test with JavaScript code
    Custom(String),
}

/// Result of a detection test
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DetectionResult {
    /// Type of test
    pub test_type: DetectionTestType,
    
    /// Whether the test passed (no detection)
    pub passed: bool,
    
    /// Score for the test (0.0 - 1.0, higher is better)
    pub score: f64,
    
    /// Detailed information about the test
    pub details: HashMap<String, String>,
    
    /// Raw JavaScript result, if any
    pub raw_result: Option<String>,
    
    /// Duration of the test
    pub duration: Duration,
}

/// Test for detecting anti-bot systems
#[derive(Debug, Clone)]
pub struct DetectionTest {
    /// Type of test
    test_type: DetectionTestType,
    
    /// Javascript code to execute for the test
    js_code: String,
    
    /// Function to evaluate the result
    evaluator: fn(&str) -> (bool, f64, HashMap<String, String>),
}

impl DetectionTest {
    /// Create a new detection test
    pub fn new(
        test_type: DetectionTestType,
        js_code: &str,
        evaluator: fn(&str) -> (bool, f64, HashMap<String, String>),
    ) -> Self {
        Self {
            test_type,
            js_code: js_code.to_string(),
            evaluator,
        }
    }
    
    /// Create a WebDriver detection test
    pub fn webdriver() -> Self {
        Self::new(
            DetectionTestType::WebDriver,
            r#"
            (() => {
                const tests = [
                    // Test for navigator.webdriver property
                    {
                        name: 'navigator.webdriver',
                        result: navigator.webdriver === undefined || navigator.webdriver === false,
                        value: navigator.webdriver
                    },
                    // Test for Chrome's automation object
                    {
                        name: 'window.chrome.automation',
                        result: !window.chrome || window.chrome.automation === undefined,
                        value: window.chrome && window.chrome.automation
                    },
                    // Test for document.$cdc_ property (ChromeDriver)
                    {
                        name: 'document.$cdc_',
                        result: !Object.keys(document).some(key => key.startsWith('$cdc_')),
                        value: Object.keys(document).filter(key => key.startsWith('$cdc_')).join(',')
                    },
                    // Test for __webdriver_script_fn property
                    {
                        name: '__webdriver_script_fn',
                        result: !window.__webdriver_script_fn,
                        value: window.__webdriver_script_fn
                    },
                    // Test for _selenium property
                    {
                        name: '_selenium',
                        result: !window._selenium,
                        value: window._selenium
                    },
                    // Test for __selenium_unwrapped property
                    {
                        name: '__selenium_unwrapped',
                        result: !window.__selenium_unwrapped,
                        value: window.__selenium_unwrapped
                    },
                    // Test for __driver_evaluate property
                    {
                        name: '__driver_evaluate',
                        result: !window.__driver_evaluate,
                        value: window.__driver_evaluate
                    },
                    // Test for __webdriver_evaluate property
                    {
                        name: '__webdriver_evaluate',
                        result: !window.__webdriver_evaluate,
                        value: window.__webdriver_evaluate
                    }
                ];
                
                return JSON.stringify({
                    tests: tests,
                    passed: tests.every(test => test.result),
                    failedTests: tests.filter(test => !test.result).map(test => test.name)
                });
            })();
            "#,
            |result| {
                let parsed: serde_json::Value = serde_json::from_str(result).unwrap_or(serde_json::Value::Null);
                let mut details = HashMap::new();
                
                let passed = if let Some(passed) = parsed.get("passed").and_then(|v| v.as_bool()) {
                    passed
                } else {
                    false
                };
                
                // Calculate score based on number of passed tests
                let score = if let Some(tests) = parsed.get("tests").and_then(|v| v.as_array()) {
                    let total = tests.len() as f64;
                    let passed_count = tests.iter()
                        .filter(|t| t.get("result").and_then(|v| v.as_bool()).unwrap_or(false))
                        .count() as f64;
                    
                    passed_count / total
                } else {
                    0.0
                };
                
                // Add failed tests to details
                if let Some(failed_tests) = parsed.get("failedTests").and_then(|v| v.as_array()) {
                    details.insert("failed_tests".to_string(), 
                        failed_tests.iter()
                            .filter_map(|v| v.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                }
                
                (passed, score, details)
            },
        )
    }
    
    /// Create a Canvas fingerprinting detection test
    pub fn canvas_fingerprinting() -> Self {
        Self::new(
            DetectionTestType::CanvasFingerprinting,
            r#"
            (() => {
                // Create a canvas
                const canvas = document.createElement('canvas');
                canvas.width = 200;
                canvas.height = 50;
                
                const ctx = canvas.getContext('2d');
                
                // Draw some text with a specific font
                ctx.fillStyle = '#f60';
                ctx.font = '18pt Arial';
                ctx.textBaseline = 'alphabetic';
                ctx.fillText('Canvas Test: 🦙 !@#$%^&*()_+', 10, 30);
                
                // Add a different colored subpixel rectangle
                ctx.fillStyle = '#069';
                ctx.fillRect(100, 30, 80, 10);
                
                // Get the image data from the canvas
                let imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
                
                // Run the test multiple times
                const runs = 3;
                let hashes = [];
                
                for (let i = 0; i < runs; i++) {
                    // Hash the image data
                    let hash = 0;
                    for (let j = 0; j < imageData.data.length; j++) {
                        hash = ((hash << 5) - hash) + imageData.data[j];
                        hash = hash & hash; // Convert to 32bit integer
                    }
                    
                    hashes.push(hash);
                    
                    // Minor delay
                    for (let k = 0; k < 10000; k++) {}
                    
                    // Get image data again
                    imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
                }
                
                // Check if all hashes are identical
                const allSame = hashes.every(hash => hash === hashes[0]);
                
                // If the hashes are all identical, this likely isn't being protected
                // However, we want to check if they're *too* different, which might indicate
                // very aggressive randomization that could be detected
                
                // Calculate average difference between hashes
                let totalDiff = 0;
                let count = 0;
                
                for (let i = 0; i < hashes.length; i++) {
                    for (let j = i + 1; j < hashes.length; j++) {
                        totalDiff += Math.abs(hashes[i] - hashes[j]);
                        count++;
                    }
                }
                
                const avgDiff = count > 0 ? totalDiff / count : 0;
                
                // Too much difference is also bad
                const tooMuchDiff = avgDiff > 1000000;
                
                // If they're all the same, that's detectable
                // If they're too different, that's also detectable
                const passed = !allSame && !tooMuchDiff;
                
                // Calculate a score (1.0 is best)
                let score = 0.0;
                
                if (allSame) {
                    score = 0.0; // No protection
                } else if (tooMuchDiff) {
                    score = 0.3; // Too much randomization
                } else {
                    // Calculate score based on average difference
                    // Optimal range is between 1000-100000
                    if (avgDiff < 1000) {
                        score = avgDiff / 1000;
                    } else if (avgDiff > 100000) {
                        score = 1.0 - ((avgDiff - 100000) / 900000);
                        score = Math.max(0.3, score);
                    } else {
                        // Normalize score between 0.7 and 1.0
                        score = 0.7 + 0.3 * (avgDiff - 1000) / 99000;
                    }
                }
                
                return JSON.stringify({
                    hashes: hashes,
                    allSame: allSame,
                    avgDiff: avgDiff,
                    tooMuchDiff: tooMuchDiff,
                    passed: passed,
                    score: score
                });
            })();
            "#,
            |result| {
                let parsed: serde_json::Value = serde_json::from_str(result).unwrap_or(serde_json::Value::Null);
                let mut details = HashMap::new();
                
                let passed = parsed.get("passed").and_then(|v| v.as_bool()).unwrap_or(false);
                let score = parsed.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
                
                if let Some(all_same) = parsed.get("allSame").and_then(|v| v.as_bool()) {
                    details.insert("all_same".to_string(), all_same.to_string());
                }
                
                if let Some(avg_diff) = parsed.get("avgDiff").and_then(|v| v.as_f64()) {
                    details.insert("average_difference".to_string(), avg_diff.to_string());
                }
                
                if let Some(too_much_diff) = parsed.get("tooMuchDiff").and_then(|v| v.as_bool()) {
                    details.insert("too_much_difference".to_string(), too_much_diff.to_string());
                }
                
                (passed, score, details)
            },
        )
    }
    
    /// Run the test against a target
    pub async fn run(&self, target: &mut dyn StealthTarget) -> Result<DetectionResult> {
        let start = std::time::Instant::now();
        
        let result = target.execute_script(&self.js_code)?;
        
        let duration = start.elapsed();
        
        let (passed, score, details) = (self.evaluator)(&result);
        
        Ok(DetectionResult {
            test_type: self.test_type.clone(),
            passed,
            score,
            details,
            raw_result: Some(result),
            duration,
        })
    }
    
    /// Get the type of the test
    pub fn test_type(&self) -> &DetectionTestType {
        &self.test_type
    }
    
    /// Get the JavaScript code for the test
    pub fn js_code(&self) -> &str {
        &self.js_code
    }
}

/// Suite of detection tests to run together
#[derive(Debug, Default)]
pub struct DetectionTestSuite {
    /// Tests in the suite
    tests: Vec<DetectionTest>,
}

impl DetectionTestSuite {
    /// Create a new empty test suite
    pub fn new() -> Self {
        Self {
            tests: Vec::new(),
        }
    }
    
    /// Add a test to the suite
    pub fn add_test(&mut self, test: DetectionTest) -> &mut Self {
        self.tests.push(test);
        self
    }
    
    /// Add all standard tests to the suite
    pub fn with_standard_tests(mut self) -> Self {
        self.add_test(DetectionTest::webdriver());
        self.add_test(DetectionTest::canvas_fingerprinting());
        // Add more standard tests here as they are implemented
        self
    }
    
    /// Run all tests in the suite
    pub async fn run_all(&self, target: &mut dyn StealthTarget) -> Result<Vec<DetectionResult>> {
        let mut results = Vec::new();
        
        for test in &self.tests {
            let result = test.run(target).await?;
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// Get the overall score from the test results
    pub fn overall_score(results: &[DetectionResult]) -> f64 {
        if results.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = results.iter().map(|r| r.score).sum();
        sum / results.len() as f64
    }
    
    /// Check if all tests passed
    pub fn all_passed(results: &[DetectionResult]) -> bool {
        !results.is_empty() && results.iter().all(|r| r.passed)
    }
    
    /// Get the number of tests in the suite
    pub fn test_count(&self) -> usize {
        self.tests.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evasion::StealthTarget;
    use std::collections::HashMap;
    use llama_moonlight_headers::{BrowserType, DeviceType, PlatformType};
    
    #[derive(Debug)]
    struct MockStealthTarget {
        browser_type: BrowserType,
        device_type: DeviceType,
        platform_type: PlatformType,
        script_results: HashMap<String, String>,
    }
    
    impl MockStealthTarget {
        fn new() -> Self {
            let mut script_results = HashMap::new();
            
            // Mock result for webdriver test (passed)
            script_results.insert("webdriver".to_string(), r#"{"tests":[{"name":"navigator.webdriver","result":true,"value":false},{"name":"window.chrome.automation","result":true,"value":null},{"name":"document.$cdc_","result":true,"value":""},{"name":"__webdriver_script_fn","result":true,"value":null},{"name":"_selenium","result":true,"value":null},{"name":"__selenium_unwrapped","result":true,"value":null},{"name":"__driver_evaluate","result":true,"value":null},{"name":"__webdriver_evaluate","result":true,"value":null}],"passed":true,"failedTests":[]}"#);
            
            // Mock result for canvas fingerprinting test (passed)
            script_results.insert("canvas".to_string(), r#"{"hashes":[-1601234567,-1601234568,-1601234570],"allSame":false,"avgDiff":1.5,"tooMuchDiff":false,"passed":true,"score":0.8}"#);
            
            Self {
                browser_type: BrowserType::Chrome,
                device_type: DeviceType::Desktop,
                platform_type: PlatformType::Windows,
                script_results,
            }
        }
    }
    
    impl StealthTarget for MockStealthTarget {
        fn execute_script(&mut self, script: &str) -> Result<String> {
            // Return mock results based on the script type
            if script.contains("navigator.webdriver") {
                Ok(self.script_results.get("webdriver").cloned().unwrap_or_default())
            } else if script.contains("canvas") {
                Ok(self.script_results.get("canvas").cloned().unwrap_or_default())
            } else {
                Ok("{}".to_string())
            }
        }
        
        fn browser_type(&self) -> BrowserType {
            self.browser_type.clone()
        }
        
        fn device_type(&self) -> DeviceType {
            self.device_type.clone()
        }
        
        fn platform_type(&self) -> PlatformType {
            self.platform_type.clone()
        }
        
        fn set_header(&mut self, _name: &str, _value: &str) -> Result<()> {
            Ok(())
        }
        
        fn get_header(&self, _name: &str) -> Option<String> {
            None
        }
        
        fn remove_header(&mut self, _name: &str) -> Result<()> {
            Ok(())
        }
        
        fn intercept_requests(&mut self, _pattern: &str, _handler: crate::evasion::InterceptHandler) -> Result<()> {
            Ok(())
        }
        
        fn set_cookie(&mut self, _name: &str, _value: &str, _domain: &str) -> Result<()> {
            Ok(())
        }
    }
    
    #[tokio::test]
    async fn test_webdriver_detection() {
        let mut target = MockStealthTarget::new();
        let test = DetectionTest::webdriver();
        
        let result = test.run(&mut target).await.unwrap();
        
        assert_eq!(result.test_type, DetectionTestType::WebDriver);
        assert!(result.passed);
        assert_eq!(result.score, 1.0);
    }
    
    #[tokio::test]
    async fn test_canvas_fingerprinting_detection() {
        let mut target = MockStealthTarget::new();
        let test = DetectionTest::canvas_fingerprinting();
        
        let result = test.run(&mut target).await.unwrap();
        
        assert_eq!(result.test_type, DetectionTestType::CanvasFingerprinting);
        assert!(result.passed);
        assert_eq!(result.score, 0.8);
    }
    
    #[tokio::test]
    async fn test_detection_suite() {
        let mut target = MockStealthTarget::new();
        let suite = DetectionTestSuite::new()
            .with_standard_tests();
        
        let results = suite.run_all(&mut target).await.unwrap();
        
        assert_eq!(results.len(), 2);
        assert!(DetectionTestSuite::all_passed(&results));
        
        // The overall score should be the average of the individual scores
        // 1.0 (webdriver) + 0.8 (canvas) / 2 = 0.9
        assert_eq!(DetectionTestSuite::overall_score(&results), 0.9);
    }
} 