use crate::TestUtilError;
use llama_moonlight_core::{Browser, BrowserContext, Page};
use pretty_assertions::assert_eq;
use std::{path::Path, time::Duration};
use serde_json::Value;

/// Asserts that a page's title contains the expected string
pub async fn assert_title_contains(page: &Page, expected: &str) -> Result<(), TestUtilError> {
    let title = page.title().await.map_err(TestUtilError::from)?;
    
    if !title.contains(expected) {
        return Err(TestUtilError::AssertionError(format!(
            "Title '{}' does not contain '{}'",
            title, expected
        )));
    }
    
    Ok(())
}

/// Asserts that a page's URL equals the expected URL
pub async fn assert_url_equals(page: &Page, expected: &str) -> Result<(), TestUtilError> {
    let url = page.url().await.map_err(TestUtilError::from)?;
    
    if url != expected {
        return Err(TestUtilError::AssertionError(format!(
            "URL '{}' does not equal '{}'",
            url, expected
        )));
    }
    
    Ok(())
}

/// Asserts that a page's URL contains the expected string
pub async fn assert_url_contains(page: &Page, expected: &str) -> Result<(), TestUtilError> {
    let url = page.url().await.map_err(TestUtilError::from)?;
    
    if !url.contains(expected) {
        return Err(TestUtilError::AssertionError(format!(
            "URL '{}' does not contain '{}'",
            url, expected
        )));
    }
    
    Ok(())
}

/// Asserts that a page contains an element matching the selector
pub async fn assert_element_exists(page: &Page, selector: &str) -> Result<(), TestUtilError> {
    let element = page.query_selector(selector).await.map_err(TestUtilError::from)?;
    
    if element.is_none() {
        return Err(TestUtilError::AssertionError(format!(
            "Element with selector '{}' not found",
            selector
        )));
    }
    
    Ok(())
}

/// Asserts that a page does not contain an element matching the selector
pub async fn assert_element_not_exists(page: &Page, selector: &str) -> Result<(), TestUtilError> {
    let element = page.query_selector(selector).await.map_err(TestUtilError::from)?;
    
    if element.is_some() {
        return Err(TestUtilError::AssertionError(format!(
            "Element with selector '{}' found but expected not to exist",
            selector
        )));
    }
    
    Ok(())
}

/// Asserts that an element's text content contains the expected string
pub async fn assert_text_contains(page: &Page, selector: &str, expected: &str) -> Result<(), TestUtilError> {
    let element = page.query_selector(selector).await.map_err(TestUtilError::from)?;
    
    let element = element.ok_or_else(|| TestUtilError::AssertionError(format!(
        "Element with selector '{}' not found",
        selector
    )))?;
    
    let text = element.text_content().await.map_err(TestUtilError::from)?;
    
    if !text.contains(expected) {
        return Err(TestUtilError::AssertionError(format!(
            "Text content '{}' does not contain '{}'",
            text, expected
        )));
    }
    
    Ok(())
}

/// Asserts that an element's text content equals the expected string, ignoring whitespace
pub async fn assert_text_equals(page: &Page, selector: &str, expected: &str) -> Result<(), TestUtilError> {
    let element = page.query_selector(selector).await.map_err(TestUtilError::from)?;
    
    let element = element.ok_or_else(|| TestUtilError::AssertionError(format!(
        "Element with selector '{}' not found",
        selector
    )))?;
    
    let text = element.text_content().await.map_err(TestUtilError::from)?;
    let text = text.trim();
    let expected = expected.trim();
    
    if text != expected {
        return Err(TestUtilError::AssertionError(format!(
            "Text content '{}' does not equal '{}'",
            text, expected
        )));
    }
    
    Ok(())
}

/// Asserts that an element has a specific attribute with a specific value
pub async fn assert_attribute_equals(page: &Page, selector: &str, attribute: &str, expected: &str) -> Result<(), TestUtilError> {
    let element = page.query_selector(selector).await.map_err(TestUtilError::from)?;
    
    let element = element.ok_or_else(|| TestUtilError::AssertionError(format!(
        "Element with selector '{}' not found",
        selector
    )))?;
    
    let attr_value = element.get_attribute(attribute).await.map_err(TestUtilError::from)?;
    
    let attr_value = attr_value.ok_or_else(|| TestUtilError::AssertionError(format!(
        "Attribute '{}' not found on element with selector '{}'",
        attribute, selector
    )))?;
    
    if attr_value != expected {
        return Err(TestUtilError::AssertionError(format!(
            "Attribute '{}' value '{}' does not equal '{}'",
            attribute, attr_value, expected
        )));
    }
    
    Ok(())
}

/// Asserts that a JavaScript expression evaluates to true
pub async fn assert_js_expression(page: &Page, expression: &str) -> Result<(), TestUtilError> {
    let result: bool = page.evaluate(expression).await.map_err(TestUtilError::from)?;
    
    if !result {
        return Err(TestUtilError::AssertionError(format!(
            "JavaScript expression '{}' evaluated to false",
            expression
        )));
    }
    
    Ok(())
}

/// Asserts that a JSON value has a specific path and value
pub fn assert_json_path_equals(json: &Value, path: &str, expected: &str) -> Result<(), TestUtilError> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = json;
    
    for part in &parts {
        if let Some(index) = part.parse::<usize>().ok() {
            if let Some(array) = current.as_array() {
                if index < array.len() {
                    current = &array[index];
                } else {
                    return Err(TestUtilError::AssertionError(format!(
                        "Array index {} out of bounds in path '{}'",
                        index, path
                    )));
                }
            } else {
                return Err(TestUtilError::AssertionError(format!(
                    "Expected array at path '{}'",
                    path
                )));
            }
        } else {
            if let Some(obj) = current.as_object() {
                if let Some(value) = obj.get(*part) {
                    current = value;
                } else {
                    return Err(TestUtilError::AssertionError(format!(
                        "Key '{}' not found in object at path '{}'",
                        part, path
                    )));
                }
            } else {
                return Err(TestUtilError::AssertionError(format!(
                    "Expected object at path '{}'",
                    path
                )));
            }
        }
    }
    
    let actual = current.as_str().ok_or_else(|| TestUtilError::AssertionError(format!(
        "Value at path '{}' is not a string",
        path
    )))?;
    
    if actual != expected {
        return Err(TestUtilError::AssertionError(format!(
            "Value at path '{}' is '{}', expected '{}'",
            path, actual, expected
        )));
    }
    
    Ok(())
}

/// Asserts that a file exists at the specified path
pub fn assert_file_exists(path: &Path) -> Result<(), TestUtilError> {
    if !path.exists() {
        return Err(TestUtilError::AssertionError(format!(
            "File does not exist at path '{}'",
            path.display()
        )));
    }
    
    if !path.is_file() {
        return Err(TestUtilError::AssertionError(format!(
            "Path '{}' exists but is not a file",
            path.display()
        )));
    }
    
    Ok(())
}

/// Wait for an element to appear with a timeout
pub async fn wait_for_element(page: &Page, selector: &str, timeout_ms: u64) -> Result<(), TestUtilError> {
    let start = std::time::Instant::now();
    let timeout = Duration::from_millis(timeout_ms);
    
    loop {
        let element = page.query_selector(selector).await.map_err(TestUtilError::from)?;
        
        if element.is_some() {
            return Ok(());
        }
        
        if start.elapsed() > timeout {
            return Err(TestUtilError::TimeoutError);
        }
        
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// Wait for a specific text to appear with a timeout
pub async fn wait_for_text(page: &Page, selector: &str, text: &str, timeout_ms: u64) -> Result<(), TestUtilError> {
    let start = std::time::Instant::now();
    let timeout = Duration::from_millis(timeout_ms);
    
    loop {
        let element = page.query_selector(selector).await.map_err(TestUtilError::from)?;
        
        if let Some(element) = element {
            let content = element.text_content().await.map_err(TestUtilError::from)?;
            if content.contains(text) {
                return Ok(());
            }
        }
        
        if start.elapsed() > timeout {
            return Err(TestUtilError::TimeoutError);
        }
        
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// Wait for a page navigation to complete
pub async fn wait_for_navigation(page: &Page, url_contains: &str, timeout_ms: u64) -> Result<(), TestUtilError> {
    let start = std::time::Instant::now();
    let timeout = Duration::from_millis(timeout_ms);
    
    loop {
        let current_url = page.url().await.map_err(TestUtilError::from)?;
        
        if current_url.contains(url_contains) {
            return Ok(());
        }
        
        if start.elapsed() > timeout {
            return Err(TestUtilError::TimeoutError);
        }
        
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
} 