//! Models Service gRPC Implementation
//!
//! Provides model listing via gRPC.

use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::api::AppState;
use crate::grpc::barq::{
    models_service_server::ModelsService,
    ListModelsRequest, ListModelsResponse, Model,
};
use crate::grpc::auth::ApiKeyInterceptor;

pub struct ModelsServiceImpl {
    state: Arc<AppState>,
    #[allow(dead_code)]
    auth: ApiKeyInterceptor,
}

impl ModelsServiceImpl {
    pub fn new(state: Arc<AppState>, auth: ApiKeyInterceptor) -> Self {
        Self { state, auth }
    }
}

#[tonic::async_trait]
impl ModelsService for ModelsServiceImpl {
    async fn list(
        &self,
        _request: Request<ListModelsRequest>,
    ) -> Result<Response<ListModelsResponse>, Status> {
        // Get models from all providers
        let providers = self.state.account_manager.list_providers().await;
        
        let models: Vec<Model> = providers
            .into_iter()
            .flat_map(|p| {
                let provider_name = p.name.clone();
                get_provider_models_by_name(&provider_name).into_iter().map(move |(id, name)| {
                    Model {
                        id: id.to_string(),
                        name: name.to_string(),
                        provider: provider_name.clone(),
                    }
                })
            })
            .collect();

        Ok(Response::new(ListModelsResponse { models }))
    }
}

fn get_provider_models_by_name(provider_name: &str) -> Vec<(&'static str, &'static str)> {
    match provider_name.to_lowercase().as_str() {
        "openai" => vec![
            ("gpt-4o", "GPT-4o"),
            ("gpt-4-turbo", "GPT-4 Turbo"),
            ("gpt-3.5-turbo", "GPT-3.5 Turbo"),
        ],
        "anthropic" => vec![
            ("claude-3-opus-20240229", "Claude 3 Opus"),
            ("claude-3-sonnet-20240229", "Claude 3 Sonnet"),
            ("claude-3-haiku-20240307", "Claude 3 Haiku"),
        ],
        "groq" => vec![
            ("llama-3.3-70b-versatile", "Llama 3.3 70B"),
            ("llama-3.1-8b-instant", "Llama 3.1 8B"),
            ("mixtral-8x7b-32768", "Mixtral 8x7B"),
        ],
        "together" => vec![
            ("meta-llama/Llama-3-70b-chat-hf", "Llama 3 70B"),
            ("mistralai/Mixtral-8x7B-Instruct-v0.1", "Mixtral 8x7B"),
        ],
        "mistral" => vec![
            ("mistral-large-latest", "Mistral Large"),
            ("mistral-medium-latest", "Mistral Medium"),
        ],
        "cohere" => vec![
            ("command-r-plus", "Command R+"),
            ("command-r", "Command R"),
        ],
        _ => vec![],
    }
}

