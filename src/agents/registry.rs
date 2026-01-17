//! Provider registry for dynamic provider instantiation

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::agents::{Agent, LLMProvider, EmbeddingProvider as EmbedProvider, VectorDBProvider};
use crate::knowledge::embeddings::{self, EmbeddingProvider};
use crate::knowledge::vector_db::{self, VectorDB};
use crate::providers::{ProviderAdapter, create_adapter};
use crate::{Provider, ProviderType, ProviderPricing, ProviderHealth};

/// Registry that manages dynamic provider instances
pub struct ProviderRegistry {
    /// Cached LLM adapters by agent ID
    llm_cache: Arc<RwLock<HashMap<String, Arc<dyn ProviderAdapter>>>>,
    /// Cached embedding providers by agent ID
    embedding_cache: Arc<RwLock<HashMap<String, Arc<dyn EmbeddingProvider>>>>,
    /// Cached vector DBs by agent ID
    vector_db_cache: Arc<RwLock<HashMap<String, Arc<dyn VectorDB>>>>,
    /// Default API keys from environment
    default_keys: ProviderKeys,
}

/// Default API keys loaded from environment
#[derive(Clone)]
pub struct ProviderKeys {
    pub openai: Option<String>,
    pub anthropic: Option<String>,
    pub mistral: Option<String>,
    pub groq: Option<String>,
    pub together: Option<String>,
    pub cohere: Option<String>,
    pub voyage: Option<String>,
    pub jina: Option<String>,
    pub gemini: Option<String>,
    pub azure_openai: Option<String>,
    // Note: AWS Bedrock uses IAM credentials, not API key
}

impl ProviderKeys {
    pub fn from_env() -> Self {
        Self {
            openai: std::env::var("OPENAI_API_KEY").ok(),
            anthropic: std::env::var("ANTHROPIC_API_KEY").ok(),
            mistral: std::env::var("MISTRAL_API_KEY").ok(),
            groq: std::env::var("GROQ_API_KEY").ok(),
            together: std::env::var("TOGETHER_API_KEY").ok(),
            cohere: std::env::var("COHERE_API_KEY").ok(),
            voyage: std::env::var("VOYAGE_API_KEY").ok(),
            jina: std::env::var("JINA_API_KEY").ok(),
            gemini: std::env::var("GEMINI_API_KEY").ok(),
            azure_openai: std::env::var("AZURE_OPENAI_API_KEY").ok(),
        }
    }
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            llm_cache: Arc::new(RwLock::new(HashMap::new())),
            embedding_cache: Arc::new(RwLock::new(HashMap::new())),
            vector_db_cache: Arc::new(RwLock::new(HashMap::new())),
            default_keys: ProviderKeys::from_env(),
        }
    }

    /// Get or create LLM adapter for an agent
    pub async fn get_llm_adapter(&self, agent: &Agent) -> Option<Arc<dyn ProviderAdapter>> {
        // Check cache first
        {
            let cache = self.llm_cache.read().await;
            if let Some(adapter) = cache.get(&agent.id) {
                return Some(adapter.clone());
            }
        }

        // Create new adapter
        let config = &agent.llm_config;
        let api_key = config.api_key.clone().or_else(|| self.get_default_llm_key(config.provider));
        
        let api_key = api_key?;

        let (provider_type, base_url) = match config.provider {
            LLMProvider::OpenAI => (ProviderType::OpenAI, config.base_url.clone().unwrap_or("https://api.openai.com/v1".into())),
            LLMProvider::Anthropic => (ProviderType::Anthropic, config.base_url.clone().unwrap_or("https://api.anthropic.com/v1".into())),
            LLMProvider::Mistral => (ProviderType::Mistral, config.base_url.clone().unwrap_or("https://api.mistral.ai/v1".into())),
            LLMProvider::Groq => (ProviderType::Groq, config.base_url.clone().unwrap_or("https://api.groq.com/openai/v1".into())),
            LLMProvider::Together => (ProviderType::Together, config.base_url.clone().unwrap_or("https://api.together.xyz/v1".into())),
            LLMProvider::Cohere => (ProviderType::Cohere, config.base_url.clone().unwrap_or("https://api.cohere.com/v1".into())),
            LLMProvider::Gemini => (ProviderType::Gemini, config.base_url.clone().unwrap_or("https://generativelanguage.googleapis.com".into())),
            LLMProvider::Bedrock => (ProviderType::Bedrock, config.base_url.clone().unwrap_or("https://bedrock-runtime.us-east-1.amazonaws.com".into())),
            LLMProvider::AzureOpenAI => (ProviderType::AzureOpenAI, config.base_url.clone().unwrap_or_default()),
            LLMProvider::Local => (ProviderType::Local, config.base_url.clone().unwrap_or("http://localhost:11434".into())),
        };

        let provider = Provider {
            id: format!("agent-{}-llm", agent.id),
            name: format!("{} LLM", agent.name),
            provider_type,
            api_key,
            base_url,
            pricing: ProviderPricing::default(),
            enabled: true,
            models: Vec::new(),
            health: ProviderHealth::default(),
            headers: HashMap::new(),
        };

        let adapter = create_adapter(provider);
        
        // Cache it
        self.llm_cache.write().await.insert(agent.id.clone(), adapter.clone());
        
        Some(adapter)
    }

    /// Get or create embedding provider for an agent
    pub async fn get_embedding_provider(&self, agent: &Agent) -> Option<Arc<dyn EmbeddingProvider>> {
        // Check cache
        {
            let cache = self.embedding_cache.read().await;
            if let Some(provider) = cache.get(&agent.id) {
                return Some(provider.clone());
            }
        }

        // Create new provider
        let config = &agent.embedding_config;
        let api_key = config.api_key.clone().or_else(|| self.get_default_embedding_key(config.provider));

        let provider: Arc<dyn EmbeddingProvider> = match config.provider {
            EmbedProvider::OpenAI => {
                Arc::new(embeddings::OpenAIEmbedding::with_model(
                    api_key?,
                    &config.model,
                    config.dimension,
                ))
            }
            EmbedProvider::Cohere => Arc::new(embeddings::CohereEmbedding::new(api_key?)),
            EmbedProvider::Voyage => Arc::new(embeddings::VoyageEmbedding::new(api_key?)),
            EmbedProvider::Jina => Arc::new(embeddings::JinaEmbedding::new(api_key?)),
            EmbedProvider::Mistral => Arc::new(embeddings::MistralEmbedding::new(api_key?)),
            EmbedProvider::Ollama => {
                let url = config.base_url.clone().unwrap_or("http://localhost:11434".into());
                Arc::new(embeddings::OllamaEmbedding::with_url(url, &config.model))
            }
            EmbedProvider::Mock => Arc::new(embeddings::MockEmbedding::new(config.dimension)),
        };

        // Cache it
        self.embedding_cache.write().await.insert(agent.id.clone(), provider.clone());
        
        Some(provider)
    }

    /// Get or create vector DB for an agent
    pub async fn get_vector_db(&self, agent: &Agent) -> Arc<dyn VectorDB> {
        // Check cache
        {
            let cache = self.vector_db_cache.read().await;
            if let Some(db) = cache.get(&agent.id) {
                return db.clone();
            }
        }

        // Create new DB connection
        let config = &agent.vector_db_config;

        let db: Arc<dyn VectorDB> = match config.provider {
            VectorDBProvider::InMemory => Arc::new(vector_db::InMemoryVectorDB::new()),
            VectorDBProvider::Qdrant => {
                let url = config.url.clone().unwrap_or("http://localhost:6333".into());
                if let Some(ref key) = config.api_key {
                    Arc::new(vector_db::QdrantDB::with_api_key(url, key))
                } else {
                    Arc::new(vector_db::QdrantDB::new(url))
                }
            }
            VectorDBProvider::Pinecone => {
                let host = config.url.clone().unwrap_or_default();
                let key = config.api_key.clone().unwrap_or_default();
                Arc::new(vector_db::PineconeDB::new(host, key))
            }
            VectorDBProvider::Weaviate => {
                let url = config.url.clone().unwrap_or("http://localhost:8080".into());
                if let Some(ref key) = config.api_key {
                    Arc::new(vector_db::WeaviateDB::with_api_key(url, key))
                } else {
                    Arc::new(vector_db::WeaviateDB::new(url))
                }
            }
            VectorDBProvider::Chroma => {
                let url = config.url.clone().unwrap_or("http://localhost:8000".into());
                Arc::new(vector_db::ChromaDB::new(url))
            }
        };

        // Cache it
        self.vector_db_cache.write().await.insert(agent.id.clone(), db.clone());
        
        db
    }

    /// Invalidate cache for an agent (call when config changes)
    pub async fn invalidate_cache(&self, agent_id: &str) {
        self.llm_cache.write().await.remove(agent_id);
        self.embedding_cache.write().await.remove(agent_id);
        self.vector_db_cache.write().await.remove(agent_id);
    }

    /// Clear all caches
    pub async fn clear_all_caches(&self) {
        self.llm_cache.write().await.clear();
        self.embedding_cache.write().await.clear();
        self.vector_db_cache.write().await.clear();
    }

    fn get_default_llm_key(&self, provider: LLMProvider) -> Option<String> {
        match provider {
            LLMProvider::OpenAI => self.default_keys.openai.clone(),
            LLMProvider::Anthropic => self.default_keys.anthropic.clone(),
            LLMProvider::Mistral => self.default_keys.mistral.clone(),
            LLMProvider::Groq => self.default_keys.groq.clone(),
            LLMProvider::Together => self.default_keys.together.clone(),
            LLMProvider::Cohere => self.default_keys.cohere.clone(),
            LLMProvider::Gemini => self.default_keys.gemini.clone(),
            LLMProvider::Bedrock => Some(String::new()), // Uses IAM
            LLMProvider::AzureOpenAI => self.default_keys.azure_openai.clone(),
            LLMProvider::Local => Some(String::new()),
        }
    }

    fn get_default_embedding_key(&self, provider: EmbedProvider) -> Option<String> {
        match provider {
            EmbedProvider::OpenAI => self.default_keys.openai.clone(),
            EmbedProvider::Cohere => self.default_keys.cohere.clone(),
            EmbedProvider::Voyage => self.default_keys.voyage.clone(),
            EmbedProvider::Jina => self.default_keys.jina.clone(),
            EmbedProvider::Mistral => self.default_keys.mistral.clone(),
            EmbedProvider::Ollama => Some(String::new()),
            EmbedProvider::Mock => Some(String::new()),
        }
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self { Self::new() }
}
