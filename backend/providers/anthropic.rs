//! Anthropic Claude provider adapter

use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use std::time::Instant;

use crate::{
    ChatRequest, ChatResponse, Choice, Message, Provider, TokenUsage,
    error::{ProviderError, Result, SynapseError},
};
use super::ProviderAdapter;

/// Adapter for Anthropic Claude API
pub struct AnthropicAdapter {
    provider: Provider,
    client: Client,
}

impl AnthropicAdapter {
    pub fn new(provider: Provider) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .expect("Failed to create HTTP client");

        Self { provider, client }
    }

    /// Convert OpenAI-style messages to Anthropic format
    fn convert_messages(&self, messages: &[Message]) -> (Option<String>, Vec<serde_json::Value>) {
        let mut system_prompt = None;
        let mut anthropic_messages = Vec::new();

        for msg in messages {
            if msg.role == "system" {
                system_prompt = Some(msg.content.clone());
            } else {
                anthropic_messages.push(serde_json::json!({
                    "role": msg.role,
                    "content": msg.content
                }));
            }
        }

        (system_prompt, anthropic_messages)
    }
}

#[async_trait]
impl ProviderAdapter for AnthropicAdapter {
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse> {
        let start = Instant::now();
        let url = format!("{}/messages", self.provider.base_url);

        // Convert messages
        let (system_prompt, messages) = self.convert_messages(&request.messages);

        // Build request payload
        let mut payload = serde_json::json!({
            "model": request.model,
            "messages": messages,
            "max_tokens": request.max_tokens,
            "temperature": request.temperature,
        });

        if let Some(system) = system_prompt {
            payload["system"] = serde_json::Value::String(system);
        }

        if let Some(top_p) = request.top_p {
            payload["top_p"] = serde_json::Value::from(top_p);
        }

        if let Some(ref stop) = request.stop {
            payload["stop_sequences"] = serde_json::Value::from(stop.clone());
        }

        // Make request
        let mut req = self.client
            .post(&url)
            .header("x-api-key", &self.provider.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json");

        // Add custom headers
        for (key, value) in &self.provider.headers {
            req = req.header(key, value);
        }

        let response = req
            .json(&payload)
            .send()
            .await
            .map_err(|e| SynapseError::Provider(ProviderError::Network(e.to_string())))?;

        // Check status
        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(SynapseError::Provider(ProviderError::RateLimited));
        }

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(SynapseError::Provider(ProviderError::AuthFailed));
        }

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(SynapseError::Provider(ProviderError::RequestFailed(
                format!("Status: {}, Body: {}", status, error_text)
            )));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| SynapseError::Provider(ProviderError::InvalidResponse(e.to_string())))?;

        // Parse response - Anthropic returns content differently
        let content = body["content"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|c| c["text"].as_str())
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
            finish_reason: body["stop_reason"]
                .as_str()
                .unwrap_or("end_turn")
                .to_string(),
        }];

        let usage = TokenUsage {
            prompt_tokens: body["usage"]["input_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: body["usage"]["output_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: (body["usage"]["input_tokens"].as_u64().unwrap_or(0)
                + body["usage"]["output_tokens"].as_u64().unwrap_or(0)) as u32,
        };

        let latency_ms = start.elapsed().as_millis() as u64;
        let cost = self.calculate_cost(&usage);

        Ok(ChatResponse {
            id: body["id"].as_str().unwrap_or("").to_string(),
            provider: self.provider.name.clone(),
            model: request.model.clone(),
            choices,
            usage,
            created: Utc::now(),
            latency_ms,
            cost,
        })
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        Ok(vec![
            "claude-3-opus-20240229".to_string(),
            "claude-3-sonnet-20240229".to_string(),
            "claude-3-haiku-20240307".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            "claude-3-5-haiku-20241022".to_string(),
        ])
    }

    async fn health_check(&self) -> Result<bool> {
        // Anthropic doesn't have a simple health endpoint, so we check if we can reach it
        let url = format!("{}/messages", self.provider.base_url);
        
        let response = self.client
            .post(&url)
            .header("x-api-key", &self.provider.api_key)
            .header("anthropic-version", "2023-06-01")
            .timeout(std::time::Duration::from_secs(10))
            .json(&serde_json::json!({
                "model": "claude-3-haiku-20240307",
                "messages": [{"role": "user", "content": "ping"}],
                "max_tokens": 1
            }))
            .send()
            .await;

        Ok(response.map(|r| r.status().is_success() || r.status().as_u16() == 400).unwrap_or(false))
    }

    fn provider(&self) -> &Provider {
        &self.provider
    }
}

impl AnthropicAdapter {
    fn calculate_cost(&self, usage: &TokenUsage) -> f64 {
        let input_cost = (usage.prompt_tokens as f64 / 1_000_000.0) 
            * self.provider.pricing.input_token_cost;
        let output_cost = (usage.completion_tokens as f64 / 1_000_000.0) 
            * self.provider.pricing.output_token_cost;
        input_cost + output_cost
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ProviderPricing, ProviderType, ProviderHealth};

    fn test_provider() -> Provider {
        Provider {
            id: "test-anthropic".to_string(),
            name: "Anthropic Test".to_string(),
            provider_type: ProviderType::Anthropic,
            api_key: "test-key".to_string(),
            base_url: "https://api.anthropic.com/v1".to_string(),
            pricing: ProviderPricing {
                input_token_cost: 15.0,
                output_token_cost: 75.0,
            },
            enabled: true,
            health: ProviderHealth::default(),
            headers: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_adapter_creation() {
        let provider = test_provider();
        let adapter = AnthropicAdapter::new(provider.clone());
        assert_eq!(adapter.name(), "Anthropic Test");
    }

    #[test]
    fn test_message_conversion() {
        let provider = test_provider();
        let adapter = AnthropicAdapter::new(provider);

        let messages = vec![
            Message::system("You are helpful"),
            Message::user("Hello"),
            Message::assistant("Hi there!"),
        ];

        let (system, converted) = adapter.convert_messages(&messages);
        
        assert_eq!(system, Some("You are helpful".to_string()));
        assert_eq!(converted.len(), 2);
    }
}
