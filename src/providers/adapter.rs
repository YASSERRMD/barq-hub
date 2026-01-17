//! Provider adapter trait

use async_trait::async_trait;
use crate::{ChatRequest, ChatResponse, Provider};
use crate::error::Result;

/// Trait that all provider adapters must implement
#[async_trait]
pub trait ProviderAdapter: Send + Sync {
    /// Handle a chat completion request
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse>;

    /// Get available models for this provider
    async fn list_models(&self) -> Result<Vec<String>>;

    /// Check if provider is healthy
    async fn health_check(&self) -> Result<bool>;

    /// Get the underlying provider configuration
    fn provider(&self) -> &Provider;

    /// Get provider name
    fn name(&self) -> &str {
        &self.provider().name
    }

    /// Estimate tokens for a request (rough estimate)
    fn estimate_tokens(&self, text: &str) -> u32 {
        // Rough estimate: ~4 characters per token
        (text.len() as f64 / 4.0).ceil() as u32
    }
}
