//! Provider Account API handlers

use axum::{extract::{Path, State}, http::StatusCode, Json};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::api::state::AppState;
use crate::providers::account_manager::{
    ProviderAccount, ProviderDefinition, ProviderCategory, QuotaPeriod,
    AccountConfig, ApiKeyConfig, AzureConfig, AwsConfig, VectorDbConfig,
    AccountUpdate, ProviderUsageSummary, AccountDetailedStatus, QuotaUpdate,
};
use crate::types::ProviderModel;
use crate::error::Result;

/// List all providers
pub async fn list_providers(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<ProviderDefinition>> {
    Json(state.account_manager.list_providers().await)
}

/// Get accounts for a provider
pub async fn get_provider_accounts(
    State(state): State<Arc<AppState>>,
    Path(provider_id): Path<String>,
) -> Json<Vec<ProviderAccount>> {
    Json(state.account_manager.get_accounts(&provider_id).await)
}

/// Create a new provider account
pub async fn create_account(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateAccountRequest>,
) -> Result<(StatusCode, Json<ProviderAccount>)> {
    let config = match &req.config {
        AccountConfigRequest::ApiKey { api_key, organization_id, custom_endpoint } => {
            AccountConfig::ApiKey(ApiKeyConfig {
                api_key: api_key.clone(),
                organization_id: organization_id.clone(),
                custom_endpoint: custom_endpoint.clone(),
            })
        }
        AccountConfigRequest::Azure { endpoint, deployment_name, api_version, api_key } => {
            AccountConfig::Azure(AzureConfig {
                endpoint: endpoint.clone(),
                deployment_name: deployment_name.clone(),
                api_version: api_version.clone(),
                api_key: api_key.clone(),
            })
        }
        AccountConfigRequest::Aws { region, access_key_id, secret_access_key } => {
            AccountConfig::Aws(AwsConfig {
                region: region.clone(),
                access_key_id: access_key_id.clone(),
                secret_access_key: secret_access_key.clone(),
            })
        }
        AccountConfigRequest::VectorDb { url, api_key, collection_name } => {
            AccountConfig::VectorDb(VectorDbConfig {
                url: url.clone(),
                api_key: api_key.clone(),
                collection_name: collection_name.clone(),
            })
        }
    };
    
    let mut account = ProviderAccount::new(req.name.clone(), req.provider_id.clone(), config.clone());
    
    if let Some(priority) = req.priority {
        account.priority = priority;
    }
    if let Some(ref models) = req.models {
        account.models = models.clone();
    }
    
    // Set up quota tiers
    if let Some(ref quotas) = req.quotas {
        for q in quotas {
            account.set_quota(q.period, q.token_limit, q.request_limit);
        }
    }
    
    // Save to database if connected
    if let Some(ref pool) = state.db_pool {
        let api_key_encrypted = match &config {
            AccountConfig::ApiKey(cfg) => cfg.api_key.clone(),
            AccountConfig::Azure(cfg) => cfg.api_key.clone(),
            AccountConfig::Aws(_) => String::new(),
            AccountConfig::VectorDb(cfg) => cfg.api_key.clone().unwrap_or_default(),
        };
        
        let endpoint = match &config {
            AccountConfig::ApiKey(cfg) => cfg.custom_endpoint.clone(),
            AccountConfig::Azure(cfg) => Some(cfg.endpoint.clone()),
            AccountConfig::VectorDb(cfg) => Some(cfg.url.clone()),
            _ => None,
        };
        
        let models_json = serde_json::to_value(&account.models).unwrap_or_default();
        let quota_json = serde_json::to_value(&account.quotas).unwrap_or_default();
        
        let repo = crate::db::ProviderAccountRepository::new(pool.clone());
        if let Err(e) = repo.create(
            &account.id,
            &req.provider_id,
            &req.name,
            &api_key_encrypted,
            endpoint.as_deref(),
            None,
            None,
            &models_json,
            &quota_json,
        ).await {
            tracing::error!("Failed to save account to database: {}", e);
            // Continue anyway - it's saved in memory
        } else {
            tracing::info!("Saved provider account {} to database", account.id);
        }
    }
    
    let account = state.account_manager.add_account(account).await?;
    Ok((StatusCode::CREATED, Json(account)))
}

#[derive(Deserialize)]
pub struct CreateAccountRequest {
    pub name: String,
    pub provider_id: String,
    pub config: AccountConfigRequest,
    pub priority: Option<i32>,
    pub models: Option<Vec<ProviderModel>>,
    pub quotas: Option<Vec<QuotaRequest>>,
}

#[derive(Deserialize)]
pub struct QuotaRequest {
    pub period: QuotaPeriod,
    pub token_limit: u64,
    pub request_limit: Option<u64>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AccountConfigRequest {
    ApiKey {
        api_key: String,
        organization_id: Option<String>,
        custom_endpoint: Option<String>,
    },
    Azure {
        endpoint: String,
        deployment_name: String,
        api_version: String,
        api_key: String,
    },
    Aws {
        region: String,
        access_key_id: String,
        secret_access_key: String,
    },
    VectorDb {
        url: String,
        api_key: Option<String>,
        collection_name: Option<String>,
    },
}

/// Update an account
pub async fn update_account(
    State(state): State<Arc<AppState>>,
    Path(account_id): Path<String>,
    Json(req): Json<UpdateAccountRequest>,
) -> Result<Json<ProviderAccount>> {
    let update = AccountUpdate {
        name: req.name,
        enabled: req.enabled,
        priority: req.priority,
        models: req.models,
        quotas: req.quotas.map(|qs| {
            qs.into_iter().map(|q| QuotaUpdate {
                period: q.period,
                token_limit: q.token_limit,
                request_limit: q.request_limit,
            }).collect()
        }),
        remove_quotas: req.remove_quotas,
    };
    
    let account = state.account_manager.update_account(&account_id, update).await?;
    
    // Update database
    if let Some(ref pool) = state.db_pool {
        let repo = crate::db::ProviderAccountRepository::new(pool.clone());
        let models_json = serde_json::to_value(&account.models).unwrap_or_default();
        let quota_json = serde_json::to_value(&account.quotas).unwrap_or_default();
        
        if let Err(e) = repo.update(
            &account.id,
            &account.name,
            account.enabled,
            account.priority,
            &models_json,
            &quota_json,
        ).await {
            tracing::error!("Failed to update account in database: {}", e);
        }
    }
    
    Ok(Json(account))
}

#[derive(Deserialize)]
pub struct UpdateAccountRequest {
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub priority: Option<i32>,
    pub models: Option<Vec<ProviderModel>>,
    pub quotas: Option<Vec<QuotaRequest>>,
    pub remove_quotas: Option<Vec<QuotaPeriod>>,
}

/// Delete an account
pub async fn delete_account(
    State(state): State<Arc<AppState>>,
    Path(account_id): Path<String>,
) -> StatusCode {
    // Delete from database if connected
    if let Some(ref pool) = state.db_pool {
        let repo = crate::db::ProviderAccountRepository::new(pool.clone());
        if let Err(e) = repo.delete(&account_id).await {
            tracing::warn!("Failed to delete account from database: {}", e);
        } else {
            tracing::info!("Deleted provider account {} from database", account_id);
        }
    }
    
    if state.account_manager.delete_account(&account_id).await {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

/// Set default account for a provider
pub async fn set_default_account(
    State(state): State<Arc<AppState>>,
    Path((provider_id, account_id)): Path<(String, String)>,
) -> StatusCode {
    state.account_manager.set_default(&provider_id, &account_id).await;
    StatusCode::OK
}

/// Get usage summary for a provider
pub async fn get_provider_usage(
    State(state): State<Arc<AppState>>,
    Path(provider_id): Path<String>,
) -> Json<ProviderUsageSummary> {
    Json(state.account_manager.get_usage_summary(&provider_id).await)
}

/// Record usage
pub async fn record_usage(
    State(state): State<Arc<AppState>>,
    Path(account_id): Path<String>,
    Json(req): Json<RecordUsageRequest>,
) -> StatusCode {
    state.account_manager.record_usage(&account_id, req.tokens, req.requests).await;
    StatusCode::OK
}

#[derive(Deserialize)]
pub struct RecordUsageRequest {
    pub tokens: u64,
    pub requests: u64,
}

/// Get available account for a provider (for load balancing)
pub async fn get_available_account(
    State(state): State<Arc<AppState>>,
    Path(provider_id): Path<String>,
) -> Result<Json<ProviderAccount>> {
    match state.account_manager.get_available_account(&provider_id).await {
        Some(account) => Ok(Json(account)),
        None => Err(crate::error::SynapseError::NotFound(
            format!("No available account for provider: {}. All accounts may have exhausted their quota.", provider_id)
        )),
    }
}

/// Get detailed status of all accounts for a provider
pub async fn get_account_statuses(
    State(state): State<Arc<AppState>>,
    Path(provider_id): Path<String>,
) -> Json<Vec<AccountDetailedStatus>> {
    Json(state.account_manager.get_detailed_statuses(&provider_id).await)
}
