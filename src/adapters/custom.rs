//! # Custom Adapter Module
//!
//! This module provides the Custom adapter implementation for
//! any generic OpenAI-compatible endpoint.

use crate::{
    adapters::base::{AdapterTrait, AdapterUtils},
    error::ProxyError,
    schemas::{ChatCompletionRequest, ChatCompletionResponse},
};
#[cfg(feature = "server")]
use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use reqwest::Client;
use tracing::debug;

/// # Custom Adapter
///
/// Generic adapter for any OpenAI-compatible endpoint that doesn't
/// fit into the specific adapter categories.
#[derive(Clone, Debug)]
pub struct CustomAdapter {
    /// Base URL for the custom endpoint
    base_url: String,
    /// Model identifier
    model_id: String,
    /// Optional authentication token
    token: Option<String>,
    /// HTTP client with connection pooling
    client: Client,
}

impl CustomAdapter {
    /// Create a new Custom adapter instance
    pub fn new(base_url: String, model_id: String, token: Option<String>, client: Client) -> Self {
        Self {
            base_url,
            model_id,
            token,
            client,
        }
    }

    /// Get base URL (public accessor)
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Get model ID (public accessor)
    pub fn model_id(&self) -> &str {
        &self.model_id
    }

    /// Get token (public accessor)
    pub fn token(&self) -> &Option<String> {
        &self.token
    }

    /// Process chat completion requests
    #[cfg(feature = "server")]
    pub async fn chat_completions_http(&self, req: ChatCompletionRequest) -> Result<Response, ProxyError> {
        AdapterUtils::log_request("custom", &AdapterUtils::extract_model(&req, &self.model_id), req.messages.len());

        let start_time = std::time::Instant::now();

        // Build the endpoint URL - assume OpenAI-compatible
        let url = format!("{}/chat/completions", self.base_url);

        // Forward the request to the custom endpoint
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
                debug!("Custom endpoint request failed: {}", e);
                ProxyError::Upstream(e.to_string())
            })?;

        let status = resp.status();
        debug!("Custom endpoint response status: {}", status);

        let response_bytes = resp
            .bytes()
            .await
            .map_err(|e| {
                debug!("Failed to read custom endpoint response body: {}", e);
                ProxyError::Upstream(format!("error reading response body: {}", e))
            })?;

        let response_time = start_time.elapsed().as_millis() as u64;
        AdapterUtils::log_response("custom", &AdapterUtils::extract_model(&req, &self.model_id), status.is_success(), response_time);

        if !status.is_success() {
            let error_text = String::from_utf8_lossy(&response_bytes);
            debug!("Custom endpoint error response: {}", error_text);
            return Err(ProxyError::Upstream(format!("HTTP {}: {}", status, error_text)));
        }

        let json = serde_json::from_slice::<serde_json::Value>(&response_bytes)
            .map_err(|e| {
                debug!("Failed to parse custom endpoint JSON response: {}", e);
                ProxyError::Upstream(format!("error decoding response body: {} (body: {})", e, String::from_utf8_lossy(&response_bytes)))
            })?;

        debug!("Successfully forwarded custom endpoint request");
        Ok((StatusCode::OK, Json(json)).into_response())
    }
}

#[async_trait::async_trait]
impl AdapterTrait for CustomAdapter {
    fn name(&self) -> &'static str {
        "custom"
    }

    fn base_url(&self) -> &str {
        &self.base_url
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn has_auth(&self) -> bool {
        self.token.is_some()
    }

    #[cfg(feature = "server")]
    async fn chat_completions(&self, request: ChatCompletionRequest) -> Result<ChatCompletionResponse, ProxyError> {
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