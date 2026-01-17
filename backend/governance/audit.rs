//! Audit logging

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub user_id: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub details: serde_json::Value,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub success: bool,
    pub error: Option<String>,
}

impl AuditEvent {
    pub fn new(user_id: impl Into<String>, action: impl Into<String>, resource_type: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            user_id: user_id.into(),
            action: action.into(),
            resource_type: resource_type.into(),
            resource_id: None,
            details: serde_json::Value::Null,
            ip_address: None,
            user_agent: None,
            success: true,
            error: None,
        }
    }

    pub fn with_resource(mut self, id: impl Into<String>) -> Self {
        self.resource_id = Some(id.into());
        self
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = details;
        self
    }

    pub fn with_ip(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    pub fn failed(mut self, error: impl Into<String>) -> Self {
        self.success = false;
        self.error = Some(error.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditQuery {
    pub user_id: Option<String>,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub success_only: Option<bool>,
    pub limit: usize,
}

impl AuditQuery {
    pub fn new() -> Self {
        Self {
            user_id: None,
            action: None,
            resource_type: None,
            from: None,
            to: None,
            success_only: None,
            limit: 100,
        }
    }

    pub fn for_user(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }
}

impl Default for AuditQuery {
    fn default() -> Self { Self::new() }
}

pub struct AuditService {
    events: Arc<RwLock<Vec<AuditEvent>>>,
}

impl AuditService {
    pub fn new() -> Self {
        Self { events: Arc::new(RwLock::new(Vec::new())) }
    }

    pub async fn log(&self, event: AuditEvent) {
        self.events.write().await.push(event);
    }

    pub async fn query(&self, query: AuditQuery) -> Vec<AuditEvent> {
        let events = self.events.read().await;
        
        events.iter()
            .filter(|e| {
                if let Some(ref user_id) = query.user_id {
                    if &e.user_id != user_id { return false; }
                }
                if let Some(ref action) = query.action {
                    if &e.action != action { return false; }
                }
                if let Some(ref rt) = query.resource_type {
                    if &e.resource_type != rt { return false; }
                }
                if let Some(from) = query.from {
                    if e.timestamp < from { return false; }
                }
                if let Some(to) = query.to {
                    if e.timestamp > to { return false; }
                }
                if let Some(success_only) = query.success_only {
                    if success_only && !e.success { return false; }
                }
                true
            })
            .rev()
            .take(query.limit)
            .cloned()
            .collect()
    }

    pub async fn count(&self) -> usize {
        self.events.read().await.len()
    }

    pub async fn export(&self, from: DateTime<Utc>, to: DateTime<Utc>) -> Vec<AuditEvent> {
        let events = self.events.read().await;
        events.iter()
            .filter(|e| e.timestamp >= from && e.timestamp <= to)
            .cloned()
            .collect()
    }
}

impl Default for AuditService {
    fn default() -> Self { Self::new() }
}

// Convenience logging functions
impl AuditService {
    pub async fn log_login(&self, user_id: &str, ip: Option<&str>, success: bool) {
        let mut event = AuditEvent::new(user_id, "login", "session");
        if let Some(ip) = ip { event = event.with_ip(ip); }
        if !success { event = event.failed("authentication_failed"); }
        self.log(event).await;
    }

    pub async fn log_api_call(&self, user_id: &str, endpoint: &str, method: &str) {
        let event = AuditEvent::new(user_id, format!("{}:{}", method, endpoint), "api")
            .with_details(serde_json::json!({"method": method, "endpoint": endpoint}));
        self.log(event).await;
    }

    pub async fn log_workflow_execution(&self, user_id: &str, workflow_id: &str, execution_id: &str) {
        let event = AuditEvent::new(user_id, "execute", "workflow")
            .with_resource(workflow_id)
            .with_details(serde_json::json!({"execution_id": execution_id}));
        self.log(event).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audit_logging() {
        let audit = AuditService::new();
        
        audit.log_login("user1", Some("192.168.1.1"), true).await;
        audit.log_api_call("user1", "/v1/chat", "POST").await;
        
        assert_eq!(audit.count().await, 2);
    }

    #[tokio::test]
    async fn test_audit_query() {
        let audit = AuditService::new();
        
        audit.log_login("user1", None, true).await;
        audit.log_login("user2", None, true).await;
        
        let results = audit.query(AuditQuery::new().for_user("user1")).await;
        assert_eq!(results.len(), 1);
    }
}
