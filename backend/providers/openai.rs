//! OpenAI provider adapter

use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use std::time::Instant;

use crate::{
    ChatRequest, ChatResponse, Choice, Message, Provider, TokenUsage,
    error::{ProviderError, Result, SynapseError},
};
use super::ProviderAdapter;

/// Adapter for OpenAI API (and compatible APIs like Azure, Groq, Together)
pub struct OpenAIAdapter {
    provider: Provider,
    client: Client,
}

impl OpenAIAdapter {
    pub fn new(provider: Provider, client: Client) -> Self {
        Self { provider, client }
    }
}

#[async_trait]
impl ProviderAdapter for OpenAIAdapter {
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse> {
        let start = Instant::now();
        let url = format!("{}/chat/completions", self.provider.base_url);

        // Build request payload
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
            "top_p": request.top_p,
            "stop": request.stop,
        });

        // Make request
        let mut req = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.provider.api_key))
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

        // Parse response
        let choices: Vec<Choice> = body["choices"]
            .as_array()
            .ok_or_else(|| SynapseError::Provider(ProviderError::InvalidResponse(
                "Missing choices array".to_string()
            )))?
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

        // Calculate cost
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
        // Return commonly used models for OpenAI
        Ok(vec![
            "gpt-4".to_string(),
            "gpt-4-turbo".to_string(),
            "gpt-4-turbo-preview".to_string(),
            "gpt-4o".to_string(),
            "gpt-4o-mini".to_string(),
            "gpt-3.5-turbo".to_string(),
            "gpt-3.5-turbo-16k".to_string(),
        ])
    }

    async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/models", self.provider.base_url);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.provider.api_key))
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await;

        Ok(response.map(|r| r.status().is_success()).unwrap_or(false))
    }

    fn provider(&self) -> &Provider {
        &self.provider
    }
}

impl OpenAIAdapter {
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
            id: "test-openai".to_string(),
            name: "OpenAI Test".to_string(),
            provider_type: ProviderType::OpenAI,
            api_key: "test-key".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            pricing: ProviderPricing {
                input_token_cost: 30.0,  // $30/1M
                output_token_cost: 60.0, // $60/1M
            },
            enabled: true,
            health: ProviderHealth::default(),
            headers: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_adapter_creation() {
        let provider = test_provider();
        let client = Client::new();
        let adapter = OpenAIAdapter::new(provider.clone(), client);
        assert_eq!(adapter.name(), "OpenAI Test");
    }

    #[test]
    fn test_cost_calculation() {
        let provider = test_provider();
        let client = Client::new();
        let adapter = OpenAIAdapter::new(provider, client);
        
        let usage = TokenUsage {
            prompt_tokens: 1000,
            completion_tokens: 500,
            total_tokens: 1500,
        };
        
        let cost = adapter.calculate_cost(&usage);
        // 1000/1M * 30 + 500/1M * 60 = 0.03 + 0.03 = 0.06
        assert!((cost - 0.06).abs() < 0.0001);
    }

    #[test]
    fn test_token_estimation() {
        let provider = test_provider();
        let client = Client::new();
        let adapter = OpenAIAdapter::new(provider, client);
        
        let text = "Hello, world! This is a test.";
        let estimate = adapter.estimate_tokens(text);
        
        // ~28 chars / 4 = ~7 tokens
        assert!(estimate >= 5 && estimate <= 10);
    }
}
