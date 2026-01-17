//! AWS Bedrock provider adapter

use async_trait::async_trait;
use std::collections::HashMap;
use chrono::Utc;

use crate::{ChatRequest, ChatResponse, Choice, Message, TokenUsage, Provider};
use crate::error::{Result, SynapseError, ProviderError};
use crate::providers::ProviderAdapter;

pub struct BedrockAdapter {
    provider: Provider,
    client: reqwest::Client,
    region: String,
}

impl BedrockAdapter {
    pub fn new(provider: Provider) -> Self {
        let region = provider.headers.get("region")
            .cloned()
            .unwrap_or_else(|| "us-east-1".to_string());
        
        Self {
            provider,
            client: reqwest::Client::new(),
            region,
        }
    }

    fn calculate_cost(&self, usage: &TokenUsage) -> f64 {
        let input_cost = (usage.prompt_tokens as f64 / 1_000_000.0)
            * self.provider.pricing.input_token_cost;
        let output_cost = (usage.completion_tokens as f64 / 1_000_000.0)
            * self.provider.pricing.output_token_cost;
        input_cost + output_cost
    }

    fn get_model_family(&self, model: &str) -> &'static str {
        if model.contains("claude") { "anthropic" }
        else if model.contains("llama") || model.contains("meta") { "meta" }
        else if model.contains("titan") { "amazon" }
        else if model.contains("mistral") { "mistral" }
        else { "anthropic" }
    }

    fn build_request_body(&self, request: &ChatRequest) -> serde_json::Value {
        let model_family = self.get_model_family(&request.model);
        
        match model_family {
            "anthropic" => {
                let (system, messages) = self.convert_messages_anthropic(&request.messages);
                let mut body = serde_json::json!({
                    "anthropic_version": "bedrock-2023-05-31",
                    "max_tokens": request.max_tokens,
                    "temperature": request.temperature,
                    "messages": messages
                });
                if let Some(sys) = system {
                    body["system"] = serde_json::json!(sys);
                }
                body
            }
            "meta" => {
                let prompt = self.format_llama_prompt(&request.messages);
                serde_json::json!({
                    "prompt": prompt,
                    "max_gen_len": request.max_tokens,
                    "temperature": request.temperature,
                })
            }
            "mistral" => {
                let prompt = self.format_mistral_prompt(&request.messages);
                serde_json::json!({
                    "prompt": prompt,
                    "max_tokens": request.max_tokens,
                    "temperature": request.temperature,
                })
            }
            _ => {
                let prompt = self.format_simple_prompt(&request.messages);
                serde_json::json!({
                    "inputText": prompt,
                    "textGenerationConfig": {
                        "maxTokenCount": request.max_tokens,
                        "temperature": request.temperature,
                    }
                })
            }
        }
    }

    fn convert_messages_anthropic(&self, messages: &[Message]) -> (Option<String>, Vec<serde_json::Value>) {
        let mut system = None;
        let mut converted = Vec::new();
        for msg in messages {
            match msg.role.as_str() {
                "system" => system = Some(msg.content.clone()),
                role => converted.push(serde_json::json!({"role": role, "content": msg.content})),
            }
        }
        (system, converted)
    }

    fn format_llama_prompt(&self, messages: &[Message]) -> String {
        messages.iter().map(|m| format!("[{}]: {}", m.role.to_uppercase(), m.content)).collect::<Vec<_>>().join("\n")
    }

    fn format_mistral_prompt(&self, messages: &[Message]) -> String {
        messages.iter().map(|m| format!("<{}>{}</{}>", m.role, m.content, m.role)).collect::<Vec<_>>().join("\n")
    }

    fn format_simple_prompt(&self, messages: &[Message]) -> String {
        messages.iter().map(|m| m.content.clone()).collect::<Vec<_>>().join("\n\n")
    }

    fn parse_response(&self, model: &str, body: &serde_json::Value) -> (String, TokenUsage) {
        let model_family = self.get_model_family(model);
        
        let content = match model_family {
            "anthropic" => body["content"][0]["text"].as_str().unwrap_or("").to_string(),
            "meta" => body["generation"].as_str().unwrap_or("").to_string(),
            "mistral" => body["outputs"][0]["text"].as_str().unwrap_or("").to_string(),
            _ => body["results"][0]["outputText"].as_str().unwrap_or("").to_string(),
        };

        let usage = match model_family {
            "anthropic" => TokenUsage {
                prompt_tokens: body["usage"]["input_tokens"].as_u64().unwrap_or(0) as u32,
                completion_tokens: body["usage"]["output_tokens"].as_u64().unwrap_or(0) as u32,
                total_tokens: 0,
            },
            _ => TokenUsage {
                prompt_tokens: body["prompt_token_count"].as_u64().unwrap_or(0) as u32,
                completion_tokens: body["generation_token_count"].as_u64().unwrap_or(0) as u32,
                total_tokens: 0,
            },
        };

        (content, TokenUsage {
            total_tokens: usage.prompt_tokens + usage.completion_tokens,
            ..usage
        })
    }
}

#[async_trait]
impl ProviderAdapter for BedrockAdapter {
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse> {
        let start = std::time::Instant::now();
        
        // Use Bedrock runtime API
        let url = format!(
            "{}/model/{}/invoke",
            self.provider.base_url,
            request.model
        );

        let payload = self.build_request_body(request);

        let response = self.client.post(&url)
            .header("Content-Type", "application/json")
            // AWS SigV4 would be added here in production
            .json(&payload)
            .send()
            .await
            .map_err(|e| SynapseError::Provider(ProviderError::Network(e.to_string())))?;

        let status = response.status();
        let body: serde_json::Value = response.json().await
            .map_err(|e| SynapseError::Provider(ProviderError::InvalidResponse(e.to_string())))?;

        if !status.is_success() {
            let error = body["message"].as_str().unwrap_or("Unknown error");
            return Err(SynapseError::Provider(ProviderError::RequestFailed(error.to_string())));
        }

        let (content, usage) = self.parse_response(&request.model, &body);
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
            provider: "bedrock".to_string(),
            created: Utc::now(),
            latency_ms,
        })
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        Ok(vec![
            "anthropic.claude-3-sonnet-20240229-v1:0".into(),
            "anthropic.claude-3-haiku-20240307-v1:0".into(),
            "amazon.titan-text-express-v1".into(),
            "meta.llama3-70b-instruct-v1:0".into(),
        ])
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(true) // Bedrock health check would require AWS auth
    }

    fn provider(&self) -> &Provider { &self.provider }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ProviderType, ProviderPricing, ProviderHealth};

    fn create_test_provider() -> Provider {
        Provider {
            id: "bedrock-test".to_string(),
            name: "Bedrock".to_string(),
            provider_type: ProviderType::Bedrock,
            api_key: "".to_string(),
            base_url: "https://bedrock-runtime.us-east-1.amazonaws.com".to_string(),
            pricing: ProviderPricing::default(),
            enabled: true,
            health: ProviderHealth::default(),
            headers: HashMap::new(),
        }
    }

    #[test]
    fn test_adapter_creation() {
        let provider = create_test_provider();
        let adapter = BedrockAdapter::new(provider);
        assert_eq!(adapter.name(), "Bedrock");
    }

    #[test]
    fn test_model_family_detection() {
        let provider = create_test_provider();
        let adapter = BedrockAdapter::new(provider);
        
        assert_eq!(adapter.get_model_family("anthropic.claude-3-sonnet"), "anthropic");
        assert_eq!(adapter.get_model_family("meta.llama3-70b"), "meta");
        assert_eq!(adapter.get_model_family("amazon.titan-text"), "amazon");
    }
}
