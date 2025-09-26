//! # Azure OpenAI Adapter Module
//!
//! This module provides the Azure OpenAI Service adapter implementation
//! with Azure-specific authentication and endpoint handling.

use crate::{
    adapters::base::{AdapterTrait, AdapterUtils},
    error::ProxyError,
    schemas::{ChatCompletionRequest, ChatCompletionResponse},
};
#[cfg(feature = "server")]
use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use reqwest::Client;
use tracing::debug;

/// # Azure OpenAI Adapter
///
/// Adapter for Microsoft Azure OpenAI Service with Azure-specific
/// authentication and endpoint handling.
#[derive(Clone, Debug)]
pub struct AzureOpenAIAdapter {
    /// Base URL for Azure OpenAI Service
    base: String,
    /// Model identifier
    model_id: String,
    /// Azure API key
    api_key: Option<String>,
    /// HTTP client with connection pooling
    client: Client,
}

impl AzureOpenAIAdapter {
    /// Create a new Azure OpenAI adapter instance
    pub fn new(base: String, model_id: String, api_key: Option<String>, client: Client) -> Self {
        Self {
            base,
            model_id,
            api_key,
            client,
        }
    }

    /// Get the model ID for this adapter
    pub fn model_id(&self) -> &str {
        &self.model_id
    }

    /// Process chat completion requests with Azure-specific handling
    #[cfg(feature = "server")]
    pub async fn chat_completions_http(&self, req: ChatCompletionRequest) -> Result<Response, ProxyError> {
        AdapterUtils::log_request("azure", &AdapterUtils::extract_model(&req, &self.model_id), req.messages.len());

        let start_time = std::time::Instant::now();

        // Build Azure OpenAI endpoint URL
        // Azure format: https://{resource}.openai.azure.com/openai/deployments/{deployment-id}/chat/completions?api-version=2023-12-01-preview
        let url = format!("{}/openai/deployments/{}/chat/completions?api-version=2023-12-01-preview",
                         self.base, self.model_id);

        // Forward the request to the Azure endpoint
        let mut request_builder = self.client.post(url).json(&req);

        // Add Azure API key authentication
        if let Some(api_key) = &self.api_key {
            request_builder = request_builder.header("api-key", api_key);
        }

        // Send the request and await the response
        let resp = request_builder
            .send()
            .await
            .map_err(|e| {
                debug!("Azure OpenAI request failed: {}", e);
                ProxyError::Upstream(e.to_string())
            })?;

        let status = resp.status();
        debug!("Azure OpenAI response status: {}", status);

        let response_bytes = resp
            .bytes()
            .await
            .map_err(|e| {
                debug!("Failed to read Azure response body: {}", e);
                ProxyError::Upstream(format!("error reading response body: {}", e))
            })?;

        let response_time = start_time.elapsed().as_millis() as u64;
        AdapterUtils::log_response("azure", &AdapterUtils::extract_model(&req, &self.model_id), status.is_success(), response_time);

        if !status.is_success() {
            let error_text = String::from_utf8_lossy(&response_bytes);
            debug!("Azure error response: {}", error_text);
            return Err(ProxyError::Upstream(format!("HTTP {}: {}", status, error_text)));
        }

        let json = serde_json::from_slice::<serde_json::Value>(&response_bytes)
            .map_err(|e| {
                debug!("Failed to parse Azure JSON response: {}", e);
                ProxyError::Upstream(format!("error decoding response body: {} (body: {})", e, String::from_utf8_lossy(&response_bytes)))
            })?;

        debug!("Successfully forwarded Azure OpenAI request");
        Ok((StatusCode::OK, Json(json)).into_response())
    }
}

#[async_trait::async_trait]
impl AdapterTrait for AzureOpenAIAdapter {
    fn name(&self) -> &'static str {
        "azure"
    }

    fn base_url(&self) -> &str {
        &self.base
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn has_auth(&self) -> bool {
        self.api_key.is_some()
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