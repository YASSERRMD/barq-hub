//! Knowledge API handlers

use axum::{extract::{Path, State, Query}, http::StatusCode, Json};
use std::sync::Arc;
use crate::api::state::AppState;
use crate::knowledge::{Document, SearchQuery, SearchResult, RAGContext};
use crate::error::Result;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct IngestRequest {
    pub title: String,
    pub content: String,
    pub source: Option<String>,
}

#[derive(Serialize)]
pub struct IngestResponse {
    pub document_id: String,
    pub chunks_created: usize,
}

#[derive(Deserialize)]
pub struct SearchParams {
    pub q: String,
    pub top_k: Option<usize>,
    pub min_score: Option<f32>,
}

pub async fn ingest_document(
    State(state): State<Arc<AppState>>,
    Json(req): Json<IngestRequest>,
) -> Result<(StatusCode, Json<IngestResponse>)> {
    let doc = Document::new(&req.title, &req.content, req.source.as_deref().unwrap_or("api"));
    let doc_id = doc.id.clone();
    let chunks = state.rag_engine.ingest_document(doc).await?;
    Ok((StatusCode::CREATED, Json(IngestResponse { document_id: doc_id, chunks_created: chunks })))
}

pub async fn search_knowledge(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<SearchResult>>> {
    let query = SearchQuery {
        query: params.q,
        top_k: params.top_k.unwrap_or(5),
        min_score: params.min_score,
        filters: std::collections::HashMap::new(),
    };
    let results = state.rag_engine.search(query).await?;
    Ok(Json(results))
}

pub async fn get_rag_context(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchParams>,
) -> Result<Json<RAGContext>> {
    let context = state.rag_engine.build_context(&params.q, params.top_k.unwrap_or(5)).await?;
    Ok(Json(context))
}

pub async fn delete_document(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let deleted = state.rag_engine.delete_document(&id).await?;
    Ok(Json(serde_json::json!({"deleted_chunks": deleted})))
}

pub async fn knowledge_stats(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let chunk_count = state.rag_engine.document_count().await;
    
    // Get collections from database if connected
    let mut collections = Vec::new();
    if let Some(ref pool) = state.db_pool {
        if let Ok(rows) = sqlx::query_as::<_, (String, String, Option<String>, i32, i32)>(
            "SELECT id, name, description, document_count, chunk_count FROM knowledge_collections ORDER BY name"
        )
        .fetch_all(pool)
        .await {
            for (id, name, desc, doc_count, chunk_count) in rows {
                collections.push(serde_json::json!({
                    "id": id,
                    "name": name,
                    "description": desc.unwrap_or_default(),
                    "document_count": doc_count,
                    "chunk_count": chunk_count
                }));
            }
        }
    }
    
    Json(serde_json::json!({
        "total_chunks": chunk_count,
        "collections": collections,
        "documents": []
    }))
}

