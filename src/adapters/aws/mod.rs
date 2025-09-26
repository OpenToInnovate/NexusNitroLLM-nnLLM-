//! # AWS Bedrock Adapter Module
//!
//! This module provides the AWS Bedrock adapter implementation
//! with AWS-specific authentication and API format handling.

use crate::{
    adapters::base::{AdapterTrait, AdapterUtils},
    error::ProxyError,
    schemas::{ChatCompletionRequest, ChatCompletionResponse},
};
#[cfg(feature = "server")]
use axum::response::Response;
use reqwest::Client;

/// # AWS Bedrock Adapter
///
/// Adapter for Amazon Web Services Bedrock with AWS-specific
/// authentication and API format conversion.
#[derive(Clone, Debug)]
pub struct AWSBedrockAdapter {
    /// Base URL for AWS Bedrock
    base: String,
    /// Model identifier
    model_id: String,
    /// AWS access key
    access_key: Option<String>,
    /// HTTP client with connection pooling
    #[allow(dead_code)]
    client: Client,
}

impl AWSBedrockAdapter {
    /// Create a new AWS Bedrock adapter instance
    pub fn new(base: String, model_id: String, access_key: Option<String>, client: Client) -> Self {
        Self {
            base,
            model_id,
            access_key,
            client,
        }
    }

    /// Process chat completion requests with AWS Bedrock-specific handling
    #[cfg(feature = "server")]
    pub async fn chat_completions_http(&self, req: ChatCompletionRequest) -> Result<Response, ProxyError> {
        AdapterUtils::log_request("aws", &AdapterUtils::extract_model(&req, &self.model_id), req.messages.len());

        let start_time = std::time::Instant::now();

        // AWS Bedrock integration is more complex and would require:
        // 1. Converting OpenAI format to Bedrock format
        // 2. AWS Signature V4 authentication
        // 3. Model-specific request formatting
        // For now, return a not implemented error

        let response_time = start_time.elapsed().as_millis() as u64;
        AdapterUtils::log_response("aws", &AdapterUtils::extract_model(&req, &self.model_id), false, response_time);

        Err(ProxyError::BadRequest(
            "AWS Bedrock integration not yet implemented".to_string()
        ))
    }
}

#[async_trait::async_trait]
impl AdapterTrait for AWSBedrockAdapter {
    fn name(&self) -> &'static str {
        "aws"
    }

    fn base_url(&self) -> &str {
        &self.base
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn has_auth(&self) -> bool {
        self.access_key.is_some()
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