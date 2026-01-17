//! Vector database providers

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::knowledge::{Chunk, SearchResult};
use crate::error::{Result, SynapseError};

/// Trait for vector database providers
#[async_trait]
pub trait VectorDB: Send + Sync {
    async fn create_collection(&self, name: &str, dimension: usize) -> Result<()>;
    async fn delete_collection(&self, name: &str) -> Result<()>;
    async fn insert(&self, collection: &str, id: &str, vector: &[f32], metadata: HashMap<String, String>) -> Result<()>;
    async fn insert_batch(&self, collection: &str, items: Vec<VectorItem>) -> Result<()>;
    async fn search(&self, collection: &str, vector: &[f32], top_k: usize) -> Result<Vec<VectorSearchResult>>;
    async fn delete(&self, collection: &str, id: &str) -> Result<()>;
    async fn count(&self, collection: &str) -> Result<usize>;
    fn name(&self) -> &str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorItem {
    pub id: String,
    pub vector: Vec<f32>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchResult {
    pub id: String,
    pub score: f32,
    pub metadata: HashMap<String, String>,
}

// ============================================================================
// Qdrant Vector Database
// ============================================================================

pub struct QdrantDB {
    client: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
}

impl QdrantDB {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.into(),
            api_key: None,
        }
    }

    pub fn with_api_key(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.into(),
            api_key: Some(api_key.into()),
        }
    }

    pub fn cloud(url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self::with_api_key(url, api_key)
    }

    fn build_request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.client.request(method, &url);
        if let Some(ref key) = self.api_key {
            req = req.header("api-key", key);
        }
        req
    }
}

#[async_trait]
impl VectorDB for QdrantDB {
    async fn create_collection(&self, name: &str, dimension: usize) -> Result<()> {
        let resp = self.build_request(reqwest::Method::PUT, &format!("/collections/{}", name))
            .json(&serde_json::json!({
                "vectors": {
                    "size": dimension,
                    "distance": "Cosine"
                }
            }))
            .send().await
            .map_err(|e| SynapseError::Internal(e.to_string()))?;

        if !resp.status().is_success() && resp.status().as_u16() != 409 {
            return Err(SynapseError::Internal(format!("Failed to create collection: {}", resp.status())));
        }
        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        self.build_request(reqwest::Method::DELETE, &format!("/collections/{}", name))
            .send().await
            .map_err(|e| SynapseError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn insert(&self, collection: &str, id: &str, vector: &[f32], metadata: HashMap<String, String>) -> Result<()> {
        self.insert_batch(collection, vec![VectorItem {
            id: id.to_string(),
            vector: vector.to_vec(),
            metadata,
        }]).await
    }

    async fn insert_batch(&self, collection: &str, items: Vec<VectorItem>) -> Result<()> {
        let points: Vec<_> = items.iter().enumerate().map(|(i, item)| {
            serde_json::json!({
                "id": i as u64 + 1,
                "vector": item.vector,
                "payload": {
                    "doc_id": item.id,
                    "metadata": item.metadata
                }
            })
        }).collect();

        self.build_request(reqwest::Method::PUT, &format!("/collections/{}/points", collection))
            .json(&serde_json::json!({ "points": points }))
            .send().await
            .map_err(|e| SynapseError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn search(&self, collection: &str, vector: &[f32], top_k: usize) -> Result<Vec<VectorSearchResult>> {
        let resp = self.build_request(reqwest::Method::POST, &format!("/collections/{}/points/search", collection))
            .json(&serde_json::json!({
                "vector": vector,
                "limit": top_k,
                "with_payload": true
            }))
            .send().await
            .map_err(|e| SynapseError::Internal(e.to_string()))?;

        let body: serde_json::Value = resp.json().await
            .map_err(|e| SynapseError::Internal(e.to_string()))?;

        let results = body["result"].as_array()
            .ok_or_else(|| SynapseError::Internal("No results".into()))?
            .iter()
            .filter_map(|r| {
                let id = r["payload"]["doc_id"].as_str()?.to_string();
                let score = r["score"].as_f64()? as f32;
                let metadata = r["payload"]["metadata"].as_object()
                    .map(|m| m.iter().filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string()))).collect())
                    .unwrap_or_default();
                Some(VectorSearchResult { id, score, metadata })
            })
            .collect();

        Ok(results)
    }

    async fn delete(&self, collection: &str, id: &str) -> Result<()> {
        self.build_request(reqwest::Method::POST, &format!("/collections/{}/points/delete", collection))
            .json(&serde_json::json!({
                "filter": {
                    "must": [{ "key": "doc_id", "match": { "value": id } }]
                }
            }))
            .send().await
            .map_err(|e| SynapseError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn count(&self, collection: &str) -> Result<usize> {
        let resp = self.build_request(reqwest::Method::GET, &format!("/collections/{}", collection))
            .send().await
            .map_err(|e| SynapseError::Internal(e.to_string()))?;

        let body: serde_json::Value = resp.json().await
            .map_err(|e| SynapseError::Internal(e.to_string()))?;

        Ok(body["result"]["points_count"].as_u64().unwrap_or(0) as usize)
    }

    fn name(&self) -> &str { "qdrant" }
}

// ============================================================================
// Pinecone Vector Database
// ============================================================================

pub struct PineconeDB {
    client: reqwest::Client,
    host: String,
    api_key: String,
}

impl PineconeDB {
    pub fn new(host: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            host: host.into(),
            api_key: api_key.into(),
        }
    }
}

#[async_trait]
impl VectorDB for PineconeDB {
    async fn create_collection(&self, _name: &str, _dimension: usize) -> Result<()> {
        // Pinecone indexes are created via console/API separately
        Ok(())
    }

    async fn delete_collection(&self, _name: &str) -> Result<()> {
        Ok(())
    }

    async fn insert(&self, _collection: &str, id: &str, vector: &[f32], metadata: HashMap<String, String>) -> Result<()> {
        self.insert_batch(_collection, vec![VectorItem {
            id: id.to_string(),
            vector: vector.to_vec(),
            metadata,
        }]).await
    }

    async fn insert_batch(&self, _collection: &str, items: Vec<VectorItem>) -> Result<()> {
        let vectors: Vec<_> = items.iter().map(|item| {
            serde_json::json!({
                "id": item.id,
                "values": item.vector,
                "metadata": item.metadata
            })
        }).collect();

        self.client.post(format!("{}/vectors/upsert", self.host))
            .header("Api-Key", &self.api_key)
            .json(&serde_json::json!({ "vectors": vectors }))
            .send().await
            .map_err(|e| SynapseError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn search(&self, _collection: &str, vector: &[f32], top_k: usize) -> Result<Vec<VectorSearchResult>> {
        let resp = self.client.post(format!("{}/query", self.host))
            .header("Api-Key", &self.api_key)
            .json(&serde_json::json!({
                "vector": vector,
                "topK": top_k,
                "includeMetadata": true
            }))
            .send().await
            .map_err(|e| SynapseError::Internal(e.to_string()))?;

        let body: serde_json::Value = resp.json().await
            .map_err(|e| SynapseError::Internal(e.to_string()))?;

        let results = body["matches"].as_array()
            .ok_or_else(|| SynapseError::Internal("No matches".into()))?
            .iter()
            .filter_map(|r| {
                let id = r["id"].as_str()?.to_string();
                let score = r["score"].as_f64()? as f32;
                let metadata = r["metadata"].as_object()
                    .map(|m| m.iter().filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string()))).collect())
                    .unwrap_or_default();
                Some(VectorSearchResult { id, score, metadata })
            })
            .collect();

        Ok(results)
    }

    async fn delete(&self, _collection: &str, id: &str) -> Result<()> {
        self.client.post(format!("{}/vectors/delete", self.host))
            .header("Api-Key", &self.api_key)
            .json(&serde_json::json!({ "ids": [id] }))
            .send().await
            .map_err(|e| SynapseError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn count(&self, _collection: &str) -> Result<usize> {
        let resp = self.client.post(format!("{}/describe_index_stats", self.host))
            .header("Api-Key", &self.api_key)
            .json(&serde_json::json!({}))
            .send().await
            .map_err(|e| SynapseError::Internal(e.to_string()))?;

        let body: serde_json::Value = resp.json().await
            .map_err(|e| SynapseError::Internal(e.to_string()))?;

        Ok(body["totalVectorCount"].as_u64().unwrap_or(0) as usize)
    }

    fn name(&self) -> &str { "pinecone" }
}

// ============================================================================
// Weaviate Vector Database
// ============================================================================

pub struct WeaviateDB {
    client: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
}

impl WeaviateDB {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.into(),
            api_key: None,
        }
    }

    pub fn with_api_key(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.into(),
            api_key: Some(api_key.into()),
        }
    }

    fn build_request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.client.request(method, &url);
        if let Some(ref key) = self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }
        req
    }
}

#[async_trait]
impl VectorDB for WeaviateDB {
    async fn create_collection(&self, name: &str, dimension: usize) -> Result<()> {
        self.build_request(reqwest::Method::POST, "/v1/schema")
            .json(&serde_json::json!({
                "class": name,
                "vectorizer": "none",
                "properties": [{
                    "name": "content",
                    "dataType": ["text"]
                }]
            }))
            .send().await
            .map_err(|e| SynapseError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        self.build_request(reqwest::Method::DELETE, &format!("/v1/schema/{}", name))
            .send().await
            .map_err(|e| SynapseError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn insert(&self, collection: &str, id: &str, vector: &[f32], metadata: HashMap<String, String>) -> Result<()> {
        self.build_request(reqwest::Method::POST, &format!("/v1/objects"))
            .json(&serde_json::json!({
                "class": collection,
                "id": id,
                "vector": vector,
                "properties": metadata
            }))
            .send().await
            .map_err(|e| SynapseError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn insert_batch(&self, collection: &str, items: Vec<VectorItem>) -> Result<()> {
        let objects: Vec<_> = items.iter().map(|item| {
            serde_json::json!({
                "class": collection,
                "id": item.id,
                "vector": item.vector,
                "properties": item.metadata
            })
        }).collect();

        self.build_request(reqwest::Method::POST, "/v1/batch/objects")
            .json(&serde_json::json!({ "objects": objects }))
            .send().await
            .map_err(|e| SynapseError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn search(&self, collection: &str, vector: &[f32], top_k: usize) -> Result<Vec<VectorSearchResult>> {
        let query = format!(r#"{{
            Get {{
                {}(nearVector: {{vector: {:?}}}, limit: {}) {{
                    _additional {{ id distance }}
                }}
            }}
        }}"#, collection, vector, top_k);

        let resp = self.build_request(reqwest::Method::POST, "/v1/graphql")
            .json(&serde_json::json!({ "query": query }))
            .send().await
            .map_err(|e| SynapseError::Internal(e.to_string()))?;

        let body: serde_json::Value = resp.json().await
            .map_err(|e| SynapseError::Internal(e.to_string()))?;

        let path = format!("data.Get.{}", collection);
        let results = body.pointer(&path.replace(".", "/"))
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|r| {
                let id = r["_additional"]["id"].as_str()?.to_string();
                let distance = r["_additional"]["distance"].as_f64()? as f32;
                Some(VectorSearchResult {
                    id,
                    score: 1.0 - distance, // Convert distance to similarity
                    metadata: HashMap::new(),
                })
            }).collect())
            .unwrap_or_default();

        Ok(results)
    }

    async fn delete(&self, _collection: &str, id: &str) -> Result<()> {
        self.build_request(reqwest::Method::DELETE, &format!("/v1/objects/{}", id))
            .send().await
            .map_err(|e| SynapseError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn count(&self, collection: &str) -> Result<usize> {
        let query = format!(r#"{{ Aggregate {{ {}{{ meta {{ count }} }} }} }}"#, collection);
        
        let resp = self.build_request(reqwest::Method::POST, "/v1/graphql")
            .json(&serde_json::json!({ "query": query }))
            .send().await
            .map_err(|e| SynapseError::Internal(e.to_string()))?;

        let body: serde_json::Value = resp.json().await
            .map_err(|e| SynapseError::Internal(e.to_string()))?;

        Ok(body["data"]["Aggregate"][collection][0]["meta"]["count"]
            .as_u64().unwrap_or(0) as usize)
    }

    fn name(&self) -> &str { "weaviate" }
}

// ============================================================================
// Chroma Vector Database
// ============================================================================

pub struct ChromaDB {
    client: reqwest::Client,
    base_url: String,
}

impl ChromaDB {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.into(),
        }
    }

    pub fn local() -> Self {
        Self::new("http://localhost:8000")
    }
}

#[async_trait]
impl VectorDB for ChromaDB {
    async fn create_collection(&self, name: &str, _dimension: usize) -> Result<()> {
        self.client.post(format!("{}/api/v1/collections", self.base_url))
            .json(&serde_json::json!({ "name": name }))
            .send().await
            .map_err(|e| SynapseError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        self.client.delete(format!("{}/api/v1/collections/{}", self.base_url, name))
            .send().await
            .map_err(|e| SynapseError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn insert(&self, collection: &str, id: &str, vector: &[f32], metadata: HashMap<String, String>) -> Result<()> {
        self.insert_batch(collection, vec![VectorItem {
            id: id.to_string(),
            vector: vector.to_vec(),
            metadata,
        }]).await
    }

    async fn insert_batch(&self, collection: &str, items: Vec<VectorItem>) -> Result<()> {
        let ids: Vec<_> = items.iter().map(|i| &i.id).collect();
        let embeddings: Vec<_> = items.iter().map(|i| &i.vector).collect();
        let metadatas: Vec<_> = items.iter().map(|i| &i.metadata).collect();

        self.client.post(format!("{}/api/v1/collections/{}/add", self.base_url, collection))
            .json(&serde_json::json!({
                "ids": ids,
                "embeddings": embeddings,
                "metadatas": metadatas
            }))
            .send().await
            .map_err(|e| SynapseError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn search(&self, collection: &str, vector: &[f32], top_k: usize) -> Result<Vec<VectorSearchResult>> {
        let resp = self.client.post(format!("{}/api/v1/collections/{}/query", self.base_url, collection))
            .json(&serde_json::json!({
                "query_embeddings": [vector],
                "n_results": top_k,
                "include": ["metadatas", "distances"]
            }))
            .send().await
            .map_err(|e| SynapseError::Internal(e.to_string()))?;

        let body: serde_json::Value = resp.json().await
            .map_err(|e| SynapseError::Internal(e.to_string()))?;

        let ids = body["ids"][0].as_array();
        let distances = body["distances"][0].as_array();
        let metadatas = body["metadatas"][0].as_array();

        let results = ids.zip(distances).zip(metadatas)
            .map(|((ids, dists), metas)| {
                ids.iter().zip(dists.iter()).zip(metas.iter())
                    .filter_map(|((id, dist), meta)| {
                        Some(VectorSearchResult {
                            id: id.as_str()?.to_string(),
                            score: 1.0 - dist.as_f64()? as f32,
                            metadata: meta.as_object()
                                .map(|m| m.iter().filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string()))).collect())
                                .unwrap_or_default(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(results)
    }

    async fn delete(&self, collection: &str, id: &str) -> Result<()> {
        self.client.post(format!("{}/api/v1/collections/{}/delete", self.base_url, collection))
            .json(&serde_json::json!({ "ids": [id] }))
            .send().await
            .map_err(|e| SynapseError::Internal(e.to_string()))?;
        Ok(())
    }

    async fn count(&self, collection: &str) -> Result<usize> {
        let resp = self.client.get(format!("{}/api/v1/collections/{}/count", self.base_url, collection))
            .send().await
            .map_err(|e| SynapseError::Internal(e.to_string()))?;

        let count: usize = resp.json().await
            .map_err(|e| SynapseError::Internal(e.to_string()))?;
        Ok(count)
    }

    fn name(&self) -> &str { "chroma" }
}

// ============================================================================
// In-Memory Vector Store (for development/testing)
// ============================================================================

use std::sync::Arc;
use tokio::sync::RwLock;

pub struct InMemoryVectorDB {
    collections: Arc<RwLock<HashMap<String, Vec<VectorItem>>>>,
}

impl InMemoryVectorDB {
    pub fn new() -> Self {
        Self { collections: Arc::new(RwLock::new(HashMap::new())) }
    }

    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() { return 0.0; }
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm_a == 0.0 || norm_b == 0.0 { 0.0 } else { dot / (norm_a * norm_b) }
    }
}

impl Default for InMemoryVectorDB {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl VectorDB for InMemoryVectorDB {
    async fn create_collection(&self, name: &str, _dimension: usize) -> Result<()> {
        self.collections.write().await.insert(name.to_string(), Vec::new());
        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        self.collections.write().await.remove(name);
        Ok(())
    }

    async fn insert(&self, collection: &str, id: &str, vector: &[f32], metadata: HashMap<String, String>) -> Result<()> {
        let mut cols = self.collections.write().await;
        let col = cols.entry(collection.to_string()).or_insert_with(Vec::new);
        col.push(VectorItem { id: id.to_string(), vector: vector.to_vec(), metadata });
        Ok(())
    }

    async fn insert_batch(&self, collection: &str, items: Vec<VectorItem>) -> Result<()> {
        let mut cols = self.collections.write().await;
        let col = cols.entry(collection.to_string()).or_insert_with(Vec::new);
        col.extend(items);
        Ok(())
    }

    async fn search(&self, collection: &str, vector: &[f32], top_k: usize) -> Result<Vec<VectorSearchResult>> {
        let cols = self.collections.read().await;
        let col = cols.get(collection).ok_or_else(|| SynapseError::Internal("Collection not found".into()))?;

        let mut results: Vec<_> = col.iter()
            .map(|item| {
                let score = Self::cosine_similarity(vector, &item.vector);
                VectorSearchResult { id: item.id.clone(), score, metadata: item.metadata.clone() }
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        Ok(results.into_iter().take(top_k).collect())
    }

    async fn delete(&self, collection: &str, id: &str) -> Result<()> {
        let mut cols = self.collections.write().await;
        if let Some(col) = cols.get_mut(collection) {
            col.retain(|item| item.id != id);
        }
        Ok(())
    }

    async fn count(&self, collection: &str) -> Result<usize> {
        let cols = self.collections.read().await;
        Ok(cols.get(collection).map(|c| c.len()).unwrap_or(0))
    }

    fn name(&self) -> &str { "in_memory" }
}

// ============================================================================
// Vector DB Factory
// ============================================================================

pub enum VectorDBType {
    InMemory,
    Qdrant { url: String, api_key: Option<String> },
    Pinecone { host: String, api_key: String },
    Weaviate { url: String, api_key: Option<String> },
    Chroma { url: String },
}

pub fn create_vector_db(db_type: VectorDBType) -> Box<dyn VectorDB> {
    match db_type {
        VectorDBType::InMemory => Box::new(InMemoryVectorDB::new()),
        VectorDBType::Qdrant { url, api_key } => {
            if let Some(key) = api_key {
                Box::new(QdrantDB::with_api_key(url, key))
            } else {
                Box::new(QdrantDB::new(url))
            }
        }
        VectorDBType::Pinecone { host, api_key } => Box::new(PineconeDB::new(host, api_key)),
        VectorDBType::Weaviate { url, api_key } => {
            if let Some(key) = api_key {
                Box::new(WeaviateDB::with_api_key(url, key))
            } else {
                Box::new(WeaviateDB::new(url))
            }
        }
        VectorDBType::Chroma { url } => Box::new(ChromaDB::new(url)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_db() {
        let db = InMemoryVectorDB::new();
        db.create_collection("test", 3).await.unwrap();
        
        db.insert("test", "1", &[1.0, 0.0, 0.0], HashMap::new()).await.unwrap();
        db.insert("test", "2", &[0.0, 1.0, 0.0], HashMap::new()).await.unwrap();
        
        let results = db.search("test", &[1.0, 0.0, 0.0], 2).await.unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "1");
        assert!((results[0].score - 1.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_in_memory_count() {
        let db = InMemoryVectorDB::new();
        db.create_collection("test", 3).await.unwrap();
        db.insert("test", "1", &[1.0, 0.0, 0.0], HashMap::new()).await.unwrap();
        
        assert_eq!(db.count("test").await.unwrap(), 1);
    }
}
