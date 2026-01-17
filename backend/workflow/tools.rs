//! Tool system for workflow nodes

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use crate::error::{Result, SynapseError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
}

#[async_trait]
pub trait Tool: Send + Sync {
    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value>;
    fn schema(&self) -> ToolSchema;
    fn name(&self) -> &str { "unknown" }
}

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut registry = Self { tools: HashMap::new() };
        registry.register(Box::new(HTTPTool::new()));
        registry.register(Box::new(TextTool::new()));
        registry
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) {
        let name = tool.schema().name.clone();
        self.tools.insert(name, Arc::from(tool));
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    pub async fn execute(&self, tool_name: &str, input: serde_json::Value) -> Result<serde_json::Value> {
        let tool = self.tools.get(tool_name)
            .ok_or_else(|| SynapseError::Validation(format!("Tool not found: {}", tool_name)))?;
        tool.execute(input).await
    }

    pub fn list_tools(&self) -> Vec<ToolSchema> {
        self.tools.values().map(|t| t.schema()).collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self { Self::new() }
}

pub struct HTTPTool { client: reqwest::Client }

impl HTTPTool {
    pub fn new() -> Self {
        Self { client: reqwest::Client::builder().timeout(std::time::Duration::from_secs(60)).build().unwrap() }
    }
}

#[async_trait]
impl Tool for HTTPTool {
    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value> {
        let method = input["method"].as_str().unwrap_or("GET");
        let url = input["url"].as_str().ok_or_else(|| SynapseError::Validation("Missing url".into()))?;
        
        let req = match method { "POST" => self.client.post(url), "PUT" => self.client.put(url), _ => self.client.get(url) };
        let resp = req.send().await.map_err(|e| SynapseError::Internal(e.to_string()))?;
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        
        Ok(serde_json::json!({"status": status, "body": body}))
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema { name: "http".into(), description: "HTTP requests".into(), input_schema: serde_json::json!({}), output_schema: serde_json::json!({}) }
    }
}

pub struct TextTool;
impl TextTool { pub fn new() -> Self { Self } }

#[async_trait]
impl Tool for TextTool {
    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value> {
        let text = input["text"].as_str().unwrap_or("");
        let op = input["operation"].as_str().unwrap_or("identity");
        let result = match op {
            "uppercase" => text.to_uppercase(),
            "lowercase" => text.to_lowercase(),
            "trim" => text.trim().to_string(),
            _ => text.to_string(),
        };
        Ok(serde_json::Value::String(result))
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema { name: "text".into(), description: "Text operations".into(), input_schema: serde_json::json!({}), output_schema: serde_json::json!({}) }
    }
}
