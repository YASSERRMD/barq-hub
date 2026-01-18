//! Admin API handlers for users, roles, and system health

use axum::{extract::{Path, State}, http::StatusCode, Json};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::api::state::AppState;
use crate::governance::{User, ApiKey, UserUpdate};
use crate::error::Result;

// ============== USER MANAGEMENT ==============

#[derive(Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub api_keys_count: usize,
}

#[derive(Serialize)]
pub struct UserStats {
    pub total_users: usize,
    pub active_users: usize,
    pub admins: usize,
}

pub async fn list_users(State(state): State<Arc<AppState>>) -> Json<Vec<UserResponse>> {
    let users = state.auth_service.list_users().await;
    let mut responses = Vec::new();
    
    for user in users {
        let api_keys = state.auth_service.list_api_keys(&user.id).await;
        responses.push(UserResponse {
            id: user.id,
            email: user.email,
            name: user.name,
            role: user.role,
            enabled: user.enabled,
            created_at: user.created_at,
            last_login: user.last_login,
            api_keys_count: api_keys.len(),
        });
    }
    
    Json(responses)
}

pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> Result<Json<UserResponse>> {
    match state.auth_service.get_user(&user_id).await {
        Some(user) => {
            let api_keys = state.auth_service.list_api_keys(&user.id).await;
            Ok(Json(UserResponse {
                id: user.id,
                email: user.email,
                name: user.name,
                role: user.role,
                enabled: user.enabled,
                created_at: user.created_at,
                last_login: user.last_login,
                api_keys_count: api_keys.len(),
            }))
        }
        None => Err(crate::error::SynapseError::NotFound("User not found".into())),
    }
}

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub name: String,
    pub password: String,
    pub role: Option<String>,
}

pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>)> {
    let user = state.auth_service.create_user(&req.email, &req.name, &req.password).await?;
    
    // Set role if provided
    if let Some(role) = req.role {
        state.auth_service.update_user(&user.id, UserUpdate {
            name: None,
            email: None,
            role: Some(role),
            enabled: None,
            password: None,
        }).await;
    }
    
    let user = state.auth_service.get_user(&user.id).await.unwrap();
    
    Ok((StatusCode::CREATED, Json(UserResponse {
        id: user.id,
        email: user.email,
        name: user.name,
        role: user.role,
        enabled: user.enabled,
        created_at: user.created_at,
        last_login: user.last_login,
        api_keys_count: 0,
    })))
}

pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
    Json(update): Json<UserUpdate>,
) -> Result<Json<UserResponse>> {
    match state.auth_service.update_user(&user_id, update).await {
        Some(user) => {
            let api_keys = state.auth_service.list_api_keys(&user.id).await;
            Ok(Json(UserResponse {
                id: user.id,
                email: user.email,
                name: user.name,
                role: user.role,
                enabled: user.enabled,
                created_at: user.created_at,
                last_login: user.last_login,
                api_keys_count: api_keys.len(),
            }))
        }
        None => Err(crate::error::SynapseError::NotFound("User not found".into())),
    }
}

pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> StatusCode {
    if state.auth_service.delete_user(&user_id).await {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

pub async fn get_user_stats(State(state): State<Arc<AppState>>) -> Json<UserStats> {
    let users = state.auth_service.list_users().await;
    let admins = users.iter().filter(|u| u.role == "admin").count();
    
    Json(UserStats {
        total_users: users.len(),
        active_users: users.iter().filter(|u| u.enabled).count(),
        admins,
    })
}

// ============== API KEYS ==============

#[derive(Serialize)]
pub struct ApiKeyResponse {
    pub id: String,
    pub name: String,
    pub prefix: String,
    pub scopes: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub enabled: bool,
}

pub async fn list_user_api_keys(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> Json<Vec<ApiKeyResponse>> {
    let keys = state.auth_service.list_api_keys(&user_id).await;
    Json(keys.into_iter().map(|k| ApiKeyResponse {
        id: k.id,
        name: k.name,
        prefix: k.prefix,
        scopes: k.scopes,
        created_at: k.created_at,
        last_used: k.last_used,
        expires_at: k.expires_at,
        enabled: k.enabled,
    }).collect())
}

#[derive(Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub scopes: Option<Vec<String>>,
    pub expires_in_days: Option<i64>,
}

#[derive(Serialize)]
pub struct CreateApiKeyResponse {
    pub key: String,
    pub api_key: ApiKeyResponse,
}

pub async fn create_api_key(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
    Json(req): Json<CreateApiKeyRequest>,
) -> (StatusCode, Json<CreateApiKeyResponse>) {
    let (raw_key, api_key) = state.auth_service.create_api_key(
        &user_id, 
        &req.name, 
        req.scopes.unwrap_or_default(),
        req.expires_in_days
    ).await;
    
    (StatusCode::CREATED, Json(CreateApiKeyResponse {
        key: raw_key,
        api_key: ApiKeyResponse {
            id: api_key.id,
            name: api_key.name,
            prefix: api_key.prefix,
            scopes: api_key.scopes,
            created_at: api_key.created_at,
            last_used: api_key.last_used,
            expires_at: api_key.expires_at,
            enabled: api_key.enabled,
        },
    }))
}

pub async fn delete_api_key(
    State(state): State<Arc<AppState>>,
    Path(key_id): Path<String>,
) -> StatusCode {
    if state.auth_service.delete_api_key(&key_id).await {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

// ============== SYSTEM HEALTH ==============

#[derive(Serialize)]
pub struct ServiceHealth {
    pub name: String,
    pub status: String, // "healthy", "degraded", "down"
    pub latency_ms: u64,
    pub details: String,
    pub last_check: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct SystemMetrics {
    pub uptime_seconds: u64,
    pub total_requests: u64,
    pub active_sessions: usize,
    pub active_accounts: usize,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub services: Vec<ServiceHealth>,
    pub metrics: SystemMetrics,
}

pub async fn get_system_health(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    let now = Utc::now();
    let mut services = Vec::new();
    
    // Backend API is always healthy if we're responding
    services.push(ServiceHealth {
        name: "Backend API".to_string(),
        status: "healthy".to_string(),
        latency_ms: 1,
        details: "Responding to requests".to_string(),
        last_check: now,
    });
    
    // Check PostgreSQL connection
    let (pg_status, pg_details, pg_latency) = if let Some(ref pool) = state.db_pool {
        let start = std::time::Instant::now();
        match sqlx::query("SELECT 1").fetch_one(pool).await {
            Ok(_) => {
                let latency = start.elapsed().as_millis() as u64;
                ("healthy".to_string(), "Connected".to_string(), latency)
            },
            Err(e) => ("down".to_string(), format!("Error: {}", e), 0),
        }
    } else {
        ("down".to_string(), "Not configured".to_string(), 0)
    };
    
    services.push(ServiceHealth {
        name: "PostgreSQL".to_string(),
        status: pg_status.clone(),
        latency_ms: pg_latency,
        details: pg_details.clone(),
        last_check: now,
    });
    
    // Check Redis connection (we don't have direct access, assume healthy if DB is healthy)
    let (redis_status, redis_details) = if pg_status == "healthy" {
        ("healthy".to_string(), "Assumed connected".to_string())
    } else {
        ("unknown".to_string(), "Cannot verify".to_string())
    };
    
    services.push(ServiceHealth {
        name: "Redis".to_string(),
        status: redis_status,
        latency_ms: 0,
        details: redis_details,
        last_check: now,
    });
    
    // Check LLM Providers
    let provider_count = state.account_manager.list_providers().await.len();
    let (provider_status, provider_details) = if provider_count > 0 {
        ("healthy".to_string(), format!("{} providers loaded", provider_count))
    } else {
        ("degraded".to_string(), "No providers configured".to_string())
    };
    
    services.push(ServiceHealth {
        name: "LLM Providers".to_string(),
        status: provider_status,
        latency_ms: 0,
        details: provider_details,
        last_check: now,
    });
    
    let all_healthy = services.iter().all(|s| s.status == "healthy");
    let any_down = services.iter().any(|s| s.status == "down");
    
    let overall_status = if all_healthy {
        "healthy"
    } else if any_down {
        "degraded"
    } else {
        "degraded"
    };
    
    Json(HealthResponse {
        status: overall_status.to_string(),
        services,
        metrics: SystemMetrics {
            uptime_seconds: state.uptime_seconds(),
            total_requests: 0, // Would come from metrics counter
            active_sessions: 0,
            active_accounts: provider_count,
        },
    })
}

// ============== ROLES MANAGEMENT ==============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub permissions: Vec<String>,
    pub is_system: bool,
}

pub async fn list_role_definitions() -> Json<Vec<RoleDefinition>> {
    // Return predefined roles
    Json(vec![
        RoleDefinition {
            id: "admin".to_string(),
            name: "Admin".to_string(),
            description: "Full system access".to_string(),
            permissions: vec![
                "agents:read".to_string(), "agents:write".to_string(), "agents:delete".to_string(),
                "knowledge:read".to_string(), "knowledge:write".to_string(),
                "providers:read".to_string(), "providers:write".to_string(),
                "billing:read".to_string(), "billing:write".to_string(),
                "users:read".to_string(), "users:write".to_string(),
            ],
            is_system: true,
        },
        RoleDefinition {
            id: "developer".to_string(),
            name: "Developer".to_string(),
            description: "Agent and knowledge management".to_string(),
            permissions: vec![
                "agents:read".to_string(), "agents:write".to_string(), "agents:delete".to_string(),
                "knowledge:read".to_string(), "knowledge:write".to_string(),
                "providers:read".to_string(),
            ],
            is_system: true,
        },
        RoleDefinition {
            id: "user".to_string(),
            name: "User".to_string(),
            description: "Standard user access".to_string(),
            permissions: vec![
                "agents:read".to_string(),
                "knowledge:read".to_string(),
            ],
            is_system: true,
        },
        RoleDefinition {
            id: "viewer".to_string(),
            name: "Viewer".to_string(),
            description: "Read-only access".to_string(),
            permissions: vec![
                "agents:read".to_string(),
                "knowledge:read".to_string(),
                "billing:read".to_string(),
            ],
            is_system: true,
        },
    ])
}

pub async fn list_permissions() -> Json<Vec<Permission>> {
    Json(vec![
        Permission { id: "agents:read".to_string(), name: "View Agents".to_string(), description: "View agent configurations".to_string(), category: "Agents".to_string() },
        Permission { id: "agents:write".to_string(), name: "Manage Agents".to_string(), description: "Create and modify agents".to_string(), category: "Agents".to_string() },
        Permission { id: "agents:delete".to_string(), name: "Delete Agents".to_string(), description: "Delete agent configurations".to_string(), category: "Agents".to_string() },
        Permission { id: "knowledge:read".to_string(), name: "View Knowledge".to_string(), description: "Search knowledge base".to_string(), category: "Knowledge".to_string() },
        Permission { id: "knowledge:write".to_string(), name: "Manage Knowledge".to_string(), description: "Upload and modify documents".to_string(), category: "Knowledge".to_string() },
        Permission { id: "providers:read".to_string(), name: "View Providers".to_string(), description: "View provider configurations".to_string(), category: "Admin".to_string() },
        Permission { id: "providers:write".to_string(), name: "Manage Providers".to_string(), description: "Configure LLM providers".to_string(), category: "Admin".to_string() },
        Permission { id: "billing:read".to_string(), name: "View Billing".to_string(), description: "View usage and costs".to_string(), category: "Billing".to_string() },
        Permission { id: "billing:write".to_string(), name: "Manage Billing".to_string(), description: "Manage budgets and limits".to_string(), category: "Billing".to_string() },
        Permission { id: "users:read".to_string(), name: "View Users".to_string(), description: "View user list".to_string(), category: "Admin".to_string() },
        Permission { id: "users:write".to_string(), name: "Manage Users".to_string(), description: "Create and modify users".to_string(), category: "Admin".to_string() },
    ])
}

// ============== APPLICATION MANAGEMENT ==============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Application {
    pub id: String,
    pub name: String,
    pub description: String,
    pub api_key_prefix: String,
    pub api_key_hash: String,
    pub scopes: Vec<String>,
    pub rate_limit: u32,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub requests_today: u64,
}

#[derive(Serialize)]
pub struct ApplicationResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub api_key_prefix: String,
    pub scopes: Vec<String>,
    pub rate_limit: u32,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub requests_today: u64,
}

#[derive(Deserialize)]
pub struct CreateApplicationRequest {
    pub name: String,
    pub description: Option<String>,
    pub scopes: Vec<String>,
    pub rate_limit: Option<u32>,
    pub expires_in_days: Option<i64>,
}

#[derive(Serialize)]
pub struct CreateApplicationResponse {
    pub application: ApplicationResponse,
    pub api_key: String, // Full key, only shown once
}

pub async fn list_applications(State(state): State<Arc<AppState>>) -> Json<Vec<ApplicationResponse>> {
    if let Some(repo) = &state.application_repo {
        match repo.list_all().await {
            Ok(apps) => {
                let responses = apps.into_iter().map(|app| {
                    let scopes: Vec<String> = serde_json::from_value(app.scopes).unwrap_or_default();
                    ApplicationResponse {
                        id: app.id,
                        name: app.name,
                        description: app.description.unwrap_or_default(),
                        api_key_prefix: app.api_key_prefix,
                        scopes,
                        rate_limit: app.rate_limit as u32,
                        status: app.status,
                        created_at: app.created_at,
                        last_used: app.last_used,
                        expires_at: app.expires_at,
                        requests_today: app.requests_today as u64,
                    }
                }).collect();
                Json(responses)
            },
            Err(e) => {
                tracing::error!("Failed to list applications: {}", e);
                Json(Vec::new())
            }
        }
    } else {
        // Fallback or empty if no DB
        Json(Vec::new())
    }
}

pub async fn create_application(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateApplicationRequest>,
) -> Result<(StatusCode, Json<CreateApplicationResponse>)> {
    use uuid::Uuid;
    use md5::{Md5, Digest};
    
    // Generate API key
    let raw_key = format!("sk-synapse-{}", Uuid::new_v4().to_string().replace("-", ""));
    let prefix = format!("sk-...{}", &raw_key[raw_key.len()-4..]);
    
    // Hash key
    let mut hasher = Md5::new();
    hasher.update(raw_key.as_bytes());
    let key_hash = format!("{:x}", hasher.finalize());

    if let Some(repo) = &state.application_repo {
        let app_id = Uuid::new_v4().to_string();
        let scopes_json = serde_json::to_value(&req.scopes).unwrap_or(serde_json::json!([]));
        
        let expires_at = req.expires_in_days.map(|d| Utc::now() + chrono::Duration::days(d));

        match repo.create(
            &app_id,
            &req.name,
            req.description.as_deref(),
            &key_hash,
            &prefix,
            &scopes_json,
            req.rate_limit.unwrap_or(100) as i32,
            expires_at
        ).await {
            Ok(row) => {
                let scopes: Vec<String> = serde_json::from_value(row.scopes).unwrap_or_default();
                Ok((StatusCode::CREATED, Json(CreateApplicationResponse {
                    application: ApplicationResponse {
                        id: row.id,
                        name: row.name,
                        description: row.description.unwrap_or_default(),
                        api_key_prefix: row.api_key_prefix,
                        scopes,
                        rate_limit: row.rate_limit as u32,
                        status: row.status,
                        created_at: row.created_at,
                        last_used: row.last_used,
                        expires_at: row.expires_at,
                        requests_today: row.requests_today as u64,
                    },
                    api_key: raw_key,
                })))
            },
            Err(e) => {
                tracing::error!("Failed to create application: {}", e);
                Err(crate::error::SynapseError::DatabaseError(e.to_string()))
            }
        }
    } else {
        Err(crate::error::SynapseError::Internal("Database not available".into()))
    }
}

#[derive(Deserialize)]
pub struct UpdateApplicationRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub scopes: Option<Vec<String>>,
    pub rate_limit: Option<u32>,
}

pub async fn update_application(
    State(state): State<Arc<AppState>>,
    Path(app_id): Path<String>,
    Json(req): Json<UpdateApplicationRequest>,
) -> Result<Json<ApplicationResponse>> {
    if let Some(repo) = &state.application_repo {
        let scopes_json = req.scopes.map(|s| serde_json::to_value(s).unwrap_or(serde_json::json!([])));
        
        match repo.update(
            &app_id,
            req.name.as_deref(),
            req.description.as_deref(),
            scopes_json.as_ref(),
            req.rate_limit.map(|r| r as i32),
            None // Not updating expires_at through this endpoint for now
        ).await {
            Ok(Some(row)) => {
                let scopes: Vec<String> = serde_json::from_value(row.scopes).unwrap_or_default();
                Ok(Json(ApplicationResponse {
                    id: row.id,
                    name: row.name,
                    description: row.description.unwrap_or_default(),
                    api_key_prefix: row.api_key_prefix,
                    scopes,
                    rate_limit: row.rate_limit as u32,
                    status: row.status,
                    created_at: row.created_at,
                    last_used: row.last_used,
                    expires_at: row.expires_at,
                    requests_today: row.requests_today as u64,
                }))
            },
            Ok(None) => Err(crate::error::SynapseError::NotFound("Application not found".into())),
            Err(e) => {
                tracing::error!("Failed to update application: {}", e);
                Err(crate::error::SynapseError::DatabaseError(e.to_string()))
            }
        }
    } else {
        Err(crate::error::SynapseError::Internal("Database not available".into()))
    }
}

pub async fn delete_application(
    State(state): State<Arc<AppState>>,
    Path(app_id): Path<String>,
) -> Result<StatusCode> {
    if let Some(repo) = &state.application_repo {
        match repo.delete(&app_id).await {
            Ok(deleted) => {
                if deleted {
                    Ok(StatusCode::NO_CONTENT)
                } else {
                    Err(crate::error::SynapseError::NotFound("Application not found".into()))
                }
            },
            Err(e) => {
                tracing::error!("Failed to delete application: {}", e);
                Err(crate::error::SynapseError::DatabaseError(e.to_string()))
            }
        }
    } else {
        Err(crate::error::SynapseError::Internal("Database not available".into()))
    }
}

pub async fn rotate_application_key(
    State(state): State<Arc<AppState>>,
    Path(app_id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    use uuid::Uuid;
    use md5::{Md5, Digest};

    if let Some(repo) = &state.application_repo {
        // Generate new key
        let raw_key = format!("sk-synapse-{}", Uuid::new_v4().to_string().replace("-", ""));
        let prefix = format!("sk-...{}", &raw_key[raw_key.len()-4..]);
        
        // Hash key
        let mut hasher = Md5::new();
        hasher.update(raw_key.as_bytes());
        let key_hash = format!("{:x}", hasher.finalize());
        
        match repo.rotate_key(&app_id, &key_hash, &prefix).await {
            Ok(rotated) => {
                if rotated {
                    Ok(Json(serde_json::json!({ "api_key": raw_key })))
                } else {
                    Err(crate::error::SynapseError::NotFound("Application not found".into()))
                }
            },
            Err(e) => {
                tracing::error!("Failed to rotate key: {}", e);
                Err(crate::error::SynapseError::DatabaseError(e.to_string()))
            }
        }
    } else {
        Err(crate::error::SynapseError::Internal("Database not available".into()))
    }
}

#[derive(Serialize)]
pub struct ApiScope {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
}

pub async fn list_api_scopes() -> Json<Vec<ApiScope>> {
    Json(vec![
        ApiScope { id: "llm:chat".to_string(), name: "Chat Completions".to_string(), description: "Generate text with LLMs".to_string(), category: "LLM".to_string() },
        ApiScope { id: "llm:models".to_string(), name: "List Models".to_string(), description: "View available models".to_string(), category: "LLM".to_string() },
        ApiScope { id: "embedding:create".to_string(), name: "Create Embeddings".to_string(), description: "Generate vector embeddings".to_string(), category: "Embedding".to_string() },
        ApiScope { id: "knowledge:search".to_string(), name: "Search Knowledge".to_string(), description: "Query vector store".to_string(), category: "Vector Store".to_string() },
        ApiScope { id: "knowledge:ingest".to_string(), name: "Ingest Documents".to_string(), description: "Add documents to knowledge base".to_string(), category: "Vector Store".to_string() },
        ApiScope { id: "agents:chat".to_string(), name: "Agent Chat".to_string(), description: "Interact with agents".to_string(), category: "Agents".to_string() },
        ApiScope { id: "agents:manage".to_string(), name: "Manage Agents".to_string(), description: "Create/update agents".to_string(), category: "Agents".to_string() },
    ])
}
