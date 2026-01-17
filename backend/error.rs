//! Error types for SYNAPSE Brain

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// Main error type for SYNAPSE operations
#[derive(Debug, Error)]
pub enum SynapseError {
    #[error("Provider error: {0}")]
    Provider(#[from] ProviderError),

    #[error("Routing error: {0}")]
    Routing(#[from] RoutingError),

    #[error("Cost error: {0}")]
    Cost(#[from] CostError),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Provider-specific errors
#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("Request to provider failed: {0}")]
    RequestFailed(String),

    #[error("Invalid response from provider: {0}")]
    InvalidResponse(String),

    #[error("Provider not found: {0}")]
    NotFound(String),

    #[error("Provider rate limited")]
    RateLimited,

    #[error("Provider authentication failed")]
    AuthFailed,

    #[error("Provider timeout")]
    Timeout,

    #[error("Network error: {0}")]
    Network(String),
}

/// Routing-specific errors
#[derive(Debug, Error)]
pub enum RoutingError {
    #[error("No providers available")]
    NoProvidersAvailable,

    #[error("Provider not found at index: {0}")]
    InvalidProviderIndex(usize),

    #[error("Provider not found: {0}")]
    ProviderNotFound(String),

    #[error("No suitable provider for model: {0}")]
    NoProviderForModel(String),

    #[error("All providers failed")]
    AllProvidersFailed,
}

/// Cost-related errors
#[derive(Debug, Error)]
pub enum CostError {
    #[error("Budget exceeded for user: {0}")]
    BudgetExceeded(String),

    #[error("Invalid cost calculation")]
    InvalidCalculation,

    #[error("Missing pricing information for provider: {0}")]
    MissingPricing(String),
}

/// Result type alias for SYNAPSE operations
pub type Result<T> = std::result::Result<T, SynapseError>;

/// API error response structure
#[derive(Debug, serde::Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
    pub details: Option<serde_json::Value>,
}

impl IntoResponse for SynapseError {
    fn into_response(self) -> Response {
        let (status, error_code) = match &self {
            SynapseError::Provider(ProviderError::RateLimited) => {
                (StatusCode::TOO_MANY_REQUESTS, "RATE_LIMITED")
            }
            SynapseError::Provider(ProviderError::AuthFailed) => {
                (StatusCode::UNAUTHORIZED, "AUTH_FAILED")
            }
            SynapseError::Provider(ProviderError::Timeout) => {
                (StatusCode::GATEWAY_TIMEOUT, "PROVIDER_TIMEOUT")
            }
            SynapseError::Provider(_) => (StatusCode::BAD_GATEWAY, "PROVIDER_ERROR"),
            SynapseError::Routing(RoutingError::NoProvidersAvailable) => {
                (StatusCode::SERVICE_UNAVAILABLE, "NO_PROVIDERS")
            }
            SynapseError::Routing(_) => (StatusCode::BAD_REQUEST, "ROUTING_ERROR"),
            SynapseError::Cost(CostError::BudgetExceeded(_)) => {
                (StatusCode::PAYMENT_REQUIRED, "BUDGET_EXCEEDED")
            }
            SynapseError::Cost(_) => (StatusCode::INTERNAL_SERVER_ERROR, "COST_ERROR"),
            SynapseError::Validation(_) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR"),
            SynapseError::NotFound(_) => (StatusCode::NOT_FOUND, "NOT_FOUND"),
            SynapseError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR"),
            SynapseError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR"),
            SynapseError::Config(_) => (StatusCode::INTERNAL_SERVER_ERROR, "CONFIG_ERROR"),
            SynapseError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"),
        };

        let body = Json(json!({
            "error": self.to_string(),
            "code": error_code,
        }));

        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_error_display() {
        let err = ProviderError::RequestFailed("connection refused".to_string());
        assert!(err.to_string().contains("connection refused"));
    }

    #[test]
    fn test_routing_error_display() {
        let err = RoutingError::NoProvidersAvailable;
        assert_eq!(err.to_string(), "No providers available");
    }

    #[test]
    fn test_cost_error_display() {
        let err = CostError::BudgetExceeded("user123".to_string());
        assert!(err.to_string().contains("user123"));
    }
}
