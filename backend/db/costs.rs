//! Cost tracking repository for database operations

use chrono::{DateTime, Utc};
use sqlx::{FromRow, types::BigDecimal};
use super::DbPool;

#[derive(Debug, Clone, FromRow)]
pub struct CostEntryRow {
    pub id: String,
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub application_id: Option<String>,
    pub provider: String,
    pub model: String,
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
    pub cost: BigDecimal,
    pub request_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct BudgetRow {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub monthly_limit: BigDecimal,
    pub current_spend: BigDecimal,
    pub enforce: bool,
    pub reset_day: i32,
    pub last_reset: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct CostRepository {
    pool: DbPool,
}

impl CostRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    // Cost Entries
    pub async fn create_entry(
        &self,
        id: &str,
        user_id: Option<&str>,
        agent_id: Option<&str>,
        application_id: Option<&str>,
        provider: &str,
        model: &str,
        prompt_tokens: i32,
        completion_tokens: i32,
        cost: f64,
        request_id: Option<&str>,
    ) -> Result<CostEntryRow, sqlx::Error> {
        sqlx::query_as::<_, CostEntryRow>(
            r#"
            INSERT INTO cost_entries (id, user_id, agent_id, application_id, provider, model, prompt_tokens, completion_tokens, total_tokens, cost, request_id, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW())
            RETURNING *
            "#
        )
        .bind(id)
        .bind(user_id)
        .bind(agent_id)
        .bind(application_id)
        .bind(provider)
        .bind(model)
        .bind(prompt_tokens)
        .bind(completion_tokens)
        .bind(prompt_tokens + completion_tokens)
        .bind(BigDecimal::try_from(cost).unwrap_or_default())
        .bind(request_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn list_recent(&self, limit: i64) -> Result<Vec<CostEntryRow>, sqlx::Error> {
        sqlx::query_as::<_, CostEntryRow>(
            "SELECT * FROM cost_entries ORDER BY created_at DESC LIMIT $1"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn list_by_user(&self, user_id: &str, limit: i64) -> Result<Vec<CostEntryRow>, sqlx::Error> {
        sqlx::query_as::<_, CostEntryRow>(
            "SELECT * FROM cost_entries WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2"
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_total_cost(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<f64, sqlx::Error> {
        let row: (Option<BigDecimal>,) = sqlx::query_as(
            "SELECT SUM(cost) FROM cost_entries WHERE created_at BETWEEN $1 AND $2"
        )
        .bind(start)
        .bind(end)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(row.0.map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0))
    }

    pub async fn get_total_tokens(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<i64, sqlx::Error> {
        let row: (Option<i64>,) = sqlx::query_as(
            "SELECT SUM(total_tokens) FROM cost_entries WHERE created_at BETWEEN $1 AND $2"
        )
        .bind(start)
        .bind(end)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(row.0.unwrap_or(0))
    }

    // Budgets
    pub async fn get_budget(&self, entity_type: &str, entity_id: &str) -> Result<Option<BudgetRow>, sqlx::Error> {
        sqlx::query_as::<_, BudgetRow>(
            "SELECT * FROM budgets WHERE entity_type = $1 AND entity_id = $2"
        )
        .bind(entity_type)
        .bind(entity_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn set_budget(
        &self,
        id: &str,
        entity_type: &str,
        entity_id: &str,
        monthly_limit: f64,
        enforce: bool,
    ) -> Result<BudgetRow, sqlx::Error> {
        sqlx::query_as::<_, BudgetRow>(
            r#"
            INSERT INTO budgets (id, entity_type, entity_id, monthly_limit, enforce, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
            ON CONFLICT (entity_type, entity_id) 
            DO UPDATE SET monthly_limit = $4, enforce = $5, updated_at = NOW()
            RETURNING *
            "#
        )
        .bind(id)
        .bind(entity_type)
        .bind(entity_id)
        .bind(BigDecimal::try_from(monthly_limit).unwrap_or_default())
        .bind(enforce)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update_budget_spend(&self, entity_type: &str, entity_id: &str, amount: f64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE budgets SET current_spend = current_spend + $1, updated_at = NOW() WHERE entity_type = $2 AND entity_id = $3"
        )
        .bind(BigDecimal::try_from(amount).unwrap_or_default())
        .bind(entity_type)
        .bind(entity_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn reset_budget(&self, entity_type: &str, entity_id: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE budgets SET current_spend = 0, last_reset = NOW(), updated_at = NOW() WHERE entity_type = $1 AND entity_id = $2"
        )
        .bind(entity_type)
        .bind(entity_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }
}
