//! Cohere provider adapter

use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use std::time::Instant;

use crate::{
    ChatRequest, ChatResponse, Choice, Message, Provider, TokenUsage,
    error::{ProviderError, Result, SynapseError},
};
use super::ProviderAdapter;

/// Adapter for Cohere API
pub struct CohereAdapter {
    provider: Provider,
    client: Client,
}

impl CohereAdapter {
    pub fn new(provider: Provider) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .expect("Failed to create HTTP client");

        Self { provider, client }
    }
}

#[async_trait]
impl ProviderAdapter for CohereAdapter {
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse> {
        let start = Instant::now();
        let url = format!("{}/chat", self.provider.base_url);

        // Convert messages to Cohere format
        // Cohere uses a different format: message + chat_history
        let mut chat_history: Vec<serde_json::Value> = Vec::new();
        let mut current_message = String::new();
        let mut preamble = String::new();

        for msg in &request.messages {
            match msg.role.as_str() {
                "system" => {
                    preamble = msg.content.clone();
                }
                "user" => {
                    if !current_message.is_empty() {
                        // Previous message was user, push it to history
                        chat_history.push(serde_json::json!({
                            "role": "USER",
                            "message": current_message
                        }));
                    }
                    current_message = msg.content.clone();
                }
                "assistant" => {
                    // Push user message first, then assistant
                    if !current_message.is_empty() {
                        chat_history.push(serde_json::json!({
                            "role": "USER",
                            "message": current_message
                        }));
                        current_message.clear();
                    }
                    chat_history.push(serde_json::json!({
                        "role": "CHATBOT",
                        "message": msg.content
                    }));
                }
                _ => {}
            }
        }

        // Build Cohere request payload
        let mut payload = serde_json::json!({
            "model": request.model,
            "message": current_message,
            "temperature": request.temperature,
        });

        if !chat_history.is_empty() {
            payload["chat_history"] = serde_json::json!(chat_history);
        }

        if !preamble.is_empty() {
            payload["preamble"] = serde_json::json!(preamble);
        }

        if request.max_tokens > 0 {
            payload["max_tokens"] = serde_json::json!(request.max_tokens);
        }

        // Make request to Cohere
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.provider.api_key))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
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

        // Parse Cohere response
        let text = body["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let choices = vec![Choice {
            index: 0,
            message: Message {
                role: "assistant".to_string(),
                content: text,
                function_call: None,
                tool_calls: None,
            },
            finish_reason: body["finish_reason"]
                .as_str()
                .unwrap_or("stop")
                .to_string(),
        }];

        // Parse token usage from Cohere response
        let usage = TokenUsage {
            prompt_tokens: body["meta"]["tokens"]["input_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: body["meta"]["tokens"]["output_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: (body["meta"]["tokens"]["input_tokens"].as_u64().unwrap_or(0) 
                + body["meta"]["tokens"]["output_tokens"].as_u64().unwrap_or(0)) as u32,
        };

        let latency_ms = start.elapsed().as_millis() as u64;

        // Calculate cost (Cohere pricing varies by model)
        let cost = self.calculate_cost(&usage);

        Ok(ChatResponse {
            id: body["generation_id"].as_str().unwrap_or("").to_string(),
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
            "command-r-plus".to_string(),
            "command-r".to_string(),
            "command".to_string(),
            "command-light".to_string(),
            "command-a-03-2025".to_string(),
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

impl CohereAdapter {
    fn calculate_cost(&self, usage: &TokenUsage) -> f64 {
        // Default Cohere pricing (can be customized per model)
        let input_cost = (usage.prompt_tokens as f64 / 1_000_000.0) * 2.5;  // $2.5/1M input
        let output_cost = (usage.completion_tokens as f64 / 1_000_000.0) * 10.0; // $10/1M output
        input_cost + output_cost
    }
}
