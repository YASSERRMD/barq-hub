//! Local LLM provider adapter (Ollama, llama.cpp, etc.)

use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use std::time::Instant;

use crate::{
    ChatRequest, ChatResponse, Choice, Message, Provider, TokenUsage,
    error::{ProviderError, Result, SynapseError},
};
use super::ProviderAdapter;

/// Adapter for local LLM servers (Ollama, llama.cpp server, vLLM)
pub struct LocalAdapter {
    provider: Provider,
    client: Client,
}

impl LocalAdapter {
    pub fn new(provider: Provider) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(300)) // Longer timeout for local
            .build()
            .expect("Failed to create HTTP client");

        Self { provider, client }
    }
}

#[async_trait]
impl ProviderAdapter for LocalAdapter {
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse> {
        let start = Instant::now();
        
        // Check if this is Ollama format or OpenAI-compatible format
        let is_ollama = self.provider.base_url.contains("11434") 
            || self.provider.base_url.contains("ollama");

        if is_ollama {
            self.chat_ollama(request, start).await
        } else {
            self.chat_openai_compatible(request, start).await
        }
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        let url = if self.provider.base_url.contains("11434") {
            format!("{}/api/tags", self.provider.base_url)
        } else {
            format!("{}/models", self.provider.base_url)
        };

        let response = self.client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| SynapseError::Provider(ProviderError::Network(e.to_string())))?;

        if !response.status().is_success() {
            return Ok(vec![]); // Return empty if we can't list
        }

        let body: serde_json::Value = response
            .json()
            .await
            .unwrap_or(serde_json::Value::Null);

        // Try Ollama format
        if let Some(models) = body["models"].as_array() {
            return Ok(models
                .iter()
                .filter_map(|m| m["name"].as_str().map(|s| s.to_string()))
                .collect());
        }

        // Try OpenAI format
        if let Some(data) = body["data"].as_array() {
            return Ok(data
                .iter()
                .filter_map(|m| m["id"].as_str().map(|s| s.to_string()))
                .collect());
        }

        Ok(vec![])
    }

    async fn health_check(&self) -> Result<bool> {
        let url = if self.provider.base_url.contains("11434") {
            format!("{}/", self.provider.base_url)
        } else {
            format!("{}/health", self.provider.base_url)
        };

        let response = self.client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await;

        Ok(response.map(|r| r.status().is_success()).unwrap_or(false))
    }

    fn provider(&self) -> &Provider {
        &self.provider
    }
}

impl LocalAdapter {
    /// Handle Ollama-specific API format
    async fn chat_ollama(&self, request: &ChatRequest, start: Instant) -> Result<ChatResponse> {
        let url = format!("{}/api/chat", self.provider.base_url);

        let messages: Vec<serde_json::Value> = request.messages
            .iter()
            .map(|m| serde_json::json!({
                "role": m.role,
                "content": m.content
            }))
            .collect();

        let payload = serde_json::json!({
            "model": request.model,
            "messages": messages,
            "stream": false,
            "options": {
                "temperature": request.temperature,
                "num_predict": request.max_tokens as i32,
            }
        });

        let response = self.client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| SynapseError::Provider(ProviderError::Network(e.to_string())))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SynapseError::Provider(ProviderError::RequestFailed(error_text)));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| SynapseError::Provider(ProviderError::InvalidResponse(e.to_string())))?;

        let content = body["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let choices = vec![Choice {
            index: 0,
            message: Message {
                role: "assistant".to_string(),
                content,
                function_call: None,
                tool_calls: None,
            },
            finish_reason: "stop".to_string(),
        }];

        // Ollama provides eval_count and prompt_eval_count
        let usage = TokenUsage {
            prompt_tokens: body["prompt_eval_count"].as_u64().unwrap_or(0) as u32,
            completion_tokens: body["eval_count"].as_u64().unwrap_or(0) as u32,
            total_tokens: (body["prompt_eval_count"].as_u64().unwrap_or(0)
                + body["eval_count"].as_u64().unwrap_or(0)) as u32,
        };

        let latency_ms = start.elapsed().as_millis() as u64;

        Ok(ChatResponse {
            id: uuid::Uuid::new_v4().to_string(),
            provider: self.provider.name.clone(),
            model: request.model.clone(),
            choices,
            usage,
            created: Utc::now(),
            latency_ms,
            cost: 0.0, // Local models are free
        })
    }

    /// Handle OpenAI-compatible local servers (vLLM, llama.cpp server)
    async fn chat_openai_compatible(&self, request: &ChatRequest, start: Instant) -> Result<ChatResponse> {
        let url = format!("{}/chat/completions", self.provider.base_url);

        let payload = serde_json::json!({
            "model": request.model,
            "messages": request.messages.iter().map(|m| {
                serde_json::json!({
                    "role": m.role,
                    "content": m.content
                })
            }).collect::<Vec<_>>(),
            "temperature": request.temperature,
            "max_tokens": request.max_tokens,
        });

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| SynapseError::Provider(ProviderError::Network(e.to_string())))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SynapseError::Provider(ProviderError::RequestFailed(error_text)));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| SynapseError::Provider(ProviderError::InvalidResponse(e.to_string())))?;

        let choices: Vec<Choice> = body["choices"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .enumerate()
            .map(|(i, choice)| Choice {
                index: i,
                message: Message {
                    role: choice["message"]["role"]
                        .as_str()
                        .unwrap_or("assistant")
                        .to_string(),
                    content: choice["message"]["content"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    function_call: None,
                    tool_calls: None,
                },
                finish_reason: choice["finish_reason"]
                    .as_str()
                    .unwrap_or("stop")
                    .to_string(),
            })
            .collect();

        let usage = TokenUsage {
            prompt_tokens: body["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: body["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: body["usage"]["total_tokens"].as_u64().unwrap_or(0) as u32,
        };

        let latency_ms = start.elapsed().as_millis() as u64;

        Ok(ChatResponse {
            id: body["id"].as_str().unwrap_or("").to_string(),
            provider: self.provider.name.clone(),
            model: request.model.clone(),
            choices,
            usage,
            created: Utc::now(),
            latency_ms,
            cost: 0.0, // Local models are free
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ProviderPricing, ProviderType, ProviderHealth};

    fn test_provider() -> Provider {
        Provider {
            id: "test-local".to_string(),
            name: "Local Ollama".to_string(),
            provider_type: ProviderType::Local,
            api_key: "".to_string(),
            base_url: "http://localhost:11434".to_string(),
            pricing: ProviderPricing::default(),
            enabled: true,
            health: ProviderHealth::default(),
            headers: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_adapter_creation() {
        let provider = test_provider();
        let adapter = LocalAdapter::new(provider.clone());
        assert_eq!(adapter.name(), "Local Ollama");
    }
}
