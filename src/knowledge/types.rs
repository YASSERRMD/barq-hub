//! Knowledge types

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub title: String,
    pub content: String,
    pub source: String,
    pub mime_type: String,
    pub metadata: std::collections::HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Document {
    pub fn new(title: impl Into<String>, content: impl Into<String>, source: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title: title.into(),
            content: content.into(),
            source: source.into(),
            mime_type: "text/plain".into(),
            metadata: std::collections::HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub id: String,
    pub document_id: String,
    pub content: String,
    pub start_offset: usize,
    pub end_offset: usize,
    pub chunk_index: usize,
    pub embedding: Option<Vec<f32>>,
    pub metadata: std::collections::HashMap<String, String>,
}

impl Chunk {
    pub fn new(document_id: &str, content: impl Into<String>, start: usize, end: usize, index: usize) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            document_id: document_id.into(),
            content: content.into(),
            start_offset: start,
            end_offset: end,
            chunk_index: index,
            embedding: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub chunk: Chunk,
    pub score: f32,
    pub document_title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub top_k: usize,
    pub min_score: Option<f32>,
    pub filters: std::collections::HashMap<String, String>,
}

impl SearchQuery {
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            top_k: 5,
            min_score: None,
            filters: std::collections::HashMap::new(),
        }
    }

    pub fn with_top_k(mut self, k: usize) -> Self {
        self.top_k = k;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGContext {
    pub query: String,
    pub chunks: Vec<SearchResult>,
    pub augmented_prompt: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_creation() {
        let doc = Document::new("Test", "Content", "test.txt");
        assert_eq!(doc.title, "Test");
        assert!(!doc.id.is_empty());
    }

    #[test]
    fn test_chunk_creation() {
        let chunk = Chunk::new("doc1", "Hello world", 0, 11, 0);
        assert_eq!(chunk.document_id, "doc1");
        assert!(chunk.embedding.is_none());
    }
}
