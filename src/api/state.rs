//! Application state with database connection

use std::sync::Arc;
use tokio::sync::RwLock;
use crate::{Provider, router::SmartRouter, cost::CostManager};
use crate::workflow::{StateStore, ToolRegistry};
use crate::knowledge::{RAGEngine, VectorStore, embeddings::MockEmbedding};
use crate::governance::{AuthService, RBACService, AuditService};
use crate::agents::{AgentStore, registry::ProviderRegistry, runtime::AgentRuntime};
use crate::providers::ProviderAccountManager;
use crate::db::DbPool;
use crate::db::ApplicationRepository;

/// Shared application state
pub struct AppState {
    pub router: Arc<SmartRouter>,
    pub cost_manager: Arc<CostManager>,
    pub providers: Arc<RwLock<Vec<Provider>>>,
    pub start_time: std::time::Instant,
    pub version: String,
    // Phase 2: Workflow
    pub workflow_store: Arc<StateStore>,
    pub tool_registry: Arc<ToolRegistry>,
    // Phase 3: Knowledge
    pub rag_engine: Arc<RAGEngine>,
    // Phase 4: Governance
    pub auth_service: Arc<AuthService>,
    pub rbac_service: Arc<RBACService>,
    pub audit_service: Arc<AuditService>,
    // Agents with dynamic providers
    pub agent_store: Arc<AgentStore>,
    pub provider_registry: Arc<ProviderRegistry>,
    pub agent_runtime: Arc<AgentRuntime>,
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
        
        let router = Arc::new(SmartRouter::new(providers.clone()));
        let workflow_store = Arc::new(StateStore::new());
        let tool_registry = Arc::new(ToolRegistry::new());
        
        // Create RAG engine with mock embeddings
        let embedding_provider = Arc::new(MockEmbedding::new(128));
        let vector_store = Arc::new(VectorStore::new());
        let rag_engine = Arc::new(RAGEngine::new(embedding_provider, vector_store));
        
        // Governance services with database
        let auth_service = Arc::new(AuthService::with_pool(pool.clone()));
        let rbac_service = Arc::new(RBACService::new());
        let audit_service = Arc::new(AuditService::new());
        
        // Agent management
        let agent_store = Arc::new(AgentStore::new());
        let provider_registry = Arc::new(ProviderRegistry::new());
        let agent_runtime = Arc::new(AgentRuntime::new(agent_store.clone(), provider_registry.clone()));
        
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
            providers: Arc::new(RwLock::new(providers)),
            start_time: std::time::Instant::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            workflow_store,
            tool_registry,
            rag_engine,
            auth_service,
            rbac_service,
            audit_service,
            agent_store,
            provider_registry,
            agent_runtime,
            account_manager,
            db_pool: Some(pool),
            application_repo: Some(application_repo),
        })
    }
    
    /// Create new state without database (in-memory only)
    pub fn new(providers: Vec<Provider>) -> Self {
        let router = Arc::new(SmartRouter::new(providers.clone()));
        let workflow_store = Arc::new(StateStore::new());
        let tool_registry = Arc::new(ToolRegistry::new());
        
        // Create RAG engine with mock embeddings
        let embedding_model = Arc::new(MockEmbedding::new(128));
        let vector_store = Arc::new(VectorStore::new());
        let rag_engine = Arc::new(RAGEngine::new(embedding_model, vector_store));
        
        // Services
        let auth_service = Arc::new(AuthService::new());
        let rbac_service = Arc::new(RBACService::new());
        let audit_service = Arc::new(AuditService::new());
        
        // Agent runtime
        let agent_store = Arc::new(AgentStore::new());
        let provider_registry = Arc::new(ProviderRegistry::new());
        // Populate registry with initial providers - skipping for now to avoid compilation error if register missing
        
        let agent_runtime = Arc::new(AgentRuntime::new(
            agent_store.clone(),
            provider_registry.clone(),
        ));
        
        let account_manager = ProviderAccountManager::new();
        
        Self {
            router,
            cost_manager: Arc::new(CostManager::new()),
            providers: Arc::new(RwLock::new(providers)),
            start_time: std::time::Instant::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            workflow_store,
            tool_registry,
            rag_engine,
            auth_service,
            rbac_service,
            audit_service,
            agent_store,
            provider_registry,
            agent_runtime,
            account_manager: Arc::new(account_manager),
            db_pool: None,
            application_repo: None,
        }
    }

    pub async fn reload_providers(&mut self, providers: Vec<Provider>) {
        *self.providers.write().await = providers.clone();
        self.router = Arc::new(SmartRouter::new(providers));
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
            api_key: row.api_key_encrypted.clone(), // In production, this would be decrypted
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
