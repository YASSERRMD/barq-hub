//! Role-Based Access Control

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub description: String,
    pub permissions: HashSet<String>,
}

impl Role {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self { name: name.into(), description: description.into(), permissions: HashSet::new() }
    }

    pub fn with_permission(mut self, permission: impl Into<String>) -> Self {
        self.permissions.insert(permission.into());
        self
    }

    pub fn with_permissions(mut self, perms: Vec<&str>) -> Self {
        for p in perms { self.permissions.insert(p.into()); }
        self
    }
}

pub struct RBACService {
    roles: Arc<RwLock<HashMap<String, Role>>>,
    user_roles: Arc<RwLock<HashMap<String, String>>>,
}

impl RBACService {
    pub fn new() -> Self {
        let service = Self {
            roles: Arc::new(RwLock::new(HashMap::new())),
            user_roles: Arc::new(RwLock::new(HashMap::new())),
        };
        
        // Initialize with tokio runtime
        service
    }

    pub async fn init_default_roles(&self) {
        let roles = vec![
            Role::new("admin", "Full access").with_permissions(vec!["*"]),
            Role::new("developer", "Development access").with_permissions(vec![
                "workflows:read", "workflows:write", "workflows:execute",
                "knowledge:read", "knowledge:write",
                "chat:use", "providers:read",
            ]),
            Role::new("analyst", "Read-only access").with_permissions(vec![
                "workflows:read", "knowledge:read", "chat:use", "costs:read",
            ]),
            Role::new("user", "Basic access").with_permissions(vec!["chat:use", "workflows:execute"]),
            Role::new("billing", "Billing access").with_permissions(vec!["costs:read", "costs:write", "budgets:read", "budgets:write"]),
            Role::new("auditor", "Audit access").with_permissions(vec!["audit:read", "costs:read"]),
        ];

        let mut store = self.roles.write().await;
        for role in roles {
            store.insert(role.name.clone(), role);
        }
    }

    pub async fn create_role(&self, role: Role) {
        self.roles.write().await.insert(role.name.clone(), role);
    }

    pub async fn get_role(&self, name: &str) -> Option<Role> {
        self.roles.read().await.get(name).cloned()
    }

    pub async fn assign_role(&self, user_id: &str, role_name: &str) -> bool {
        if self.roles.read().await.contains_key(role_name) {
            self.user_roles.write().await.insert(user_id.into(), role_name.into());
            true
        } else {
            false
        }
    }

    pub async fn get_user_role(&self, user_id: &str) -> Option<Role> {
        let role_name = self.user_roles.read().await.get(user_id)?.clone();
        self.get_role(&role_name).await
    }

    pub async fn check_permission(&self, user_id: &str, permission: &str) -> bool {
        let role = match self.get_user_role(user_id).await {
            Some(r) => r,
            None => return false,
        };

        // Admin has all permissions
        if role.permissions.contains("*") {
            return true;
        }

        // Check exact permission
        if role.permissions.contains(permission) {
            return true;
        }

        // Check wildcard (e.g., "workflows:*")
        let parts: Vec<&str> = permission.split(':').collect();
        if parts.len() >= 2 {
            let wildcard = format!("{}:*", parts[0]);
            if role.permissions.contains(&wildcard) {
                return true;
            }
        }

        false
    }

    pub async fn list_roles(&self) -> Vec<Role> {
        self.roles.read().await.values().cloned().collect()
    }
}

impl Default for RBACService {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_role_assignment() {
        let rbac = RBACService::new();
        rbac.init_default_roles().await;
        
        rbac.assign_role("user1", "developer").await;
        
        assert!(rbac.check_permission("user1", "workflows:read").await);
        assert!(rbac.check_permission("user1", "chat:use").await);
        assert!(!rbac.check_permission("user1", "audit:read").await);
    }

    #[tokio::test]
    async fn test_admin_all_permissions() {
        let rbac = RBACService::new();
        rbac.init_default_roles().await;
        
        rbac.assign_role("admin1", "admin").await;
        
        assert!(rbac.check_permission("admin1", "anything:here").await);
        assert!(rbac.check_permission("admin1", "workflows:delete").await);
    }
}
