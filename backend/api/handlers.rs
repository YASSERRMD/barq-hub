//! API route handlers

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    ChatRequest, ChatResponse, Provider, HealthStatus, ComponentHealth,
    CostEntry, Budget,
    error::Result,
};
use super::state::AppState;
use crate::db::{ProviderAccountRepository, ProviderAccountRow};
use serde_json::Value as JsonValue;

// ============================================================================
// Chat Completions
// ============================================================================

/// POST /v1/chat/completions
pub async fn chat_completions(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>> {
    let user_id = request.user_id.clone().unwrap_or_else(|| "anonymous".to_string());

    // Check budget (estimate ~1000 tokens)
    state.cost_manager.can_request(&user_id, 0.01).await?;

    // If explicit provider is specified, query database directly for account
    if let Some(ref provider_id) = request.provider {
        if let Some(ref pool) = state.db_pool {
            let repo = ProviderAccountRepository::new(pool.clone());
            
            // Try to get default account first, then any enabled account
            let target_account = if let Ok(Some(account)) = repo.get_default(provider_id).await {
                Some(account)
            } else if let Ok(accounts) = repo.list_by_provider(provider_id).await {
                accounts.into_iter().find(|a| a.enabled && !a.api_key_encrypted.clone().unwrap_or_default().is_empty())
            } else {
                None
            };

            if let Some(account) = target_account {
                let api_key = account.api_key_encrypted.clone().unwrap_or_default();
                
                if !api_key.is_empty() {
                    let provider_type = match provider_id.as_str() {
                        "openai" => crate::ProviderType::OpenAI,
                        "anthropic" => crate::ProviderType::Anthropic,
                        "mistral" => crate::ProviderType::Mistral,
                        "cohere" => crate::ProviderType::Cohere,
                        "groq" => crate::ProviderType::Groq,
                        "together" => crate::ProviderType::Together,
                        "gemini" => crate::ProviderType::Gemini,
                        "azure" | "azure-openai" => crate::ProviderType::AzureOpenAI,
                        "bedrock" => crate::ProviderType::Bedrock,
                        _ => crate::ProviderType::OpenAI,
                    };

                    let base_url = get_provider_base_url(provider_id, account.endpoint.as_deref());

                    let provider = crate::Provider {
                        id: provider_id.clone(),
                        name: account.name.clone(),
                        provider_type,
                        api_key,
                        models: Vec::new(),
                        base_url,
                        pricing: crate::ProviderPricing {
                            input_token_cost: 0.0,
                            output_token_cost: 0.0,
                        },
                        enabled: true,
                        health: crate::ProviderHealth::default(),
                        headers: std::collections::HashMap::new(),
                    };

                    let adapter = crate::providers::create_adapter(provider, state.http_client.clone());
                    match adapter.chat(&request).await {
                        Ok(response) => {
                            // Record cost to database
                            state.cost_manager.record_cost(
                                &response.provider,
                                &response.model,
                                &response.usage,
                                response.cost,
                                &user_id,
                                &response.id,
                            ).await?;

                            return Ok(Json(response));
                        }
                        Err(e) => {
                            tracing::warn!(provider = %provider_id, error = %e, "Provider failed, trying router");
                        }
                    }
                }
            }
        }
    }

    // Fall back to router for providers not in database
    let response = state.router.route_with_fallback(&request).await?;

    // Record cost
    state.cost_manager.record_cost(
        &response.provider,
        &response.model,
        &response.usage,
        response.cost,
        &user_id,
        &response.id,
    ).await?;

    Ok(Json(response))
}

/// Get the base URL for a provider (database-backed version)
fn get_provider_base_url(provider_id: &str, custom_endpoint: Option<&str>) -> String {
    if let Some(endpoint) = custom_endpoint {
        if !endpoint.is_empty() {
            return endpoint.to_string();
        }
    }

    match provider_id {
        "openai" => "https://api.openai.com/v1".to_string(),
        "anthropic" => "https://api.anthropic.com/v1".to_string(),
        "mistral" => "https://api.mistral.ai/v1".to_string(),
        "cohere" => "https://api.cohere.ai/v1".to_string(),
        "groq" => "https://api.groq.com/openai/v1".to_string(),
        "together" => "https://api.together.xyz/v1".to_string(),
        "gemini" => "https://generativelanguage.googleapis.com/v1".to_string(),
        _ => format!("https://api.{}.com/v1", provider_id),
    }
}

// ============================================================================
// Providers
// ============================================================================

/// GET /v1/providers
pub async fn list_providers(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<ProviderInfo>> {
    let providers = state.providers.read().await;
    
    let info: Vec<ProviderInfo> = providers
        .iter()
        .map(|p| ProviderInfo {
            id: p.id.clone(),
            name: p.name.clone(),
            provider_type: format!("{:?}", p.provider_type),
            base_url: p.base_url.clone(),
            enabled: p.enabled,
            healthy: p.health.healthy,
        })
        .collect();
    
    Json(info)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub base_url: String,
    pub enabled: bool,
    pub healthy: bool,
}

/// POST /v1/providers
pub async fn create_provider(
    State(state): State<Arc<AppState>>,
    Json(provider): Json<Provider>,
) -> Result<(StatusCode, Json<Provider>)> {
    let mut providers = state.providers.write().await;
    providers.push(provider.clone());
    Ok((StatusCode::CREATED, Json(provider)))
}

/// DELETE /v1/providers/:id
pub async fn delete_provider(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> StatusCode {
    let mut providers = state.providers.write().await;
    let len_before = providers.len();
    providers.retain(|p| p.id != id);
    
    if providers.len() < len_before {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

// ============================================================================
// Health & Status
// ============================================================================

/// GET /health
pub async fn health_check(
    State(state): State<Arc<AppState>>,
) -> Json<HealthStatus> {
    let mut components = std::collections::HashMap::new();
    
    // Check providers
    let provider_health = state.router.health_check_all().await;
    for (id, healthy) in provider_health {
        components.insert(id.clone(), ComponentHealth {
            name: id,
            healthy,
            message: None,
            latency_ms: None,
        });
    }
    
    let all_healthy = components.values().all(|c| c.healthy) || components.is_empty();
    
    Json(HealthStatus {
        status: if all_healthy { "healthy".to_string() } else { "degraded".to_string() },
        uptime_seconds: state.uptime_seconds(),
        version: state.version.clone(),
        components,
    })
}

/// GET /v1/status
pub async fn status(
    State(state): State<Arc<AppState>>,
) -> Json<StatusResponse> {
    let providers = state.router.list_providers();
    
    Json(StatusResponse {
        status: "running".to_string(),
        version: state.version.clone(),
        uptime_seconds: state.uptime_seconds(),
        providers_count: providers.len(),
        providers,
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CostSummary {
    pub total_cost: f64,
    pub request_count: usize,
    pub total_tokens: u64,
    pub by_provider: Vec<ProviderBreakdown>,
    pub by_model: Vec<ModelBreakdown>,
    pub by_user: Vec<UserBreakdown>,
    pub period_start: chrono::DateTime<chrono::Utc>,
    pub period_end: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderBreakdown {
    pub provider: String,
    pub cost: f64,
    pub requests: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelBreakdown {
    pub model: String,
    pub cost: f64,
    pub requests: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserBreakdown {
    pub user_id: String,
    pub cost: f64,
    pub requests: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub providers_count: usize,
    pub providers: Vec<String>,
}

// ============================================================================
// Costs
// ============================================================================

/// GET /v1/costs
pub async fn get_costs(
    State(state): State<Arc<AppState>>,
    Query(params): Query<CostQueryParams>,
) -> Json<CostSummary> {
    let end = Utc::now();
    let start = params.days
        .map(|d| end - Duration::days(d as i64))
        .unwrap_or_else(|| end - Duration::days(30));
    
    // Try database first
    if let Some(ref pool) = state.db_pool {
        // Total stats
        let (request_count, total_cost, total_tokens) = sqlx::query_as::<_, (i64, f64, i64)>(
            "SELECT COUNT(*), COALESCE(SUM(cost::float8), 0.0), COALESCE(SUM(total_tokens), 0) FROM cost_entries WHERE created_at >= $1 AND created_at <= $2"
        )
        .bind(start)
        .bind(end)
        .fetch_one(pool)
        .await
        .unwrap_or((0, 0.0, 0));

        // By Provider
        let provider_rows = sqlx::query_as::<_, (String, f64, i64)>(
            "SELECT provider, COALESCE(SUM(cost::float8), 0.0), COUNT(*) FROM cost_entries WHERE created_at >= $1 AND created_at <= $2 GROUP BY provider"
        )
        .bind(start)
        .bind(end)
        .fetch_all(pool)
        .await
        .unwrap_or_default();
        
        let by_provider: Vec<ProviderBreakdown> = provider_rows.into_iter().map(|(p, c, r)| ProviderBreakdown {
            provider: p,
            cost: c,
            requests: r,
        }).collect();

        // By Model
        let model_rows = sqlx::query_as::<_, (String, f64, i64)>(
            "SELECT model, COALESCE(SUM(cost::float8), 0.0), COUNT(*) FROM cost_entries WHERE created_at >= $1 AND created_at <= $2 GROUP BY model"
        )
        .bind(start)
        .bind(end)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        let by_model: Vec<ModelBreakdown> = model_rows.into_iter().map(|(m, c, r)| ModelBreakdown {
            model: m,
            cost: c,
            requests: r,
        }).collect();
        
        // By User
        let user_rows = sqlx::query_as::<_, (String, f64, i64)>(
            "SELECT user_id, COALESCE(SUM(cost::float8), 0.0), COUNT(*) FROM cost_entries WHERE created_at >= $1 AND created_at <= $2 GROUP BY user_id"
        )
        .bind(start)
        .bind(end)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        let by_user: Vec<UserBreakdown> = user_rows.into_iter().map(|(u, c, r)| UserBreakdown {
            user_id: u,
            cost: c,
            requests: r,
        }).collect();

        return Json(CostSummary {
            total_cost,
            request_count: request_count as usize,
            total_tokens: total_tokens as u64,
            by_provider,
            by_model,
            by_user,
            period_start: start,
            period_end: end,
        });
    }
    
    // Fallback moved to CostManager if needed, or return empty
    // For now returning empty struct since we focused on DB
    Json(CostSummary {
        total_cost: 0.0,
        request_count: 0,
        total_tokens: 0,
        by_provider: vec![],
        by_model: vec![],
        by_user: vec![],
        period_start: start,
        period_end: end,
    })
}

#[derive(Debug, Deserialize)]
pub struct CostQueryParams {
    pub days: Option<u32>,
}

/// GET /v1/costs/recent
pub async fn get_recent_costs(
    State(state): State<Arc<AppState>>,
    Query(params): Query<LimitParams>,
) -> Json<Vec<CostEntry>> {
    let limit = params.limit.unwrap_or(100) as i64;
    
    // Try database first
    if let Some(ref pool) = state.db_pool {
        if let Ok(rows) = sqlx::query_as::<_, (String, Option<String>, Option<String>, String, String, i32, i32, i32, f64, chrono::DateTime<chrono::Utc>)>(
            "SELECT id, user_id, agent_id, provider, model, prompt_tokens, completion_tokens, total_tokens, cost::float8, created_at FROM cost_entries ORDER BY created_at DESC LIMIT $1"
        )
        .bind(limit)
        .fetch_all(pool)
        .await {
            let entries: Vec<CostEntry> = rows.into_iter().map(|(id, user_id, _agent_id, provider, model, prompt_tokens, completion_tokens, _total_tokens, cost, created_at)| {
                CostEntry {
                    id,
                    timestamp: created_at,
                    provider,
                    model,
                    input_tokens: prompt_tokens as u32,
                    output_tokens: completion_tokens as u32,
                    cost,
                    user_id: user_id.unwrap_or_default(),
                    request_id: "".to_string(), // Default empty as it's not in the query yet
                }
            }).collect();
            return Json(entries);
        }
    }
    
    // Fallback to in-memory
    let entries = state.cost_manager.get_recent_entries(limit as usize).await;
    Json(entries)
}

#[derive(Debug, Deserialize)]
pub struct LimitParams {
    pub limit: Option<usize>,
}

/// GET /v1/costs/user/:user_id
pub async fn get_user_costs(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
    Query(params): Query<LimitParams>,
) -> Json<Vec<CostEntry>> {
    let limit = params.limit.unwrap_or(100);
    let entries = state.cost_manager.get_user_entries(&user_id, limit).await;
    Json(entries)
}

// ============================================================================
// Budgets
// ============================================================================

/// GET /v1/budgets/:entity_id
pub async fn get_budget(
    State(state): State<Arc<AppState>>,
    Path(entity_id): Path<String>,
) -> Result<Json<Budget>> {
    match state.cost_manager.get_budget(&entity_id).await {
        Some(budget) => Ok(Json(budget)),
        None => Err(crate::error::SynapseError::Validation(
            format!("Budget not found for: {}", entity_id)
        )),
    }
}

/// POST /v1/budgets
pub async fn set_budget(
    State(state): State<Arc<AppState>>,
    Json(budget_req): Json<SetBudgetRequest>,
) -> StatusCode {
    state.cost_manager.set_budget(
        &budget_req.entity_id,
        budget_req.monthly_limit,
        budget_req.enforce.unwrap_or(true),
    ).await;
    
    StatusCode::CREATED
}

#[derive(Debug, Deserialize)]
pub struct SetBudgetRequest {
    pub entity_id: String,
    pub monthly_limit: f64,
    pub enforce: Option<bool>,
}

// ============================================================================
// Models
// ============================================================================

/// GET /v1/models
pub async fn list_models(
    State(_state): State<Arc<AppState>>,
) -> Json<ModelsResponse> {
    // Return static list of commonly used models
    let models = vec![
        ModelInfo { id: "gpt-4".to_string(), provider: "openai".to_string() },
        ModelInfo { id: "gpt-4-turbo".to_string(), provider: "openai".to_string() },
        ModelInfo { id: "gpt-4o".to_string(), provider: "openai".to_string() },
        ModelInfo { id: "gpt-4o-mini".to_string(), provider: "openai".to_string() },
        ModelInfo { id: "gpt-3.5-turbo".to_string(), provider: "openai".to_string() },
        ModelInfo { id: "claude-3-opus-20240229".to_string(), provider: "anthropic".to_string() },
        ModelInfo { id: "claude-3-sonnet-20240229".to_string(), provider: "anthropic".to_string() },
        ModelInfo { id: "claude-3-5-sonnet-20241022".to_string(), provider: "anthropic".to_string() },
        ModelInfo { id: "mistral-large-latest".to_string(), provider: "mistral".to_string() },
        ModelInfo { id: "mistral-medium-latest".to_string(), provider: "mistral".to_string() },
    ];
    
    Json(ModelsResponse { data: models })
}

#[derive(Debug, Serialize)]
pub struct ModelsResponse {
    pub data: Vec<ModelInfo>,
}

#[derive(Debug, Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub provider: String,
}

// ============================================================================
// Error Handler
// ============================================================================

/// Fallback handler for 404
pub async fn not_found() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "error": "Not found",
            "code": "NOT_FOUND"
        }))
    )
}
