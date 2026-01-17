//! Governance API handlers

use axum::{extract::{Path, State, Query}, http::StatusCode, Json};
use std::sync::Arc;
use crate::api::state::AppState;
use crate::governance::{User, Session, Role, AuditEvent, AuditQuery};
use crate::error::Result;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LoginRequest { pub email: String, pub password: String }

#[derive(Deserialize)]
pub struct RegisterRequest { pub email: String, pub name: String, pub password: String }

#[derive(Deserialize)]
pub struct AssignRoleRequest { pub user_id: String, pub role: String }

// Response that frontend expects
#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
}

impl From<&User> for UserResponse {
    fn from(user: &User) -> Self {
        Self {
            id: user.id.clone(),
            email: user.email.clone(),
            name: user.name.clone(),
            role: user.role.clone(),
        }
    }
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub token: String,
    pub user: UserResponse,
}

pub async fn register(State(state): State<Arc<AppState>>, Json(req): Json<RegisterRequest>) -> Result<(StatusCode, Json<RegisterResponse>)> {
    let user = state.auth_service.create_user(&req.email, &req.name, &req.password).await?;
    state.rbac_service.assign_role(&user.id, "user").await;
    
    // Auto-login after registration
    if let Some(session) = state.auth_service.authenticate(&req.email, &req.password).await {
        Ok((StatusCode::CREATED, Json(RegisterResponse {
            token: session.token,
            user: UserResponse::from(&user),
        })))
    } else {
        Ok((StatusCode::CREATED, Json(RegisterResponse {
            token: String::new(),
            user: UserResponse::from(&user),
        })))
    }
}

pub async fn login(State(state): State<Arc<AppState>>, Json(req): Json<LoginRequest>) -> Result<Json<LoginResponse>> {
    match state.auth_service.authenticate(&req.email, &req.password).await {
        Some(session) => {
            state.audit_service.log_login(&session.user_id, None, true).await;
            
            // Get user info
            if let Some(user) = state.auth_service.validate_session(&session.token).await {
                Ok(Json(LoginResponse {
                    token: session.token,
                    user: UserResponse::from(&user),
                }))
            } else {
                Ok(Json(LoginResponse {
                    token: session.token,
                    user: UserResponse {
                        id: session.user_id,
                        email: req.email,
                        name: "User".into(),
                        role: "user".into(),
                    },
                }))
            }
        }
        None => Err(crate::error::SynapseError::Validation("Invalid credentials".into())),
    }
}

pub async fn logout(State(state): State<Arc<AppState>>, Path(token): Path<String>) -> StatusCode {
    state.auth_service.logout(&token).await;
    StatusCode::NO_CONTENT
}

pub async fn list_roles(State(state): State<Arc<AppState>>) -> Json<Vec<Role>> {
    Json(state.rbac_service.list_roles().await)
}

pub async fn assign_role(State(state): State<Arc<AppState>>, Json(req): Json<AssignRoleRequest>) -> StatusCode {
    if state.rbac_service.assign_role(&req.user_id, &req.role).await {
        StatusCode::OK
    } else {
        StatusCode::BAD_REQUEST
    }
}

pub async fn get_audit_logs(State(state): State<Arc<AppState>>, Query(query): Query<AuditQueryParams>) -> Json<Vec<AuditEvent>> {
    // Try to get from database first
    if let Some(ref pool) = state.db_pool {
        let limit = query.limit.unwrap_or(100) as i64;
        if let Ok(rows) = sqlx::query_as::<_, (String, Option<String>, String, Option<String>, Option<String>, serde_json::Value, chrono::DateTime<chrono::Utc>)>(
            "SELECT id, user_id, action, resource_type, resource_id, details, created_at FROM audit_logs ORDER BY created_at DESC LIMIT $1"
        )
        .bind(limit)
        .fetch_all(pool)
        .await {
            let events: Vec<AuditEvent> = rows.into_iter().map(|(id, user_id, action, resource_type, resource_id, details, created_at)| {
                AuditEvent {
                    id,
                    timestamp: created_at,
                    user_id: user_id.unwrap_or_default(),
                    action,
                    resource_type: resource_type.unwrap_or_else(|| "unknown".to_string()),
                    resource_id,
                    details,
                    ip_address: None,
                    user_agent: None,
                    success: true,
                    error: None,
                }
            }).collect();
            return Json(events);
        }
    }
    
    // Fallback to in-memory
    let q = AuditQuery {
        user_id: query.user_id,
        action: query.action,
        resource_type: None,
        from: None,
        to: None,
        success_only: query.success_only,
        limit: query.limit.unwrap_or(100),
    };
    Json(state.audit_service.query(q).await)
}

#[derive(Deserialize)]
pub struct AuditQueryParams {
    pub user_id: Option<String>,
    pub action: Option<String>,
    pub success_only: Option<bool>,
    pub limit: Option<usize>,
}
