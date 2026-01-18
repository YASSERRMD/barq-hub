//! API Key Authentication Interceptor for gRPC
//!
//! Validates API keys from request metadata against the database.

use std::sync::Arc;
use tonic::{Request, Status};
use crate::db::ApplicationRepository;

/// Authentication interceptor that validates API keys
#[derive(Clone)]
pub struct ApiKeyInterceptor {
    app_repo: Arc<ApplicationRepository>,
}

impl ApiKeyInterceptor {
    pub fn new(app_repo: Arc<ApplicationRepository>) -> Self {
        Self { app_repo }
    }

    /// Validate the API key from the request metadata
    pub async fn validate(&self, request: Request<()>) -> Result<Request<()>, Status> {
        // Extract API key from metadata
        let api_key = self.extract_api_key(&request)?;

        // Validate against database
        match self.app_repo.validate_api_key(&api_key).await {
            Ok(Some(app)) => {
                if app.status != "active" {
                    return Err(Status::permission_denied("API key is suspended or expired"));
                }
                
                // Update last_used timestamp (fire and forget)
                let repo = self.app_repo.clone();
                let app_id = app.id.clone();
                tokio::spawn(async move {
                    let _ = repo.update_last_used(&app_id).await;
                });
                
                Ok(request)
            }
            Ok(None) => Err(Status::unauthenticated("Invalid API key")),
            Err(e) => {
                tracing::error!("Database error validating API key: {}", e);
                Err(Status::internal("Authentication service unavailable"))
            }
        }
    }

    fn extract_api_key<T>(&self, request: &Request<T>) -> Result<String, Status> {
        // Try x-api-key header first
        if let Some(key) = request.metadata().get("x-api-key") {
            return key
                .to_str()
                .map(|s| s.to_string())
                .map_err(|_| Status::invalid_argument("Invalid API key format"));
        }

        // Try authorization header (Bearer token style)
        if let Some(auth) = request.metadata().get("authorization") {
            let auth_str = auth
                .to_str()
                .map_err(|_| Status::invalid_argument("Invalid authorization header"))?;
            
            if let Some(key) = auth_str.strip_prefix("Bearer ") {
                return Ok(key.to_string());
            }
        }

        Err(Status::unauthenticated("Missing API key. Provide via 'x-api-key' or 'Authorization: Bearer <key>'"))
    }
}

/// Macro to create an authenticated service wrapper
#[macro_export]
macro_rules! require_auth {
    ($interceptor:expr, $request:expr) => {
        $interceptor.validate($request).await?
    };
}
