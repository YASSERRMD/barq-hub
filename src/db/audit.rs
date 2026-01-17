//! Audit log repository for database operations

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use serde_json::Value as JsonValue;
use super::DbPool;

#[derive(Debug, Clone, FromRow)]
pub struct AuditLogRow {
    pub id: String,
    pub user_id: Option<String>,
    pub action: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub details: JsonValue,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub struct AuditRepository {
    pool: DbPool,
}

impl AuditRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        id: &str,
        user_id: Option<&str>,
        action: &str,
        resource_type: Option<&str>,
        resource_id: Option<&str>,
        details: &JsonValue,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<AuditLogRow, sqlx::Error> {
        sqlx::query_as::<_, AuditLogRow>(
            r#"
            INSERT INTO audit_logs (id, user_id, action, resource_type, resource_id, details, ip_address, user_agent, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7::inet, $8, NOW())
            RETURNING id, user_id, action, resource_type, resource_id, details, ip_address::text, user_agent, created_at
            "#
        )
        .bind(id)
        .bind(user_id)
        .bind(action)
        .bind(resource_type)
        .bind(resource_id)
        .bind(details)
        .bind(ip_address)
        .bind(user_agent)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn list_recent(&self, limit: i64) -> Result<Vec<AuditLogRow>, sqlx::Error> {
        sqlx::query_as::<_, AuditLogRow>(
            r#"
            SELECT id, user_id, action, resource_type, resource_id, details, ip_address::text, user_agent, created_at
            FROM audit_logs 
            ORDER BY created_at DESC 
            LIMIT $1
            "#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn list_by_user(&self, user_id: &str, limit: i64) -> Result<Vec<AuditLogRow>, sqlx::Error> {
        sqlx::query_as::<_, AuditLogRow>(
            r#"
            SELECT id, user_id, action, resource_type, resource_id, details, ip_address::text, user_agent, created_at
            FROM audit_logs 
            WHERE user_id = $1 
            ORDER BY created_at DESC 
            LIMIT $2
            "#
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn list_by_action(&self, action: &str, limit: i64) -> Result<Vec<AuditLogRow>, sqlx::Error> {
        sqlx::query_as::<_, AuditLogRow>(
            r#"
            SELECT id, user_id, action, resource_type, resource_id, details, ip_address::text, user_agent, created_at
            FROM audit_logs 
            WHERE action = $1 
            ORDER BY created_at DESC 
            LIMIT $2
            "#
        )
        .bind(action)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn list_by_resource(&self, resource_type: &str, resource_id: &str, limit: i64) -> Result<Vec<AuditLogRow>, sqlx::Error> {
        sqlx::query_as::<_, AuditLogRow>(
            r#"
            SELECT id, user_id, action, resource_type, resource_id, details, ip_address::text, user_agent, created_at
            FROM audit_logs 
            WHERE resource_type = $1 AND resource_id = $2 
            ORDER BY created_at DESC 
            LIMIT $3
            "#
        )
        .bind(resource_type)
        .bind(resource_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn count(&self) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM audit_logs")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }

    pub async fn count_by_action(&self, action: &str) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM audit_logs WHERE action = $1")
            .bind(action)
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }
}
