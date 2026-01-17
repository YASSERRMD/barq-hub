//! Smart router for provider selection
//!
//! Implements intelligent routing based on cost, latency, quality, and load balancing.

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    Provider, ProviderPreference, ChatRequest, ChatResponse,
    error::{Result, RoutingError, SynapseError},
    providers::{ProviderAdapter, create_adapter},
};

/// Smart router for selecting the best provider
pub struct SmartRouter {
    /// Available adapters indexed by provider ID
    adapters: Vec<(String, Arc<dyn ProviderAdapter>)>,
    /// Round-robin counter for load balancing
    round_robin_counter: RwLock<usize>,
    /// Provider health scores
    health_scores: RwLock<std::collections::HashMap<String, f64>>,
}

impl SmartRouter {
    /// Create a new router with the given providers
    pub fn new(providers: Vec<Provider>) -> Self {
        let adapters: Vec<_> = providers
            .into_iter()
            .filter(|p| p.enabled)
            .map(|p| {
                let id = p.id.clone();
                let adapter = create_adapter(p);
                (id, adapter)
            })
            .collect();

        Self {
            adapters,
            round_robin_counter: RwLock::new(0),
            health_scores: RwLock::new(std::collections::HashMap::new()),
        }
    }

    /// Route a request to the best provider based on preference
    pub async fn route(&self, request: &ChatRequest) -> Result<ChatResponse> {
        let preference = request.provider_preference.unwrap_or(ProviderPreference::CostOptimal);
        
        let adapter = self.select_provider(preference).await?;
        
        // Execute the request
        adapter.chat(request).await
    }

    /// Route with fallback - try multiple providers on failure
    /// If an explicit `provider` is specified in the request, only that provider is tried
    pub async fn route_with_fallback(&self, request: &ChatRequest) -> Result<ChatResponse> {
        // If explicit provider is specified, use only that provider
        if let Some(ref provider_id) = request.provider {
            return self.route_to_provider(provider_id, request).await;
        }
        
        let mut last_error = None;

        // Get ordered list of adapters to try
        let adapters = self.get_fallback_order().await;

        for (provider_id, adapter) in adapters {
            match adapter.chat(request).await {
                Ok(response) => {
                    // Update health score on success
                    self.update_health_score(&provider_id, true).await;
                    return Ok(response);
                }
                Err(e) => {
                    // Update health score on failure
                    self.update_health_score(&provider_id, false).await;
                    tracing::warn!(provider = %provider_id, error = %e, "Provider failed, trying next");
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| 
            SynapseError::Routing(RoutingError::AllProvidersFailed)
        ))
    }

    /// Route directly to a specific provider by ID
    pub async fn route_to_provider(&self, provider_id: &str, request: &ChatRequest) -> Result<ChatResponse> {
        let adapter = self.adapters
            .iter()
            .find(|(id, _)| id == provider_id || id.to_lowercase() == provider_id.to_lowercase())
            .map(|(_, adapter)| adapter.clone())
            .ok_or_else(|| SynapseError::Routing(RoutingError::ProviderNotFound(provider_id.to_string())))?;
        
        match adapter.chat(request).await {
            Ok(response) => {
                self.update_health_score(provider_id, true).await;
                Ok(response)
            }
            Err(e) => {
                self.update_health_score(provider_id, false).await;
                Err(e)
            }
        }
    }

    /// Select a provider based on the given preference
    async fn select_provider(&self, preference: ProviderPreference) -> Result<Arc<dyn ProviderAdapter>> {
        if self.adapters.is_empty() {
            return Err(SynapseError::Routing(RoutingError::NoProvidersAvailable));
        }

        match preference {
            ProviderPreference::CostOptimal => self.select_cheapest().await,
            ProviderPreference::LatencyOptimal => self.select_fastest().await,
            ProviderPreference::QualityTier => self.select_highest_quality().await,
            ProviderPreference::LoadBalanced => self.select_round_robin().await,
            ProviderPreference::SpecificProvider(idx) => self.select_by_index(idx).await,
        }
    }

    /// Select the cheapest provider
    async fn select_cheapest(&self) -> Result<Arc<dyn ProviderAdapter>> {
        self.adapters
            .iter()
            .min_by(|a, b| {
                let cost_a = a.1.provider().pricing.input_token_cost 
                    + a.1.provider().pricing.output_token_cost;
                let cost_b = b.1.provider().pricing.input_token_cost 
                    + b.1.provider().pricing.output_token_cost;
                cost_a.partial_cmp(&cost_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(_, adapter)| adapter.clone())
            .ok_or_else(|| SynapseError::Routing(RoutingError::NoProvidersAvailable))
    }

    /// Select the fastest provider based on health scores
    async fn select_fastest(&self) -> Result<Arc<dyn ProviderAdapter>> {
        let health_scores = self.health_scores.read().await;
        
        self.adapters
            .iter()
            .max_by(|a, b| {
                let score_a = health_scores.get(&a.0).unwrap_or(&0.5);
                let score_b = health_scores.get(&b.0).unwrap_or(&0.5);
                score_a.partial_cmp(score_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(_, adapter)| adapter.clone())
            .ok_or_else(|| SynapseError::Routing(RoutingError::NoProvidersAvailable))
    }

    /// Select highest quality provider (predefined quality ranking)
    async fn select_highest_quality(&self) -> Result<Arc<dyn ProviderAdapter>> {
        // Quality ranking: GPT-4 > Claude 3 Opus > Mistral Large > others
        let quality_order = ["gpt-4", "claude-3-opus", "mistral-large", "gpt-3.5"];

        for quality in quality_order {
            if let Some((_, adapter)) = self.adapters.iter()
                .find(|(_, a)| a.provider().name.to_lowercase().contains(quality)) {
                return Ok(adapter.clone());
            }
        }

        // Fall back to first available
        self.adapters
            .first()
            .map(|(_, adapter)| adapter.clone())
            .ok_or_else(|| SynapseError::Routing(RoutingError::NoProvidersAvailable))
    }

    /// Round-robin load balancing
    async fn select_round_robin(&self) -> Result<Arc<dyn ProviderAdapter>> {
        let mut counter = self.round_robin_counter.write().await;
        let idx = *counter % self.adapters.len();
        *counter = counter.wrapping_add(1);

        self.adapters
            .get(idx)
            .map(|(_, adapter)| adapter.clone())
            .ok_or_else(|| SynapseError::Routing(RoutingError::NoProvidersAvailable))
    }

    /// Select by specific index
    async fn select_by_index(&self, idx: usize) -> Result<Arc<dyn ProviderAdapter>> {
        self.adapters
            .get(idx)
            .map(|(_, adapter)| adapter.clone())
            .ok_or_else(|| SynapseError::Routing(RoutingError::InvalidProviderIndex(idx)))
    }

    /// Get adapters in fallback order (prioritize healthy ones)
    async fn get_fallback_order(&self) -> Vec<(String, Arc<dyn ProviderAdapter>)> {
        let health_scores = self.health_scores.read().await;
        
        let mut adapters: Vec<_> = self.adapters.clone();
        adapters.sort_by(|a, b| {
            let score_a = health_scores.get(&a.0).unwrap_or(&0.5);
            let score_b = health_scores.get(&b.0).unwrap_or(&0.5);
            score_b.partial_cmp(score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        adapters
    }

    /// Update health score for a provider
    async fn update_health_score(&self, provider_id: &str, success: bool) {
        let mut scores = self.health_scores.write().await;
        let current = scores.get(provider_id).copied().unwrap_or(0.5);
        
        // Exponential moving average
        let new_score = if success {
            current * 0.9 + 0.1 * 1.0
        } else {
            current * 0.9 + 0.1 * 0.0
        };
        
        scores.insert(provider_id.to_string(), new_score);
    }

    /// Get list of available provider names
    pub fn list_providers(&self) -> Vec<String> {
        self.adapters
            .iter()
            .map(|(_, a)| a.provider().name.clone())
            .collect()
    }

    /// Health check all providers
    pub async fn health_check_all(&self) -> std::collections::HashMap<String, bool> {
        let mut results = std::collections::HashMap::new();
        
        for (id, adapter) in &self.adapters {
            let healthy = adapter.health_check().await.unwrap_or(false);
            results.insert(id.clone(), healthy);
            self.update_health_score(id, healthy).await;
        }
        
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ProviderPricing, ProviderType, ProviderHealth};

    fn test_providers() -> Vec<Provider> {
        vec![
            Provider {
                id: "cheap".to_string(),
                name: "Cheap Provider".to_string(),
                provider_type: ProviderType::OpenAI,
                api_key: "test".to_string(),
                base_url: "https://api.openai.com/v1".to_string(),
                pricing: ProviderPricing {
                    input_token_cost: 1.0,
                    output_token_cost: 2.0,
                },
                enabled: true,
                health: ProviderHealth::default(),
                headers: std::collections::HashMap::new(),
            },
            Provider {
                id: "expensive".to_string(),
                name: "Expensive GPT-4 Provider".to_string(),
                provider_type: ProviderType::OpenAI,
                api_key: "test".to_string(),
                base_url: "https://api.openai.com/v1".to_string(),
                pricing: ProviderPricing {
                    input_token_cost: 30.0,
                    output_token_cost: 60.0,
                },
                enabled: true,
                health: ProviderHealth::default(),
                headers: std::collections::HashMap::new(),
            },
        ]
    }

    #[test]
    fn test_router_creation() {
        let router = SmartRouter::new(test_providers());
        assert_eq!(router.list_providers().len(), 2);
    }

    #[tokio::test]
    async fn test_cost_optimal_routing() {
        let router = SmartRouter::new(test_providers());
        let adapter = router.select_cheapest().await.unwrap();
        assert!(adapter.provider().name.contains("Cheap"));
    }

    #[test]
    fn test_disabled_providers_filtered() {
        let mut providers = test_providers();
        providers[0].enabled = false;
        
        let router = SmartRouter::new(providers);
        assert_eq!(router.list_providers().len(), 1);
    }
}
