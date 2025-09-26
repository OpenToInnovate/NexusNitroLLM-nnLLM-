//! # Base Adapter Functionality
//!
//! Common functionality and traits shared across all LLM adapters.
//! This provides the foundation for consistent adapter behavior.

use crate::{
    config::Config,
    error::ProxyError,
    schemas::{ChatCompletionRequest, ChatCompletionResponse},
};
use crate::core::http_client::{HttpClientBuilder, HttpClientError};
use reqwest::Client;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::debug;

/// Common adapter configuration
#[derive(Debug, Clone)]
pub struct AdapterConfig {
    pub base_url: String,
    pub model_id: String,
    pub token: Option<String>,
}

impl AdapterConfig {
    pub fn new(base_url: impl Into<String>, model_id: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            model_id: model_id.into(),
            token: None,
        }
    }

    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }
}

/// Base adapter trait that all LLM adapters must implement
#[async_trait::async_trait]
pub trait AdapterTrait: Send + Sync {
    /// Get the adapter name for logging and metrics
    fn name(&self) -> &'static str;

    /// Get the base URL for this adapter
    fn base_url(&self) -> &str;

    /// Get the model ID for this adapter
    fn model_id(&self) -> &str;

    /// Check if the adapter has authentication configured
    fn has_auth(&self) -> bool;

    /// Process a chat completion request
    async fn chat_completions(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, ProxyError>;
}

/// Utility functions for adapters
pub struct AdapterUtils;

impl AdapterUtils {
    /// Create an HTTP client from configuration
    pub fn create_http_client(config: &Config) -> Result<Client, HttpClientError> {
        HttpClientBuilder::from_config(config).build()
    }

    /// Create a production HTTP client
    pub fn create_production_http_client() -> Result<Client, HttpClientError> {
        HttpClientBuilder::production().build()
    }

    /// Generate a consistent hash for caching and request deduplication
    pub fn generate_request_hash(request: &ChatCompletionRequest) -> u64 {
        let mut hasher = DefaultHasher::new();

        // Hash the essential parts of the request for caching
        request.messages.hash(&mut hasher);
        if let Some(ref model) = request.model {
            model.hash(&mut hasher);
        }
        if let Some(temperature) = request.temperature {
            temperature.to_bits().hash(&mut hasher);
        }
        if let Some(max_tokens) = request.max_tokens {
            max_tokens.hash(&mut hasher);
        }

        hasher.finish()
    }

    /// Get current timestamp for response metadata
    pub fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0)
    }

    /// Validate URL format
    pub fn validate_url(url: &str) -> Result<(), ProxyError> {
        if url.is_empty() {
            return Err(ProxyError::BadRequest("URL cannot be empty".to_string()));
        }

        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(ProxyError::BadRequest("URL must start with http:// or https://".to_string()));
        }

        Ok(())
    }

    /// Extract model from request or use default
    pub fn extract_model(request: &ChatCompletionRequest, default_model: &str) -> String {
        request.model.clone().unwrap_or_else(|| default_model.to_string())
    }

    /// Log adapter request for debugging
    pub fn log_request(adapter_name: &str, model: &str, message_count: usize) {
        debug!(
            adapter = adapter_name,
            model = model,
            message_count = message_count,
            "Processing chat completion request"
        );
    }

    /// Log adapter response for debugging
    pub fn log_response(adapter_name: &str, model: &str, success: bool, response_time_ms: u64) {
        debug!(
            adapter = adapter_name,
            model = model,
            success = success,
            response_time_ms = response_time_ms,
            "Completed chat completion request"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schemas::Message;

    #[test]
    fn test_adapter_config_creation() {
        let config = AdapterConfig::new("https://api.example.com", "test-model")
            .with_token("test-token");

        assert_eq!(config.base_url, "https://api.example.com");
        assert_eq!(config.model_id, "test-model");
        assert_eq!(config.token, Some("test-token".to_string()));
    }

    #[test]
    fn test_url_validation() {
        assert!(AdapterUtils::validate_url("https://api.example.com").is_ok());
        assert!(AdapterUtils::validate_url("http://localhost:8000").is_ok());
        assert!(AdapterUtils::validate_url("").is_err());
        assert!(AdapterUtils::validate_url("ftp://example.com").is_err());
    }

    #[test]
    fn test_request_hash_consistency() {
        let request = ChatCompletionRequest {
            messages: vec![Message {
                role: "user".to_string(),
                content: Some("test".to_string()),
                name: None,
                tool_calls: None,
                function_call: None,
                tool_call_id: None,
            }],
            model: Some("test-model".to_string()),
            temperature: Some(0.7),
            max_tokens: Some(100),
            ..Default::default()
        };

        let hash1 = AdapterUtils::generate_request_hash(&request);
        let hash2 = AdapterUtils::generate_request_hash(&request);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_model_extraction() {
        let request = ChatCompletionRequest {
            model: Some("custom-model".to_string()),
            ..Default::default()
        };

        assert_eq!(AdapterUtils::extract_model(&request, "default"), "custom-model");

        let request_no_model = ChatCompletionRequest {
            model: None,
            ..Default::default()
        };

        assert_eq!(AdapterUtils::extract_model(&request_no_model, "default"), "default");
    }
}