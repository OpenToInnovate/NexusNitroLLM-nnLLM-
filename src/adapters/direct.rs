//! # Direct Adapter Module
//!
//! This module provides the Direct adapter implementation for
//! embedded LLM integration that bypasses HTTP entirely.

use crate::{
    adapters::base::{AdapterTrait, AdapterUtils},
    error::ProxyError,
    schemas::{ChatCompletionRequest, ChatCompletionResponse},
};

/// # Direct Adapter
///
/// Direct integration adapter that bypasses HTTP for maximum performance
/// in embedded applications or when the LLM is running in the same process.
#[derive(Clone, Debug)]
pub struct DirectAdapter {
    /// Model ID for direct LightLLM integration
    model_id: String,
    /// Optional authentication token
    token: Option<String>,
}

impl DirectAdapter {
    /// Create a new Direct adapter instance
    pub fn new(model_id: String, token: Option<String>) -> Self {
        Self {
            model_id,
            token,
        }
    }

    /// Process chat completion requests directly
    pub async fn chat_completions(&self, req: ChatCompletionRequest) -> Result<ChatCompletionResponse, ProxyError> {
        AdapterUtils::log_request("direct", &AdapterUtils::extract_model(&req, &self.model_id), req.messages.len());

        // Direct mode would integrate with the LLM library directly
        // This would require linking against the LLM inference engine
        // For now, return a not implemented error

        Err(ProxyError::BadRequest(
            "Direct mode integration not yet implemented".to_string()
        ))
    }
}

#[async_trait::async_trait]
impl AdapterTrait for DirectAdapter {
    fn name(&self) -> &'static str {
        "direct"
    }

    fn base_url(&self) -> &str {
        "direct://"
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn has_auth(&self) -> bool {
        self.token.is_some()
    }

    async fn chat_completions(&self, request: ChatCompletionRequest) -> Result<ChatCompletionResponse, ProxyError> {
        self.chat_completions(request).await
    }
}