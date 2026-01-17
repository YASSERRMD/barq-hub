//! API middleware

use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    body::Body,
};
use tracing::{info, warn};
use std::time::Instant;

/// Request logging middleware
pub async fn logging_middleware(request: Request<Body>, next: Next) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path().to_string();

    let response = next.run(request).await;

    let latency = start.elapsed();
    let status = response.status();

    if status.is_server_error() {
        warn!(
            method = %method,
            path = %path,
            status = %status.as_u16(),
            latency_ms = %latency.as_millis(),
            "Request failed"
        );
    } else {
        info!(
            method = %method,
            path = %path,
            status = %status.as_u16(),
            latency_ms = %latency.as_millis(),
            "Request completed"
        );
    }

    response
}

/// Rate limiting state (simple in-memory)
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct RateLimiter {
    requests: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
    max_requests: usize,
    window_secs: u64,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            max_requests,
            window_secs,
        }
    }

    pub async fn check(&self, key: &str) -> bool {
        let now = Instant::now();
        let window = std::time::Duration::from_secs(self.window_secs);
        
        let mut requests = self.requests.write().await;
        let entry = requests.entry(key.to_string()).or_insert_with(Vec::new);
        
        // Remove old requests
        entry.retain(|t| now.duration_since(*t) < window);
        
        if entry.len() >= self.max_requests {
            return false;
        }
        
        entry.push(now);
        true
    }
}

/// Optional: API key validation middleware
pub async fn api_key_middleware(
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get API key from header
    let api_key = request
        .headers()
        .get("x-api-key")
        .or_else(|| request.headers().get("authorization"))
        .and_then(|v| v.to_str().ok());

    // For now, accept all requests (implement proper validation later)
    // In production, validate against stored API keys
    if api_key.is_none() {
        // Allow requests without API key for development
        // In production, uncomment the following:
        // return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(request).await)
}
