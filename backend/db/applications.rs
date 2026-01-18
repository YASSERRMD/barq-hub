//! Application repository for database operations (external service API keys)

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use serde_json::Value as JsonValue;
use super::DbPool;

#[derive(Debug, Clone, FromRow)]
pub struct ApplicationRow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub api_key_hash: String,
    pub api_key_prefix: String,
    pub scopes: JsonValue,
    pub rate_limit: i32,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub requests_today: i32,
    pub requests_reset_at: DateTime<Utc>,
}

enum CreateApplicationArg {
    String(String),
    Int(i32),
    Json(JsonValue),
    DateTime(DateTime<Utc>),
}

pub struct ApplicationRepository {
    pool: DbPool,
}

impl ApplicationRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        id: &str,
        name: &str,
        description: Option<&str>,
        api_key_hash: &str,
        api_key_prefix: &str,
        scopes: &JsonValue,
        rate_limit: i32,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<ApplicationRow, sqlx::Error> {
        sqlx::query_as::<_, ApplicationRow>(
            r#"
            INSERT INTO applications (id, name, description, api_key_hash, api_key_prefix, scopes, rate_limit, status, expires_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, 'active', $8, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .bind(api_key_hash)
        .bind(api_key_prefix)
        .bind(scopes)
        .bind(rate_limit)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<ApplicationRow>, sqlx::Error> {
        sqlx::query_as::<_, ApplicationRow>("SELECT * FROM applications WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn find_by_key_prefix(&self, prefix: &str) -> Result<Option<ApplicationRow>, sqlx::Error> {
        sqlx::query_as::<_, ApplicationRow>("SELECT * FROM applications WHERE api_key_prefix = $1")
            .bind(prefix)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn list_all(&self) -> Result<Vec<ApplicationRow>, sqlx::Error> {
        sqlx::query_as::<_, ApplicationRow>("SELECT * FROM applications ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await
    }

    pub async fn list_active(&self) -> Result<Vec<ApplicationRow>, sqlx::Error> {
        sqlx::query_as::<_, ApplicationRow>(
            "SELECT * FROM applications WHERE status = 'active' ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn update_status(&self, id: &str, status: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("UPDATE applications SET status = $1, updated_at = NOW() WHERE id = $2")
            .bind(status)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn update_last_used(&self, id: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("UPDATE applications SET last_used = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn increment_requests(&self, id: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE applications 
            SET requests_today = CASE 
                WHEN DATE(requests_reset_at) < CURRENT_DATE THEN 1 
                ELSE requests_today + 1 
            END,
            requests_reset_at = CASE 
                WHEN DATE(requests_reset_at) < CURRENT_DATE THEN NOW() 
                ELSE requests_reset_at 
            END,
            last_used = NOW()
            WHERE id = $1
            "#
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn rotate_key(&self, id: &str, new_key_hash: &str, new_prefix: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE applications SET api_key_hash = $1, api_key_prefix = $2, updated_at = NOW() WHERE id = $3"
        )
        .bind(new_key_hash)
        .bind(new_prefix)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn update(
        &self,
        id: &str,
        name: Option<&str>,
        description: Option<&str>,
        scopes: Option<&JsonValue>,
        rate_limit: Option<i32>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<Option<ApplicationRow>, sqlx::Error> {
        // Build dynamic update query
        let mut query = "UPDATE applications SET updated_at = NOW()".to_string();
        let mut arg_index = 1;
        let mut args = Vec::new();

        if let Some(n) = name {
            query.push_str(&format!(", name = ${}", arg_index));
            args.push(CreateApplicationArg::String(n.to_string()));
            arg_index += 1;
        }
        if let Some(d) = description {
            query.push_str(&format!(", description = ${}", arg_index));
            args.push(CreateApplicationArg::String(d.to_string()));
            arg_index += 1;
        }
        if let Some(s) = scopes {
            query.push_str(&format!(", scopes = ${}", arg_index));
            args.push(CreateApplicationArg::Json(s.clone()));
            arg_index += 1;
        }
        if let Some(r) = rate_limit {
            query.push_str(&format!(", rate_limit = ${}", arg_index));
            args.push(CreateApplicationArg::Int(r));
            arg_index += 1;
        }
        // Handle expires_at explicitly (nullable)
        // Since the arg is Option, passing Some(None) would be tricky with this simplified builder 
        // For now let's assume if it's passed it's a value. 
        // If we want to clear it, we might need a dedicated enum or logic.
        // But for this simple implementation, let's just update if present.
        if let Some(e) = expires_at {
             query.push_str(&format!(", expires_at = ${}", arg_index));
             args.push(CreateApplicationArg::DateTime(e));
             arg_index += 1;
        }

        query.push_str(&format!(" WHERE id = ${} RETURNING *", arg_index));
        // Id is the last arg
        
        let mut query_builder = sqlx::query_as::<_, ApplicationRow>(&query);
        
        for arg in args {
            match arg {
                CreateApplicationArg::String(s) => query_builder = query_builder.bind(s),
                CreateApplicationArg::Int(i) => query_builder = query_builder.bind(i),
                CreateApplicationArg::Json(j) => query_builder = query_builder.bind(j),
                CreateApplicationArg::DateTime(d) => query_builder = query_builder.bind(d),
            }
        }
        query_builder = query_builder.bind(id);

        query_builder.fetch_optional(&self.pool).await
    }

    pub async fn delete(&self, id: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM applications WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn count(&self) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM applications")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }

    pub async fn count_by_status(&self, status: &str) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM applications WHERE status = $1")
            .bind(status)
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }
}
