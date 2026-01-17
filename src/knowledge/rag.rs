//! RAG (Retrieval Augmented Generation) engine

use std::sync::Arc;
use crate::knowledge::{Document, Chunk, SearchQuery, SearchResult, RAGContext, EmbeddingProvider, DocumentChunker, VectorStore};
use crate::error::Result;

pub struct RAGEngine {
    embedding_provider: Arc<dyn EmbeddingProvider>,
    chunker: DocumentChunker,
    vector_store: Arc<VectorStore>,
}

impl RAGEngine {
    pub fn new(embedding_provider: Arc<dyn EmbeddingProvider>, vector_store: Arc<VectorStore>) -> Self {
        Self {
            embedding_provider,
            chunker: DocumentChunker::default(),
            vector_store,
        }
    }

    pub fn with_chunker(mut self, chunker: DocumentChunker) -> Self {
        self.chunker = chunker;
        self
    }

    pub async fn ingest_document(&self, document: Document) -> Result<usize> {
        // Add document reference
        self.vector_store.add_document(&document.id, &document.title).await;

        // Chunk document
        let chunks = self.chunker.chunk(&document.id, &document.content);
        
        // Get embeddings
        let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
        let embeddings = self.embedding_provider.embed_batch(&texts).await?;

        // Add chunks with embeddings
        let chunks_with_embeddings: Vec<Chunk> = chunks.into_iter()
            .zip(embeddings.into_iter())
            .map(|(c, e)| c.with_embedding(e))
            .collect();

        let count = chunks_with_embeddings.len();
        self.vector_store.add_chunks(chunks_with_embeddings).await?;

        Ok(count)
    }

    pub async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>> {
        let query_embedding = self.embedding_provider.embed(&query.query).await?;
        Ok(self.vector_store.search(&query_embedding, query.top_k, query.min_score).await)
    }

    pub async fn build_context(&self, query: &str, top_k: usize) -> Result<RAGContext> {
        let search_query = SearchQuery::new(query).with_top_k(top_k);
        let results = self.search(search_query).await?;

        let context_text: String = results.iter()
            .enumerate()
            .map(|(i, r)| format!("[{}] {}: {}", i + 1, r.document_title, r.chunk.content))
            .collect::<Vec<_>>()
            .join("\n\n");

        let augmented_prompt = format!(
            "Use the following context to answer the question.\n\nContext:\n{}\n\nQuestion: {}",
            context_text, query
        );

        Ok(RAGContext {
            query: query.into(),
            chunks: results,
            augmented_prompt,
        })
    }

    pub async fn delete_document(&self, doc_id: &str) -> Result<usize> {
        self.vector_store.delete_document(doc_id).await
    }

    pub async fn document_count(&self) -> usize {
        self.vector_store.count().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::embeddings::MockEmbedding;

    #[tokio::test]
    async fn test_ingest_and_search() {
        let embedding = Arc::new(MockEmbedding::new(128));
        let store = Arc::new(VectorStore::new());
        let rag = RAGEngine::new(embedding, store);

        let doc = Document::new("Test Doc", "This is a test document with some content.", "test.txt");
        let count = rag.ingest_document(doc).await.unwrap();
        assert!(count > 0);

        let results = rag.search(SearchQuery::new("test")).await.unwrap();
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_build_context() {
        let embedding = Arc::new(MockEmbedding::new(128));
        let store = Arc::new(VectorStore::new());
        let rag = RAGEngine::new(embedding, store);

        let doc = Document::new("Guide", "How to use this system effectively.", "guide.txt");
        rag.ingest_document(doc).await.unwrap();

        let context = rag.build_context("how to use", 3).await.unwrap();
        assert!(!context.augmented_prompt.is_empty());
        assert!(context.augmented_prompt.contains("how to use"));
    }
}
