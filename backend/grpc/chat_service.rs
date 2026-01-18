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
        _request: Request<ChatRequest>,
    ) -> Result<Response<Self::CompleteStreamStream>, Status> {
        Err(Status::unimplemented("Streaming not yet implemented"))
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
