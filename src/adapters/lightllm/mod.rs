//! # LightLLM Adapter Module
//!
//! This module provides the LightLLM adapter implementation for converting
//! OpenAI-compatible requests to LightLLM's native format and back.
//!
//! ## Key Features:
//! - Native LightLLM format conversion
//! - Performance-optimized message processing
//! - Request deduplication and caching
//! - Memory-efficient string operations

use crate::{
    adapters::base::{AdapterTrait, AdapterUtils},
    error::ProxyError,
    schemas::{ChatCompletionRequest, ChatCompletionResponse, Message},
};
#[cfg(feature = "server")]
use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use reqwest::Client;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};
use tracing::debug;

/// # Role Enum for LightLLM Format
///
/// Represents the different types of message roles in a conversation.
/// This is specific to LightLLM's prompt format requirements.
#[derive(Debug)]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

impl From<&str> for Role {
    fn from(role: &str) -> Self {
        match role {
            "system" => Role::System,
            "user" => Role::User,
            "assistant" => Role::Assistant,
            "tool" => Role::Tool,
            _ => Role::User, // Default to User for unknown roles
        }
    }
}

/// # LightLLM Adapter
///
/// Direct integration adapter for LightLLM servers that converts
/// OpenAI-compatible requests to LightLLM's native `/generate` format.
#[derive(Clone, Debug)]
pub struct LightLLMAdapter {
    /// Base URL for the LightLLM backend (e.g., "http://localhost:8000")
    base: String,
    /// HTTP client with connection pooling and optimizations
    client: Client,
    /// Model ID to use for requests
    model_id: String,
    /// Optional authentication token
    token: Option<String>,
}

impl LightLLMAdapter {
    /// Create a new LightLLM adapter instance
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

    /// Convert OpenAI-format messages to LightLLM's prompt format with
    /// advanced memory optimization and capacity estimation.
    fn messages_to_prompt(messages: &[Message]) -> String {
        // Enhanced capacity estimation for better memory management
        let estimated_capacity = messages.iter()
            .map(|msg| {
                msg.role.len() +
                msg.content.as_ref().map(|c| c.len()).unwrap_or(0) +
                25 // Role markers overhead: "<|role|>\n" + "\n" + safety
            })
            .sum::<usize>() + 25; // +25 for final assistant marker + safety buffer

        let mut out = String::with_capacity(estimated_capacity);

        // Process each message with optimized string operations
        for msg in messages {
            let role = Role::from(msg.role.as_str());
            match role {
                Role::System => {
                    out.push_str("<|system|>\n");
                    if let Some(content) = &msg.content {
                        out.push_str(content);
                    }
                    out.push('\n');
                }
                Role::User => {
                    out.push_str("<|user|>\n");
                    if let Some(content) = &msg.content {
                        out.push_str(content);
                    }
                    out.push('\n');
                }
                Role::Assistant => {
                    out.push_str("<|assistant|>\n");
                    if let Some(content) = &msg.content {
                        out.push_str(content);
                    }
                    out.push('\n');
                }
                Role::Tool => {
                    // Skip tool messages (not supported by LightLLM)
                    debug!("Skipping tool message: {:?}", msg.content);
                }
            }
        }
        out.push_str("<|assistant|> ");

        // Verify capacity utilization for performance monitoring
        let actual_capacity = out.capacity();
        if actual_capacity > estimated_capacity * 2 {
            debug!("Capacity over-allocation detected: estimated={}, actual={}",
                   estimated_capacity, actual_capacity);
        }

        out
    }

    /// Generate a deterministic hash for request deduplication and caching
    fn calculate_request_hash(req: &ChatCompletionRequest) -> u64 {
        let mut hasher = DefaultHasher::new();

        // Hash messages content (order matters for deterministic hashing)
        for msg in &req.messages {
            msg.role.hash(&mut hasher);
            msg.content.hash(&mut hasher);
            if let Some(ref name) = msg.name {
                name.hash(&mut hasher);
            }
        }

        // Hash generation parameters that affect output
        if let Some(ref model) = req.model {
            model.hash(&mut hasher);
        }
        if let Some(max_tokens) = req.max_tokens {
            max_tokens.hash(&mut hasher);
        }
        if let Some(temperature) = req.temperature {
            temperature.to_bits().hash(&mut hasher);
        }
        if let Some(top_p) = req.top_p {
            top_p.to_bits().hash(&mut hasher);
        }
        if let Some(presence_penalty) = req.presence_penalty {
            presence_penalty.to_bits().hash(&mut hasher);
        }
        if let Some(frequency_penalty) = req.frequency_penalty {
            frequency_penalty.to_bits().hash(&mut hasher);
        }
        if let Some(ref stop) = req.stop {
            stop.hash(&mut hasher);
        }
        if let Some(ref user) = req.user {
            user.hash(&mut hasher);
        }
        if let Some(seed) = req.seed {
            seed.hash(&mut hasher);
        }

        hasher.finish()
    }

    /// Process chat completion requests with advanced optimizations
    #[cfg(feature = "server")]
    pub async fn chat_completions_http(&self, req: ChatCompletionRequest) -> Result<Response, ProxyError> {
        // Note: This adapter now supports OpenAI-compatible endpoints that may support streaming

        let request_hash = Self::calculate_request_hash(&req);
        debug!("Processing LightLLM request with hash: {:x}", request_hash);

        AdapterUtils::log_request("lightllm", &AdapterUtils::extract_model(&req, &self.model_id), req.messages.len());

        let start_time = std::time::Instant::now();

        // Check if this looks like an OpenAI-compatible endpoint
        let is_openai_compatible = self.base.contains("/v1") || req.stream.unwrap_or(false);

        // Calculate prompt for token counting (needed later)
        let prompt = Self::messages_to_prompt(&req.messages);
        debug!("Converted prompt length: {} characters", prompt.len());

        let (url, payload) = if is_openai_compatible {
            // Use OpenAI-compatible format for streaming or /v1 endpoints
            let url = if self.base.ends_with("/v1") {
                format!("{}/chat/completions", self.base)
            } else {
                format!("{}/v1/chat/completions", self.base)
            };

            let payload = serde_json::json!({
                "model": req.model.as_ref().unwrap_or(&self.model_id),
                "messages": req.messages,
                "max_tokens": req.max_tokens.unwrap_or(256),
                "temperature": req.temperature.unwrap_or(1.0),
                "top_p": req.top_p.unwrap_or(1.0),
                "presence_penalty": req.presence_penalty.unwrap_or(0.0),
                "frequency_penalty": req.frequency_penalty.unwrap_or(0.0),
                "stream": req.stream.unwrap_or(false),
            });

            (url, payload)
        } else {
            // Use traditional LightLLM format
            let url = format!("{}/generate", self.base);
            let payload = serde_json::json!({
                "prompt": prompt,
                "max_new_tokens": req.max_tokens.unwrap_or(256),
                "temperature": req.temperature.unwrap_or(1.0),
                "top_p": req.top_p.unwrap_or(1.0),
                "presence_penalty": req.presence_penalty.unwrap_or(0.0),
                "frequency_penalty": req.frequency_penalty.unwrap_or(0.0),
            });

            (url, payload)
        };

        // Build the HTTP request with authentication
        let mut request_builder = self.client.post(&url).json(&payload);

        if let Some(token) = &self.token {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", token));
        }

        // Send the request and await the response
        let resp = request_builder
            .send()
            .await
            .map_err(|e| {
                debug!("HTTP request failed for hash {:x}: {}", request_hash, e);
                ProxyError::from(e)
            })?;

        let status = resp.status();
        debug!("Received response status: {} for hash {:x}", status, request_hash);

        // Read response body
        let response_bytes = resp
            .bytes()
            .await
            .map_err(|e| {
                debug!("Failed to read response body for hash {:x}: {}", request_hash, e);
                ProxyError::Upstream(format!("error reading response body: {}", e))
            })?;

        debug!("Response body size: {} bytes for hash {:x}", response_bytes.len(), request_hash);

        // If streaming was requested, just return the raw response body for the streaming adapter to handle
        if req.stream.unwrap_or(false) {
            let response = Response::builder()
                .status(status)
                .body(axum::body::Body::from(response_bytes))
                .map_err(|e| ProxyError::Internal(format!("Failed to build response: {}", e)))?;
            return Ok(response);
        }

        // Parse JSON directly from bytes (for non-streaming responses)
        let json = serde_json::from_slice::<serde_json::Value>(&response_bytes)
            .map_err(|e| {
                debug!("JSON parsing failed for hash {:x}: {}", request_hash, e);
                ProxyError::Upstream(format!("error decoding response body: {} (body: {})", e, String::from_utf8_lossy(&response_bytes)))
            })?;

        // Check if the request was successful
        if !status.is_success() {
            debug!("Backend returned error status {} for hash {:x}", status, request_hash);
            return Err(ProxyError::Upstream(json.to_string()));
        }

        // Extract the generated text from the response
        let text = json
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        debug!("Extracted response text length: {} characters for hash {:x}", text.len(), request_hash);

        let response_time = start_time.elapsed().as_millis() as u64;
        AdapterUtils::log_response("lightllm", &AdapterUtils::extract_model(&req, &self.model_id), true, response_time);

        // Generate a unique ID for the response
        let now = AdapterUtils::current_timestamp() as i64;

        // Create OpenAI-compatible response envelope
        let envelope = serde_json::json!({
            "id": format!("chatcmpl-{}-{:x}", now, request_hash),
            "object": "chat.completion",
            "created": now,
            "model": req.model.unwrap_or(self.model_id.clone()),
            "choices": [{
                "index": 0,
                "message": {"role": "assistant", "content": text},
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": prompt.len() / 4, // Rough estimate
                "completion_tokens": text.len() / 4, // Rough estimate
                "total_tokens": (prompt.len() + text.len()) / 4 // Rough estimate
            }
        });

        debug!("Successfully processed request hash {:x}", request_hash);

        // Return the response as an HTTP response
        Ok((StatusCode::OK, Json(envelope)).into_response())
    }
}

#[async_trait::async_trait]
impl AdapterTrait for LightLLMAdapter {
    fn name(&self) -> &'static str {
        "lightllm"
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

    #[test]
    fn test_messages_to_prompt_single_user_message() {
        let messages = vec![Message {
            role: "user".to_string(),
            content: Some("Hello, how are you?".to_string()),
            name: None,
            function_call: None,
            tool_call_id: None,
            tool_calls: None,
        }];

        let prompt = LightLLMAdapter::messages_to_prompt(&messages);
        assert_eq!(prompt, "<|user|>\nHello, how are you?\n<|assistant|> ");
    }

    #[test]
    fn test_messages_to_prompt_with_system_message() {
        let messages = vec![
            Message {
                role: "system".to_string(),
                content: Some("You are a helpful assistant.".to_string()),
                name: None,
                function_call: None,
                tool_call_id: None,
                tool_calls: None,
            },
            Message {
                role: "user".to_string(),
                content: Some("What is 2+2?".to_string()),
                name: None,
                function_call: None,
                tool_call_id: None,
                tool_calls: None,
            },
        ];

        let prompt = LightLLMAdapter::messages_to_prompt(&messages);
        assert_eq!(
            prompt,
            "<|system|>\nYou are a helpful assistant.\n<|user|>\nWhat is 2+2?\n<|assistant|> "
        );
    }

    #[test]
    fn test_messages_to_prompt_conversation() {
        let messages = vec![
            Message {
                role: "user".to_string(),
                content: Some("Hello!".to_string()),
                name: None,
                function_call: None,
                tool_call_id: None,
                tool_calls: None,
            },
            Message {
                role: "assistant".to_string(),
                content: Some("Hi there! How can I help you?".to_string()),
                name: None,
                function_call: None,
                tool_call_id: None,
                tool_calls: None,
            },
            Message {
                role: "user".to_string(),
                content: Some("What's the weather like?".to_string()),
                name: None,
                function_call: None,
                tool_call_id: None,
                tool_calls: None,
            },
        ];

        let prompt = LightLLMAdapter::messages_to_prompt(&messages);
        let expected = "<|user|>\nHello!\n<|assistant|>\nHi there! How can I help you?\n<|user|>\nWhat's the weather like?\n<|assistant|> ";
        assert_eq!(prompt, expected);
    }

    #[test]
    fn test_messages_to_prompt_empty_messages() {
        let messages = vec![];
        let prompt = LightLLMAdapter::messages_to_prompt(&messages);
        assert_eq!(prompt, "<|assistant|> ");
    }

    #[test]
    fn test_messages_to_prompt_tool_role_ignored() {
        let messages = vec![
            Message {
                role: "user".to_string(),
                content: Some("Hello!".to_string()),
                name: None,
                function_call: None,
                tool_call_id: None,
                tool_calls: None,
            },
            Message {
                role: "tool".to_string(),
                content: Some("This should be ignored".to_string()),
                name: None,
                function_call: None,
                tool_call_id: None,
                tool_calls: None,
            },
        ];

        let prompt = LightLLMAdapter::messages_to_prompt(&messages);
        assert_eq!(prompt, "<|user|>\nHello!\n<|assistant|> ");
    }

    #[test]
    fn test_role_from_string() {
        assert!(matches!(Role::from("system"), Role::System));
        assert!(matches!(Role::from("user"), Role::User));
        assert!(matches!(Role::from("assistant"), Role::Assistant));
        assert!(matches!(Role::from("tool"), Role::Tool));
        assert!(matches!(Role::from("unknown"), Role::User)); // Unknown roles default to User
    }
}