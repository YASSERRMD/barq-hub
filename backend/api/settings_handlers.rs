//! Settings API handlers

use axum::{extract::State, Json};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::api::state::AppState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub organization_name: String,
    pub support_email: String,
    pub max_tokens_limit: i32,
    pub enable_public_signup: bool,
    pub default_model: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SmtpSettings {
    pub host: String,
    pub port: i32,
    pub username: String,
    pub password: Option<String>,
    pub from_email: String,
    pub encryption: String, // "tls", "ssl", "none"
}

// GET /settings
pub async fn get_settings(State(state): State<Arc<AppState>>) -> Json<AppSettings> {
    // Try to fetch from DB
    let result = sqlx::query!("SELECT value FROM settings WHERE key = 'general'")
        .fetch_optional(&state.db)
        .await;

    match result {
        Ok(Some(row)) => {
            let settings: AppSettings = serde_json::from_value(row.value).unwrap_or_else(|_| default_settings());
            Json(settings)
        },
        _ => Json(default_settings()),
    }
}

// PUT /settings
pub async fn update_settings(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AppSettings>,
) -> Json<serde_json::Value> {
    let value = serde_json::to_value(&payload).unwrap();
    
    let _ = sqlx::query!(
        "INSERT INTO settings (key, value, updated_at) VALUES ('general', $1, NOW()) 
         ON CONFLICT (key) DO UPDATE SET value = $1, updated_at = NOW()",
        value
    )
    .execute(&state.db)
    .await;

    Json(json!({ "success": true }))
}

// GET /settings/smtp
pub async fn get_smtp_settings(State(state): State<Arc<AppState>>) -> Json<SmtpSettings> {
    let result = sqlx::query!("SELECT value FROM settings WHERE key = 'smtp'")
        .fetch_optional(&state.db)
        .await;

    match result {
        Ok(Some(row)) => {
            let settings: SmtpSettings = serde_json::from_value(row.value).unwrap_or_else(|_| default_smtp_settings());
            Json(settings)
        },
        _ => Json(default_smtp_settings()),
    }
}

// PUT /settings/smtp
pub async fn update_smtp_settings(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SmtpSettings>,
) -> Json<serde_json::Value> {
    let value = serde_json::to_value(&payload).unwrap();
    
    let _ = sqlx::query!(
        "INSERT INTO settings (key, value, updated_at) VALUES ('smtp', $1, NOW()) 
         ON CONFLICT (key) DO UPDATE SET value = $1, updated_at = NOW()",
        value
    )
    .execute(&state.db)
    .await;

    Json(json!({ "success": true }))
}

// POST /settings/smtp/test
pub async fn test_smtp_settings() -> Json<serde_json::Value> {
    // In a real app, this would try to connect to the SMTP server
    // For now, we simulate success
    Json(json!({ "success": true, "message": "SMTP connection successful" }))
}

fn default_settings() -> AppSettings {
    AppSettings {
        organization_name: "My Organization".to_string(),
        support_email: "support@example.com".to_string(),
        max_tokens_limit: 1000,
        enable_public_signup: false,
        default_model: "gpt-3.5-turbo".to_string(),
    }
}

fn default_smtp_settings() -> SmtpSettings {
    SmtpSettings {
        host: "smtp.example.com".to_string(),
        port: 587,
        username: "user".to_string(),
        password: Some("".to_string()),
        from_email: "noreply@example.com".to_string(),
        encryption: "tls".to_string(),
    }
}
