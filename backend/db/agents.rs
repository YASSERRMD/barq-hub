//! Agent repository for database operations

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use serde_json::Value as JsonValue;
use super::DbPool;

#[derive(Debug, Clone, FromRow)]
pub struct AgentRow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub system_prompt: Option<String>,
    pub llm_config: JsonValue,
    pub embedding_config: Option<JsonValue>,
    pub vector_db_config: Option<JsonValue>,
    pub knowledge_collection: Option<String>,
    pub enabled: bool,
    pub status: String,
    pub metadata: Option<JsonValue>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
}

pub struct AgentRepository {
    pool: DbPool,
}

impl AgentRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        id: &str,
        name: &str,
        description: Option<&str>,
        system_prompt: Option<&str>,
        llm_config: &JsonValue,
        embedding_config: Option<&JsonValue>,
        vector_db_config: Option<&JsonValue>,
        created_by: Option<&str>,
    ) -> Result<AgentRow, sqlx::Error> {
        sqlx::query_as::<_, AgentRow>(
            r#"
            INSERT INTO agents (id, name, description, system_prompt, llm_config, embedding_config, vector_db_config, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .bind(system_prompt)
        .bind(llm_config)
        .bind(embedding_config)
        .bind(vector_db_config)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<AgentRow>, sqlx::Error> {
        sqlx::query_as::<_, AgentRow>("SELECT * FROM agents WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn list_all(&self) -> Result<Vec<AgentRow>, sqlx::Error> {
        sqlx::query_as::<_, AgentRow>("SELECT * FROM agents ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await
    }

    pub async fn list_by_status(&self, status: &str) -> Result<Vec<AgentRow>, sqlx::Error> {
        sqlx::query_as::<_, AgentRow>("SELECT * FROM agents WHERE status = $1 ORDER BY created_at DESC")
            .bind(status)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn update_llm_config(&self, id: &str, llm_config: &JsonValue) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("UPDATE agents SET llm_config = $1, updated_at = NOW() WHERE id = $2")
            .bind(llm_config)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn update_embedding_config(&self, id: &str, embedding_config: &JsonValue) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("UPDATE agents SET embedding_config = $1, updated_at = NOW() WHERE id = $2")
            .bind(embedding_config)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn update_vector_db_config(&self, id: &str, vector_db_config: &JsonValue) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("UPDATE agents SET vector_db_config = $1, updated_at = NOW() WHERE id = $2")
            .bind(vector_db_config)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn update_knowledge_collection(&self, id: &str, collection: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("UPDATE agents SET knowledge_collection = $1, updated_at = NOW() WHERE id = $2")
            .bind(collection)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn delete(&self, id: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM agents WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn count(&self) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM agents")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }
}
