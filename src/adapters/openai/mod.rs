//! # OpenAI Adapter Module
//!
//! This module provides the OpenAI adapter implementation for direct
//! communication with OpenAI API and OpenAI-compatible endpoints.
//!
//! ## Key Features:
//! - Zero-copy request forwarding
//! - Direct JSON pass-through
//! - Full OpenAI API compatibility
//! - Streaming support
//! - Bearer token authentication

use crate::{
    adapters::base::{AdapterTrait, AdapterUtils},
    error::ProxyError,
    schemas::{ChatCompletionRequest, ChatCompletionResponse},
};
#[cfg(feature = "server")]
use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use reqwest::Client;
use tracing::debug;

/// # OpenAI Adapter
///
/// Direct pass-through adapter for OpenAI API and OpenAI-compatible endpoints.
/// This adapter forwards requests without modification, making it very efficient
/// for services that already use the OpenAI format.
#[derive(Clone, Debug)]
pub struct OpenAIAdapter {
    /// Base URL for the OpenAI-compatible endpoint (e.g., "https://api.openai.com/v1")
    base: String,
    /// HTTP client with connection pooling and optimizations
    client: Client,
    /// Model ID to use for requests (currently unused but kept for compatibility)
    model_id: String,
    /// Optional authentication token
    token: Option<String>,
}

impl OpenAIAdapter {
    /// Create a new OpenAI adapter instance
    pub fn new(base: String, model_id: String, token: Option<String>, client: Client) -> Self {
        Self {
            base,
            client,
            model_id,
            token,
        }
    }

    /// Get the model ID for this adapter
    pub fn model_id(&self) -> &str {
        &self.model_id
    }

    /// Process chat completion requests with direct forwarding
    #[cfg(feature = "server")]
    pub async fn chat_completions_http(&self, req: ChatCompletionRequest) -> Result<Response, ProxyError> {
        AdapterUtils::log_request("openai", &AdapterUtils::extract_model(&req, &self.model_id), req.messages.len());

        let start_time = std::time::Instant::now();

        // Build the OpenAI API endpoint URL
        let url = format!("{}/chat/completions", self.base);

        // Forward the request as-is to the OpenAI-compatible endpoint
        let mut request_builder = self.client.post(url).json(&req);

        // Add authentication header if token is present
        if let Some(token) = &self.token {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", token));
        }

        // Send the request and await the response
        let resp = request_builder
            .send()
            .await
            .map_err(|e| {
                debug!("OpenAI request failed: {}", e);
                ProxyError::Upstream(e.to_string())
            })?;

        let status = resp.status();
        debug!("OpenAI response status: {}", status);

        // Use bytes() instead of text() to avoid unnecessary string conversion
        let response_bytes = resp
            .bytes()
            .await
            .map_err(|e| {
                debug!("Failed to read OpenAI response body: {}", e);
                ProxyError::Upstream(format!("error reading response body: {}", e))
            })?;

        let response_time = start_time.elapsed().as_millis() as u64;
        AdapterUtils::log_response("openai", &AdapterUtils::extract_model(&req, &self.model_id), status.is_success(), response_time);

        // Check if the request was successful
        if !status.is_success() {
            let error_text = String::from_utf8_lossy(&response_bytes);
            debug!("OpenAI error response: {}", error_text);
            return Err(ProxyError::Upstream(format!("HTTP {}: {}", status, error_text)));
        }

        // If streaming was requested, just return the raw response body for the streaming adapter to handle
        if req.stream.unwrap_or(false) {
            let response = Response::builder()
                .status(status)
                .body(axum::body::Body::from(response_bytes))
                .map_err(|e| ProxyError::Internal(format!("Failed to build response: {}", e)))?;
            return Ok(response);
        }

        // Parse JSON directly from bytes (zero-copy operation) for non-streaming responses
        let json = serde_json::from_slice::<serde_json::Value>(&response_bytes)
            .map_err(|e| {
                debug!("Failed to parse OpenAI JSON response: {}", e);
                ProxyError::Upstream(format!("error decoding response body: {} (body: {})", e, String::from_utf8_lossy(&response_bytes)))
            })?;

        debug!("Successfully forwarded OpenAI request");

        // Return the response as-is (no format conversion needed)
        Ok((StatusCode::OK, Json(json)).into_response())
    }
}

#[async_trait::async_trait]
impl AdapterTrait for OpenAIAdapter {
    fn name(&self) -> &'static str {
        "openai"
    }

    fn base_url(&self) -> &str {
        &self.base
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn has_auth(&self) -> bool {
        self.token.is_some()
    }

    #[cfg(feature = "server")]
    async fn chat_completions(&self, request: ChatCompletionRequest) -> Result<ChatCompletionResponse, ProxyError> {
        // Get the HTTP response from the HTTP implementation
        let http_response = self.chat_completions_http(request).await?;

        // Extract the response body
        let body_bytes = axum::body::to_bytes(http_response.into_body(), usize::MAX)
            .await
            .map_err(|e| ProxyError::Internal(format!("Failed to read response body: {}", e)))?;

        // Parse the JSON response into ChatCompletionResponse
        let response: ChatCompletionResponse = serde_json::from_slice(&body_bytes)
            .map_err(|e| ProxyError::Internal(format!("Failed to parse response JSON: {}", e)))?;

        Ok(response)
    }

    #[cfg(not(feature = "server"))]
    async fn chat_completions(&self, _request: ChatCompletionRequest) -> Result<ChatCompletionResponse, ProxyError> {
        Err(ProxyError::Internal("Server feature not enabled".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::http_client::HttpClientBuilder;

    #[tokio::test]
    async fn test_openai_adapter_creation() {
        let client = HttpClientBuilder::new().build().unwrap();
        let adapter = OpenAIAdapter::new(
            "https://api.openai.com/v1".to_string(),
            "gpt-3.5-turbo".to_string(),
            Some("test-token".to_string()),
            client,
        );

        assert_eq!(adapter.name(), "openai");
        assert_eq!(adapter.base_url(), "https://api.openai.com/v1");
        assert_eq!(adapter.model_id(), "gpt-3.5-turbo");
        assert!(adapter.has_auth());
    }

    #[test]
    fn test_openai_adapter_without_auth() {
        let client = HttpClientBuilder::new().build().unwrap();
        let adapter = OpenAIAdapter::new(
            "https://api.openai.com/v1".to_string(),
            "gpt-3.5-turbo".to_string(),
            None,
            client,
        );

        assert!(!adapter.has_auth());
    }
}