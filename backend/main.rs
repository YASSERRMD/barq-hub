//! BARQ HUB - AI Management Console
//!
//! Main entry point for the server.

use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use barq_hub::{
    api::{create_router, AppState},
    config::Config,
    Provider, ProviderPricing, ProviderType, ProviderHealth,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "barq_hub=info,tower_http=info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting BARQ HUB v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = Config::load().expect("Failed to load configuration");
    tracing::info!("Configuration loaded");

    // Initialize providers from environment
    let providers = initialize_providers(&config);
    tracing::info!("Initialized {} providers", providers.len());

    // Create application state with database
    let state = match create_state_with_database(providers.clone(), &config).await {
        Ok(state) => {
            tracing::info!("Database connection established");
            Arc::new(state)
        }
        Err(e) => {
            tracing::warn!("Database connection failed: {}. Using in-memory storage.", e);
            tracing::warn!("Data will NOT be persisted across restarts!");
            Arc::new(AppState::new(providers))
        }
    };

    // Create router
    let app = create_router(state.clone());

    // Start HTTP server
    let http_addr = config.server_addr();
    tracing::info!("HTTP server listening on http://{}", http_addr);
    tracing::info!("Database connected: {}", state.is_database_connected());

    let http_listener = tokio::net::TcpListener::bind(&http_addr).await?;
    let http_server = async move {
        axum::serve(http_listener, app).await
    };

    // Start gRPC server if database is available
    let grpc_server = if let Some(ref app_repo) = state.application_repo {
        use tonic::transport::Server;
        use barq_hub::grpc::{
            barq::{chat_service_server::ChatServiceServer, models_service_server::ModelsServiceServer},
            ApiKeyInterceptor, ChatServiceImpl, ModelsServiceImpl,
        };

        let grpc_addr = "0.0.0.0:4002".parse().expect("Invalid gRPC address");
        let auth = ApiKeyInterceptor::new(app_repo.clone());
        
        let chat_service = ChatServiceImpl::new(state.clone(), auth.clone());
        let models_service = ModelsServiceImpl::new(state.clone(), auth.clone());

        tracing::info!("gRPC server listening on {}", grpc_addr);
        
        Some(Server::builder()
            .add_service(ChatServiceServer::new(chat_service))
            .add_service(ModelsServiceServer::new(models_service))
            .serve(grpc_addr))
    } else {
        tracing::warn!("gRPC server disabled: database not available");
        None
    };

    // Run both servers concurrently
    if let Some(grpc) = grpc_server {
        tokio::select! {
            result = http_server => {
                if let Err(e) = result {
                    tracing::error!("HTTP server error: {}", e);
                }
            }
            result = grpc => {
                if let Err(e) = result {
                    tracing::error!("gRPC server error: {}", e);
                }
            }
        }
    } else {
        http_server.await?;
    }

    Ok(())
}

/// Create state with database connection
async fn create_state_with_database(providers: Vec<Provider>, config: &Config) -> Result<AppState, Box<dyn std::error::Error>> {
    // Get database URL from environment
    let database_url = std::env::var("DATABASE_URL")
        .or_else(|_| std::env::var("POSTGRES_URL"))
        .unwrap_or_else(|_| {
            let host = std::env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".to_string());
            let port = std::env::var("POSTGRES_PORT").unwrap_or_else(|_| "5432".to_string());
            let user = std::env::var("POSTGRES_USER").unwrap_or_else(|_| "synapse".to_string());
            let pass = std::env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "synapse123".to_string());
            let db = std::env::var("POSTGRES_DB").unwrap_or_else(|_| "synapse".to_string());
            format!("postgres://{}:{}@{}:{}/{}", user, pass, host, port, db)
        });

    tracing::info!("Connecting to database...");
    AppState::with_database(providers, &database_url).await
}

/// Initialize providers from configuration
fn initialize_providers(config: &Config) -> Vec<Provider> {
    let mut providers = Vec::new();

    // OpenAI
    if let Some(ref api_key) = config.providers.openai_api_key {
        providers.push(Provider {
            id: "openai-default".to_string(),
            name: "OpenAI".to_string(),
            provider_type: ProviderType::OpenAI,
            api_key: api_key.clone(),
            base_url: "https://api.openai.com/v1".to_string(),
            pricing: ProviderPricing {
                input_token_cost: 30.0,   // GPT-4 pricing (per 1M tokens)
                output_token_cost: 60.0,
            },
            enabled: true,
            models: Vec::new(),
            health: ProviderHealth::default(),
            headers: std::collections::HashMap::new(),
        });
        tracing::info!("Configured OpenAI provider");
    }

    // Anthropic
    if let Some(ref api_key) = config.providers.anthropic_api_key {
        providers.push(Provider {
            id: "anthropic-default".to_string(),
            name: "Anthropic".to_string(),
            provider_type: ProviderType::Anthropic,
            api_key: api_key.clone(),
            base_url: "https://api.anthropic.com/v1".to_string(),
            pricing: ProviderPricing {
                input_token_cost: 15.0,   // Claude 3 Sonnet pricing
                output_token_cost: 75.0,
            },
            enabled: true,
            models: Vec::new(),
            health: ProviderHealth::default(),
            headers: std::collections::HashMap::new(),
        });
        tracing::info!("Configured Anthropic provider");
    }

    // Mistral
    if let Some(ref api_key) = config.providers.mistral_api_key {
        providers.push(Provider {
            id: "mistral-default".to_string(),
            name: "Mistral".to_string(),
            provider_type: ProviderType::Mistral,
            api_key: api_key.clone(),
            base_url: "https://api.mistral.ai/v1".to_string(),
            pricing: ProviderPricing {
                input_token_cost: 8.0,
                output_token_cost: 24.0,
            },
            enabled: true,
            models: Vec::new(),
            health: ProviderHealth::default(),
            headers: std::collections::HashMap::new(),
        });
        tracing::info!("Configured Mistral provider");
    }

    // Groq
    if let Some(ref api_key) = config.providers.groq_api_key {
        providers.push(Provider {
            id: "groq-default".to_string(),
            name: "Groq".to_string(),
            provider_type: ProviderType::Groq,
            api_key: api_key.clone(),
            base_url: "https://api.groq.com/openai/v1".to_string(),
            pricing: ProviderPricing {
                input_token_cost: 0.27,   // Llama pricing
                output_token_cost: 0.27,
            },
            enabled: true,
            models: Vec::new(),
            health: ProviderHealth::default(),
            headers: std::collections::HashMap::new(),
        });
        tracing::info!("Configured Groq provider");
    }

    // Together
    if let Some(ref api_key) = config.providers.together_api_key {
        providers.push(Provider {
            id: "together-default".to_string(),
            name: "Together".to_string(),
            provider_type: ProviderType::Together,
            api_key: api_key.clone(),
            base_url: "https://api.together.xyz/v1".to_string(),
            pricing: ProviderPricing {
                input_token_cost: 0.2,
                output_token_cost: 0.2,
            },
            enabled: true,
            models: Vec::new(),
            health: ProviderHealth::default(),
            headers: std::collections::HashMap::new(),
        });
        tracing::info!("Configured Together provider");
    }

    // Cohere
    if let Some(ref api_key) = config.providers.cohere_api_key {
        providers.push(Provider {
            id: "cohere-default".to_string(),
            name: "Cohere".to_string(),
            provider_type: ProviderType::Cohere,
            api_key: api_key.clone(),
            base_url: "https://api.cohere.com/v1".to_string(),
            pricing: ProviderPricing {
                input_token_cost: 1.0,
                output_token_cost: 2.0,
            },
            enabled: true,
            models: Vec::new(),
            health: ProviderHealth::default(),
            headers: std::collections::HashMap::new(),
        });
        tracing::info!("Configured Cohere provider");
    }

    // Local Ollama (always available for development)
    providers.push(Provider {
        id: "local-ollama".to_string(),
        name: "Local Ollama".to_string(),
        provider_type: ProviderType::Local,
        api_key: String::new(),
        base_url: "http://localhost:11434".to_string(),
        pricing: ProviderPricing::default(), // Free
        enabled: true,
        models: Vec::new(),
        health: ProviderHealth::default(),
        headers: std::collections::HashMap::new(),
    });

    providers
}
