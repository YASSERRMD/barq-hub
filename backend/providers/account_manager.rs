//! Provider Account Management with Multi-Tier Quota-Based Rotation

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::types::ProviderModel;

/// Quota period types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum QuotaPeriod {
    Minute,
    Hour,
    Day,
    Month,
}

impl QuotaPeriod {
    pub fn duration(&self) -> Duration {
        match self {
            QuotaPeriod::Minute => Duration::minutes(1),
            QuotaPeriod::Hour => Duration::hours(1),
            QuotaPeriod::Day => Duration::days(1),
            QuotaPeriod::Month => Duration::days(30),
        }
    }
    
    pub fn display_name(&self) -> &'static str {
        match self {
            QuotaPeriod::Minute => "per minute",
            QuotaPeriod::Hour => "per hour",
            QuotaPeriod::Day => "per day",
            QuotaPeriod::Month => "per month",
        }
    }
    
    pub fn all() -> Vec<QuotaPeriod> {
        vec![QuotaPeriod::Minute, QuotaPeriod::Hour, QuotaPeriod::Day, QuotaPeriod::Month]
    }
}

/// Provider category
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProviderCategory {
    LlmEmbedding,
    VectorDb,
}

/// Provider type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    Llm,
    Embedding,
    Both,
    VectorDb,
}

/// Configuration for Azure OpenAI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureConfig {
    pub endpoint: String,
    pub deployment_name: String,
    pub api_version: String,
    pub api_key: String,
}

/// Configuration for AWS Bedrock
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsConfig {
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
}

/// Configuration for standard API key providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyConfig {
    pub api_key: String,
    pub organization_id: Option<String>,
    pub custom_endpoint: Option<String>,
}

/// Configuration for vector databases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorDbConfig {
    pub url: String,
    pub api_key: Option<String>,
    pub collection_name: Option<String>,
}

/// Provider account configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AccountConfig {
    ApiKey(ApiKeyConfig),
    Azure(AzureConfig),
    Aws(AwsConfig),
    VectorDb(VectorDbConfig),
}

/// Single quota tier with its own usage tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaTier {
    pub period: QuotaPeriod,
    pub token_limit: u64,
    pub request_limit: Option<u64>,
    pub tokens_used: u64,
    pub requests_used: u64,
    pub period_start: DateTime<Utc>,
}

impl QuotaTier {
    pub fn new(period: QuotaPeriod, token_limit: u64, request_limit: Option<u64>) -> Self {
        Self {
            period,
            token_limit,
            request_limit,
            tokens_used: 0,
            requests_used: 0,
            period_start: Utc::now(),
        }
    }
    
    /// Check if this tier's period has expired
    pub fn is_period_expired(&self) -> bool {
        Utc::now() > self.period_start + self.period.duration()
    }
    
    /// Reset if period expired
    pub fn reset_if_expired(&mut self) -> bool {
        if self.is_period_expired() {
            self.tokens_used = 0;
            self.requests_used = 0;
            self.period_start = Utc::now();
            true
        } else {
            false
        }
    }
    
    /// Check if this tier has quota remaining (non-mutating check)
    pub fn has_quota_available(&self) -> bool {
        // Check if period expired - if so, quota is available
        if self.is_period_expired() {
            return true;
        }
        
        if self.tokens_used >= self.token_limit {
            return false;
        }
        
        if let Some(req_limit) = self.request_limit {
            if self.requests_used >= req_limit {
                return false;
            }
        }
        
        true
    }

    /// Check if this tier has quota remaining (mutating - resets if expired)
    pub fn has_quota(&mut self) -> bool {
        self.reset_if_expired();
        self.has_quota_available()
    }
    
    /// Record usage
    pub fn record_usage(&mut self, tokens: u64, requests: u64) {
        self.reset_if_expired();
        self.tokens_used += tokens;
        self.requests_used += requests;
    }
    
    /// Get remaining tokens
    pub fn remaining_tokens(&self) -> u64 {
        if self.tokens_used >= self.token_limit {
            0
        } else {
            self.token_limit - self.tokens_used
        }
    }
    
    /// Time until this tier resets
    pub fn time_until_reset(&self) -> Duration {
        let reset_time = self.period_start + self.period.duration();
        let now = Utc::now();
        
        if now >= reset_time {
            Duration::zero()
        } else {
            reset_time - now
        }
    }
    
    /// Get usage percentage
    pub fn usage_percentage(&self) -> f64 {
        if self.token_limit == 0 {
            0.0
        } else {
            (self.tokens_used as f64 / self.token_limit as f64) * 100.0
        }
    }
}

/// A single provider account with MULTIPLE quota tiers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderAccount {
    pub id: String,
    pub name: String,
    pub provider_id: String,
    pub config: AccountConfig,
    pub enabled: bool,
    pub is_default: bool,
    pub priority: i32,
    
    // Multiple quota tiers - keyed by period
    pub quotas: HashMap<QuotaPeriod, QuotaTier>,
    
    // Custom models for this account
    pub models: Vec<ProviderModel>,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ProviderAccount {
    pub fn new(name: String, provider_id: String, config: AccountConfig) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            provider_id,
            config,
            enabled: true,
            is_default: false,
            priority: 0,
            quotas: HashMap::new(),
            models: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    
    /// Add or update a quota tier
    pub fn set_quota(&mut self, period: QuotaPeriod, token_limit: u64, request_limit: Option<u64>) {
        self.quotas.insert(period, QuotaTier::new(period, token_limit, request_limit));
        self.updated_at = Utc::now();
    }
    
    /// Remove a quota tier
    pub fn remove_quota(&mut self, period: QuotaPeriod) {
        self.quotas.remove(&period);
        self.updated_at = Utc::now();
    }
    
    /// Check if ALL quota tiers have remaining capacity (non-mutating)
    pub fn has_quota_available(&self) -> bool {
        if self.quotas.is_empty() {
            return true; // No quotas = unlimited
        }
        
        for tier in self.quotas.values() {
            if !tier.has_quota_available() {
                return false;
            }
        }
        
        true
    }

    /// Check if ALL quota tiers have remaining capacity (mutating)
    pub fn has_quota(&mut self) -> bool {
        if self.quotas.is_empty() {
            return true;
        }
        
        for tier in self.quotas.values_mut() {
            if !tier.has_quota() {
                return false;
            }
        }
        
        true
    }
    
    /// Record usage across all tiers
    pub fn record_usage(&mut self, tokens: u64, requests: u64) {
        for tier in self.quotas.values_mut() {
            tier.record_usage(tokens, requests);
        }
        self.updated_at = Utc::now();
    }
    
    /// Get the most restrictive (smallest) remaining quota
    pub fn min_remaining_tokens(&self) -> Option<u64> {
        self.quotas.values()
            .map(|t| t.remaining_tokens())
            .min()
    }
    
    /// Get the tier that will reset soonest
    pub fn next_reset(&self) -> Option<(QuotaPeriod, Duration)> {
        self.quotas.iter()
            .filter(|(_, t)| !t.has_quota_available() || t.usage_percentage() > 80.0)
            .map(|(p, t)| (*p, t.time_until_reset()))
            .min_by_key(|(_, d)| d.num_seconds())
    }
    
    /// Get status of all quota tiers
    pub fn quota_statuses(&self) -> Vec<QuotaTierStatus> {
        self.quotas.values()
            .map(|t| QuotaTierStatus {
                period: t.period,
                token_limit: t.token_limit,
                tokens_used: t.tokens_used,
                request_limit: t.request_limit,
                requests_used: t.requests_used,
                remaining_tokens: t.remaining_tokens(),
                usage_percentage: t.usage_percentage(),
                seconds_until_reset: t.time_until_reset().num_seconds(),
                has_quota: t.tokens_used < t.token_limit,
            })
            .collect()
    }
    
    /// Check which specific tier is blocking (if any)
    pub fn blocking_tier(&mut self) -> Option<QuotaPeriod> {
        for (period, tier) in self.quotas.iter_mut() {
            if !tier.has_quota() {
                return Some(*period);
            }
        }
        None
    }
}

/// Status of a single quota tier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaTierStatus {
    pub period: QuotaPeriod,
    pub token_limit: u64,
    pub tokens_used: u64,
    pub request_limit: Option<u64>,
    pub requests_used: u64,
    pub remaining_tokens: u64,
    pub usage_percentage: f64,
    pub seconds_until_reset: i64,
    pub has_quota: bool,
}

/// Provider definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderDefinition {
    pub id: String,
    pub name: String,
    pub category: ProviderCategory,
    pub provider_type: ProviderType,
    pub requires_azure_config: bool,
    pub requires_aws_config: bool,
    pub default_models: Vec<ProviderModel>,
    pub supported_quota_periods: Vec<QuotaPeriod>,
}

/// Provider Account Manager
pub struct ProviderAccountManager {
    accounts: Arc<RwLock<HashMap<String, ProviderAccount>>>,
    providers: Arc<RwLock<HashMap<String, ProviderDefinition>>>,
    // Track which account was original default before fallback
    original_accounts: Arc<RwLock<HashMap<String, String>>>,
}

impl ProviderAccountManager {
    pub fn new() -> Self {
        let mut providers = HashMap::new();

        let make_model = |name: &str| {
            let id = name.to_lowercase().replace(" ", "-");
            let mut caps = vec![crate::types::ModelCapability::LLM];
            if name.contains("embed") || name.contains("embedding") {
                caps = vec![crate::types::ModelCapability::Embedding];
            }
            ProviderModel {
                id,
                name: name.to_string(),
                capabilities: caps,
                input_token_cost: None,
                output_token_cost: None,
            }
        };
        
        // LLM/Embedding providers - Updated to 2025/2026 models
        let llm_providers = vec![
            ("openai", "OpenAI", ProviderType::Both, false, false,
             vec!["gpt-4o", "gpt-4o-mini", "gpt-4-turbo", "o1", "o1-mini", "text-embedding-3-small", "text-embedding-3-large"]),
            ("anthropic", "Anthropic", ProviderType::Llm, false, false,
             vec!["claude-3-5-sonnet-20241022", "claude-3-5-haiku-20241022", "claude-3-opus-20240229"]),
            ("gemini", "Google Gemini", ProviderType::Both, false, false,
             vec!["gemini-2.0-flash", "gemini-1.5-pro", "gemini-1.5-flash", "text-embedding-004"]),
            ("mistral", "Mistral AI", ProviderType::Both, false, false,
             vec!["mistral-large-latest", "mistral-small-latest", "codestral-latest", "mistral-embed"]),
            ("groq", "Groq", ProviderType::Llm, false, false,
             vec!["llama-3.3-70b-versatile", "llama-3.1-8b-instant", "mixtral-8x7b-32768"]),
            ("together", "Together AI", ProviderType::Both, false, false,
             vec!["meta-llama/Llama-3.3-70B-Instruct-Turbo", "meta-llama/Llama-3.2-90B-Vision-Instruct-Turbo"]),
            ("cohere", "Cohere", ProviderType::Both, false, false,
             vec!["command-a-03-2025", "command-r-plus-08-2024", "command-r-08-2024", "embed-v4.0"]),
            ("voyage", "Voyage AI", ProviderType::Embedding, false, false,
             vec!["voyage-3", "voyage-3-lite", "voyage-code-3"]),
            ("jina", "Jina AI", ProviderType::Embedding, false, false,
             vec!["jina-embeddings-v3", "jina-clip-v2"]),
        ];
        
        for (id, name, ptype, azure, aws, models) in llm_providers {
            providers.insert(id.to_string(), ProviderDefinition {
                id: id.to_string(),
                name: name.to_string(),
                category: ProviderCategory::LlmEmbedding,
                provider_type: ptype,
                requires_azure_config: azure,
                requires_aws_config: aws,
                default_models: models.iter().map(|s| make_model(s)).collect(),
                supported_quota_periods: QuotaPeriod::all(),
            });
        }
        
        // Azure OpenAI
        providers.insert("azure".to_string(), ProviderDefinition {
            id: "azure".to_string(),
            name: "Azure OpenAI".to_string(),
            category: ProviderCategory::LlmEmbedding,
            provider_type: ProviderType::Both,
            requires_azure_config: true,
            requires_aws_config: false,
            default_models: vec!["gpt-4", "gpt-35-turbo"].into_iter().map(make_model).collect(),
            supported_quota_periods: QuotaPeriod::all(),
        });
        
        // AWS Bedrock
        providers.insert("bedrock".to_string(), ProviderDefinition {
            id: "bedrock".to_string(),
            name: "AWS Bedrock".to_string(),
            category: ProviderCategory::LlmEmbedding,
            provider_type: ProviderType::Both,
            requires_azure_config: false,
            requires_aws_config: true,
            default_models: vec!["anthropic.claude-3-sonnet", "amazon.titan-embed-text-v1"].into_iter().map(make_model).collect(),
            supported_quota_periods: QuotaPeriod::all(),
        });
        
        // Vector databases
        let vector_dbs = vec![
            ("qdrant", "Qdrant"),
            ("pinecone", "Pinecone"),
            ("weaviate", "Weaviate"),
            ("chroma", "Chroma"),
            ("milvus", "Milvus"),
        ];
        
        for (id, name) in vector_dbs {
            providers.insert(id.to_string(), ProviderDefinition {
                id: id.to_string(),
                name: name.to_string(),
                category: ProviderCategory::VectorDb,
                provider_type: ProviderType::VectorDb,
                requires_azure_config: false,
                requires_aws_config: false,
                default_models: vec![ProviderModel { 
                    id: "default".into(), 
                    name: "Default".into(), 
                    capabilities: vec![], 
                    input_token_cost: None, 
                    output_token_cost: None 
                }],
                supported_quota_periods: vec![QuotaPeriod::Month],
            });
        }
        
        Self {
            accounts: Arc::new(RwLock::new(HashMap::new())),
            providers: Arc::new(RwLock::new(providers)),
            original_accounts: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn list_providers(&self) -> Vec<ProviderDefinition> {
        self.providers.read().await.values().cloned().collect()
    }
    
    pub async fn list_providers_by_category(&self, category: ProviderCategory) -> Vec<ProviderDefinition> {
        self.providers.read().await
            .values()
            .filter(|p| p.category == category)
            .cloned()
            .collect()
    }
    
    pub async fn add_account(&self, account: ProviderAccount) -> crate::error::Result<ProviderAccount> {
        let mut accounts = self.accounts.write().await;
        
        let is_first = !accounts.values().any(|a| a.provider_id == account.provider_id);
        let mut account = account;
        if is_first {
            account.is_default = true;
        }
        
        accounts.insert(account.id.clone(), account.clone());
        Ok(account)
    }
    
    pub async fn get_accounts(&self, provider_id: &str) -> Vec<ProviderAccount> {
        self.accounts.read().await
            .values()
            .filter(|a| a.provider_id == provider_id && a.enabled)
            .cloned()
            .collect()
    }
    
    /// Get the best available account - with automatic fallback and return-to-original logic
    pub async fn get_available_account(&self, provider_id: &str) -> Option<ProviderAccount> {
        let mut accounts = self.accounts.write().await;
        let mut original = self.original_accounts.write().await;
        
        // Collect account info for sorting
        let mut candidates: Vec<(String, bool, i32, u64, bool)> = accounts.values()
            .filter(|a| a.provider_id == provider_id && a.enabled)
            .map(|a| (
                a.id.clone(),
                a.is_default,
                a.priority,
                a.min_remaining_tokens().unwrap_or(u64::MAX),
                a.has_quota_available(),
            ))
            .collect();
        
        if candidates.is_empty() {
            return None;
        }
        
        // Sort by: default first, then priority, then remaining quota
        candidates.sort_by(|a, b| {
            match (a.1, b.1) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.2.cmp(&b.2).then_with(|| b.3.cmp(&a.3))
            }
        });
        
        // Check if original default account has quota again (return-to-original)
        if let Some(original_id) = original.get(provider_id).cloned() {
            if let Some(acc) = accounts.get_mut(&original_id) {
                if acc.has_quota() {
                    original.remove(provider_id);
                    return Some(acc.clone());
                }
            }
        }
        
        // Find first account with quota
        let mut result_id: Option<String> = None;
        let mut need_to_store_default = false;
        
        for (id, is_default, _, _, has_quota) in &candidates {
            if *has_quota {
                result_id = Some(id.clone());
                need_to_store_default = !is_default;
                break;
            }
        }
        
        if need_to_store_default && !original.contains_key(provider_id) {
            if let Some(def_id) = candidates.iter().find(|(_, is_def, _, _, _)| *is_def).map(|(id, _, _, _, _)| id.clone()) {
                original.insert(provider_id.to_string(), def_id);
            }
        }
        
        result_id.and_then(|id| accounts.get(&id).cloned())
    }
    
    pub async fn record_usage(&self, account_id: &str, tokens: u64, requests: u64) {
        let mut accounts = self.accounts.write().await;
        if let Some(account) = accounts.get_mut(account_id) {
            account.record_usage(tokens, requests);
        }
    }
    
    pub async fn update_account(&self, account_id: &str, update: AccountUpdate) -> crate::error::Result<ProviderAccount> {
        let mut accounts = self.accounts.write().await;
        
        let account = accounts.get_mut(account_id)
            .ok_or_else(|| crate::error::SynapseError::NotFound("Account not found".into()))?;
        
        if let Some(name) = update.name {
            account.name = name;
        }
        if let Some(enabled) = update.enabled {
            account.enabled = enabled;
        }
        if let Some(priority) = update.priority {
            account.priority = priority;
        }
        if let Some(models) = update.models {
            account.models = models;
        }
        
        // Update quota tiers
        if let Some(quotas) = update.quotas {
            for quota in quotas {
                account.set_quota(quota.period, quota.token_limit, quota.request_limit);
            }
        }
        
        // Remove quota tiers
        if let Some(remove_quotas) = update.remove_quotas {
            for period in remove_quotas {
                account.remove_quota(period);
            }
        }
        
        account.updated_at = Utc::now();
        
        Ok(account.clone())
    }
    
    pub async fn delete_account(&self, account_id: &str) -> bool {
        self.accounts.write().await.remove(account_id).is_some()
    }
    
    pub async fn set_default(&self, provider_id: &str, account_id: &str) {
        let mut accounts = self.accounts.write().await;
        
        for account in accounts.values_mut() {
            if account.provider_id == provider_id {
                account.is_default = account.id == account_id;
            }
        }
    }
    
    pub async fn get_usage_summary(&self, provider_id: &str) -> ProviderUsageSummary {
        let accounts = self.accounts.read().await;
        
        let provider_accounts: Vec<_> = accounts.values()
            .filter(|a| a.provider_id == provider_id)
            .collect();
        
        let mut active_accounts = 0;
        let mut exhausted_accounts = 0;
        
        for account in &provider_accounts {
            if account.enabled && account.has_quota_available() {
                active_accounts += 1;
            } else if account.enabled {
                exhausted_accounts += 1;
            }
        }
        
        ProviderUsageSummary {
            provider_id: provider_id.to_string(),
            total_accounts: provider_accounts.len(),
            active_accounts,
            exhausted_accounts,
        }
    }
    
    /// Get detailed status for all accounts of a provider
    pub async fn get_detailed_statuses(&self, provider_id: &str) -> Vec<AccountDetailedStatus> {
        let accounts = self.accounts.read().await;
        
        accounts.values()
            .filter(|a| a.provider_id == provider_id)
            .map(|a| {
                let mut acc = a.clone();
                let blocking_tier = acc.blocking_tier();
                
                AccountDetailedStatus {
                    id: a.id.clone(),
                    name: a.name.clone(),
                    enabled: a.enabled,
                    is_default: a.is_default,
                    priority: a.priority,
                    has_quota: acc.has_quota(),
                    blocking_tier,
                    next_reset: acc.next_reset().map(|(p, d)| NextReset {
                        period: p,
                        seconds: d.num_seconds(),
                    }),
                    quota_tiers: a.quota_statuses(),
                }
            })
            .collect()
    }
}

impl Default for ProviderAccountManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Next reset info
#[derive(Debug, Clone, Serialize)]
pub struct NextReset {
    pub period: QuotaPeriod,
    pub seconds: i64,
}

/// Detailed account status
#[derive(Debug, Clone, Serialize)]
pub struct AccountDetailedStatus {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub is_default: bool,
    pub priority: i32,
    pub has_quota: bool,
    pub blocking_tier: Option<QuotaPeriod>,
    pub next_reset: Option<NextReset>,
    pub quota_tiers: Vec<QuotaTierStatus>,
}

/// Account update request
#[derive(Debug, Clone, Deserialize)]
pub struct AccountUpdate {
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub priority: Option<i32>,
    pub models: Option<Vec<ProviderModel>>,
    pub quotas: Option<Vec<QuotaUpdate>>,
    pub remove_quotas: Option<Vec<QuotaPeriod>>,
}

/// Quota tier update
#[derive(Debug, Clone, Deserialize)]
pub struct QuotaUpdate {
    pub period: QuotaPeriod,
    pub token_limit: u64,
    pub request_limit: Option<u64>,
}

/// Usage summary
#[derive(Debug, Clone, Serialize)]
pub struct ProviderUsageSummary {
    pub provider_id: String,
    pub total_accounts: usize,
    pub active_accounts: usize,
    pub exhausted_accounts: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_multi_tier_quota() {
        let mut account = ProviderAccount::new(
            "Test".into(),
            "openai".into(),
            AccountConfig::ApiKey(ApiKeyConfig {
                api_key: "sk-test".into(),
                organization_id: None,
                custom_endpoint: None,
            }),
        );
        
        // Set multiple quotas
        account.set_quota(QuotaPeriod::Minute, 1000, None);
        account.set_quota(QuotaPeriod::Hour, 10000, None);
        account.set_quota(QuotaPeriod::Day, 100000, None);
        account.set_quota(QuotaPeriod::Month, 1000000, None);
        
        assert!(account.has_quota());
        
        // Use up minute quota
        account.record_usage(1000, 1);
        assert!(!account.has_quota()); // Should fail - minute exhausted
        
        // Check blocking tier
        assert_eq!(account.blocking_tier(), Some(QuotaPeriod::Minute));
    }
    
    #[tokio::test]
    async fn test_account_rotation() {
        let manager = ProviderAccountManager::new();
        
        let mut account1 = ProviderAccount::new(
            "Primary".into(),
            "openai".into(),
            AccountConfig::ApiKey(ApiKeyConfig {
                api_key: "sk-1".into(),
                organization_id: None,
                custom_endpoint: None,
            }),
        );
        account1.set_quota(QuotaPeriod::Minute, 100, None);
        account1.priority = 1;
        
        let mut account2 = ProviderAccount::new(
            "Backup".into(),
            "openai".into(),
            AccountConfig::ApiKey(ApiKeyConfig {
                api_key: "sk-2".into(),
                organization_id: None,
                custom_endpoint: None,
            }),
        );
        account2.set_quota(QuotaPeriod::Minute, 100, None);
        account2.priority = 2;
        
        let acc1_id = account1.id.clone();
        
        manager.add_account(account1).await.unwrap();
        manager.add_account(account2).await.unwrap();
        
        // First should be primary (default)
        let available = manager.get_available_account("openai").await;
        assert!(available.is_some());
        assert_eq!(available.unwrap().name, "Primary");
        
        // Exhaust primary
        manager.record_usage(&acc1_id, 100, 1).await;
        
        // Should now get backup
        let available = manager.get_available_account("openai").await;
        assert!(available.is_some());
        assert_eq!(available.unwrap().name, "Backup");
    }
}
