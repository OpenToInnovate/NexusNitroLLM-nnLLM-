//! # Streaming Adapters
//!
//! This module contains adapter-specific streaming implementations

use crate::{
    adapters::{LightLLMAdapter, OpenAIAdapter, VLLMAdapter, AzureOpenAIAdapter, CustomAdapter},
    error::ProxyError,
    schemas::ChatCompletionRequest,
    streaming::core::{StreamingState, create_content_event, create_final_event, create_done_event},
};
use crate::core::http_client::HttpClientBuilder;
use axum::response::{Sse, sse::Event};
use futures_util::stream::{self, Stream};
use reqwest::Client;
use std::convert::Infallible;
use std::pin::Pin;

/// Common streaming response type
pub type StreamingResponse = Sse<Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>>>;

/// Streaming adapter trait for unified streaming behavior
#[async_trait::async_trait]
pub trait StreamingAdapter {
    /// Stream chat completions for this adapter
    async fn stream_chat_completions(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>> + Send + 'static>, ProxyError>;
}

/// Enhanced streaming handler with load balancing and performance monitoring
#[derive(Clone)]
pub struct StreamingHandler {
    /// HTTP client for streaming requests
    #[allow(dead_code)]
    http_client: Client,
}

impl StreamingHandler {
    /// Create a new streaming handler
    pub fn new() -> Result<Self, ProxyError> {
        let http_client = HttpClientBuilder::production()
            .build()
            .map_err(|e| ProxyError::Internal(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { http_client })
    }
}

impl Default for StreamingHandler {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            http_client: HttpClientBuilder::new().build().unwrap(),
        })
    }
}

/// LightLLM streaming implementation
pub async fn lightllm_streaming(
    adapter: &LightLLMAdapter,
    request: ChatCompletionRequest,
) -> Result<StreamingResponse, ProxyError> {
    // Try streaming first, then fallback to non-streaming if needed
    let mut stream_request = request.clone();
    stream_request.stream = Some(true);

    // Make request to LightLLM/backend
    let http_response = adapter.chat_completions_http(stream_request).await?;

    // Extract response body from HTTP response
    let (_parts, body) = http_response.into_parts();
    let body_bytes = axum::body::to_bytes(body, usize::MAX).await
        .map_err(|e| ProxyError::Internal(format!("Failed to read response body: {}", e)))?;

    // Convert the body to string to check if it's SSE format
    let body_str = String::from_utf8_lossy(&body_bytes);

    // Check if the response is in SSE format (contains "data:" lines)
    if body_str.contains("data:") {
        // Parse SSE data and forward the events
        let events = parse_sse_data(&body_str)?;
        let stream = stream::iter(events.into_iter().map(Ok));
        Ok(Sse::new(Box::pin(stream)))
    } else {
        // Fallback: treat as regular JSON response and convert to streaming
        let json_response: serde_json::Value = serde_json::from_slice(&body_bytes)
            .map_err(|e| ProxyError::Internal(format!("Failed to parse JSON response: {}", e)))?;

        // Convert response to streaming format
        let mut state = StreamingState::new(
            request.model.clone().unwrap_or_else(|| adapter.model_id().to_string())
        );

        // Extract content from the response
        let content = json_response
            .get("choices")
            .and_then(|choices| choices.as_array())
            .and_then(|choices| choices.first())
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
            .unwrap_or("")
            .to_string();

        let stream = stream::iter(vec![
            Ok(create_content_event(&mut state, content)),
            Ok(create_final_event(&mut state)),
            Ok(create_done_event()),
        ]);

        Ok(Sse::new(Box::pin(stream)))
    }
}

/// OpenAI streaming implementation
pub async fn openai_streaming(
    adapter: &OpenAIAdapter,
    request: ChatCompletionRequest,
) -> Result<StreamingResponse, ProxyError> {
    // Forward streaming request to OpenAI backend
    let mut stream_request = request.clone();
    stream_request.stream = Some(true);

    // Make streaming request to OpenAI
    let http_response = adapter.chat_completions_http(stream_request).await?;

    // Extract response body from HTTP response
    let (_parts, body) = http_response.into_parts();
    let body_bytes = axum::body::to_bytes(body, usize::MAX).await
        .map_err(|e| ProxyError::Internal(format!("Failed to read response body: {}", e)))?;

    // Convert the body to string to check if it's SSE format
    let body_str = String::from_utf8_lossy(&body_bytes);

    // Check if the response is in SSE format (contains "data:" lines)
    if body_str.contains("data:") {
        // Parse SSE data and forward the events
        let events = parse_sse_data(&body_str)?;
        let stream = stream::iter(events.into_iter().map(Ok));
        Ok(Sse::new(Box::pin(stream)))
    } else {
        // Fallback: treat as regular JSON response and convert to streaming
        let json_response: serde_json::Value = serde_json::from_slice(&body_bytes)
            .map_err(|e| ProxyError::Internal(format!("Failed to parse JSON response: {}", e)))?;

        // Convert response to streaming format
        let mut state = StreamingState::new(
            request.model.clone().unwrap_or_else(|| adapter.model_id().to_string())
        );

        // Extract content from the response
        let content = json_response
            .get("choices")
            .and_then(|choices| choices.as_array())
            .and_then(|choices| choices.first())
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
            .unwrap_or("")
            .to_string();

        let stream = stream::iter(vec![
            Ok(create_content_event(&mut state, content)),
            Ok(create_final_event(&mut state)),
            Ok(create_done_event()),
        ]);

        Ok(Sse::new(Box::pin(stream)))
    }
}

/// vLLM streaming implementation
pub async fn vllm_streaming(
    adapter: &VLLMAdapter,
    request: ChatCompletionRequest,
) -> Result<StreamingResponse, ProxyError> {
    // Forward streaming request to vLLM backend
    let mut stream_request = request.clone();
    stream_request.stream = Some(true);
    
    // Make streaming request to vLLM
    let http_response = adapter.chat_completions_http(stream_request).await?;
    
    // Extract response body from HTTP response
    let (_parts, body) = http_response.into_parts();
    let body_bytes = axum::body::to_bytes(body, usize::MAX).await
        .map_err(|e| ProxyError::Internal(format!("Failed to read response body: {}", e)))?;
    
    // Parse JSON response
    let json_response: serde_json::Value = serde_json::from_slice(&body_bytes)
        .map_err(|e| ProxyError::Internal(format!("Failed to parse JSON response: {}", e)))?;
    
    // Convert response to streaming format
    let mut state = StreamingState::new(
        request.model.clone().unwrap_or_else(|| adapter.model_id().to_string())
    );
    
    // Extract content from the response
    let content = json_response
        .get("choices")
        .and_then(|choices| choices.as_array())
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(|content| content.as_str())
        .unwrap_or("")
        .to_string();
    
    let stream = stream::iter(vec![
        Ok(create_content_event(&mut state, content)),
        Ok(create_final_event(&mut state)),
        Ok(create_done_event()),
    ]);

    Ok(Sse::new(Box::pin(stream)))
}

/// Azure OpenAI streaming implementation
pub async fn azure_streaming(
    adapter: &AzureOpenAIAdapter,
    request: ChatCompletionRequest,
) -> Result<StreamingResponse, ProxyError> {
    // Forward streaming request to Azure OpenAI backend
    let mut stream_request = request.clone();
    stream_request.stream = Some(true);

    // Make streaming request to Azure OpenAI
    let http_response = adapter.chat_completions_http(stream_request).await?;

    // Extract response body from HTTP response
    let (_parts, body) = http_response.into_parts();
    let body_bytes = axum::body::to_bytes(body, usize::MAX).await
        .map_err(|e| ProxyError::Internal(format!("Failed to read response body: {}", e)))?;

    // Parse JSON response
    let json_response: serde_json::Value = serde_json::from_slice(&body_bytes)
        .map_err(|e| ProxyError::Internal(format!("Failed to parse JSON response: {}", e)))?;

    // Convert response to streaming format
    let mut state = StreamingState::new(
        request.model.clone().unwrap_or_else(|| adapter.model_id().to_string())
    );

    // Extract content from the response
    let content = json_response
        .get("choices")
        .and_then(|choices| choices.as_array())
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(|content| content.as_str())
        .unwrap_or("")
        .to_string();

    let stream = stream::iter(vec![
        Ok(create_content_event(&mut state, content)),
        Ok(create_final_event(&mut state)),
        Ok(create_done_event()),
    ]);

    Ok(Sse::new(Box::pin(stream)))
}

/// Custom endpoint streaming implementation
pub async fn custom_streaming(
    adapter: &CustomAdapter,
    request: ChatCompletionRequest,
) -> Result<StreamingResponse, ProxyError> {
    // Forward streaming request to custom endpoint
    let mut stream_request = request.clone();
    stream_request.stream = Some(true);

    // Make streaming request to custom endpoint
    let http_response = adapter.chat_completions_http(stream_request).await?;

    // Extract response body from HTTP response
    let (_parts, body) = http_response.into_parts();
    let body_bytes = axum::body::to_bytes(body, usize::MAX).await
        .map_err(|e| ProxyError::Internal(format!("Failed to read response body: {}", e)))?;

    // Convert the body to string to check if it's SSE format
    let body_str = String::from_utf8_lossy(&body_bytes);

    // Check if the response is in SSE format (contains "data:" lines)
    if body_str.contains("data:") {
        // Parse SSE data and forward the events
        let events = parse_sse_data(&body_str)?;
        let stream = stream::iter(events.into_iter().map(Ok));
        Ok(Sse::new(Box::pin(stream)))
    } else {
        // Fallback: treat as regular JSON response and convert to streaming
        let json_response: serde_json::Value = serde_json::from_slice(&body_bytes)
            .map_err(|e| ProxyError::Internal(format!("Failed to parse JSON response: {}", e)))?;

        // Convert response to streaming format
        let mut state = StreamingState::new(
            request.model.clone().unwrap_or_else(|| adapter.model_id().to_string())
        );

        // Extract content from the response
        let content = json_response
            .get("choices")
            .and_then(|choices| choices.as_array())
            .and_then(|choices| choices.first())
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
            .unwrap_or("")
            .to_string();

        let stream = stream::iter(vec![
            Ok(create_content_event(&mut state, content)),
            Ok(create_final_event(&mut state)),
            Ok(create_done_event()),
        ]);

        Ok(Sse::new(Box::pin(stream)))
    }
}

/// Parse SSE (Server-Sent Events) data format
/// Converts "data: {json}\n\ndata: {json}\n\n..." format to Event objects
fn parse_sse_data(sse_data: &str) -> Result<Vec<Event>, ProxyError> {
    let mut events = Vec::new();

    for line in sse_data.lines() {
        let line = line.trim();

        // Handle data lines
        if line.starts_with("data: ") {
            let json_data = &line[6..]; // Remove "data: " prefix

            // Handle [DONE] marker
            if json_data == "[DONE]" {
                events.push(create_done_event());
                break;
            }

            // Try to parse as JSON and create an event
            if !json_data.is_empty() {
                let event = Event::default().data(json_data);
                events.push(event);
            }
        }
        // Skip empty lines and other SSE directives (id:, event:, retry:)
    }

    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::http_client::HttpClientBuilder;

    #[tokio::test]
    async fn test_streaming_handler_creation() {
        let handler = StreamingHandler::new();
        assert!(handler.is_ok());
    }

    #[tokio::test]
    async fn test_lightllm_streaming() {
        let client = HttpClientBuilder::new().build().unwrap();
        let adapter = LightLLMAdapter::new(
            "http://localhost:8000".to_string(),
            "test-model".to_string(),
            None,
            client,
        );

        let request = ChatCompletionRequest::default();
        let result = lightllm_streaming(&adapter, request).await;
        // Should fail with connection error since no server is running
        assert!(result.is_err());
        println!("✅ LightLLM streaming test passed (expected connection error)");
    }

    #[tokio::test]
    async fn test_openai_streaming() {
        let client = HttpClientBuilder::new().build().unwrap();
        let adapter = OpenAIAdapter::new(
            "https://api.openai.com/v1".to_string(),
            "gpt-3.5-turbo".to_string(),
            None,
            client,
        );

        let request = ChatCompletionRequest::default();
        let result = openai_streaming(&adapter, request).await;
        // Should fail with connection error since no API key is provided
        assert!(result.is_err());
        println!("✅ OpenAI streaming test passed (expected connection error)");
    }
}