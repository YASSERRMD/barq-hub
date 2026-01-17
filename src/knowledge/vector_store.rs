//! Vector store for semantic search

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::knowledge::{Chunk, SearchResult};
use crate::error::Result;

pub struct VectorStore {
    chunks: Arc<RwLock<HashMap<String, Chunk>>>,
    documents: Arc<RwLock<HashMap<String, String>>>, // doc_id -> title
}

impl VectorStore {
    pub fn new() -> Self {
        Self {
            chunks: Arc::new(RwLock::new(HashMap::new())),
            documents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_document(&self, doc_id: &str, title: &str) {
        self.documents.write().await.insert(doc_id.into(), title.into());
    }

    pub async fn add_chunk(&self, chunk: Chunk) -> Result<()> {
        self.chunks.write().await.insert(chunk.id.clone(), chunk);
        Ok(())
    }

    pub async fn add_chunks(&self, chunks: Vec<Chunk>) -> Result<()> {
        let mut store = self.chunks.write().await;
        for chunk in chunks {
            store.insert(chunk.id.clone(), chunk);
        }
        Ok(())
    }

    pub async fn search(&self, query_embedding: &[f32], top_k: usize, min_score: Option<f32>) -> Vec<SearchResult> {
        let chunks = self.chunks.read().await;
        let docs = self.documents.read().await;
        
        let mut results: Vec<_> = chunks.values()
            .filter_map(|chunk| {
                chunk.embedding.as_ref().map(|emb| {
                    let score = Self::cosine_similarity(query_embedding, emb);
                    (chunk.clone(), score)
                })
            })
            .filter(|(_, score)| min_score.map_or(true, |min| *score >= min))
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        results.into_iter()
            .take(top_k)
            .map(|(chunk, score)| {
                let title = docs.get(&chunk.document_id).cloned().unwrap_or_default();
                SearchResult { chunk, score, document_title: title }
            })
            .collect()
    }

    pub async fn delete_document(&self, doc_id: &str) -> Result<usize> {
        let mut chunks = self.chunks.write().await;
        let before = chunks.len();
        chunks.retain(|_, c| c.document_id != doc_id);
        self.documents.write().await.remove(doc_id);
        Ok(before - chunks.len())
    }

    pub async fn count(&self) -> usize {
        self.chunks.read().await.len()
    }

    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() { return 0.0; }
        
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm_a == 0.0 || norm_b == 0.0 { 0.0 } else { dot / (norm_a * norm_b) }
    }
}

impl Default for VectorStore {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_and_search() {
        let store = VectorStore::new();
        store.add_document("doc1", "Test Doc").await;
        
        let chunk = Chunk::new("doc1", "Hello", 0, 5, 0)
            .with_embedding(vec![1.0, 0.0, 0.0]);
        store.add_chunk(chunk).await.unwrap();

        let results = store.search(&[1.0, 0.0, 0.0], 5, None).await;
        assert_eq!(results.len(), 1);
        assert!((results[0].score - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((VectorStore::cosine_similarity(&a, &b) - 1.0).abs() < 0.01);

        let c = vec![0.0, 1.0, 0.0];
        assert!(VectorStore::cosine_similarity(&a, &c).abs() < 0.01);
    }
}
