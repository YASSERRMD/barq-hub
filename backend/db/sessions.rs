//! Session repository for database operations

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use super::DbPool;

#[derive(Debug, Clone, FromRow)]
pub struct SessionRow {
    pub id: String,
    pub token: String,
    pub user_id: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

pub struct SessionRepository {
    pool: DbPool,
}

impl SessionRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, id: &str, token: &str, user_id: &str, expires_at: DateTime<Utc>) -> Result<SessionRow, sqlx::Error> {
        sqlx::query_as::<_, SessionRow>(
            r#"
            INSERT INTO sessions (id, token, user_id, expires_at, created_at)
            VALUES ($1, $2, $3, $4, NOW())
            RETURNING *
            "#
        )
        .bind(id)
        .bind(token)
        .bind(user_id)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_by_token(&self, token: &str) -> Result<Option<SessionRow>, sqlx::Error> {
        sqlx::query_as::<_, SessionRow>(
            "SELECT * FROM sessions WHERE token = $1 AND expires_at > NOW()"
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn delete_by_token(&self, token: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM sessions WHERE token = $1")
            .bind(token)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_expired(&self) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM sessions WHERE expires_at < NOW()")
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn delete_by_user(&self, user_id: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM sessions WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }
}
