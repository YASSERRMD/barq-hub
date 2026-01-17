//! Google Gemini provider adapter

use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;

use crate::{ChatRequest, ChatResponse, Choice, Message, TokenUsage, Provider};
use crate::error::{Result, SynapseError, ProviderError};
use crate::providers::ProviderAdapter;

pub struct GeminiAdapter {
    provider: Provider,
    client: reqwest::Client,
}

impl GeminiAdapter {
    pub fn new(provider: Provider) -> Self {
        Self {
            provider,
            client: reqwest::Client::new(),
        }
    }

    fn calculate_cost(&self, usage: &TokenUsage) -> f64 {
        let input_cost = (usage.prompt_tokens as f64 / 1_000_000.0)
            * self.provider.pricing.input_token_cost;
        let output_cost = (usage.completion_tokens as f64 / 1_000_000.0)
            * self.provider.pricing.output_token_cost;
        input_cost + output_cost
    }

    fn convert_messages(&self, messages: &[Message]) -> Vec<serde_json::Value> {
        messages.iter().filter_map(|msg| {
            let role = match msg.role.as_str() {
                "system" => return None, // System is handled separately
                "assistant" => "model",
                _ => "user",
            };
            Some(serde_json::json!({
                "role": role,
                "parts": [{"text": msg.content}]
            }))
        }).collect()
    }

    fn get_system_instruction(&self, messages: &[Message]) -> Option<String> {
        messages.iter()
            .find(|m| m.role == "system")
            .map(|m| m.content.clone())
    }
}

#[async_trait]
impl ProviderAdapter for GeminiAdapter {
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse> {
        let start = std::time::Instant::now();
        
        let url = format!(
            "{}/v1beta/models/{}:generateContent?key={}",
            self.provider.base_url,
            request.model,
            self.provider.api_key
        );

        let contents = self.convert_messages(&request.messages);
        let system_instruction = self.get_system_instruction(&request.messages);

        let mut payload = serde_json::json!({
            "contents": contents,
            "generationConfig": {
                "temperature": request.temperature,
                "maxOutputTokens": request.max_tokens,
            }
        });

        if let Some(sys) = system_instruction {
            payload["systemInstruction"] = serde_json::json!({
                "parts": [{"text": sys}]
            });
        }

        let response = self.client.post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| SynapseError::Provider(ProviderError::Network(e.to_string())))?;

        let status = response.status();
        let body: serde_json::Value = response.json().await
            .map_err(|e| SynapseError::Provider(ProviderError::InvalidResponse(e.to_string())))?;

        if !status.is_success() {
            let error = body["error"]["message"].as_str().unwrap_or("Unknown error");
            return Err(SynapseError::Provider(ProviderError::RequestFailed(error.to_string())));
        }

        let content = body["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let usage = TokenUsage {
            prompt_tokens: body["usageMetadata"]["promptTokenCount"].as_u64().unwrap_or(0) as u32,
            completion_tokens: body["usageMetadata"]["candidatesTokenCount"].as_u64().unwrap_or(0) as u32,
            total_tokens: body["usageMetadata"]["totalTokenCount"].as_u64().unwrap_or(0) as u32,
        };

        let cost = self.calculate_cost(&usage);
        let latency_ms = start.elapsed().as_millis() as u64;

        Ok(ChatResponse {
            id: uuid::Uuid::new_v4().to_string(),
            model: request.model.clone(),
            choices: vec![Choice {
                index: 0,
                message: Message::assistant(&content),
                finish_reason: "stop".to_string(),
            }],
            usage,
            cost,
            provider: "gemini".to_string(),
            created: Utc::now(),
            latency_ms,
        })
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        Ok(vec![
            "gemini-1.5-pro".into(),
            "gemini-1.5-flash".into(),
            "gemini-1.0-pro".into(),
        ])
    }

    async fn health_check(&self) -> Result<bool> {
        let url = format!(
            "{}/v1beta/models?key={}",
            self.provider.base_url,
            self.provider.api_key
        );
        
        let resp = self.client.get(&url).send().await;
        Ok(resp.is_ok() && resp.unwrap().status().is_success())
    }

    fn provider(&self) -> &Provider {
        &self.provider
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ProviderType, ProviderPricing, ProviderHealth};

    fn create_test_provider() -> Provider {
        Provider {
            id: "gemini-test".to_string(),
            name: "Gemini".to_string(),
            provider_type: ProviderType::Gemini,
            api_key: "test-key".to_string(),
            base_url: "https://generativelanguage.googleapis.com".to_string(),
            pricing: ProviderPricing::default(),
            enabled: true,
            health: ProviderHealth::default(),
            headers: HashMap::new(),
        }
    }

    #[test]
    fn test_adapter_creation() {
        let provider = create_test_provider();
        let adapter = GeminiAdapter::new(provider);
        assert_eq!(adapter.name(), "Gemini");
    }

    #[test]
    fn test_message_conversion() {
        let provider = create_test_provider();
        let adapter = GeminiAdapter::new(provider);
        
        let messages = vec![
            Message::system("You are helpful"),
            Message::user("Hello"),
            Message::assistant("Hi there"),
        ];
        
        let converted = adapter.convert_messages(&messages);
        assert_eq!(converted.len(), 2); // System is filtered out
    }
}
