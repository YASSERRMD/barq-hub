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
use crate::db::{ProviderAccountRepository, ProviderAccountRow};
use uuid::Uuid;

/// Helper to convert DB row to ProviderAccount
fn row_to_account(row: ProviderAccountRow) -> ProviderAccount {
    // Parse models from JSON
    let models: Vec<crate::types::ProviderModel> = serde_json::from_value(row.models.clone()).unwrap_or_default();
    
    // Create account with basic ApiKey config (API key is encrypted in DB)
    // We try to reconstruct the config based on fields available
    let config = if let Some(endpoint) = &row.endpoint {
        if row.provider_id == "azure" || row.provider_id == "azure-openai" {
             AccountConfig::Azure(AzureConfig {
                endpoint: endpoint.clone(),
                deployment_name: row.deployment_name.unwrap_or_default(),
                api_version: "2023-05-15".to_string(), // Default or stored in metadata
                api_key: row.api_key_encrypted.unwrap_or_default(),
            })
        } else if let Ok(vector_cfg) = serde_json::from_value::<VectorDbConfig>(row.config.clone()) {
             AccountConfig::VectorDb(vector_cfg)
        } else {
             AccountConfig::ApiKey(ApiKeyConfig {
                api_key: row.api_key_encrypted.unwrap_or_default(),
                organization_id: None,
                custom_endpoint: Some(endpoint.clone()),
            })
        }
    } else {
         AccountConfig::ApiKey(ApiKeyConfig {
            api_key: row.api_key_encrypted.unwrap_or_default(),
            organization_id: None,
            custom_endpoint: None,
        })
    };
    
    let mut account = ProviderAccount::new(
        row.name,
        row.provider_id,
        config,
    );
    
    // Override fields from DB
    account.id = row.id;
    account.enabled = row.enabled;
    account.is_default = row.is_default;
    account.priority = row.priority;
    account.models = models;
    account.created_at = row.created_at;
    account.updated_at = row.updated_at;

    // Restore quotas
    if let Ok(quotas) = serde_json::from_value(row.quota_config) {
        account.quotas = quotas;
    }
    
    account
}

/// List all providers
pub async fn list_providers(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<ProviderDefinition>> {
    // Providers definitions are code-based for now as they define capabilities
    // But we could load them from DB if needed. User focused on accounts.
    Json(state.account_manager.list_providers().await)
}

/// Get accounts for a provider
pub async fn get_provider_accounts(
    State(state): State<Arc<AppState>>,
    Path(provider_id): Path<String>,
) -> Result<Json<Vec<ProviderAccount>>> {
    if let Some(ref pool) = state.db_pool {
        let repo = ProviderAccountRepository::new(pool.clone());
        let rows = repo.list_by_provider(&provider_id).await.map_err(|e| crate::error::SynapseError::DatabaseError(e.to_string()))?;
        let accounts = rows.into_iter().map(row_to_account).collect();
        Ok(Json(accounts))
    } else {
         Err(crate::error::SynapseError::DatabaseError("Database not connected".to_string()))
    }
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
    
    // Save to database
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

        let region = match &config {
             AccountConfig::Aws(cfg) => Some(cfg.region.clone()),
             _ => None,
        };

        let deployment_name = match &config {
             AccountConfig::Azure(cfg) => Some(cfg.deployment_name.clone()),
             _ => None,
        };
        
        let models_json = serde_json::to_value(&account.models).unwrap_or_default();
        let quota_json = serde_json::to_value(&account.quotas).unwrap_or_default();
        
        // Ensure ID is set
        if account.id.is_empty() {
            account.id = Uuid::new_v4().to_string();
        }

        let config_json = serde_json::to_value(&config).unwrap_or_default();

        let repo = ProviderAccountRepository::new(pool.clone());
        let _ = repo.create(
            &account.id,
            &req.provider_id,
            &req.name,
            &api_key_encrypted,
            endpoint.as_deref(),
            region.as_deref(),
            deployment_name.as_deref(),
            &models_json,
            &quota_json,
            &config_json,
        ).await.map_err(|e| crate::error::SynapseError::DatabaseError(e.to_string()))?;

        tracing::info!("Saved provider account {} to database", account.id);
        Ok((StatusCode::CREATED, Json(account)))
    } else {
        Err(crate::error::SynapseError::DatabaseError("Database not connected".to_string()))
    }
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
    if let Some(ref pool) = state.db_pool {
        let repo = ProviderAccountRepository::new(pool.clone());
        
        // Get existing account first to merge fields
        let existing_row = repo.find_by_id(&account_id).await
            .map_err(|e| crate::error::SynapseError::DatabaseError(e.to_string()))?
            .ok_or_else(|| crate::error::SynapseError::NotFound("Account not found".to_string()))?;
            
        let mut account = row_to_account(existing_row);

        if let Some(name) = req.name {
            account.name = name;
        }
        if let Some(enabled) = req.enabled {
            account.enabled = enabled;
        }
        if let Some(priority) = req.priority {
            account.priority = priority;
        }
        if let Some(models) = req.models {
            account.models = models;
        }
        
        // Update quota tiers
        if let Some(quotas) = req.quotas {
            for quota in quotas {
                account.set_quota(quota.period, quota.token_limit, quota.request_limit);
            }
        }
        
        // Remove quota tiers
        if let Some(remove_quotas) = req.remove_quotas {
            for period in remove_quotas {
                account.remove_quota(period);
            }
        }
        
        let models_json = serde_json::to_value(&account.models).unwrap_or_default();
        let quota_json = serde_json::to_value(&account.quotas).unwrap_or_default();
        let config_json = serde_json::to_value(&account.config).unwrap_or_default();
        
        repo.update(
            &account.id,
            &account.name,
            account.enabled,
            account.priority,
            &models_json,
            &quota_json,
            &config_json,
        ).await.map_err(|e| crate::error::SynapseError::DatabaseError(e.to_string()))?;
        
        Ok(Json(account))
    } else {
        Err(crate::error::SynapseError::DatabaseError("Database not connected".to_string()))
    }
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
    if let Some(ref pool) = state.db_pool {
        let repo = ProviderAccountRepository::new(pool.clone());
        if let Ok(true) = repo.delete(&account_id).await {
            StatusCode::NO_CONTENT
        } else {
             StatusCode::NOT_FOUND
        }
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

/// Set default account for a provider
pub async fn set_default_account(
    State(state): State<Arc<AppState>>,
    Path((provider_id, account_id)): Path<(String, String)>,
) -> StatusCode {
    if let Some(ref pool) = state.db_pool {
         let repo = ProviderAccountRepository::new(pool.clone());
         if let Ok(_) = repo.set_default(&provider_id, &account_id).await {
             StatusCode::OK
         } else {
             StatusCode::INTERNAL_SERVER_ERROR
         }
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

/// Get usage summary for a provider
pub async fn get_provider_usage(
    State(state): State<Arc<AppState>>,
    Path(provider_id): Path<String>,
) -> Json<ProviderUsageSummary> {
    // Usage stats might still need to aggregate from DB logs/metrics
    // For now, we return empty structure or implement DB aggregation 
    // This part might still use AccountManager if we were tracking in-memory usage
    // But user wants everything in DB.
    // TODO: Implement DB-based usage aggregation
    Json(ProviderUsageSummary {
        provider_id,
        total_accounts: 0,
        active_accounts: 0,
        exhausted_accounts: 0,
    })
}

/// Record usage
pub async fn record_usage(
    State(_state): State<Arc<AppState>>,
    Path(_account_id): Path<String>,
    Json(_req): Json<RecordUsageRequest>,
) -> StatusCode {
    // TODO: Write to cost_records table
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
    if let Some(ref pool) = state.db_pool {
        let repo = ProviderAccountRepository::new(pool.clone());
        
        // Logic: Try default first, then enabled accounts with quota
        if let Ok(Some(account_row)) = repo.get_default(&provider_id).await {
             Ok(Json(row_to_account(account_row)))
        } else {
            // Get all and pick one
            let rows = repo.list_by_provider(&provider_id).await.map_err(|e| crate::error::SynapseError::DatabaseError(e.to_string()))?;
            if let Some(row) = rows.into_iter().find(|r| r.enabled && !r.api_key_encrypted.clone().unwrap_or_default().is_empty()) {
                Ok(Json(row_to_account(row)))
            } else {
                Err(crate::error::SynapseError::NotFound(
                    format!("No available account for provider: {}", provider_id)
                ))
            }
        }
    } else {
         Err(crate::error::SynapseError::DatabaseError("Database not connected".to_string()))
    }
}

/// Get detailed status of all accounts for a provider
pub async fn get_account_statuses(
    State(state): State<Arc<AppState>>,
    Path(provider_id): Path<String>,
) -> Result<Json<Vec<AccountDetailedStatus>>> {
    if let Some(ref pool) = state.db_pool {
        let repo = ProviderAccountRepository::new(pool.clone());
        let rows = repo.list_by_provider(&provider_id).await.map_err(|e| crate::error::SynapseError::DatabaseError(e.to_string()))?;
        
        let statuses = rows.into_iter().map(|row| {
             let acc = row_to_account(row);
             AccountDetailedStatus {
                id: acc.id,
                name: acc.name,
                enabled: acc.enabled,
                is_default: acc.is_default,
                priority: acc.priority,
                has_quota: true, // TODO: Check actual quota usage from DB
                blocking_tier: None,
                next_reset: None,
                quota_tiers: vec![],
             }
        }).collect();
        
        Ok(Json(statuses))
    } else {
        Err(crate::error::SynapseError::DatabaseError("Database not connected".to_string()))
    }
}
