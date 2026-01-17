//! Authentication with database persistence

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use std::sync::Arc;
use crate::db::{DbPool, UserRepository, SessionRepository};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

impl User {
    pub fn new(email: impl Into<String>, name: impl Into<String>, password_hash: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            email: email.into(),
            name: name.into(),
            password_hash: password_hash.into(),
            role: "user".into(),
            enabled: true,
            created_at: Utc::now(),
            last_login: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub token: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl Session {
    pub fn new(user_id: impl Into<String>, ttl_hours: i64) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.into(),
            token: Uuid::new_v4().to_string(),
            created_at: now,
            expires_at: now + Duration::hours(ttl_hours),
        }
    }

    pub fn is_valid(&self) -> bool {
        Utc::now() < self.expires_at
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub key_hash: String,
    pub prefix: String,
    pub scopes: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub enabled: bool,
}

/// AuthService with database persistence
pub struct AuthService {
    user_repo: Option<UserRepository>,
    session_repo: Option<SessionRepository>,
    pool: Option<DbPool>,
    // Fallback in-memory (for when DB is not available)
    fallback_users: Arc<tokio::sync::RwLock<std::collections::HashMap<String, User>>>,
    fallback_sessions: Arc<tokio::sync::RwLock<std::collections::HashMap<String, Session>>>,
}

impl AuthService {
    /// Create with database pool
    pub fn with_pool(pool: DbPool) -> Self {
        let user_repo = UserRepository::new(pool.clone());
        let session_repo = SessionRepository::new(pool.clone());
        
        Self {
            user_repo: Some(user_repo),
            session_repo: Some(session_repo),
            pool: Some(pool),
            fallback_users: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            fallback_sessions: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Create without database (in-memory fallback)
    pub fn new() -> Self {
        let mut users = std::collections::HashMap::new();
        
        // Seed default admin user
        let admin = User {
            id: "admin-001".to_string(),
            email: "admin@synapse.local".to_string(),
            name: "Admin".to_string(),
            password_hash: Self::hash_password_static("admin123"),
            role: "admin".to_string(),
            enabled: true,
            created_at: Utc::now(),
            last_login: None,
        };
        users.insert(admin.id.clone(), admin);

        // Test user
        let test_user = User {
            id: "user-001".to_string(),
            email: "user@synapse.local".to_string(),
            name: "Test User".to_string(),
            password_hash: Self::hash_password_static("user123"),
            role: "user".to_string(),
            enabled: true,
            created_at: Utc::now(),
            last_login: None,
        };
        users.insert(test_user.id.clone(), test_user);

        Self {
            user_repo: None,
            session_repo: None,
            pool: None,
            fallback_users: Arc::new(tokio::sync::RwLock::new(users)),
            fallback_sessions: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    fn hash_password_static(password: &str) -> String {
        use md5::{Md5, Digest};
        let mut hasher = Md5::new();
        hasher.update(password.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn hash_password(password: &str) -> String {
        Self::hash_password_static(password)
    }

    fn verify_password(password: &str, hash: &str) -> bool {
        Self::hash_password(password) == hash
    }

    pub async fn create_user(&self, email: &str, name: &str, password: &str) -> crate::error::Result<User> {
        let password_hash = Self::hash_password(password);
        let id = Uuid::new_v4().to_string();

        if let Some(ref repo) = self.user_repo {
            let row = repo.create(&id, email, name, &password_hash, "user").await
                .map_err(|e| crate::error::SynapseError::DatabaseError(e.to_string()))?;
            
            Ok(User {
                id: row.id,
                email: row.email,
                name: row.name,
                password_hash: row.password_hash,
                role: row.role,
                enabled: row.enabled,
                created_at: row.created_at,
                last_login: row.last_login,
            })
        } else {
            // Fallback to in-memory
            let user = User {
                id,
                email: email.to_string(),
                name: name.to_string(),
                password_hash,
                role: "user".to_string(),
                enabled: true,
                created_at: Utc::now(),
                last_login: None,
            };
            self.fallback_users.write().await.insert(user.id.clone(), user.clone());
            Ok(user)
        }
    }

    pub async fn authenticate(&self, email: &str, password: &str) -> Option<Session> {
        let user = if let Some(ref repo) = self.user_repo {
            repo.find_by_email(email).await.ok().flatten().map(|row| User {
                id: row.id,
                email: row.email,
                name: row.name,
                password_hash: row.password_hash,
                role: row.role,
                enabled: row.enabled,
                created_at: row.created_at,
                last_login: row.last_login,
            })
        } else {
            let users = self.fallback_users.read().await;
            users.values().find(|u| u.email == email).cloned()
        };

        let user = user?;
        if !user.enabled || !Self::verify_password(password, &user.password_hash) {
            return None;
        }

        let session = Session::new(&user.id, 24);

        if let Some(ref repo) = self.session_repo {
            if let Ok(_) = repo.create(&session.id, &session.token, &session.user_id, session.expires_at).await {
                // Update last login
                if let Some(ref user_repo) = self.user_repo {
                    let _ = user_repo.update_last_login(&user.id).await;
                }
            }
        } else {
            self.fallback_sessions.write().await.insert(session.token.clone(), session.clone());
        }

        Some(session)
    }

    pub async fn validate_session(&self, token: &str) -> Option<User> {
        let session = if let Some(ref repo) = self.session_repo {
            repo.find_by_token(token).await.ok().flatten().map(|row| Session {
                id: row.id,
                user_id: row.user_id,
                token: row.token,
                created_at: row.created_at,
                expires_at: row.expires_at,
            })
        } else {
            self.fallback_sessions.read().await.get(token).cloned()
        };

        let session = session?;
        if !session.is_valid() {
            return None;
        }

        self.get_user(&session.user_id).await
    }

    pub async fn get_user(&self, user_id: &str) -> Option<User> {
        if let Some(ref repo) = self.user_repo {
            repo.find_by_id(user_id).await.ok().flatten().map(|row| User {
                id: row.id,
                email: row.email,
                name: row.name,
                password_hash: row.password_hash,
                role: row.role,
                enabled: row.enabled,
                created_at: row.created_at,
                last_login: row.last_login,
            })
        } else {
            self.fallback_users.read().await.get(user_id).cloned()
        }
    }

    pub async fn list_users(&self) -> Vec<User> {
        if let Some(ref repo) = self.user_repo {
            repo.list_all().await.ok().unwrap_or_default().into_iter().map(|row| User {
                id: row.id,
                email: row.email,
                name: row.name,
                password_hash: row.password_hash,
                role: row.role,
                enabled: row.enabled,
                created_at: row.created_at,
                last_login: row.last_login,
            }).collect()
        } else {
            self.fallback_users.read().await.values().cloned().collect()
        }
    }

    pub async fn update_user(&self, user_id: &str, update: UserUpdate) -> Option<User> {
        if let Some(ref repo) = self.user_repo {
            let result = repo.update(
                user_id,
                update.name.as_deref(),
                update.email.as_deref(),
                update.role.as_deref(),
                update.enabled,
            ).await.ok().flatten();

            if let Some(ref password) = update.password {
                let _ = repo.update_password(user_id, &Self::hash_password(password)).await;
            }

            result.map(|row| User {
                id: row.id,
                email: row.email,
                name: row.name,
                password_hash: row.password_hash,
                role: row.role,
                enabled: row.enabled,
                created_at: row.created_at,
                last_login: row.last_login,
            })
        } else {
            let mut users = self.fallback_users.write().await;
            if let Some(user) = users.get_mut(user_id) {
                if let Some(name) = update.name { user.name = name; }
                if let Some(email) = update.email { user.email = email; }
                if let Some(role) = update.role { user.role = role; }
                if let Some(enabled) = update.enabled { user.enabled = enabled; }
                if let Some(password) = update.password { user.password_hash = Self::hash_password(&password); }
                Some(user.clone())
            } else {
                None
            }
        }
    }

    pub async fn delete_user(&self, user_id: &str) -> bool {
        if let Some(ref repo) = self.user_repo {
            repo.delete(user_id).await.ok().unwrap_or(false)
        } else {
            self.fallback_users.write().await.remove(user_id).is_some()
        }
    }

    pub async fn logout(&self, token: &str) {
        if let Some(ref repo) = self.session_repo {
            let _ = repo.delete_by_token(token).await;
        } else {
            self.fallback_sessions.write().await.remove(token);
        }
    }

    // API Key stubs (would need full implementation)
    pub async fn list_api_keys(&self, _user_id: &str) -> Vec<ApiKey> {
        Vec::new() // TODO: Implement with database
    }

    pub async fn create_api_key(&self, user_id: &str, name: &str, scopes: Vec<String>, expires_in_days: Option<i64>) -> (String, ApiKey) {
        let raw_key = format!("sk-{}", Uuid::new_v4().to_string().replace("-", ""));
        let prefix = format!("sk-...{}", &raw_key[raw_key.len()-4..]);
        
        let api_key = ApiKey {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            name: name.to_string(),
            key_hash: Self::hash_password(&raw_key),
            prefix,
            scopes,
            created_at: Utc::now(),
            last_used: None,
            expires_at: expires_in_days.map(|d| Utc::now() + Duration::days(d)),
            enabled: true,
        };
        
        (raw_key, api_key)
    }

    pub async fn delete_api_key(&self, _key_id: &str) -> bool {
        false // TODO: Implement with database
    }

    pub async fn validate_api_key(&self, _key: &str) -> Option<(ApiKey, User)> {
        None // TODO: Implement with database
    }
}

impl Default for AuthService {
    fn default() -> Self { Self::new() }
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserUpdate {
    pub name: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
    pub enabled: Option<bool>,
    pub password: Option<String>,
}
