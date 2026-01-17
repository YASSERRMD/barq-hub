//! Cost tracking and budget management

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Datelike};
use uuid::Uuid;

use crate::{
    CostEntry, CostSummary, Budget, TokenUsage, Provider,
    error::{Result, CostError, SynapseError},
};

/// Manages cost tracking and budget enforcement
pub struct CostManager {
    /// Cost ledger
    entries: Arc<RwLock<Vec<CostEntry>>>,
    /// User/org budgets
    budgets: Arc<RwLock<HashMap<String, Budget>>>,
}

impl CostManager {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(Vec::new())),
            budgets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record a cost entry
    pub async fn record_cost(
        &self,
        provider: &str,
        model: &str,
        usage: &TokenUsage,
        cost: f64,
        user_id: &str,
        request_id: &str,
    ) -> Result<CostEntry> {
        let entry = CostEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            provider: provider.to_string(),
            model: model.to_string(),
            input_tokens: usage.prompt_tokens,
            output_tokens: usage.completion_tokens,
            cost,
            user_id: user_id.to_string(),
            request_id: request_id.to_string(),
        };

        // Record the entry
        self.entries.write().await.push(entry.clone());

        // Update budget if exists
        let mut budgets = self.budgets.write().await;
        if let Some(budget) = budgets.get_mut(user_id) {
            budget.spent_this_month += cost;
        }

        Ok(entry)
    }

    /// Calculate cost based on usage and provider pricing
    pub fn calculate_cost(usage: &TokenUsage, provider: &Provider) -> f64 {
        let input_cost = (usage.prompt_tokens as f64 / 1_000_000.0) 
            * provider.pricing.input_token_cost;
        let output_cost = (usage.completion_tokens as f64 / 1_000_000.0) 
            * provider.pricing.output_token_cost;
        input_cost + output_cost
    }

    /// Check if a user can make a request (budget check)
    pub async fn can_request(&self, user_id: &str, estimated_cost: f64) -> Result<bool> {
        let budgets = self.budgets.read().await;
        
        if let Some(budget) = budgets.get(user_id) {
            if budget.enforce_limit {
                if budget.spent_this_month + estimated_cost > budget.monthly_limit {
                    return Err(SynapseError::Cost(CostError::BudgetExceeded(user_id.to_string())));
                }
            }
        }
        
        Ok(true)
    }

    /// Set a budget for a user/org
    pub async fn set_budget(&self, entity_id: &str, monthly_limit: f64, enforce: bool) {
        let mut budgets = self.budgets.write().await;
        
        budgets.insert(entity_id.to_string(), Budget {
            entity_id: entity_id.to_string(),
            monthly_limit,
            spent_this_month: 0.0,
            enforce_limit: enforce,
            alert_thresholds: vec![0.5, 0.8, 0.9, 1.0],
            reset_day: 1,
        });
    }

    /// Get budget for an entity
    pub async fn get_budget(&self, entity_id: &str) -> Option<Budget> {
        self.budgets.read().await.get(entity_id).cloned()
    }

    /// Reset monthly budgets (call on reset day)
    pub async fn reset_monthly_budgets(&self) {
        let today = Utc::now().day() as u8;
        let mut budgets = self.budgets.write().await;
        
        for budget in budgets.values_mut() {
            if budget.reset_day == today {
                budget.spent_this_month = 0.0;
            }
        }
    }

    /// Get cost summary for a time period
    pub async fn get_summary(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> CostSummary {
        let entries = self.entries.read().await;
        
        let filtered: Vec<_> = entries
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .collect();

        let total_cost: f64 = filtered.iter().map(|e| e.cost).sum();
        let total_tokens: u64 = filtered
            .iter()
            .map(|e| (e.input_tokens + e.output_tokens) as u64)
            .sum();

        let mut by_provider: HashMap<String, f64> = HashMap::new();
        let mut by_model: HashMap<String, f64> = HashMap::new();
        let mut by_user: HashMap<String, f64> = HashMap::new();

        for entry in &filtered {
            *by_provider.entry(entry.provider.clone()).or_insert(0.0) += entry.cost;
            *by_model.entry(entry.model.clone()).or_insert(0.0) += entry.cost;
            *by_user.entry(entry.user_id.clone()).or_insert(0.0) += entry.cost;
        }

        CostSummary {
            total_cost,
            request_count: filtered.len(),
            total_tokens,
            by_provider,
            by_model,
            by_user,
            period_start: start,
            period_end: end,
        }
    }

    /// Get recent cost entries
    pub async fn get_recent_entries(&self, limit: usize) -> Vec<CostEntry> {
        let entries = self.entries.read().await;
        entries.iter().rev().take(limit).cloned().collect()
    }

    /// Get entries for a specific user
    pub async fn get_user_entries(&self, user_id: &str, limit: usize) -> Vec<CostEntry> {
        let entries = self.entries.read().await;
        entries
            .iter()
            .filter(|e| e.user_id == user_id)
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Check budget thresholds and return alerts
    pub async fn check_alerts(&self, entity_id: &str) -> Vec<String> {
        let budgets = self.budgets.read().await;
        let mut alerts = Vec::new();

        if let Some(budget) = budgets.get(entity_id) {
            let usage_pct = budget.spent_this_month / budget.monthly_limit;
            
            for threshold in &budget.alert_thresholds {
                if usage_pct >= *threshold {
                    alerts.push(format!(
                        "Budget alert: {} has used {:.1}% of ${:.2} monthly limit",
                        entity_id,
                        usage_pct * 100.0,
                        budget.monthly_limit
                    ));
                }
            }
        }

        alerts
    }
}

impl Default for CostManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ProviderPricing, ProviderType, ProviderHealth};

    fn test_provider() -> Provider {
        Provider {
            id: "test".to_string(),
            name: "Test".to_string(),
            provider_type: ProviderType::OpenAI,
            api_key: "test".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            pricing: ProviderPricing {
                input_token_cost: 30.0,   // $30/1M
                output_token_cost: 60.0,  // $60/1M
            },
            enabled: true,
            health: ProviderHealth::default(),
            headers: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_cost_calculation() {
        let provider = test_provider();
        let usage = TokenUsage {
            prompt_tokens: 1000,
            completion_tokens: 500,
            total_tokens: 1500,
        };

        let cost = CostManager::calculate_cost(&usage, &provider);
        // 1000/1M * 30 + 500/1M * 60 = 0.03 + 0.03 = 0.06
        assert!((cost - 0.06).abs() < 0.0001);
    }

    #[tokio::test]
    async fn test_record_cost() {
        let manager = CostManager::new();
        let usage = TokenUsage {
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
        };

        let entry = manager.record_cost(
            "OpenAI", "gpt-4", &usage, 0.01, "user1", "req1"
        ).await.unwrap();

        assert_eq!(entry.provider, "OpenAI");
        assert_eq!(entry.user_id, "user1");
    }

    #[tokio::test]
    async fn test_budget_enforcement() {
        let manager = CostManager::new();
        
        // Set budget
        manager.set_budget("user1", 10.0, true).await;
        
        // Should allow small request
        assert!(manager.can_request("user1", 5.0).await.is_ok());
        
        // Record cost
        let usage = TokenUsage::default();
        manager.record_cost("OpenAI", "gpt-4", &usage, 8.0, "user1", "req1").await.unwrap();
        
        // Should reject request that exceeds budget
        assert!(manager.can_request("user1", 3.0).await.is_err());
    }

    #[tokio::test]
    async fn test_budget_without_enforcement() {
        let manager = CostManager::new();
        
        // Set budget without enforcement
        manager.set_budget("user1", 10.0, false).await;
        
        // Should always allow
        assert!(manager.can_request("user1", 100.0).await.is_ok());
    }
}
