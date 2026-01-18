//! API routes configuration

use axum::{middleware, routing::{get, post, delete, put}, Router};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tower_http::compression::CompressionLayer;

use super::{handlers, state::AppState, middleware::logging_middleware};
use super::{governance_handlers, provider_handlers, admin_handlers, voice_handlers, settings_handlers};

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
        // Settings
        .route("/settings", get(settings_handlers::get_settings))
        .route("/settings", put(settings_handlers::update_settings))
        .route("/settings/smtp", get(settings_handlers::get_smtp_settings))
        .route("/settings/smtp", put(settings_handlers::update_smtp_settings))
        .route("/settings/smtp/test", post(settings_handlers::test_smtp_settings))
        // Status
        .route("/status", get(handlers::status))
        // Governance (Phase 4)
        .route("/auth/register", post(governance_handlers::register))
        .route("/auth/login", post(governance_handlers::login))
        .route("/auth/logout/:token", post(governance_handlers::logout))
        .route("/roles", get(governance_handlers::list_roles))
        .route("/roles/assign", post(governance_handlers::assign_role))
        .route("/audit", get(governance_handlers::get_audit_logs))
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
