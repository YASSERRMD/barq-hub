//! Chat Service gRPC Implementation
//!
//! Provides LLM chat completions via gRPC.

use std::sync::Arc;
use tonic::{Request, Response, Status};
use tokio_stream::wrappers::ReceiverStream;

use crate::api::AppState;
use crate::grpc::barq::{
    chat_service_server::ChatService,
    ChatRequest, ChatResponse, ChatChunk,
};
use crate::grpc::auth::ApiKeyInterceptor;
use crate::db::ProviderAccountRepository;

pub struct ChatServiceImpl {
    state: Arc<AppState>,
    #[allow(dead_code)]
    auth: ApiKeyInterceptor,
}

impl ChatServiceImpl {
    pub fn new(state: Arc<AppState>, auth: ApiKeyInterceptor) -> Self {
        Self { state, auth }
    }
}

#[tonic::async_trait]
impl ChatService for ChatServiceImpl {
    async fn complete(
        &self,
        request: Request<ChatRequest>,
    ) -> Result<Response<ChatResponse>, Status> {
        let req = request.into_inner();
        
        // Convert to internal format
        let messages: Vec<crate::types::Message> = req.messages
            .into_iter()
            .map(|m| crate::types::Message {
                role: m.role,
                content: m.content,
                function_call: None,
                tool_calls: None,
            })
            .collect();

        let chat_request = crate::types::ChatRequest {
            model: req.model.clone(),
            provider: req.provider.clone(),
            messages,
            temperature: req.temperature.unwrap_or(0.7),
            max_tokens: req.max_tokens.map(|t| t as u32).unwrap_or(2048),
            ..Default::default()
        };

        // If provider is specified, query database directly for account
        if let Some(ref provider_id) = req.provider {
            if let Some(ref pool) = self.state.db_pool {
                let repo = ProviderAccountRepository::new(pool.clone());
                
                // Get the best available account from database
                if let Ok(Some(account_row)) = repo.get_default(provider_id).await {
                    let api_key = account_row.api_key_encrypted.clone().unwrap_or_default();
                    
                    if !api_key.is_empty() {
                        let provider_type = match provider_id.as_str() {
                            "openai" => crate::ProviderType::OpenAI,
                            "anthropic" => crate::ProviderType::Anthropic,
                            "mistral" => crate::ProviderType::Mistral,
                            "cohere" => crate::ProviderType::Cohere,
                            "groq" => crate::ProviderType::Groq,
                            "together" => crate::ProviderType::Together,
                            "gemini" => crate::ProviderType::Gemini,
                            "azure" | "azure-openai" => crate::ProviderType::AzureOpenAI,
                            "bedrock" => crate::ProviderType::Bedrock,
                            _ => crate::ProviderType::OpenAI,
                        };

                        let base_url = get_provider_base_url(provider_id, account_row.endpoint.as_deref());

                        let provider = crate::Provider {
                            id: provider_id.clone(),
                            name: account_row.name.clone(),
                            provider_type,
                            api_key,
                            models: Vec::new(),
                            base_url,
                            pricing: crate::ProviderPricing {
                                input_token_cost: 0.0,
                                output_token_cost: 0.0,
                            },
                            enabled: true,
                            health: crate::ProviderHealth::default(),
                            headers: std::collections::HashMap::new(),
                        };

                        let adapter = crate::providers::create_adapter(provider, self.state.http_client.clone());
                        match adapter.chat(&chat_request).await {
                            Ok(response) => {
                                // TODO: Record usage to database for billing/audit
                                return Ok(Response::new(ChatResponse {
                                    id: response.id,
                                    model: response.model,
                                    content: response.choices.first()
                                        .map(|c| c.message.content.clone())
                                        .unwrap_or_default(),
                                    input_tokens: response.usage.prompt_tokens as i32,
                                    output_tokens: response.usage.completion_tokens as i32,
                                    provider: response.provider.clone(),
                                }));
                            }
                            Err(e) => {
                                tracing::warn!(provider = %provider_id, error = %e, "Provider failed, trying router");
                            }
                        }
                    }
                } else {
                    // Try to get any enabled account for this provider
                    if let Ok(accounts) = repo.list_by_provider(provider_id).await {
                        for account_row in accounts {
                            if !account_row.enabled {
                                continue;
                            }
                            let api_key = account_row.api_key_encrypted.clone().unwrap_or_default();
                            if api_key.is_empty() {
                                continue;
                            }

                            let provider_type = match provider_id.as_str() {
                                "openai" => crate::ProviderType::OpenAI,
                                "anthropic" => crate::ProviderType::Anthropic,
                                "mistral" => crate::ProviderType::Mistral,
                                "cohere" => crate::ProviderType::Cohere,
                                "groq" => crate::ProviderType::Groq,
                                "together" => crate::ProviderType::Together,
                                "gemini" => crate::ProviderType::Gemini,
                                "azure" | "azure-openai" => crate::ProviderType::AzureOpenAI,
                                "bedrock" => crate::ProviderType::Bedrock,
                                _ => crate::ProviderType::OpenAI,
                            };

                            let base_url = get_provider_base_url(provider_id, account_row.endpoint.as_deref());

                            let provider = crate::Provider {
                                id: provider_id.clone(),
                                name: account_row.name.clone(),
                                provider_type,
                                api_key,
                                models: Vec::new(),
                                base_url,
                                pricing: crate::ProviderPricing {
                                    input_token_cost: 0.0,
                                    output_token_cost: 0.0,
                                },
                                enabled: true,
                                health: crate::ProviderHealth::default(),
                                headers: std::collections::HashMap::new(),
                            };

                            let adapter = crate::providers::create_adapter(provider, self.state.http_client.clone());
                            match adapter.chat(&chat_request).await {
                                Ok(response) => {
                                    return Ok(Response::new(ChatResponse {
                                        id: response.id,
                                        model: response.model,
                                        content: response.choices.first()
                                            .map(|c| c.message.content.clone())
                                            .unwrap_or_default(),
                                        input_tokens: response.usage.prompt_tokens as i32,
                                        output_tokens: response.usage.completion_tokens as i32,
                                        provider: response.provider.clone(),
                                    }));
                                }
                                Err(e) => {
                                    tracing::warn!(provider = %provider_id, account = %account_row.name, error = %e, "Account failed, trying next");
                                    continue;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fallback to router
        match self.state.router.route(&chat_request).await {
            Ok(response) => {
                Ok(Response::new(ChatResponse {
                    id: response.id,
                    model: response.model,
                    content: response.choices.first()
                        .map(|c| c.message.content.clone())
                        .unwrap_or_default(),
                    input_tokens: response.usage.prompt_tokens as i32,
                    output_tokens: response.usage.completion_tokens as i32,
                    provider: response.provider.clone(),
                }))
            }
            Err(e) => {
                tracing::error!("Chat completion error: {}", e);
                Err(Status::internal(format!("Completion failed: {}", e)))
            }
        }
    }

    type CompleteStreamStream = ReceiverStream<Result<ChatChunk, Status>>;

    async fn complete_stream(
        &self,
        request: Request<ChatRequest>,
    ) -> Result<Response<Self::CompleteStreamStream>, Status> {
        let req = request.into_inner();
        let (tx, rx) = tokio::sync::mpsc::channel(32);

        // Get provider account from database
        let provider_id = req.provider.clone().unwrap_or_else(|| "openai".to_string());
        
        let (api_key, base_url, _provider_name) = if let Some(ref pool) = self.state.db_pool {
            let repo = ProviderAccountRepository::new(pool.clone());
            
            if let Ok(Some(account)) = repo.get_default(&provider_id).await {
                let key = account.api_key_encrypted.clone().unwrap_or_default();
                let url = get_provider_base_url(&provider_id, account.endpoint.as_deref());
                (key, url, account.name.clone())
            } else if let Ok(accounts) = repo.list_by_provider(&provider_id).await {
                if let Some(account) = accounts.into_iter().find(|a| a.enabled && !a.api_key_encrypted.clone().unwrap_or_default().is_empty()) {
                    let key = account.api_key_encrypted.clone().unwrap_or_default();
                    let url = get_provider_base_url(&provider_id, account.endpoint.as_deref());
                    (key, url, account.name.clone())
                } else {
                    return Err(Status::not_found("No enabled account found for provider"));
                }
            } else {
                return Err(Status::not_found("Provider not found"));
            }
        } else {
            return Err(Status::unavailable("Database not available"));
        };

        if api_key.is_empty() {
            return Err(Status::unauthenticated("No API key configured for provider"));
        }

        let model = req.model.clone();
        let messages = req.messages.clone();
        let temperature = req.temperature.unwrap_or(0.7);
        let max_tokens = req.max_tokens.unwrap_or(2048);
        let http_client = self.state.http_client.clone();

        // Spawn task to handle streaming
        tokio::spawn(async move {
            let url = format!("{}/chat/completions", base_url);
            
            let payload = serde_json::json!({
                "model": model,
                "messages": messages.iter().map(|m| {
                    serde_json::json!({
                        "role": m.role,
                        "content": m.content
                    })
                }).collect::<Vec<_>>(),
                "temperature": temperature,
                "max_tokens": max_tokens,
                "stream": true
            });

            let response = http_client
                .post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await;

            match response {
                Ok(resp) => {
                    if !resp.status().is_success() {
                        let _ = tx.send(Err(Status::internal(format!("Provider error: {}", resp.status())))).await;
                        return;
                    }

                    let mut stream = resp.bytes_stream();
                    use futures_util::StreamExt;
                    
                    let mut buffer = String::new();
                    let stream_id = uuid::Uuid::new_v4().to_string();

                    while let Some(chunk_result) = stream.next().await {
                        match chunk_result {
                            Ok(bytes) => {
                                buffer.push_str(&String::from_utf8_lossy(&bytes));
                                
                                // Process complete SSE lines
                                while let Some(line_end) = buffer.find('\n') {
                                    let line = buffer[..line_end].trim().to_string();
                                    buffer = buffer[line_end + 1..].to_string();
                                    
                                    if line.starts_with("data: ") {
                                        let data = &line[6..];
                                        if data == "[DONE]" {
                                            let _ = tx.send(Ok(ChatChunk {
                                                id: stream_id.clone(),
                                                delta: String::new(),
                                                done: true,
                                            })).await;
                                            return;
                                        }
                                        
                                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                                            if let Some(delta) = json["choices"][0]["delta"]["content"].as_str() {
                                                let _ = tx.send(Ok(ChatChunk {
                                                    id: stream_id.clone(),
                                                    delta: delta.to_string(),
                                                    done: false,
                                                })).await;
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(Err(Status::internal(format!("Stream error: {}", e)))).await;
                                return;
                            }
                        }
                    }

                    // Send done signal if not already sent
                    let _ = tx.send(Ok(ChatChunk {
                        id: uuid::Uuid::new_v4().to_string(),
                        delta: String::new(),
                        done: true,
                    })).await;
                }
                Err(e) => {
                    let _ = tx.send(Err(Status::internal(format!("Request failed: {}", e)))).await;
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

fn get_provider_base_url(provider_id: &str, custom_endpoint: Option<&str>) -> String {
    if let Some(endpoint) = custom_endpoint {
        if !endpoint.is_empty() {
            return endpoint.to_string();
        }
    }

    match provider_id {
        "openai" => "https://api.openai.com/v1".to_string(),
        "anthropic" => "https://api.anthropic.com/v1".to_string(),
        "mistral" => "https://api.mistral.ai/v1".to_string(),
        "cohere" => "https://api.cohere.ai/v1".to_string(),
        "groq" => "https://api.groq.com/openai/v1".to_string(),
        "together" => "https://api.together.xyz/v1".to_string(),
        "gemini" => "https://generativelanguage.googleapis.com/v1".to_string(),
        _ => format!("https://api.{}.com/v1", provider_id),
    }
}
