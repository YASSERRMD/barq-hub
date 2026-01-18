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

pub struct ChatServiceImpl {
    state: Arc<AppState>,
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
        
        // Convert to internal format using Message helpers
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
            messages,
            temperature: req.temperature.unwrap_or(0.7),
            max_tokens: req.max_tokens.map(|t| t as u32).unwrap_or(2048),
            ..Default::default()
        };

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
        // Streaming not yet implemented
        Err(Status::unimplemented("Streaming not yet implemented"))
    }
}
