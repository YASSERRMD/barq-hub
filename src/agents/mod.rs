//! Agent management with dynamic provider configuration

pub mod registry;
pub mod runtime;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Agent configuration with assigned providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub owner_id: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    
    /// LLM provider configuration
    pub llm_config: LLMConfig,
    
    /// Embedding provider configuration
    pub embedding_config: EmbeddingConfig,
    
    /// Vector database configuration
    pub vector_db_config: VectorDBConfig,
    
    /// System prompt for this agent
    pub system_prompt: Option<String>,
    
    /// Knowledge collection name (for RAG)
    pub knowledge_collection: Option<String>,
    
    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl Agent {
    pub fn new(name: impl Into<String>, owner_id: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            description: String::new(),
            owner_id: owner_id.into(),
            enabled: true,
            created_at: now,
            updated_at: now,
            llm_config: LLMConfig::default(),
            embedding_config: EmbeddingConfig::default(),
            vector_db_config: VectorDBConfig::default(),
            system_prompt: None,
            knowledge_collection: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn with_llm(mut self, config: LLMConfig) -> Self {
        self.llm_config = config;
        self.updated_at = Utc::now();
        self
    }

    pub fn with_embedding(mut self, config: EmbeddingConfig) -> Self {
        self.embedding_config = config;
        self.updated_at = Utc::now();
        self
    }

    pub fn with_vector_db(mut self, config: VectorDBConfig) -> Self {
        self.vector_db_config = config;
        self.updated_at = Utc::now();
        self
    }

    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self.updated_at = Utc::now();
        self
    }

    pub fn with_knowledge_collection(mut self, collection: impl Into<String>) -> Self {
        self.knowledge_collection = Some(collection.into());
        self.updated_at = Utc::now();
        self
    }
}

/// LLM provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub provider: LLMProvider,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            provider: LLMProvider::OpenAI,
            model: "gpt-4".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
            api_key: None,
            base_url: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LLMProvider {
    OpenAI,
    Anthropic,
    Mistral,
    Groq,
    Together,
    Cohere,
    Gemini,
    Bedrock,
    AzureOpenAI,
    Local,
}

/// Embedding provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub provider: EmbeddingProvider,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub dimension: usize,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            provider: EmbeddingProvider::OpenAI,
            model: "text-embedding-3-small".to_string(),
            api_key: None,
            base_url: None,
            dimension: 1536,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EmbeddingProvider {
    OpenAI,
    Cohere,
    Voyage,
    Jina,
    Mistral,
    Ollama,
    Mock,
}

/// Vector database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorDBConfig {
    pub provider: VectorDBProvider,
    pub url: Option<String>,
    pub api_key: Option<String>,
    pub collection_name: String,
}

impl Default for VectorDBConfig {
    fn default() -> Self {
        Self {
            provider: VectorDBProvider::InMemory,
            url: None,
            api_key: None,
            collection_name: "default".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VectorDBProvider {
    InMemory,
    Qdrant,
    Pinecone,
    Weaviate,
    Chroma,
}

/// Agent store for managing agents
pub struct AgentStore {
    agents: Arc<RwLock<HashMap<String, Agent>>>,
}

impl AgentStore {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create(&self, agent: Agent) -> Agent {
        let id = agent.id.clone();
        self.agents.write().await.insert(id, agent.clone());
        agent
    }

    pub async fn get(&self, id: &str) -> Option<Agent> {
        self.agents.read().await.get(id).cloned()
    }

    pub async fn get_by_name(&self, name: &str) -> Option<Agent> {
        self.agents.read().await.values().find(|a| a.name == name).cloned()
    }

    pub async fn list(&self) -> Vec<Agent> {
        self.agents.read().await.values().cloned().collect()
    }

    pub async fn list_by_owner(&self, owner_id: &str) -> Vec<Agent> {
        self.agents.read().await.values()
            .filter(|a| a.owner_id == owner_id)
            .cloned()
            .collect()
    }

    pub async fn update(&self, agent: Agent) -> Option<Agent> {
        let mut agents = self.agents.write().await;
        if agents.contains_key(&agent.id) {
            let mut updated = agent.clone();
            updated.updated_at = Utc::now();
            agents.insert(agent.id.clone(), updated.clone());
            Some(updated)
        } else {
            None
        }
    }

    pub async fn update_llm_config(&self, id: &str, config: LLMConfig) -> Option<Agent> {
        let mut agents = self.agents.write().await;
        if let Some(agent) = agents.get_mut(id) {
            agent.llm_config = config;
            agent.updated_at = Utc::now();
            Some(agent.clone())
        } else {
            None
        }
    }

    pub async fn update_embedding_config(&self, id: &str, config: EmbeddingConfig) -> Option<Agent> {
        let mut agents = self.agents.write().await;
        if let Some(agent) = agents.get_mut(id) {
            agent.embedding_config = config;
            agent.updated_at = Utc::now();
            Some(agent.clone())
        } else {
            None
        }
    }

    pub async fn update_vector_db_config(&self, id: &str, config: VectorDBConfig) -> Option<Agent> {
        let mut agents = self.agents.write().await;
        if let Some(agent) = agents.get_mut(id) {
            agent.vector_db_config = config;
            agent.updated_at = Utc::now();
            Some(agent.clone())
        } else {
            None
        }
    }

    pub async fn delete(&self, id: &str) -> bool {
        self.agents.write().await.remove(id).is_some()
    }

    pub async fn toggle_enabled(&self, id: &str) -> Option<bool> {
        let mut agents = self.agents.write().await;
        if let Some(agent) = agents.get_mut(id) {
            agent.enabled = !agent.enabled;
            agent.updated_at = Utc::now();
            Some(agent.enabled)
        } else {
            None
        }
    }
}

impl Default for AgentStore {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_agent() {
        let store = AgentStore::new();
        let agent = Agent::new("Test Agent", "user1")
            .with_description("A test agent")
            .with_llm(LLMConfig {
                provider: LLMProvider::Anthropic,
                model: "claude-3-opus".into(),
                ..Default::default()
            });
        
        let created = store.create(agent).await;
        assert_eq!(created.name, "Test Agent");
        assert_eq!(created.llm_config.provider, LLMProvider::Anthropic);
    }

    #[tokio::test]
    async fn test_update_config() {
        let store = AgentStore::new();
        let agent = Agent::new("Test", "user1");
        store.create(agent.clone()).await;
        
        let new_llm = LLMConfig {
            provider: LLMProvider::Mistral,
            model: "mistral-large".into(),
            ..Default::default()
        };
        
        let updated = store.update_llm_config(&agent.id, new_llm).await.unwrap();
        assert_eq!(updated.llm_config.provider, LLMProvider::Mistral);
    }

    #[tokio::test]
    async fn test_list_by_owner() {
        let store = AgentStore::new();
        store.create(Agent::new("Agent1", "user1")).await;
        store.create(Agent::new("Agent2", "user1")).await;
        store.create(Agent::new("Agent3", "user2")).await;
        
        let user1_agents = store.list_by_owner("user1").await;
        assert_eq!(user1_agents.len(), 2);
    }
}
