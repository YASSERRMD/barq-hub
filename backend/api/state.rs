//! Application state with database connection

use std::sync::Arc;
use tokio::sync::RwLock;
use crate::{Provider, router::SmartRouter, cost::CostManager};

use crate::governance::{AuthService, RBACService, AuditService};
use crate::providers::ProviderAccountManager;
use crate::db::DbPool;
use crate::db::ApplicationRepository;

/// Shared application state
pub struct AppState {
    pub router: Arc<SmartRouter>,
    pub cost_manager: Arc<CostManager>,
    pub providers: Arc<RwLock<Vec<Provider>>>,
    pub http_client: reqwest::Client,
    pub start_time: std::time::Instant,
    pub version: String,
    // Phase 4: Governance
    pub auth_service: Arc<AuthService>,
    pub rbac_service: Arc<RBACService>,
    pub audit_service: Arc<AuditService>,
    // Provider account management
    pub account_manager: Arc<ProviderAccountManager>,
    // Database pool
    pub db_pool: Option<DbPool>,
    // Repositories
    pub application_repo: Option<Arc<ApplicationRepository>>,
}

impl AppState {
    /// Create new state with database connection
    pub async fn with_database(providers: Vec<Provider>, database_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize database pool
        let pool = crate::db::pool::init_pool(database_url).await?;
        tracing::info!("Database connection established");
        
        let application_repo = Arc::new(ApplicationRepository::new(pool.clone()));
        
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .expect("Failed to create HTTP client");
            
        let router = Arc::new(SmartRouter::new(providers.clone(), http_client.clone()));
        // Governance services with database
        let auth_service = Arc::new(AuthService::with_pool(pool.clone()));
        let rbac_service = Arc::new(RBACService::new());
        let audit_service = Arc::new(AuditService::new());
        
        // Provider account manager - load from database
        let account_manager = Arc::new(ProviderAccountManager::new());
        
        // Load accounts from database
        if let Ok(repo_accounts) = load_accounts_from_db(&pool).await {
            for account in repo_accounts {
                if let Err(e) = account_manager.add_account(account).await {
                    tracing::warn!("Failed to load account: {}", e);
                }
            }
            tracing::info!("Loaded provider accounts from database");
        }
        
        Ok(Self {
            router,
            cost_manager: Arc::new(CostManager::new()),
            providers: Arc::new(RwLock::new(providers.clone())),
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .expect("Failed to create HTTP client"),
            start_time: std::time::Instant::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            auth_service,
            rbac_service,
            audit_service,
            account_manager,
            db_pool: Some(pool),
            application_repo: Some(application_repo),
        })
    }
    
    /// Create new state without database (in-memory only)
    pub fn new(providers: Vec<Provider>) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .expect("Failed to create HTTP client");

        let router = Arc::new(SmartRouter::new(providers.clone(), http_client.clone()));
        
        // Services
        let auth_service = Arc::new(AuthService::new());
        let rbac_service = Arc::new(RBACService::new());
        let audit_service = Arc::new(AuditService::new());
        
        let account_manager = ProviderAccountManager::new();
        
        Self {
            router,
            cost_manager: Arc::new(CostManager::new()),
            providers: Arc::new(RwLock::new(providers.clone())),
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .expect("Failed to create HTTP client"),
            start_time: std::time::Instant::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            auth_service,
            rbac_service,
            audit_service,
            account_manager: Arc::new(account_manager),
            db_pool: None,
            application_repo: None,
        }
    }

    pub async fn reload_providers(&mut self, providers: Vec<Provider>) {
        *self.providers.write().await = providers.clone();
        self.router = Arc::new(SmartRouter::new(providers, self.http_client.clone()));
    }

    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    pub fn is_database_connected(&self) -> bool {
        self.db_pool.is_some()
    }
}

/// Load provider accounts from database
async fn load_accounts_from_db(pool: &crate::db::DbPool) -> Result<Vec<crate::providers::ProviderAccount>, Box<dyn std::error::Error + Send + Sync>> {
    use crate::providers::account_manager::{ProviderAccount, AccountConfig, ApiKeyConfig};
    use crate::db::ProviderAccountRepository;
    
    let repo = ProviderAccountRepository::new(pool.clone());
    let rows = repo.list_all().await?;
    
    let mut accounts = Vec::new();
    for row in rows {
        // Parse models from JSON
        let models: Vec<crate::types::ProviderModel> = serde_json::from_value(row.models.clone()).unwrap_or_default();
        
        // Create account with basic ApiKey config (API key is encrypted in DB)
        let config = AccountConfig::ApiKey(ApiKeyConfig {
            api_key: row.api_key_encrypted.clone().unwrap_or_default(), // In production, this would be decrypted
            organization_id: None,
            custom_endpoint: row.endpoint.clone(),
        });
        
        let mut account = ProviderAccount::new(
            row.name.clone(),
            row.provider_id.clone(),
            config,
        );
        
        // Override the auto-generated ID with the one from DB
        account.id = row.id.clone();
        account.enabled = row.enabled;
        account.is_default = row.is_default;
        account.priority = row.priority;
        account.models = models;
        account.created_at = row.created_at;
        account.updated_at = row.updated_at;

        // Restore quotas
        if let Ok(quotas) = serde_json::from_value(row.quota_config.clone()) {
            account.quotas = quotas;
        }
        
        accounts.push(account);
    }
    
    Ok(accounts)
}
