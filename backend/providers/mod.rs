//! Provider adapters for different LLM providers
//!
//! This module contains adapters for connecting to various LLM providers.

mod adapter;
mod openai;
mod anthropic;
mod mistral;
mod local;
mod gemini;
mod bedrock;
mod azure_openai;
mod cohere;
pub mod account_manager;

pub use adapter::ProviderAdapter;
pub use account_manager::{ProviderAccountManager, ProviderAccount, QuotaPeriod, ProviderCategory};
pub use openai::OpenAIAdapter;
pub use anthropic::AnthropicAdapter;
pub use mistral::MistralAdapter;
pub use local::LocalAdapter;
pub use gemini::GeminiAdapter;
pub use bedrock::BedrockAdapter;
pub use azure_openai::AzureOpenAIAdapter;
pub use cohere::CohereAdapter;

use crate::{Provider, ProviderType};
use std::sync::Arc;

/// Factory function to create the appropriate adapter for a provider
pub fn create_adapter(provider: Provider, client: reqwest::Client) -> Arc<dyn ProviderAdapter> {
    match provider.provider_type {
        ProviderType::OpenAI => Arc::new(OpenAIAdapter::new(provider, client)),
        ProviderType::Anthropic => Arc::new(AnthropicAdapter::new(provider)),
        ProviderType::Mistral => Arc::new(MistralAdapter::new(provider)),
        ProviderType::Local => Arc::new(LocalAdapter::new(provider)),
        ProviderType::Groq => Arc::new(OpenAIAdapter::new(provider, client)), // OpenAI-compatible
        ProviderType::Together => Arc::new(OpenAIAdapter::new(provider, client)), // OpenAI-compatible
        ProviderType::Cohere => Arc::new(CohereAdapter::new(provider)), // Native Cohere API
        ProviderType::Gemini => Arc::new(GeminiAdapter::new(provider)),
        ProviderType::Bedrock => Arc::new(BedrockAdapter::new(provider)),
        ProviderType::AzureOpenAI => Arc::new(AzureOpenAIAdapter::new(provider)),
        ProviderType::Azure => Arc::new(AzureOpenAIAdapter::new(provider)), // Alias
    }
}

