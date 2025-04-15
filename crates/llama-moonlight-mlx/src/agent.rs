use crate::MlxError;
use llama_moonlight_core::Page;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Name of the agent
    pub name: String,
    
    /// Agent description
    pub description: Option<String>,
    
    /// Maximum number of actions the agent can take
    pub max_actions: Option<usize>,
    
    /// Text model to use for reasoning (model name)
    pub text_model: Option<String>,
    
    /// Vision model to use for image processing (model name)
    pub vision_model: Option<String>,
    
    /// Prompt template for the agent
    pub prompt_template: Option<String>,
    
    /// Whether to take screenshots for observations
    pub use_screenshots: Option<bool>,
    
    /// Whether to extract text from the page for observations
    pub extract_text: Option<bool>,
    
    /// Available actions for the agent
    pub available_actions: Option<Vec<String>>,
    
    /// Memory capacity (number of past interactions to remember)
    pub memory_capacity: Option<usize>,
    
    /// Custom parameters
    #[serde(flatten)]
    pub custom_params: std::collections::HashMap<String, serde_json::Value>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            name: "BrowserAgent".to_string(),
            description: Some("A browser automation agent".to_string()),
            max_actions: Some(10),
            text_model: None,
            vision_model: None,
            prompt_template: None,
            use_screenshots: Some(true),
            extract_text: Some(true),
            available_actions: Some(vec![
                "click".to_string(),
                "type".to_string(),
                "navigate".to_string(),
                "wait".to_string(),
                "extract".to_string(),
            ]),
            memory_capacity: Some(5),
            custom_params: std::collections::HashMap::new(),
        }
    }
}

/// Agent action type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionType {
    /// Click on an element
    Click,
    /// Type text into an element
    Type,
    /// Navigate to a URL
    Navigate,
    /// Wait for an element or time
    Wait,
    /// Extract data from the page
    Extract,
    /// Custom action
    Custom(String),
}

/// Agent action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAction {
    /// Action type
    pub action_type: ActionType,
    
    /// Action parameters (depends on action type)
    pub parameters: serde_json::Value,
    
    /// Reason for taking this action
    pub reason: Option<String>,
    
    /// Whether the action was successful
    pub success: Option<bool>,
    
    /// Error message if the action failed
    pub error: Option<String>,
}

/// Agent observation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentObservation {
    /// Page title
    pub title: String,
    
    /// Page URL
    pub url: String,
    
    /// Page content (HTML or text)
    pub content: Option<String>,
    
    /// Screenshot path
    pub screenshot: Option<PathBuf>,
    
    /// Visible elements with their properties
    pub elements: Option<Vec<ElementInfo>>,
    
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Information about an element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementInfo {
    /// Element tag
    pub tag: String,
    
    /// Element ID
    pub id: Option<String>,
    
    /// Element classes
    pub classes: Option<Vec<String>>,
    
    /// Element text content
    pub text: Option<String>,
    
    /// Whether the element is visible
    pub visible: bool,
    
    /// Element bounding box
    pub bounding_box: Option<BoundingBox>,
    
    /// Element attributes
    pub attributes: std::collections::HashMap<String, String>,
}

/// Bounding box
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    /// X coordinate
    pub x: f32,
    /// Y coordinate
    pub y: f32,
    /// Width
    pub width: f32,
    /// Height
    pub height: f32,
}

/// Agent for autonomous browser automation
pub struct Agent {
    /// Agent configuration
    config: AgentConfig,
    
    /// Page that the agent is controlling
    page: Arc<Page>,
    
    /// Agent memory (previous actions and observations)
    memory: Vec<(AgentAction, AgentObservation)>,
    
    /// Current observation
    current_observation: Option<AgentObservation>,
    
    /// Number of actions taken
    action_count: usize,
}

impl Agent {
    /// Create a new agent
    pub fn new(config: AgentConfig, page: Arc<Page>) -> Self {
        Self {
            config,
            page,
            memory: Vec::new(),
            current_observation: None,
            action_count: 0,
        }
    }
    
    /// Get the agent's configuration
    pub fn config(&self) -> &AgentConfig {
        &self.config
    }
    
    /// Get the page that the agent is controlling
    pub fn page(&self) -> Arc<Page> {
        self.page.clone()
    }
    
    /// Get the agent's memory
    pub fn memory(&self) -> &[(AgentAction, AgentObservation)] {
        &self.memory
    }
    
    /// Get the current observation
    pub fn current_observation(&self) -> Option<&AgentObservation> {
        self.current_observation.as_ref()
    }
    
    /// Observe the current page state
    pub async fn observe(&mut self) -> Result<AgentObservation, MlxError> {
        let title = self.page.title().await.map_err(|e| MlxError::Agent(format!(
            "Failed to get page title: {}", e
        )))?;
        
        let url = self.page.url().await.map_err(|e| MlxError::Agent(format!(
            "Failed to get page URL: {}", e
        )))?;
        
        let mut observation = AgentObservation {
            title,
            url,
            content: None,
            screenshot: None,
            elements: None,
            timestamp: chrono::Utc::now(),
        };
        
        // Extract text content if configured
        if self.config.extract_text.unwrap_or(true) {
            observation.content = Some(self.page.content().await.map_err(|e| MlxError::Agent(format!(
                "Failed to get page content: {}", e
            )))?);
        }
        
        // Take screenshot if configured
        if self.config.use_screenshots.unwrap_or(true) {
            let screenshot_path = std::env::temp_dir().join(format!("agent_screenshot_{}.png", chrono::Utc::now().timestamp()));
            self.page.screenshot(screenshot_path.to_str().unwrap()).await.map_err(|e| MlxError::Agent(format!(
                "Failed to take screenshot: {}", e
            )))?;
            observation.screenshot = Some(screenshot_path);
        }
        
        // Extract visible elements
        // In a real implementation, this would extract elements from the page
        // For now, we'll just create a placeholder
        observation.elements = Some(vec![
            ElementInfo {
                tag: "div".to_string(),
                id: Some("main".to_string()),
                classes: Some(vec!["container".to_string()]),
                text: Some("Main content".to_string()),
                visible: true,
                bounding_box: Some(BoundingBox {
                    x: 0.0,
                    y: 0.0,
                    width: 100.0,
                    height: 100.0,
                }),
                attributes: std::collections::HashMap::new(),
            }
        ]);
        
        self.current_observation = Some(observation.clone());
        Ok(observation)
    }
    
    /// Plan the next action based on the current observation
    pub async fn plan_action(&self) -> Result<AgentAction, MlxError> {
        // In a real implementation, this would use the text model to generate an action
        // For now, we'll just create a placeholder action
        let action = AgentAction {
            action_type: ActionType::Click,
            parameters: serde_json::json!({
                "selector": "#main"
            }),
            reason: Some("Clicking on the main content".to_string()),
            success: None,
            error: None,
        };
        
        Ok(action)
    }
    
    /// Execute an action
    pub async fn execute_action(&mut self, action: AgentAction) -> Result<AgentAction, MlxError> {
        let mut result = action.clone();
        
        match action.action_type {
            ActionType::Click => {
                let selector = action.parameters["selector"].as_str().ok_or_else(|| {
                    MlxError::Agent("Missing selector parameter for click action".to_string())
                })?;
                
                match self.page.click(selector).await {
                    Ok(_) => {
                        result.success = Some(true);
                    }
                    Err(e) => {
                        result.success = Some(false);
                        result.error = Some(format!("Failed to click on selector '{}': {}", selector, e));
                    }
                }
            }
            ActionType::Type => {
                let selector = action.parameters["selector"].as_str().ok_or_else(|| {
                    MlxError::Agent("Missing selector parameter for type action".to_string())
                })?;
                
                let text = action.parameters["text"].as_str().ok_or_else(|| {
                    MlxError::Agent("Missing text parameter for type action".to_string())
                })?;
                
                match self.page.type_text(selector, text).await {
                    Ok(_) => {
                        result.success = Some(true);
                    }
                    Err(e) => {
                        result.success = Some(false);
                        result.error = Some(format!("Failed to type text into selector '{}': {}", selector, e));
                    }
                }
            }
            ActionType::Navigate => {
                let url = action.parameters["url"].as_str().ok_or_else(|| {
                    MlxError::Agent("Missing URL parameter for navigate action".to_string())
                })?;
                
                match self.page.goto(url).await {
                    Ok(_) => {
                        result.success = Some(true);
                    }
                    Err(e) => {
                        result.success = Some(false);
                        result.error = Some(format!("Failed to navigate to URL '{}': {}", url, e));
                    }
                }
            }
            ActionType::Wait => {
                let milliseconds = action.parameters["milliseconds"].as_u64().unwrap_or(1000);
                tokio::time::sleep(std::time::Duration::from_millis(milliseconds)).await;
                result.success = Some(true);
            }
            ActionType::Extract => {
                let selector = action.parameters["selector"].as_str().ok_or_else(|| {
                    MlxError::Agent("Missing selector parameter for extract action".to_string())
                })?;
                
                // In a real implementation, this would extract data from the page
                // For now, just set success to true
                result.success = Some(true);
            }
            ActionType::Custom(ref name) => {
                // Custom actions are not implemented in this placeholder
                result.success = Some(false);
                result.error = Some(format!("Custom action '{}' is not implemented", name));
            }
        }
        
        // Update action count
        self.action_count += 1;
        
        // If we have a current observation, add the action-observation pair to memory
        if let Some(observation) = self.current_observation.clone() {
            // Add to memory
            self.memory.push((result.clone(), observation));
            
            // Trim memory if it exceeds capacity
            if let Some(capacity) = self.config.memory_capacity {
                if self.memory.len() > capacity {
                    self.memory.remove(0);
                }
            }
        }
        
        Ok(result)
    }
    
    /// Run the agent autonomously
    pub async fn run(&mut self) -> Result<Vec<(AgentAction, AgentObservation)>, MlxError> {
        let max_actions = self.config.max_actions.unwrap_or(10);
        
        for _ in 0..max_actions {
            // Observe
            let observation = self.observe().await?;
            
            // Plan action
            let action = self.plan_action().await?;
            
            // Execute action
            let result = self.execute_action(action).await?;
            
            // Stop if action failed
            if let Some(false) = result.success {
                break;
            }
        }
        
        Ok(self.memory.clone())
    }
} 