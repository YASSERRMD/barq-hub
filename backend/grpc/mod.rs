//! gRPC module for Barq Hub
//!
//! Provides gRPC services with API key authentication.

pub mod auth;
pub mod chat_service;
pub mod models_service;

pub use auth::ApiKeyInterceptor;
pub use chat_service::ChatServiceImpl;
pub use models_service::ModelsServiceImpl;

// Include generated protobuf code
pub mod barq {
    tonic::include_proto!("barq");
}
