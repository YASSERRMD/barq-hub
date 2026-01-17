//! Embedding providers for vector generation

use async_trait::async_trait;
use crate::error::{Result, SynapseError, ProviderError};

/// Trait for embedding providers
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
    fn dimension(&self) -> usize;
    fn name(&self) -> &str;
}

// ============================================================================
// OpenAI Embeddings
// ============================================================================

pub struct OpenAIEmbedding {
    client: reqwest::Client,
    api_key: String,
    model: String,
    dimension: usize,
}

impl OpenAIEmbedding {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::with_model(api_key, "text-embedding-3-small", 1536)
    }

    pub fn with_model(api_key: impl Into<String>, model: &str, dimension: usize) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.into(),
            model: model.to_string(),
            dimension,
        }
    }

    pub fn ada_002(api_key: impl Into<String>) -> Self {
        Self::with_model(api_key, "text-embedding-ada-002", 1536)
    }

    pub fn small_3(api_key: impl Into<String>) -> Self {
        Self::with_model(api_key, "text-embedding-3-small", 1536)
    }

    pub fn large_3(api_key: impl Into<String>) -> Self {
        Self::with_model(api_key, "text-embedding-3-large", 3072)
    }
}

#[async_trait]
impl EmbeddingProvider for OpenAIEmbedding {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let resp = self.client.post("https://api.openai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({ "input": text, "model": self.model }))
            .send().await
            .map_err(|e| SynapseError::Provider(ProviderError::Network(e.to_string())))?;

        let body: serde_json::Value = resp.json().await
            .map_err(|e| SynapseError::Provider(ProviderError::InvalidResponse(e.to_string())))?;

        Self::parse_embedding(&body, 0)
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let resp = self.client.post("https://api.openai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({ "input": texts, "model": self.model }))
            .send().await
            .map_err(|e| SynapseError::Provider(ProviderError::Network(e.to_string())))?;

        let body: serde_json::Value = resp.json().await
            .map_err(|e| SynapseError::Provider(ProviderError::InvalidResponse(e.to_string())))?;

        let data = body["data"].as_array()
            .ok_or_else(|| SynapseError::Provider(ProviderError::InvalidResponse("No data".into())))?;

        data.iter().enumerate()
            .map(|(i, _)| Self::parse_embedding(&body, i))
            .collect()
    }

    fn dimension(&self) -> usize { self.dimension }
    fn name(&self) -> &str { "openai" }
}

impl OpenAIEmbedding {
    fn parse_embedding(body: &serde_json::Value, index: usize) -> Result<Vec<f32>> {
        body["data"][index]["embedding"].as_array()
            .ok_or_else(|| SynapseError::Provider(ProviderError::InvalidResponse("No embedding".into())))?
            .iter()
            .map(|v| v.as_f64().map(|f| f as f32).ok_or_else(|| 
                SynapseError::Provider(ProviderError::InvalidResponse("Invalid float".into()))))
            .collect()
    }
}

// ============================================================================
// Cohere Embeddings
// ============================================================================

pub struct CohereEmbedding {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl CohereEmbedding {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.into(),
            model: "embed-english-v3.0".to_string(),
        }
    }

    pub fn multilingual(api_key: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.into(),
            model: "embed-multilingual-v3.0".to_string(),
        }
    }
}

#[async_trait]
impl EmbeddingProvider for CohereEmbedding {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let batch = self.embed_batch(&[text.to_string()]).await?;
        batch.into_iter().next().ok_or_else(|| 
            SynapseError::Provider(ProviderError::InvalidResponse("No embedding".into())))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let resp = self.client.post("https://api.cohere.ai/v1/embed")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "texts": texts,
                "model": self.model,
                "input_type": "search_document",
                "truncate": "END"
            }))
            .send().await
            .map_err(|e| SynapseError::Provider(ProviderError::Network(e.to_string())))?;

        let body: serde_json::Value = resp.json().await
            .map_err(|e| SynapseError::Provider(ProviderError::InvalidResponse(e.to_string())))?;

        body["embeddings"].as_array()
            .ok_or_else(|| SynapseError::Provider(ProviderError::InvalidResponse("No embeddings".into())))?
            .iter()
            .map(|emb| {
                emb.as_array()
                    .ok_or_else(|| SynapseError::Provider(ProviderError::InvalidResponse("Invalid array".into())))?
                    .iter()
                    .map(|v| v.as_f64().map(|f| f as f32).ok_or_else(||
                        SynapseError::Provider(ProviderError::InvalidResponse("Invalid float".into()))))
                    .collect()
            })
            .collect()
    }

    fn dimension(&self) -> usize { 1024 }
    fn name(&self) -> &str { "cohere" }
}

// ============================================================================
// Voyage AI Embeddings
// ============================================================================

pub struct VoyageEmbedding {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl VoyageEmbedding {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.into(),
            model: "voyage-2".to_string(),
        }
    }

    pub fn large(api_key: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.into(),
            model: "voyage-large-2".to_string(),
        }
    }

    pub fn code(api_key: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.into(),
            model: "voyage-code-2".to_string(),
        }
    }
}

#[async_trait]
impl EmbeddingProvider for VoyageEmbedding {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let batch = self.embed_batch(&[text.to_string()]).await?;
        batch.into_iter().next().ok_or_else(||
            SynapseError::Provider(ProviderError::InvalidResponse("No embedding".into())))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let resp = self.client.post("https://api.voyageai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "input": texts,
                "model": self.model,
            }))
            .send().await
            .map_err(|e| SynapseError::Provider(ProviderError::Network(e.to_string())))?;

        let body: serde_json::Value = resp.json().await
            .map_err(|e| SynapseError::Provider(ProviderError::InvalidResponse(e.to_string())))?;

        body["data"].as_array()
            .ok_or_else(|| SynapseError::Provider(ProviderError::InvalidResponse("No data".into())))?
            .iter()
            .map(|item| {
                item["embedding"].as_array()
                    .ok_or_else(|| SynapseError::Provider(ProviderError::InvalidResponse("No embedding".into())))?
                    .iter()
                    .map(|v| v.as_f64().map(|f| f as f32).ok_or_else(||
                        SynapseError::Provider(ProviderError::InvalidResponse("Invalid float".into()))))
                    .collect()
            })
            .collect()
    }

    fn dimension(&self) -> usize { 1024 }
    fn name(&self) -> &str { "voyage" }
}

// ============================================================================
// Jina AI Embeddings
// ============================================================================

pub struct JinaEmbedding {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl JinaEmbedding {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.into(),
            model: "jina-embeddings-v2-base-en".to_string(),
        }
    }

    pub fn small(api_key: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.into(),
            model: "jina-embeddings-v2-small-en".to_string(),
        }
    }
}

#[async_trait]
impl EmbeddingProvider for JinaEmbedding {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let batch = self.embed_batch(&[text.to_string()]).await?;
        batch.into_iter().next().ok_or_else(||
            SynapseError::Provider(ProviderError::InvalidResponse("No embedding".into())))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let resp = self.client.post("https://api.jina.ai/v1/embeddings")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "input": texts,
                "model": self.model,
            }))
            .send().await
            .map_err(|e| SynapseError::Provider(ProviderError::Network(e.to_string())))?;

        let body: serde_json::Value = resp.json().await
            .map_err(|e| SynapseError::Provider(ProviderError::InvalidResponse(e.to_string())))?;

        body["data"].as_array()
            .ok_or_else(|| SynapseError::Provider(ProviderError::InvalidResponse("No data".into())))?
            .iter()
            .map(|item| {
                item["embedding"].as_array()
                    .ok_or_else(|| SynapseError::Provider(ProviderError::InvalidResponse("No embedding".into())))?
                    .iter()
                    .map(|v| v.as_f64().map(|f| f as f32).ok_or_else(||
                        SynapseError::Provider(ProviderError::InvalidResponse("Invalid float".into()))))
                    .collect()
            })
            .collect()
    }

    fn dimension(&self) -> usize { 768 }
    fn name(&self) -> &str { "jina" }
}

// ============================================================================
// Mistral Embeddings
// ============================================================================

pub struct MistralEmbedding {
    client: reqwest::Client,
    api_key: String,
}

impl MistralEmbedding {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.into(),
        }
    }
}

#[async_trait]
impl EmbeddingProvider for MistralEmbedding {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let batch = self.embed_batch(&[text.to_string()]).await?;
        batch.into_iter().next().ok_or_else(||
            SynapseError::Provider(ProviderError::InvalidResponse("No embedding".into())))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let resp = self.client.post("https://api.mistral.ai/v1/embeddings")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "input": texts,
                "model": "mistral-embed",
            }))
            .send().await
            .map_err(|e| SynapseError::Provider(ProviderError::Network(e.to_string())))?;

        let body: serde_json::Value = resp.json().await
            .map_err(|e| SynapseError::Provider(ProviderError::InvalidResponse(e.to_string())))?;

        body["data"].as_array()
            .ok_or_else(|| SynapseError::Provider(ProviderError::InvalidResponse("No data".into())))?
            .iter()
            .map(|item| {
                item["embedding"].as_array()
                    .ok_or_else(|| SynapseError::Provider(ProviderError::InvalidResponse("No embedding".into())))?
                    .iter()
                    .map(|v| v.as_f64().map(|f| f as f32).ok_or_else(||
                        SynapseError::Provider(ProviderError::InvalidResponse("Invalid float".into()))))
                    .collect()
            })
            .collect()
    }

    fn dimension(&self) -> usize { 1024 }
    fn name(&self) -> &str { "mistral" }
}

// ============================================================================
// Local/Ollama Embeddings
// ============================================================================

pub struct OllamaEmbedding {
    client: reqwest::Client,
    base_url: String,
    model: String,
}

impl OllamaEmbedding {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "http://localhost:11434".to_string(),
            model: model.into(),
        }
    }

    pub fn with_url(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.into(),
            model: model.into(),
        }
    }

    pub fn nomic() -> Self { Self::new("nomic-embed-text") }
    pub fn mxbai() -> Self { Self::new("mxbai-embed-large") }
}

#[async_trait]
impl EmbeddingProvider for OllamaEmbedding {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let resp = self.client.post(format!("{}/api/embeddings", self.base_url))
            .json(&serde_json::json!({
                "model": self.model,
                "prompt": text,
            }))
            .send().await
            .map_err(|e| SynapseError::Provider(ProviderError::Network(e.to_string())))?;

        let body: serde_json::Value = resp.json().await
            .map_err(|e| SynapseError::Provider(ProviderError::InvalidResponse(e.to_string())))?;

        body["embedding"].as_array()
            .ok_or_else(|| SynapseError::Provider(ProviderError::InvalidResponse("No embedding".into())))?
            .iter()
            .map(|v| v.as_f64().map(|f| f as f32).ok_or_else(||
                SynapseError::Provider(ProviderError::InvalidResponse("Invalid float".into()))))
            .collect()
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::new();
        for text in texts {
            results.push(self.embed(text).await?);
        }
        Ok(results)
    }

    fn dimension(&self) -> usize { 768 } // Varies by model
    fn name(&self) -> &str { "ollama" }
}

// ============================================================================
// Mock Embedding (for testing)
// ============================================================================

pub struct MockEmbedding { dim: usize }

impl MockEmbedding {
    pub fn new(dim: usize) -> Self { Self { dim } }
}

#[async_trait]
impl EmbeddingProvider for MockEmbedding {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let hash = text.bytes().fold(0u64, |acc, b| acc.wrapping_add(b as u64).wrapping_mul(31));
        Ok((0..self.dim).map(|i| ((hash.wrapping_add(i as u64) % 1000) as f32 / 1000.0) - 0.5).collect())
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::new();
        for text in texts { results.push(self.embed(text).await?); }
        Ok(results)
    }

    fn dimension(&self) -> usize { self.dim }
    fn name(&self) -> &str { "mock" }
}

// ============================================================================
// Google Gemini Embeddings
// ============================================================================

pub struct GeminiEmbedding {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl GeminiEmbedding {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.into(),
            model: "text-embedding-004".to_string(),
        }
    }

    pub fn with_model(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.into(),
            model: model.into(),
        }
    }
}

#[async_trait]
impl EmbeddingProvider for GeminiEmbedding {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let batch = self.embed_batch(&[text.to_string()]).await?;
        batch.into_iter().next().ok_or_else(||
            SynapseError::Provider(ProviderError::InvalidResponse("No embedding".into())))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:batchEmbedContents?key={}",
            self.model, self.api_key
        );

        let requests: Vec<_> = texts.iter().map(|t| {
            serde_json::json!({
                "model": format!("models/{}", self.model),
                "content": {"parts": [{"text": t}]}
            })
        }).collect();

        let resp = self.client.post(&url)
            .json(&serde_json::json!({ "requests": requests }))
            .send().await
            .map_err(|e| SynapseError::Provider(ProviderError::Network(e.to_string())))?;

        let body: serde_json::Value = resp.json().await
            .map_err(|e| SynapseError::Provider(ProviderError::InvalidResponse(e.to_string())))?;

        body["embeddings"].as_array()
            .ok_or_else(|| SynapseError::Provider(ProviderError::InvalidResponse("No embeddings".into())))?
            .iter()
            .map(|emb| {
                emb["values"].as_array()
                    .ok_or_else(|| SynapseError::Provider(ProviderError::InvalidResponse("No values".into())))?
                    .iter()
                    .map(|v| v.as_f64().map(|f| f as f32).ok_or_else(||
                        SynapseError::Provider(ProviderError::InvalidResponse("Invalid float".into()))))
                    .collect()
            })
            .collect()
    }

    fn dimension(&self) -> usize { 768 }
    fn name(&self) -> &str { "gemini" }
}

// ============================================================================
// Azure OpenAI Embeddings
// ============================================================================

pub struct AzureOpenAIEmbedding {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
    deployment: String,
    api_version: String,
}

impl AzureOpenAIEmbedding {
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>, deployment: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.into(),
            api_key: api_key.into(),
            deployment: deployment.into(),
            api_version: "2024-02-15-preview".to_string(),
        }
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.api_version = version.into();
        self
    }
}

#[async_trait]
impl EmbeddingProvider for AzureOpenAIEmbedding {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let batch = self.embed_batch(&[text.to_string()]).await?;
        batch.into_iter().next().ok_or_else(||
            SynapseError::Provider(ProviderError::InvalidResponse("No embedding".into())))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let url = format!(
            "{}/openai/deployments/{}/embeddings?api-version={}",
            self.base_url, self.deployment, self.api_version
        );

        let resp = self.client.post(&url)
            .header("api-key", &self.api_key)
            .json(&serde_json::json!({ "input": texts }))
            .send().await
            .map_err(|e| SynapseError::Provider(ProviderError::Network(e.to_string())))?;

        let body: serde_json::Value = resp.json().await
            .map_err(|e| SynapseError::Provider(ProviderError::InvalidResponse(e.to_string())))?;

        body["data"].as_array()
            .ok_or_else(|| SynapseError::Provider(ProviderError::InvalidResponse("No data".into())))?
            .iter()
            .map(|item| {
                item["embedding"].as_array()
                    .ok_or_else(|| SynapseError::Provider(ProviderError::InvalidResponse("No embedding".into())))?
                    .iter()
                    .map(|v| v.as_f64().map(|f| f as f32).ok_or_else(||
                        SynapseError::Provider(ProviderError::InvalidResponse("Invalid float".into()))))
                    .collect()
            })
            .collect()
    }

    fn dimension(&self) -> usize { 1536 }
    fn name(&self) -> &str { "azure_openai" }
}

// ============================================================================
// AWS Bedrock Embeddings (Titan, Cohere)
// ============================================================================

pub struct BedrockEmbedding {
    client: reqwest::Client,
    base_url: String,
    model: String,
}

impl BedrockEmbedding {
    pub fn new(region: impl Into<String>) -> Self {
        let region = region.into();
        Self {
            client: reqwest::Client::new(),
            base_url: format!("https://bedrock-runtime.{}.amazonaws.com", region),
            model: "amazon.titan-embed-text-v1".to_string(),
        }
    }

    pub fn titan(region: impl Into<String>) -> Self {
        Self::new(region)
    }

    pub fn cohere(region: impl Into<String>) -> Self {
        let region = region.into();
        Self {
            client: reqwest::Client::new(),
            base_url: format!("https://bedrock-runtime.{}.amazonaws.com", region),
            model: "cohere.embed-english-v3".to_string(),
        }
    }
}

#[async_trait]
impl EmbeddingProvider for BedrockEmbedding {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let url = format!("{}/model/{}/invoke", self.base_url, self.model);

        let payload = if self.model.contains("titan") {
            serde_json::json!({ "inputText": text })
        } else {
            serde_json::json!({ "texts": [text], "input_type": "search_document" })
        };

        let resp = self.client.post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send().await
            .map_err(|e| SynapseError::Provider(ProviderError::Network(e.to_string())))?;

        let body: serde_json::Value = resp.json().await
            .map_err(|e| SynapseError::Provider(ProviderError::InvalidResponse(e.to_string())))?;

        let embedding_key = if self.model.contains("titan") { "embedding" } else { "embeddings" };
        let embedding_data = if self.model.contains("titan") {
            body[embedding_key].as_array()
        } else {
            body[embedding_key][0].as_array()
        };

        embedding_data
            .ok_or_else(|| SynapseError::Provider(ProviderError::InvalidResponse("No embedding".into())))?
            .iter()
            .map(|v| v.as_f64().map(|f| f as f32).ok_or_else(||
                SynapseError::Provider(ProviderError::InvalidResponse("Invalid float".into()))))
            .collect()
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::new();
        for text in texts {
            results.push(self.embed(text).await?);
        }
        Ok(results)
    }

    fn dimension(&self) -> usize { 1024 }
    fn name(&self) -> &str { "bedrock" }
}

// ============================================================================
// Embedding Provider Factory
// ============================================================================

pub enum EmbeddingProviderType {
    OpenAI { api_key: String, model: Option<String> },
    Cohere { api_key: String },
    Voyage { api_key: String },
    Jina { api_key: String },
    Mistral { api_key: String },
    Ollama { model: String, base_url: Option<String> },
    Gemini { api_key: String },
    AzureOpenAI { base_url: String, api_key: String, deployment: String },
    Bedrock { region: String, model: Option<String> },
    Mock { dimension: usize },
}

pub fn create_embedding_provider(provider_type: EmbeddingProviderType) -> Box<dyn EmbeddingProvider> {
    match provider_type {
        EmbeddingProviderType::OpenAI { api_key, model } => {
            if let Some(m) = model {
                Box::new(OpenAIEmbedding::with_model(api_key, &m, 1536))
            } else {
                Box::new(OpenAIEmbedding::new(api_key))
            }
        }
        EmbeddingProviderType::Cohere { api_key } => Box::new(CohereEmbedding::new(api_key)),
        EmbeddingProviderType::Voyage { api_key } => Box::new(VoyageEmbedding::new(api_key)),
        EmbeddingProviderType::Jina { api_key } => Box::new(JinaEmbedding::new(api_key)),
        EmbeddingProviderType::Mistral { api_key } => Box::new(MistralEmbedding::new(api_key)),
        EmbeddingProviderType::Ollama { model, base_url } => {
            if let Some(url) = base_url {
                Box::new(OllamaEmbedding::with_url(url, model))
            } else {
                Box::new(OllamaEmbedding::new(model))
            }
        }
        EmbeddingProviderType::Gemini { api_key } => Box::new(GeminiEmbedding::new(api_key)),
        EmbeddingProviderType::AzureOpenAI { base_url, api_key, deployment } => {
            Box::new(AzureOpenAIEmbedding::new(base_url, api_key, deployment))
        }
        EmbeddingProviderType::Bedrock { region, model } => {
            if let Some(_m) = model {
                Box::new(BedrockEmbedding::cohere(region))
            } else {
                Box::new(BedrockEmbedding::titan(region))
            }
        }
        EmbeddingProviderType::Mock { dimension } => Box::new(MockEmbedding::new(dimension)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_embedding() {
        let emb = MockEmbedding::new(128);
        let result = emb.embed("test").await.unwrap();
        assert_eq!(result.len(), 128);
    }

    #[tokio::test]
    async fn test_mock_batch() {
        let emb = MockEmbedding::new(64);
        let result = emb.embed_batch(&["a".into(), "b".into()]).await.unwrap();
        assert_eq!(result.len(), 2);
    }
}
