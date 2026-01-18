//! Core types for SYNAPSE Brain
//!
//! This module defines all the fundamental data structures used across the platform.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// ============================================================================
// Provider Types
// ============================================================================

/// Represents an LLM provider (OpenAI, Anthropic, Mistral, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    /// Unique identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Provider type (openai, anthropic, mistral, local, bedrock, azure)
    pub provider_type: ProviderType,
    /// List of configured models and their capabilities
    #[serde(default)]
    pub models: Vec<ProviderModel>,
    /// API key for authentication
    #[serde(skip_serializing)]
    pub api_key: String,
    /// Base URL for API calls
    pub base_url: String,
    /// Pricing information
    pub pricing: ProviderPricing,
    /// Whether provider is enabled
    pub enabled: bool,
    /// Health status
    #[serde(default)]
    pub health: ProviderHealth,
    /// Custom headers
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
}

impl Default for Provider {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: "Default Provider".to_string(),
            provider_type: ProviderType::OpenAI,
            api_key: String::new(),
            models: Vec::new(),
            base_url: "https://api.openai.com/v1".to_string(),
            pricing: ProviderPricing::default(),
            enabled: true,
            health: ProviderHealth::default(),
            headers: std::collections::HashMap::new(),
        }
    }
}

/// Type of LLM provider
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    OpenAI,
    Anthropic,
    Mistral,
    Local,
    Bedrock,
    Azure,
    AzureOpenAI,
    Groq,
    Together,
    Cohere,
    Gemini,
}

impl std::str::FromStr for ProviderType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "openai" => Ok(Self::OpenAI),
            "anthropic" => Ok(Self::Anthropic),
            "mistral" => Ok(Self::Mistral),
            "local" | "ollama" => Ok(Self::Local),
            "bedrock" => Ok(Self::Bedrock),
            "azure" => Ok(Self::Azure),
            "azure_openai" | "azureopenai" => Ok(Self::AzureOpenAI),
            "groq" => Ok(Self::Groq),
            "together" => Ok(Self::Together),
            "cohere" => Ok(Self::Cohere),
            "gemini" | "google" => Ok(Self::Gemini),
            _ => Err(format!("Unknown provider type: {}", s)),
        }
    }
}

/// Capability of a model (LLM, Embedding, TTS, STT)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelCapability {
    LLM,
    Embedding,
    TTS,
    STT,
    ImageGeneration,
}

/// Configuration for a specific model under a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderModel {
    /// Model identifier (e.g., "gpt-4-turbo")
    pub id: String,
    /// Display name
    pub name: String,
    /// Capabilities this model supports
    pub capabilities: Vec<ModelCapability>,
    /// Cost per 1M input tokens (USD) - overrides provider default if present
    pub input_token_cost: Option<f64>,
    /// Cost per 1M output tokens (USD) - overrides provider default if present
    pub output_token_cost: Option<f64>,
}

/// Pricing information for a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPricing {
    /// Cost per 1M input tokens (USD)
    pub input_token_cost: f64,
    /// Cost per 1M output tokens (USD)
    pub output_token_cost: f64,
}

impl Default for ProviderPricing {
    fn default() -> Self {
        Self {
            input_token_cost: 0.0,
            output_token_cost: 0.0,
        }
    }
}

/// Health status of a provider
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderHealth {
    /// Whether provider is currently healthy
    pub healthy: bool,
    /// Last successful request time
    pub last_success: Option<DateTime<Utc>>,
    /// Last error time
    pub last_error: Option<DateTime<Utc>>,
    /// Average latency in ms
    pub avg_latency_ms: Option<f64>,
    /// Error rate (0.0 - 1.0)
    pub error_rate: f64,
}

// ============================================================================
// Request/Response Types
// ============================================================================

/// Chat completion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    /// Model to use
    pub model: String,
    /// Explicit provider to use (e.g., "openai", "azure", "anthropic", "bedrock")
    /// If specified, routes directly to this provider
    pub provider: Option<String>,
    /// Conversation messages
    pub messages: Vec<Message>,
    /// Temperature (0.0 - 2.0)
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    /// Maximum tokens to generate
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    /// Top-p sampling
    pub top_p: Option<f32>,
    /// Stop sequences
    pub stop: Option<Vec<String>>,
    /// Provider preference for routing (deprecated, use `provider` instead)
    pub provider_preference: Option<ProviderPreference>,
    /// User ID for tracking
    pub user_id: Option<String>,
    /// Request metadata
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,
}

fn default_temperature() -> f32 {
    0.7
}

fn default_max_tokens() -> u32 {
    2048
}

impl Default for ChatRequest {
    fn default() -> Self {
        Self {
            model: "gpt-4".to_string(),
            provider: None,
            messages: vec![],
            temperature: 0.7,
            max_tokens: 2048,
            top_p: None,
            stop: None,
            provider_preference: None,
            user_id: None,
            metadata: std::collections::HashMap::new(),
        }
    }
}

/// Chat completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// Unique response ID
    pub id: String,
    /// Provider that served the request
    pub provider: String,
    /// Model used
    pub model: String,
    /// Generated choices
    pub choices: Vec<Choice>,
    /// Token usage statistics
    pub usage: TokenUsage,
    /// Timestamp
    pub created: DateTime<Utc>,
    /// Latency in milliseconds
    pub latency_ms: u64,
    /// Cost of this request
    pub cost: f64,
}

/// A single completion choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    /// Choice index
    pub index: usize,
    /// Generated message
    pub message: Message,
    /// Reason for stopping
    pub finish_reason: String,
}

/// A chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Role: "system", "user", "assistant", "function"
    pub role: String,
    /// Message content
    pub content: String,
    /// Function call (if applicable)
    pub function_call: Option<FunctionCall>,
    /// Tool calls (for assistant messages)
    pub tool_calls: Option<Vec<ToolCall>>,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
            function_call: None,
            tool_calls: None,
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
            function_call: None,
            tool_calls: None,
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
            function_call: None,
            tool_calls: None,
        }
    }
}

/// Function call in a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

/// Tool call in a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: FunctionCall,
}

/// Token usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Tokens in the prompt
    pub prompt_tokens: u32,
    /// Tokens in the completion
    pub completion_tokens: u32,
    /// Total tokens used
    pub total_tokens: u32,
}

// ============================================================================
// Routing Types
// ============================================================================

/// Provider selection preference
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderPreference {
    /// Select cheapest provider
    CostOptimal,
    /// Select fastest provider
    LatencyOptimal,
    /// Select highest quality provider
    QualityTier,
    /// Load balance across providers
    LoadBalanced,
    /// Use specific provider by index
    SpecificProvider(usize),
}

// ============================================================================
// Cost Types
// ============================================================================

/// A single cost entry for tracking usage
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CostEntry {
    /// Unique entry ID
    pub id: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Provider used
    pub provider: String,
    /// Model used
    pub model: String,
    /// Input tokens
    pub input_tokens: u32,
    /// Output tokens
    pub output_tokens: u32,
    /// Total cost in USD
    pub cost: f64,
    /// User ID
    pub user_id: String,
    /// Request ID for correlation
    pub request_id: String,
}

/// Summary of costs over a period
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CostSummary {
    /// Total cost in USD
    pub total_cost: f64,
    /// Total requests
    pub request_count: usize,
    /// Total tokens
    pub total_tokens: u64,
    /// Cost breakdown by provider
    pub by_provider: std::collections::HashMap<String, f64>,
    /// Cost breakdown by model
    pub by_model: std::collections::HashMap<String, f64>,
    /// Cost breakdown by user
    pub by_user: std::collections::HashMap<String, f64>,
    /// Start date of period
    pub period_start: DateTime<Utc>,
    /// End date of period
    pub period_end: DateTime<Utc>,
}

/// Budget configuration for a user or organization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Budget {
    /// User or org ID
    pub entity_id: String,
    /// Monthly limit in USD
    pub monthly_limit: f64,
    /// Amount spent this month
    pub spent_this_month: f64,
    /// Whether to hard-stop on limit
    pub enforce_limit: bool,
    /// Alert thresholds (e.g., 0.8 = 80%)
    pub alert_thresholds: Vec<f64>,
    /// Reset day of month (1-28)
    pub reset_day: u8,
}

// ============================================================================
// Health & Metrics Types
// ============================================================================

/// System health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Overall health
    pub status: String,
    /// Uptime in seconds
    pub uptime_seconds: u64,
    /// Version
    pub version: String,
    /// Component health
    pub components: std::collections::HashMap<String, ComponentHealth>,
}

/// Health of a single component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub healthy: bool,
    pub message: Option<String>,
    pub latency_ms: Option<u64>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = Provider {
            id: "openai-1".to_string(),
            name: "OpenAI".to_string(),
            provider_type: ProviderType::OpenAI,
            api_key: "sk-test".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            pricing: ProviderPricing {
                input_token_cost: 0.03,
                output_token_cost: 0.06,
            },
            enabled: true,
            health: ProviderHealth::default(),
            headers: std::collections::HashMap::new(),
        };
        assert_eq!(provider.name, "OpenAI");
        assert_eq!(provider.provider_type, ProviderType::OpenAI);
    }

    #[test]
    fn test_request_serialization() {
        let req = ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![Message::user("Hello")],
            temperature: 0.7,
            max_tokens: 1000,
            ..Default::default()
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("gpt-4"));
        assert!(json.contains("Hello"));
    }

    #[test]
    fn test_message_helpers() {
        let system = Message::system("You are a helpful assistant");
        assert_eq!(system.role, "system");

        let user = Message::user("Hello");
        assert_eq!(user.role, "user");

        let assistant = Message::assistant("Hi there!");
        assert_eq!(assistant.role, "assistant");
    }

    #[test]
    fn test_token_usage_default() {
        let usage = TokenUsage::default();
        assert_eq!(usage.prompt_tokens, 0);
        assert_eq!(usage.completion_tokens, 0);
        assert_eq!(usage.total_tokens, 0);
    }

    #[test]
    fn test_provider_pricing() {
        let pricing = ProviderPricing {
            input_token_cost: 0.015,
            output_token_cost: 0.002,
        };
        // 100K input tokens, 50K output tokens
        let input_cost = (100_000.0 / 1_000_000.0) * pricing.input_token_cost;
        let output_cost = (50_000.0 / 1_000_000.0) * pricing.output_token_cost;
        let total = input_cost + output_cost;
        assert!((total - 0.0016).abs() < 0.0001); // ~$0.0016
    }
}
