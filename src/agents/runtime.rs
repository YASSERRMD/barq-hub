//! Agent runtime execution

use std::sync::Arc;
use std::collections::HashMap;

use crate::agents::{Agent, AgentStore};
use crate::agents::registry::ProviderRegistry;
use crate::{ChatRequest, ChatResponse, Message};
use crate::knowledge::{RAGEngine, SearchQuery, DocumentChunker, VectorStore};
use crate::error::{Result, SynapseError};

/// Agent execution runtime
pub struct AgentRuntime {
    agent_store: Arc<AgentStore>,
    provider_registry: Arc<ProviderRegistry>,
}

impl AgentRuntime {
    pub fn new(agent_store: Arc<AgentStore>, provider_registry: Arc<ProviderRegistry>) -> Self {
        Self { agent_store, provider_registry }
    }

    /// Execute a chat request using the agent's configuration
    pub async fn chat(&self, agent_id: &str, user_message: &str, context: Option<HashMap<String, String>>) -> Result<ChatResponse> {
        let agent = self.agent_store.get(agent_id).await
            .ok_or_else(|| SynapseError::Validation("Agent not found".into()))?;

        if !agent.enabled {
            return Err(SynapseError::Validation("Agent is disabled".into()));
        }

        // Get LLM adapter for this agent
        let adapter = self.provider_registry.get_llm_adapter(&agent).await
            .ok_or_else(|| SynapseError::Validation("No LLM configured for agent".into()))?;

        // Build messages
        let mut messages = Vec::new();

        // Add system prompt if configured
        if let Some(ref system_prompt) = agent.system_prompt {
            messages.push(Message::system(system_prompt));
        }

        // If agent has RAG enabled, get context
        if agent.knowledge_collection.is_some() {
            if let Some(rag_context) = self.get_rag_context(&agent, user_message).await? {
                messages.push(Message::system(&format!(
                    "Use the following context to help answer:\n\n{}", rag_context
                )));
            }
        }

        // Add user message
        messages.push(Message::user(user_message));

        // Create request
        let request = ChatRequest {
            model: agent.llm_config.model.clone(),
            messages,
            temperature: agent.llm_config.temperature,
            max_tokens: agent.llm_config.max_tokens,
            ..Default::default()
        };

        // Execute
        adapter.chat(&request).await
    }

    /// Get RAG context for the agent
    async fn get_rag_context(&self, agent: &Agent, query: &str) -> Result<Option<String>> {
        let collection = match &agent.knowledge_collection {
            Some(c) => c,
            None => return Ok(None),
        };

        // Get providers for this agent
        let embedding_provider = match self.provider_registry.get_embedding_provider(agent).await {
            Some(p) => p,
            None => return Ok(None),
        };

        let vector_db = self.provider_registry.get_vector_db(agent).await;

        // Embed the query
        let query_embedding = embedding_provider.embed(query).await?;

        // Search vector DB
        let results = vector_db.search(collection, &query_embedding, 5).await?;

        if results.is_empty() {
            return Ok(None);
        }

        // Build context string
        let context: String = results.iter()
            .enumerate()
            .map(|(i, r)| format!("[{}] {}", i + 1, r.metadata.get("content").unwrap_or(&r.id)))
            .collect::<Vec<_>>()
            .join("\n\n");

        Ok(Some(context))
    }

    /// Ingest a document into the agent's knowledge base
    pub async fn ingest_document(&self, agent_id: &str, title: &str, content: &str) -> Result<usize> {
        let agent = self.agent_store.get(agent_id).await
            .ok_or_else(|| SynapseError::Validation("Agent not found".into()))?;

        let collection = agent.knowledge_collection.as_ref()
            .ok_or_else(|| SynapseError::Validation("Agent has no knowledge collection".into()))?;

        // Get providers
        let embedding_provider = self.provider_registry.get_embedding_provider(&agent).await
            .ok_or_else(|| SynapseError::Validation("No embedding provider configured".into()))?;

        let vector_db = self.provider_registry.get_vector_db(&agent).await;

        // Ensure collection exists
        let _ = vector_db.create_collection(collection, embedding_provider.dimension()).await;

        // Chunk the document
        let chunker = DocumentChunker::default();
        let doc_id = uuid::Uuid::new_v4().to_string();
        let chunks = chunker.chunk(&doc_id, content);

        // Embed and store each chunk
        let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
        let embeddings = embedding_provider.embed_batch(&texts).await?;

        let items: Vec<_> = chunks.iter().zip(embeddings.iter())
            .map(|(chunk, embedding)| {
                let mut metadata = HashMap::new();
                metadata.insert("title".to_string(), title.to_string());
                metadata.insert("content".to_string(), chunk.content.clone());
                metadata.insert("chunk_index".to_string(), chunk.chunk_index.to_string());
                
                crate::knowledge::vector_db::VectorItem {
                    id: chunk.id.clone(),
                    vector: embedding.clone(),
                    metadata,
                }
            })
            .collect();

        let count = items.len();
        vector_db.insert_batch(collection, items).await?;

        Ok(count)
    }

    /// Search the agent's knowledge base
    pub async fn search_knowledge(&self, agent_id: &str, query: &str, top_k: usize) -> Result<Vec<crate::knowledge::vector_db::VectorSearchResult>> {
        let agent = self.agent_store.get(agent_id).await
            .ok_or_else(|| SynapseError::Validation("Agent not found".into()))?;

        let collection = agent.knowledge_collection.as_ref()
            .ok_or_else(|| SynapseError::Validation("Agent has no knowledge collection".into()))?;

        let embedding_provider = self.provider_registry.get_embedding_provider(&agent).await
            .ok_or_else(|| SynapseError::Validation("No embedding provider configured".into()))?;

        let vector_db = self.provider_registry.get_vector_db(&agent).await;

        let query_embedding = embedding_provider.embed(query).await?;
        vector_db.search(collection, &query_embedding, top_k).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::{LLMConfig, LLMProvider, EmbeddingConfig, EmbeddingProvider as EP, VectorDBConfig, VectorDBProvider};

    #[tokio::test]
    async fn test_agent_runtime_creation() {
        let store = Arc::new(AgentStore::new());
        let registry = Arc::new(ProviderRegistry::new());
        let _runtime = AgentRuntime::new(store, registry);
    }

    #[tokio::test]
    async fn test_ingest_with_mock() {
        let store = Arc::new(AgentStore::new());
        let registry = Arc::new(ProviderRegistry::new());
        let runtime = AgentRuntime::new(store.clone(), registry);

        // Create agent with mock embedding and in-memory DB
        let agent = Agent::new("Test Agent", "user1")
            .with_embedding(EmbeddingConfig {
                provider: EP::Mock,
                model: "mock".into(),
                dimension: 128,
                ..Default::default()
            })
            .with_vector_db(VectorDBConfig {
                provider: VectorDBProvider::InMemory,
                collection_name: "test".into(),
                ..Default::default()
            })
            .with_knowledge_collection("test");

        store.create(agent.clone()).await;

        // Ingest document
        let count = runtime.ingest_document(&agent.id, "Test Doc", "Hello world this is test content").await.unwrap();
        assert!(count > 0);

        // Search
        let results = runtime.search_knowledge(&agent.id, "hello", 5).await.unwrap();
        assert!(!results.is_empty());
    }
}
