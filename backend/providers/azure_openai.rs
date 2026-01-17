//! Azure OpenAI provider adapter

use async_trait::async_trait;
use std::collections::HashMap;
use chrono::Utc;

use crate::{ChatRequest, ChatResponse, Choice, Message, TokenUsage, Provider};
use crate::error::{Result, SynapseError, ProviderError};
use crate::providers::ProviderAdapter;

pub struct AzureOpenAIAdapter {
    provider: Provider,
    client: reqwest::Client,
    api_version: String,
}

impl AzureOpenAIAdapter {
    pub fn new(provider: Provider) -> Self {
        let api_version = provider.headers.get("api-version")
            .cloned()
            .unwrap_or_else(|| "2024-02-15-preview".to_string());
        
        Self {
            provider,
            client: reqwest::Client::new(),
            api_version,
        }
    }

    fn calculate_cost(&self, usage: &TokenUsage) -> f64 {
        let input_cost = (usage.prompt_tokens as f64 / 1_000_000.0)
            * self.provider.pricing.input_token_cost;
        let output_cost = (usage.completion_tokens as f64 / 1_000_000.0)
            * self.provider.pricing.output_token_cost;
        input_cost + output_cost
    }

    fn build_messages(&self, messages: &[Message]) -> Vec<serde_json::Value> {
        messages.iter().map(|msg| {
            serde_json::json!({
                "role": msg.role,
                "content": msg.content
            })
        }).collect()
    }
}

#[async_trait]
impl ProviderAdapter for AzureOpenAIAdapter {
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse> {
        let start = std::time::Instant::now();
        
        // Azure OpenAI URL format: 
        // {base_url}/openai/deployments/{deployment-name}/chat/completions?api-version={version}
        let url = format!(
            "{}/openai/deployments/{}/chat/completions?api-version={}",
            self.provider.base_url,
            request.model,  // In Azure, model = deployment name
            self.api_version
        );

        let payload = serde_json::json!({
            "messages": self.build_messages(&request.messages),
            "temperature": request.temperature,
            "max_tokens": request.max_tokens,
            "top_p": request.top_p,
        });

        let response = self.client.post(&url)
            .header("api-key", &self.provider.api_key)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| SynapseError::Provider(ProviderError::Network(e.to_string())))?;

        let status = response.status();
        let body: serde_json::Value = response.json().await
            .map_err(|e| SynapseError::Provider(ProviderError::InvalidResponse(e.to_string())))?;

        if !status.is_success() {
            let error = body["error"]["message"].as_str().unwrap_or("Unknown error");
            if status.as_u16() == 429 {
                return Err(SynapseError::Provider(ProviderError::RateLimited));
            }
            return Err(SynapseError::Provider(ProviderError::RequestFailed(error.to_string())));
        }

        let choice = &body["choices"][0];
        let content = choice["message"]["content"].as_str().unwrap_or("").to_string();
        let finish_reason = choice["finish_reason"].as_str().unwrap_or("stop").to_string();

        let usage = TokenUsage {
            prompt_tokens: body["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: body["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: body["usage"]["total_tokens"].as_u64().unwrap_or(0) as u32,
        };

        let cost = self.calculate_cost(&usage);
        let latency_ms = start.elapsed().as_millis() as u64;

        Ok(ChatResponse {
            id: body["id"].as_str().unwrap_or("").to_string(),
            model: body["model"].as_str().unwrap_or(&request.model).to_string(),
            choices: vec![Choice {
                index: 0,
                message: Message::assistant(&content),
                finish_reason,
            }],
            usage,
            cost,
            provider: "azure_openai".to_string(),
            created: Utc::now(),
            latency_ms,
        })
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        // Azure deployments are project-specific
        Ok(vec![
            "gpt-4".into(),
            "gpt-4-turbo".into(),
            "gpt-35-turbo".into(),
        ])
    }

    async fn health_check(&self) -> Result<bool> {
        // Simple check - try to list deployments
        let url = format!("{}/openai/deployments?api-version={}", self.provider.base_url, self.api_version);
        let resp = self.client.get(&url)
            .header("api-key", &self.provider.api_key)
            .send()
            .await;
        Ok(resp.is_ok() && resp.unwrap().status().is_success())
    }

    fn provider(&self) -> &Provider { &self.provider }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ProviderType, ProviderPricing, ProviderHealth};

    fn create_test_provider() -> Provider {
        Provider {
            id: "azure-test".to_string(),
            name: "Azure OpenAI".to_string(),
            provider_type: ProviderType::AzureOpenAI,
            api_key: "test-key".to_string(),
            base_url: "https://myresource.openai.azure.com".to_string(),
            pricing: ProviderPricing::default(),
            enabled: true,
            health: ProviderHealth::default(),
            headers: HashMap::new(),
        }
    }

    #[test]
    fn test_adapter_creation() {
        let provider = create_test_provider();
        let adapter = AzureOpenAIAdapter::new(provider);
        assert_eq!(adapter.name(), "Azure OpenAI");
    }

    #[test]
    fn test_message_building() {
        let provider = create_test_provider();
        let adapter = AzureOpenAIAdapter::new(provider);
        
        let messages = vec![
            Message::system("You are helpful"),
            Message::user("Hello"),
        ];
        
        let built = adapter.build_messages(&messages);
        assert_eq!(built.len(), 2);
        assert_eq!(built[0]["role"], "system");
    }
}
