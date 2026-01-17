//! User repository for database operations

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use super::DbPool;

#[derive(Debug, Clone, FromRow)]
pub struct UserRow {
    pub id: String,
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub role: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

pub struct UserRepository {
    pool: DbPool,
}

impl UserRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, id: &str, email: &str, name: &str, password_hash: &str, role: &str) -> Result<UserRow, sqlx::Error> {
        sqlx::query_as::<_, UserRow>(
            r#"
            INSERT INTO users (id, email, name, password_hash, role, enabled, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, true, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(id)
        .bind(email)
        .bind(name)
        .bind(password_hash)
        .bind(role)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<UserRow>, sqlx::Error> {
        sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<UserRow>, sqlx::Error> {
        sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn list_all(&self) -> Result<Vec<UserRow>, sqlx::Error> {
        sqlx::query_as::<_, UserRow>("SELECT * FROM users ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await
    }

    pub async fn update(&self, id: &str, name: Option<&str>, email: Option<&str>, role: Option<&str>, enabled: Option<bool>) -> Result<Option<UserRow>, sqlx::Error> {
        let mut query = String::from("UPDATE users SET updated_at = NOW()");
        let mut param_idx = 1;
        
        if name.is_some() { query.push_str(&format!(", name = ${}", { param_idx += 1; param_idx })); }
        if email.is_some() { query.push_str(&format!(", email = ${}", { param_idx += 1; param_idx })); }
        if role.is_some() { query.push_str(&format!(", role = ${}", { param_idx += 1; param_idx })); }
        if enabled.is_some() { query.push_str(&format!(", enabled = ${}", { param_idx += 1; param_idx })); }
        
        query.push_str(" WHERE id = $1 RETURNING *");
        
        let mut q = sqlx::query_as::<_, UserRow>(&query).bind(id);
        if let Some(v) = name { q = q.bind(v); }
        if let Some(v) = email { q = q.bind(v); }
        if let Some(v) = role { q = q.bind(v); }
        if let Some(v) = enabled { q = q.bind(v); }
        
        q.fetch_optional(&self.pool).await
    }

    pub async fn update_password(&self, id: &str, password_hash: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
            .bind(password_hash)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn update_last_login(&self, id: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("UPDATE users SET last_login = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn delete(&self, id: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn count(&self) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }

    pub async fn count_by_role(&self, role: &str) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE role = $1")
            .bind(role)
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }
}
