//! # Request Routing Module
//!
//! Implements intelligent request routing for load balancing and failover.

use crate::{
    adapters::Adapter,
    schemas::ChatCompletionRequest,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// # Routing Configuration
///
/// Configuration for request routing behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    /// Whether routing is enabled
    pub enabled: bool,
    /// Load balancing strategy
    pub strategy: LoadBalancingStrategy,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            strategy: LoadBalancingStrategy::RoundRobin,
        }
    }
}

/// # Load Balancing Strategy
///
/// Strategy for distributing requests across multiple backends.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    /// Round-robin distribution
    RoundRobin,
    /// Weighted round-robin
    WeightedRoundRobin,
    /// Least connections
    LeastConnections,
}

/// # Request Router
///
/// Routes requests to appropriate backends.
pub struct RequestRouter {
    /// Configuration
    config: RoutingConfig,
    /// Available adapters
    adapters: Vec<Arc<Adapter>>,
    /// Current index for round-robin
    current_index: std::sync::atomic::AtomicUsize,
}

impl RequestRouter {
    /// Create a new request router
    pub fn new(config: RoutingConfig, adapters: Vec<Arc<Adapter>>) -> Self {
        Self {
            config,
            adapters,
            current_index: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Route a request to an appropriate adapter
    pub async fn route_request(&self, _request: &ChatCompletionRequest) -> Result<Arc<Adapter>, crate::error::ProxyError> {
        if !self.config.enabled || self.adapters.is_empty() {
            return Err(crate::error::ProxyError::Internal("No adapters available".to_string()));
        }

        // Simple round-robin routing
        let index = self.current_index.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % self.adapters.len();
        Ok(self.adapters[index].clone())
    }
}