//! # Application State
//!
//! This module defines the shared application state that is passed
//! to all HTTP handlers, consolidating configuration and adapter management.

use crate::{
    adapters::Adapter,
    config::Config,
    core::http_client::HttpClientBuilder,
    streaming::StreamingHandler,
};

/// # Application State
///
/// Shared state passed to all HTTP handlers containing configuration,
/// adapters, and other shared resources.
#[derive(Clone)]
pub struct AppState {
    /// Application configuration
    pub config: Config,
    /// LLM adapter for handling requests
    pub adapter: Adapter,
    /// Streaming handler for SSE responses
    pub streaming_handler: StreamingHandler,
    /// HTTP client for making requests
    pub http_client: reqwest::Client,
}

impl AppState {
    /// Create new application state from configuration
    pub async fn new(config: Config) -> Self {
        // Create the adapter based on configuration
        let adapter = Adapter::from_config(&config);

        // Create HTTP client using our centralized factory
        let http_client = HttpClientBuilder::from_config(&config)
            .build()
            .unwrap_or_else(|_| HttpClientBuilder::new().build().unwrap());

        // Create streaming handler
        let streaming_handler = StreamingHandler::default();

        Self {
            config,
            adapter,
            streaming_handler,
            http_client,
        }
    }

    /// Get a reference to the config
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get a reference to the adapter
    pub fn adapter(&self) -> &Adapter {
        &self.adapter
    }

    /// Get a reference to the streaming handler
    pub fn streaming_handler(&self) -> &StreamingHandler {
        &self.streaming_handler
    }

    /// Get a reference to the HTTP client
    pub fn http_client(&self) -> &reqwest::Client {
        &self.http_client
    }

    /// Check if streaming is enabled and supported
    pub fn supports_streaming(&self) -> bool {
        self.config.enable_streaming && self.adapter.supports_streaming()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_state_creation() {
        let config = Config::for_test();
        let state = AppState::new(config).await;

        assert!(state.config().backend_url.len() > 0);
        assert!(state.adapter().name().len() > 0);
    }

    #[tokio::test]
    async fn test_streaming_support() {
        let mut config = Config::for_test();
        config.enable_streaming = true;
        config.backend_url = "http://localhost:8000".to_string(); // LightLLM supports streaming

        let state = AppState::new(config).await;
        assert!(state.supports_streaming());
    }

    #[tokio::test]
    async fn test_streaming_disabled() {
        let mut config = Config::for_test();
        config.enable_streaming = false;

        let state = AppState::new(config).await;
        assert!(!state.supports_streaming());
    }
}