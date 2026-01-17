//! Agent API handlers

use axum::{extract::{Path, State}, http::StatusCode, Json};
use std::sync::Arc;
use crate::api::state::AppState;
use crate::agents::{Agent, LLMConfig, EmbeddingConfig, VectorDBConfig};
use crate::error::Result;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    pub description: Option<String>,
    pub llm_config: Option<LLMConfig>,
    pub embedding_config: Option<EmbeddingConfig>,
    pub vector_db_config: Option<VectorDBConfig>,
    pub system_prompt: Option<String>,
    pub knowledge_collection: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateAgentRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub system_prompt: Option<String>,
    pub knowledge_collection: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Serialize)]
pub struct AgentListItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub llm_provider: String,
    pub llm_model: String,
    pub embedding_provider: String,
    pub vector_db_provider: String,
}

impl From<Agent> for AgentListItem {
    fn from(a: Agent) -> Self {
        Self {
            id: a.id,
            name: a.name,
            description: a.description,
            enabled: a.enabled,
            llm_provider: format!("{:?}", a.llm_config.provider).to_lowercase(),
            llm_model: a.llm_config.model,
            embedding_provider: format!("{:?}", a.embedding_config.provider).to_lowercase(),
            vector_db_provider: format!("{:?}", a.vector_db_config.provider).to_lowercase(),
        }
    }
}

/// GET /v1/agents - List all agents
pub async fn list_agents(State(state): State<Arc<AppState>>) -> Json<Vec<AgentListItem>> {
    // Try to get from database first
    if let Some(ref pool) = state.db_pool {
        if let Ok(rows) = sqlx::query_as::<_, (String, String, Option<String>, serde_json::Value, bool)>(
            "SELECT id, name, description, llm_config, enabled FROM agents ORDER BY created_at DESC"
        )
        .fetch_all(pool)
        .await {
            let items: Vec<AgentListItem> = rows.into_iter().map(|(id, name, desc, llm_config, enabled)| {
                let provider = llm_config.get("provider").and_then(|v| v.as_str()).unwrap_or("openai").to_string();
                let model = llm_config.get("model").and_then(|v| v.as_str()).unwrap_or("gpt-4o").to_string();
                AgentListItem {
                    id,
                    name,
                    description: desc.unwrap_or_default(),
                    enabled,
                    llm_provider: provider,
                    llm_model: model,
                    embedding_provider: "openai".to_string(),
                    vector_db_provider: "inmemory".to_string(),
                }
            }).collect();
            return Json(items);
        }
    }
    
    // Fallback to in-memory store
    let agents = state.agent_store.list().await;
    Json(agents.into_iter().map(AgentListItem::from).collect())
}

/// GET /v1/agents/:id - Get agent details
pub async fn get_agent(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Result<Json<Agent>> {
    match state.agent_store.get(&id).await {
        Some(a) => Ok(Json(a)),
        None => Err(crate::error::SynapseError::Validation("Agent not found".into())),
    }
}

/// POST /v1/agents - Create new agent
pub async fn create_agent(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateAgentRequest>,
) -> Result<(StatusCode, Json<Agent>)> {
    let mut agent = Agent::new(&req.name, "api");
    
    if let Some(desc) = req.description {
        agent = agent.with_description(desc);
    }
    if let Some(config) = req.llm_config {
        agent = agent.with_llm(config);
    }
    if let Some(config) = req.embedding_config {
        agent = agent.with_embedding(config);
    }
    if let Some(config) = req.vector_db_config {
        agent = agent.with_vector_db(config);
    }
    if let Some(prompt) = req.system_prompt {
        agent = agent.with_system_prompt(prompt);
    }
    if let Some(collection) = req.knowledge_collection {
        agent = agent.with_knowledge_collection(collection);
    }

    let created = state.agent_store.create(agent).await;
    Ok((StatusCode::CREATED, Json(created)))
}

/// PUT /v1/agents/:id - Update agent
pub async fn update_agent(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateAgentRequest>,
) -> Result<Json<Agent>> {
    let mut agent = state.agent_store.get(&id).await
        .ok_or_else(|| crate::error::SynapseError::Validation("Agent not found".into()))?;

    if let Some(name) = req.name { agent.name = name; }
    if let Some(desc) = req.description { agent.description = desc; }
    if let Some(prompt) = req.system_prompt { agent.system_prompt = Some(prompt); }
    if let Some(coll) = req.knowledge_collection { agent.knowledge_collection = Some(coll); }
    if let Some(enabled) = req.enabled { agent.enabled = enabled; }

    let updated = state.agent_store.update(agent).await
        .ok_or_else(|| crate::error::SynapseError::Internal("Failed to update".into()))?;

    // Invalidate cache
    state.provider_registry.invalidate_cache(&id).await;

    Ok(Json(updated))
}

/// DELETE /v1/agents/:id - Delete agent
pub async fn delete_agent(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> StatusCode {
    state.provider_registry.invalidate_cache(&id).await;
    if state.agent_store.delete(&id).await {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

/// PUT /v1/agents/:id/llm - Update LLM config
pub async fn update_llm_config(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(config): Json<LLMConfig>,
) -> Result<Json<Agent>> {
    let agent = state.agent_store.update_llm_config(&id, config).await
        .ok_or_else(|| crate::error::SynapseError::Validation("Agent not found".into()))?;
    state.provider_registry.invalidate_cache(&id).await;
    Ok(Json(agent))
}

/// PUT /v1/agents/:id/embedding - Update embedding config
pub async fn update_embedding_config(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(config): Json<EmbeddingConfig>,
) -> Result<Json<Agent>> {
    let agent = state.agent_store.update_embedding_config(&id, config).await
        .ok_or_else(|| crate::error::SynapseError::Validation("Agent not found".into()))?;
    state.provider_registry.invalidate_cache(&id).await;
    Ok(Json(agent))
}

/// PUT /v1/agents/:id/vectordb - Update vector DB config
pub async fn update_vectordb_config(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(config): Json<VectorDBConfig>,
) -> Result<Json<Agent>> {
    let agent = state.agent_store.update_vector_db_config(&id, config).await
        .ok_or_else(|| crate::error::SynapseError::Validation("Agent not found".into()))?;
    state.provider_registry.invalidate_cache(&id).await;
    Ok(Json(agent))
}

/// POST /v1/agents/:id/chat - Chat with agent
#[derive(Deserialize)]
pub struct ChatWithAgentRequest {
    pub message: String,
}

pub async fn chat_with_agent(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<ChatWithAgentRequest>,
) -> Result<Json<crate::ChatResponse>> {
    let response = state.agent_runtime.chat(&id, &req.message, None).await?;
    Ok(Json(response))
}

/// POST /v1/agents/:id/knowledge/ingest - Ingest document into agent's knowledge base
#[derive(Deserialize)]
pub struct IngestToAgentRequest {
    pub title: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct IngestToAgentResponse {
    pub chunks_created: usize,
}

pub async fn ingest_to_agent(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<IngestToAgentRequest>,
) -> Result<Json<IngestToAgentResponse>> {
    let count = state.agent_runtime.ingest_document(&id, &req.title, &req.content).await?;
    Ok(Json(IngestToAgentResponse { chunks_created: count }))
}

/// GET /v1/agents/:id/knowledge/search - Search agent's knowledge base
#[derive(Deserialize)]
pub struct SearchAgentKnowledgeParams {
    pub q: String,
    pub top_k: Option<usize>,
}

pub async fn search_agent_knowledge(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    axum::extract::Query(params): axum::extract::Query<SearchAgentKnowledgeParams>,
) -> Result<Json<Vec<crate::knowledge::vector_db::VectorSearchResult>>> {
    let results = state.agent_runtime.search_knowledge(&id, &params.q, params.top_k.unwrap_or(5)).await?;
    Ok(Json(results))
}

/// GET /v1/agents/providers - Get available provider options
#[derive(Serialize)]
pub struct ProvidersInfo {
    pub llm_providers: Vec<&'static str>,
    pub embedding_providers: Vec<&'static str>,
    pub vector_db_providers: Vec<&'static str>,
}

pub async fn list_provider_options() -> Json<ProvidersInfo> {
    Json(ProvidersInfo {
        llm_providers: vec![
            "openai", "anthropic", "mistral", "groq", "together", "cohere",
            "gemini", "bedrock", "azureopenai", "local"
        ],
        embedding_providers: vec![
            "openai", "cohere", "voyage", "jina", "mistral", "ollama",
            "gemini", "azureopenai", "bedrock", "mock"
        ],
        vector_db_providers: vec!["inmemory", "qdrant", "pinecone", "weaviate", "chroma"],
    })
}
