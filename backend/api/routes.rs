//! API routes configuration

use axum::{middleware, routing::{get, post, delete, put}, Router};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tower_http::compression::CompressionLayer;

use super::{handlers, state::AppState, middleware::logging_middleware};
use super::{workflow_handlers, knowledge_handlers, governance_handlers, agent_handlers, provider_handlers, admin_handlers, voice_handlers};

/// Create the API router with all routes
pub fn create_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);

    // API v1 routes
    let api_v1 = Router::new()
        // Chat completions
        .route("/chat/completions", post(handlers::chat_completions))
        // Old providers (legacy)
        .route("/providers", get(handlers::list_providers))
        .route("/providers", post(handlers::create_provider))
        .route("/providers/:id", delete(handlers::delete_provider))
        // Models
        .route("/models", get(handlers::list_models))
        // Costs
        .route("/costs", get(handlers::get_costs))
        .route("/costs/recent", get(handlers::get_recent_costs))
        .route("/costs/user/:user_id", get(handlers::get_user_costs))
        // Budgets
        .route("/budgets", post(handlers::set_budget))
        .route("/budgets/:entity_id", get(handlers::get_budget))
        // Status
        .route("/status", get(handlers::status))
        // Workflows (Phase 2)
        .route("/workflows", get(workflow_handlers::list_workflows))
        .route("/workflows", post(workflow_handlers::create_workflow))
        .route("/workflows/:id", get(workflow_handlers::get_workflow))
        .route("/workflows/:id", delete(workflow_handlers::delete_workflow))
        .route("/workflows/:id/execute", post(workflow_handlers::execute_workflow))
        .route("/executions", get(workflow_handlers::list_executions))
        .route("/executions/:id", get(workflow_handlers::get_execution))
        // Knowledge (Phase 3)
        .route("/knowledge/ingest", post(knowledge_handlers::ingest_document))
        .route("/knowledge/search", get(knowledge_handlers::search_knowledge))
        .route("/knowledge/rag", get(knowledge_handlers::get_rag_context))
        .route("/knowledge/documents/:id", delete(knowledge_handlers::delete_document))
        .route("/knowledge/stats", get(knowledge_handlers::knowledge_stats))
        // Governance (Phase 4)
        .route("/auth/register", post(governance_handlers::register))
        .route("/auth/login", post(governance_handlers::login))
        .route("/auth/logout/:token", post(governance_handlers::logout))
        .route("/roles", get(governance_handlers::list_roles))
        .route("/roles/assign", post(governance_handlers::assign_role))
        .route("/audit", get(governance_handlers::get_audit_logs))
        // Agents (Dynamic provider configuration)
        .route("/agents", get(agent_handlers::list_agents))
        .route("/agents", post(agent_handlers::create_agent))
        .route("/agents/providers", get(agent_handlers::list_provider_options))
        .route("/agents/:id", get(agent_handlers::get_agent))
        .route("/agents/:id", put(agent_handlers::update_agent))
        .route("/agents/:id", delete(agent_handlers::delete_agent))
        .route("/agents/:id/llm", put(agent_handlers::update_llm_config))
        .route("/agents/:id/embedding", put(agent_handlers::update_embedding_config))
        .route("/agents/:id/vectordb", put(agent_handlers::update_vectordb_config))
        .route("/agents/:id/chat", post(agent_handlers::chat_with_agent))
        .route("/agents/:id/knowledge/ingest", post(agent_handlers::ingest_to_agent))
        .route("/agents/:id/knowledge/search", get(agent_handlers::search_agent_knowledge))
        // Provider Account Management (NEW)
        .route("/provider-accounts/providers", get(provider_handlers::list_providers))
        .route("/provider-accounts/:provider_id/accounts", get(provider_handlers::get_provider_accounts))
        .route("/provider-accounts/:provider_id/usage", get(provider_handlers::get_provider_usage))
        .route("/provider-accounts/:provider_id/statuses", get(provider_handlers::get_account_statuses))
        .route("/provider-accounts/:provider_id/available", get(provider_handlers::get_available_account))
        .route("/provider-accounts/:provider_id/:account_id/default", put(provider_handlers::set_default_account))
        .route("/provider-accounts/accounts", post(provider_handlers::create_account))
        .route("/provider-accounts/accounts/:account_id", put(provider_handlers::update_account))
        .route("/provider-accounts/accounts/:account_id", delete(provider_handlers::delete_account))
        .route("/provider-accounts/accounts/:account_id/usage", post(provider_handlers::record_usage))
        // Admin: User Management
        .route("/admin/users", get(admin_handlers::list_users))
        .route("/admin/users", post(admin_handlers::create_user))
        .route("/admin/users/stats", get(admin_handlers::get_user_stats))
        .route("/admin/users/:user_id", get(admin_handlers::get_user))
        .route("/admin/users/:user_id", put(admin_handlers::update_user))
        .route("/admin/users/:user_id", delete(admin_handlers::delete_user))
        .route("/admin/users/:user_id/api-keys", get(admin_handlers::list_user_api_keys))
        .route("/admin/users/:user_id/api-keys", post(admin_handlers::create_api_key))
        .route("/admin/api-keys/:key_id", delete(admin_handlers::delete_api_key))
        // Admin: System Health
        .route("/admin/health", get(admin_handlers::get_system_health))
        // Admin: Roles & Permissions
        .route("/admin/roles/definitions", get(admin_handlers::list_role_definitions))
        .route("/admin/permissions", get(admin_handlers::list_permissions))
        // Admin: Applications (API Keys for external services)
        .route("/admin/applications", get(admin_handlers::list_applications))
        .route("/admin/applications", post(admin_handlers::create_application))
        .route("/admin/applications/scopes", get(admin_handlers::list_api_scopes))
        .route("/admin/applications/:app_id", put(admin_handlers::update_application))
        .route("/admin/applications/:app_id", delete(admin_handlers::delete_application))
        .route("/admin/applications/:app_id/rotate", post(admin_handlers::rotate_application_key))
        // Voice: TTS (Text-to-Speech)
        .route("/voice/tts/providers", get(voice_handlers::list_tts_providers))
        .route("/voice/tts/providers/:provider_id/voices", get(voice_handlers::list_tts_voices))
        .route("/voice/tts/synthesize", post(voice_handlers::synthesize_speech))
        // Voice: STT (Speech-to-Text)
        .route("/voice/stt/providers", get(voice_handlers::list_stt_providers))
        .route("/voice/stt/providers/:provider_id/models", get(voice_handlers::list_stt_models))
        .route("/voice/stt/transcribe", post(voice_handlers::transcribe_audio))
        // Voice: Languages
        .route("/voice/languages", get(voice_handlers::get_supported_languages))
        // Voice: WebRTC
        .route("/voice/webrtc/sessions", post(voice_handlers::create_webrtc_session))
        .route("/voice/webrtc/sessions/:session_id", get(voice_handlers::get_webrtc_session));

    Router::new()
        .route("/health", get(handlers::health_check))
        .nest("/v1", api_v1)
        .fallback(handlers::not_found)
        .layer(middleware::from_fn(logging_middleware))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(cors)
        .with_state(state)
}
