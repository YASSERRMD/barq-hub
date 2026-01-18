//! Provider Account repository for database operations

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use serde_json::Value as JsonValue;
use super::DbPool;

#[derive(Debug, Clone, FromRow)]
pub struct ProviderAccountRow {
    pub id: String,
    pub provider_id: String,
    pub name: String,
    pub api_key_encrypted: Option<String>,
    pub enabled: bool,
    pub is_default: bool,
    pub priority: i32,
    pub endpoint: Option<String>,
    pub region: Option<String>,
    pub deployment_name: Option<String>,
    pub quota_config: JsonValue,
    pub models: JsonValue,
    pub config: JsonValue,
    pub metadata: JsonValue,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct ProviderAccountRepository {
    pool: DbPool,
}

impl ProviderAccountRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        id: &str,
        provider_id: &str,
        name: &str,
        api_key_encrypted: &str,
        endpoint: Option<&str>,
        region: Option<&str>,
        deployment_name: Option<&str>,
        models: &JsonValue,
        quota_config: &JsonValue,
        config: &JsonValue,
    ) -> Result<ProviderAccountRow, sqlx::Error> {
        sqlx::query_as::<_, ProviderAccountRow>(
            r#"
            INSERT INTO provider_accounts (id, provider_id, name, api_key_encrypted, endpoint, region, deployment_name, models, quota_config, config, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(id)
        .bind(provider_id)
        .bind(name)
        .bind(api_key_encrypted)
        .bind(endpoint)
        .bind(region)
        .bind(deployment_name)
        .bind(models)
        .bind(quota_config)
        .bind(config)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<ProviderAccountRow>, sqlx::Error> {
        sqlx::query_as::<_, ProviderAccountRow>("SELECT * FROM provider_accounts WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn list_by_provider(&self, provider_id: &str) -> Result<Vec<ProviderAccountRow>, sqlx::Error> {
        sqlx::query_as::<_, ProviderAccountRow>(
            "SELECT * FROM provider_accounts WHERE provider_id = $1 ORDER BY priority DESC, is_default DESC"
        )
        .bind(provider_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn list_all(&self) -> Result<Vec<ProviderAccountRow>, sqlx::Error> {
        sqlx::query_as::<_, ProviderAccountRow>("SELECT * FROM provider_accounts ORDER BY provider_id, priority DESC")
            .fetch_all(&self.pool)
            .await
    }

    pub async fn get_default(&self, provider_id: &str) -> Result<Option<ProviderAccountRow>, sqlx::Error> {
        sqlx::query_as::<_, ProviderAccountRow>(
            "SELECT * FROM provider_accounts WHERE provider_id = $1 AND is_default = true AND enabled = true LIMIT 1"
        )
        .bind(provider_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn set_default(&self, provider_id: &str, account_id: &str) -> Result<bool, sqlx::Error> {
        // First, unset all defaults for this provider
        sqlx::query("UPDATE provider_accounts SET is_default = false WHERE provider_id = $1")
            .bind(provider_id)
            .execute(&self.pool)
            .await?;
        
        // Then set the new default
        let result = sqlx::query("UPDATE provider_accounts SET is_default = true, updated_at = NOW() WHERE id = $1")
            .bind(account_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn update_quota(&self, id: &str, quota_config: &JsonValue) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("UPDATE provider_accounts SET quota_config = $1, updated_at = NOW() WHERE id = $2")
            .bind(quota_config)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn update_enabled(&self, id: &str, enabled: bool) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("UPDATE provider_accounts SET enabled = $1, updated_at = NOW() WHERE id = $2")
            .bind(enabled)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn update(
        &self,
        id: &str,
        name: &str,
        enabled: bool,
        priority: i32,
        models: &JsonValue,
        quota_config: &JsonValue,
        config: &JsonValue,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE provider_accounts SET name = $1, enabled = $2, priority = $3, models = $4, quota_config = $5, config = $6, updated_at = NOW() WHERE id = $7"
        )
        .bind(name)
        .bind(enabled)
        .bind(priority)
        .bind(models)
        .bind(quota_config)
        .bind(config)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn delete(&self, id: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM provider_accounts WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn count_by_provider(&self, provider_id: &str) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM provider_accounts WHERE provider_id = $1")
            .bind(provider_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }
}
