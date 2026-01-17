//! Knowledge and RAG system
//!
//! Provides document ingestion, chunking, embeddings, vector storage, and RAG.

mod types;
pub mod embeddings;
mod chunker;
mod vector_store;
pub mod vector_db;
mod rag;

pub use types::*;
pub use embeddings::{EmbeddingProvider, create_embedding_provider, EmbeddingProviderType};
pub use chunker::DocumentChunker;
pub use vector_store::VectorStore;
pub use vector_db::{VectorDB, VectorDBType, VectorItem, VectorSearchResult, create_vector_db};
pub use rag::RAGEngine;
