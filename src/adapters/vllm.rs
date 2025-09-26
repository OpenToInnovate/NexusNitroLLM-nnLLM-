//! # vLLM Adapter Module
//!
//! This module provides the vLLM adapter implementation for
//! OpenAI-compatible vLLM server integration.

use crate::{
    adapters::base::{AdapterTrait, AdapterUtils},
    error::ProxyError,
    schemas::{ChatCompletionRequest, ChatCompletionResponse},
};
#[cfg(feature = "server")]
use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use reqwest::Client;
use tracing::debug;

/// # vLLM Adapter
///
/// Adapter for vLLM servers that provide OpenAI-compatible endpoints
/// with vLLM-specific optimizations.
#[derive(Clone, Debug)]
pub struct VLLMAdapter {
    /// Base URL for the vLLM server
    base: String,
    /// Model identifier
    model_id: String,
    /// Optional authentication token
    token: Option<String>,
    /// HTTP client with connection pooling
    client: Client,
}

impl VLLMAdapter {
    /// Create a new vLLM adapter instance
    pub fn new(base: String, model_id: String, token: Option<String>, client: Client) -> Self {
        Self {
            base,
            model_id,
            token,
            client,
        }
    }

    /// Get the model ID for this adapter
    pub fn model_id(&self) -> &str {
        &self.model_id
    }

    /// Process chat completion requests
    #[cfg(feature = "server")]
    pub async fn chat_completions_http(&self, req: ChatCompletionRequest) -> Result<Response, ProxyError> {
        AdapterUtils::log_request("vllm", &AdapterUtils::extract_model(&req, &self.model_id), req.messages.len());

        let start_time = std::time::Instant::now();

        // Build the vLLM API endpoint URL (OpenAI-compatible)
        let url = format!("{}/v1/chat/completions", self.base);

        // Forward the request to the vLLM endpoint
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
                debug!("vLLM request failed: {}", e);
                ProxyError::Upstream(e.to_string())
            })?;

        let status = resp.status();
        debug!("vLLM response status: {}", status);

        let response_bytes = resp
            .bytes()
            .await
            .map_err(|e| {
                debug!("Failed to read vLLM response body: {}", e);
                ProxyError::Upstream(format!("error reading response body: {}", e))
            })?;

        let response_time = start_time.elapsed().as_millis() as u64;
        AdapterUtils::log_response("vllm", &AdapterUtils::extract_model(&req, &self.model_id), status.is_success(), response_time);

        if !status.is_success() {
            let error_text = String::from_utf8_lossy(&response_bytes);
            debug!("vLLM error response: {}", error_text);
            return Err(ProxyError::Upstream(format!("HTTP {}: {}", status, error_text)));
        }

        let json = serde_json::from_slice::<serde_json::Value>(&response_bytes)
            .map_err(|e| {
                debug!("Failed to parse vLLM JSON response: {}", e);
                ProxyError::Upstream(format!("error decoding response body: {} (body: {})", e, String::from_utf8_lossy(&response_bytes)))
            })?;

        debug!("Successfully forwarded vLLM request");
        Ok((StatusCode::OK, Json(json)).into_response())
    }
}

#[async_trait::async_trait]
impl AdapterTrait for VLLMAdapter {
    fn name(&self) -> &'static str {
        "vllm"
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